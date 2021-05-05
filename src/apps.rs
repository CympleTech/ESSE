use tdn::types::{
    group::GroupId,
    primitive::{HandleResult, PeerAddr, Result},
    rpc::RpcHandler,
};

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

pub(crate) fn _app_layer_handle(
    _gid: GroupId,
    _fgid: GroupId,
    _addr: PeerAddr,
    _data: Vec<u8>,
) -> Result<HandleResult> {
    todo!()
}

pub(crate) fn _app_group_handle() -> Result<HandleResult> {
    todo!()
}

pub(crate) fn _app_migrate() -> Result<()> {
    todo!()
}
