mod friend;
mod message;
mod request;

pub(crate) use self::friend::Friend;
pub(crate) use self::message::{from_model, handle_nmsg, Message};
pub(crate) use self::request::Request;

use chat_types::{MessageType, NetworkMessage};
use std::path::PathBuf;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::{HandleResult, PeerId, Result},
};
use tokio::sync::RwLock;

use crate::apps::group::GroupChat;
use crate::group::Group;
use crate::rpc::session_create;
use crate::storage::{
    read_avatar, read_db_file, read_file, read_image, read_record, write_avatar_sync, write_file,
    write_file_sync, write_image, write_image_sync, write_record_sync,
};

pub(crate) async fn from_network_message(
    group: &Arc<RwLock<Group>>,
    nmsg: NetworkMessage,
    base: &PathBuf,
    ogid: &GroupId,
    results: &mut HandleResult,
) -> Result<(MessageType, String)> {
    match nmsg {
        NetworkMessage::String(content) => Ok((MessageType::String, content)),
        NetworkMessage::Transfer(content) => Ok((MessageType::Transfer, content)),
        NetworkMessage::Image(bytes) => {
            let image_name = write_image_sync(base, ogid, bytes)?;
            Ok((MessageType::Image, image_name))
        }
        NetworkMessage::File(old_name, bytes) => {
            let filename = write_file_sync(base, ogid, &old_name, bytes)?;
            Ok((MessageType::File, filename))
        }
        NetworkMessage::Contact(name, rgid, addr, avatar_bytes) => {
            write_avatar_sync(base, ogid, &rgid, avatar_bytes)?;
            let tmp_name = name.replace(";", "-;");
            let contact_values = format!("{};;{};;{}", tmp_name, rgid.to_hex(), addr.to_hex());
            Ok((MessageType::Contact, contact_values))
        }
        NetworkMessage::Emoji => {
            // TODO
            Ok((MessageType::Emoji, "".to_owned()))
        }
        NetworkMessage::Record(bytes, time) => {
            let record_name = write_record_sync(base, ogid, time, bytes)?;
            Ok((MessageType::Record, record_name))
        }
        NetworkMessage::Invite(content) => {
            // check is Tmp group.
            let itype = InviteType::deserialize(&content)?;
            match itype {
                InviteType::Group(gcd, addr, name) => {
                    // 1 add group chat.
                    let db = group.read().await.group_db(&ogid)?;
                    let mut g = GroupChat::from(gcd, 0, addr, name);
                    g.insert(&db)?;

                    // 2 add new session.
                    let mut session = g.to_session();
                    let s_db = group.read().await.session_db(&ogid)?;
                    session.insert(&s_db)?;
                    results.rpcs.push(session_create(*ogid, &session));
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
    group: &Arc<RwLock<Group>>,
    base: &PathBuf,
    ogid: &GroupId,
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
            let image_name = write_image(base, ogid, &bytes).await?;
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
            let filename = write_file(base, ogid, &old_name, &bytes).await?;
            Ok((NetworkMessage::File(filename.clone(), bytes), filename))
        }
        MessageType::Contact => {
            let cid: i64 = content.parse()?;
            let db = group.read().await.chat_db(ogid)?;
            let contact = Friend::get(&db, &cid)?;
            drop(db);
            let avatar_bytes = read_avatar(base, ogid, &contact.gid).await?;
            let tmp_name = contact.name.replace(";", "-;");
            let contact_values = format!(
                "{};;{};;{}",
                tmp_name,
                contact.gid.to_hex(),
                contact.addr.to_hex()
            );
            Ok((
                NetworkMessage::Contact(contact.name, contact.gid, contact.addr, avatar_bytes),
                contact_values,
            ))
        }
        MessageType::Record => {
            let (bytes, time) = if let Some(i) = content.find('-') {
                let time = content[0..i].parse().unwrap_or(0);
                let bytes = read_record(base, ogid, &content[i + 1..]).await?;
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
    base: &PathBuf,
    gid: &GroupId,
    mtype: MessageType,
    content: String,
) -> Result<NetworkMessage> {
    // handle message's type.
    match mtype {
        MessageType::String => Ok(NetworkMessage::String(content)),
        MessageType::Image => {
            let bytes = read_image(base, gid, &content).await?;
            Ok(NetworkMessage::Image(bytes))
        }
        MessageType::File => {
            let bytes = read_db_file(base, gid, &content).await?;
            Ok(NetworkMessage::File(content, bytes))
        }
        MessageType::Contact => {
            let v: Vec<&str> = content.split(";;").collect();
            if v.len() != 3 {
                return Err(anyhow!("message is invalid"));
            }
            let cname = v[0].to_owned();
            let cgid = GroupId::from_hex(v[1])?;
            let caddr = PeerId::from_hex(v[2])?;
            let avatar_bytes = read_avatar(base, gid, &cgid).await?;
            Ok(NetworkMessage::Contact(cname, cgid, caddr, avatar_bytes))
        }
        MessageType::Record => {
            let (bytes, time) = if let Some(i) = content.find('-') {
                let time = content[0..i].parse().unwrap_or(0);
                let bytes = read_record(base, gid, &content[i + 1..]).await?;
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

pub(crate) async fn _clear_message(
    _base: &PathBuf,
    _ogid: &GroupId,
    _mtype: &MessageType,
    _content: &str,
) -> Result<()> {
    todo!()
}

/// Invite types.
pub(crate) enum InviteType {
    Group(GroupId, PeerId, String),
}

impl InviteType {
    pub fn serialize(&self) -> String {
        match self {
            InviteType::Group(gcd, addr, name) => {
                format!("0;;{};;{};;{}", gcd.to_hex(), addr.to_hex(), name)
            }
        }
    }

    pub fn deserialize(s: &str) -> Result<InviteType> {
        match &s[0..3] {
            "0;;" => {
                if s.len() < 134 {
                    Err(anyhow!("invite invalid"))
                } else {
                    let gcd = GroupId::from_hex(&s[3..67])?;
                    let addr = PeerId::from_hex(&s[69..133])?;
                    let name = s[135..].to_owned();
                    Ok(InviteType::Group(gcd, addr, name))
                }
            }
            _ => Err(anyhow!("invite invalid")),
        }
    }
}
