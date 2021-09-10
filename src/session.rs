use std::path::PathBuf;
use tdn::types::{
    group::GroupId,
    primitive::{PeerAddr, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use crate::storage::session_db;

pub(crate) enum SessionType {
    Chat,
    Group,
    Files,
    Device,
    Assistant,
    Domain,
    Service,
}

impl SessionType {
    pub fn to_int(&self) -> i64 {
        match self {
            SessionType::Chat => 0,
            SessionType::Group => 1,
            SessionType::Files => 2,
            SessionType::Device => 3,
            SessionType::Assistant => 4,
            SessionType::Domain => 5,
            SessionType::Service => 6,
        }
    }

    fn from_int(i: i64) -> Self {
        match i {
            0 => SessionType::Chat,
            1 => SessionType::Group,
            2 => SessionType::Files,
            3 => SessionType::Device,
            4 => SessionType::Assistant,
            5 => SessionType::Domain,
            6 => SessionType::Service,
            _ => SessionType::Chat,
        }
    }
}

pub(crate) struct Session {
    pub id: i64,
    fid: i64,
    pub gid: GroupId,
    pub addr: PeerAddr,
    pub s_type: SessionType,
    name: String,
    is_top: bool,
    is_close: bool,
    pub last_datetime: i64,
    pub last_content: String,
    pub last_readed: bool,
}

impl Session {
    pub fn new(
        fid: i64,
        gid: GroupId,
        addr: PeerAddr,
        s_type: SessionType,
        name: String,
        datetime: i64,
    ) -> Self {
        Self {
            fid,
            gid,
            addr,
            s_type,
            name,
            id: 0,
            is_top: false,
            is_close: false,
            last_datetime: datetime,
            last_content: "".to_owned(),
            last_readed: true,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.fid,
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.s_type.to_int(),
            self.name,
            self.is_top,
            self.is_close,
            self.last_datetime,
            self.last_content,
            self.last_readed,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            last_readed: v.pop().unwrap().as_bool(),
            last_content: v.pop().unwrap().as_string(),
            last_datetime: v.pop().unwrap().as_i64(),
            is_close: v.pop().unwrap().as_bool(),
            is_top: v.pop().unwrap().as_bool(),
            name: v.pop().unwrap().as_string(),
            s_type: SessionType::from_int(v.pop().unwrap().as_i64()),
            addr: PeerAddr::from_hex(v.pop().unwrap().as_str()).unwrap_or(PeerAddr::default()),
            gid: GroupId::from_hex(v.pop().unwrap().as_str()).unwrap_or(GroupId::default()),
            fid: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from sessions WHERE fid = {} AND s_type = {}",
            self.fid,
            self.s_type.to_int()
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;

            let sql = format!("UPDATE sessions SET gid = '{}', addr='{}', name = '{}', is_top = '{}', is_close = false WHERE id = {}",
                self.gid.to_hex(),
                self.addr.to_hex(),
                self.name,
                self.is_top,
                self.id,
            );
            db.update(&sql)?;
        } else {
            let sql = format!("INSERT INTO sessions (fid, gid, addr, s_type, name, is_top, is_close, last_datetime, last_content, last_readed) VALUES ({}, '{}', '{}', {}, '{}', {}, {}, {}, '{}', {})",
            self.fid,
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.s_type.to_int(),
            self.name,
            self.is_top,
            self.is_close,
            self.last_datetime,
            self.last_content,
            self.last_readed,
        );
            let id = db.insert(&sql)?;
            self.id = id;
        }

        Ok(())
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<Session> {
        let sql = format!("SELECT id, fid, gid, addr, s_type, name, is_top, is_close, last_datetime, last_content, last_readed FROM sessions WHERE id = {}", id);
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Session::from_values(matrix.pop().unwrap())) // safe unwrap()
        } else {
            Err(anyhow!("session missing."))
        }
    }

    pub fn list(db: &DStorage) -> Result<Vec<Session>> {
        let matrix = db.query("SELECT id, fid, gid, addr, s_type, name, is_top, is_close, last_datetime, last_content, last_readed FROM sessions ORDER BY last_datetime DESC")?;
        let mut sessions = vec![];
        for values in matrix {
            sessions.push(Session::from_values(values));
        }
        Ok(sessions)
    }

    pub fn update(db: &DStorage, id: &i64, is_top: bool, is_close: bool) -> Result<usize> {
        db.update(&format!(
            "UPDATE sessions SET is_top = {}, is_close = {} WHERE id = {}",
            is_top, is_close, id
        ))
    }

    pub fn delete(db: &DStorage, fid: &i64, s_type: &SessionType) -> Result<i64> {
        let sql = format!(
            "SELECT id from sessions WHERE fid = {} AND s_type = {}",
            fid,
            s_type.to_int()
        );
        let mut matrix = db.query(&sql)?;
        if let Some(mut values) = matrix.pop() {
            let id = values.pop().unwrap().as_i64(); // safe unwrap.
            db.delete(&format!("DELETE FROM sessions WHERE id = {}", id))?;
            Ok(id)
        } else {
            Err(anyhow!("session missing"))
        }
    }

    pub fn close(db: &DStorage, fid: &i64, s_type: &SessionType) -> Result<i64> {
        let sql = format!(
            "SELECT id from sessions WHERE fid = {} AND s_type = {}",
            fid,
            s_type.to_int()
        );
        let mut matrix = db.query(&sql)?;
        if let Some(mut values) = matrix.pop() {
            let id = values.pop().unwrap().as_i64(); // safe unwrap.
            let s = format!("UPDATE sessions SET is_close = 1 WHERE id = {}", id);
            db.update(&s)?;
            Ok(id)
        } else {
            Err(anyhow!("session missing"))
        }
    }

    pub fn last(
        db: &DStorage,
        fid: &i64,
        s_type: &SessionType,
        datetime: &i64,
        content: &str,
        readed: bool,
    ) -> Result<i64> {
        let sql = format!(
            "SELECT id from sessions WHERE fid = {} AND s_type = {}",
            fid,
            s_type.to_int()
        );
        let mut matrix = db.query(&sql)?;

        if let Some(mut values) = matrix.pop() {
            let id = values.pop().unwrap().as_i64();
            db.update(&format!("UPDATE sessions SET is_close = false, last_datetime = {}, last_content = '{}', last_readed = {} WHERE id = {}", datetime, content, if readed { 1 } else { 0 }, id))?;
            Ok(id)
        } else {
            Err(anyhow!("session missing"))
        }
    }

    pub fn readed(db: &DStorage, id: &i64) -> Result<usize> {
        db.update(&format!(
            "UPDATE sessions SET last_readed = 1 WHERE id = {}",
            id
        ))
    }
}

#[inline]
pub(crate) fn connect_session(
    base: &PathBuf,
    mgid: &GroupId,
    s_type: &SessionType,
    fid: &i64,
    addr: &PeerAddr,
) -> Result<Option<Session>> {
    let db = session_db(base, mgid)?;

    let sql = format!("SELECT id, fid, gid, addr, s_type, name, is_top, is_close, last_datetime, last_content, last_readed FROM sessions WHERE s_type = {} AND fid = {}", s_type.to_int(), fid);

    let mut matrix = db.query(&sql)?;
    if matrix.len() > 0 {
        let session = Session::from_values(matrix.pop().unwrap()); // safe unwrap()

        let _ = db.update(&format!(
            "UPDATE sessions SET addr = '{}' WHERE id = {}",
            addr.to_hex(),
            session.id,
        ));

        Ok(Some(session))
    } else {
        Ok(None)
    }
}
