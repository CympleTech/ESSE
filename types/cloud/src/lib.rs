use serde::{Deserialize, Serialize};
use tdn_did::Proof;
use tdn_types::group::GroupId;

/// Personal data cloud service default TDN GROUP ID.
#[rustfmt::skip]
pub const CLOUD_ID: GroupId = GroupId([
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 5,
]);

/// ESSE service to peer layer Event.
#[derive(Serialize, Deserialize)]
pub struct LayerServerEvent(pub ServerEvent, pub Proof);

/// ESSE peer to layer Event.
#[derive(Serialize, Deserialize)]
pub struct LayerPeerEvent(pub PeerEvent, pub Proof);

/// ESSE service to peer Event.
#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    /// check result status.
    /// params: provider name, is support request proxy.
    Status(String, bool),
}

/// ESSE peer to service Event.
#[derive(Serialize, Deserialize)]
pub enum PeerEvent {
    /// check service status is ok.
    Check,
}
