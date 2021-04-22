use tdn::types::{
    primitive::HandleResult,
    rpc::{json, RpcHandler},
};

use crate::rpc::RpcState;

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("domain-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });
}
