use serde::{Deserialize, Serialize};
use tdn_types::{group::GroupId, primitives::PeerId};

// Same ID can has many name !.

/// Group chat app(service) default TDN GROUP ID.
pub const DOMAIN_ID: GroupId = 3;

/// ESSE domain service to peer layer Event.
#[derive(Serialize, Deserialize)]
pub enum LayerServerEvent {
    /// check result status.
    /// params: provider name, is support request proxy.
    Status(String, bool),
    /// register result.
    /// params: name, is_ok.
    Result(String, bool),
    /// a identity info.
    /// params: user_id, user_name, user_bio, user_avatar.
    Info(PeerId, String, String, Vec<u8>),
    /// not found a user by name.
    None(String),
    /// current name is active.
    /// params: name, is_actived
    Actived(String, bool),
    /// current name is deleted.
    /// params: name.
    Deleted(String),
    /// response the make friend.
    /// params: remote_id, name, is_ok.
    Response(PeerId, String, bool),
}

/// ESSE domain peer to service layer Event.
#[derive(Serialize, Deserialize)]
pub enum LayerPeerEvent {
    /// check service status is ok.
    Check,
    /// register new unique identity to service.
    /// params: name, bio, avatar.
    Register(String, String, Vec<u8>),
    /// update user info.
    /// params: name, bio, avatar.
    Update(String, String, Vec<u8>),
    /// search a identity info.
    /// params: name.
    Search(String),
    /// make a friend request,
    /// params: remote_name, my_name, request_remark.
    Request(String, String, String),
    /// suspend the name.
    /// params: name.
    Suspend(String),
    /// active the name.
    /// params: name.
    Active(String),
    /// delete the name.
    /// params: name.
    Delete(String),
}
