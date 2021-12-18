use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{PeerId, Result},
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
    pub g_id: GroupId,
    /// group chat server addresse.
    pub g_addr: PeerId,
    /// group chat name.
    pub g_name: String,
    /// group is delete by owner.
    pub close: bool,
}

impl GroupChat {
    pub fn new(g_addr: PeerId, g_name: String) -> Self {
        let g_id = GroupId(rand::thread_rng().gen::<[u8; 32]>());

        Self {
            g_id,
            g_addr,
            g_name,
            id: 0,
            height: 0,
            close: false,
        }
    }

    fn new_from(g_id: GroupId, height: i64, g_addr: PeerId, g_name: String) -> Self {
        Self {
            g_id,
            g_addr,
            g_name,
            height,
            close: false,
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
            self.g_id,
            self.g_addr,
            SessionType::Group,
            self.g_name.clone(),
            datetime,
        )
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.g_id.to_hex(),
            self.g_addr.to_hex(),
            self.g_name,
            self.close,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            close: v.pop().unwrap().as_bool(),
            g_name: v.pop().unwrap().as_string(),
            g_addr: PeerId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            g_id: GroupId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            height: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn all(db: &DStorage) -> Result<Vec<GroupChat>> {
        let matrix = db.query("SELECT id, height, gcd, addr, name, close FROM groups")?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(Self::from_values(values));
        }
        Ok(groups)
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<GroupChat> {
        let sql = format!(
            "SELECT id, height, gcd, addr, name, close FROM groups WHERE id = {}",
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

    pub fn get_id(db: &DStorage, gid: &GroupId) -> Result<GroupChat> {
        let sql = format!(
            "SELECT id, height, gcd, addr, name, close FROM groups WHERE gcd = '{}'",
            gid.to_hex()
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            Ok(Self::from_values(values))
        } else {
            Err(anyhow!("missing group chat"))
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from groups WHERE gcd = '{}'",
            self.g_id.to_hex()
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
            let sql = format!(
                "UPDATE groups SET height = {}, addr='{}', name = '{}' WHERE id = {}",
                self.height,
                self.g_addr.to_hex(),
                self.g_name,
                self.id
            );
            db.update(&sql)?;
        } else {
            let sql = format!(
                "INSERT INTO groups (height, gcd, addr, name, close) VALUES ({}, '{}', '{}', '{}', {})",
                self.height,
                self.g_id.to_hex(),
                self.g_addr.to_hex(),
                self.g_name,
                self.close,
            );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn add_height(db: &DStorage, id: i64, height: i64) -> Result<usize> {
        let sql = format!("UPDATE groups SET height={} WHERE id = {}", height, id,);
        db.update(&sql)
    }

    pub fn close(db: &DStorage, gcd: &GroupId) -> Result<GroupChat> {
        let group = Self::get_id(db, gcd)?;
        let sql = format!("UPDATE groups SET close = true WHERE id = {}", group.id);
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
