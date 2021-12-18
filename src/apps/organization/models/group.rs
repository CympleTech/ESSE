use rand::Rng;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{PeerId, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use group_types::{GroupInfo, GroupType};

use crate::session::{Session, SessionType};
use crate::storage::write_avatar_sync;

use super::GroupChatKey;

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
    pub g_addr: PeerId,
    /// group chat name.
    pub g_name: String,
    /// group chat simple intro.
    g_bio: String,
    /// group chat is created ok.
    is_ok: bool,
    /// group chat is closed.
    is_closed: bool,
    /// group chat need manager agree.
    pub is_need_agree: bool,
    /// group chat encrypted-key.
    pub key: GroupChatKey,
    /// group chat created time.
    pub datetime: i64,
    /// is remote.
    pub is_remote: bool,
}

impl GroupChat {
    pub fn new(
        owner: GroupId,
        g_type: GroupType,
        g_addr: PeerId,
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
        }
    }

    fn new_from(
        g_id: GroupId,
        height: i64,
        owner: GroupId,
        g_type: GroupType,
        g_addr: PeerId,
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
        }
    }

    pub fn from_info(
        key: GroupChatKey,
        info: GroupInfo,
        height: i64,
        addr: PeerId,
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

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            is_remote: v.pop().unwrap().as_bool(),
            datetime: v.pop().unwrap().as_i64(),
            key: GroupChatKey::from_hex(v.pop().unwrap().as_string())
                .unwrap_or(GroupChatKey::new(vec![])),
            is_closed: v.pop().unwrap().as_bool(),
            is_need_agree: v.pop().unwrap().as_bool(),
            is_ok: v.pop().unwrap().as_bool(),
            g_bio: v.pop().unwrap().as_string(),
            g_name: v.pop().unwrap().as_string(),
            g_addr: PeerId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
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
            groups.push(GroupChat::from_values(values));
        }
        Ok(groups)
    }

    /// use in rpc when load account groups.
    pub fn _all_ok(db: &DStorage) -> Result<Vec<GroupChat>> {
        let matrix = db.query("SELECT id, height, owner, gcd, gtype, addr, name, bio, is_ok, is_need_agree, is_closed, key, datetime, is_remote FROM groups WHERE is_closed = false")?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(GroupChat::from_values(values));
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
            return Ok(Some(GroupChat::from_values(values)));
        }
        Ok(None)
    }

    pub fn get_id(db: &DStorage, id: &i64) -> Result<Option<GroupChat>> {
        let sql = format!("SELECT id, height, owner, gcd, gtype, addr, name, bio, is_ok, is_need_agree, is_closed, key, datetime, is_remote FROM groups WHERE id = {} AND is_deleted = false", id);
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Some(GroupChat::from_values(values)));
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
