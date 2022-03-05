use std::sync::Arc;
use tdn::types::{
    primitives::HandleResult,
    rpc::{json, RpcHandler, RpcParam},
};

use crate::global::Global;

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<Global>) {
    handler.add_method(
        "cloud-echo",
        |params: Vec<RpcParam>, _state: Arc<Global>| async move {
            Ok(HandleResult::rpc(json!(params)))
        },
    );
}
