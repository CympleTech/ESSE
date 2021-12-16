use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{PeerId, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use crate::session::{Session, SessionType};

use super::Request;

pub(crate) struct Friend {
    pub id: i64,
    pub gid: GroupId,
    pub addr: PeerId,
    pub name: String,
    pub wallet: String,
    pub remark: String,
    pub is_closed: bool,
    pub is_deleted: bool,
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
            is_deleted: false,
        }
    }

    pub fn contains_addr(&self, addr: &PeerId) -> bool {
        &self.addr == addr
    }

    /// here is zero-copy and unwrap is safe. checked.
    fn from_values(mut v: Vec<DsValue>, contains_deleted: bool) -> Friend {
        let is_deleted = if contains_deleted {
            v.pop().unwrap().as_bool()
        } else {
            false
        };

        Friend {
            is_deleted,
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
        if let Some(mut friend) = Friend::get_it(&db, &request.gid)? {
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

    pub fn get(db: &DStorage, gid: &GroupId) -> Result<Option<Friend>> {
        let sql = format!("SELECT id, gid, addr, name, wallet, remark, is_closed, datetime FROM friends WHERE gid = '{}' and is_deleted = false", gid.to_hex());
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            return Ok(Some(Friend::from_values(matrix.pop().unwrap(), false))); // safe unwrap()
        }
        Ok(None)
    }

    pub fn get_it(db: &DStorage, gid: &GroupId) -> Result<Option<Friend>> {
        let sql = format!("SELECT id, gid, addr, name, wallet, remark, is_closed, datetime, is_deleted FROM friends WHERE gid = '{}'", gid.to_hex());
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            return Ok(Some(Friend::from_values(matrix.pop().unwrap(), true))); // safe unwrap()
        }
        Ok(None)
    }

    pub fn get_id(db: &DStorage, id: i64) -> Result<Option<Friend>> {
        let sql = format!("SELECT id, gid, addr, name, wallet, remark, is_closed, datetime, is_deleted FROM friends WHERE id = {}", id);
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            return Ok(Some(Friend::from_values(matrix.pop().unwrap(), true))); // safe unwrap()
        }
        Ok(None)
    }

    /// use in rpc when load account friends.
    pub fn all(db: &DStorage) -> Result<Vec<Friend>> {
        let matrix = db.query("SELECT id, gid, addr, name, wallet, remark, is_closed, datetime FROM friends where is_deleted = false")?;
        let mut friends = vec![];
        for values in matrix {
            friends.push(Friend::from_values(values, false));
        }
        Ok(friends)
    }

    /// use in rpc when load account friends.
    pub fn _all_ok(db: &DStorage) -> Result<Vec<Friend>> {
        let matrix = db.query("SELECT id, gid, addr, name, wallet, remark, is_closed, datetime FROM friends where is_closed = false")?;
        let mut friends = vec![];
        for values in matrix {
            friends.push(Friend::from_values(values, false));
        }
        Ok(friends)
    }

    /// use in layer load friends ids.
    pub fn _all_id(db: &DStorage) -> Result<Vec<(GroupId, i64)>> {
        let matrix =
            db.query("SELECT id, gid FROM friends where is_closed = false ORDER BY id DESC")?;
        let mut friends = vec![];
        for mut values in matrix {
            friends.push((
                GroupId::from_hex(values.pop().unwrap().as_str()).unwrap_or(GroupId::default()),
                values.pop().unwrap().as_i64(),
            ));
        }
        Ok(friends)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!("INSERT INTO friends (gid, addr, name, wallet, remark, is_closed, datetime, is_deleted) VALUES ('{}', '{}', '{}', '{}', '{}', {}, {}, false)",
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
        let sql = format!("UPDATE friends SET addr = '{}', name = '{}', wallet = '{}', remark = '{}', is_closed = {}, is_deleted = {} WHERE id = {}",
            self.addr.to_hex(),
            self.name,
            self.wallet,
            self.remark,
            self.is_closed,
            self.is_deleted,
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
    pub fn delete(&self, db: &DStorage) -> Result<usize> {
        let sql = format!(
            "UPDATE friends SET is_deleted = true, is_closed = true WHERE id = {}",
            self.id
        );
        db.update(&sql)
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
