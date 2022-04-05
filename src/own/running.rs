use esse_primitives::id_to_str;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::primitives::{Peer, PeerId, PeerKey, Result};
use tdn_storage::local::DStorage;

use crate::apps::device::Device;
use crate::migrate::CONSENSUS_DB;

pub(crate) struct RunningAccount {
    /// secret keypair.
    pub keypair: PeerKey,
    /// device's name.
    pub device_name: String,
    /// device's info.
    pub device_info: String,
    /// distribute connected devices.
    pub distributes: Vec<(Peer, i64, bool)>,
    /// uptime
    pub uptime: u32,
}

impl RunningAccount {
    pub fn init(keypair: PeerKey, base: &PathBuf, key: &str, pid: &PeerId) -> Result<Self> {
        let mut db_path = base.clone();
        db_path.push(id_to_str(&pid));
        db_path.push(CONSENSUS_DB);
        let db = DStorage::open(db_path, key)?;
        let distributes = Device::distributes(&db)?;
        let (device_name, device_info) = Device::device_info(&db)?;
        db.close()?;

        let start = SystemTime::now();
        let uptime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as u32; // safe for all life.

        Ok(Self {
            keypair,
            distributes,
            device_name,
            device_info,
            uptime,
        })
    }

    pub fn online(&mut self, peer: &Peer) -> Result<i64> {
        for i in self.distributes.iter_mut() {
            if &i.0 == peer {
                i.2 = true;
                return Ok(i.1);
            }
        }
        Err(anyhow!("missing distribute device"))
    }

    pub fn offline(&mut self, peer: &Peer) -> Result<i64> {
        for i in self.distributes.iter_mut() {
            if &i.0 == peer {
                i.2 = false;
                return Ok(i.1);
            }
        }
        Err(anyhow!("missing distribute device"))
    }
}
