use chat_types::{MessageType, CHAT_ID};
use esse_primitives::id_from_str;
use std::sync::Arc;
use tdn::types::{
    message::SendType,
    primitives::{HandleResult, PeerId},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};

//use crate::event::InnerEvent;
use crate::global::Global;
use crate::rpc::session_create;
use crate::storage::{chat_db, delete_avatar, session_db};

use super::layer::{update_session, LayerEvent};
use super::{raw_to_network_message, Friend, Message, Request};

#[inline]
pub(crate) fn friend_info(friend: &Friend) -> RpcParam {
    rpc_response(0, "chat-friend-info", json!(friend.to_rpc()))
}

#[inline]
pub(crate) fn friend_update(fid: i64, remark: &str) -> RpcParam {
    rpc_response(0, "chat-friend-update", json!([fid, remark]))
}

#[inline]
pub(crate) fn friend_close(fid: i64) -> RpcParam {
    rpc_response(0, "chat-friend-close", json!([fid]))
}

#[inline]
pub(crate) fn friend_delete(fid: i64) -> RpcParam {
    rpc_response(0, "chat-friend-delete", json!([fid]))
}

#[inline]
pub(crate) fn request_create(req: &Request) -> RpcParam {
    rpc_response(0, "chat-request-create", json!(req.to_rpc()))
}

#[inline]
pub(crate) fn request_delivery(id: i64, is_d: bool) -> RpcParam {
    rpc_response(0, "chat-request-delivery", json!([id, is_d]))
}

#[inline]
pub(crate) fn request_agree(id: i64, friend: &Friend) -> RpcParam {
    rpc_response(0, "chat-request-agree", json!([id, friend.to_rpc()]))
}

#[inline]
pub(crate) fn request_reject(id: i64) -> RpcParam {
    rpc_response(0, "chat-request-reject", json!([id]))
}

#[inline]
pub(crate) fn request_delete(id: i64) -> RpcParam {
    rpc_response(0, "chat-request-delete", json!([id]))
}

#[inline]
pub(crate) fn message_create(msg: &Message) -> RpcParam {
    rpc_response(0, "chat-message-create", json!(msg.to_rpc()))
}

#[inline]
pub(crate) fn message_delivery(id: i64, is_d: bool) -> RpcParam {
    rpc_response(0, "chat-message-delivery", json!([id, is_d]))
}

#[inline]
pub(crate) fn message_delete(id: i64) -> RpcParam {
    rpc_response(0, "chat-message-delete", json!([id]))
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

#[inline]
fn detail_list(friend: Friend, messages: Vec<Message>) -> RpcParam {
    let mut message_results = vec![];
    for msg in messages {
        message_results.push(msg.to_rpc());
    }
    json!([friend.to_rpc(), message_results])
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<Global>) {
    handler.add_method("chat-echo", |params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "chat-friend-list",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let need_online = params[0].as_bool().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let friends = Friend::list(&db)?;

            let mut results = vec![];
            let layer_lock = state.layer.read().await;
            if need_online {
                for friend in friends {
                    let online = layer_lock.chat_is_online(&friend.pid);
                    results.push(friend.to_rpc_online(online));
                }
            } else {
                for friend in friends {
                    results.push(friend.to_rpc());
                }
            }
            drop(layer_lock);

            Ok(HandleResult::rpc(json!(results)))
        },
    );

    handler.add_method(
        "chat-friend-update",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let remark = params[1].as_str().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let mut f = Friend::get(&db, &id)?;
            f.remark = remark.to_owned();
            f.me_update(&db)?;
            drop(db);

            // state.group.write().await.broadcast(
            //     &gid,
            //     InnerEvent::SessionFriendUpdate(f.gid, f.remark),
            //     FRIEND_TABLE_PATH,
            //     f.id,
            //     &mut results,
            // )?;

            Ok(results)
        },
    );

    handler.add_method(
        "chat-friend-close",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let friend = Friend::get(&db, &id)?;
            friend.close(&db)?;
            drop(db);

            let online = state.layer.write().await.chat_rm_online(&friend.pid);
            if let Some(faddr) = online {
                let data = bincode::serialize(&LayerEvent::Close)?;
                results
                    .layers
                    .push((CHAT_ID, SendType::Event(0, friend.pid, data)));

                results
                    .layers
                    .push((CHAT_ID, SendType::Disconnect(friend.pid)));
            }

            // state.group.write().await.broadcast(
            //     &gid,
            //     InnerEvent::SessionFriendClose(friend.gid),
            //     FRIEND_TABLE_PATH,
            //     friend.id,
            //     &mut results,
            // )?;

            Ok(results)
        },
    );

    handler.add_method(
        "chat-friend-delete",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let friend = Friend::get(&db, &id)?;
            Friend::delete(&db, &id)?;
            drop(db);

            let online = state.layer.write().await.chat_rm_online(&friend.pid);
            delete_avatar(&state.base, &pid, &friend.pid).await?;

            if let Some(faddr) = online {
                let data = bincode::serialize(&LayerEvent::Close)?;
                results
                    .layers
                    .push((CHAT_ID, SendType::Event(0, friend.pid, data)));

                results
                    .layers
                    .push((CHAT_ID, SendType::Disconnect(friend.pid)));
            }

            // state.group.write().await.broadcast(
            //     &gid,
            //     InnerEvent::SessionFriendDelete(friend.gid),
            //     FRIEND_TABLE_PATH,
            //     friend.id,
            //     &mut results,
            // )?;

            Ok(results)
        },
    );

    handler.add_method(
        "chat-request-list",
        |_params: Vec<RpcParam>, state: Arc<Global>| async move {
            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;
            let requests = Request::list(&db)?;
            drop(db);
            Ok(HandleResult::rpc(request_list(requests)))
        },
    );

    handler.add_method(
        "chat-request-create",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let remote_pid = id_from_str(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let remote_name = params[1].as_str().ok_or(RpcError::ParseError)?.to_string();
            let remark = params[2].as_str().ok_or(RpcError::ParseError)?.to_string();

            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            if Friend::is_friend(&db, &remote_pid)? {
                debug!("Had friend.");
                return Ok(HandleResult::new());
            }

            if let Ok(req) = Request::get_id(&db, &remote_pid) {
                debug!("Had request.");
                Request::delete(&db, &req.id)?;
            }

            let mut request =
                Request::new(remote_pid, remote_name.clone(), remark.clone(), true, false);
            request.insert(&db)?;
            drop(db);

            let mut results = HandleResult::rpc(json!(request.to_rpc()));

            let name = state.group.read().await.account(&pid)?.name.clone();
            let req = LayerEvent::Request(name, request.remark);
            let data = bincode::serialize(&req).unwrap_or(vec![]);
            let msg = SendType::Event(0, request.pid, data);
            results.layers.push((CHAT_ID, msg));

            Ok(results)
        },
    );

    handler.add_method(
        "chat-request-agree",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let mut request = Request::get(&db, &id)?;

            // group_lock.broadcast(
            //     &gid,
            //     InnerEvent::SessionRequestHandle(request.gid, true, vec![]),
            //     REQUEST_TABLE_PATH,
            //     request.id,
            //     &mut results,
            // )?;
            request.is_ok = true;
            request.is_over = true;
            request.update(&db)?;

            let friend = Friend::from_remote(&db, request.pid, request.name, "".to_owned())?;
            results.rpcs.push(json!([id, friend.to_rpc()]));

            // ADD NEW SESSION.
            let s_db = session_db(&state.base, &pid, &db_key)?;
            let mut session = friend.to_session();
            session.insert(&s_db)?;
            results.rpcs.push(session_create(&session));

            let data = bincode::serialize(&LayerEvent::Agree).unwrap_or(vec![]);
            let msg = SendType::Event(0, friend.pid, data);
            results.layers.push((CHAT_ID, msg));

            Ok(results)
        },
    );

    handler.add_method(
        "chat-request-reject",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let mut req = Request::get(&db, &id)?;
            req.is_ok = false;
            req.is_over = true;
            req.update(&db)?;
            drop(db);

            let data = bincode::serialize(&LayerEvent::Reject).unwrap_or(vec![]);
            let msg = SendType::Event(0, req.pid, data);
            let mut results = HandleResult::layer(CHAT_ID, msg);

            // state.group.write().await.broadcast(
            //     &gid,
            //     InnerEvent::SessionRequestHandle(req.gid, false, vec![]),
            //     REQUEST_TABLE_PATH,
            //     req.id,
            //     &mut results,
            // )?;
            Ok(results)
        },
    );

    handler.add_method(
        "chat-request-delete",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let req = Request::get(&db, &id)?;
            Request::delete(&db, &id)?;

            // delete avatar. check had friend.
            if Friend::get_id(&db, &req.pid).is_err() {
                delete_avatar(&state.base, &pid, &req.pid).await?;
            }
            drop(db);

            let results = HandleResult::new();
            // state.group.write().await.broadcast(
            //     &gid,
            //     InnerEvent::SessionRequestDelete(req.gid),
            //     REQUEST_TABLE_PATH,
            //     req.id,
            //     &mut results,
            // )?;
            Ok(results)
        },
    );

    handler.add_method(
        "chat-detail",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let friend = Friend::get(&db, &id)?;
            let messages = Message::get_by_fid(&db, &id)?;
            drop(db);

            Ok(HandleResult::rpc(detail_list(friend, messages)))
        },
    );

    handler.add_method(
        "chat-message-list",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let fid = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let messages = Message::get_by_fid(&db, &fid)?;
            drop(db);
            Ok(HandleResult::rpc(message_list(messages)))
        },
    );

    handler.add_method(
        "chat-message-create",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let fid = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let fpid = id_from_str(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let m_type = MessageType::from_int(params[2].as_i64().ok_or(RpcError::ParseError)?);
            let content = params[3].as_str().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let (nm, raw) =
                raw_to_network_message(&pid, &state.base, &db_key, &m_type, content).await?;
            let mut msg = Message::new(&pid, fid, true, m_type, raw, false);
            msg.insert(&db)?;

            let mut results = HandleResult::rpc(json!(msg.to_rpc()));

            let event = LayerEvent::Message(msg.hash, nm);
            let data = bincode::serialize(&event).unwrap_or(vec![]);
            results
                .layers
                .push((CHAT_ID, SendType::Event(0, fpid, data)));

            // UPDATE SESSION.
            let s_db = session_db(&state.base, &pid, &db_key)?;
            update_session(&s_db, &fid, &msg, &mut results);

            Ok(results)
        },
    );

    handler.add_method(
        "chat-message-delete",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = chat_db(&state.base, &pid, &db_key)?;

            let msg = Message::get(&db, &id)?;
            Message::delete(&db, &id)?;
            drop(db);

            Ok(HandleResult::new())
        },
    );
}
