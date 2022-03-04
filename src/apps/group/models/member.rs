use esse_primitives::{id_from_str, id_to_str};
use std::path::PathBuf;
use tdn::types::{
    primitives::{PeerId, Result},
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
    /// member's Did(PeerId)
    pub pid: PeerId,
    /// member's name.
    pub name: String,
    /// if leave from group.
    pub leave: bool,
}

impl Member {
    pub fn new(height: i64, fid: i64, pid: PeerId, name: String) -> Self {
        Self {
            height,
            fid,
            pid,
            name,
            leave: false,
            id: 0,
        }
    }

    pub fn info(id: i64, fid: i64, pid: PeerId, name: String) -> Self {
        Self {
            id,
            fid,
            pid,
            name,
            leave: false,
            height: 0,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.fid,
            id_to_str(&self.pid),
            self.name,
            self.leave,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            leave: v.pop().unwrap().as_bool(),
            name: v.pop().unwrap().as_string(),
            pid: id_from_str(v.pop().unwrap().as_str()).unwrap_or(Default::default()),
            fid: v.pop().unwrap().as_i64(),
            height: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn list(db: &DStorage, fid: &i64) -> Result<Vec<Member>> {
        let matrix = db.query(&format!(
            "SELECT id, height, fid, pid, name, leave FROM members WHERE fid = {}",
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
            "SELECT id from members WHERE fid = {} AND pid = '{}'",
            self.fid,
            id_to_str(&self.pid)
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
            let sql = format!(
                "UPDATE members SET height = {}, name = '{}', leave = false WHERE id = {}",
                self.height, self.name, self.id,
            );
            db.update(&sql)?;
        } else {
            let sql = format!("INSERT INTO members (height, fid, pid, name, leave) VALUES ({}, {}, '{}', '{}', false)",
            self.height,
            self.fid,
            id_to_str(&self.pid),
            self.name,
        );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn _get(db: &DStorage, id: &i64) -> Result<Member> {
        let mut matrix = db.query(&format!(
            "SELECT id, height, fid, pid, name, leave FROM members WHERE id = {}",
            id,
        ))?;
        if matrix.len() > 0 {
            Ok(Self::from_values(matrix.pop().unwrap())) // safe unwrap.
        } else {
            Err(anyhow!("missing member"))
        }
    }

    pub fn get_id(db: &DStorage, fid: &i64, pid: &PeerId) -> Result<i64> {
        let mut matrix = db.query(&format!(
            "SELECT id FROM members WHERE fid = {} AND pid = '{}'",
            fid,
            id_to_str(pid)
        ))?;
        if matrix.len() > 0 {
            Ok(matrix.pop().unwrap().pop().unwrap().as_i64()) // safe unwrap.
        } else {
            Err(anyhow!("missing member"))
        }
    }

    pub fn update(db: &DStorage, id: &i64, height: &i64, name: &str) -> Result<usize> {
        let sql = format!(
            "UPDATE members SET height = {}, name='{}' WHERE id = {}",
            height, name, id,
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
        gid: &PeerId,
        db: &DStorage,
        fid: &i64,
        from: &i64,
        to: &i64,
    ) -> Result<(Vec<(i64, PeerId, String, Vec<u8>)>, Vec<(i64, PeerId)>)> {
        let sql = format!("SELECT id, height, fid, pid, name, leave FROM members WHERE fid = {} AND height BETWEEN {} AND {}", fid, from, to);
        let matrix = db.query(&sql)?;
        let mut adds = vec![];
        let mut leaves = vec![];
        for values in matrix {
            let m = Self::from_values(values);
            if m.leave {
                leaves.push((m.height, m.pid));
            } else {
                let mavatar = read_avatar(base, gid, &m.pid).await.unwrap_or(vec![]);
                adds.push((m.height, m.pid, m.name, mavatar))
            }
        }
        Ok((adds, leaves))
    }
}
