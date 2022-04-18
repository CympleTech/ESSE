use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::primitives::{Peer, PeerId, Result};
use tdn::types::rpc::{json, RpcParam};
use tdn_storage::local::{DStorage, DsValue};

pub(crate) struct Device {
    pub id: i64,
    pub name: String,
    pub info: String,
    pub assist: PeerId,
    pub peer: Peer,
    pub lasttime: i64,
    pub online: bool,
}

impl Device {
    pub fn new(peer: Peer) -> Self {
        let start = SystemTime::now();
        let lasttime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        let assist = peer.id;
        Self {
            lasttime,
            assist,
            peer,
            id: 0,
            name: String::new(),
            info: String::new(),
            online: true,
        }
    }

    /// here is zero-copy and unwrap is safe. checked.
    fn from_values(mut v: Vec<DsValue>) -> Device {
        Device {
            lasttime: v.pop().unwrap().as_i64(),
            peer: Peer::from_string(v.pop().unwrap().as_str()).unwrap_or(Peer::default()),
            assist: PeerId::from_hex(v.pop().unwrap().as_str()).unwrap_or(PeerId::default()),
            info: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            id: v.pop().unwrap().as_i64(),
            online: false,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.name,
            self.info,
            self.assist.to_hex(),
            self.peer.to_string(),
            self.lasttime,
            if self.online { "1" } else { "0" },
        ])
    }

    /// load account devices.
    pub fn list(db: &DStorage) -> Result<Vec<Device>> {
        let matrix = db.query("SELECT id, name, info, assist, peer, lasttime FROM devices")?;
        let mut devices = vec![];
        for values in matrix {
            devices.push(Device::from_values(values));
        }
        Ok(devices)
    }

    pub fn _get(db: &DStorage, aid: &PeerId) -> Result<Option<Device>> {
        let mut matrix = db.query(&format!(
            "SELECT id, name, info, assist, peer, lasttime FROM devices WHERE assist = '{}'",
            aid.to_hex()
        ))?;
        if let Some(values) = matrix.pop() {
            Ok(Some(Device::from_values(values)))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!(
            "INSERT INTO devices (name, info, assist, peer, lasttime) VALUES ('{}', '{}', '{}', '{}', {})",
            self.name,
            self.info,
            self.assist.to_hex(),
            self.peer.to_string(),
            self.lasttime,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn update(db: &DStorage, id: i64, name: &str, info: &str) -> Result<usize> {
        let sql = format!(
            "UPDATE devices SET name='{}', info = '{}' WHERE id = {}",
            name, info, id
        );
        db.update(&sql)
    }
}
