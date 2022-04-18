use group_types::GroupChatId;
use std::collections::HashMap;
use tdn::types::primitives::{PeerId, Result};

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

    // pub fn broadcast(&self, user: User, results: &mut HandleResult) {
    //     let info = GroupEvent::InfoRes(user);
    //     let data = bincode::serialize(&info).unwrap_or(vec![]);

    //     // TODO GROUPS
    // }
}

/// online connected layer session.
pub(crate) struct LayerSession {
    pub height: i64,
    /// session network addr.
    pub addrs: Vec<PeerId>,
    /// session database id.
    pub sid: i64,
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
    fn new(addr: PeerId, sid: i64, db_id: i64, height: i64) -> Self {
        Self {
            sid,
            db_id,
            height,
            addrs: vec![addr],
            suspend_me: false,
            suspend_remote: false,
            remain: 0,
        }
    }

    pub fn info(&self) -> (i64, i64, i64, PeerId) {
        (self.height, self.sid, self.db_id, self.addrs[0])
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

    pub fn clear(&mut self) -> bool {
        if self.suspend_me && self.suspend_remote {
            if self.remain == 0 {
                true
            } else {
                self.remain -= 1;
                false
            }
        } else {
            false
        }
    }
}
