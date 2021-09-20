use rand::Rng;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{PeerAddr, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use group_chat_types::{GroupInfo, GroupType, NetworkMessage};

use crate::apps::chat::{Friend, MessageType};
use crate::session::{Session, SessionType};
use crate::storage::{
    chat_db, group_chat_db, read_avatar, read_file, read_record, write_avatar_sync,
    write_file_sync, write_image_sync, write_record_sync,
};

pub(crate) struct GroupChatKey(Vec<u8>);

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
            return Err(anyhow!("Hex is invalid"));
        }
        let mut value = vec![];

        for i in 0..(s.len() / 2) {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
                .map_err(|_e| anyhow!("Hex is invalid"))?;
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
pub(crate) struct GroupChat {
    /// db auto-increment id.
    pub id: i64,
    /// consensus height.
    pub height: i64,
    /// group chat owner.
    pub owner: GroupId,
    /// group chat id.
    pub g_id: GroupId,
    /// group chat type.
    pub g_type: GroupType,
    /// group chat server addresse.
    pub g_addr: PeerAddr,
    /// group chat name.
    pub g_name: String,
    /// group chat simple intro.
    g_bio: String,
    /// group chat is created ok.
    is_ok: bool,
    /// group chat is closed.
    is_closed: bool,
    /// group chat need manager agree.
    is_need_agree: bool,
    /// group chat encrypted-key.
    pub key: GroupChatKey,
    /// group chat created time.
    pub datetime: i64,
    /// is deleted.
    is_deleted: bool,
    /// is remote.
    pub is_remote: bool,
}

impl GroupChat {
    pub fn new(
        owner: GroupId,
        g_type: GroupType,
        g_addr: PeerAddr,
        g_name: String,
        g_bio: String,
        is_need_agree: bool,
        is_ok: bool,
        is_remote: bool,
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
            is_ok,
            is_remote,
            id: 0,
            height: 0,
            is_closed: false,
            is_deleted: false,
        }
    }

    fn new_from(
        g_id: GroupId,
        height: i64,
        owner: GroupId,
        g_type: GroupType,
        g_addr: PeerAddr,
        g_name: String,
        g_bio: String,
        is_need_agree: bool,
        key: GroupChatKey,
        is_remote: bool,
    ) -> Self {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

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
            height,
            is_remote,
            is_ok: true,
            is_closed: false,
            is_deleted: false,
        }
    }

    pub fn from_info(
        key: GroupChatKey,
        info: GroupInfo,
        height: i64,
        addr: PeerAddr,
        base: &PathBuf,
        mgid: &GroupId,
        is_remote: bool,
    ) -> Result<Self> {
        match info {
            GroupInfo::Common(owner, _, _, g_id, g_type, agree, name, g_bio, avatar) => {
                write_avatar_sync(base, &mgid, &g_id, avatar)?;
                Ok(Self::new_from(
                    g_id, height, owner, g_type, addr, name, g_bio, agree, key, is_remote,
                ))
            }
            GroupInfo::Encrypted(owner, _, _, g_id, agree, _hash, _name, _bio, avatar) => {
                // TODO decrypted.

                let g_type = GroupType::Encrypted;
                let name = "".to_owned();
                let bio = "".to_owned();

                write_avatar_sync(base, &mgid, &g_id, avatar)?;

                Ok(Self::new_from(
                    g_id, height, owner, g_type, addr, name, bio, agree, key, is_remote,
                ))
            }
        }
    }

    pub fn to_session(&self) -> Session {
        Session::new(
            self.id,
            self.g_id,
            self.g_addr,
            SessionType::Group,
            self.g_name.clone(),
            self.datetime,
        )
    }

    pub fn to_group_info(self, name: String, avatar: Vec<u8>, owner_avatar: Vec<u8>) -> GroupInfo {
        match self.g_type {
            GroupType::Private | GroupType::Open => GroupInfo::Common(
                self.owner,
                name,
                owner_avatar,
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
                name,
                owner_avatar,
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
            self.is_ok,
            self.is_closed,
            self.is_need_agree,
            self.is_remote,
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
            is_remote: v.pop().unwrap().as_bool(),
            datetime: v.pop().unwrap().as_i64(),
            key: GroupChatKey::from_hex(v.pop().unwrap().as_string())
                .unwrap_or(GroupChatKey::new(vec![])),
            is_closed: v.pop().unwrap().as_bool(),
            is_need_agree: v.pop().unwrap().as_bool(),
            is_ok: v.pop().unwrap().as_bool(),
            g_bio: v.pop().unwrap().as_string(),
            g_name: v.pop().unwrap().as_string(),
            g_addr: PeerAddr::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            g_type: GroupType::from_u32(v.pop().unwrap().as_i64() as u32),
            g_id: GroupId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            owner: GroupId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            height: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    /// use in rpc when load account friends.
    pub fn all(db: &DStorage) -> Result<Vec<GroupChat>> {
        let matrix = db.query("SELECT id, height, owner, gcd, gtype, addr, name, bio, is_ok, is_need_agree, is_closed, key, datetime, is_remote FROM groups WHERE is_deleted = false")?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(GroupChat::from_values(values, false));
        }
        Ok(groups)
    }

    /// use in rpc when load account groups.
    pub fn all_ok(db: &DStorage) -> Result<Vec<GroupChat>> {
        let matrix = db.query("SELECT id, height, owner, gcd, gtype, addr, name, bio, is_ok, is_need_agree, is_closed, key, datetime, is_remote FROM groups WHERE is_closed = false")?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(GroupChat::from_values(values, false));
        }
        Ok(groups)
    }

    /// list all local group chat as running layer.
    pub fn all_local(db: &DStorage, owner: &GroupId) -> Result<Vec<(i64, GroupId, i64)>> {
        let matrix = db.query(&format!("SELECT id, gcd, height FROM groups WHERE owner = '{}' and is_remote = false and is_closed = false", owner.to_hex()))?;
        let mut groups = vec![];
        for mut values in matrix {
            let height = values.pop().unwrap().as_i64();
            let gcd =
                GroupId::from_hex(values.pop().unwrap().as_string()).unwrap_or(Default::default());
            let id = values.pop().unwrap().as_i64();
            groups.push((id, gcd, height));
        }
        Ok(groups)
    }

    pub fn get(db: &DStorage, gid: &GroupId) -> Result<Option<GroupChat>> {
        let sql = format!("SELECT id, height, owner, gcd, gtype, addr, name, bio, is_ok, is_need_agree, is_closed, key, datetime, is_remote FROM groups WHERE gcd = '{}' AND is_deleted = false", gid.to_hex());
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Some(GroupChat::from_values(values, false)));
        }
        Ok(None)
    }

    pub fn get_id(db: &DStorage, id: &i64) -> Result<Option<GroupChat>> {
        let sql = format!("SELECT id, height, owner, gcd, gtype, addr, name, bio, is_ok, is_need_agree, is_closed, key, datetime, is_remote FROM groups WHERE id = {} AND is_deleted = false", id);
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Some(GroupChat::from_values(values, false)));
        }
        Ok(None)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from groups WHERE gcd = '{}'",
            self.g_id.to_hex()
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
            let sql = format!("UPDATE groups SET height = {}, owner = '{}', gtype = {}, addr='{}', name = '{}', bio = '{}', is_ok = {}, is_need_agree = {}, is_closed = {}, key = '{}', datetime = {}, is_remote = {}, is_deleted = false WHERE id = {}",
                self.height,
                self.owner.to_hex(),
                self.g_type.to_u32(),
                self.g_addr.to_hex(),
                self.g_name,
                self.g_bio,
                self.is_ok,
                self.is_need_agree,
                self.is_closed,
                self.key.to_hex(),
                self.datetime,
                self.is_remote,
                self.id
            );
            db.update(&sql)?;
        } else {
            let sql = format!("INSERT INTO groups (height, owner, gcd, gtype, addr, name, bio, is_ok, is_need_agree, is_closed, key, datetime, is_remote, is_deleted) VALUES ({}, '{}', '{}', {}, '{}', '{}', '{}', {}, {}, {}, '{}', {}, {}, false)",
                self.height,
                self.owner.to_hex(),
                self.g_id.to_hex(),
                self.g_type.to_u32(),
                self.g_addr.to_hex(),
                self.g_name,
                self.g_bio,
                self.is_ok,
                self.is_need_agree,
                self.is_closed,
                self.key.to_hex(),
                self.datetime,
                self.is_remote,
            );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn ok(&mut self, db: &DStorage) -> Result<usize> {
        self.is_ok = true;
        let sql = format!("UPDATE groups SET is_ok=1 WHERE id = {}", self.id);
        db.update(&sql)
    }

    pub fn add_height(db: &DStorage, id: i64, height: i64) -> Result<usize> {
        let sql = format!("UPDATE groups SET height={} WHERE id = {}", height, id,);
        db.update(&sql)
    }

    pub fn close(db: &DStorage, id: &i64) -> Result<usize> {
        let sql = format!("UPDATE groups SET is_closed = 1 WHERE id = {}", id);
        db.update(&sql)
    }

    /// return if is closed
    pub fn delete(db: &DStorage, id: &i64) -> Result<bool> {
        let sql = format!("SELECT is_closed FROM groups WHERE id = {}", id);
        let mut matrix = db.query(&sql)?;
        let is_closed = if let Some(mut value) = matrix.pop() {
            value.pop().unwrap().as_bool() // safe unwrap
        } else {
            false
        };

        let sql = format!(
            "UPDATE groups SET is_closed = 1, is_deleted = 1 WHERE id = {}",
            id
        );
        db.update(&sql)?;

        Ok(is_closed)
    }
}

/// Group Join Request model. include my requests and other requests.
/// When fid is 0, it's my requests.
pub(crate) struct Request {
    id: i64,
    fid: i64,
    rid: i64,
    pub gid: GroupId,
    pub addr: PeerAddr,
    pub name: String,
    key: GroupChatKey,
    remark: String,
    is_ok: bool,
    is_over: bool,
    datetime: i64,
    is_deleted: bool,
}

impl Request {
    pub fn new_by_remote(
        fid: i64,
        rid: i64,
        gid: GroupId,
        addr: PeerAddr,
        name: String,
        remark: String,
        datetime: i64,
    ) -> Self {
        Self {
            fid,
            rid,
            gid,
            addr,
            name,
            remark,
            datetime,
            key: GroupChatKey(vec![]),
            is_ok: false,
            is_over: false,
            is_deleted: false,
            id: 0,
        }
    }

    pub fn new_by_me(
        gid: GroupId,
        addr: PeerAddr,
        name: String,
        remark: String,
        key: GroupChatKey,
    ) -> Self {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Self {
            gid,
            addr,
            name,
            remark,
            datetime,
            key,
            is_ok: false,
            is_over: false,
            is_deleted: false,
            fid: 0,
            rid: 0,
            id: 0,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.fid,
            self.rid,
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.name,
            self.remark,
            self.is_ok,
            self.is_over,
            self.datetime,
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
            key: GroupChatKey(vec![]),
            datetime: v.pop().unwrap().as_i64(),
            is_over: v.pop().unwrap().as_bool(),
            is_ok: v.pop().unwrap().as_bool(),
            remark: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            addr: PeerAddr::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            gid: GroupId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            rid: v.pop().unwrap().as_i64(),
            fid: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn list(db: &DStorage, is_all: bool) -> Result<Vec<Request>> {
        let sql = if is_all {
            format!("SELECT id, fid, rid, gid, addr, name, remark, is_ok, is_over, datetime FROM requests WHERE is_deleted = false")
        } else {
            format!("SELECT id, fid, rid, gid, addr, name, remark, is_ok, is_over, datetime FROM requests WHERE is_deleted = false AND is_over = 0")
        };
        let matrix = db.query(&sql)?;
        let mut requests = vec![];
        for values in matrix {
            requests.push(Request::from_values(values, false));
        }
        Ok(requests)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!("INSERT INTO requests (fid, rid, gid, addr, name, remark, key, is_ok, is_over, datetime, is_deleted) VALUES ({}, {}, '{}', '{}', '{}', '{}', '{}', {}, {}, {}, false)",
            self.fid,
            self.rid,
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.name,
            self.remark,
            self.key.to_hex(),
            self.is_ok,
            self.is_over,
            self.datetime,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn exist(db: &DStorage, gcd: &GroupId) -> Result<bool> {
        let matrix = db.query(&format!(
            "SELECT id from requests WHERE gid = '{}' AND is_over = 0",
            gcd.to_hex(),
        ))?;
        if matrix.len() == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn over_rid(db: &DStorage, gcd: &GroupId, rid: &i64, is_ok: bool) -> Result<i64> {
        let mut matrix = db.query(&format!(
            "SELECT id from requests WHERE gid = '{}' AND rid = {} AND is_over = 0",
            gcd.to_hex(),
            rid
        ))?;
        if matrix.len() == 0 {
            return Err(anyhow!("request is missing"));
        }
        let id = matrix.pop().unwrap().pop().unwrap().as_i64(); // safe.
        let sql = format!(
            "UPDATE requests SET is_ok={}, is_over=1 WHERE id = {}",
            is_ok, id,
        );
        db.update(&sql)?;
        Ok(id)
    }

    pub fn over(db: &DStorage, gcd: &GroupId, is_ok: bool) -> Result<(i64, GroupChatKey)> {
        let matrix = db.query(&format!(
            "SELECT id, key from requests WHERE gid = '{}' AND is_over = 0 ORDER BY id",
            gcd.to_hex()
        ))?;
        let mut requests = vec![];
        for mut values in matrix {
            let id = values.pop().unwrap().as_i64();
            let key = GroupChatKey::from_hex(values.pop().unwrap().as_string())
                .unwrap_or(GroupChatKey::new(vec![]));
            requests.push((id, key));
        }

        let sql = format!(
            "UPDATE requests SET is_ok={}, is_over=1 WHERE gid = '{}' AND is_over = 0",
            is_ok,
            gcd.to_hex(),
        );
        db.update(&sql)?;

        if requests.len() > 0 {
            Ok(requests.pop().unwrap()) // safe.
        } else {
            Err(anyhow!("no requests"))
        }
    }
}

/// Group Member Model.
pub(crate) struct Member {
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
    /// is group chat manager.
    is_manager: bool,
    /// is member is block by me.
    is_block: bool,
    /// member's joined time.
    datetime: i64,
    /// member is leave or delete.
    is_deleted: bool,
}

impl Member {
    pub fn new(
        fid: i64,
        m_id: GroupId,
        m_addr: PeerAddr,
        m_name: String,
        is_manager: bool,
        datetime: i64,
    ) -> Self {
        Self {
            fid,
            m_id,
            m_addr,
            m_name,
            is_manager,
            datetime,
            id: 0,
            is_block: false,
            is_deleted: false,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.fid,
            self.m_id.to_hex(),
            self.m_addr.to_hex(),
            self.m_name,
            self.is_manager,
            self.is_block,
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
            datetime: v.pop().unwrap().as_i64(),
            is_block: v.pop().unwrap().as_bool(),
            is_manager: v.pop().unwrap().as_bool(),
            m_name: v.pop().unwrap().as_string(),
            m_addr: PeerAddr::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            m_id: GroupId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            fid: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn all(db: &DStorage, fid: &i64) -> Result<Vec<Member>> {
        let matrix = db.query(&format!(
            "SELECT id, fid, mid, addr, name, is_manager, is_block, datetime FROM members WHERE is_deleted = false AND fid = {}", fid))?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(Member::from_values(values, false));
        }
        Ok(groups)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from members WHERE fid = {} AND mid = '{}'",
            self.fid,
            self.m_id.to_hex()
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
            let sql = format!("UPDATE members SET addr='{}', name = '{}', is_manager = {}, datetime = {}, is_delete = false WHERE id = {}",
                self.m_addr.to_hex(),
                self.m_name,
                self.is_manager,
                self.datetime,
                self.id,
            );
            db.update(&sql)?;
        } else {
            let sql = format!("INSERT INTO members (fid, mid, addr, name, is_manager, is_block, datetime, is_deleted) VALUES ({}, '{}', '{}', '{}', {}, {}, {}, false)",
            self.fid,
            self.m_id.to_hex(),
            self.m_addr.to_hex(),
            self.m_name,
            self.is_manager,
            self.is_block,
            self.datetime,
        );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn get_id(db: &DStorage, fid: &i64, mid: &GroupId) -> Result<i64> {
        let mut matrix = db.query(&format!(
            "SELECT id FROM members WHERE fid = {} AND mid = '{}' AND is_delete = false",
            fid,
            mid.to_hex()
        ))?;
        if matrix.len() > 0 {
            Ok(matrix.pop().unwrap().pop().unwrap().as_i64()) // safe unwrap.
        } else {
            Err(anyhow!("missing member"))
        }
    }

    /// get member not deleted, not blocked.
    pub fn get_ok(db: &DStorage, fid: &i64, mid: &GroupId) -> Result<i64> {
        let mut matrix = db.query(&format!(
            "SELECT id FROM members WHERE is_deleted = false AND is_block = false AND fid = {} AND mid = '{}'",
            fid,
            mid.to_hex()
        ))?;
        if matrix.len() > 0 {
            Ok(matrix.pop().unwrap().pop().unwrap().as_i64()) // safe unwrap.
        } else {
            Err(anyhow!("missing member"))
        }
    }

    pub fn addr_update(db: &DStorage, fid: &i64, mid: &GroupId, addr: &PeerAddr) -> Result<usize> {
        let sql = format!(
            "UPDATE members SET addr='{}' WHERE fid = {} AND mid = '{}'",
            addr.to_hex(),
            fid,
            mid.to_hex(),
        );
        db.update(&sql)
    }

    pub fn update(db: &DStorage, id: &i64, addr: &PeerAddr, name: &str) -> Result<usize> {
        let sql = format!(
            "UPDATE members SET addr='{}', name='{}' WHERE id = {}",
            addr.to_hex(),
            name,
            id,
        );
        db.update(&sql)
    }

    pub fn leave(db: &DStorage, id: &i64) -> Result<usize> {
        let sql = format!("UPDATE members SET is_deleted = 1 WHERE id = {}", id);
        db.update(&sql)
    }

    pub fn block(db: &DStorage, id: &i64, block: bool) -> Result<usize> {
        let sql = format!("UPDATE members SET is_block={} WHERE id = {}", block, id,);
        db.update(&sql)
    }
}

/// Group Chat Message Model.
pub(crate) struct Message {
    /// db auto-increment id.
    id: i64,
    /// group message consensus height.
    height: i64,
    /// group's db id.
    fid: i64,
    /// member's db id.
    mid: i64,
    /// message is mine.
    is_me: bool,
    /// message type.
    m_type: MessageType,
    /// message content.
    pub content: String,
    /// message is delivery.
    is_delivery: bool,
    /// message created time.
    pub datetime: i64,
}

impl Message {
    pub(crate) fn new_with_time(
        height: i64,
        fid: i64,
        mid: i64,
        is_me: bool,
        m_type: MessageType,
        content: String,
        datetime: i64,
    ) -> Message {
        Self {
            fid,
            mid,
            m_type,
            content,
            datetime,
            height,
            is_me,
            is_delivery: true,
            id: 0,
        }
    }
    pub(crate) fn new(
        height: i64,
        fid: i64,
        mid: i64,
        is_me: bool,
        m_type: MessageType,
        content: String,
    ) -> Message {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Self::new_with_time(height, fid, mid, is_me, m_type, content, datetime)
    }
    /// here is zero-copy and unwrap is safe. checked.
    fn from_values(mut v: Vec<DsValue>) -> Message {
        Message {
            datetime: v.pop().unwrap().as_i64(),
            is_delivery: v.pop().unwrap().as_bool(),
            content: v.pop().unwrap().as_string(),
            m_type: MessageType::from_int(v.pop().unwrap().as_i64()),
            is_me: v.pop().unwrap().as_bool(),
            mid: v.pop().unwrap().as_i64(),
            fid: v.pop().unwrap().as_i64(),
            height: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.height,
            self.fid,
            self.mid,
            self.is_me,
            self.m_type.to_int(),
            self.content,
            self.is_delivery,
            self.datetime,
        ])
    }

    pub fn all(db: &DStorage, fid: &i64) -> Result<Vec<Message>> {
        let matrix = db.query(&format!("SELECT id, height, fid, mid, is_me, m_type, content, is_delivery, datetime FROM messages WHERE is_deleted = false AND fid = {}", fid))?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(Message::from_values(values));
        }
        Ok(groups)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from messages WHERE fid = {} AND height = {}",
            self.fid, self.height
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
        } else {
            let sql = format!("INSERT INTO messages (height, fid, mid, is_me, m_type, content, is_delivery, datetime, is_deleted) VALUES ({}, {}, {}, {}, {}, '{}', {}, {}, false)",
                self.height,
                self.fid,
                self.mid,
                self.is_me,
                self.m_type.to_int(),
                self.content,
                self.is_delivery,
                self.datetime,
            );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }
}

pub(super) async fn to_network_message(
    base: &PathBuf,
    gid: &GroupId,
    mtype: MessageType,
    content: &str,
) -> Result<(NetworkMessage, i64)> {
    let start = SystemTime::now();
    let datetime = start
        .duration_since(UNIX_EPOCH)
        .map(|s| s.as_secs())
        .unwrap_or(0) as i64; // safe for all life.

    let nmsg = match mtype {
        MessageType::String => NetworkMessage::String(content.to_owned()),
        MessageType::Image => {
            let bytes = read_file(&PathBuf::from(content)).await?;
            NetworkMessage::Image(bytes)
        }
        MessageType::File => {
            let file_path = PathBuf::from(content);
            let bytes = read_file(&file_path).await?;
            let old_name = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_owned();
            NetworkMessage::File(old_name, bytes)
        }
        MessageType::Contact => {
            let cid: i64 = content.parse()?;
            let db = chat_db(base, gid)?;
            let contact = Friend::get_id(&db, cid)?.ok_or(anyhow!("contact missind"))?;
            drop(db);
            let avatar_bytes = read_avatar(base, &gid, &contact.gid).await?;
            NetworkMessage::Contact(contact.name, contact.gid, contact.addr, avatar_bytes)
        }
        MessageType::Record => {
            let (bytes, time) = if let Some(i) = content.find('-') {
                let time = content[0..i].parse().unwrap_or(0);
                let bytes = read_record(base, &gid, &content[i + 1..]).await?;
                (bytes, time)
            } else {
                (vec![], 0)
            };
            NetworkMessage::Record(bytes, time)
        }
        MessageType::Emoji => {
            // TODO
            NetworkMessage::Emoji
        }
        MessageType::Phone => {
            // TODO
            NetworkMessage::Phone
        }
        MessageType::Video => {
            // TODO
            NetworkMessage::Video
        }
        MessageType::Invite => NetworkMessage::Invite(content.to_owned()),
    };

    Ok((nmsg, datetime))
}

pub(super) fn from_network_message(
    height: i64,
    gdid: i64,
    mid: GroupId,
    mgid: &GroupId,
    msg: NetworkMessage,
    datetime: i64,
    base: &PathBuf,
) -> Result<(Message, String)> {
    let db = group_chat_db(base, mgid)?;
    let mdid = Member::get_ok(&db, &gdid, &mid)?;
    let is_me = &mid == mgid;

    // handle event.
    let (m_type, raw) = match msg {
        NetworkMessage::String(content) => (MessageType::String, content),
        NetworkMessage::Image(bytes) => {
            let image_name = write_image_sync(base, mgid, bytes)?;
            (MessageType::Image, image_name)
        }
        NetworkMessage::File(old_name, bytes) => {
            let filename = write_file_sync(base, mgid, &old_name, bytes)?;
            (MessageType::File, filename)
        }
        NetworkMessage::Contact(name, rgid, addr, avatar_bytes) => {
            write_avatar_sync(base, mgid, &rgid, avatar_bytes)?;
            let tmp_name = name.replace(";", "-;");
            let contact_values = format!("{};;{};;{}", tmp_name, rgid.to_hex(), addr.to_hex());
            (MessageType::Contact, contact_values)
        }
        NetworkMessage::Emoji => {
            // TODO
            (MessageType::Emoji, "".to_owned())
        }
        NetworkMessage::Record(bytes, time) => {
            let record_name = write_record_sync(base, mgid, gdid, time, bytes)?;
            (MessageType::Record, record_name)
        }
        NetworkMessage::Phone => {
            // TODO
            (MessageType::Phone, "".to_owned())
        }
        NetworkMessage::Video => {
            // TODO
            (MessageType::Video, "".to_owned())
        }
        NetworkMessage::Invite(content) => (MessageType::Invite, content),
        NetworkMessage::None => {
            return Ok((
                Message::new(
                    height,
                    gdid,
                    mdid,
                    is_me,
                    MessageType::String,
                    "".to_owned(),
                ),
                "".to_owned(),
            ));
        }
    };

    let scontent = match m_type {
        MessageType::String => {
            format!("{}:{}", m_type.to_int(), raw)
        }
        _ => format!("{}:", m_type.to_int()),
    };

    let mut msg = Message::new_with_time(height, gdid, mdid, is_me, m_type, raw, datetime);
    msg.insert(&db)?;

    Ok((msg, scontent))
}
