use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tdn::{
    smol::lock::RwLock,
    types::{
        group::GroupId,
        message::SendType,
        primitive::{new_io_error, PeerAddr, Result},
    },
};

use crate::apps::chat::chat_conn;
use crate::apps::group_chat::{group_chat_conn, GROUP_ID};
use crate::group::Group;
use crate::session::{Session, SessionType};
use crate::storage::session_db;

/// ESSE app's BaseLayerEvent.
/// EVERY LAYER APP MUST EQUAL THE FIRST THREE FIELDS.
#[derive(Serialize, Deserialize)]
pub(crate) enum LayerEvent {
    /// offline, remote_gid.
    Offline(GroupId),
}

/// ESSE layers.
pub(crate) struct Layer {
    /// account_gid => running_account.
    pub runnings: HashMap<GroupId, RunningAccount>,
    /// message delivery tracking. uuid, me_gid, db_id.
    pub delivery: HashMap<u64, (GroupId, i64)>,
    /// storage base path.
    pub base: PathBuf,
    /// self peer addr.
    pub addr: PeerAddr,
    /// group info.
    pub group: Arc<RwLock<Group>>,
}

impl Layer {
    pub async fn init(base: PathBuf, addr: PeerAddr, group: Arc<RwLock<Group>>) -> Result<Layer> {
        Ok(Layer {
            base,
            group,
            addr,
            runnings: HashMap::new(),
            delivery: HashMap::new(),
        })
    }

    pub fn base(&self) -> &PathBuf {
        &self.base
    }

    pub fn running(&self, gid: &GroupId) -> Result<&RunningAccount> {
        self.runnings.get(gid).ok_or(new_io_error("not online"))
    }

    pub fn running_mut(&mut self, gid: &GroupId) -> Result<&mut RunningAccount> {
        self.runnings.get_mut(gid).ok_or(new_io_error("not online"))
    }

    pub fn add_running(&mut self, gid: &GroupId) -> Result<()> {
        if !self.runnings.contains_key(gid) {
            self.runnings.insert(*gid, RunningAccount::init());
        }

        Ok(())
    }

    pub fn remove_running(&mut self, gid: &GroupId) -> HashMap<PeerAddr, GroupId> {
        // check close the stable connection.
        let mut addrs: HashMap<PeerAddr, GroupId> = HashMap::new();
        if let Some(running) = self.runnings.remove(gid) {
            for (addr, fgid) in running.remove_onlines() {
                addrs.insert(addr, fgid);
            }
        }

        let mut need_keep = vec![];
        for (_, running) in &self.runnings {
            for addr in addrs.keys() {
                if running.check_addr_online(addr) {
                    need_keep.push(*addr);
                }
            }
        }
        for i in need_keep {
            addrs.remove(&i);
        }

        addrs
    }

    pub fn remove_all_running(&mut self) -> HashMap<PeerAddr, GroupId> {
        let mut addrs: HashMap<PeerAddr, GroupId> = HashMap::new();
        for (_, running) in self.runnings.drain() {
            for (addr, fgid) in running.remove_onlines() {
                addrs.insert(addr, fgid);
            }
        }
        addrs
    }

    pub fn get_running_remote_id(&self, mgid: &GroupId, fgid: &GroupId) -> Result<i64> {
        self.running(mgid)?.get_online_id(fgid)
    }

    pub fn merge_online(&self, mgid: &GroupId, gids: Vec<&GroupId>) -> Result<Vec<bool>> {
        let runnings = self.running(mgid)?;
        Ok(gids.iter().map(|g| runnings.is_online(g)).collect())
    }

    pub fn remove_online(&mut self, gid: &GroupId, fgid: &GroupId) -> Option<PeerAddr> {
        self.running_mut(gid).ok()?.remove_online(fgid)
    }

    pub async fn all_layer_conns(&self) -> Result<HashMap<GroupId, Vec<(GroupId, SendType)>>> {
        let mut conns = HashMap::new();
        let group_lock = self.group.read().await;
        for mgid in self.runnings.keys() {
            let mut vecs = vec![];

            let db = session_db(&self.base, &mgid)?;
            let sessions = Session::list(&db)?;
            drop(db);

            for s in sessions {
                match s.s_type {
                    SessionType::Chat => {
                        let proof = group_lock.prove_addr(mgid, &s.addr)?;
                        vecs.push((s.gid, chat_conn(proof, s.addr)));
                    }
                    SessionType::Group => {
                        let proof = group_lock.prove_addr(mgid, &s.addr)?;
                        vecs.push((GROUP_ID, group_chat_conn(proof, s.addr, s.gid)));
                    }
                    _ => {}
                }
            }

            conns.insert(*mgid, vecs);
        }

        Ok(conns)
    }

    pub fn is_online(&self, faddr: &PeerAddr) -> bool {
        for (_, running) in &self.runnings {
            running.check_addr_online(faddr);
        }
        return false;
    }
}

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
    /// online group (friends/services) => (group's address, group's db id)
    onlines: HashMap<GroupId, (Online, i64)>,
}

impl RunningAccount {
    pub fn init() -> Self {
        RunningAccount {
            onlines: HashMap::new(),
        }
    }

    pub fn get_online_id(&self, gid: &GroupId) -> Result<i64> {
        self.onlines
            .get(gid)
            .map(|(_, id)| *id)
            .ok_or(new_io_error("remote not online"))
    }

    /// get all onlines's groupid
    pub fn is_online(&self, gid: &GroupId) -> bool {
        self.onlines.contains_key(gid)
    }

    /// get online peer's addr.
    pub fn online(&self, gid: &GroupId) -> Result<PeerAddr> {
        self.onlines
            .get(gid)
            .map(|(online, _)| *online.addr())
            .ok_or(new_io_error("remote not online"))
    }

    pub fn online_direct(&self, gid: &GroupId) -> Result<PeerAddr> {
        if let Some((online, _)) = self.onlines.get(gid) {
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
            .map(|(fgid, (online, _))| (fgid, online.addr()))
            .collect()
    }

    /// check add online.
    pub fn check_add_online(&mut self, gid: GroupId, online: Online, id: i64) -> Result<()> {
        if let Some((o, _)) = self.onlines.get(&gid) {
            match (o, &online) {
                (Online::Relay(..), Online::Direct(..)) => {
                    self.onlines.insert(gid, (online, id));
                    Ok(())
                }
                _ => Err(new_io_error("remote had online")),
            }
        } else {
            self.onlines.insert(gid, (online, id));
            Ok(())
        }
    }

    /// check offline, and return is direct.
    pub fn check_offline(&mut self, gid: &GroupId, addr: &PeerAddr) -> bool {
        if let Some((online, _)) = self.onlines.remove(gid) {
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

    pub fn remove_online(&mut self, gid: &GroupId) -> Option<PeerAddr> {
        self.onlines.remove(gid).map(|(online, _)| *online.addr())
    }

    /// remove all onlines peer.
    pub fn remove_onlines(self) -> Vec<(PeerAddr, GroupId)> {
        let mut peers = vec![];
        for (fgid, (online, _)) in self.onlines {
            match online {
                Online::Direct(addr) => peers.push((addr, fgid)),
                _ => {}
            }
        }
        peers
    }

    /// check if addr is online.
    pub fn check_addr_online(&self, addr: &PeerAddr) -> bool {
        for (_, (online, _)) in &self.onlines {
            if online.addr() == addr {
                return true;
            }
        }
        false
    }

    /// peer leave, remove online peer.
    pub fn peer_leave(&mut self, addr: &PeerAddr) -> Vec<(GroupId, i64)> {
        let mut peers = vec![];
        for (fgid, (online, id)) in &self.onlines {
            if online.addr() == addr {
                peers.push((*fgid, *id))
            }
        }

        for i in &peers {
            self.onlines.remove(&i.0);
        }
        peers
    }

    /// list all onlines groups.
    pub fn _list_onlines(&self) -> Vec<(&GroupId, &PeerAddr)> {
        self.onlines
            .iter()
            .map(|(k, (v, _))| (k, v.addr()))
            .collect()
    }
}
