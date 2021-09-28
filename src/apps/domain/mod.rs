use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::{HandleResult, PeerAddr},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};

use crate::rpc::RpcState;

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("domain-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "domain-add",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _provider = PeerAddr::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let _name = params[2].as_str().ok_or(RpcError::ParseError)?.to_string();

            Ok(HandleResult::rpc(json!(params)))
        },
    );

    handler.add_method(
        "domain-remove",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            Ok(HandleResult::rpc(json!(params)))
        },
    );

    handler.add_method(
        "domain-register",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let _provider = PeerAddr::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let _symbol = params[2].as_str().ok_or(RpcError::ParseError)?.to_string();
            let _bio = params[3].as_str().ok_or(RpcError::ParseError)?.to_string();

            let _me = state.group.read().await.clone_user(&gid)?;

            // Send to remote domain service.

            //

            Ok(HandleResult::rpc(json!(params)))
        },
    );
}
