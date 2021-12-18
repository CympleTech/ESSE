mod layer;
mod models;

pub use organization_types::ORGANIZATION_ID as GROUP_ID;
use tdn::types::{group::GroupId, message::SendType, primitive::HandleResult};

/// Send to organization service.
#[inline]
pub(crate) fn add_layer(results: &mut HandleResult, gid: GroupId, msg: SendType) {
    results.layers.push((gid, GROUP_ID, msg));
}

pub(crate) mod rpc;
pub(crate) use layer::handle;
pub(crate) use rpc::new_rpc_handler;
