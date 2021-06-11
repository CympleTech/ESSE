use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead};
use aes_gcm::Aes256Gcm;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::{EventId, GroupId},
    primitive::{new_io_error, Result},
};
use tdn_did::{genereate_id, Keypair};
use tdn_storage::local::{DStorage, DsValue};

pub(crate) struct Account {
    pub id: i64,
    pub gid: GroupId,
    pub name: String,
    pub lock: String,      // hashed-key.
    pub mnemonic: Vec<u8>, // encrypted value.
    pub secret: Vec<u8>,   // encrypted value.
    pub avatar: Vec<u8>,
    pub height: u64,
    pub event: EventId,
    pub datetime: i64,
}

impl Account {
    pub fn new(
        gid: GroupId,
        name: String,
        avatar: Vec<u8>,
        lock: String,
        mnemonic: Vec<u8>,
        secret: Vec<u8>,
    ) -> Self {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Account {
            id: 0,
            height: 0,
            event: EventId::default(),
            gid,
            name,
            lock,
            mnemonic,
            secret,
            avatar,
            datetime,
        }
    }

    pub fn generate(
        skey: &[u8], // &[u8; 32]
        name: &str,
        mnemonic: &str,
        lock: &str,
        avatar: Vec<u8>,
    ) -> Result<(Account, Keypair)> {
        let (gid, sk) = genereate_id(mnemonic.as_bytes());

        let cipher = Aes256Gcm::new(GenericArray::from_slice(skey)); // 256-bit key.
        let hash_nonce = blake3::hash(lock.as_bytes());
        let nonce = GenericArray::from_slice(&hash_nonce.as_bytes()[0..12]); // 96-bit key.

        let mnemonic_bytes = cipher
            .encrypt(nonce, mnemonic.as_bytes())
            .map_err(|_e| new_io_error("mnemonic lock invalid."))?;

        let sk_bytes = cipher
            .encrypt(nonce, sk.to_bytes().as_ref())
            .map_err(|_e| new_io_error("secret lock invalid."))?;

        Ok((
            Account::new(
                gid,
                name.to_string(),
                avatar,
                lock.to_string(),
                mnemonic_bytes,
                sk_bytes,
            ),
            sk,
        ))
    }

    pub fn _check_lock(&self, _lock: &str) -> bool {
        // TODO
        true
    }

    pub fn pin(&mut self, skey: &[u8], old: &str, new: &str) -> Result<()> {
        self.lock = new.to_string();

        let cipher = Aes256Gcm::new(GenericArray::from_slice(skey)); // 256-bit key.

        let hash_old_nonce = blake3::hash(old.as_bytes());
        let hash_new_nonce = blake3::hash(new.as_bytes());
        let old_nonce = GenericArray::from_slice(&hash_old_nonce.as_bytes()[0..12]); // 96-bit key.
        let new_nonce = GenericArray::from_slice(&hash_new_nonce.as_bytes()[0..12]); // 96-bit key.

        let mnemonic = cipher
            .decrypt(old_nonce, self.mnemonic.as_ref())
            .map_err(|_e| new_io_error("mnemonic unlock invalid."))?;

        self.mnemonic = cipher
            .encrypt(new_nonce, mnemonic.as_ref())
            .map_err(|_e| new_io_error("mnemonic lock invalid."))?;

        let secret = cipher
            .decrypt(old_nonce, self.secret.as_ref())
            .map_err(|_e| new_io_error("secret unlock invalid."))?;

        self.secret = cipher
            .encrypt(new_nonce, secret.as_ref())
            .map_err(|_e| new_io_error("secret lock invalid."))?;

        Ok(())
    }

    pub fn mnemonic(&self, skey: &[u8], lock: &str) -> Result<String> {
        let cipher = Aes256Gcm::new(GenericArray::from_slice(skey)); // 256-bit key.
        let hash_nonce = blake3::hash(lock.as_bytes());
        let nonce = GenericArray::from_slice(&hash_nonce.as_bytes()[0..12]); // 96-bit key.

        let plaintext = cipher
            .decrypt(nonce, self.mnemonic.as_ref())
            .map_err(|_e| new_io_error("mnemonic unlock invalid."))?;

        String::from_utf8(plaintext).map_err(|_e| new_io_error("mnemonic unlock invalid."))
    }

    pub fn secret(&self, skey: &[u8], lock: &str) -> Result<Keypair> {
        let cipher = Aes256Gcm::new(GenericArray::from_slice(skey)); // 256-bit key.
        let hash_nonce = blake3::hash(lock.as_bytes());
        let nonce = GenericArray::from_slice(&hash_nonce.as_bytes()[0..12]); // 96-bit key.

        let plaintext = cipher
            .decrypt(nonce, self.secret.as_ref())
            .map_err(|_e| new_io_error("secret unlock invalid."))?;

        Keypair::from_bytes(&plaintext).map_err(|_e| new_io_error("secret unlock invalid."))
    }

    /// here is zero-copy and unwrap is safe. checked.
    fn from_values(mut v: Vec<DsValue>) -> Account {
        Account {
            datetime: v.pop().unwrap().as_i64(),
            event: EventId::from_hex(v.pop().unwrap().as_str()).unwrap_or(EventId::default()),
            height: v.pop().unwrap().as_i64() as u64,
            avatar: base64::decode(v.pop().unwrap().as_string()).unwrap_or(vec![]),
            secret: base64::decode(v.pop().unwrap().as_string()).unwrap_or(vec![]),
            mnemonic: base64::decode(v.pop().unwrap().as_string()).unwrap_or(vec![]),
            lock: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            gid: GroupId::from_hex(v.pop().unwrap().as_str()).unwrap_or(GroupId::default()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn _get(db: &DStorage, gid: &GroupId) -> Result<Option<Account>> {
        let sql = format!(
            "SELECT id, gid, name, lock, mnemonic, secret, avatar, height, event, datetime FROM accounts WHERE gid = '{}'",
            gid.to_hex()
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            if values.len() == 10 {
                return Ok(Some(Account::from_values(values)));
            }
        }
        Ok(None)
    }

    pub fn all(db: &DStorage) -> Result<Vec<Account>> {
        let matrix = db.query(
            "SELECT id, gid, name, lock, mnemonic, secret, avatar, height, event, datetime FROM accounts ORDER BY datetime DESC",
        )?;
        let mut accounts = vec![];
        for values in matrix {
            if values.len() == 10 {
                accounts.push(Account::from_values(values));
            }
        }
        Ok(accounts)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from accounts WHERE gid = '{}'",
            self.gid.to_hex()
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
            self.update(db)?;
        } else {
            let sql = format!("INSERT INTO accounts (gid, name, lock, mnemonic, secret, avatar, height,event, datetime) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', {})",
            self.gid.to_hex(),
            self.name,
            self.lock,
            base64::encode(&self.mnemonic),
            base64::encode(&self.secret),
            base64::encode(&self.avatar),
            self.height,
            self.event.to_hex(),
            self.datetime,
        );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn update(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("UPDATE accounts SET gid='{}', name='{}', lock='{}', mnemonic='{}', secret='{}', avatar='{}', height={}, event='{}', datetime={} WHERE id = {}",
            self.gid.to_hex(),
            self.name,
            self.lock,
            base64::encode(&self.mnemonic),
            base64::encode(&self.secret),
            base64::encode(&self.avatar),
            self.height,
            self.datetime,
            self.event.to_hex(),
            self.id,
        );
        db.update(&sql)
    }

    pub fn update_info(&self, db: &DStorage) -> Result<usize> {
        let sql = format!(
            "UPDATE accounts SET name='{}', avatar='{}' WHERE id = {}",
            self.name,
            base64::encode(&self.avatar),
            self.id,
        );
        db.update(&sql)
    }

    pub fn _delete(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("DELETE FROM accounts WHERE id = {}", self.id);
        db.delete(&sql)
    }

    pub fn update_consensus(&mut self, db: &DStorage, height: u64, eid: EventId) -> Result<usize> {
        self.height = height;
        self.event = eid;
        let sql = format!(
            "UPDATE accounts SET height={}, event='{}' WHERE id = {}",
            self.height,
            self.event.to_hex(),
            self.id,
        );
        db.update(&sql)
    }
}
