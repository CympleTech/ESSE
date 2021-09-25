use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{PeerAddr, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use super::GroupChatKey;

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

    pub fn over_rid(db: &DStorage, gid: &i64, rid: &i64, is_ok: bool) -> Result<i64> {
        let mut matrix = db.query(&format!(
            "SELECT id from requests WHERE fid = {} AND rid = {} AND is_over = 0",
            gid, rid
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
