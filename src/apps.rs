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

pub(crate) mod chat;
pub(crate) mod cloud;
pub(crate) mod device;
pub(crate) mod domain;
pub(crate) mod file;
pub(crate) mod group;
pub(crate) mod jarvis;
//pub(crate) mod dao;
pub(crate) mod wallet;

pub(crate) fn app_rpc_inject(handler: &mut RpcHandler<RpcState>) {
    device::new_rpc_handler(handler);
    chat::new_rpc_handler(handler);
    jarvis::new_rpc_handler(handler);
    domain::new_rpc_handler(handler);
    file::new_rpc_handler(handler);
    group::new_rpc_handler(handler);
    wallet::new_rpc_handler(handler);
    //dao::new_rpc_handler(handler);
    cloud::new_rpc_handler(handler);
}

pub(crate) async fn app_layer_handle(
    layer: &Arc<RwLock<Layer>>,
    fgid: GroupId,
    mgid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    match (fgid, mgid) {
        (group::GROUP_ID, _) => group::handle_peer(layer, mgid, msg).await,
        (_, group::GROUP_ID) => group::handle_server(layer, fgid, msg).await,
        //(dao::GROUP_ID, _) => dao::handle(layer, fgid, mgid, false, msg).await,
        (domain::GROUP_ID, _) => domain::handle(layer, mgid, msg).await,
        (cloud::GROUP_ID, _) => cloud::handle(layer, mgid, msg).await,
        _ => chat::handle(layer, fgid, mgid, msg).await,
    }
}

pub(crate) fn _app_group_handle() -> Result<HandleResult> {
    todo!()
}

pub(crate) fn _app_migrate() -> Result<()> {
    todo!()
}
