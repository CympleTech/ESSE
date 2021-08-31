use std::collections::HashMap;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::SendType,
    primitive::{HandleResult, PeerAddr},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};
use tdn_did::user::User;

use crate::event::InnerEvent;
use crate::migrate::consensus::{FRIEND_TABLE_PATH, MESSAGE_TABLE_PATH, REQUEST_TABLE_PATH};
use crate::rpc::{session_create, session_last, sleep_waiting_close_stable, RpcState};
use crate::session::{Session, SessionType};
use crate::storage::{chat_db, delete_avatar, session_db};

use super::layer::LayerEvent;
use super::{Friend, Message, MessageType, Request};

#[inline]
pub(crate) fn friend_info(mgid: GroupId, friend: &Friend) -> RpcParam {
    rpc_response(0, "chat-friend-info", json!(friend.to_rpc()), mgid)
}

#[inline]
pub(crate) fn friend_update(mgid: GroupId, fid: i64, remark: &str) -> RpcParam {
    rpc_response(0, "chat-friend-update", json!([fid, remark]), mgid)
}

#[inline]
pub(crate) fn friend_close(mgid: GroupId, fid: i64) -> RpcParam {
    rpc_response(0, "chat-friend-close", json!([fid]), mgid)
}

#[inline]
pub(crate) fn friend_delete(mgid: GroupId, fid: i64) -> RpcParam {
    rpc_response(0, "chat-friend-delete", json!([fid]), mgid)
}

#[inline]
pub(crate) fn request_create(mgid: GroupId, req: &Request) -> RpcParam {
    rpc_response(0, "chat-request-create", json!(req.to_rpc()), mgid)
}

#[inline]
pub(crate) fn request_delivery(mgid: GroupId, id: i64, is_d: bool) -> RpcParam {
    rpc_response(0, "chat-request-delivery", json!([id, is_d]), mgid)
}

#[inline]
pub(crate) fn request_agree(mgid: GroupId, id: i64, friend: &Friend) -> RpcParam {
    rpc_response(0, "chat-request-agree", json!([id, friend.to_rpc()]), mgid)
}

#[inline]
pub(crate) fn request_reject(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "chat-request-reject", json!([id]), mgid)
}

#[inline]
pub(crate) fn request_delete(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "chat-request-delete", json!([id]), mgid)
}

#[inline]
pub(crate) fn message_create(mgid: GroupId, msg: &Message) -> RpcParam {
    rpc_response(0, "chat-message-create", json!(msg.to_rpc()), mgid)
}

#[inline]
pub(crate) fn message_delivery(mgid: GroupId, id: i64, is_d: bool) -> RpcParam {
    rpc_response(0, "chat-message-delivery", json!([id, is_d]), mgid)
}

#[inline]
pub(crate) fn message_delete(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "chat-message-delete", json!([id]), mgid)
}

#[inline]
fn friend_list(friends: Vec<Friend>) -> RpcParam {
    let mut results = vec![];
    for friend in friends {
        results.push(friend.to_rpc());
    }

    json!(results)
}

#[inline]
fn request_list(requests: Vec<Request>) -> RpcParam {
    let mut results = vec![];
    for request in requests {
        results.push(request.to_rpc());
    }
    json!(results)
}

#[inline]
fn message_list(messages: Vec<Message>) -> RpcParam {
    let mut results = vec![];
    for msg in messages {
        results.push(msg.to_rpc());
    }
    json!(results)
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("chat-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "chat-friend-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let layer_lock = state.layer.read().await;
            let db = chat_db(&layer_lock.base, &gid)?;
            Ok(HandleResult::rpc(friend_list(Friend::all(&db)?)))
        },
    );

    handler.add_method(
        "chat-friend-update",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let remark = params[1].as_str().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let db = chat_db(state.layer.read().await.base(), &gid)?;
            let f = if let Some(mut f) = Friend::get_id(&db, id)? {
                f.remark = remark.to_owned();
                f.me_update(&db)?;
                f
            } else {
                return Ok(results);
            };
            drop(db);
            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionFriendUpdate(f.gid, f.remark),
                FRIEND_TABLE_PATH,
                f.id,
                &mut results,
            )?;
            Ok(results)
        },
    );

    handler.add_method(
        "chat-friend-close",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let mut layer_lock = state.layer.write().await;

            let db = chat_db(layer_lock.base(), &gid)?;
            let friend = Friend::get_id(&db, id)?.ok_or(RpcError::ParseError)?;
            friend.close(&db)?;
            drop(db);

            let online = layer_lock.remove_online(&gid, &friend.gid);
            drop(layer_lock);

            if let Some(faddr) = online {
                let mut addrs: HashMap<PeerAddr, GroupId> = HashMap::new();
                addrs.insert(faddr, friend.gid);
                let sender = state.group.read().await.sender();
                tokio::spawn(sleep_waiting_close_stable(sender, HashMap::new(), addrs));
            }

            let data = postcard::to_allocvec(&LayerEvent::Close).unwrap_or(vec![]);
            let msg = SendType::Event(0, friend.addr, data);
            results.layers.push((gid, friend.gid, msg));

            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionFriendClose(friend.gid),
                FRIEND_TABLE_PATH,
                friend.id,
                &mut results,
            )?;

            Ok(results)
        },
    );

    handler.add_method(
        "chat-friend-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let mut layer_lock = state.layer.write().await;

            let db = chat_db(layer_lock.base(), &gid)?;
            let friend = Friend::get_id(&db, id)?.ok_or(RpcError::ParseError)?;
            friend.delete(&db)?;
            drop(db);

            let online = layer_lock.remove_online(&gid, &friend.gid);
            delete_avatar(layer_lock.base(), &gid, &friend.gid).await?;
            drop(layer_lock);

            if let Some(faddr) = online {
                let mut addrs: HashMap<PeerAddr, GroupId> = HashMap::new();
                addrs.insert(faddr, friend.gid);
                let sender = state.group.read().await.sender();
                tokio::spawn(sleep_waiting_close_stable(sender, HashMap::new(), addrs));
            }

            let data = postcard::to_allocvec(&LayerEvent::Close).unwrap_or(vec![]);
            let msg = SendType::Event(0, friend.addr, data);
            results.layers.push((gid, friend.gid, msg));

            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionFriendDelete(friend.gid),
                FRIEND_TABLE_PATH,
                friend.id,
                &mut results,
            )?;

            Ok(results)
        },
    );

    handler.add_method(
        "chat-request-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let layer_lock = state.layer.read().await;
            let db = chat_db(layer_lock.base(), &gid)?;
            drop(layer_lock);
            let requests = Request::all(&db)?;
            drop(db);
            Ok(HandleResult::rpc(request_list(requests)))
        },
    );

    handler.add_method(
        "chat-request-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let remote_gid = GroupId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let remote_addr = PeerAddr::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let remote_name = params[2].as_str().ok_or(RpcError::ParseError)?.to_string();
            let remark = params[3].as_str().ok_or(RpcError::ParseError)?.to_string();

            let mut request = Request::new(
                remote_gid,
                remote_addr,
                remote_name.clone(),
                remark.clone(),
                true,
                false,
            );

            let me = state.group.read().await.clone_user(&gid)?;

            let mut layer_lock = state.layer.write().await;
            let db = chat_db(layer_lock.base(), &gid)?;
            if Friend::is_friend(&db, &request.gid)? {
                debug!("had friend.");
                drop(layer_lock);
                return Ok(HandleResult::new());
            }

            if let Some(req) = Request::get(&db, &request.gid)? {
                debug!("Had this request.");
                req.delete(&db)?;
            }
            request.insert(&db)?;
            drop(db);

            let mut results = HandleResult::rpc(json!(request.to_rpc()));

            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionRequestCreate(
                    true,
                    User::new(remote_gid, remote_addr, remote_name, vec![])?,
                    remark,
                ),
                REQUEST_TABLE_PATH,
                request.id,
                &mut results,
            )?;

            results.layers.push((
                gid,
                remote_gid,
                super::layer::req_message(&mut layer_lock, me, request),
            ));

            drop(layer_lock);

            Ok(results)
        },
    );

    handler.add_method(
        "chat-request-agree",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let mut group_lock = state.group.write().await;
            let me = group_lock.clone_user(&gid)?;
            let db = chat_db(group_lock.base(), &gid)?;
            let mut results = HandleResult::new();

            if let Some(mut request) = Request::get_id(&db, id)? {
                group_lock.broadcast(
                    &gid,
                    InnerEvent::SessionRequestHandle(request.gid, true, vec![]),
                    REQUEST_TABLE_PATH,
                    request.id,
                    &mut results,
                )?;
                request.is_ok = true;
                request.is_over = true;
                request.update(&db)?;

                let friend = Friend::from_request(&db, request)?;
                results.rpcs.push(json!([id, friend.to_rpc()]));

                // ADD NEW SESSION.
                let s_db = session_db(group_lock.base(), &gid)?;
                let mut session = friend.to_session();
                session.insert(&s_db)?;
                results.rpcs.push(session_create(gid, &session));

                let proof = group_lock.prove_addr(&gid, &friend.addr)?;
                let msg = super::layer::agree_message(proof, me, friend.addr)?;
                results.layers.push((gid, friend.gid, msg));
            }

            Ok(results)
        },
    );

    handler.add_method(
        "chat-request-reject",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let mut layer_lock = state.layer.write().await;
            let db = chat_db(layer_lock.base(), &gid)?;
            let mut req = Request::get_id(&db, id)?.ok_or(RpcError::ParseError)?;
            req.is_ok = false;
            req.is_over = true;
            req.update(&db)?;
            drop(db);
            let msg = super::layer::reject_message(&mut layer_lock, id, req.addr, gid);
            drop(layer_lock);

            let mut results = HandleResult::layer(gid, req.gid, msg);
            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionRequestHandle(req.gid, false, vec![]),
                REQUEST_TABLE_PATH,
                req.id,
                &mut results,
            )?;
            Ok(results)
        },
    );

    handler.add_method(
        "chat-request-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let layer_lock = state.layer.read().await;
            let db = chat_db(layer_lock.base(), &gid)?;
            let base = layer_lock.base().clone();
            drop(layer_lock);
            let req = Request::get_id(&db, id)?.ok_or(RpcError::ParseError)?;
            req.delete(&db)?;

            // delete avatar. check had friend.
            if Friend::get(&db, &req.gid)?.is_none() {
                delete_avatar(&base, &gid, &req.gid).await?;
            }
            drop(db);

            let mut results = HandleResult::new();
            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionRequestDelete(req.gid),
                REQUEST_TABLE_PATH,
                req.id,
                &mut results,
            )?;
            Ok(results)
        },
    );

    handler.add_method(
        "chat-message-list",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let fid = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let layer_lock = state.layer.read().await;
            let db = chat_db(layer_lock.base(), &gid)?;
            drop(layer_lock);

            let messages = Message::get(&db, &fid)?;
            drop(db);
            Ok(HandleResult::rpc(message_list(messages)))
        },
    );

    handler.add_method(
        "chat-message-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let fid = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let fgid = GroupId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let m_type = MessageType::from_int(params[2].as_i64().ok_or(RpcError::ParseError)?);
            let content = params[3].as_str().ok_or(RpcError::ParseError)?.to_string();

            let mut layer_lock = state.layer.write().await;
            let base = layer_lock.base();
            let faddr = layer_lock.running(&gid)?.online(&fgid)?;

            let (msg, nw, scontent) =
                LayerEvent::from_message(base, gid, fid, m_type, content).await?;
            let event = LayerEvent::Message(msg.hash, nw);
            let s = super::layer::event_message(&mut layer_lock, msg.id, gid, faddr, &event);
            drop(layer_lock);

            let mut results = HandleResult::rpc(json!(msg.to_rpc()));
            results.layers.push((gid, fgid, s));

            // UPDATE SESSION.
            let layer_lock = state.layer.read().await;
            let s_db = session_db(&layer_lock.base, &gid)?;
            if let Ok(id) = Session::last(
                &s_db,
                &fid,
                &SessionType::Chat,
                &msg.datetime,
                &scontent,
                true,
            ) {
                results
                    .rpcs
                    .push(session_last(gid, &id, &msg.datetime, &scontent, true));
            }
            drop(layer_lock);

            match event {
                LayerEvent::Message(hash, nw) => {
                    state.group.write().await.broadcast(
                        &gid,
                        InnerEvent::SessionMessageCreate(fgid, true, hash, nw),
                        MESSAGE_TABLE_PATH,
                        msg.id,
                        &mut results,
                    )?;
                }
                _ => {}
            }

            Ok(results)
        },
    );

    handler.add_method(
        "chat-message-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let layer_lock = state.layer.read().await;
            let db = chat_db(&layer_lock.base(), &gid)?;
            drop(layer_lock);

            let msg = Message::get_id(&db, id)?.ok_or(RpcError::ParseError)?;
            msg.delete(&db)?;
            drop(db);
            let mut results = HandleResult::new();
            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionMessageDelete(msg.hash),
                MESSAGE_TABLE_PATH,
                msg.id,
                &mut results,
            )?;
            Ok(results)
        },
    );
}
