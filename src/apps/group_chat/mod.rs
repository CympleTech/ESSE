mod models;

pub use group_chat_types::GROUP_CHAT_ID as GROUP_ID;
use tdn::types::{group::GroupId, message::SendType, primitive::HandleResult};

/// Group chat server to ESSE.
#[inline]
pub(super) fn add_layer(results: &mut HandleResult, gid: GroupId, msg: SendType) {
    results.layers.push((gid, GROUP_ID, msg));
}

pub(crate) mod rpc;
pub(crate) use rpc::new_rpc_handler;

mod layer;
pub(crate) use layer::handle as layer_handle;
