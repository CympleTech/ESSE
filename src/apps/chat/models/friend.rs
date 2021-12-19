use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{PeerId, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use crate::session::{Session, SessionType};

use super::{Message, Request};

pub(crate) struct Friend {
    pub id: i64,
    pub gid: GroupId,
    pub addr: PeerId,
    pub name: String,
    pub wallet: String,
    pub remark: String,
    pub is_closed: bool,
    pub datetime: i64,
}

impl Friend {
    pub fn new(gid: GroupId, addr: PeerId, name: String, wallet: String, remark: String) -> Friend {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Friend {
            id: 0,
            gid,
            addr,
            name,
            wallet,
            remark,
            datetime,
            is_closed: false,
        }
    }

    pub fn contains_addr(&self, addr: &PeerId) -> bool {
        &self.addr == addr
    }

    /// here is zero-copy and unwrap is safe.
    fn from_values(mut v: Vec<DsValue>) -> Friend {
        Friend {
            datetime: v.pop().unwrap().as_i64(),
            is_closed: v.pop().unwrap().as_bool(),
            remark: v.pop().unwrap().as_string(),
            wallet: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            addr: PeerId::from_hex(v.pop().unwrap().as_str()).unwrap_or(PeerId::default()),
            gid: GroupId::from_hex(v.pop().unwrap().as_str()).unwrap_or(GroupId::default()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn from_request(db: &DStorage, request: Request) -> Result<Friend> {
        if let Ok(mut friend) = Friend::get_id(&db, &request.gid) {
            friend.name = request.name;
            friend.addr = request.addr;
            friend.is_closed = false;
            friend.remote_update(&db)?;
            Ok(friend)
        } else {
            let mut friend = request.to_friend();
            friend.insert(&db)?;
            Ok(friend)
        }
    }

    pub fn to_session(&self) -> Session {
        Session::new(
            self.id,
            self.gid,
            self.addr,
            SessionType::Chat,
            self.name.clone(),
            self.datetime,
        )
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.name,
            self.wallet,
            self.remark,
            self.is_closed,
            self.datetime
        ])
    }

    pub fn get_id(db: &DStorage, gid: &GroupId) -> Result<Friend> {
        let sql = format!("SELECT id, gid, addr, name, wallet, remark, is_closed, datetime FROM friends WHERE gid = '{}'", gid.to_hex());
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Friend::from_values(matrix.pop().unwrap())) // safe unwrap()
        } else {
            Err(anyhow!("friend is missing."))
        }
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<Friend> {
        let sql = format!("SELECT id, gid, addr, name, wallet, remark, is_closed, datetime, is_deleted FROM friends WHERE id = {}", id);
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Friend::from_values(matrix.pop().unwrap())) // safe unwrap()
        } else {
            Err(anyhow!("friend is missing."))
        }
    }

    /// use in rpc when load account friends.
    pub fn list(db: &DStorage) -> Result<Vec<Friend>> {
        let matrix = db.query(
            "SELECT id, gid, addr, name, wallet, remark, is_closed, datetime FROM friends",
        )?;
        let mut friends = vec![];
        for values in matrix {
            friends.push(Friend::from_values(values));
        }
        Ok(friends)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!("INSERT INTO friends (gid, addr, name, wallet, remark, is_closed, datetime) VALUES ('{}', '{}', '{}', '{}', '{}', {}, {})",
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.name,
            self.wallet,
            self.remark,
            self.is_closed,
            self.datetime,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn update(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("UPDATE friends SET addr = '{}', name = '{}', wallet = '{}', remark = '{}', is_closed = {} WHERE id = {}",
            self.addr.to_hex(),
            self.name,
            self.wallet,
            self.remark,
            self.is_closed,
            self.id
        );
        db.update(&sql)
    }

    pub fn me_update(&mut self, db: &DStorage) -> Result<usize> {
        let sql = format!(
            "UPDATE friends SET remark='{}' WHERE id = {}",
            self.remark, self.id,
        );
        db.update(&sql)
    }

    pub fn addr_update(db: &DStorage, id: i64, addr: &PeerId) -> Result<usize> {
        let sql = format!(
            "UPDATE friends SET addr='{}' WHERE id = {}",
            addr.to_hex(),
            id,
        );
        db.update(&sql)
    }

    pub fn remote_update(&self, db: &DStorage) -> Result<usize> {
        let sql = format!(
            "UPDATE friends SET addr='{}', name='{}', wallet='{}', is_closed = false, is_deleted = false WHERE id = {}",
            self.addr.to_hex(),
            self.name,
            self.wallet,
            self.id,
        );
        db.update(&sql)
    }

    /// used in rpc, when what to delete a friend.
    pub fn close(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("UPDATE friends SET is_closed = true WHERE id = {}", self.id);
        db.update(&sql)
    }

    /// used in rpc, when what to delete a friend.
    pub fn delete(db: &DStorage, id: &i64) -> Result<usize> {
        let sql = format!("DELETE FROM friends WHERE id = {}", id);
        db.update(&sql)?;

        // TODO delete friend avatar.

        // delete messages;
        Message::delete_by_fid(&db, id)
    }

    pub fn is_friend(db: &DStorage, gid: &GroupId) -> Result<bool> {
        let sql = format!(
            "SELECT id FROM friends WHERE is_closed = false and gid = '{}'",
            gid.to_hex()
        );
        let matrix = db.query(&sql)?;
        Ok(matrix.len() > 0)
    }

    /// used in layers, when receive remote had closed.
    pub fn id_close(db: &DStorage, id: i64) -> Result<usize> {
        let sql = format!("UPDATE friends SET is_closed = true WHERE id = {}", id);
        db.update(&sql)
    }
}
