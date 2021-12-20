use std::path::PathBuf;
use tdn::types::{
    group::GroupId,
    primitive::{PeerId, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use crate::storage::read_avatar;

/// Group Member Model.
pub(crate) struct Member {
    /// db auto-increment id.
    pub id: i64,
    /// group consensus height.
    pub height: i64,
    /// group's db id.
    pub fid: i64,
    /// member's Did(GroupId)
    pub m_id: GroupId,
    /// member's addresse.
    pub m_addr: PeerId,
    /// member's name.
    pub m_name: String,
    /// if leave from group.
    pub leave: bool,
}

impl Member {
    pub fn new(height: i64, fid: i64, m_id: GroupId, m_addr: PeerId, m_name: String) -> Self {
        Self {
            height,
            fid,
            m_id,
            m_addr,
            m_name,
            leave: false,
            id: 0,
        }
    }

    pub fn info(id: i64, fid: i64, m_id: GroupId, m_addr: PeerId, m_name: String) -> Self {
        Self {
            id,
            fid,
            m_id,
            m_addr,
            m_name,
            leave: false,
            height: 0,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.fid,
            self.m_id.to_hex(),
            self.m_addr.to_hex(),
            self.m_name,
            self.leave,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            leave: v.pop().unwrap().as_bool(),
            m_name: v.pop().unwrap().as_string(),
            m_addr: PeerId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            m_id: GroupId::from_hex(v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            fid: v.pop().unwrap().as_i64(),
            height: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn list(db: &DStorage, fid: &i64) -> Result<Vec<Member>> {
        let matrix = db.query(&format!(
            "SELECT id, height, fid, mid, addr, name, leave FROM members WHERE fid = {}",
            fid
        ))?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(Self::from_values(values));
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
            let sql = format!(
                "UPDATE members SET height = {}, addr='{}', name = '{}', leave = false WHERE id = {}",
                self.height,
                self.m_addr.to_hex(),
                self.m_name,
                self.id,
            );
            db.update(&sql)?;
        } else {
            let sql = format!("INSERT INTO members (height, fid, mid, addr, name, leave) VALUES ({}, {}, '{}', '{}', '{}', false)",
            self.height,
            self.fid,
            self.m_id.to_hex(),
            self.m_addr.to_hex(),
            self.m_name,
        );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn _get(db: &DStorage, id: &i64) -> Result<Member> {
        let mut matrix = db.query(&format!(
            "SELECT id, height, fid, mid, addr, name, leave FROM members WHERE id = {}",
            id,
        ))?;
        if matrix.len() > 0 {
            Ok(Self::from_values(matrix.pop().unwrap())) // safe unwrap.
        } else {
            Err(anyhow!("missing member"))
        }
    }

    pub fn get_id(db: &DStorage, fid: &i64, gid: &GroupId) -> Result<i64> {
        let mut matrix = db.query(&format!(
            "SELECT id FROM members WHERE fid = {} AND mid = '{}'",
            fid,
            gid.to_hex()
        ))?;
        if matrix.len() > 0 {
            Ok(matrix.pop().unwrap().pop().unwrap().as_i64()) // safe unwrap.
        } else {
            Err(anyhow!("missing member"))
        }
    }

    pub fn addr_update(db: &DStorage, fid: &i64, mid: &GroupId, addr: &PeerId) -> Result<i64> {
        let mdid = Self::get_id(db, fid, mid)?;
        let sql = format!(
            "UPDATE members SET addr='{}' WHERE fid = {} AND mid = '{}'",
            addr.to_hex(),
            fid,
            mid.to_hex(),
        );
        db.update(&sql)?;
        Ok(mdid)
    }

    pub fn update(
        db: &DStorage,
        id: &i64,
        height: &i64,
        addr: &PeerId,
        name: &str,
    ) -> Result<usize> {
        let sql = format!(
            "UPDATE members SET height = {}, addr='{}', name='{}' WHERE id = {}",
            height,
            addr.to_hex(),
            name,
            id,
        );
        db.update(&sql)
    }

    pub fn leave(db: &DStorage, id: &i64, height: &i64) -> Result<usize> {
        let sql = format!(
            "UPDATE members SET height = {}, leave = true WHERE id = {}",
            height, id
        );
        db.update(&sql)
    }

    pub fn delete(db: &DStorage, fid: &i64) -> Result<usize> {
        let sql = format!("DELETE FROM members WHERE fid = {}", fid);
        db.delete(&sql)
    }

    pub async fn sync(
        base: &PathBuf,
        gid: &GroupId,
        db: &DStorage,
        fid: &i64,
        from: &i64,
        to: &i64,
    ) -> Result<(
        Vec<(i64, GroupId, PeerId, String, Vec<u8>)>,
        Vec<(i64, GroupId)>,
    )> {
        let sql = format!("SELECT id, height, fid, mid, addr, name, leave FROM members WHERE fid = {} AND height BETWEEN {} AND {}", fid, from, to);
        let matrix = db.query(&sql)?;
        let mut adds = vec![];
        let mut leaves = vec![];
        for values in matrix {
            let m = Self::from_values(values);
            if m.leave {
                leaves.push((m.height, m.m_id));
            } else {
                let mavatar = read_avatar(base, gid, &m.m_id).await.unwrap_or(vec![]);
                adds.push((m.height, m.m_id, m.m_addr, m.m_name, mavatar))
            }
        }
        Ok((adds, leaves))
    }
}
