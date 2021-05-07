use tdn::types::{
    group::GroupId,
    message::RecvType,
    primitive::{HandleResult, Result},
    rpc::RpcHandler,
};

use crate::layer::Layer;
use crate::rpc::RpcState;

pub(crate) mod assistant;
pub(crate) mod chat;
pub(crate) mod device;
pub(crate) mod domain;
pub(crate) mod file;
pub(crate) mod group_chat;

pub(crate) fn app_rpc_inject(handler: &mut RpcHandler<RpcState>) {
    device::new_rpc_handler(handler);
    chat::new_rpc_handler(handler);
    assistant::new_rpc_handler(handler);
    domain::new_rpc_handler(handler);
    file::new_rpc_handler(handler);
    group_chat::new_rpc_handler(handler);
}

pub(crate) async fn app_layer_handle(
    layer: &mut Layer,
    fgid: GroupId,
    mgid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    match fgid {
        group_chat::GROUP_ID => group_chat::layer_handle(mgid, msg),
        _ => chat::layer_handle(layer, fgid, mgid, msg).await,
    }
}

pub(crate) fn _app_group_handle() -> Result<HandleResult> {
    todo!()
}

pub(crate) fn _app_migrate() -> Result<()> {
    todo!()
}
