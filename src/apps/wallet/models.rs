use std::collections::HashMap;
use tdn::types::{
    primitive::Result,
    rpc::{json, RpcParam},
};

use tdn_storage::local::{DStorage, DsValue};

#[rustfmt::skip]
pub const ETH_NODE: &'static str =
    "https://mainnet.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161";
#[rustfmt::skip]
pub const ETH_ROPSTEN: &'static str =
    "https://ropsten.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161";
#[rustfmt::skip]
pub const ETH_RINKEBY: &'static str =
    "https://rinkeby.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161";
#[rustfmt::skip]
pub const ETH_KOVAN: &'static str =
    "https://kovan.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161";
#[rustfmt::skip]
pub const ETH_LOCAL: &'static str =
    "http://localhost:8545";

#[derive(Eq, PartialEq)]
pub(crate) enum ChainToken {
    ETH,
    ERC20,
    ERC721,
    BTC,
}

impl ChainToken {
    pub fn to_i64(&self) -> i64 {
        match self {
            ChainToken::ETH => 1,
            ChainToken::ERC20 => 2,
            ChainToken::ERC721 => 3,
            ChainToken::BTC => 4,
        }
    }

    pub fn from_i64(i: i64) -> Self {
        match i {
            1 => ChainToken::ETH,
            2 => ChainToken::ERC20,
            3 => ChainToken::ERC721,
            4 => ChainToken::BTC,
            _ => ChainToken::ETH,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Network {
    EthMain,
    EthTestRopsten,
    EthTestRinkeby,
    EthTestKovan,
    EthLocal,
    BtcMain,
    BtcLocal,
}

impl Network {
    pub fn node<'a>(&self) -> &'a str {
        // TODO more.
        match self {
            Network::EthMain => ETH_NODE,
            Network::EthTestRopsten => ETH_ROPSTEN,
            Network::EthTestRinkeby => ETH_RINKEBY,
            Network::EthTestKovan => ETH_KOVAN,
            Network::EthLocal => ETH_LOCAL,
            Network::BtcMain => ETH_NODE,
            Network::BtcLocal => ETH_NODE,
        }
    }

    pub fn chain(&self) -> ChainToken {
        match self {
            Network::EthMain
            | Network::EthTestRopsten
            | Network::EthTestRinkeby
            | Network::EthTestKovan
            | Network::EthLocal => ChainToken::ETH,
            Network::BtcMain | Network::BtcLocal => ChainToken::BTC,
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            Network::EthMain => 1,
            Network::EthTestRopsten => 2,
            Network::EthTestRinkeby => 3,
            Network::EthTestKovan => 4,
            Network::EthLocal => 5,
            Network::BtcMain => 6,
            Network::BtcLocal => 7,
        }
    }

    pub fn from_i64(i: i64) -> Self {
        match i {
            1 => Network::EthMain,
            2 => Network::EthTestRopsten,
            3 => Network::EthTestRinkeby,
            4 => Network::EthTestKovan,
            5 => Network::EthLocal,
            6 => Network::BtcMain,
            7 => Network::BtcLocal,
            _ => Network::EthMain,
        }
    }
}

pub(crate) struct Address {
    pub id: i64,
    pub chain: ChainToken,
    pub index: i64,
    pub main: bool,
    pub name: String,
    pub address: String,
    /// Encrypted secret key.
    /// if this address is imported, has this field,
    /// if this address is generated, no this field.
    pub secret: Vec<u8>,
    pub balance: String,
}

impl Address {
    pub fn is_gen(&self) -> bool {
        self.secret.len() == 0
    }

    fn merge_balance(old: &str, network: &Network, add: &str) -> String {
        let mut balances: HashMap<i64, &str> = old
            .split(",")
            .flat_map(|s| {
                if s.len() > 2 {
                    let mut ss = s.split(":");
                    Some((
                        ss.next().and_then(|s| s.parse().ok()).unwrap_or(0),
                        ss.next().unwrap_or(""),
                    ))
                } else {
                    None
                }
            })
            .collect();
        balances
            .entry(network.to_i64())
            .and_modify(|old| *old = add)
            .or_insert(add);

        let mut last: Vec<String> = vec![];
        for (key, balance) in balances {
            last.push(format!("{}:{}", key, balance));
        }
        last.join(",")
    }

    fn _get_balance(network: &Network, balance: &str) -> String {
        let balances: HashMap<i64, &str> = balance
            .split(",")
            .map(|s| {
                let mut ss = s.split(":");
                (
                    ss.next().and_then(|s| s.parse().ok()).unwrap_or(0),
                    ss.next().unwrap_or(""),
                )
            })
            .collect();
        balances.get(&network.to_i64()).unwrap_or(&"").to_string()
    }

    pub fn new(chain: ChainToken, index: i64, address: String, is_main: bool) -> Self {
        Self {
            chain,
            index,
            address,
            main: is_main,
            name: format!("Account {}", index),
            secret: vec![],
            balance: "".to_owned(),
            id: 0,
        }
    }

    pub fn import(chain: ChainToken, address: String, secret: Vec<u8>) -> Self {
        Self {
            name: format!("Import {}", &address[2..4]),
            chain,
            address,
            secret,
            index: 0,
            main: false,
            balance: "".to_owned(),
            id: 0,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.chain.to_i64(),
            self.index,
            self.name,
            self.address,
            self.is_gen(),
            self.main,
            self.balance,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            balance: v.pop().unwrap().as_string(),
            secret: base64::decode(v.pop().unwrap().as_str()).unwrap_or(vec![]),
            address: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            main: v.pop().unwrap().as_bool(),
            index: v.pop().unwrap().as_i64(),
            chain: ChainToken::from_i64(v.pop().unwrap().as_i64()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let matrix = db.query(&format!(
            "SELECT id FROM addresses WHERE chain = {} AND address = '{}'",
            self.chain.to_i64(),
            self.address
        ))?;
        if matrix.len() > 0 {
            return Ok(());
        }

        let sql = format!(
            "INSERT INTO addresses (chain, indx, main, name, address, secret, balance) VALUES ({}, {}, {}, '{}', '{}', '{}', '{}')",
            self.chain.to_i64(),
            self.index,
            self.main,
            self.name,
            self.address,
            base64::encode(&self.secret),
            self.balance,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn list(db: &DStorage) -> Result<Vec<Self>> {
        let matrix = db
            .query("SELECT id, chain, indx, main, name, address, secret, balance FROM addresses")?;
        let mut addresses = vec![];
        for values in matrix {
            addresses.push(Self::from_values(values));
        }
        Ok(addresses)
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<Self> {
        let mut matrix = db.query(&format!(
            "SELECT id, chain, indx, main, name, address, secret, balance FROM addresses WHERE id = {}",
            id
        ))?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("address is missing!"))
    }

    pub fn get_by_address(db: &DStorage, address: &str) -> Result<Self> {
        let mut matrix = db.query(&format!(
            "SELECT id, chain, indx, main, name, address, secret, balance FROM addresses WHERE address = '{}'",
            address
        ))?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("address is missing!"))
    }

    pub fn next_index(db: &DStorage, chain: &ChainToken) -> Result<u32> {
        let mut matrix = db.query(&format!(
            "SELECT indx FROM addresses where chain = {} AND secret = '' ORDER BY indx ASC",
            chain.to_i64()
        ))?;
        if matrix.len() > 0 {
            let mut values = matrix.pop().unwrap(); // safe unwrap()
            let index = values.pop().unwrap().as_i64() as u32; // safe unwrap()
            return Ok(index + 1);
        } else {
            return Ok(0);
        }
    }

    pub fn update_balance(
        db: &DStorage,
        address: &str,
        network: &Network,
        balance: &str,
    ) -> Result<()> {
        let mut matrix = db.query(&format!(
            "SELECT balance FROM addresses where address = '{}'",
            address
        ))?;
        if matrix.len() > 0 {
            let mut values = matrix.pop().unwrap(); // safe unwrap()
            let old = values.pop().unwrap(); // safe unwrap()
            let new_b = Address::merge_balance(old.as_str(), network, balance);

            let sql = format!(
                "UPDATE addresses SET balance = '{}' WHERE address = '{}'",
                new_b, address
            );
            db.update(&sql)?;
        }

        Ok(())
    }

    pub fn main(db: &DStorage, id: &i64) -> Result<()> {
        db.update("UPDATE addresses SET main = false")?;
        db.update(&format!(
            "UPDATE addresses SET main = true WHERE id = {}",
            id
        ))?;
        Ok(())
    }

    pub fn _delete(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!("DELETE FROM addresses WHERE id = {}", id);
        db.delete(&sql)?;
        Balance::delete_by_address(db, id)?;
        Ok(())
    }
}

pub(crate) struct Token {
    pub id: i64,
    pub chain: ChainToken,
    pub network: Network,
    pub name: String,
    pub contract: String,
    pub decimal: i64,
}

impl Token {
    pub fn new(
        chain: ChainToken,
        network: Network,
        name: String,
        contract: String,
        decimal: i64,
    ) -> Self {
        Self {
            chain,
            network,
            name,
            contract,
            decimal,
            id: 0,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([
            self.id,
            self.chain.to_i64(),
            self.network.to_i64(),
            self.name,
            self.contract,
            self.decimal,
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            decimal: v.pop().unwrap().as_i64(),
            contract: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            network: Network::from_i64(v.pop().unwrap().as_i64()),
            chain: ChainToken::from_i64(v.pop().unwrap().as_i64()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let matrix = db.query(&format!(
            "SELECT id FROM tokens WHERE network = {} AND contract = '{}'",
            self.network.to_i64(),
            self.contract
        ))?;
        if matrix.len() > 0 {
            return Ok(());
        }

        // check exists
        let sql = format!(
            "INSERT INTO tokens (chain, network, name, contract, decimal) VALUES ({}, {}, '{}', '{}', {})",
            self.chain.to_i64(),
            self.network.to_i64(),
            self.name,
            self.contract,
            self.decimal,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn list(db: &DStorage, network: &Network) -> Result<Vec<Self>> {
        let matrix = db.query(&format!(
            "SELECT id, chain, network, name, contract, decimal FROM tokens where network = {}",
            network.to_i64()
        ))?;
        let mut tokens = vec![];
        for values in matrix {
            tokens.push(Self::from_values(values));
        }
        Ok(tokens)
    }

    pub fn get(db: &DStorage, id: &i64) -> Result<Self> {
        let mut matrix = db.query(&format!(
            "SELECT id, chain, network, name, contract, decimal FROM tokens where id = {}",
            id
        ))?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("token is missing!"))
    }

    pub fn get_by_contract(db: &DStorage, network: &Network, c: &str) -> Result<Self> {
        let mut matrix = db.query(&format!(
            "SELECT id, chain, network, name, contract, decimal FROM tokens WHERE network = {} AND contract = '{}'",
            network.to_i64(), c
        ))?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("token is missing!"))
    }

    pub fn _delete(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!("DELETE FROM tokens WHERE id = {}", id);
        db.delete(&sql)?;
        Balance::delete_by_token(db, id)?;
        Ok(())
    }
}

pub(crate) struct Balance {
    id: i64,
    address: i64,
    token: i64,
    pub value: String,
}

impl Balance {
    pub fn new(address: i64, token: i64, value: String) -> Self {
        Self {
            address,
            token,
            value,
            id: 0,
        }
    }
    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            value: v.pop().unwrap().as_string(),
            token: v.pop().unwrap().as_i64(),
            address: v.pop().unwrap().as_i64(),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn list(db: &DStorage, address: &i64, token: &i64) -> Result<Vec<Self>> {
        let matrix = db.query(&format!(
            "SELECT id, address, token, value FROM balances WHERE address = {} AND token = {}",
            address, token
        ))?;
        let mut balances = vec![];
        for values in matrix {
            balances.push(Self::from_values(values));
        }
        Ok(balances)
    }

    /// use for common and erc20.
    pub fn update(db: &DStorage, address: &i64, token: &i64, value: &str) -> Result<()> {
        let matrix = db.query(&format!(
            "SELECT id FROM balances WHERE address = {} AND token = {}",
            address, token,
        ))?;
        if matrix.len() > 0 {
            let sql = format!(
                "UPDATE balances SET value = '{}' WHERE address = {} AND token = {}",
                value, address, token
            );
            db.update(&sql)?;
            return Ok(());
        }

        let sql = format!(
            "INSERT INTO balances (address, token, value) VALUES ({}, {}, '{}')",
            address, token, value,
        );
        let _id = db.insert(&sql)?;
        Ok(())
    }

    /// use for erc721 (NFT).
    pub fn add(db: &DStorage, address: i64, token: i64, value: String) -> Result<Self> {
        let mut matrix = db.query(&format!(
            "SELECT id FROM balances WHERE address = {} AND token = {} AND value = '{}'",
            address, token, value
        ))?;
        if matrix.len() > 0 {
            let id = matrix.pop().unwrap().pop().unwrap().as_i64(); // safe unwrap()
            return Ok(Self {
                id,
                address,
                token,
                value,
            });
        }

        let sql = format!(
            "INSERT INTO balances (address, token, value) VALUES ({}, {}, '{}')",
            address, token, value
        );
        let id = db.insert(&sql)?;
        Ok(Self {
            id,
            address,
            token,
            value,
        })
    }

    pub fn _get(db: &DStorage, id: &i64) -> Result<Self> {
        let mut matrix = db.query(&format!(
            "SELECT id, address, token, value FROM balances where id = {}",
            id
        ))?;
        if matrix.len() > 0 {
            let values = matrix.pop().unwrap(); // safe unwrap()
            return Ok(Self::from_values(values));
        }
        Err(anyhow!("balance is missing!"))
    }

    pub fn delete_by_hash(db: &DStorage, hash: &str) -> Result<()> {
        let sql = format!("DELETE FROM balances WHERE value = '{}'", hash);
        db.delete(&sql)?;
        Ok(())
    }

    pub fn delete_by_address(db: &DStorage, address: &i64) -> Result<()> {
        let sql = format!("DELETE FROM balances WHERE address = {}", address);
        db.delete(&sql)?;
        Ok(())
    }

    pub fn delete_by_token(db: &DStorage, token: &i64) -> Result<()> {
        let sql = format!("DELETE FROM balances WHERE token = {}", token);
        db.delete(&sql)?;
        Ok(())
    }
}
