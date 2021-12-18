use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::HandleResult,
    rpc::{json, RpcHandler, RpcParam},
};

use crate::rpc::RpcState;

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method(
        "cloud-echo",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            Ok(HandleResult::rpc(json!(params)))
        },
    );
}
