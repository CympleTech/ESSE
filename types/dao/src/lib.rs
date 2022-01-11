use serde::{Deserialize, Serialize};
use tdn_did::Proof;
use tdn_types::{group::GroupId, primitive::PeerId};

use chat_types::NetworkMessage;

/// Dao app(service) default TDN GROUP ID.
#[rustfmt::skip]
pub const DAO_ID: GroupId = GroupId([
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 3,
]);

/// Group chat types. include: Encrypted, Private, Open.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupType {
    /// encrypted group type, data is encrypted, and it can need manager
    /// or take manager's zero-knowledge-proof.
    Encrypted,
    /// private group type, data not encrypted, and need group manager agree.
    Private,
    /// opened group type, data not encrypted, anyone can join this group.
    Open,
    /// tmp group. can use descrip local tmp group chat.
    Tmp,
}

impl GroupType {
    pub fn to_i64(&self) -> i64 {
        match self {
            GroupType::Tmp => 0,
            GroupType::Open => 1,
            GroupType::Private => 2,
            GroupType::Encrypted => 3,
        }
    }

    pub fn from_i64(u: i64) -> Self {
        match u {
            0 => GroupType::Tmp,
            1 => GroupType::Open,
            2 => GroupType::Private,
            3 => GroupType::Encrypted,
            _ => GroupType::Tmp,
        }
    }
}

/// DaoInfo transfer in the network.
#[derive(Serialize, Deserialize)]
pub enum DaoInfo {
    /// params: owner, owner_name, owner_avatar, Group ID, group_type, is_must_agree_by_manager,
    /// group_name, group_bio, group_avatar.
    Common(
        GroupId,
        String,
        Vec<u8>,
        GroupId,
        GroupType,
        bool,
        String,
        String,
        Vec<u8>,
    ),
    /// params: owner, owner_name, owner_avatar, Group ID, is_must_agree_by_manager, key_hash,
    /// group_name(bytes), group_bio(bytes), group_avatar(bytes).
    Encrypted(
        GroupId,
        String,
        Vec<u8>,
        GroupId,
        bool,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
    ),
}

/// Dao chat connect data structure.
/// params: Group ID, join_proof.
#[derive(Serialize, Deserialize)]
pub struct LayerConnect(pub GroupId, pub ConnectProof);

/// Dao chat connect success result data structure.
/// params: Group ID, group current height.
#[derive(Serialize, Deserialize)]
pub struct LayerResult(pub GroupId, pub i64);

/// Dao chat connect proof.
#[derive(Serialize, Deserialize)]
pub enum ConnectProof {
    /// when is joined in group chat, can only use had to join (connect).
    /// params: proof.
    Common(Proof),
    /// zero-knowledge-proof. not has account id.
    /// verify(proof, key_hash, current_peer_addr).
    Zkp(Proof), // TODO MOCK-PROOF
}

/// Dao chat join proof.
#[derive(Serialize, Deserialize)]
pub enum JoinProof {
    /// when join the open group chat.
    /// params: member name, member avatar.
    Open(String, Vec<u8>),
    /// when is invate, it will take group_manager's proof for invate.
    /// params: invite_by_account, invite_proof, member name, member avatar.
    Invite(GroupId, Proof, String, Vec<u8>),
    /// zero-knowledge-proof. not has account id.
    /// verify(proof, key_hash, current_peer_addr).
    Zkp(Proof), // TODO MOCK-PROOF
}

/// check result type.
#[derive(Serialize, Deserialize, Debug)]
pub enum CheckType {
    /// allow to create new group.
    Allow,
    /// cannot created, remain = 0.
    None,
    /// account is suspended.
    Suspend,
    /// cannot created, no permission.
    Deny,
}

impl CheckType {
    pub fn to_u32(&self) -> u32 {
        match self {
            CheckType::Allow => 0,
            CheckType::None => 1,
            CheckType::Suspend => 2,
            CheckType::Deny => 3,
        }
    }
}

/// ESSE Dao chat app's layer Event.
#[derive(Serialize, Deserialize)]
pub enum LayerEvent {
    /// offline. as BaseLayerEvent.
    Offline(GroupId),
    /// suspend. as BaseLayerEvent.
    Suspend(GroupId),
    /// actived. as BaseLayerEvent.
    Actived(GroupId),
    /// check if account has permission to create group, and supported group types.
    Check,
    /// result check.
    /// params: check type, provider name, remain, supported_group_types.
    CheckResult(CheckType, String, i64, Vec<GroupType>),
    /// create a Group Chat.
    /// params: group_info, proof.
    Create(DaoInfo, Proof),
    /// result create group success.
    /// params: Group ID, is_ok.
    CreateResult(GroupId, bool),
    /// join group request. Group ID, Join Proof and info, request db id.
    Request(GroupId, JoinProof),
    /// request need manager to handle.
    RequestHandle(GroupId, GroupId, PeerId, JoinProof, i64, i64),
    /// manager handle request result. Group ID, request db id, is ok.
    RequestResult(GroupId, i64, bool),
    /// agree join request.
    Agree(GroupId, DaoInfo),
    /// reject join request. Group ID, if lost efficacy.
    Reject(GroupId, bool),
    /// online group member. Group ID, member id, member address.
    MemberOnline(GroupId, GroupId, PeerId),
    /// offline group member. Group ID, member id.
    MemberOffline(GroupId, GroupId),
    /// sync online members.
    MemberOnlineSync(GroupId),
    /// sync online members result.
    MemberOnlineSyncResult(GroupId, Vec<(GroupId, PeerId)>),
    /// sync group event. Group ID, height, event.
    Sync(GroupId, i64, Event),
    /// packed sync event request. Group ID, from.
    SyncReq(GroupId, i64),
    /// packed sync event. Group ID, current height, from height, to height, packed events.
    Packed(GroupId, i64, i64, i64, Vec<PackedEvent>),
}

impl LayerEvent {
    /// get event's group id.
    pub fn gcd(&self) -> Option<&GroupId> {
        match self {
            Self::Offline(gcd) => Some(gcd),
            Self::Suspend(gcd) => Some(gcd),
            Self::Actived(gcd) => Some(gcd),
            Self::Check => None,
            Self::CheckResult(..) => None,
            Self::Create(..) => None,
            Self::CreateResult(gcd, _) => Some(gcd),
            Self::Request(gcd, _) => Some(gcd),
            Self::RequestHandle(gcd, ..) => Some(gcd),
            Self::RequestResult(gcd, ..) => Some(gcd),
            Self::Agree(gcd, ..) => Some(gcd),
            Self::Reject(gcd, ..) => Some(gcd),
            Self::MemberOnline(gcd, ..) => Some(gcd),
            Self::MemberOffline(gcd, ..) => Some(gcd),
            Self::MemberOnlineSync(gcd) => Some(gcd),
            Self::MemberOnlineSyncResult(gcd, ..) => Some(gcd),
            Self::Sync(gcd, ..) => Some(gcd),
            Self::SyncReq(gcd, ..) => Some(gcd),
            Self::Packed(gcd, ..) => Some(gcd),
        }
    }

    /// check if handle this, remote must online frist.
    pub fn need_online(&self) -> bool {
        match self {
            Self::Offline(..) => true,
            Self::Suspend(..) => true,
            Self::Actived(..) => true,
            Self::RequestHandle(..) => true,
            Self::RequestResult(..) => true,
            Self::MemberOnline(..) => true,
            Self::MemberOffline(..) => true,
            Self::MemberOnlineSync(..) => true,
            Self::MemberOnlineSyncResult(..) => true,
            Self::Sync(..) => true,
            Self::SyncReq(..) => true,
            Self::Packed(..) => true,
            _ => false,
        }
    }
}

/// Dao chat packed event.
#[derive(Serialize, Deserialize)]
pub enum PackedEvent {
    GroupInfo,
    GroupTransfer,
    GroupManagerAdd,
    GroupManagerDel,
    GroupClose,
    /// params: member id, member address, member name, member avatar.
    MemberInfo(GroupId, PeerId, String, Vec<u8>),
    /// params: member id, member address, member name, member avatar, member join time.
    MemberJoin(GroupId, PeerId, String, Vec<u8>, i64),
    /// params: member id,
    MemberLeave(GroupId),
    /// params: member id, message, message time.
    MessageCreate(GroupId, NetworkMessage, i64),
    /// had in before.
    None,
}

/// Dao chat event.
#[derive(Serialize, Deserialize, Clone)]
pub enum Event {
    GroupInfo,
    GroupTransfer,
    GroupManagerAdd,
    GroupManagerDel,
    GroupClose,
    /// params: member id, member address, member name, member avatar.
    MemberInfo(GroupId, PeerId, String, Vec<u8>),
    /// params: member id, member address, member name, member avatar, member join time.
    MemberJoin(GroupId, PeerId, String, Vec<u8>, i64),
    /// params: member id,
    MemberLeave(GroupId),
    /// params: member id, message, height.
    MessageCreate(GroupId, NetworkMessage, i64),
}
