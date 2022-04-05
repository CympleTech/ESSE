use serde::{Deserialize, Serialize};
use tdn_types::{
    group::GroupId,
    primitives::{new_io_error, PeerId, PEER_ID_LENGTH},
};

/// ESSE chat service default TDN GROUP ID.
pub const ESSE_ID: GroupId = 0;

/// message type use in network.
#[derive(Serialize, Deserialize, Clone)]
pub enum NetworkMessage {
    String(String),                   // content
    Image(Vec<u8>),                   // image bytes.
    File(String, Vec<u8>),            // filename, file bytes.
    Contact(PeerId, String, Vec<u8>), // PeerId, name, avatar bytes.
    Record(Vec<u8>, u32),             // record audio bytes.
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

pub fn id_to_str(peer: &PeerId) -> String {
    bs32::encode(&peer.0)
}

pub fn id_from_str(s: &str) -> std::io::Result<PeerId> {
    let data = bs32::decode(s).ok_or(new_io_error("id from string is failure."))?;
    if data.len() != PEER_ID_LENGTH {
        return Err(new_io_error("id from string is failure."));
    }
    let mut bytes = [0u8; PEER_ID_LENGTH];
    bytes.copy_from_slice(&data);
    Ok(PeerId(bytes))
}

pub mod bs32 {
    use std::cmp::min;

    const RFC4648_ALPHABET: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    const RFC4648_INV_ALPHABET: [i8; 43] = [
        -1, -1, 26, 27, 28, 29, 30, 31, -1, -1, -1, -1, -1, 0, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7,
        8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    ];

    pub fn encode(data: &[u8]) -> String {
        let mut ret = Vec::with_capacity((data.len() + 3) / 4 * 5);

        for chunk in data.chunks(5) {
            let buf = {
                let mut buf = [0u8; 5];
                for (i, &b) in chunk.iter().enumerate() {
                    buf[i] = b;
                }
                buf
            };
            ret.push(RFC4648_ALPHABET[((buf[0] & 0xF8) >> 3) as usize]);
            ret.push(RFC4648_ALPHABET[(((buf[0] & 0x07) << 2) | ((buf[1] & 0xC0) >> 6)) as usize]);
            ret.push(RFC4648_ALPHABET[((buf[1] & 0x3E) >> 1) as usize]);
            ret.push(RFC4648_ALPHABET[(((buf[1] & 0x01) << 4) | ((buf[2] & 0xF0) >> 4)) as usize]);
            ret.push(RFC4648_ALPHABET[(((buf[2] & 0x0F) << 1) | (buf[3] >> 7)) as usize]);
            ret.push(RFC4648_ALPHABET[((buf[3] & 0x7C) >> 2) as usize]);
            ret.push(RFC4648_ALPHABET[(((buf[3] & 0x03) << 3) | ((buf[4] & 0xE0) >> 5)) as usize]);
            ret.push(RFC4648_ALPHABET[(buf[4] & 0x1F) as usize]);
        }

        if data.len() % 5 != 0 {
            let len = ret.len();
            let num_extra = 8 - (data.len() % 5 * 8 + 4) / 5;
            ret.truncate(len - num_extra);
        }

        String::from_utf8(ret).unwrap()
    }

    pub fn decode(data: &str) -> Option<Vec<u8>> {
        if !data.is_ascii() {
            return None;
        }
        let data = data.as_bytes();
        let mut unpadded_data_length = data.len();
        for i in 1..min(6, data.len()) + 1 {
            if data[data.len() - i] != b'=' {
                break;
            }
            unpadded_data_length -= 1;
        }
        let output_length = unpadded_data_length * 5 / 8;
        let mut ret = Vec::with_capacity((output_length + 4) / 5 * 5);
        for chunk in data.chunks(8) {
            let buf = {
                let mut buf = [0u8; 8];
                for (i, &c) in chunk.iter().enumerate() {
                    match RFC4648_INV_ALPHABET
                        .get(c.to_ascii_uppercase().wrapping_sub(b'0') as usize)
                    {
                        Some(&-1) | None => return None,
                        Some(&value) => buf[i] = value as u8,
                    };
                }
                buf
            };
            ret.push((buf[0] << 3) | (buf[1] >> 2));
            ret.push((buf[1] << 6) | (buf[2] << 1) | (buf[3] >> 4));
            ret.push((buf[3] << 4) | (buf[4] >> 1));
            ret.push((buf[4] << 7) | (buf[5] << 2) | (buf[6] >> 3));
            ret.push((buf[6] << 5) | buf[7]);
        }
        ret.truncate(output_length);
        Some(ret)
    }
}
