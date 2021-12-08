use tdn::types::{
    primitive::Result,
    rpc::{json, RpcParam},
};

use tdn_storage::local::{DStorage, DsValue};

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

pub(crate) struct Token {
    pub id: i64,
    pub chain: ChainToken,
    pub contract: String,
    pub decimal: i64,
}

pub(crate) struct Address {
    pub id: i64,
    pub chain: ChainToken,
    pub index: i64,
    pub name: String,
    pub address: String,
    /// Encrypted secret key.
    /// if this address is imported, has this field,
    /// if this address is generated, no this field.
    pub secret: String,
}

impl Address {
    pub fn is_gen(&self) -> bool {
        self.secret.len() == 0
    }

    pub fn new(chain: ChainToken, index: i64, address: String) -> Self {
        Self {
            chain,
            index,
            address,
            name: format!("Account {}", index),
            secret: "".to_owned(),
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
        ])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            secret: v.pop().unwrap().as_string(),
            address: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            index: v.pop().unwrap().as_i64(),
            chain: ChainToken::from_i64(v.pop().unwrap().as_i64()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!(
            "INSERT INTO addresses (chain, indx, name, address, secret) VALUES ({}, {}, '{}', '{}', '{}')",
            self.chain.to_i64(),
            self.index,
            self.name,
            self.address,
            self.secret,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn list(db: &DStorage) -> Result<Vec<Self>> {
        let matrix = db.query(&format!(
            "SELECT id, chain, indx, name, address, secret FROM addresses"
        ))?;
        let mut addresses = vec![];
        for values in matrix {
            addresses.push(Self::from_values(values));
        }
        Ok(addresses)
    }

    pub fn next_index(db: &DStorage, chain: &ChainToken) -> Result<u32> {
        let mut matrix = db.query(&format!(
            "SELECT indx FROM addresses where chain = {} AND secret = '' ORDER BY indx DESC",
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

    pub fn _delete(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!("DELETE FROM addresses WHERE id = {}", id);
        db.delete(&sql)?;
        Ok(())
    }
}
