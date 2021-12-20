use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::{NetworkType, SendMessage, SendType},
    primitive::{HandleResult, PeerId},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};

use chat_types::MessageType;
use group_types::{Event, LayerEvent};

use crate::apps::chat::{Friend, InviteType};
use crate::layer::Online;
use crate::rpc::{session_create, session_delete, RpcState};
use crate::session::{Session, SessionType};
use crate::storage::{chat_db, group_db, read_avatar, session_db, write_avatar};

use super::layer::{broadcast, update_session};
use super::models::{to_network_message, GroupChat, Member, Message};
use super::{add_layer, add_server_layer};

#[inline]
pub(crate) fn member_join(mgid: GroupId, member: &Member) -> RpcParam {
    rpc_response(0, "group-member-join", json!(member.to_rpc()), mgid)
}

#[inline]
pub(crate) fn member_leave(mgid: GroupId, id: i64, mid: i64) -> RpcParam {
    rpc_response(0, "group-member-leave", json!([id, mid]), mgid)
}

#[inline]
pub(crate) fn member_online(mgid: GroupId, id: i64, mid: i64, maddr: &PeerId) -> RpcParam {
    rpc_response(
        0,
        "group-member-online",
        json!([id, mid, maddr.to_hex()]),
        mgid,
    )
}

#[inline]
pub(crate) fn member_offline(mgid: GroupId, gid: i64, mid: i64) -> RpcParam {
    rpc_response(0, "group-member-offline", json!([gid, mid]), mgid)
}

#[inline]
pub(crate) fn message_create(mgid: GroupId, msg: &Message) -> RpcParam {
    rpc_response(0, "group-message-create", json!(msg.to_rpc()), mgid)
}

#[inline]
fn group_list(groups: Vec<GroupChat>) -> RpcParam {
    let mut results = vec![];
    for group in groups {
        results.push(group.to_rpc());
    }

    json!(results)
}

#[inline]
fn detail_list(group: GroupChat, members: Vec<Member>, messages: Vec<Message>) -> RpcParam {
    let mut member_results = vec![];
    for m in members {
        member_results.push(m.to_rpc());
    }

    let mut message_results = vec![];
    for msg in messages {
        message_results.push(msg.to_rpc());
    }

    json!([group.to_rpc(), member_results, message_results])
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("group-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "group-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let layer_lock = state.layer.read().await;
            let db = group_db(&layer_lock.base, &gid)?;
            Ok(HandleResult::rpc(group_list(GroupChat::all(&db)?)))
        },
    );

    handler.add_method(
        "group-detail",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let db = group_db(state.layer.read().await.base(), &gid)?;
            let group = GroupChat::get(&db, &id)?;
            let members = Member::list(&db, &id)?;
            let messages = Message::list(&db, &id)?;
            Ok(HandleResult::rpc(detail_list(group, members, messages)))
        },
    );

    handler.add_method(
        "group-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let group_lock = state.group.read().await;
            let base = group_lock.base().clone();
            let addr = group_lock.addr().clone();
            let sender = group_lock.sender();
            let me = group_lock.clone_user(&gid)?;
            drop(group_lock);
            let db = group_db(&base, &gid)?;

            let mut gc = GroupChat::new(addr, name);
            let gcd = gc.g_id;
            let gheight = gc.height + 1; // add first member.

            // save db
            gc.insert(&db)?;
            let gdid = gc.id;

            let mut results = HandleResult::new();

            // add to rpcs.
            results.rpcs.push(json!(gc.to_rpc()));

            let mut m = Member::new(gheight, gc.id, gid, me.addr, me.name);
            m.insert(&db)?;
            let mid = m.id;
            let _ = write_avatar(&base, &gid, &gid, &me.avatar).await;

            // Add new session.
            let s_db = session_db(&base, &gid)?;
            let mut session = gc.to_session();
            session.insert(&s_db)?;
            let sid = session.id;
            tokio::spawn(async move {
                let _ = sender
                    .send(SendMessage::Rpc(0, session_create(gid, &session), true))
                    .await;
            });

            // Add frist member join.
            let mut layer_lock = state.layer.write().await;
            layer_lock.add_running(&gcd, gid, gdid, gheight)?;

            // Add online to layers.
            layer_lock
                .running_mut(&gcd)?
                .check_add_online(gid, Online::Direct(addr), gdid, mid)?;
            layer_lock
                .running_mut(&gid)?
                .check_add_online(gcd, Online::Direct(addr), sid, gdid)?;

            drop(layer_lock);

            // Update consensus.
            GroupChat::add_height(&db, gdid, gheight)?;

            // Online local group.
            results.networks.push(NetworkType::AddGroup(gcd));

            Ok(results)
        },
    );

    handler.add_method(
        "group-member-join",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let fid = params[1].as_i64().ok_or(RpcError::ParseError)?;

            let base = state.layer.read().await.base().clone();
            let chat_db = chat_db(&base, &gid)?;
            let f = Friend::get(&chat_db, &fid)?;
            let group_db = group_db(&base, &gid)?;
            let g = GroupChat::get(&group_db, &id)?;
            let gcd = g.g_id;
            let mut results = HandleResult::new();

            // handle invite message
            let contact_values = InviteType::Group(gcd, g.g_addr, g.g_name).serialize();
            let (msg, nw) = crate::apps::chat::LayerEvent::from_message(
                &base,
                gid,
                fid,
                MessageType::Invite,
                &contact_values,
            )
            .await?;
            let event = crate::apps::chat::LayerEvent::Message(msg.hash, nw);
            let mut layer_lock = state.layer.write().await;
            let s = crate::apps::chat::event_message(&mut layer_lock, msg.id, gid, f.addr, &event);
            drop(layer_lock);
            results.layers.push((gid, f.gid, s));
            crate::apps::chat::update_session(&base, &gid, &id, &msg, &mut results);

            // handle group member
            let avatar = read_avatar(&base, &gid, &f.gid).await.unwrap_or(vec![]);
            let event = Event::MemberJoin(f.gid, f.addr, f.name.clone(), avatar);

            if g.local {
                // local save.
                let new_h = state.layer.write().await.running_mut(&gcd)?.increased();

                let mut mem = Member::new(new_h, g.id, f.gid, f.addr, f.name);
                mem.insert(&group_db)?;
                results.rpcs.push(mem.to_rpc());
                GroupChat::add_height(&group_db, id, new_h)?;

                // broadcast.
                broadcast(
                    &LayerEvent::Sync(gcd, new_h, event),
                    &state.layer,
                    &gcd,
                    &mut results,
                )
                .await?;
            } else {
                // send to server.
                let data = bincode::serialize(&LayerEvent::Sync(gcd, 0, event))?;
                let msg = SendType::Event(0, g.g_addr, data);
                add_layer(&mut results, gid, msg);
            }

            Ok(results)
        },
    );

    handler.add_method(
        "group-message-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let m_type = MessageType::from_int(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let m_content = params[2].as_str().ok_or(RpcError::ParseError)?;

            let base = state.layer.read().await.base().clone();
            let db = group_db(&base, &gid)?;
            let group = GroupChat::get(&db, &id)?;
            let gcd = group.g_id;
            let mid = Member::get_id(&db, &id, &gid)?;

            let mut results = HandleResult::new();
            let (nmsg, datetime, raw) = to_network_message(&base, &gid, m_type, m_content).await?;
            let event = Event::MessageCreate(gid, nmsg, datetime);

            if group.local {
                // local save.
                let new_h = state.layer.write().await.running_mut(&gcd)?.increased();

                let mut msg = Message::new_with_time(new_h, id, mid, true, m_type, raw, datetime);
                msg.insert(&db)?;
                results.rpcs.push(msg.to_rpc());
                GroupChat::add_height(&db, id, new_h)?;

                // UPDATE SESSION.
                update_session(&base, &gid, &id, &msg, &mut results);

                // broadcast.
                broadcast(
                    &LayerEvent::Sync(gcd, new_h, event),
                    &state.layer,
                    &gcd,
                    &mut results,
                )
                .await?;
            } else {
                // send to server.
                let data = bincode::serialize(&LayerEvent::Sync(gcd, 0, event))?;
                let msg = SendType::Event(0, group.g_addr, data);
                add_layer(&mut results, gid, msg);
            }

            Ok(results)
        },
    );

    handler.add_method(
        "group-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gcd = GroupId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let id = params[1].as_i64().ok_or(RpcError::ParseError)?;

            let addr = state
                .layer
                .write()
                .await
                .remove_online(&gid, &gcd)
                .ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let base = state.layer.read().await.base().clone();
            let db = group_db(&base, &gid)?;
            let group = GroupChat::delete(&db, &id)?;

            let sid = Session::delete(&session_db(&base, &gid)?, &id, &SessionType::Group)?;
            results.rpcs.push(session_delete(gid, &sid));

            if group.g_addr == addr {
                // dissolve group.
                let data = bincode::serialize(&LayerEvent::Sync(gcd, 0, Event::GroupClose))?;
                for (mgid, maddr) in state.layer.read().await.running(&gcd)?.onlines() {
                    let s = SendType::Event(0, *maddr, data.clone());
                    add_server_layer(&mut results, *mgid, s);
                }
            } else {
                // leave group.
                let data = bincode::serialize(&LayerEvent::Sync(gcd, 0, Event::MemberLeave(gid)))?;
                let msg = SendType::Event(0, addr, data);
                add_layer(&mut results, gid, msg);
            }

            Ok(results)
        },
    );
}
