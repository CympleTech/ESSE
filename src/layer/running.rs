use std::collections::HashMap;
use std::path::PathBuf;
use tdn::types::{
    group::GroupId,
    primitive::{new_io_error, PeerAddr, Result},
};

use crate::models::session::Friend;
use crate::storage::session_db;

/// online info.
pub enum Online {
    /// connected to this device.
    Direct(PeerAddr),
    /// connected to other device.
    Relay(PeerAddr),
}

impl Online {
    fn addr(&self) -> &PeerAddr {
        match self {
            Online::Direct(ref addr) | Online::Relay(ref addr) => addr,
        }
    }
}

pub(crate) struct RunningAccount {
    permissioned: HashMap<GroupId, i64>,
    /// online group (friends/services) => group's address.
    onlines: HashMap<GroupId, Online>,
}

impl RunningAccount {
    pub fn init(base: &PathBuf, gid: &GroupId) -> Result<Self> {
        let mut permissioned = HashMap::new();

        // load friends to cache.
        let db = session_db(base, gid)?;
        let friends = Friend::all_id(&db)?;
        for (fgid, db_id) in friends {
            permissioned.insert(fgid, db_id);
        }

        // TODO load services to cache.

        // TODO load permissioned
        Ok(RunningAccount {
            permissioned,
            onlines: HashMap::new(),
        })
    }

    /// get all onlines's groupid
    pub fn online_groups(&self) -> Vec<&GroupId> {
        self.onlines.keys().map(|k| k).collect()
    }

    /// get online peer's addr.
    pub fn online(&self, gid: &GroupId) -> Result<PeerAddr> {
        self.onlines
            .get(gid)
            .map(|online| *online.addr())
            .ok_or(new_io_error("remote not online"))
    }

    pub fn online_direct(&self, gid: &GroupId) -> Result<PeerAddr> {
        if let Some(online) = self.onlines.get(gid) {
            match online {
                Online::Direct(addr) => return Ok(*addr),
                _ => {}
            }
        }
        Err(new_io_error("no direct online"))
    }

    /// get all online peer.
    pub fn onlines(&self) -> Vec<(&GroupId, &PeerAddr)> {
        self.onlines
            .iter()
            .map(|(fgid, online)| (fgid, online.addr()))
            .collect()
    }

    /// check add online.
    pub fn check_add_online(&mut self, gid: GroupId, online: Online) -> Result<()> {
        if let Some(o) = self.onlines.get(&gid) {
            match (o, &online) {
                (Online::Relay(..), Online::Direct(..)) => {
                    self.onlines.insert(gid, online);
                    Ok(())
                }
                _ => Err(new_io_error("remote had online")),
            }
        } else {
            self.onlines.insert(gid, online);
            Ok(())
        }
    }

    /// check offline, and return is direct.
    pub fn check_offline(&mut self, gid: &GroupId, addr: &PeerAddr) -> bool {
        if let Some(online) = self.onlines.remove(gid) {
            if online.addr() != addr {
                return false;
            }

            match online {
                Online::Direct(..) => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    /// remove all onlines peer.
    pub fn remove_onlines(self) -> Vec<(PeerAddr, GroupId)> {
        let mut peers = vec![];
        for (fgid, online) in self.onlines {
            match online {
                Online::Direct(addr) => peers.push((addr, fgid)),
                _ => {}
            }
        }
        peers
    }

    /// check if addr is online.
    pub fn check_addr_online(&self, addr: &PeerAddr) -> bool {
        for (_, online) in &self.onlines {
            if online.addr() == addr {
                return true;
            }
        }
        false
    }

    /// peer leave, remove online peer.
    pub fn peer_leave(&mut self, addr: &PeerAddr) -> Vec<(GroupId, i64)> {
        let mut peers = vec![];
        for (fgid, online) in &self.onlines {
            if online.addr() == addr {
                if let Some(i) = self.permissioned.get(fgid) {
                    peers.push((*fgid, *i))
                }
            }
        }

        for i in &peers {
            self.onlines.remove(&i.0);
        }
        peers
    }

    /// add the permissioned group.
    pub fn add_permissioned(&mut self, gid: GroupId, id: i64) {
        self.permissioned.insert(gid, id);
    }

    /// remove the permissioned group.
    pub fn remove_permissioned(&mut self, gid: &GroupId) -> Option<PeerAddr> {
        self.permissioned.remove(gid);
        self.onlines.remove(gid).and_then(|o| match o {
            Online::Direct(addr) => Some(addr),
            _ => None,
        })
    }

    /// check the group is permissioned.
    pub fn get_permissioned(&self, gid: &GroupId) -> Result<i64> {
        self.permissioned
            .get(gid)
            .cloned()
            .ok_or(new_io_error("remote missing"))
    }

    /// list all onlines groups.
    pub fn _list_onlines(&self) -> Vec<(&GroupId, &PeerAddr)> {
        self.onlines.iter().map(|(k, v)| (k, v.addr())).collect()
    }
}
