use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::{EventId, GroupId},
    primitive::{HandleResult, PeerId, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use chat_types::{MessageType, NetworkMessage};

use crate::storage::{read_avatar_sync, read_file_sync, read_image_sync, read_record_sync};

use super::from_network_message;

pub(crate) fn handle_nmsg(
    nmsg: NetworkMessage,
    is_me: bool,
    gid: GroupId,
    base: &PathBuf,
    db: &DStorage,
    fid: i64,
    hash: EventId,
    results: &mut HandleResult,
) -> Result<Message> {
    // handle event.
    let (m_type, raw) = from_network_message(nmsg, base, &gid, results)?;
    let mut msg = Message::new_with_id(hash, fid, is_me, m_type, raw, true);
    msg.insert(db)?;
    Ok(msg)
}

pub(crate) fn from_model(base: &PathBuf, gid: &GroupId, model: Message) -> Result<NetworkMessage> {
    // handle message's type.
    match model.m_type {
        MessageType::String => Ok(NetworkMessage::String(model.content)),
        MessageType::Image => {
            let bytes = read_image_sync(base, gid, &model.content)?;
            Ok(NetworkMessage::Image(bytes))
        }
        MessageType::File => {
            let bytes = read_file_sync(base, gid, &model.content)?;
            Ok(NetworkMessage::File(model.content, bytes))
        }
        MessageType::Contact => {
            let v: Vec<&str> = model.content.split(";;").collect();
            if v.len() != 3 {
                return Err(anyhow!("message is invalid"));
            }
            let cname = v[0].to_owned();
            let cgid = GroupId::from_hex(v[1])?;
            let caddr = PeerId::from_hex(v[2])?;
            let avatar_bytes = read_avatar_sync(base, gid, &cgid)?;
            Ok(NetworkMessage::Contact(cname, cgid, caddr, avatar_bytes))
        }
        MessageType::Record => {
            let (bytes, time) = if let Some(i) = model.content.find('-') {
                let time = model.content[0..i].parse().unwrap_or(0);
                let bytes = read_record_sync(base, gid, &model.content[i + 1..])?;
                (bytes, time)
            } else {
                (vec![], 0)
            };
            Ok(NetworkMessage::Record(bytes, time))
        }
        MessageType::Invite => Ok(NetworkMessage::Invite(model.content)),
        MessageType::Emoji => Ok(NetworkMessage::Emoji),
        MessageType::Phone => Ok(NetworkMessage::Phone),
        MessageType::Video => Ok(NetworkMessage::Video),
    }
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
        gid: &GroupId,
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
        bytes[0..8].copy_from_slice(&gid.0[0..8]);
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
