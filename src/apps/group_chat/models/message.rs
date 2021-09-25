use rand::Rng;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::Result,
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use group_chat_types::NetworkMessage;

use crate::apps::chat::{Friend, MessageType};
use crate::storage::{
    chat_db, group_chat_db, read_avatar, read_file, read_record, write_avatar_sync,
    write_file_sync, write_image_sync, write_record_sync,
};

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
    pub(crate) fn new(
        height: i64,
        fid: i64,
        mid: i64,
        is_me: bool,
        m_type: MessageType,
        content: String,
    ) -> Message {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Self::new_with_time(height, fid, mid, is_me, m_type, content, datetime)
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

    pub fn get(db: &DStorage, id: &i64) -> Result<Message> {
        let mut matrix = db.query(&format!("SELECT id, height, fid, mid, is_me, m_type, content, is_delivery, datetime FROM messages WHERE id = {}", id))?;
        if matrix.len() > 0 {
            Ok(Message::from_values(matrix.pop().unwrap())) // safe unwrap.
        } else {
            Err(anyhow!("missing member"))
        }
    }

    pub fn all(db: &DStorage, fid: &i64) -> Result<Vec<Message>> {
        let matrix = db.query(&format!("SELECT id, height, fid, mid, is_me, m_type, content, is_delivery, datetime FROM messages WHERE is_deleted = false AND fid = {}", fid))?;
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
            let sql = format!("INSERT INTO messages (height, fid, mid, is_me, m_type, content, is_delivery, datetime, is_deleted) VALUES ({}, {}, {}, {}, {}, '{}', {}, {}, false)",
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
}

pub(crate) async fn to_network_message(
    base: &PathBuf,
    gid: &GroupId,
    mtype: MessageType,
    content: &str,
) -> Result<(NetworkMessage, i64)> {
    let start = SystemTime::now();
    let datetime = start
        .duration_since(UNIX_EPOCH)
        .map(|s| s.as_secs())
        .unwrap_or(0) as i64; // safe for all life.

    let nmsg = match mtype {
        MessageType::String => NetworkMessage::String(content.to_owned()),
        MessageType::Image => {
            let bytes = read_file(&PathBuf::from(content)).await?;
            NetworkMessage::Image(bytes)
        }
        MessageType::File => {
            let file_path = PathBuf::from(content);
            let bytes = read_file(&file_path).await?;
            let old_name = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_owned();
            NetworkMessage::File(old_name, bytes)
        }
        MessageType::Contact => {
            let cid: i64 = content.parse()?;
            let db = chat_db(base, gid)?;
            let contact = Friend::get_id(&db, cid)?.ok_or(anyhow!("contact missind"))?;
            drop(db);
            let avatar_bytes = read_avatar(base, &gid, &contact.gid).await?;
            NetworkMessage::Contact(contact.name, contact.gid, contact.addr, avatar_bytes)
        }
        MessageType::Record => {
            let (bytes, time) = if let Some(i) = content.find('-') {
                let time = content[0..i].parse().unwrap_or(0);
                let bytes = read_record(base, &gid, &content[i + 1..]).await?;
                (bytes, time)
            } else {
                (vec![], 0)
            };
            NetworkMessage::Record(bytes, time)
        }
        MessageType::Emoji => {
            // TODO
            NetworkMessage::Emoji
        }
        MessageType::Phone => {
            // TODO
            NetworkMessage::Phone
        }
        MessageType::Video => {
            // TODO
            NetworkMessage::Video
        }
        MessageType::Invite => NetworkMessage::Invite(content.to_owned()),
    };

    Ok((nmsg, datetime))
}

pub(crate) fn from_network_message(
    height: i64,
    gdid: i64,
    mid: GroupId,
    mgid: &GroupId,
    msg: NetworkMessage,
    datetime: i64,
    base: &PathBuf,
) -> Result<(Message, String)> {
    let db = group_chat_db(base, mgid)?;
    let mdid = Member::get_ok(&db, &gdid, &mid)?;
    let is_me = &mid == mgid;

    // handle event.
    let (m_type, raw) = match msg {
        NetworkMessage::String(content) => (MessageType::String, content),
        NetworkMessage::Image(bytes) => {
            let image_name = write_image_sync(base, mgid, bytes)?;
            (MessageType::Image, image_name)
        }
        NetworkMessage::File(old_name, bytes) => {
            let filename = write_file_sync(base, mgid, &old_name, bytes)?;
            (MessageType::File, filename)
        }
        NetworkMessage::Contact(name, rgid, addr, avatar_bytes) => {
            write_avatar_sync(base, mgid, &rgid, avatar_bytes)?;
            let tmp_name = name.replace(";", "-;");
            let contact_values = format!("{};;{};;{}", tmp_name, rgid.to_hex(), addr.to_hex());
            (MessageType::Contact, contact_values)
        }
        NetworkMessage::Emoji => {
            // TODO
            (MessageType::Emoji, "".to_owned())
        }
        NetworkMessage::Record(bytes, time) => {
            let record_name = write_record_sync(base, mgid, gdid, time, bytes)?;
            (MessageType::Record, record_name)
        }
        NetworkMessage::Phone => {
            // TODO
            (MessageType::Phone, "".to_owned())
        }
        NetworkMessage::Video => {
            // TODO
            (MessageType::Video, "".to_owned())
        }
        NetworkMessage::Invite(content) => (MessageType::Invite, content),
        NetworkMessage::None => {
            return Ok((
                Message::new(
                    height,
                    gdid,
                    mdid,
                    is_me,
                    MessageType::String,
                    "".to_owned(),
                ),
                "".to_owned(),
            ));
        }
    };

    let scontent = match m_type {
        MessageType::String => {
            format!("{}:{}", m_type.to_int(), raw)
        }
        _ => format!("{}:", m_type.to_int()),
    };

    let mut msg = Message::new_with_time(height, gdid, mdid, is_me, m_type, raw, datetime);
    msg.insert(&db)?;

    Ok((msg, scontent))
}
