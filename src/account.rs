use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead};
use aes_gcm::Aes256Gcm;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::{EventId, GroupId},
    primitive::Result,
};
use tdn_did::{generate_id, Keypair, Language};
use tdn_storage::local::{DStorage, DsValue};

fn mnemonic_lang_to_i64(lang: Language) -> i64 {
    match lang {
        Language::English => 0,
        Language::SimplifiedChinese => 1,
        Language::TraditionalChinese => 2,
        Language::Czech => 3,
        Language::French => 4,
        Language::Italian => 5,
        Language::Japanese => 6,
        Language::Korean => 7,
        Language::Spanish => 8,
        Language::Portuguese => 9,
    }
}

pub fn mnemonic_lang_from_i64(u: i64) -> Language {
    match u {
        0 => Language::English,
        1 => Language::SimplifiedChinese,
        2 => Language::TraditionalChinese,
        3 => Language::Czech,
        4 => Language::French,
        5 => Language::Italian,
        6 => Language::Japanese,
        7 => Language::Korean,
        8 => Language::Spanish,
        9 => Language::Portuguese,
        _ => Language::English,
    }
}

pub(crate) struct Account {
    pub id: i64,
    pub gid: GroupId,
    pub index: i64,
    pub lang: i64,
    pub mnemonic: Vec<u8>, // encrypted value.
    pub pass: String,
    pub name: String,
    pub avatar: Vec<u8>,
    pub lock: String,    // hashed-key.
    pub secret: Vec<u8>, // encrypted value.
    pub height: u64,
    pub event: EventId,
    pub datetime: i64,
}

impl Account {
    pub fn new(
        gid: GroupId,
        index: i64,
        lang: i64,
        pass: String,
        name: String,
        lock: String,
        avatar: Vec<u8>,
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
            index,
            lang,
            pass,
            name,
            lock,
            mnemonic,
            secret,
            avatar,
            datetime,
        }
    }

    pub fn lang(&self) -> Language {
        mnemonic_lang_from_i64(self.lang)
    }

    pub fn generate(
        index: u32,
        skey: &[u8], // &[u8; 32]
        lang: i64,
        mnemonic: &str,
        pass: &str,
        name: &str,
        lock: &str,
        avatar: Vec<u8>,
    ) -> Result<(Account, Keypair)> {
        let (gid, sk) = generate_id(
            mnemonic_lang_from_i64(lang),
            mnemonic,
            index,
            0, // account default multiple address index is 0.
            if pass.len() > 0 { Some(pass) } else { None },
        )?;

        let cipher = Aes256Gcm::new(GenericArray::from_slice(skey)); // 256-bit key.
        let hash_nonce = blake3::hash(lock.as_bytes());
        let nonce = GenericArray::from_slice(&hash_nonce.as_bytes()[0..12]); // 96-bit key.

        let mnemonic_bytes = cipher
            .encrypt(nonce, mnemonic.as_bytes())
            .map_err(|_e| anyhow!("mnemonic lock invalid."))?;

        let sk_bytes = cipher
            .encrypt(nonce, sk.to_bytes().as_ref())
            .map_err(|_e| anyhow!("secret lock invalid."))?;

        Ok((
            Account::new(
                gid,
                index as i64,
                lang,
                pass.to_string(),
                name.to_string(),
                lock.to_string(),
                avatar,
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
            .map_err(|_e| anyhow!("mnemonic unlock invalid."))?;

        self.mnemonic = cipher
            .encrypt(new_nonce, mnemonic.as_ref())
            .map_err(|_e| anyhow!("mnemonic lock invalid."))?;

        let secret = cipher
            .decrypt(old_nonce, self.secret.as_ref())
            .map_err(|_e| anyhow!("secret unlock invalid."))?;

        self.secret = cipher
            .encrypt(new_nonce, secret.as_ref())
            .map_err(|_e| anyhow!("secret lock invalid."))?;

        Ok(())
    }

    pub fn mnemonic(&self, skey: &[u8], lock: &str) -> Result<String> {
        let cipher = Aes256Gcm::new(GenericArray::from_slice(skey)); // 256-bit key.
        let hash_nonce = blake3::hash(lock.as_bytes());
        let nonce = GenericArray::from_slice(&hash_nonce.as_bytes()[0..12]); // 96-bit key.

        let plaintext = cipher
            .decrypt(nonce, self.mnemonic.as_ref())
            .map_err(|_e| anyhow!("mnemonic unlock invalid."))?;

        String::from_utf8(plaintext).map_err(|_e| anyhow!("mnemonic unlock invalid."))
    }

    pub fn secret(&self, skey: &[u8], lock: &str) -> Result<Keypair> {
        let cipher = Aes256Gcm::new(GenericArray::from_slice(skey)); // 256-bit key.
        let hash_nonce = blake3::hash(lock.as_bytes());
        let nonce = GenericArray::from_slice(&hash_nonce.as_bytes()[0..12]); // 96-bit key.

        let plaintext = cipher
            .decrypt(nonce, self.secret.as_ref())
            .map_err(|_e| anyhow!("secret unlock invalid."))?;

        Keypair::from_bytes(&plaintext).map_err(|_e| anyhow!("secret unlock invalid."))
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
            pass: v.pop().unwrap().as_string(),
            lang: v.pop().unwrap().as_i64(),
            index: v.pop().unwrap().as_i64(),
            gid: GroupId::from_hex(v.pop().unwrap().as_str()).unwrap_or(GroupId::default()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn _get(db: &DStorage, gid: &GroupId) -> Result<Option<Account>> {
        let sql = format!(
            "SELECT id, gid, indx, lang, pass, name, lock, mnemonic, secret, avatar, height, event, datetime FROM accounts WHERE gid = '{}'",
            gid.to_hex()
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            if values.len() == 13 {
                return Ok(Some(Account::from_values(values)));
            }
        }
        Ok(None)
    }

    pub fn all(db: &DStorage) -> Result<Vec<Account>> {
        let matrix = db.query(
            "SELECT id, gid, indx, lang, pass, name, lock, mnemonic, secret, avatar, height, event, datetime FROM accounts ORDER BY datetime DESC",
        )?;
        let mut accounts = vec![];
        for values in matrix {
            if values.len() == 13 {
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
            let sql = format!("INSERT INTO accounts (gid, indx, lang, pass, name, lock, mnemonic, secret, avatar, height,event, datetime) VALUES ('{}', {}, {}, '{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', {})",
            self.gid.to_hex(),
            self.index,
            self.lang,
            self.pass,
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
        let sql = format!("UPDATE accounts SET name='{}', lock='{}', mnemonic='{}', secret='{}', avatar='{}', height={}, event='{}', datetime={} WHERE id = {}",
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
