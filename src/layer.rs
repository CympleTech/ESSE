use esse_primitives::id_to_str;
use group_types::GroupChatId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tdn::types::{
    message::SendType,
    primitives::{HandleResult, Peer, PeerId, Result},
};
use tokio::sync::RwLock;

use crate::account::User;
use crate::group::GroupEvent;
//use crate::apps::group::{group_conn, GROUP_ID};
use crate::own::Own;
use crate::session::{Session, SessionType};

/// ESSE layers.
pub(crate) struct Layer {
    /// group chat id => Session
    pub groups: HashMap<GroupChatId, LayerSession>,
    /// delivery feedback.
    pub delivery: HashMap<u64, i64>,
    /// delivery counter.
    delivery_count: usize,
}

impl Layer {
    pub fn init() -> Layer {
        Layer {
            groups: HashMap::new(),
            delivery: HashMap::new(),
            delivery_count: 0,
        }
    }

    pub fn delivery(&mut self, db_id: i64) -> u64 {
        let next = self.delivery_count as u64;
        self.delivery.insert(next, db_id);
        self.delivery_count += 1;
        next
    }

    pub fn clear(&mut self) {
        self.groups.clear();
        self.delivery.clear();
    }

    pub fn is_addr_online(&self, addr: &PeerId) -> bool {
        for (_, session) in &self.groups {
            if session.addrs.contains(addr) {
                return true;
            }
        }

        false
    }

    pub fn group_active(&mut self, gid: &GroupChatId, is_me: bool) -> Option<PeerId> {
        if let Some(session) = self.groups.get_mut(gid) {
            Some(session.active(is_me))
        } else {
            None
        }
    }

    pub fn group_suspend(&mut self, g: &GroupChatId, me: bool, m: bool) -> Result<Option<PeerId>> {
        if let Some(session) = self.groups.get_mut(g) {
            Ok(session.suspend(me, m))
        } else {
            Err(anyhow!("session missing!"))
        }
    }

    pub fn group(&self, gid: &GroupChatId) -> Result<&LayerSession> {
        if let Some(session) = self.groups.get(gid) {
            Ok(session)
        } else {
            Err(anyhow!("session missing!"))
        }
    }

    pub fn group_mut(&mut self, gid: &GroupChatId) -> Result<&mut LayerSession> {
        if let Some(session) = self.groups.get_mut(gid) {
            Ok(session)
        } else {
            Err(anyhow!("session missing!"))
        }
    }

    pub fn group_add(&mut self, gid: GroupChatId, pid: PeerId, sid: i64, fid: i64, h: i64) {
        if !self.groups.contains_key(&gid) {
            self.groups.insert(gid, LayerSession::new(pid, sid, fid, h));
        }
    }

    pub fn group_del(&mut self, gid: &GroupChatId) -> Option<Vec<PeerId>> {
        self.groups.remove(gid).map(|session| session.addrs)
    }

    pub fn group_add_member(&mut self, gid: &GroupChatId, addr: PeerId) {
        if let Some(session) = self.groups.get_mut(gid) {
            session.addrs.push(addr);
        }
    }

    pub fn group_del_member(&mut self, gid: &GroupChatId, index: usize) {
        if let Some(session) = self.groups.get_mut(gid) {
            session.addrs.remove(index);
        }
    }

    pub fn group_del_online(&mut self, gid: &GroupChatId, addr: &PeerId) -> bool {
        if let Some(session) = self.groups.get_mut(gid) {
            if let Some(pos) = session.addrs.iter().position(|x| x == addr) {
                session.addrs.remove(pos);
                return true;
            }
        }
        false
    }

    // pub fn remove_running(&mut self, gid: &GroupId) -> HashMap<PeerId, GroupId> {
    //     // check close the stable connection.
    //     let mut addrs: HashMap<PeerId, GroupId> = HashMap::new();
    //     if let Some(running) = self.runnings.remove(gid) {
    //         for (addr, fgid) in running.remove_onlines() {
    //             addrs.insert(addr, fgid);
    //         }
    //     }

    //     let mut need_keep = vec![];
    //     for (_, running) in &self.runnings {
    //         for addr in addrs.keys() {
    //             if running.check_addr_online(addr) {
    //                 need_keep.push(*addr);
    //             }
    //         }
    //     }
    //     for i in need_keep {
    //         addrs.remove(&i);
    //     }

    //     addrs
    // }

    // pub fn remove_all_running(&mut self) -> HashMap<PeerId, GroupId> {
    //     let mut addrs: HashMap<PeerId, GroupId> = HashMap::new();
    //     for (_, running) in self.runnings.drain() {
    //         for (addr, fgid) in running.remove_onlines() {
    //             addrs.insert(addr, fgid);
    //         }
    //     }
    //     addrs
    // }

    // pub fn get_running_remote_id(&self, mgid: &GroupId, fgid: &GroupId) -> Result<(i64, i64)> {
    //     debug!("onlines: {:?}, find: {:?}", self.runnings.keys(), mgid);
    //     self.running(mgid)?.get_online_id(fgid)
    // }

    // pub fn remove_online(&mut self, gid: &GroupId, fgid: &GroupId) -> Option<PeerId> {
    //     self.running_mut(gid).ok()?.remove_online(fgid)
    // }

    // pub async fn all_layer_conns(&self) -> Result<HashMap<GroupId, Vec<(GroupId, SendType)>>> {
    //     let mut conns = HashMap::new();
    //     let own_lock = self.group.read().await;
    //     for mgid in self.runnings.keys() {
    //         let mut vecs = vec![];

    //         let db = own_lock.session_db(&mgid)?;
    //         let sessions = Session::list(&db)?;
    //         drop(db);

    //         for s in sessions {
    //             match s.s_type {
    //                 SessionType::Chat => {
    //                     let proof = own_lock.prove_addr(mgid, &s.addr)?;
    //                     vecs.push((s.gid, chat_conn(proof, Peer::peer(s.addr))));
    //                 }
    //                 SessionType::Group => {
    //                     let proof = own_lock.prove_addr(mgid, &s.addr)?;
    //                     vecs.push((GROUP_ID, group_conn(proof, Peer::peer(s.addr), s.gid)));
    //                 }
    //                 _ => {}
    //             }
    //         }

    //         conns.insert(*mgid, vecs);
    //     }

    //     Ok(conns)
    // }

    // pub fn is_addr_online(&self, faddr: &PeerId) -> bool {
    //     for (_, running) in &self.runnings {
    //         if running.check_addr_online(faddr) {
    //             return true;
    //         }
    //     }
    //     return false;
    // }

    // pub fn is_online(&self, gid: &GroupId, fgid: &GroupId) -> bool {
    //     if let Some(running) = self.runnings.get(gid) {
    //         running.is_online(fgid)
    //     } else {
    //         false
    //     }
    // }

    // pub fn broadcast(&self, user: User, results: &mut HandleResult) {
    //     let info = GroupEvent::InfoRes(user);
    //     let data = bincode::serialize(&info).unwrap_or(vec![]);

    //     // TODO GROUPS
    // }
}

// pub(crate) struct OnlineSession {
//     pub pid: PeerId,
//     /// session database id.
//     pub id: i64,
//     /// session ref's service(friend/group) database id.
//     pub fid: i64,
//     pub suspend_me: bool,
//     pub suspend_remote: bool,
//     pub remain: u16, // keep-alive remain minutes
// }

// impl OnlineSession {
//     fn new(online: Online, db_id: i64, db_fid: i64) -> Self {
//         Self {
//             online,
//             db_id,
//             db_fid,
//             suspend_me: false,
//             suspend_remote: false,
//             remain: 0,
//         }
//     }

//     fn close_suspend(&mut self) -> bool {
//         if self.suspend_me && self.suspend_remote {
//             if self.remain == 0 {
//                 true
//             } else {
//                 self.remain -= 1;
//                 false
//             }
//         } else {
//             false
//         }
//     }
// }

/// online connected layer session.
pub(crate) struct LayerSession {
    pub height: i64,
    /// session network addr.
    pub addrs: Vec<PeerId>,
    /// session database id.
    pub s_id: i64,
    /// layer service database id.
    pub db_id: i64,
    /// if session is suspend by me.
    pub suspend_me: bool,
    /// if session is suspend by remote.
    pub suspend_remote: bool,
    /// keep alive remain minutes.
    pub remain: u16,
}

impl LayerSession {
    fn new(addr: PeerId, s_id: i64, db_id: i64, height: i64) -> Self {
        Self {
            s_id,
            db_id,
            height,
            addrs: vec![addr],
            suspend_me: false,
            suspend_remote: false,
            remain: 0,
        }
    }

    pub fn info(&self) -> (i64, i64, i64, PeerId) {
        (self.height, self.s_id, self.db_id, self.addrs[0])
    }

    pub fn increased(&mut self) -> i64 {
        self.height += 1;
        self.height
    }

    pub fn active(&mut self, is_me: bool) -> PeerId {
        if is_me {
            self.suspend_me = false;
        } else {
            self.suspend_remote = false;
        }
        self.remain = 0;
        self.addrs[0]
    }

    pub fn suspend(&mut self, is_me: bool, must: bool) -> Option<PeerId> {
        if must {
            self.suspend_me = true;
            self.suspend_remote = true;
        }

        if is_me {
            self.suspend_me = true;
        } else {
            self.suspend_remote = true;
        }

        if self.suspend_remote && self.suspend_me {
            self.remain = 6; // keep-alive 10~11 minutes 120s/time
            Some(self.addrs[0])
        } else {
            None
        }
    }

    // pub fn get_online_id(&self, gid: &GroupId) -> Result<(i64, i64)> {
    //     debug!("onlines: {:?}, find: {:?}", self.sessions.keys(), gid);
    //     self.sessions
    //         .get(gid)
    //         .map(|online| (online.db_id, online.db_fid))
    //         .ok_or(anyhow!("remote not online"))
    // }

    // /// get online peer's addr.
    // pub fn online(&self, gid: &GroupId) -> Result<PeerId> {
    //     self.sessions
    //         .get(gid)
    //         .map(|online| *online.online.addr())
    //         .ok_or(anyhow!("remote not online"))
    // }

    // pub fn online_direct(&self, gid: &GroupId) -> Result<PeerId> {
    //     if let Some(online) = self.sessions.get(gid) {
    //         match online.online {
    //             Online::Direct(addr) => return Ok(addr),
    //             _ => {}
    //         }
    //     }
    //     Err(anyhow!("no direct online"))
    // }

    // /// get all online peer.
    // pub fn onlines(&self) -> Vec<(&GroupId, &PeerId)> {
    //     self.sessions
    //         .iter()
    //         .map(|(fgid, online)| (fgid, online.online.addr()))
    //         .collect()
    // }

    // /// check add online.

    // /// check offline, and return is direct.
    // pub fn check_offline(&mut self, gid: &GroupId, addr: &PeerId) -> bool {
    //     if let Some(online) = self.sessions.remove(gid) {
    //         if online.online.addr() != addr {
    //             return false;
    //         }

    //         match online.online {
    //             Online::Direct(..) => {
    //                 return true;
    //             }
    //             _ => {}
    //         }
    //     }
    //     false
    // }

    // pub fn remove_online(&mut self, gid: &GroupId) -> Option<PeerId> {
    //     self.sessions
    //         .remove(gid)
    //         .map(|online| *online.online.addr())
    // }

    // /// remove all onlines peer.
    // pub fn remove_onlines(self) -> Vec<(PeerId, GroupId)> {
    //     let mut peers = vec![];
    //     for (fgid, online) in self.sessions {
    //         match online.online {
    //             Online::Direct(addr) => peers.push((addr, fgid)),
    //             _ => {}
    //         }
    //     }
    //     peers
    // }

    // /// check if addr is online.
    // pub fn check_addr_online(&self, addr: &PeerId) -> bool {
    //     for (_, online) in &self.sessions {
    //         if online.online.addr() == addr {
    //             return true;
    //         }
    //     }
    //     false
    // }

    // /// peer leave, remove online peer.
    // pub fn peer_leave(&mut self, addr: &PeerId) -> Vec<i64> {
    //     let mut peers = vec![];
    //     let mut deletes = vec![];
    //     for (fgid, online) in &self.sessions {
    //         if online.online.addr() == addr {
    //             peers.push(online.db_id);
    //             deletes.push(*fgid);
    //         }
    //     }
    //     for i in &deletes {
    //         self.sessions.remove(&i);
    //     }

    //     peers
    // }

    // /// list all onlines groups.
    // pub fn close_suspend(&mut self, self_addr: &PeerId) -> Vec<(GroupId, PeerId, i64)> {
    //     let mut needed = vec![];
    //     for (fgid, online) in &mut self.sessions {
    //         // when online is self. skip.
    //         if online.online == Online::Direct(*self_addr) {
    //             continue;
    //         }

    //         if online.close_suspend() {
    //             needed.push((*fgid, *online.online.addr(), online.db_id));
    //         }
    //     }

    //     for (gid, _, _) in needed.iter() {
    //         self.sessions.remove(gid);
    //     }
    //     needed
    // }
}
