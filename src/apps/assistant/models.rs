use rand::Rng;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    primitive::{new_io_error, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use crate::apps::chat::Friend;
use crate::storage::{chat_db, read_file, write_file, write_image};

#[derive(Eq, PartialEq, Clone)]
pub(crate) enum MessageType {
    String,
    Image,
    File,
    Contact,
    Emoji,
    Record,
    Answer,
}

impl MessageType {
    pub fn to_int(&self) -> i64 {
        match self {
            MessageType::String => 0,
            MessageType::Image => 1,
            MessageType::File => 2,
            MessageType::Contact => 3,
            MessageType::Emoji => 4,
            MessageType::Record => 5,
            MessageType::Answer => 6,
        }
    }

    pub fn from_int(i: i64) -> MessageType {
        match i {
            0 => MessageType::String,
            1 => MessageType::Image,
            2 => MessageType::File,
            3 => MessageType::Contact,
            4 => MessageType::Emoji,
            5 => MessageType::Record,
            6 => MessageType::Answer,
            _ => MessageType::String,
        }
    }

    pub async fn handle(
        self,
        base: &PathBuf,
        mgid: &GroupId,
        content: String,
    ) -> std::result::Result<Message, tdn::types::rpc::RpcError> {
        let (q_type, q_raw, a_type, a_raw) = match self {
            MessageType::Image => {
                let bytes = read_file(&PathBuf::from(content)).await?;
                let image_name = write_image(base, &mgid, &bytes).await?;
                (self, image_name.clone(), MessageType::Image, image_name)
            }
            MessageType::File => {
                let file_path = PathBuf::from(content);
                let bytes = read_file(&file_path).await?;
                let old_name = file_path.file_name()?.to_str()?;
                let filename = write_file(base, mgid, old_name, &bytes).await?;
                (self, filename.clone(), MessageType::File, filename)
            }
            MessageType::Contact => {
                let cid: i64 = content.parse().map_err(|_e| new_io_error("id error"))?;
                let db = chat_db(base, mgid)?;
                let contact = Friend::get_id(&db, cid)??;
                db.close()?;
                let tmp_name = contact.name.replace(";", "-;");
                let raw = format!(
                    "{};;{};;{}",
                    tmp_name,
                    contact.gid.to_hex(),
                    contact.addr.to_hex()
                );
                (self, raw.clone(), MessageType::Contact, raw)
            }
            MessageType::Answer => {
                let a_raw = format!("{}", rand::thread_rng().gen_range(0..171));
                (MessageType::String, content, MessageType::Answer, a_raw)
            }
            _ => (self.clone(), content.clone(), self, content),
        };

        Ok(Message::new(q_type, q_raw, a_type, a_raw))
    }
}

pub(crate) struct Message {
    pub id: i64,
    pub q_type: MessageType,
    pub q_content: String,
    pub a_type: MessageType,
    pub a_content: String,
    pub datetime: i64,
}

impl Message {
    pub fn new(
        q_type: MessageType,
        q_content: String,
        a_type: MessageType,
        a_content: String,
    ) -> Message {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Message {
            id: 0,
            q_type,
            q_content,
            a_type,
            a_content,
            datetime,
        }
    }

    /// here is zero-copy and unwrap is safe. checked.
    fn from_values(mut v: Vec<DsValue>) -> Message {
        Message {
            datetime: v.pop().unwrap().as_i64(),
            a_content: v.pop().unwrap().as_string(),
            a_type: MessageType::from_int(v.pop().unwrap().as_i64()),
            q_content: v.pop().unwrap().as_string(),
            q_type: MessageType::from_int(v.pop().unwrap().as_i64()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.q_type.to_int(),
            self.q_content,
            self.a_type.to_int(),
            self.a_content,
            self.datetime,
        ])
    }

    pub fn all(db: &DStorage) -> Result<Vec<Message>> {
        let sql =
            format!("SELECT id, q_type, q_content, a_type, a_content, datetime FROM messages where is_deleted = false");
        let matrix = db.query(&sql)?;
        let mut messages = vec![];
        for values in matrix {
            messages.push(Message::from_values(values));
        }

        Ok(messages)
    }

    pub fn _get(db: &DStorage, id: &i64) -> Result<Option<Message>> {
        let sql = format!(
            "SELECT id, q_type, q_content, a_type, a_content, datetime FROM messages WHERE id = {}",
            id
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Some(Message::from_values(matrix.pop().unwrap())))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!(
            "INSERT INTO messages (q_type, q_content, a_type, a_content, datetime, is_deleted) VALUES ({},'{}',{},'{}',{},false)",
            self.q_type.to_int(),
            self.q_content,
            self.a_type.to_int(),
            self.a_content,
            self.datetime,
        );
        self.id = db.insert(&sql)?;
        Ok(())
    }

    pub fn delete(db: &DStorage, id: i64) -> Result<usize> {
        let sql = format!("UPDATE messages SET is_deleted = true WHERE id = {}", id);
        db.delete(&sql)
    }
}
