use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{HandleResult, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use chat_types::{MessageType, NetworkMessage};

use crate::apps::chat::{from_network_message, raw_to_network_message, to_network_message as tnm};
use crate::storage::group_db;

use super::Member;

/// Group Chat Message Model.
pub(crate) struct Message {
    /// db auto-increment id.
    pub id: i64,
    /// group message consensus height.
    height: i64,
    /// group's db id.
    fid: i64,
    /// member's db id.
    pub mid: i64,
    /// message is mine.
    is_me: bool,
    /// message type.
    pub m_type: MessageType,
    /// message content.
    pub content: String,
    /// message is delivery.
    is_delivery: bool,
    /// message created time.
    pub datetime: i64,
}

impl Message {
    pub(crate) fn new_with_time(
        height: i64,
        fid: i64,
        mid: i64,
        is_me: bool,
        m_type: MessageType,
        content: String,
        datetime: i64,
    ) -> Message {
        Self {
            fid,
            mid,
            m_type,
            content,
            datetime,
            height,
            is_me,
            is_delivery: true,
            id: 0,
        }
    }

    /// here is zero-copy and unwrap is safe. checked.
    fn from_values(mut v: Vec<DsValue>) -> Message {
        Message {
            datetime: v.pop().unwrap().as_i64(),
            is_delivery: v.pop().unwrap().as_bool(),
            content: v.pop().unwrap().as_string(),
            m_type: MessageType::from_int(v.pop().unwrap().as_i64()),
            is_me: v.pop().unwrap().as_bool(),
            mid: v.pop().unwrap().as_i64(),
            fid: v.pop().unwrap().as_i64(),
            height: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.height,
            self.fid,
            self.mid,
            self.is_me,
            self.m_type.to_int(),
            self.content,
            self.is_delivery,
            self.datetime,
        ])
    }

    pub fn _get(db: &DStorage, id: &i64) -> Result<Message> {
        let mut matrix = db.query(&format!("SELECT id, height, fid, mid, is_me, m_type, content, is_delivery, datetime FROM messages WHERE id = {}", id))?;
        if matrix.len() > 0 {
            Ok(Message::from_values(matrix.pop().unwrap())) // safe unwrap.
        } else {
            Err(anyhow!("missing member"))
        }
    }

    pub fn list(db: &DStorage, fid: &i64) -> Result<Vec<Message>> {
        let matrix = db.query(&format!("SELECT id, height, fid, mid, is_me, m_type, content, is_delivery, datetime FROM messages WHERE fid = {}", fid))?;
        let mut groups = vec![];
        for values in matrix {
            groups.push(Message::from_values(values));
        }
        Ok(groups)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from messages WHERE fid = {} AND height = {}",
            self.fid, self.height
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
        } else {
            let sql = format!("INSERT INTO messages (height, fid, mid, is_me, m_type, content, is_delivery, datetime) VALUES ({}, {}, {}, {}, {}, '{}', {}, {})",
                self.height,
                self.fid,
                self.mid,
                self.is_me,
                self.m_type.to_int(),
                self.content,
                self.is_delivery,
                self.datetime,
            );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn delete(db: &DStorage, fid: &i64) -> Result<usize> {
        let sql = format!("DELETE FROM messages WHERE fid = {}", fid);
        db.delete(&sql)
    }

    pub async fn sync(
        base: &PathBuf,
        gid: &GroupId,
        db: &DStorage,
        fid: &i64,
        height: &i64,
    ) -> Result<Vec<(i64, GroupId, NetworkMessage, i64)>> {
        let sql = format!("SELECT id, mid FROM members WHERE fid = {}", fid);
        let m = db.query(&sql)?;
        let mut members = HashMap::new();
        for mut v in m {
            let m_s = v.pop().unwrap().as_string();
            let mid = GroupId::from_hex(m_s).unwrap_or(Default::default());
            let id = v.pop().unwrap().as_i64();
            members.insert(id, mid);
        }

        let sql = format!("SELECT id, height, fid, mid, is_me, m_type, content, is_delivery, datetime FROM messages WHERE fid = {} AND height >= {}", fid, height);
        let matrix = db.query(&sql)?;
        let mut messages = vec![];
        for values in matrix {
            let msg = Message::from_values(values);
            if let Ok(nmsg) = tnm(base, gid, msg.m_type, msg.content).await {
                let mid = members.get(&msg.mid).cloned().unwrap_or(GroupId::default());
                messages.push((msg.height, mid, nmsg, msg.datetime))
            }
        }

        Ok(messages)
    }
}

pub(crate) async fn to_network_message(
    base: &PathBuf,
    gid: &GroupId,
    mtype: MessageType,
    content: &str,
) -> Result<(NetworkMessage, i64, String)> {
    let start = SystemTime::now();
    let datetime = start
        .duration_since(UNIX_EPOCH)
        .map(|s| s.as_secs())
        .unwrap_or(0) as i64; // safe for all life.

    let (nmsg, raw) = raw_to_network_message(base, gid, &mtype, content).await?;
    Ok((nmsg, datetime, raw))
}

pub(crate) fn handle_network_message(
    height: i64,
    gdid: i64,
    mid: GroupId,
    mgid: &GroupId,
    msg: NetworkMessage,
    datetime: i64,
    base: &PathBuf,
    results: &mut HandleResult,
) -> Result<Message> {
    let db = group_db(base, mgid)?;
    let mdid = Member::get_id(&db, &gdid, &mid)?;
    let is_me = &mid == mgid;
    let (m_type, raw) = from_network_message(msg, base, mgid, results)?;
    let mut msg = Message::new_with_time(height, gdid, mdid, is_me, m_type, raw, datetime);
    msg.insert(&db)?;
    Ok(msg)
}
