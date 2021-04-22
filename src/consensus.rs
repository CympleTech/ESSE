use serde::{Deserialize, Serialize};
use tdn::types::{group::EventId, primitive::Result, rpc::RpcParam};
use tdn_storage::local::DStorage;

#[derive(Serialize, Deserialize)]
pub(crate) enum SyncModel {
    Request(RpcParam),
    Friend(RpcParam),
    Message(RpcParam),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Event {
    pub height: u64,
    pub hash: EventId,
    path: i64,
    row: i64,
}

impl Event {
    pub fn contains_hash(db: &DStorage, hash: &EventId) -> Result<bool> {
        let sql = format!("SELECT id from events WHERE hash = '{}'", hash.to_hex());
        Ok(db.query(&sql)?.len() > 0)
    }

    pub fn get_nexts(db: &DStorage, id: u64) -> Result<Vec<Event>> {
        let sql = format!(
            "SELECT id, hash, db_table, row from events WHERE id >= {} ORDER BY id",
            id
        );
        let matrix = db.query(&sql)?;
        let mut events = vec![];
        for mut values in matrix {
            let row = values.pop().unwrap().as_i64(); // safe
            let path = values.pop().unwrap().as_i64(); // safe
            let hash =
                EventId::from_hex(values.pop().unwrap().as_str()).unwrap_or(EventId::default());
            let id = values.pop().unwrap().as_i64(); // safe

            events.push(Self {
                height: id as u64,
                hash,
                path,
                row,
            })
        }

        Ok(events)
    }

    pub fn get_assign_hash(db: &DStorage, assigns: &Vec<u64>) -> Result<Vec<EventId>> {
        let sql = if assigns.len() == 1 {
            format!("SELECT id, hash from events WHERE id = {}", assigns[0])
        } else {
            let last = assigns.len() - 1;
            let mut sql = format!("SELECT id, hash from events WHERE id IN (");
            for (k, u) in assigns.iter().enumerate() {
                if last == k {
                    sql.push_str(&format!("{})", u));
                } else {
                    sql.push_str(&format!("{},", u));
                }
            }
            sql
        };

        let matrix = db.query(&sql)?;
        let mut hashes = vec![];
        for mut values in matrix {
            hashes.push(
                EventId::from_hex(values.pop().unwrap().as_str()).unwrap_or(EventId::default()),
            );
        }

        if assigns.len() > hashes.len() {
            for _ in 0..(assigns.len() - hashes.len()) {
                hashes.push(EventId::default());
            }
        }

        Ok(hashes)
    }

    pub(crate) fn merge(
        db: &DStorage,
        hash: EventId,
        path: i64,
        row: i64,
        index: u64,
    ) -> Result<()> {
        // check if height is had.
        let check_sql = format!("SELECT id from events WHERE id = {}", index);
        let check_matrix = db.query(&check_sql)?;
        if check_matrix.len() > 0 {
            let first_sql = format!(
                "SELECT id from events WHERE id >= {} ORDER BY id DESC",
                index
            );
            let matrix = db.query(&first_sql)?;
            for mut values in matrix {
                if let Some(id) = values.pop() {
                    let now_id = id.as_i64();
                    let sql = format!(
                        "UPDATE events SET id = {} WHERE id = {}",
                        now_id + 1,
                        now_id
                    );
                    db.update(&sql)?;
                }
            }
        }

        let sql = format!(
            "INSERT INTO events (id, hash, db_table, row) VALUES ({}, '{}', {}, {})",
            index,
            hash.to_hex(),
            path,
            row,
        );
        db.insert(&sql)?;

        Ok(())
    }
}
