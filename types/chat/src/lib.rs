use serde::{Deserialize, Serialize};
use tdn_types::{group::GroupId, primitive::PeerId};

/// message type use in network.
#[derive(Serialize, Deserialize, Clone)]
pub enum NetworkMessage {
    String(String),                            // content
    Image(Vec<u8>),                            // image bytes.
    File(String, Vec<u8>),                     // filename, file bytes.
    Contact(String, GroupId, PeerId, Vec<u8>), // name, gid, addr, avatar bytes.
    Record(Vec<u8>, u32),                      // record audio bytes.
    Emoji,
    Phone,
    Video,
    Invite(String),
    Transfer(String),
}

/// common message types.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MessageType {
    String,
    Image,
    File,
    Contact,
    Record,
    Emoji,
    Phone,
    Video,
    Invite,
    Transfer,
}

impl MessageType {
    pub fn to_int(&self) -> i64 {
        match self {
            MessageType::String => 0,
            MessageType::Image => 1,
            MessageType::File => 2,
            MessageType::Contact => 3,
            MessageType::Record => 4,
            MessageType::Emoji => 5,
            MessageType::Phone => 6,
            MessageType::Video => 7,
            MessageType::Invite => 8,
            MessageType::Transfer => 9,
        }
    }

    pub fn from_int(i: i64) -> MessageType {
        match i {
            0 => MessageType::String,
            1 => MessageType::Image,
            2 => MessageType::File,
            3 => MessageType::Contact,
            4 => MessageType::Record,
            5 => MessageType::Emoji,
            6 => MessageType::Phone,
            7 => MessageType::Video,
            8 => MessageType::Invite,
            9 => MessageType::Transfer,
            _ => MessageType::String,
        }
    }
}
