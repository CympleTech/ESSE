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

pub(crate) struct Address {
    pub id: i64,
    pub index: i64,
    pub chain: ChainToken,
    pub address: String,
    pub public_key: String,
    pub secret_key: String,
}

pub(crate) struct Token {
    pub id: i64,
    pub chain: ChainToken,
    pub contract: String,
    pub decimal: i64,
}

impl Address {
    pub fn generate(index: i64, chain: ChainToken) -> Self {
        let address = String::new();
        let public_key = String::new();
        let secret_key = String::new();

        Self {
            index,
            chain,
            address,
            public_key,
            secret_key,
            id: 0,
        }
    }
}
