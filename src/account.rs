use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::EventId,
    primitives::{PeerId, PeerKey, Result},
};
use tdn_did::{generate_eth_account, generate_peer, Language};
use tdn_storage::local::{DStorage, DsValue};
use web3::signing::Key;

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
    pub lock: String,     // hashed-lock.
    pub secret: Vec<u8>,  // encrypted value.
    pub encrypt: Vec<u8>, // encrypted encrypt key.
    pub wallet: String,   // main wallet info.
    pub pub_height: u64,  // public information height.
    pub own_height: u64,  // own data consensus height.
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
        wallet: String,
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
            wallet,
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
        let sk = generate_peer(
            lang,
            mnemonic,
            index,
            0, // account default multiple address index is 0.
            if pass.len() > 0 { Some(pass) } else { None },
        )?;

        // Default ETH wallet account.
        let wallet_pass = if pass.len() > 0 { Some(pass) } else { None };
        let wallet_sk = generate_eth_account(lang, mnemonic, index, 0, wallet_pass)?;
        let wallet_address = format!("{:?}", (&wallet_sk).address());
        let wallet = ChainToken::ETH.update_main(&wallet_address, "");
        let w = Address::new(ChainToken::ETH, 0, wallet_address, true);

        let key = rand::thread_rng().gen::<[u8; 32]>();
        let ckey = encrypt_key(salt, lock, &key)?;
        let mut ebytes = encrypt_multiple(
            salt,
            lock,
            &ckey,
            vec![&sk.to_db_bytes(), mnemonic.as_bytes()],
        )?;
        let mnemonic = ebytes.pop().unwrap_or(vec![]);
        let secret = ebytes.pop().unwrap_or(vec![]);
        let index = index as i64;

        Ok((
            Account::new(
                sk.peer_id(),
                index,
                rlang,
                pass.to_string(),
                name.to_string(),
                hash_pin(lock)?,
                avatar,
                mnemonic,
                secret,
                ckey,
                key.to_vec(),
                wallet,
            ),
            sk,
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
            wallet: v.pop().unwrap().as_string(),
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
            "SELECT id, pid, indx, lang, pass, name, lock, mnemonic, secret, encrypt, avatar, wallet, pub_height, own_height, event, datetime FROM accounts WHERE pid = '{}'",
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
            "SELECT id, pid, indx, lang, pass, name, lock, mnemonic, secret, encrypt, avatar, wallet, pub_height, own_height, event, datetime FROM accounts ORDER BY datetime DESC",
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
            let sql = format!("INSERT INTO accounts (pid, indx, lang, pass, name, lock, mnemonic, secret, encrypt, avatar, wallet, pub_height, own_height, event, datetime) VALUES ('{}', {}, {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, {}, '{}', {})",
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
            self.wallet,
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
        let sql = format!("UPDATE accounts SET name='{}', lock='{}', encrypt='{}', avatar='{}', wallet='{}', pub_height={}, own_height={}, event='{}', datetime={} WHERE id = {}",
            self.name,
            self.lock,
            base64::encode(&self.encrypt),
            base64::encode(&self.avatar),
            self.wallet,
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
            "UPDATE accounts SET name='{}', avatar='{}', wallet='{}', pub_height={} WHERE id = {}",
            self.name,
            base64::encode(&self.avatar),
            self.wallet,
            self.pub_height,
            self.id,
        );
        db.update(&sql)
    }

    pub fn _delete(&self, db: &DStorage) -> Result<usize> {
        let sql = format!("DELETE FROM accounts WHERE id = {}", self.id);
        db.delete(&sql)
    }

    pub fn update_consensus(&mut self, db: &DStorage, height: u64, eid: EventId) -> Result<usize> {
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
    pub id: PeerId,
    pub name: String,
    pub wallet: String,
    pub height: u64,
    pub avatar: Vec<u8>,
}

impl User {
    pub fn new(id: PeerId, name: String, avatar: Vec<u8>, wallet: String, height: u64) -> Self {
        Self {
            id,
            name,
            avatar,
            wallet,
            height,
        }
    }

    pub fn info(name: String, wallet: String, height: u64, avatar: Vec<u8>) -> Self {
        Self {
            id: PeerId::default(),
            name,
            wallet,
            height,
            avatar,
        }
    }
}
