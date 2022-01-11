use serde::{Deserialize, Serialize};
use tdn_did::Proof;
use tdn_types::{group::GroupId, primitive::PeerId};

use chat_types::NetworkMessage;

/// Group chat app(service) default TDN GROUP ID.
#[rustfmt::skip]
pub const GROUP_CHAT_ID: GroupId = GroupId([
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 2,
]);

/// Group chat connect data structure.
/// params: Group ID, join_proof.
#[derive(Serialize, Deserialize)]
pub struct LayerConnect(pub GroupId, pub Proof);

/// Group chat connect success result data structure.
/// params: Group ID, group name, group current height.
#[derive(Serialize, Deserialize)]
pub struct LayerResult(pub GroupId, pub String, pub i64);

/// ESSE Group chat app's layer Event.
#[derive(Serialize, Deserialize)]
pub enum LayerEvent {
    /// offline. as BaseLayerEvent.
    Offline(GroupId),
    /// suspend. as BaseLayerEvent.
    Suspend(GroupId),
    /// actived. as BaseLayerEvent.
    Actived(GroupId),
    /// online group member. Group ID, member id, member address.
    MemberOnline(GroupId, GroupId, PeerId),
    /// offline group member. Group ID, member id.
    MemberOffline(GroupId, GroupId),
    /// sync online members.
    MemberOnlineSync(GroupId),
    /// sync online members result.
    MemberOnlineSyncResult(GroupId, Vec<(GroupId, PeerId)>),
    /// Change the group name.
    GroupName(GroupId, String),
    /// close the group chat.
    GroupClose(GroupId),
    /// sync group event. Group ID, height, event.
    Sync(GroupId, i64, Event),
    /// peer sync event request. Group ID, from.
    SyncReq(GroupId, i64),
    /// sync members status.
    /// Group ID, current height, from height, to height,
    /// add members(height, member id, addr, name, avatar),
    /// leaved members(height, member id),
    /// add messages(height, member id, message, time).
    SyncRes(
        GroupId,
        i64,
        i64,
        i64,
        Vec<(i64, GroupId, PeerId, String, Vec<u8>)>,
        Vec<(i64, GroupId)>,
        Vec<(i64, GroupId, NetworkMessage, i64)>,
    ),
}

impl LayerEvent {
    /// get event's group id.
    pub fn gcd(&self) -> &GroupId {
        match self {
            Self::Offline(gcd) => gcd,
            Self::Suspend(gcd) => gcd,
            Self::Actived(gcd) => gcd,
            Self::MemberOnline(gcd, ..) => gcd,
            Self::MemberOffline(gcd, ..) => gcd,
            Self::MemberOnlineSync(gcd) => gcd,
            Self::MemberOnlineSyncResult(gcd, ..) => gcd,
            Self::GroupName(gcd, ..) => gcd,
            Self::GroupClose(gcd) => gcd,
            Self::Sync(gcd, ..) => gcd,
            Self::SyncReq(gcd, ..) => gcd,
            Self::SyncRes(gcd, ..) => gcd,
        }
    }
}

/// Group chat event.
#[derive(Serialize, Deserialize, Clone)]
pub enum Event {
    /// params: member id, member address, member name, member avatar.
    MemberJoin(GroupId, PeerId, String, Vec<u8>),
    /// params: member id,
    MemberLeave(GroupId),
    /// params: member id, message, message time.
    MessageCreate(GroupId, NetworkMessage, i64),
}
