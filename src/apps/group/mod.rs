mod layer;
mod models;

pub use group_types::GROUP_CHAT_ID as GROUP_ID;
use tdn::types::{group::GroupId, message::SendType, primitive::HandleResult};

/// Send to group chat service.
#[inline]
pub(crate) fn add_layer(results: &mut HandleResult, gid: GroupId, msg: SendType) {
    results.layers.push((gid, GROUP_ID, msg));
}

/// Send to group chat member.
#[inline]
pub fn add_server_layer(results: &mut HandleResult, gid: GroupId, msg: SendType) {
    results.layers.push((GROUP_ID, gid, msg));
}

pub(crate) mod rpc;
pub(crate) use layer::{group_conn, handle_peer, handle_server};
pub(crate) use models::GroupChat;
pub(crate) use models::Member;
pub(crate) use rpc::new_rpc_handler;
