use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{new_io_error, PeerAddr, Result},
};

use tdn_did::Keypair;

use crate::models::device::Device;
use crate::storage::consensus_db;

pub(crate) struct RunningAccount {
    /// secret keypair.
    pub keypair: Keypair,
    /// device's name.
    pub device_name: String,
    /// device's info.
    pub device_info: String,
    /// distribute connected devices.
    pub distributes: HashMap<PeerAddr, (i64, bool)>,
    /// uptime
    pub uptime: u32,
}

impl RunningAccount {
    pub fn init(keypair: Keypair, base: &PathBuf, gid: &GroupId) -> Result<Self> {
        // load devices to runnings.
        let db = consensus_db(base, gid)?;
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

    pub fn add_online(&mut self, addr: &PeerAddr) -> Result<i64> {
        if let Some(v) = self.distributes.get_mut(addr) {
            v.1 = true;
            Ok(v.0)
        } else {
            Err(new_io_error("device missing"))
        }
    }

    pub fn offline(&mut self, addr: &PeerAddr) -> Result<i64> {
        if let Some(v) = self.distributes.get_mut(addr) {
            v.1 = false;
            Ok(v.0)
        } else {
            Err(new_io_error("device missing"))
        }
    }
}
