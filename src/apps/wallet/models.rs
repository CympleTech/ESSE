use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
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
    fn to_i64(&self) -> i64 {
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
}

impl Address {
    pub fn new(chain: ChainToken, index: i64, address: String) -> Self {
        Self {
            chain,
            index,
            address,
            name: format!("Account {}", index),
            id: 0,
        }
    }

    pub fn to_rpc(&self) -> RpcParam {
        json!([self.id, self.chain.to_i64(), self.index, self.address])
    }

    fn from_values(mut v: Vec<DsValue>) -> Self {
        Self {
            address: v.pop().unwrap().as_string(),
            name: v.pop().unwrap().as_string(),
            index: v.pop().unwrap().as_i64(),
            chain: ChainToken::from_i64(v.pop().unwrap().as_i64()),
            id: v.pop().unwrap().as_i64(),
        }
    }

    pub fn insert(&mut self, db: &DStorage) -> Result<()> {
        let sql = format!(
            "INSERT INTO addresses (chain, indx, name, address) VALUES ({}, {}, '{}', '{}')",
            self.chain.to_i64(),
            self.index,
            self.name,
            self.address,
        );
        let id = db.insert(&sql)?;
        self.id = id;
        Ok(())
    }

    pub fn list(db: &DStorage) -> Result<Vec<Self>> {
        let matrix = db.query(&format!(
            "SELECT id, chain, indx, name, address FROM addresses"
        ))?;
        let mut addresses = vec![];
        for values in matrix {
            addresses.push(Self::from_values(values));
        }
        Ok(addresses)
    }

    pub fn delete(db: &DStorage, id: &i64) -> Result<()> {
        let sql = format!("DELETE FROM addresses WHERE id = {}", id);
        db.delete(&sql)?;
        Ok(())
    }
}
