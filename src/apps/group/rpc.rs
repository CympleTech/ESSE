use esse_primitives::MessageType;
use group_types::{Event, LayerEvent, GROUP_CHAT_ID};
use std::sync::Arc;
use tdn::types::{
    message::{RpcSendMessage, SendType},
    primitives::HandleResult,
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};

use crate::global::Global;
use crate::group::{raw_to_network_message, Friend, InviteType};
use crate::rpc::{session_create, session_delete, session_update_name};
use crate::session::{Session, SessionType};
use crate::storage::{chat_db, group_db, read_avatar, session_db, write_avatar};

use super::layer::{broadcast, update_session};
use super::models::{to_network_message, GroupChat, Member, Message};

#[inline]
pub(crate) fn member_join(member: &Member) -> RpcParam {
    rpc_response(0, "group-member-join", json!(member.to_rpc()))
}

#[inline]
pub(crate) fn member_leave(id: i64, mid: i64) -> RpcParam {
    rpc_response(0, "group-member-leave", json!([id, mid]))
}

#[inline]
pub(crate) fn member_online(id: i64, mid: i64) -> RpcParam {
    rpc_response(0, "group-member-online", json!([id, mid]))
}

#[inline]
pub(crate) fn member_offline(id: i64, mid: i64) -> RpcParam {
    rpc_response(0, "group-member-offline", json!([id, mid]))
}

#[inline]
pub(crate) fn group_name(id: &i64, name: &str) -> RpcParam {
    rpc_response(0, "group-name", json!([id, name]))
}

#[inline]
pub(crate) fn message_create(msg: &Message) -> RpcParam {
    rpc_response(0, "group-message-create", json!(msg.to_rpc()))
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

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<Global>) {
    handler.add_method(
        "group-list",
        |_params: Vec<RpcParam>, state: Arc<Global>| async move {
            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = group_db(&state.base, &pid, &db_key)?;

            Ok(HandleResult::rpc(group_list(GroupChat::all(&db)?)))
        },
    );

    handler.add_method(
        "group-detail",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = group_db(&state.base, &pid, &db_key)?;
            let group = GroupChat::get(&db, &id)?;
            let members = Member::list(&db, &id)?;
            let messages = Message::list(&db, &id)?;
            Ok(HandleResult::rpc(detail_list(group, members, messages)))
        },
    );

    handler.add_method(
        "group-create",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let pid = state.pid().await;
            let own_lock = state.own.read().await;
            let db_key = own_lock.db_key(&pid)?;
            let me = own_lock.clone_user(&pid)?;
            drop(own_lock);

            let db = group_db(&state.base, &pid, &db_key)?;
            let s_db = session_db(&state.base, &pid, &db_key)?;

            let mut gc = GroupChat::new(pid, name);
            let gh = gc.height + 1; // add first member.

            // save db
            gc.insert(&db)?;
            let id = gc.id;
            let gid = gc.gid;

            let mut results = HandleResult::new();

            let mut m = Member::new(gh, id, pid, me.name);
            m.insert(&db)?;
            let _ = write_avatar(&state.base, &pid, &pid, &me.avatar).await;

            // Add new session.
            let mut session = gc.to_session();
            session.insert(&s_db)?;
            let sid = session.id;
            let sender = state.rpc_send.clone();
            tokio::spawn(async move {
                let _ = sender
                    .send(RpcSendMessage(0, session_create(&session), true))
                    .await;
            });

            // add to rpcs.
            results.rpcs.push(json!([sid, id]));

            // Add frist member join.
            state.layer.write().await.group_add(gid, pid, sid, id, gh);

            // Update consensus.
            GroupChat::add_height(&db, id, gh)?;

            Ok(results)
        },
    );

    handler.add_method(
        "group-member-join",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let fid = params[1].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let group_db = group_db(&state.base, &pid, &db_key)?;
            let chat_db = chat_db(&state.base, &pid, &db_key)?;
            let s_db = session_db(&state.base, &pid, &db_key)?;

            let f = Friend::get(&chat_db, &fid)?;
            let g = GroupChat::get(&group_db, &id)?;
            let gid = g.gid;
            let mut results = HandleResult::new();

            // handle invite message
            let contact = InviteType::Group(gid, g.addr, g.name).serialize();
            let m_type = MessageType::Invite;
            let (nm, raw) =
                raw_to_network_message(&pid, &state.base, &db_key, &m_type, &contact).await?;
            let mut msg = crate::group::Message::new(&pid, f.id, true, m_type, raw, false);
            msg.insert(&chat_db)?;
            let event = crate::group::GroupEvent::Message(msg.hash, nm);
            let tid = state.layer.write().await.delivery(msg.id);
            let data = bincode::serialize(&event).unwrap_or(vec![]);
            results.groups.push(SendType::Event(tid, f.pid, data));

            // update session.
            crate::group::update_session(&s_db, &id, &msg, &mut results);

            // handle group member
            let avatar = read_avatar(&state.base, &pid, &f.pid)
                .await
                .unwrap_or(vec![]);
            let event = Event::MemberJoin(f.pid, f.name.clone(), avatar);

            if g.local {
                // local save.
                let new_h = state.layer.write().await.group_mut(&gid)?.increased();

                let mut mem = Member::new(new_h, g.id, f.pid, f.name);
                mem.insert(&group_db)?;
                results.rpcs.push(mem.to_rpc());
                GroupChat::add_height(&group_db, id, new_h)?;

                // broadcast.
                let data = LayerEvent::Sync(gid, new_h, event);
                broadcast(&gid, &state, &data, &mut results).await?;
            } else {
                // send to server.
                let data = bincode::serialize(&LayerEvent::Sync(gid, 0, event))?;
                let msg = SendType::Event(0, g.addr, data);
                results.layers.push((GROUP_CHAT_ID, msg));
            }

            Ok(results)
        },
    );

    handler.add_method(
        "group-message-create",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let m_type = MessageType::from_int(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let m_content = params[2].as_str().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = group_db(&state.base, &pid, &db_key)?;
            let s_db = session_db(&state.base, &pid, &db_key)?;

            let group = GroupChat::get(&db, &id)?;
            let gid = group.gid;
            let mid = Member::get_id(&db, &id, &pid)?;

            let mut results = HandleResult::new();
            let (nmsg, datetime, raw) =
                to_network_message(&pid, &state.base, &db_key, m_type, m_content).await?;
            let event = Event::MessageCreate(pid, nmsg, datetime);

            if group.local {
                // local save.
                let new_h = state.layer.write().await.group_mut(&gid)?.increased();

                let mut msg = Message::new_with_time(new_h, id, mid, true, m_type, raw, datetime);
                msg.insert(&db)?;
                results.rpcs.push(msg.to_rpc());
                GroupChat::add_height(&db, id, new_h)?;

                // UPDATE SESSION.
                update_session(&s_db, &id, &msg, &mut results);

                // broadcast.
                let data = LayerEvent::Sync(gid, new_h, event);
                broadcast(&gid, &state, &data, &mut results).await?;
            } else {
                // send to server.
                let data = bincode::serialize(&LayerEvent::Sync(gid, 0, event))?;
                let msg = SendType::Event(0, group.addr, data);
                results.layers.push((GROUP_CHAT_ID, msg));
            }

            Ok(results)
        },
    );

    handler.add_method(
        "group-name",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let name = params[1].as_str().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = group_db(&state.base, &pid, &db_key)?;
            let s_db = session_db(&state.base, &pid, &db_key)?;

            let g = GroupChat::get(&db, &id)?;
            let data = LayerEvent::GroupName(g.gid, name.to_owned());

            if g.local {
                if let Ok(sid) = Session::update_name_by_id(&s_db, &id, &SessionType::Group, &name)
                {
                    results.rpcs.push(session_update_name(&sid, &name));
                }

                results.rpcs.push(json!([id, name]));
                broadcast(&g.gid, &state, &data, &mut results).await?;
            } else {
                let d = bincode::serialize(&data)?;
                let msg = SendType::Event(0, g.addr, d);
                results.layers.push((GROUP_CHAT_ID, msg));
            }

            Ok(results)
        },
    );

    handler.add_method(
        "group-delete",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = group_db(&state.base, &pid, &db_key)?;
            let s_db = session_db(&state.base, &pid, &db_key)?;

            let g = GroupChat::delete(&db, &id)?;
            let sid = Session::delete(&s_db, &id, &SessionType::Group)?;
            results.rpcs.push(session_delete(&sid));

            if g.local {
                // dissolve group.
                let data = bincode::serialize(&LayerEvent::GroupClose(g.gid))?;
                if let Some(addrs) = state.layer.write().await.group_del(&g.gid) {
                    for addr in addrs {
                        let s = SendType::Event(0, addr, data.clone());
                        results.layers.push((GROUP_CHAT_ID, s));
                    }
                }
            } else {
                // leave group.
                let d = bincode::serialize(&LayerEvent::Sync(g.gid, 0, Event::MemberLeave(pid)))?;
                let msg = SendType::Event(0, g.addr, d);
                results.layers.push((GROUP_CHAT_ID, msg));
            }

            Ok(results)
        },
    );
}
