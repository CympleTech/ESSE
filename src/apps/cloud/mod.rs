mod layer;
mod models;

pub use cloud_types::CLOUD_ID as GROUP_ID;
use cloud_types::{LayerPeerEvent, PeerEvent};
use tdn::types::{
    group::GroupId,
    message::SendType,
    primitive::{HandleResult, PeerId, Result},
};
use tdn_did::Proof;

/// Send to domain service.
#[inline]
pub(crate) fn add_layer(
    results: &mut HandleResult,
    addr: PeerId,
    event: PeerEvent,
    ogid: GroupId,
) -> Result<()> {
    let proof = Proof::default();
    let data = bincode::serialize(&LayerPeerEvent(event, proof))?;
    let s = SendType::Event(0, addr, data);
    results.layers.push((ogid, GROUP_ID, s));
    Ok(())
}

pub(crate) mod rpc;
pub(crate) use layer::handle;
pub(crate) use rpc::new_rpc_handler;
