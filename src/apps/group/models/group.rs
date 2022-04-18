use group_types::GroupChatId;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    primitives::{PeerId, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use crate::session::{Session, SessionType};

use super::{Member, Message};

/// Group Chat Model.
pub(crate) struct GroupChat {
    /// db auto-increment id.
    pub id: i64,
    /// consensus height.
    pub height: i64,
    /// group chat id.
    pub gid: GroupChatId,
    /// group chat server addresse.
    pub addr: PeerId,
    /// group chat name.
    pub name: String,
    /// group is delete by owner.
    pub close: bool,
    /// group is in my device.
    pub local: bool,
}

impl GroupChat {
    pub fn new(addr: PeerId, name: String) -> Self {
        let gid = rand::thread_rng().gen::<GroupChatId>();

        Self {
            gid,
            addr,
            name,
            id: 0,
            height: 0,
            close: false,
            local: true,
        }
    }

    pub fn from(gid: GroupChatId, height: i64, addr: PeerId, name: String) -> Self {
        Self {
            gid,
            addr,
            name,
            height,
            close: false,
            local: false,
            id: 0,
        }
    }

    pub fn to_session(&self) -> Session {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Session::new(
            self.id,
            self.gid.to_string(),
            self.addr,
            SessionType::Group,
            self.name.clone(),
            datetime,
        )
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.gid,
            self.addr.to_hex(),
            self.name,
            self.close,
            self.local,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            local: v.pop().unwrap().as_bool(),
            close: v.pop().unwrap().as_bool(),
            name: v.pop().unwrap().as_string(),
            addr: PeerId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            gid: v.pop().unwrap().as_i64() as GroupChatId,
            height: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn local(db: &DStorage) -> Result<Vec<GroupChat>> {
        let matrix = db.query(
            "SELECT id, height, gid, addr, name, is_close, is_local FROM groups WHERE is_local = true",
        )?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(Self::from_values(values));
        }
        Ok(groups)
    }

    pub fn all(db: &DStorage) -> Result<Vec<GroupChat>> {
        let matrix =
            db.query("SELECT id, height, gid, addr, name, is_close, is_local FROM groups")?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(Self::from_values(values));
        }
        Ok(groups)
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<GroupChat> {
        let sql = format!(
            "SELECT id, height, gid, addr, name, is_close, is_local FROM groups WHERE id = {}",
            id
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            Ok(Self::from_values(values))
        } else {
            Err(anyhow!("missing group chat"))
        }
    }

    pub fn get_id(db: &DStorage, gid: &GroupChatId, addr: &PeerId) -> Result<GroupChat> {
        let sql = format!("SELECT id, height, gid, addr, name, is_close, is_local FROM groups WHERE gid = {} AND addr = '{}'", gid, addr.to_hex());
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            Ok(Self::from_values(values))
        } else {
            Err(anyhow!("missing group chat"))
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let unique_check = db.query(&format!(
            "SELECT id from groups WHERE gid = {} AND addr = '{}'",
            self.gid,
            self.addr.to_hex()
        ))?;
        if unique_check.len() > 0 {
            self.gid += 1;
            return self.insert(db);
        } else {
            let sql = format!(
                "INSERT INTO groups (height, gid, addr, name, is_close, is_local) VALUES ({}, {}, '{}', '{}', {}, {})",
                self.height,
                self.gid,
                self.addr.to_hex(),
                self.name,
                self.close,
                self.local,
            );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn add_height(db: &DStorage, id: i64, height: i64) -> Result<usize> {
        let sql = format!("UPDATE groups SET height={} WHERE id = {}", height, id);
        db.update(&sql)
    }

    pub fn update_name(db: &DStorage, id: &i64, name: &str) -> Result<usize> {
        let sql = format!("UPDATE groups SET name='{}' WHERE id = {}", name, id);
        db.update(&sql)
    }

    pub fn close(db: &DStorage, id: &i64) -> Result<GroupChat> {
        let sql = format!("UPDATE groups SET is_close = true WHERE id = {}", id);
        db.update(&sql)?;
        Self::get(db, id)
    }

    pub fn close_id(db: &DStorage, gid: &GroupChatId, addr: &PeerId) -> Result<GroupChat> {
        let group = Self::get_id(db, gid, addr)?;
        let sql = format!("UPDATE groups SET is_close = true WHERE id = {}", group.id);
        db.update(&sql)?;
        Ok(group)
    }

    pub fn delete(db: &DStorage, id: &i64) -> Result<GroupChat> {
        let group = Self::get(db, id)?;
        let sql = format!("DELETE FROM groups WHERE id = {}", id);
        db.delete(&sql)?;

        // delete all members and messages;
        let _ = Member::delete(db, id);
        let _ = Message::delete(db, id);
        Ok(group)
    }
}
