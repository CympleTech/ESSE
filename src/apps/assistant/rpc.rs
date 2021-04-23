use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::HandleResult,
    rpc::{json, rpc_response, RpcHandler, RpcParam},
};

use crate::rpc::RpcState;
use crate::storage::assistant_db;

use super::{Message, MessageType};

#[inline]
pub(crate) fn _assistant_create(mgid: GroupId, device: &Message) -> RpcParam {
    rpc_response(0, "assistant-create", json!(device.to_rpc()), mgid)
}

#[inline]
pub(crate) fn _assistant_delete(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "assistant-delete", json!([id]), mgid)
}

#[inline]
pub(crate) fn _assistant_update(mgid: GroupId, id: i64, message: &Message) -> RpcParam {
    rpc_response(
        0,
        "assistant-update",
        json!([id, message.a_type.to_int(), message.a_content]),
        mgid,
    )
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("assistant-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "assistant-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let db = assistant_db(state.layer.read().await.base(), &gid)?;
            let devices = Message::all(&db)?;
            db.close()?;
            let mut results = vec![];
            for device in devices {
                results.push(device.to_rpc());
            }
            Ok(HandleResult::rpc(json!(results)))
        },
    );

    handler.add_method(
        "assistant-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let q_type = MessageType::from_int(params[0].as_i64()?);
            let q_content = params[1].as_str()?.to_string();

            let base = state.layer.read().await.base().clone();
            let q_raw = q_type.handle(&base, &gid, q_content).await?;

            // echo
            let a_type = q_type.clone();
            let a_content = q_raw.clone();

            let mut msg = Message::new(q_type, q_raw, a_type, a_content);
            let db = assistant_db(state.layer.read().await.base(), &gid)?;
            msg.insert(&db)?;
            db.close()?;

            let results = HandleResult::rpc(json!(msg.to_rpc()));
            Ok(results)
        },
    );

    handler.add_method(
        "assistant-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;
            let db = assistant_db(state.layer.read().await.base(), &gid)?;
            Message::delete(&db, id)?;
            db.close()?;
            Ok(HandleResult::new())
        },
    );
}
