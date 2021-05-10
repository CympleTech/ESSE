use group_chat_types::{GroupInfo, GroupType};
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{new_io_error, PeerAddr, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use crate::apps::chat::MessageType;

pub(super) struct GroupChatKey(Vec<u8>);

impl GroupChatKey {
    pub fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    pub fn key(&self) -> &[u8] {
        &self.0
    }

    pub fn hash(&self) -> Vec<u8> {
        vec![] // TODO
    }

    pub fn from_hex(s: impl ToString) -> Result<Self> {
        let s = s.to_string();
        if s.len() % 2 != 0 {
            return Err(new_io_error("Hex is invalid"));
        }
        let mut value = vec![];

        for i in 0..(s.len() / 2) {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
                .map_err(|_e| new_io_error("Hex is invalid"))?;
            value.push(res);
        }

        Ok(Self(value))
    }

    pub fn to_hex(&self) -> String {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        hex
    }
}

/// Group Chat Model.
pub(super) struct GroupChat {
    /// db auto-increment id.
    pub id: i64,
    /// group chat owner.
    pub owner: GroupId,
    /// group chat id.
    pub g_id: GroupId,
    /// group chat type.
    g_type: GroupType,
    /// group chat server addresse.
    g_addr: PeerAddr,
    /// group chat name.
    g_name: String,
    /// group chat simple intro.
    g_bio: String,
    /// group chat is set top sessions.
    is_top: bool,
    /// group chat is created ok.
    is_ok: bool,
    /// group chat is closed.
    is_closed: bool,
    /// group chat need manager agree.
    is_need_agree: bool,
    /// group chat encrypted-key.
    key: GroupChatKey,
    /// group chat lastest message time. (only ESSE used)
    last_datetime: i64,
    /// group chat lastest message content.  (only ESSE used)
    last_content: String,
    /// group chat lastest message readed.  (only ESSE used)
    last_readed: bool,
    /// group chat created time.
    datetime: i64,
    /// group chat is online.
    online: bool,
    /// is deleted.
    is_deleted: bool,
}

impl GroupChat {
    pub fn new(
        owner: GroupId,
        g_type: GroupType,
        g_addr: PeerAddr,
        g_name: String,
        g_bio: String,
        is_need_agree: bool,
    ) -> Self {
        let g_id = GroupId(rand::thread_rng().gen::<[u8; 32]>());

        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        let key = GroupChatKey(vec![]);

        Self {
            owner,
            g_id,
            g_type,
            g_addr,
            g_name,
            g_bio,
            is_need_agree,
            key,
            datetime,
            id: 0,
            is_top: true,
            is_ok: false,
            is_closed: false,
            last_datetime: datetime,
            last_content: Default::default(),
            last_readed: true,
            online: false,
            is_deleted: false,
        }
    }

    pub fn to_group_info(self, avatar: Vec<u8>) -> GroupInfo {
        match self.g_type {
            GroupType::Common | GroupType::Open => GroupInfo::Common(
                self.owner,
                self.g_id,
                self.g_type,
                self.is_need_agree,
                self.g_name,
                self.g_bio,
                avatar,
            ),
            GroupType::Encrypted => GroupInfo::Common(
                // TODO encrypted
                self.owner,
                self.g_id,
                self.g_type,
                self.is_need_agree,
                self.g_name,
                self.g_bio,
                avatar,
            ),
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.owner.to_hex(),
            self.g_id.to_hex(),
            self.g_type.to_u32(),
            self.g_addr.to_hex(),
            self.g_name,
            self.g_bio,
            if self.is_top { "1" } else { "0" },
            if self.is_ok { "1" } else { "0" },
            if self.is_closed { "1" } else { "0" },
            if self.is_need_agree { "1" } else { "0" },
            self.last_datetime,
            self.last_content,
            if self.last_readed { "1" } else { "0" },
            if self.online { "1" } else { "0" },
        ])
    }

    fn from_values(mut v: Vec<DsValue>, contains_deleted: bool) -> Self {
        let is_deleted = if contains_deleted {
            v.pop().unwrap().as_bool()
        } else {
            false
        };

        Self {
            is_deleted,
            online: false,
            datetime: v.pop().unwrap().as_i64(),
            last_readed: v.pop().unwrap().as_bool(),
            last_content: v.pop().unwrap().as_string(),
            last_datetime: v.pop().unwrap().as_i64(),
            key: GroupChatKey::from_hex(v.pop().unwrap().as_string())
                .unwrap_or(GroupChatKey::new(vec![])),
            is_closed: v.pop().unwrap().as_bool(),
            is_need_agree: v.pop().unwrap().as_bool(),
            is_ok: v.pop().unwrap().as_bool(),
            is_top: v.pop().unwrap().as_bool(),
            g_bio: v.pop().unwrap().as_string(),
            g_name: v.pop().unwrap().as_string(),
            g_addr: PeerAddr::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            g_type: GroupType::from_u32(v.pop().unwrap().as_i64() as u32),
            g_id: GroupId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            owner: GroupId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn get(db: &DStorage, gid: &GroupId) -> Result<Option<GroupChat>> {
        let sql = format!("SELECT id, owner, gcd, gtype, addr, name, bio, is_top, is_ok, is_need_agree, is_closed, key, last_datetime, last_content, last_readed, datetime FROM groups WHERE gcd = '{}' AND is_deleted = false", gid.to_hex());
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Some(GroupChat::from_values(values, false)));
        }
        Ok(None)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!("INSERT INTO groups (owner, gcd, gtype, addr, name, bio, is_top, is_ok, is_need_agree, is_closed, key, last_datetime, last_content, last_readed, datetime, is_deleted) VALUES ('{}', '{}', {}, '{}', '{}', '{}', {}, {}, {}, {}, '{}', {}, '{}', {}, {}, false)",
            self.owner.to_hex(),
            self.g_id.to_hex(),
            self.g_type.to_u32(),
            self.g_addr.to_hex(),
            self.g_name,
            self.g_bio,
            if self.is_top { 1 } else { 0 },
            if self.is_ok { 1 } else { 0 },
            if self.is_need_agree { 1 } else { 0 },
            if self.is_closed { 1 } else { 0 },
            self.key.to_hex(),
            self.last_datetime,
            self.last_content,
            if self.last_readed { 1 } else { 0 },
            self.datetime,
        );
        println!("{}", sql);
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn ok(&mut self, db: &DStorage) -> Result<usize> {
        self.is_ok = true;
        let sql = format!("UPDATE groups SET is_ok=1 WHERE id = {}", self.id);
        db.update(&sql)
    }
}

/// Group Member Model.
pub(super) struct Member {
    /// db auto-increment id.
    id: i64,
    /// group's db id.
    fid: i64,
    /// member's Did(GroupId)
    m_id: GroupId,
    /// member's addresse.
    m_addr: PeerAddr,
    /// member's name.
    m_name: String,
    /// member's remark.
    m_remark: String,
    /// is group chat manager.
    is_manager: bool,
    /// member's joined time.
    datetime: i64,
}

/// Group Chat Message Model.
pub(super) struct Message {
    /// db auto-increment id.
    id: i64,
    /// group message consensus height.
    height: i64,
    /// message is mine.
    is_me: bool,
    /// group's db id.
    fid: i64,
    /// member's db id.
    m_id: i64,
    /// message type.
    m_type: MessageType,
    /// message content.
    m_content: String,
    /// message is delivery.
    m_delivery: bool,
    /// message created time.
    datetime: i64,
}
