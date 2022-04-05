use esse_primitives::{MessageType, NetworkMessage};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::EventId,
    primitives::{HandleResult, PeerId, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use super::{from_network_message, to_network_message};

pub(crate) async fn handle_nmsg(
    own: &PeerId,
    base: &PathBuf,
    db_key: &str,
    nmsg: NetworkMessage,
    is_me: bool,
    db: &DStorage,
    fid: i64,
    hash: EventId,
    results: &mut HandleResult,
) -> Result<Message> {
    // handle event.
    let (m_type, raw) = from_network_message(own, base, db_key, nmsg, results).await?;
    let mut msg = Message::new_with_id(hash, fid, is_me, m_type, raw, true);
    msg.insert(db)?;
    Ok(msg)
}

pub(crate) async fn from_model(
    own: &PeerId,
    base: &PathBuf,
    model: Message,
) -> Result<NetworkMessage> {
    to_network_message(own, base, model.m_type, model.content).await
}

pub(crate) struct Message {
    pub id: i64,
    pub hash: EventId,
    pub fid: i64,
    pub is_me: bool,
    pub m_type: MessageType,
    pub content: String,
    pub is_delivery: bool,
    pub datetime: i64,
}

impl Message {
    pub fn new(
        pid: &PeerId,
        fid: i64,
        is_me: bool,
        m_type: MessageType,
        content: String,
        is_delivery: bool,
    ) -> Message {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&pid.0[0..8]);
        bytes[8..16].copy_from_slice(&(fid as u64).to_le_bytes()); // 8-bytes.
        bytes[16..24].copy_from_slice(&(datetime as u64).to_le_bytes()); // 8-bytes.
        let content_bytes = content.as_bytes();
        if content_bytes.len() >= 8 {
            bytes[24..32].copy_from_slice(&content_bytes[0..8]);
        } else {
            bytes[24..(24 + content_bytes.len())].copy_from_slice(&content_bytes);
        }

        Message {
            id: 0,
            hash: EventId(bytes),
            fid,
            is_me,
            m_type,
            content,
            is_delivery,
            datetime,
        }
    }

    pub fn new_with_id(
        hash: EventId,
        fid: i64,
        is_me: bool,
        m_type: MessageType,
        content: String,
        is_delivery: bool,
    ) -> Message {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Message {
            id: 0,
            hash,
            fid,
            is_me,
            m_type,
            content,
            is_delivery,
            datetime,
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
            fid: v.pop().unwrap().as_i64(),
            hash: EventId::from_hex(v.pop().unwrap().as_str()).unwrap_or(EventId::default()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.hash.to_hex(),
            self.fid,
            self.is_me,
            self.m_type.to_int(),
            self.content,
            self.is_delivery,
            self.datetime,
        ])
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<Message> {
        let sql = format!("SELECT id, hash, fid, is_me, m_type, content, is_delivery, datetime FROM messages WHERE id = {}", id);
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Message::from_values(matrix.pop().unwrap())) // safe unwrap()
        } else {
            Err(anyhow!("message is missing."))
        }
    }

    pub fn get_by_fid(db: &DStorage, fid: &i64) -> Result<Vec<Message>> {
        let sql = format!("SELECT id, hash, fid, is_me, m_type, content, is_delivery, datetime FROM messages WHERE fid = {}", fid);
        let matrix = db.query(&sql)?;
        let mut messages = vec![];
        for values in matrix {
            messages.push(Message::from_values(values));
        }
        Ok(messages)
    }

    pub fn get_by_hash(db: &DStorage, hash: &EventId) -> Result<Message> {
        let sql = format!("SELECT id, hash, fid, is_me, m_type, content, is_delivery, datetime FROM messages WHERE hash = {}", hash.to_hex());
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Message::from_values(matrix.pop().unwrap()))
        } else {
            Err(anyhow!("message is missing."))
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!(
            "INSERT INTO messages (hash, fid, is_me, m_type, content, is_delivery, datetime) VALUES ('{}',{},{},{},'{}',{},{})",
            self.hash.to_hex(),
            self.fid,
            self.is_me,
            self.m_type.to_int(),
            self.content,
            self.is_delivery,
            self.datetime,
        );
        self.id = db.insert(&sql)?;
        Ok(())
    }

    pub fn delivery(db: &DStorage, id: i64, is_delivery: bool) -> Result<usize> {
        let sql = format!(
            "UPDATE messages SET is_delivery={} WHERE id = {}",
            is_delivery, id,
        );
        db.update(&sql)
    }

    pub fn delete(db: &DStorage, id: &i64) -> Result<usize> {
        let sql = format!("DELETE FROM messages WHERE id = {}", id);
        // TODO delete content
        db.delete(&sql)
    }

    pub fn delete_by_fid(db: &DStorage, fid: &i64) -> Result<usize> {
        let sql = format!("DELETE FROM messages WHERE fid = {}", fid);
        let size = db.delete(&sql)?;
        // TOOD delete content.
        Ok(size)
    }

    pub fn exist(db: &DStorage, hash: &EventId) -> Result<bool> {
        let sql = format!("SELECT id FROM messages WHERE hash = '{}'", hash.to_hex());
        let matrix = db.query(&sql)?;
        Ok(matrix.len() > 0)
    }
}
