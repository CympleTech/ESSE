use esse_primitives::MessageType;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    primitives::Result,
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

pub(crate) struct Message {
    pub id: i64,
    pub is_me: bool,
    pub m_type: MessageType,
    pub content: String,
    pub datetime: i64,
}

impl Message {
    pub fn new(m_type: MessageType, content: String, is_me: bool) -> Message {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Message {
            id: 0,
            is_me,
            m_type,
            content,
            datetime,
        }
    }

    /// here is zero-copy and unwrap is safe. checked.
    fn from_values(mut v: Vec<DsValue>) -> Message {
        Message {
            datetime: v.pop().unwrap().as_i64(),
            content: v.pop().unwrap().as_string(),
            m_type: MessageType::from_int(v.pop().unwrap().as_i64()),
            is_me: v.pop().unwrap().as_bool(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.is_me,
            self.m_type.to_int(),
            self.content,
            self.datetime,
        ])
    }

    pub fn list(db: &DStorage) -> Result<Vec<Message>> {
        let sql = format!("SELECT id, is_me, m_type, content, datetime FROM messages");
        let matrix = db.query(&sql)?;
        let mut messages = vec![];
        for values in matrix {
            messages.push(Message::from_values(values));
        }

        Ok(messages)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!(
            "INSERT INTO messages (is_me, m_type, content, datetime) VALUES ({}, {}, '{}',{})",
            self.is_me,
            self.m_type.to_int(),
            self.content,
            self.datetime,
        );
        self.id = db.insert(&sql)?;
        Ok(())
    }

    pub fn delete(db: &DStorage, id: i64) -> Result<usize> {
        let sql = format!("DELETE FROM messages WHERE id = {}", id);
        db.delete(&sql)
    }
}
