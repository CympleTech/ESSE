use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::{NetworkType, SendType},
    primitive::{HandleResult, PeerId},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};

use chat_types::MessageType;
use group_types::{Event, LayerEvent};

use crate::apps::chat::Friend;
use crate::layer::Online;
use crate::rpc::{session_create, session_delete, session_last, RpcState};
use crate::session::{Session, SessionType};
use crate::storage::{chat_db, group_db, session_db, write_avatar};

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
pub(crate) fn member_info(
    mgid: GroupId,
    id: i64,
    mid: i64,
    addr: PeerId,
    name: String,
) -> RpcParam {
    rpc_response(
        0,
        "group-member-info",
        json!([id, mid, addr.to_hex(), name]),
        mgid,
    )
}

#[inline]
pub(crate) fn member_online(mgid: GroupId, gid: i64, mid: GroupId, maddr: PeerId) -> RpcParam {
    rpc_response(
        0,
        "group-member-online",
        json!([gid, mid.to_hex(), maddr.to_hex()]),
        mgid,
    )
}

#[inline]
pub(crate) fn member_offline(mgid: GroupId, gid: i64, mid: GroupId) -> RpcParam {
    rpc_response(0, "group-member-offline", json!([gid, mid.to_hex()]), mgid)
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
            let members = Member::all(&db, &id)?;
            let messages = Message::all(&db, &id)?;
            Ok(HandleResult::rpc(detail_list(group, members, messages)))
        },
    );

    handler.add_method(
        "group-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let base = state.layer.read().await.base().clone();
            let db = group_db(&base, &gid)?;
            let addr = state.layer.read().await.addr.clone();

            let mut gc = GroupChat::new(addr, name);
            let gcd = gc.g_id;
            let gheight = gc.height;

            // save db
            gc.insert(&db)?;
            let gdid = gc.id;

            let mut results = HandleResult::new();
            let me = state.group.read().await.clone_user(&gid)?;

            // add to rpcs.
            results.rpcs.push(json!(gc.to_rpc()));

            let mut m = Member::new(gheight, gc.id, gid, me.addr, me.name);
            m.insert(&db)?;
            let mid = m.id;
            let _ = write_avatar(&base, &gid, &gid, &me.avatar).await;

            // Add new session.
            let s_db = session_db(state.layer.read().await.base(), &gid)?;
            let mut session = gc.to_session();
            session.insert(&s_db)?;
            results.rpcs.push(session_create(gid, &session));

            // Add frist member join.
            let mut layer_lock = state.layer.write().await;
            layer_lock.add_running(&gcd, gid, gdid, gheight)?;
            let height = layer_lock.running_mut(&gcd)?.increased();

            // Add online to layers.
            layer_lock
                .running_mut(&gcd)?
                .check_add_online(gid, Online::Direct(addr), gdid, mid)?;
            layer_lock.running_mut(&gid)?.check_add_online(
                gcd,
                Online::Direct(addr),
                session.id,
                gdid,
            )?;

            drop(layer_lock);

            // Update consensus.
            GroupChat::add_height(&db, gdid, height)?;

            // Online local group.
            results.networks.push(NetworkType::AddGroup(gcd));

            Ok(results)
        },
    );

    handler.add_method(
        "group-invite",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let gcd = GroupId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;

            let ids: Vec<i64> = params[2]
                .as_array()
                .ok_or(RpcError::ParseError)?
                .iter()
                .filter_map(|v| v.as_i64())
                .collect();

            let group_lock = state.group.read().await;
            let base = group_lock.base().clone();

            let group_db = group_db(&base, &gid)?;
            let chat = chat_db(&base, &gid)?;
            let gc = GroupChat::get(&group_db, &id)?;

            let mut invites = vec![];
            for fid in ids {
                let friend = Friend::get_id(&chat, fid)?.ok_or(RpcError::ParseError)?;
                if Member::get_id(&group_db, &id, &friend.gid).is_err() {
                    // TODO add friend to group chat.

                    invites.push((friend.id, friend.gid, friend.addr));
                }
            }
            drop(chat);

            let tmp_name = gc.g_name.replace(";", "-;");

            let s_db = session_db(&base, &gid)?;

            let mut results = HandleResult::new();
            let mut layer_lock = state.layer.write().await;
            for (fid, fgid, faddr) in invites {
                let contact_values =
                    format!("{};;{};;{}", gcd.to_hex(), gc.g_addr.to_hex(), tmp_name);

                let (msg, nw, sc) = crate::apps::chat::LayerEvent::from_message(
                    &base,
                    gid,
                    fid,
                    MessageType::Invite,
                    contact_values,
                )
                .await?;
                let event = crate::apps::chat::LayerEvent::Message(msg.hash, nw);
                let s =
                    crate::apps::chat::event_message(&mut layer_lock, msg.id, gid, faddr, &event);
                results.layers.push((gid, fgid, s));

                if let Ok(id) =
                    Session::last(&s_db, &fid, &SessionType::Chat, &msg.datetime, &sc, true)
                {
                    results
                        .rpcs
                        .push(session_last(gid, &id, &msg.datetime, &sc, false));
                }
            }
            Ok(results)
        },
    );

    handler.add_method(
        "group-message-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gcd = GroupId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let m_type = MessageType::from_int(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let m_content = params[2].as_str().ok_or(RpcError::ParseError)?;

            let addr = state.layer.read().await.running(&gid)?.online(&gcd)?;
            let mut results = HandleResult::new();

            let base = state.group.read().await.base().clone();
            let (nmsg, datetime) = to_network_message(&base, &gid, m_type, m_content).await?;
            let event = Event::MessageCreate(gid, nmsg, datetime);
            let data = bincode::serialize(&LayerEvent::Sync(gcd, 0, event))?;
            let msg = SendType::Event(0, addr, data);
            add_layer(&mut results, gid, msg);

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
                    println!("--- DEBUG broadcast to: {:?}", mgid);
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
