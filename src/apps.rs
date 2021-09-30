use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::RecvType,
    primitive::{HandleResult, Result},
    rpc::RpcHandler,
};
use tokio::sync::RwLock;

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
    layer: &Arc<RwLock<Layer>>,
    fgid: GroupId,
    mgid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    println!("Handle Sync: fgid: {:?}, mgid: {:?}", fgid, mgid);
    match (fgid, mgid) {
        (group_chat::GROUP_ID, _) => group_chat::layer_handle(layer, fgid, mgid, false, msg).await,
        (_, group_chat::GROUP_ID) => group_chat::layer_handle(layer, fgid, mgid, true, msg).await,
        (domain::GROUP_ID, _) => domain::layer_handle(layer, mgid, msg).await,
        _ => chat::layer_handle(layer, fgid, mgid, msg).await,
    }
}

pub(crate) fn _app_group_handle() -> Result<HandleResult> {
    todo!()
}

pub(crate) fn _app_migrate() -> Result<()> {
    todo!()
}
