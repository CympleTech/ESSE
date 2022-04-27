use esse_primitives::NetworkMessage;
use serde::{Deserialize, Serialize};
use tdn_types::{group::GroupId, primitives::PeerId};

/// Dao app(service) default TDN GROUP ID.
pub const DAO_ID: GroupId = 2;

/// Dao ID type.
pub type DaoId = u64;

/// DAO types. include: Encrypted, Private, Open.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum DaoType {
    /// encrypted dao, data is encrypted, and it can need manager
    /// or take manager's zero-knowledge-proof.
    Encrypted,
    /// private dao, data not encrypted, and need manager agree.
    Private,
    /// opened dao, data not encrypted, anyone can join this dao.
    Open,
}

impl DaoType {
    pub fn to_i64(&self) -> i64 {
        match self {
            DaoType::Open => 0,
            DaoType::Private => 1,
            DaoType::Encrypted => 2,
        }
    }

    pub fn from_i64(u: i64) -> Self {
        match u {
            1 => DaoType::Private,
            2 => DaoType::Encrypted,
            _ => DaoType::Open,
        }
    }
}

/// DaoInfo transfer in the network.
#[derive(Serialize, Deserialize)]
pub enum DaoInfo {
    /// params: owner, owner_name, owner_avatar, dao ID, dao_type, is_must_agree_by_manager,
    /// dao_name, dao_bio, dao_avatar.
    Common(
        PeerId,
        String,
        Vec<u8>,
        DaoId,
        DaoType,
        bool,
        String,
        String,
        Vec<u8>,
    ),
    /// params: owner, owner_name, owner_avatar, dao ID, is_must_agree_by_manager, key_hash,
    /// dao_name(bytes), dao_bio(bytes), dao_avatar(bytes).
    Encrypted(
        PeerId,
        String,
        Vec<u8>,
        DaoId,
        bool,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
    ),
}

/// Dao connect data structure.
/// params: DAO ID, join_proof.
#[derive(Serialize, Deserialize)]
pub struct LayerConnect(pub DaoId, pub ConnectProof);

/// Dao connect success result data structure.
/// params: DAO ID, dao current height.
#[derive(Serialize, Deserialize)]
pub struct LayerResult(pub DaoId, pub i64);

/// Dao connect proof.
#[derive(Serialize, Deserialize)]
pub enum ConnectProof {
    /// when is joined in dao, can only use had to join (connect).
    /// params: proof.
    Common,
    /// zero-knowledge-proof. not has account id.
    /// verify(proof, key_hash, current_peer_addr).
    Zkp, // TODO MOCK-PROOF
}

/// Dao join proof.
#[derive(Serialize, Deserialize)]
pub enum JoinProof {
    /// when join the open dao.
    /// params: member name, member avatar.
    Open(String, Vec<u8>),
    /// when is invate, it will take dao manager's proof for invate.
    /// params: invite_by_account, member name, member avatar.
    Invite(PeerId, String, Vec<u8>),
    /// zero-knowledge-proof. not has account id.
    /// verify(proof, key_hash, current_peer_addr).
    Zkp, // TODO MOCK-PROOF
}

/// check result type.
#[derive(Serialize, Deserialize, Debug)]
pub enum CheckType {
    /// allow to create new dao.
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

/// ESSE DAO service's layer Event.
#[derive(Serialize, Deserialize)]
pub enum LayerEvent {
    /// offline. as BaseLayerEvent.
    Offline(DaoId),
    /// suspend. as BaseLayerEvent.
    Suspend(DaoId),
    /// actived. as BaseLayerEvent.
    Actived(DaoId),
    /// check if account has permission to create dao service, and supported types.
    Check,
    /// result check.
    /// params: check type, provider name, remain, supported_dao_types.
    CheckResult(CheckType, String, i64, Vec<DaoType>),
    /// create a DAO.
    /// params: dao_info.
    Create(DaoInfo),
    /// result create DAO success.
    /// params: DAO ID, is_ok.
    CreateResult(DaoId, bool),
    /// join DAO request. DAO ID, Join Proof and info, request db id.
    Request(DaoId, JoinProof),
    /// request need manager to handle.
    RequestHandle(DaoId, PeerId, JoinProof, i64, i64),
    /// manager handle request result. DAO ID, request db id, is ok.
    RequestResult(DaoId, i64, bool),
    /// agree join request.
    Agree(DaoId, DaoInfo),
    /// reject join request. DAO ID, if lost efficacy.
    Reject(DaoId, bool),
    /// online DAO member. DAO ID, member id.
    MemberOnline(DaoId, PeerId),
    /// offline DAO member. DAO ID, member id.
    MemberOffline(DaoId, PeerId),
    /// sync online members.
    MemberOnlineSync(DaoId),
    /// sync online members result.
    MemberOnlineSyncResult(DaoId, Vec<PeerId>),
    /// sync event. DAO ID, height, event.
    Sync(DaoId, i64, Event),
    /// packed sync event request. DAO ID, from.
    SyncReq(DaoId, i64),
    /// packed sync event. DAO ID, current height, from height, to height, packed events.
    Packed(DaoId, i64, i64, i64, Vec<PackedEvent>),
}

impl LayerEvent {
    /// get event's DAO id.
    pub fn dao_id(&self) -> Option<&DaoId> {
        match self {
            Self::Offline(did) => Some(did),
            Self::Suspend(did) => Some(did),
            Self::Actived(did) => Some(did),
            Self::Check => None,
            Self::CheckResult(..) => None,
            Self::Create(..) => None,
            Self::CreateResult(did, _) => Some(did),
            Self::Request(did, _) => Some(did),
            Self::RequestHandle(did, ..) => Some(did),
            Self::RequestResult(did, ..) => Some(did),
            Self::Agree(did, ..) => Some(did),
            Self::Reject(did, ..) => Some(did),
            Self::MemberOnline(did, ..) => Some(did),
            Self::MemberOffline(did, ..) => Some(did),
            Self::MemberOnlineSync(did) => Some(did),
            Self::MemberOnlineSyncResult(did, ..) => Some(did),
            Self::Sync(did, ..) => Some(did),
            Self::SyncReq(did, ..) => Some(did),
            Self::Packed(did, ..) => Some(did),
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

/// DAO packed event.
#[derive(Serialize, Deserialize)]
pub enum PackedEvent {
    Info,
    Transfer,
    ManagerAdd,
    ManagerDel,
    Close,
    /// params: member id, member name, member avatar.
    MemberInfo(PeerId, String, Vec<u8>),
    /// params: member id, member name, member avatar, member join time.
    MemberJoin(PeerId, String, Vec<u8>, i64),
    /// params: member id,
    MemberLeave(PeerId),
    /// params: member id, message, message time.
    MessageCreate(PeerId, NetworkMessage, i64),
    /// had in before.
    None,
}

/// Dao chat event.
#[derive(Serialize, Deserialize, Clone)]
pub enum Event {
    Info,
    Transfer,
    ManagerAdd,
    ManagerDel,
    Close,
    /// params: member id, member name, member avatar.
    MemberInfo(PeerId, String, Vec<u8>),
    /// params: member id, member name, member avatar, member join time.
    MemberJoin(PeerId, String, Vec<u8>, i64),
    /// params: member id,
    MemberLeave(PeerId),
    /// params: member id, message, height.
    MessageCreate(PeerId, NetworkMessage, i64),
}
