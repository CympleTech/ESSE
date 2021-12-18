use tdn::types::primitive::Result;

mod consensus;
mod group;
mod member;
mod message;
mod provider;
mod request;

// models.
pub(crate) use consensus::{Consensus, ConsensusType};
pub(crate) use group::GroupChat;
pub(crate) use member::Member;
pub(crate) use message::Message;
pub(crate) use provider::Provider;
pub(crate) use request::Request;

pub(crate) use message::{from_network_message, to_network_message};

pub(crate) struct GroupChatKey(Vec<u8>);

impl GroupChatKey {
    pub fn new(value: Vec<u8>) -> Self {
        Self(value)
    }

    pub fn _key(&self) -> &[u8] {
        &self.0
    }

    pub fn _hash(&self) -> Vec<u8> {
        vec![] // TODO
    }

    pub fn from_hex(s: impl ToString) -> Result<Self> {
        let s = s.to_string();
        if s.len() % 2 != 0 {
            return Err(anyhow!("Hex is invalid"));
        }
        let mut value = vec![];

        for i in 0..(s.len() / 2) {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
                .map_err(|_e| anyhow!("Hex is invalid"))?;
            value.push(res);
        }

        Ok(Self(value))
    }

    pub fn to_hex(&self) -> String {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        hex
    }
}
