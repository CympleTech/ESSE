use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tdn::{
    smol::lock::RwLock,
    types::{
        group::GroupId,
        message::{RecvType, SendType},
        primitive::{new_io_error, HandleResult, PeerAddr, Result},
    },
};
use tdn_did::user::User;

use crate::apps::chat::conn_req_message;
use crate::apps::chat::Friend;
use crate::group::Group;
use crate::storage::{session_db, write_avatar_sync};

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
            self.runnings
                .insert(*gid, RunningAccount::init(&self.base, gid)?);
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

    pub fn get_remote_id(&self, mgid: &GroupId, fgid: &GroupId) -> Result<i64> {
        self.running(mgid)?.get_permissioned(fgid)
    }

    pub fn all_friends(&self, gid: &GroupId) -> Result<Vec<Friend>> {
        let db = session_db(&self.base, &gid)?;
        let friends = Friend::all_ok(&db)?;
        drop(db);
        Ok(friends)
    }

    pub fn all_friends_with_online(&self, gid: &GroupId) -> Result<Vec<Friend>> {
        let db = session_db(&self.base, &gid)?;
        let mut friends = Friend::all(&db)?;
        drop(db);

        let keys: HashMap<GroupId, usize> = friends
            .iter()
            .enumerate()
            .map(|(i, f)| (f.gid, i))
            .collect();

        for fgid in self.running(gid)?.online_groups() {
            friends[keys[fgid]].online = true; // safe vec index.
        }

        Ok(friends)
    }

    pub fn update_friend(&self, gid: &GroupId, fid: i64, remote: User) -> Result<Friend> {
        let db = session_db(&self.base, &gid)?;
        if let Some(mut friend) = Friend::get_id(&db, fid)? {
            friend.name = remote.name;
            friend.addr = remote.addr;
            friend.remote_update(&db)?;
            drop(db);
            write_avatar_sync(&self.base, gid, &remote.id, remote.avatar)?;
            Ok(friend)
        } else {
            drop(db);
            Err(new_io_error("missing friend id"))
        }
    }

    pub fn remove_friend(&mut self, gid: &GroupId, fgid: &GroupId) -> Option<PeerAddr> {
        self.running_mut(gid).ok()?.remove_permissioned(fgid)
    }

    pub async fn all_friend_conns(&self) -> HashMap<GroupId, Vec<(GroupId, SendType)>> {
        let mut conns = HashMap::new();
        for mgid in self.runnings.keys() {
            if let Ok(friends) = self.all_friends(mgid) {
                let mut vecs = vec![];
                for friend in friends {
                    if let Ok(msg) = conn_req_message(self, &friend.gid, friend.addr).await {
                        vecs.push((friend.gid, msg));
                    }
                }
                conns.insert(*mgid, vecs);
            }
        }
        conns
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
