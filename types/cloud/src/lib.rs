use serde::{Deserialize, Serialize};
use tdn_types::group::GroupId;

/// Personal data cloud service default TDN GROUP ID.
pub const CLOUD_ID: GroupId = 5;

/// ESSE service to peer layer Event.
#[derive(Serialize, Deserialize)]
pub struct LayerServerEvent(pub ServerEvent);

/// ESSE peer to layer Event.
#[derive(Serialize, Deserialize)]
pub struct LayerPeerEvent(pub PeerEvent);

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
