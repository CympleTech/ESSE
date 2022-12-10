use esse_primitives::{id_from_str, id_to_str};
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    primitives::{PeerId, Result},
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

use crate::session::{Session, SessionType};

use super::Message;

pub(crate) struct Friend {
    pub id: i64,
    pub pid: PeerId,
    pub name: String,
    pub cloud: PeerId,
    pub cloud_key: [u8; 32],
    pub height: i64,
    pub remark: String,
    pub is_closed: bool,
    pub datetime: i64,
}

impl Friend {
    pub fn new(
        pid: PeerId,
        name: String,
        cloud: PeerId,
        cloud_key: [u8; 32],
        remark: String,
        height: i64,
    ) -> Friend {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Friend {
            id: 0,
            pid,
            name,
            cloud,
            cloud_key,
            height,
            remark,
            datetime,
            is_closed: false,
        }
    }

    /// here is zero-copy and unwrap is safe.
    fn from_values(mut v: Vec<DsValue>) -> Friend {
        Friend {
            datetime: v.pop().unwrap().as_i64(),
            is_closed: v.pop().unwrap().as_bool(),
            remark: v.pop().unwrap().as_string(),
            height: v.pop().unwrap().as_i64(),
            cloud_key: hex::decode(v.pop().unwrap().as_str())
                .map(|bytes| {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&bytes);
                    key
                })
                .unwrap_or([0u8; 32]),
            cloud: PeerId::from_hex(v.pop().unwrap().as_str()).unwrap_or(PeerId::default()),
            name: v.pop().unwrap().as_string(),
            pid: id_from_str(v.pop().unwrap().as_str()).unwrap_or(PeerId::default()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn from_remote(
        db: &DStorage,
        pid: PeerId,
        name: String,
        cloud: PeerId,
        cloud_key: [u8; 32],
    ) -> Result<Friend> {
        if let Ok(mut friend) = Friend::get_id(&db, &pid) {
            friend.name = name;
            friend.cloud = cloud;
            friend.cloud_key = cloud_key;
            friend.is_closed = false;
            friend.remote_update(&db)?;
            Ok(friend)
        } else {
            let mut friend = Friend::new(pid, name, cloud, cloud_key, "".to_owned(), 0);
            friend.insert(&db)?;
            Ok(friend)
        }
    }

    pub fn to_session(&self) -> Session {
        Session::new(
            self.id,
            id_to_str(&self.pid),
            self.pid,
            SessionType::Chat,
            self.name.clone(),
            self.datetime,
        )
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            id_to_str(&self.pid),
            self.name,
            self.cloud.to_hex(),
            self.remark,
            self.is_closed,
            self.datetime
        ])
    }

    pub fn to_rpc_online(&self, online: bool) -> RpcParam {
        json!([
            self.id,
            id_to_str(&self.pid),
            self.name,
            self.cloud.to_hex(),
            self.remark,
            self.is_closed,
            self.datetime,
            online
        ])
    }

    pub fn get_id(db: &DStorage, pid: &PeerId) -> Result<Friend> {
        let sql = format!("SELECT id, pid, name, cloud, cloud_key, height, remark, is_closed, datetime FROM friends WHERE pid = '{}'", id_to_str(pid));
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Friend::from_values(matrix.pop().unwrap())) // safe unwrap()
        } else {
            Err(anyhow!("friend is missing."))
        }
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<Friend> {
        let sql = format!("SELECT id, pid, name, cloud, cloud_key, height, remark, is_closed, datetime FROM friends WHERE id = {}", id);
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            Ok(Friend::from_values(matrix.pop().unwrap())) // safe unwrap()
        } else {
            Err(anyhow!("friend is missing."))
        }
    }

    /// use in rpc when load account friends.
    pub fn list(db: &DStorage) -> Result<Vec<Friend>> {
        let matrix = db.query(
            "SELECT id, pid, name, cloud, cloud_key, height, remark, is_closed, datetime FROM friends",
        )?;
        let mut friends = vec![];
        for values in matrix {
            friends.push(Friend::from_values(values));
        }
        Ok(friends)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!("INSERT INTO friends (pid, name, cloud, cloud_key, height, remark, is_closed, datetime) VALUES ('{}', '{}', '{}', '{}', {}, '{}', {}, {})",
            id_to_str(&self.pid),
            self.name,
            self.cloud.to_hex(),
            hex::encode(&self.cloud_key),
            self.height,
            self.remark,
            self.is_closed,
            self.datetime,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn update(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("UPDATE friends SET name='{}', cloud='{}', cloud_key='{}', height={}, remark='{}', is_closed={} WHERE id={}",
            self.name,
            self.cloud.to_hex(),
            hex::encode(&self.cloud_key),
            self.height,
            self.remark,
            self.is_closed,
            self.id
        );
        db.update(&sql)
    }

    pub fn me_update(&mut self, db: &DStorage) -> Result<usize> {
        let sql = format!(
            "UPDATE friends SET remark='{}' WHERE id = {}",
            self.remark, self.id,
        );
        db.update(&sql)
    }

    pub fn remote_update(&self, db: &DStorage) -> Result<usize> {
        let sql = format!(
            "UPDATE friends SET name='{}', cloud='{}', cloud_key='{}', height={}, is_closed = false WHERE id = {}",
            self.name,
            self.cloud.to_hex(),
            hex::encode(&self.cloud_key),
            self.height,
            self.id,
        );
        db.update(&sql)
    }

    /// used in rpc, when what to delete a friend.
    pub fn close(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("UPDATE friends SET is_closed = true WHERE id = {}", self.id);
        db.update(&sql)
    }

    /// used in rpc, when what to delete a friend.
    pub fn delete(db: &DStorage, id: &i64) -> Result<usize> {
        let sql = format!("DELETE FROM friends WHERE id = {}", id);
        db.update(&sql)?;

        // TODO delete friend avatar.

        // delete messages;
        Message::delete_by_fid(&db, id)
    }

    pub fn is_friend(db: &DStorage, pid: &PeerId) -> Result<bool> {
        let sql = format!(
            "SELECT id FROM friends WHERE is_closed = false and pid = '{}'",
            id_to_str(pid)
        );
        let matrix = db.query(&sql)?;
        Ok(matrix.len() > 0)
    }

    /// used in layers, when receive remote had closed.
    pub fn id_close(db: &DStorage, id: i64) -> Result<usize> {
        let sql = format!("UPDATE friends SET is_closed = true WHERE id = {}", id);
        db.update(&sql)
    }
}
