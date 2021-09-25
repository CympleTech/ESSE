use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tdn::types::{
    group::{EventId, GroupId},
    message::{RecvType, SendType},
    primitive::{DeliveryType, HandleResult, PeerAddr, Result},
    rpc::RpcError,
};
use tdn_did::{user::User, Proof};
use tokio::sync::RwLock;

use crate::event::InnerEvent;
use crate::layer::{Layer, Online};
use crate::migrate::consensus::{FRIEND_TABLE_PATH, MESSAGE_TABLE_PATH, REQUEST_TABLE_PATH};
use crate::rpc::{session_connect, session_create, session_last, session_lost, session_suspend};
use crate::session::{connect_session, Session, SessionType};
use crate::storage::{
    chat_db, read_avatar, read_file, read_record, session_db, write_avatar_sync, write_file,
    write_image,
};

use super::models::{Friend, Message, MessageType, NetworkMessage, Request};
use super::rpc;

/// ESSE chat layer Event.
#[derive(Serialize, Deserialize)]
pub(crate) enum LayerEvent {
    /// offline. extend BaseLayerEvent.
    Offline(GroupId),
    /// suspend. extend BaseLayerEvent.
    Suspend(GroupId),
    /// actived. extend BaseLayerEvent.
    Actived(GroupId),
    /// make friendship request.
    Request(User, String),
    /// agree friendship request.
    Agree(User, Proof),
    /// reject friendship request.
    Reject,
    /// receiver gid, sender gid, message.
    Message(EventId, NetworkMessage),
    /// receiver gid, sender user.
    Info(User),
    /// close friendship.
    Close,
}

pub(crate) async fn handle(
    arc_layer: &Arc<RwLock<Layer>>,
    fgid: GroupId,
    mgid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    println!("---------DEBUG--------- GOT CHAT EVENT");
    let mut results = HandleResult::new();
    let mut layer = arc_layer.write().await;

    match msg {
        RecvType::Leave(addr) => {
            for (mgid, running) in &mut layer.runnings {
                for sid in running.peer_leave(&addr) {
                    results.rpcs.push(session_lost(*mgid, &sid));
                }
            }
        }
        RecvType::Connect(addr, data) | RecvType::ResultConnect(addr, data) => {
            // ESSE chat layer connect date structure.
            if handle_connect(&mgid, &fgid, &addr, data, &mut layer, &mut results)? {
                let proof = layer.group.read().await.prove_addr(&mgid, &addr)?;
                let data = bincode::serialize(&proof).unwrap_or(vec![]);
                let msg = SendType::Result(0, addr, true, false, data);
                results.layers.push((mgid, fgid, msg));
            } else {
                let msg = SendType::Result(0, addr, false, false, vec![]);
                results.layers.push((mgid, fgid, msg));
            }
        }
        RecvType::Result(addr, is_ok, data) => {
            // ESSE chat layer result date structure.
            if is_ok {
                if !handle_connect(&mgid, &fgid, &addr, data, &mut layer, &mut results)? {
                    let msg = SendType::Result(0, addr, false, false, vec![]);
                    results.layers.push((mgid, fgid, msg));
                }
            } else {
                let db = chat_db(&layer.base, &mgid)?;
                if let Some(friend) = Friend::get_it(&db, &fgid)? {
                    if friend.contains_addr(&addr) {
                        results.rpcs.push(rpc::friend_close(mgid, friend.id));
                        friend.close(&db)?;
                    }
                }
            }
        }
        RecvType::Event(addr, bytes) => {
            return LayerEvent::handle(fgid, mgid, &mut layer, addr, bytes).await;
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }
        RecvType::Delivery(t, tid, is_ok) => {
            println!("delivery: tid: {}, is_ok: {}", tid, is_ok);
            // TODO maybe send failure need handle.
            if is_ok {
                if let Some((gid, db_id)) = layer.delivery.remove(&tid) {
                    let db = chat_db(&layer.base, &mgid)?;
                    let resp = match t {
                        DeliveryType::Event => {
                            Message::delivery(&db, db_id, true)?;
                            rpc::message_delivery(gid, db_id, true)
                        }
                        DeliveryType::Connect => {
                            // request.
                            Request::delivery(&db, db_id, true)?;
                            rpc::request_delivery(gid, db_id, true)
                        }
                        DeliveryType::Result => {
                            // response. TODO better for it.
                            Request::delivery(&db, db_id, true)?;
                            rpc::request_delivery(gid, db_id, true)
                        }
                    };
                    drop(db);
                    results.rpcs.push(resp);
                }
            }
        }
    }

    Ok(results)
}

fn handle_connect(
    mgid: &GroupId,
    fgid: &GroupId,
    addr: &PeerAddr,
    data: Vec<u8>,
    layer: &mut Layer,
    results: &mut HandleResult,
) -> Result<bool> {
    // 0. deserialize connect data.
    let proof: Proof = bincode::deserialize(&data)?;

    // 1. check verify.
    proof.verify(fgid, addr, &layer.addr)?;

    // 2. check friendship.
    let friend = update_friend(&layer.base, mgid, fgid, addr)?;
    if friend.is_none() {
        return Ok(false);
    }
    let fid = friend.unwrap().id; // safe.

    // 3. get session.
    let session_some = connect_session(&layer.base, mgid, &SessionType::Chat, &fid, addr)?;
    if session_some.is_none() {
        return Ok(false);
    }
    let sid = session_some.unwrap().id;

    // 4. active this session.
    layer
        .running_mut(mgid)?
        .check_add_online(*fgid, Online::Direct(*addr), sid, fid)?;

    // 5. session online to UI.
    results.rpcs.push(session_connect(*mgid, &sid, addr));
    Ok(true)
}

impl LayerEvent {
    pub async fn handle(
        fgid: GroupId,
        mgid: GroupId,
        layer: &mut Layer,
        addr: PeerAddr,
        bytes: Vec<u8>,
    ) -> Result<HandleResult> {
        let event: LayerEvent = bincode::deserialize(&bytes)?;

        let mut results = HandleResult::new();

        match event {
            LayerEvent::Offline(_) => {
                let (sid, _fid) = layer.get_running_remote_id(&mgid, &fgid)?;
                layer.running_mut(&mgid)?.check_offline(&fgid, &addr);
                results.rpcs.push(session_lost(mgid, &sid));
            }
            LayerEvent::Suspend(_) => {
                let (sid, _fid) = layer.get_running_remote_id(&mgid, &fgid)?;
                if layer.running_mut(&mgid)?.suspend(&fgid, false, false)? {
                    results.rpcs.push(session_suspend(mgid, &sid));
                }
            }
            LayerEvent::Actived(_) => {
                let (sid, _fid) = layer.get_running_remote_id(&mgid, &fgid)?;
                let _ = layer.running_mut(&mgid)?.active(&fgid, false);
                results.rpcs.push(session_connect(mgid, &sid, &addr));
            }
            LayerEvent::Request(remote, remark) => {
                if load_friend(&layer.base, &mgid, &fgid)?.is_none() {
                    // check if exist request.
                    let db = chat_db(&layer.base, &mgid)?;
                    if let Some(req) = Request::get(&db, &remote.id)? {
                        req.delete(&db)?; // delete the old request.
                        results.rpcs.push(rpc::request_delete(mgid, req.id));
                    }
                    let mut request = Request::new(
                        remote.id,
                        remote.addr,
                        remote.name.clone(),
                        remark.clone(),
                        false,
                        true,
                    );
                    // save to db.
                    request.insert(&db)?;
                    drop(db);
                    // save the avatar.
                    write_avatar_sync(&layer.base, &mgid, &request.gid, remote.avatar.clone())?;

                    layer.group.write().await.broadcast(
                        &mgid,
                        InnerEvent::SessionRequestCreate(false, remote, remark),
                        REQUEST_TABLE_PATH,
                        request.id,
                        &mut results,
                    )?;

                    results.rpcs.push(rpc::request_create(mgid, &request));
                    return Ok(results);
                } else {
                    let group_lock = layer.group.read().await;
                    let me = group_lock.clone_user(&mgid)?;
                    let proof = group_lock.prove_addr(&mgid, &addr)?;
                    drop(group_lock);
                    let msg = agree_message(proof, me, addr)?;
                    results.layers.push((mgid, fgid, msg));
                }
            }
            LayerEvent::Agree(remote, proof) => {
                // 0. check verify.
                proof.verify(&fgid, &addr, &layer.addr)?;
                // 1. check friendship.
                if load_friend(&layer.base, &mgid, &fgid)?.is_none() {
                    // agree request for friend.
                    let db = chat_db(&layer.base, &mgid)?;
                    if let Some(mut request) = Request::get(&db, &remote.id)? {
                        layer.group.write().await.broadcast(
                            &mgid,
                            InnerEvent::SessionRequestHandle(
                                request.gid,
                                true,
                                remote.avatar.clone(),
                            ),
                            REQUEST_TABLE_PATH,
                            request.id,
                            &mut results,
                        )?;
                        request.is_over = true;
                        request.is_ok = true;
                        request.update(&db)?;
                        let request_id = request.id;
                        let friend = Friend::from_request(&db, request)?;
                        write_avatar_sync(&layer.base, &mgid, &remote.id, remote.avatar)?;
                        results
                            .rpcs
                            .push(rpc::request_agree(mgid, request_id, &friend));

                        // ADD NEW SESSION.
                        let s_db = session_db(&layer.base, &mgid)?;
                        let mut session = friend.to_session();
                        session.insert(&s_db)?;
                        results.rpcs.push(session_create(mgid, &session));
                    }
                    drop(db);
                }
            }
            LayerEvent::Reject => {
                let db = chat_db(&layer.base, &mgid)?;
                if let Some(mut request) = Request::get(&db, &fgid)? {
                    layer.group.write().await.broadcast(
                        &mgid,
                        InnerEvent::SessionRequestHandle(request.gid, false, vec![]),
                        REQUEST_TABLE_PATH,
                        request.id,
                        &mut results,
                    )?;
                    request.is_over = true;
                    request.is_ok = false;
                    request.update(&db)?;
                    results.rpcs.push(rpc::request_reject(mgid, request.id));
                }
                drop(db);
            }
            LayerEvent::Message(hash, m) => {
                let (_sid, fid) = layer.get_running_remote_id(&mgid, &fgid)?;
                let db = chat_db(&layer.base, &mgid)?;
                if !Message::exist(&db, &hash)? {
                    let (msg, scontent) =
                        m.clone().handle(false, mgid, &layer.base, &db, fid, hash)?;
                    layer.group.write().await.broadcast(
                        &mgid,
                        InnerEvent::SessionMessageCreate(fgid, false, hash, m),
                        MESSAGE_TABLE_PATH,
                        msg.id,
                        &mut results,
                    )?;
                    results.rpcs.push(rpc::message_create(mgid, &msg));

                    // UPDATE SESSION.
                    let s_db = session_db(&layer.base, &mgid)?;
                    if let Ok(id) = Session::last(
                        &s_db,
                        &fid,
                        &SessionType::Chat,
                        &msg.datetime,
                        &scontent,
                        true,
                    ) {
                        results
                            .rpcs
                            .push(session_last(mgid, &id, &msg.datetime, &scontent, false));
                    }
                }
            }
            LayerEvent::Info(remote) => {
                let (_sid, fid) = layer.get_running_remote_id(&mgid, &fgid)?;
                let avatar = remote.avatar.clone();
                let db = chat_db(&layer.base, &mgid)?;
                let mut f = Friend::get_id(&db, fid)?.ok_or(anyhow!("friend not found"))?;
                f.name = remote.name;
                f.addr = remote.addr;
                f.remote_update(&db)?;
                drop(db);
                write_avatar_sync(&layer.base, &mgid, &remote.id, remote.avatar)?;

                layer.group.write().await.broadcast(
                    &mgid,
                    InnerEvent::SessionFriendInfo(f.gid, f.addr, f.name.clone(), avatar),
                    FRIEND_TABLE_PATH,
                    f.id,
                    &mut results,
                )?;
                results.rpcs.push(rpc::friend_info(mgid, &f));
            }
            LayerEvent::Close => {
                let (_sid, fid) = layer.get_running_remote_id(&mgid, &fgid)?;
                layer.group.write().await.broadcast(
                    &mgid,
                    InnerEvent::SessionFriendClose(fgid),
                    FRIEND_TABLE_PATH,
                    fid,
                    &mut results,
                )?;
                layer.remove_online(&mgid, &fgid);
                let db = chat_db(&layer.base, &mgid)?;
                Friend::id_close(&db, fid)?;
                drop(db);
                results.rpcs.push(rpc::friend_close(mgid, fid));
                if !layer.is_online(&addr) {
                    results
                        .layers
                        .push((mgid, fgid, SendType::Disconnect(addr)))
                }
            }
        }

        Ok(results)
    }

    pub async fn from_message(
        base: &PathBuf,
        mgid: GroupId,
        fid: i64,
        m_type: MessageType,
        content: String,
    ) -> std::result::Result<(Message, NetworkMessage, String), tdn::types::rpc::RpcError> {
        let db = chat_db(&base, &mgid)?;

        // handle message's type.
        let (nm_type, raw) = match m_type {
            MessageType::String => (NetworkMessage::String(content.clone()), content),
            MessageType::Image => {
                let bytes = read_file(&PathBuf::from(content)).await?;
                let image_name = write_image(base, &mgid, &bytes).await?;
                (NetworkMessage::Image(bytes), image_name)
            }
            MessageType::File => {
                let file_path = PathBuf::from(content);
                let bytes = read_file(&file_path).await?;
                let old_name = file_path
                    .file_name()
                    .ok_or(RpcError::ParseError)?
                    .to_str()
                    .ok_or(RpcError::ParseError)?;
                let filename = write_file(base, &mgid, old_name, &bytes).await?;
                (NetworkMessage::File(filename.clone(), bytes), filename)
            }
            MessageType::Contact => {
                let cid: i64 = content.parse().map_err(|_| anyhow!("parse i64 failure!"))?;
                let contact = Friend::get_id(&db, cid)?.ok_or(RpcError::ParseError)?;
                let avatar_bytes = read_avatar(base, &mgid, &contact.gid).await?;
                let tmp_name = contact.name.replace(";", "-;");
                let contact_values = format!(
                    "{};;{};;{}",
                    tmp_name,
                    contact.gid.to_hex(),
                    contact.addr.to_hex()
                );
                (
                    NetworkMessage::Contact(contact.name, contact.gid, contact.addr, avatar_bytes),
                    contact_values,
                )
            }
            MessageType::Record => {
                let (bytes, time) = if let Some(i) = content.find('-') {
                    let time = content[0..i].parse().unwrap_or(0);
                    let bytes = read_record(base, &mgid, &content[i + 1..]).await?;
                    (bytes, time)
                } else {
                    (vec![], 0)
                };
                (NetworkMessage::Record(bytes, time), content)
            }
            MessageType::Emoji => {
                // TODO
                (NetworkMessage::Emoji, content)
            }
            MessageType::Phone => {
                // TODO
                (NetworkMessage::Phone, content)
            }
            MessageType::Video => {
                // TODO
                (NetworkMessage::Video, content)
            }
            MessageType::Invite => (NetworkMessage::Invite(content.clone()), content),
        };

        let scontent = match m_type {
            MessageType::String => {
                format!("{}:{}", m_type.to_int(), raw)
            }
            _ => format!("{}:", m_type.to_int()),
        };

        let mut msg = Message::new(&mgid, fid, true, m_type, raw, false);
        msg.insert(&db)?;
        drop(db);
        Ok((msg, nm_type, scontent))
    }
}

#[inline]
fn load_friend(base: &PathBuf, mgid: &GroupId, fgid: &GroupId) -> Result<Option<Friend>> {
    let db = chat_db(base, mgid)?;
    Friend::get(&db, fgid)
}

#[inline]
fn update_friend(
    base: &PathBuf,
    mgid: &GroupId,
    fgid: &GroupId,
    addr: &PeerAddr,
) -> Result<Option<Friend>> {
    let db = chat_db(base, mgid)?;
    if let Some(friend) = Friend::get(&db, fgid)? {
        if &friend.addr != addr {
            let _ = Friend::addr_update(&db, friend.id, addr);
        }
        Ok(Some(friend))
    } else {
        Ok(None)
    }
}

pub(super) fn req_message(layer: &mut Layer, me: User, request: Request) -> SendType {
    // update delivery.
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (me.id, request.id));
    let req = LayerEvent::Request(me, request.remark);
    let data = bincode::serialize(&req).unwrap_or(vec![]);
    SendType::Event(uid, request.addr, data)
}

pub(super) fn reject_message(
    layer: &mut Layer,
    tid: i64,
    addr: PeerAddr,
    me_id: GroupId,
) -> SendType {
    let data = bincode::serialize(&LayerEvent::Reject).unwrap_or(vec![]);
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (me_id, tid));
    SendType::Event(uid, addr, data)
}

pub(crate) fn event_message(
    layer: &mut Layer,
    tid: i64,
    me_id: GroupId,
    addr: PeerAddr,
    event: &LayerEvent,
) -> SendType {
    let data = bincode::serialize(event).unwrap_or(vec![]);
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (me_id, tid));
    SendType::Event(uid, addr, data)
}

pub(crate) fn chat_conn(proof: Proof, addr: PeerAddr) -> SendType {
    let data = bincode::serialize(&proof).unwrap_or(vec![]);
    SendType::Connect(0, addr, None, None, data)
}

pub(super) fn agree_message(proof: Proof, me: User, addr: PeerAddr) -> Result<SendType> {
    let data = bincode::serialize(&LayerEvent::Agree(me, proof)).unwrap_or(vec![]);
    Ok(SendType::Event(0, addr, data))
}

// maybe need if gid or addr in blocklist.
fn _res_reject() -> Vec<u8> {
    bincode::serialize(&LayerEvent::Reject).unwrap_or(vec![])
}
