use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::EventId,
    primitives::{PeerId, PeerKey, Result},
};
use tdn_did::{generate_eth_account, Language};
use tdn_storage::local::{DStorage, DsValue};

use esse_primitives::{id_from_str, id_to_str};

use crate::apps::wallet::models::{Address, ChainToken};
use crate::utils::crypto::{
    check_pin, decrypt, decrypt_key, encrypt_key, encrypt_multiple, hash_pin,
};

fn _lang_to_i64(lang: Language) -> i64 {
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

pub fn lang_from_i64(u: i64) -> Language {
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
    pub pid: PeerId,
    pub index: i64,
    pub lang: i64,
    pub mnemonic: Vec<u8>, // encrypted value.
    pub pass: String,
    pub name: String,
    pub avatar: Vec<u8>,
    pub lock: String,        // hashed-lock.
    pub secret: Vec<u8>,     // encrypted value.
    pub encrypt: Vec<u8>,    // encrypted encrypt key.
    pub cloud: PeerId,       // main cloud service.
    pub cloud_key: [u8; 32], // main cloud session key.
    pub pub_height: u64,     // public information height.
    pub own_height: u64,     // own data consensus height.
    pub event: EventId,
    pub datetime: i64,
    plainkey: Vec<u8>,
}

impl Account {
    pub fn new(
        pid: PeerId,
        index: i64,
        lang: i64,
        pass: String,
        name: String,
        lock: String,
        avatar: Vec<u8>,
        mnemonic: Vec<u8>,
        secret: Vec<u8>,
        encrypt: Vec<u8>,
        plainkey: Vec<u8>,
        cloud: PeerId,
        cloud_key: [u8; 32],
    ) -> Self {
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as i64; // safe for all life.

        Account {
            id: 0,
            pub_height: 1,
            own_height: 0,
            event: EventId::default(),
            pid,
            index,
            lang,
            pass,
            name,
            lock,
            mnemonic,
            secret,
            encrypt,
            cloud,
            cloud_key,
            plainkey,
            avatar,
            datetime,
        }
    }

    pub fn lang(&self) -> Language {
        lang_from_i64(self.lang)
    }

    pub fn generate(
        index: u32,
        salt: &[u8], // &[u8; 32]
        rlang: i64,
        mnemonic: &str,
        pass: &str,
        name: &str,
        lock: &str,
        avatar: Vec<u8>,
    ) -> Result<(Account, PeerKey, Address)> {
        let lang = lang_from_i64(rlang);

        println!("Lang: {:?}, seed :{}", lang, mnemonic);

        // Default ETH wallet account.
        let wpass = if pass.len() > 0 { Some(pass) } else { None };
        let key = generate_eth_account(lang, mnemonic, index, 0, wpass)?;
        let address = key.peer_id().to_hex();
        println!("Lang: {:?}, seed: {}, address: {}", lang, mnemonic, address);

        let w = Address::new(ChainToken::ETH, 0, address, true);

        let mut rng = ChaChaRng::from_entropy();
        let mut eckey = [0u8; 32];
        rng.fill_bytes(&mut eckey);
        let ckey = encrypt_key(salt, lock, &eckey)?;
        let mut ebytes = encrypt_multiple(
            salt,
            lock,
            &ckey,
            vec![&key.to_db_bytes(), mnemonic.as_bytes()],
        )?;
        let mnemonic = ebytes.pop().unwrap_or(vec![]);
        let secret = ebytes.pop().unwrap_or(vec![]);
        let index = index as i64;

        Ok((
            Account::new(
                key.peer_id(),
                index,
                rlang,
                pass.to_string(),
                name.to_string(),
                hash_pin(lock)?,
                avatar,
                mnemonic,
                secret,
                ckey,
                eckey.to_vec(),
                PeerId::default(),
                [0u8; 32],
            ),
            key,
            w,
        ))
    }

    pub fn check_lock(&self, lock: &str) -> Result<()> {
        if check_pin(lock, &self.lock)? {
            Ok(())
        } else {
            Err(anyhow!("lock is invalid!"))
        }
    }

    // when success login, cache plain encrypt key for database use.
    pub fn cache_plainkey(&mut self, salt: &[u8], lock: &str) -> Result<()> {
        self.plainkey = decrypt_key(salt, lock, &self.encrypt)?;
        Ok(())
    }

    pub fn plainkey(&self) -> String {
        hex::encode(&self.plainkey)
    }

    pub fn pin(&mut self, salt: &[u8], old: &str, new: &str) -> Result<()> {
        self.check_lock(old)?;
        self.lock = hash_pin(new)?;
        let key = decrypt_key(salt, old, &self.encrypt)?;
        self.plainkey = key;
        self.encrypt = encrypt_key(salt, new, &self.plainkey)?;

        Ok(())
    }

    pub fn mnemonic(&self, salt: &[u8], lock: &str) -> Result<String> {
        self.check_lock(lock)?;
        let pbytes = decrypt(salt, lock, &self.encrypt, &self.mnemonic)?;
        String::from_utf8(pbytes).or(Err(anyhow!("mnemonic unlock invalid.")))
    }

    pub fn secret(&self, salt: &[u8], lock: &str) -> Result<PeerKey> {
        self.check_lock(lock)?;
        let pbytes = decrypt(salt, lock, &self.encrypt, &self.secret)?;
        PeerKey::from_db_bytes(&pbytes).or(Err(anyhow!("secret unlock invalid.")))
    }

    /// here is zero-copy and unwrap is safe. checked.
    fn from_values(mut v: Vec<DsValue>) -> Account {
        Account {
            datetime: v.pop().unwrap().as_i64(),
            event: EventId::from_hex(v.pop().unwrap().as_str()).unwrap_or(EventId::default()),
            own_height: v.pop().unwrap().as_i64() as u64,
            pub_height: v.pop().unwrap().as_i64() as u64,
            cloud_key: hex::decode(v.pop().unwrap().as_str())
                .map(|bytes| {
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&bytes);
                    key
                })
                .unwrap_or([0u8; 32]),
            cloud: PeerId::from_hex(v.pop().unwrap().as_str()).unwrap_or(PeerId::default()),
            avatar: base64::decode(v.pop().unwrap().as_str()).unwrap_or(vec![]),
            encrypt: base64::decode(v.pop().unwrap().as_str()).unwrap_or(vec![]),
            secret: base64::decode(v.pop().unwrap().as_str()).unwrap_or(vec![]),
            mnemonic: base64::decode(v.pop().unwrap().as_str()).unwrap_or(vec![]),
            lock: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            pass: v.pop().unwrap().as_string(),
            lang: v.pop().unwrap().as_i64(),
            index: v.pop().unwrap().as_i64(),
            pid: id_from_str(v.pop().unwrap().as_str()).unwrap_or(PeerId::default()),
            id: v.pop().unwrap().as_i64(),
            plainkey: vec![],
        }
    }

    pub fn get(db: &DStorage, pid: &PeerId) -> Result<Account> {
        let sql = format!(
            "SELECT id, pid, indx, lang, pass, name, lock, mnemonic, secret, encrypt, avatar, cloud, cloud_key, pub_height, own_height, event, datetime FROM accounts WHERE pid = '{}'",
            id_to_str(pid)
        );
        let mut matrix = db.query(&sql)?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            Ok(Account::from_values(values))
        } else {
            Err(anyhow!("account is missing."))
        }
    }

    pub fn all(db: &DStorage) -> Result<Vec<Account>> {
        let matrix = db.query(
            "SELECT id, pid, indx, lang, pass, name, lock, mnemonic, secret, encrypt, avatar, cloud, cloud_key, pub_height, own_height, event, datetime FROM accounts ORDER BY datetime DESC",
        )?;
        let mut accounts = vec![];
        for values in matrix {
            accounts.push(Account::from_values(values));
        }
        Ok(accounts)
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from accounts WHERE pid = '{}'",
            id_to_str(&self.pid)
        ))?;
        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            self.id = id;
            self.update(db)?;
        } else {
            let sql = format!("INSERT INTO accounts (pid, indx, lang, pass, name, lock, mnemonic, secret, encrypt, avatar, cloud, cloud_key, pub_height, own_height, event, datetime) VALUES ('{}', {}, {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, {}, '{}', {})",
            id_to_str(&self.pid),
            self.index,
            self.lang,
            self.pass,
            self.name,
            self.lock,
            base64::encode(&self.mnemonic),
            base64::encode(&self.secret),
            base64::encode(&self.encrypt),
            base64::encode(&self.avatar),
            self.cloud.to_hex(),
            hex::encode(&self.cloud_key),
            self.pub_height,
            self.own_height,
            self.event.to_hex(),
            self.datetime,
        );
            let id = db.insert(&sql)?;
            self.id = id;
        }
        Ok(())
    }

    pub fn update(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("UPDATE accounts SET name='{}', lock='{}', encrypt='{}', avatar='{}', cloud='{}', cloud_key='{}', pub_height={}, own_height={}, event='{}', datetime={} WHERE id = {}",
            self.name,
            self.lock,
            base64::encode(&self.encrypt),
            base64::encode(&self.avatar),
            self.cloud.to_hex(),
            hex::encode(&self.cloud_key),
            self.pub_height,
            self.own_height,
            self.datetime,
            self.event.to_hex(),
            self.id,
        );
        db.update(&sql)
    }

    pub fn update_info(&self, db: &DStorage) -> Result<usize> {
        let sql = format!(
            "UPDATE accounts SET name='{}', avatar='{}', cloud='{}', cloud_key='{}', pub_height={} WHERE id = {}",
            self.name,
            base64::encode(&self.avatar),
            self.cloud.to_hex(),
            hex::encode(&self.cloud_key),
            self.pub_height,
            self.id,
        );
        db.update(&sql)
    }

    pub fn _delete(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("DELETE FROM accounts WHERE id = {}", self.id);
        db.delete(&sql)
    }

    pub fn _update_consensus(&mut self, db: &DStorage, height: u64, eid: EventId) -> Result<usize> {
        self.own_height = height;
        self.event = eid;
        let sql = format!(
            "UPDATE accounts SET own_height={}, event='{}' WHERE id = {}",
            self.own_height,
            self.event.to_hex(),
            self.id,
        );
        db.update(&sql)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct User {
    pub height: u64,
    pub name: String,
    pub cloud: PeerId,
    pub cloud_key: [u8; 32],
    pub avatar: Vec<u8>,
}

impl User {
    pub fn info(
        height: u64,
        name: String,
        cloud: PeerId,
        cloud_key: [u8; 32],
        avatar: Vec<u8>,
    ) -> Self {
        Self {
            height,
            name,
            cloud,
            cloud_key,
            avatar,
        }
    }
}
