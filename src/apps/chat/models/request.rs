use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{PeerId, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

#[derive(Clone)]
pub(crate) struct Request {
    pub id: i64,
    pub gid: GroupId,
    pub addr: PeerId,
    pub name: String,
    pub remark: String,
    pub is_me: bool,
    pub is_ok: bool,
    pub is_over: bool,
    pub is_delivery: bool,
    pub datetime: i64,
}

impl Request {
    pub fn new(
        gid: GroupId,
        addr: PeerId,
        name: String,
        remark: String,
        is_me: bool,
        is_delivery: bool,
    ) -> Request {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Request {
            id: 0,
            gid,
            addr,
            name,
            remark,
            is_me,
            is_ok: false,
            is_over: false,
            is_delivery,
            datetime: datetime,
        }
    }

    /// here is zero-copy and unwrap is safe. checked.
    fn from_values(mut v: Vec<DsValue>) -> Request {
        Request {
            datetime: v.pop().unwrap().as_i64(),
            is_delivery: v.pop().unwrap().as_bool(),
            is_over: v.pop().unwrap().as_bool(),
            is_ok: v.pop().unwrap().as_bool(),
            is_me: v.pop().unwrap().as_bool(),
            remark: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            addr: PeerId::from_hex(v.pop().unwrap().as_str()).unwrap_or(PeerId::default()),
            gid: GroupId::from_hex(v.pop().unwrap().as_str()).unwrap_or(GroupId::default()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.name,
            self.remark,
            self.is_me,
            self.is_ok,
            self.is_over,
            self.is_delivery,
            self.datetime,
        ])
    }

    pub fn get_id(db: &DStorage, gid: &GroupId) -> Result<Request> {
        let sql = format!("SELECT id, gid, addr, name, remark, is_me, is_ok, is_over, is_delivery, datetime FROM requests WHERE gid = '{}'", gid.to_hex());
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Request::from_values(matrix.pop().unwrap())) // safe unwrap()
        } else {
            Err(anyhow!("request is missing"))
        }
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<Request> {
        let sql = format!("SELECT id, gid, addr, name, remark, is_me, is_ok, is_over, is_delivery, datetime FROM requests WHERE id = {}", id);
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Request::from_values(matrix.pop().unwrap())) // safe unwrap()
        } else {
            Err(anyhow!("request is missing"))
        }
    }

    pub fn list(db: &DStorage) -> Result<Vec<Request>> {
        let matrix = db.query("SELECT id, gid, addr, name, remark, is_me, is_ok, is_over, is_delivery, datetime FROM requests ORDER BY id DESC")?;
        let mut requests = vec![];
        for values in matrix {
            requests.push(Request::from_values(values));
        }
        Ok(requests)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!("INSERT INTO requests (gid, addr, name, remark, is_me, is_ok, is_over, is_delivery, datetime) VALUES ('{}', '{}', '{}', '{}', {}, {}, {}, {}, {})",
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.name,
            self.remark,
            self.is_me,
            self.is_ok,
            self.is_over,
            self.is_delivery,
            self.datetime,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn update(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("UPDATE requests SET gid='{}', addr='{}', name='{}', remark='{}', is_me={}, is_ok={}, is_over={}, is_delivery={}, datetime={} WHERE id = {}",
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.name,
            self.remark,
            self.is_me,
            self.is_ok,
            self.is_over,
            self.is_delivery,
            self.datetime,
            self.id,
        );
        db.update(&sql)
    }

    pub fn delivery(db: &DStorage, id: i64, is_delivery: bool) -> Result<usize> {
        let sql = format!(
            "UPDATE requests SET is_delivery={} WHERE id = {}",
            if is_delivery { 1 } else { 0 },
            id,
        );
        db.update(&sql)
    }

    pub fn delete(db: &DStorage, id: &i64) -> Result<usize> {
        let sql = format!("DELETE FROM requests WHERE id = {}", id);
        let size = db.delete(&sql)?;
        // TODO delete avatar.
        Ok(size)
    }
}
