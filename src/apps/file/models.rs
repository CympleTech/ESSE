use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    primitives::Result,
    rpc::{json, RpcParam},
};
use tdn_storage::local::{DStorage, DsValue};

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub(crate) enum RootDirectory {
    Star,
    Trash,
    Session,
    Document,
    Image,
    Music,
    Video,
}

impl RootDirectory {
    fn to_i64(&self) -> i64 {
        match self {
            RootDirectory::Star => 0,
            RootDirectory::Trash => 1,
            RootDirectory::Session => 2,
            RootDirectory::Document => 3,
            RootDirectory::Image => 4,
            RootDirectory::Music => 5,
            RootDirectory::Video => 6,
        }
    }

    pub fn from_i64(i: i64) -> Self {
        match i {
            0 => RootDirectory::Star,
            1 => RootDirectory::Trash,
            2 => RootDirectory::Session,
            3 => RootDirectory::Document,
            4 => RootDirectory::Image,
            5 => RootDirectory::Music,
            6 => RootDirectory::Video,
            _ => RootDirectory::Trash,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct FileDid([u8; 32]);

impl FileDid {
    pub fn generate() -> Self {
        Self(rand::thread_rng().gen::<[u8; 32]>())
    }

    pub fn to_hex(&self) -> String {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        hex
    }

    pub fn from_hex(s: &str) -> Result<Self> {
        if s.len() != 64 {
            return Err(anyhow::anyhow!("Hex is invalid!"));
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)?;
            bytes[i] = res;
        }

        Ok(Self(bytes))
    }
}

pub(crate) struct File {
    pub id: i64,
    pub did: FileDid,
    pub parent: i64, // if root directory, parent is 0.
    pub root: RootDirectory,
    pub name: String,
    pub starred: bool,
    //pub device: Vec<i64>,
    pub datetime: i64,
}

impl File {
    pub fn generate(root: RootDirectory, parent: i64, name: String) -> Self {
        let did = FileDid::generate();
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Self {
            did,
            parent,
            root,
            name,
            datetime,
            id: 0,
            starred: false,
        }
    }

    pub fn storage_name(&self) -> String {
        self.did.to_hex()
    }

    fn _read(&self) -> Vec<u8> {
        todo!()
    }

    fn _write(&self, _bytes: &[u8]) -> Result<()> {
        todo!()
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.did.to_hex(),
            self.parent,
            self.root.to_i64(),
            self.name,
            self.starred,
            self.datetime,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            datetime: v.pop().unwrap().as_i64(),
            starred: v.pop().unwrap().as_bool(),
            name: v.pop().unwrap().as_string(),
            root: RootDirectory::from_i64(v.pop().unwrap().as_i64()),
            parent: v.pop().unwrap().as_i64(),
            did: FileDid::from_hex(&v.pop().unwrap().as_string()).unwrap_or(Default::default()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<Self> {
        let sql = format!(
            "SELECT id, did, parent, root, name, starred, datetime FROM files WHERE id = {}",
            id
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("file is missing"))
    }

    pub fn list(db: &DStorage, root: &RootDirectory, parent: &i64) -> Result<Vec<Self>> {
        let sql = if root == &RootDirectory::Star {
            format!(
                "SELECT id, did, parent, root, name, starred, datetime FROM files WHERE starred = true AND root != {}",
                RootDirectory::Trash.to_i64()
            )
        } else {
            format!(
                "SELECT id, did, parent, root, name, starred, datetime FROM files WHERE parent = {} AND root = {}",
                parent, root.to_i64()
            )
        };

        let matrix = db.query(&sql)?;
        let mut files = vec![];
        for values in matrix {
            files.push(Self::from_values(values));
        }
        Ok(files)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!(
            "INSERT INTO files (did, parent, root, name, starred, device, datetime) VALUES ('{}', {}, {}, '{}', {}, '', {})",
            self.did.to_hex(),
            self.parent,
            self.root.to_i64(),
            self.name,
            self.starred,
            self.datetime,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn star(db: &DStorage, id: &i64, starred: bool) -> Result<()> {
        let sql = format!("UPDATE files SET starred = {} WHERE id = {}", starred, id);
        db.update(&sql)?;
        Ok(())
    }

    pub fn trash(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!(
            "UPDATE files SET root = {} WHERE id = {}",
            RootDirectory::Trash.to_i64(),
            id
        );
        db.update(&sql)?;
        Ok(())
    }

    pub fn delete(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!("DELETE FROM files WHERE id = {}", id);
        db.delete(&sql)?;
        Ok(())
    }

    pub fn update(&self, db: &DStorage) -> Result<()> {
        let sql = format!(
            "UPDATE files SET parent = {}, root = {}, name = '{}' WHERE id = {}",
            self.parent,
            self.root.to_i64(),
            self.name,
            self.id
        );
        db.update(&sql)?;
        Ok(())
    }
}
