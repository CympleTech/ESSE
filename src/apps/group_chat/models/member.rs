use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{PeerAddr, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

/// Group Member Model.
pub(crate) struct Member {
    /// db auto-increment id.
    pub id: i64,
    /// group's db id.
    fid: i64,
    /// member's Did(GroupId)
    pub m_id: GroupId,
    /// member's addresse.
    pub m_addr: PeerAddr,
    /// member's name.
    pub m_name: String,
    /// is group chat manager.
    is_manager: bool,
    /// is member is block by me.
    is_block: bool,
    /// member's joined time.
    pub datetime: i64,
    /// member is leave or delete.
    is_deleted: bool,
}

impl Member {
    pub fn new_notime(
        fid: i64,
        m_id: GroupId,
        m_addr: PeerAddr,
        m_name: String,
        is_manager: bool,
    ) -> Self {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

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

    pub fn get(db: &DStorage, id: &i64) -> Result<Member> {
        let mut matrix = db.query(&format!(
            "SELECT id, fid, mid, addr, name, is_manager, is_block, datetime FROM members WHERE id = {}",
            id,
        ))?;
        if matrix.len() > 0 {
            Ok(Member::from_values(matrix.pop().unwrap(), false)) // safe unwrap.
        } else {
            Err(anyhow!("missing member"))
        }
    }

    pub fn get_id(db: &DStorage, fid: &i64, mid: &GroupId) -> Result<(i64, bool)> {
        let mut matrix = db.query(&format!(
            "SELECT id, is_manager FROM members WHERE fid = {} AND mid = '{}' AND is_deleted = false",
            fid,
            mid.to_hex()
        ))?;
        if matrix.len() > 0 {
            let mut values = matrix.pop().unwrap();
            let is_manager = values.pop().unwrap().as_bool(); // safe unwrap.
            let id = values.pop().unwrap().as_i64(); // safe unwrap.
            Ok((id, is_manager)) // safe unwrap.
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
