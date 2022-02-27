mod friend;
mod message;
mod request;

pub(crate) use self::friend::Friend;
pub(crate) use self::message::{from_model, handle_nmsg, Message};
pub(crate) use self::request::Request;

use chat_types::{MessageType, NetworkMessage};
use esse_primitives::{id_from_str, id_to_str};
use group_types::GroupChatId;
use std::path::PathBuf;
use tdn::types::primitives::{HandleResult, PeerId, Result};

//use crate::apps::group::GroupChat;
use crate::rpc::session_create;
use crate::storage::{
    chat_db, group_db, read_avatar, read_db_file, read_file, read_image, read_record, session_db,
    write_avatar_sync, write_file, write_file_sync, write_image, write_image_sync,
    write_record_sync,
};

pub(crate) async fn from_network_message(
    own: &PeerId,
    base: &PathBuf,
    db_key: &str,
    nmsg: NetworkMessage,
    results: &mut HandleResult,
) -> Result<(MessageType, String)> {
    match nmsg {
        NetworkMessage::String(content) => Ok((MessageType::String, content)),
        NetworkMessage::Transfer(content) => Ok((MessageType::Transfer, content)),
        NetworkMessage::Image(bytes) => {
            let image_name = write_image_sync(base, own, bytes)?;
            Ok((MessageType::Image, image_name))
        }
        NetworkMessage::File(old_name, bytes) => {
            let filename = write_file_sync(base, own, &old_name, bytes)?;
            Ok((MessageType::File, filename))
        }
        NetworkMessage::Contact(pid, name, avatar_bytes) => {
            write_avatar_sync(base, own, &pid, avatar_bytes)?;
            let contact_values = format!("{};;{}", id_to_str(&pid), name);
            Ok((MessageType::Contact, contact_values))
        }
        NetworkMessage::Emoji => {
            // TODO
            Ok((MessageType::Emoji, "".to_owned()))
        }
        NetworkMessage::Record(bytes, time) => {
            let record_name = write_record_sync(base, own, time, bytes)?;
            Ok((MessageType::Record, record_name))
        }
        NetworkMessage::Invite(content) => {
            // check is Tmp group.
            let itype = InviteType::deserialize(&content)?;
            match itype {
                InviteType::Group(gcd, addr, name) => {
                    // 1 add group chat.
                    let db = group_db(base, own, db_key)?;
                    //let mut g = GroupChat::from(gcd, 0, addr, name);
                    //g.insert(&db)?;

                    // 2 add new session.
                    //let mut session = g.to_session();
                    //let s_db = session_db(base, own, db_key)?;
                    //session.insert(&s_db)?;
                    //results.rpcs.push(session_create(&session));
                }
            }

            Ok((MessageType::Invite, content))
        }
        NetworkMessage::Phone => {
            // TODO
            Ok((MessageType::Phone, "".to_owned()))
        }
        NetworkMessage::Video => {
            // TODO
            Ok((MessageType::Video, "".to_owned()))
        }
    }
}

pub(crate) async fn raw_to_network_message(
    own: &PeerId,
    base: &PathBuf,
    db_key: &str,
    mtype: &MessageType,
    content: &str,
) -> Result<(NetworkMessage, String)> {
    match mtype {
        MessageType::String => Ok((
            NetworkMessage::String(content.to_owned()),
            content.to_owned(),
        )),
        MessageType::Image => {
            let bytes = read_file(&PathBuf::from(content)).await?;
            let image_name = write_image(base, own, &bytes).await?;
            Ok((NetworkMessage::Image(bytes), image_name))
        }
        MessageType::File => {
            let file_path = PathBuf::from(content);
            let bytes = read_file(&file_path).await?;
            let old_name = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_owned();
            let filename = write_file(base, own, &old_name, &bytes).await?;
            Ok((NetworkMessage::File(filename.clone(), bytes), filename))
        }
        MessageType::Contact => {
            let cid: i64 = content.parse()?;
            let db = chat_db(base, own, db_key)?;
            let contact = Friend::get(&db, &cid)?;
            drop(db);
            let avatar_bytes = read_avatar(base, own, &contact.pid).await?;
            let contact_values = format!("{};;{}", id_to_str(&contact.pid), contact.name);
            Ok((
                NetworkMessage::Contact(contact.pid, contact.name, avatar_bytes),
                contact_values,
            ))
        }
        MessageType::Record => {
            let (bytes, time) = if let Some(i) = content.find('-') {
                let time = content[0..i].parse().unwrap_or(0);
                let bytes = read_record(base, own, &content[i + 1..]).await?;
                (bytes, time)
            } else {
                (vec![], 0)
            };
            Ok((NetworkMessage::Record(bytes, time), content.to_owned()))
        }
        MessageType::Emoji => {
            // TODO
            Ok((NetworkMessage::Emoji, content.to_owned()))
        }
        MessageType::Phone => {
            // TODO
            Ok((NetworkMessage::Phone, content.to_owned()))
        }
        MessageType::Video => {
            // TODO
            Ok((NetworkMessage::Video, content.to_owned()))
        }
        MessageType::Invite => Ok((
            NetworkMessage::Invite(content.to_owned()),
            content.to_owned(),
        )),
        MessageType::Transfer => Ok((
            NetworkMessage::Transfer(content.to_owned()),
            content.to_owned(),
        )),
    }
}

pub(crate) async fn to_network_message(
    own: &PeerId,
    base: &PathBuf,
    mtype: MessageType,
    content: String,
) -> Result<NetworkMessage> {
    // handle message's type.
    match mtype {
        MessageType::String => Ok(NetworkMessage::String(content)),
        MessageType::Image => {
            let bytes = read_image(base, own, &content).await?;
            Ok(NetworkMessage::Image(bytes))
        }
        MessageType::File => {
            let bytes = read_db_file(base, own, &content).await?;
            Ok(NetworkMessage::File(content, bytes))
        }
        MessageType::Contact => {
            let index = content.find(";;").ok_or(anyhow!("message is invalid"))?;
            if content.len() < index + 2 {
                return Err(anyhow!("message is invalid"));
            }
            let cpid = id_from_str(&content[0..index])?;
            let cname = content[index + 2..].to_owned();

            let avatar_bytes = read_avatar(base, own, &cpid).await?;
            Ok(NetworkMessage::Contact(cpid, cname, avatar_bytes))
        }
        MessageType::Record => {
            let (bytes, time) = if let Some(i) = content.find('-') {
                let time = content[0..i].parse().unwrap_or(0);
                let bytes = read_record(base, own, &content[i + 1..]).await?;
                (bytes, time)
            } else {
                (vec![], 0)
            };
            Ok(NetworkMessage::Record(bytes, time))
        }
        MessageType::Invite => Ok(NetworkMessage::Invite(content)),
        MessageType::Transfer => Ok(NetworkMessage::Transfer(content)),
        MessageType::Emoji => Ok(NetworkMessage::Emoji),
        MessageType::Phone => Ok(NetworkMessage::Phone),
        MessageType::Video => Ok(NetworkMessage::Video),
    }
}

/// clear a message.
pub(crate) async fn _clear_message() -> Result<()> {
    todo!()
}

/// Invite types.
pub(crate) enum InviteType {
    Group(GroupChatId, PeerId, String),
}

impl InviteType {
    pub fn serialize(&self) -> String {
        match self {
            InviteType::Group(gid, addr, name) => {
                let gcd = hex::encode(&gid.to_le_bytes());
                format!("0;;{};;{};;{}", gcd, addr.to_hex(), name)
            }
        }
    }

    pub fn deserialize(s: &str) -> Result<InviteType> {
        match &s[0..3] {
            "0;;" => {
                if s.len() < 103 {
                    // 16(gid) + 64(pid) + 7
                    Err(anyhow!("invite invalid"))
                } else {
                    let mut gid_bytes = [0u8; 8];
                    gid_bytes.copy_from_slice(&hex::decode(&s[3..19])?);
                    let gid = GroupChatId::from_le_bytes(gid_bytes);
                    let addr = PeerId::from_hex(&s[22..86])?;
                    let name = s[88..].to_owned();
                    Ok(InviteType::Group(gid, addr, name))
                }
            }
            _ => Err(anyhow!("invite invalid")),
        }
    }
}
