use esse_primitives::{MessageType, NetworkMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tdn::types::{
    group::EventId,
    message::{RecvType, SendType},
    primitives::{DeliveryType, HandleResult, Peer, PeerId, Result},
};
use tdn_storage::local::DStorage;

use crate::account::{Account, User};
use crate::global::Global;
use crate::rpc::{
    notice_menu, session_connect, session_create, session_last, session_lost, session_suspend,
    session_update_name,
};
use crate::session::{connect_session, Session, SessionType};
use crate::storage::{account_db, chat_db, session_db, write_avatar_sync};

mod handle;
mod models;
mod rpc;

pub(crate) use handle::{group_conn, group_handle, update_session};
pub(crate) use models::{
    from_model, from_network_message, handle_nmsg, raw_to_network_message, to_network_message,
    Friend, InviteType, Message, Request,
};
pub(crate) use rpc::group_rpc;

/// ESSE groups.
pub(crate) struct Group {
    /// friend pid => Session
    pub sessions: HashMap<PeerId, GroupSession>,
    /// delivery feedback.
    pub delivery: HashMap<u64, i64>,
    /// delivery counter.
    delivery_count: usize,
}

/// online connected layer session.
pub(crate) struct GroupSession {
    /// consensus height.
    pub height: i64,
    /// session database id.
    pub sid: i64,
    /// friend database id.
    pub fid: i64,
    /// if session is suspend by me.
    pub suspend_me: bool,
    /// if session is suspend by remote.
    pub suspend_remote: bool,
    /// keep alive remain minutes.
    pub remain: u16,
}

/// ESSE group Event (Chat).
#[derive(Serialize, Deserialize)]
pub(crate) enum GroupEvent {
    /// offline. extend BaseGroupEvent.
    Offline,
    /// suspend. extend BaseGroupEvent.
    Suspend,
    /// actived. extend BaseGroupEvent.
    Actived,
    /// make friendship request.
    /// params is name, remark.
    Request(String, String),
    /// agree friendship request.
    /// params is gid.
    Agree,
    /// reject friendship request.
    Reject,
    /// receiver gid, sender gid, message.
    Message(EventId, NetworkMessage),
    /// request user info.
    InfoReq(u64),
    /// user full info.
    InfoRes(User),
    /// close friendship.
    Close,
}

impl Group {
    pub fn init() -> Group {
        Group {
            sessions: HashMap::new(),
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
        self.sessions.clear();
        self.delivery.clear();
        self.delivery_count = 0;
    }

    pub fn add(&mut self, pid: PeerId, sid: i64, fid: i64, h: i64) {
        self.sessions
            .entry(pid)
            .and_modify(|s| {
                s.sid = sid;
                s.fid = fid;
                s.height = h;
            })
            .or_insert(GroupSession::new(sid, fid, h));
    }

    pub fn get(&self, pid: &PeerId) -> Result<(i64, i64)> {
        if let Some(session) = self.sessions.get(pid) {
            Ok((session.sid, session.fid))
        } else {
            Err(anyhow!("session missing!"))
        }
    }

    pub fn is_online(&self, pid: &PeerId) -> bool {
        self.sessions.contains_key(pid)
    }

    pub fn rm_online(&mut self, pid: &PeerId) -> bool {
        if self.sessions.contains_key(pid) {
            self.sessions.remove(pid);
            true
        } else {
            false
        }
    }

    pub fn active(&mut self, pid: &PeerId, is_me: bool) -> Result<()> {
        if let Some(session) = self.sessions.get_mut(pid) {
            Ok(session.active(is_me))
        } else {
            Err(anyhow!("session missing!"))
        }
    }

    pub fn suspend(&mut self, pid: &PeerId, me: bool, m: bool) -> Result<()> {
        if let Some(session) = self.sessions.get_mut(pid) {
            Ok(session.suspend(me, m))
        } else {
            Err(anyhow!("session missing!"))
        }
    }

    pub fn broadcast(&self, user: User, results: &mut HandleResult) {
        let info = GroupEvent::InfoRes(user);
        let data = bincode::serialize(&info).unwrap_or(vec![]);

        for fpid in self.sessions.keys() {
            let msg = SendType::Event(0, *fpid, data.clone());
            results.groups.push(msg);
        }
    }
}

impl GroupSession {
    fn new(sid: i64, fid: i64, height: i64) -> Self {
        Self {
            sid,
            fid,
            height,
            suspend_me: false,
            suspend_remote: false,
            remain: 0,
        }
    }

    pub fn info(&self) -> (i64, i64, i64) {
        (self.height, self.sid, self.fid)
    }

    pub fn increased(&mut self) -> i64 {
        self.height += 1;
        self.height
    }

    pub fn active(&mut self, is_me: bool) {
        if is_me {
            self.suspend_me = false;
        } else {
            self.suspend_remote = false;
        }
        self.remain = 0;
    }

    pub fn suspend(&mut self, is_me: bool, must: bool) {
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
        }
    }
}
