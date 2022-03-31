use serde::{Deserialize, Serialize};
use tdn_types::{group::GroupId, primitives::PeerId};

/// Personal data cloud service default TDN GROUP ID.
pub const CLOUD_ID: GroupId = 5;

/// ESSE service to peer Event.
#[derive(Serialize, Deserialize)]
pub enum LayerServerEvent {
    /// check result status.
    /// params: provider name, free space, VIP space & price.
    Status(String, u64, Vec<(u64, u64)>),
    /// Peer check result: PeerId, is running.
    PeerStatus(PeerId, bool),
    /// Sync event.
    SyncEvent,
    /// Sync file.
    SyncFile,
}

/// ESSE peer to service Event.
#[derive(Serialize, Deserialize)]
pub enum LayerPeerEvent {
    /// check service info.
    Check,
    /// check PeerId is running at this service.
    PeerCheck(PeerId),
    /// Send sync event.
    Event,
    /// Send sync file.
    File,
}
