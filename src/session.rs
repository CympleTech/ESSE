use tdn::types::{
    group::GroupId,
    primitive::{new_io_error, PeerAddr, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

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
    fn to_int(&self) -> i64 {
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
    id: i64,
    fid: i64,
    pub gid: GroupId,
    pub addr: PeerAddr,
    pub s_type: SessionType,
    name: String,
    is_top: bool,
    last_datetime: i64,
    last_content: String,
    last_readed: bool,
    pub online: bool,
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
            last_datetime: datetime,
            last_content: "".to_owned(),
            last_readed: true,
            online: false,
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
            self.last_datetime,
            self.last_content,
            self.last_readed,
            self.online
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            last_readed: v.pop().unwrap().as_bool(),
            last_content: v.pop().unwrap().as_string(),
            last_datetime: v.pop().unwrap().as_i64(),
            is_top: v.pop().unwrap().as_bool(),
            name: v.pop().unwrap().as_string(),
            s_type: SessionType::from_int(v.pop().unwrap().as_i64()),
            addr: PeerAddr::from_hex(v.pop().unwrap().as_str()).unwrap_or(PeerAddr::default()),
            gid: GroupId::from_hex(v.pop().unwrap().as_str()).unwrap_or(GroupId::default()),
            fid: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
            online: false,
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!("INSERT INTO sessions (fid, gid, addr, s_type, name, is_top, last_datetime, last_content, last_readed) VALUES ({}, '{}', '{}', {}, '{}', {}, {}, '{}', {})",
            self.fid,
            self.gid.to_hex(),
            self.addr.to_hex(),
            self.s_type.to_int(),
            self.name,
            if self.is_top { 1 } else { 0 },
            self.last_datetime,
            self.last_content,
            if self.last_readed { 1 } else { 0 },
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn get(db: &DStorage, id: i64) -> Result<Session> {
        let sql = format!("SELECT id, fid, gid, addr, s_type, name, is_top, last_datetime, last_content, last_readed FROM sessions WHERE id = {}", id);
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Session::from_values(matrix.pop().unwrap())) // safe unwrap()
        } else {
            Err(new_io_error("session missing."))
        }
    }

    pub fn list(db: &DStorage) -> Result<Vec<Session>> {
        let matrix = db.query("SELECT id, fid, gid, addr, s_type, name, is_top, last_datetime, last_content, last_readed FROM sessions ORDER BY last_datetime DESC")?;
        let mut sessions = vec![];
        for values in matrix {
            sessions.push(Session::from_values(values));
        }
        Ok(sessions)
    }

    pub fn top(db: &DStorage, id: &i64, is_top: bool) -> Result<usize> {
        db.update(&format!("UPDATE sessions SET is_top = 1 WHERE id = {}", id))
    }

    pub fn last(
        db: &DStorage,
        id: &i64,
        datetime: &i64,
        content: &str,
        readed: bool,
    ) -> Result<usize> {
        db.update(&format!("UPDATE sessions SET last_datetime = {}, last_content = '{}', last_readed = {} WHERE id = {}", datetime, content, if readed { 1 } else { 0 }, id))
    }

    pub fn read(db: &DStorage, id: &i64) -> Result<usize> {
        db.update(&format!(
            "UPDATE sessions SET last_readed = 1 WHERE id = {}",
            id
        ))
    }
}
