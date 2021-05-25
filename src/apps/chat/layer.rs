use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tdn::{
    smol::lock::RwLock,
    types::{
        group::{EventId, GroupId},
        message::{RecvType, SendType},
        primitive::{new_io_error, DeliveryType, HandleResult, PeerAddr, Result},
    },
};
use tdn_did::{user::User, Proof};

use crate::event::{InnerEvent, StatusEvent};
use crate::layer::{Layer, Online};
use crate::migrate::consensus::{FRIEND_TABLE_PATH, MESSAGE_TABLE_PATH, REQUEST_TABLE_PATH};
use crate::storage::{
    chat_db, read_avatar, read_file, read_record, write_avatar_sync, write_file, write_image,
};

use super::models::{Friend, Message, MessageType, NetworkMessage, Request};
use super::rpc;

/// Layer Request for stable connected.
/// Params: User, remote_id, remark.
/// this user if already friend, only has gid.
#[derive(Serialize, Deserialize)]
enum LayerRequest {
    /// Requst for connect, had friendship.
    /// params: signature with PeerAddr.
    Connect(Proof),
    /// Requst for make friendship.
    /// Params: remote_user, me_id, remark.
    Friend(User, String),
}

/// Layer Response for stable connected.
#[derive(Serialize, Deserialize)]
pub(crate) enum LayerResponse {
    /// Connected with stable, had friendship.
    Connect(Proof),
    /// Agree a friend request.
    /// Params: User, remote_id.
    /// this user if already, only has gid.
    Agree(User, Proof),
    /// Reject a friend request.
    /// Params: me_id, remote_id.
    Reject,
    // TODO service connected info.
    //Service,
}

/// ESSE chat layer Event.
#[derive(Serialize, Deserialize)]
pub(crate) enum LayerEvent {
    /// receiver gid, sender gid. as BaseLayerEvent.
    Offline(GroupId),
    /// receiver gid, sender gid. as BaseLayerEvent.
    OnlinePing,
    /// receiver gid, sender gid. as BaseLayerEvent.
    OnlinePong,
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
    let mut results = HandleResult::new();
    let mut layer = arc_layer.write().await;

    match msg {
        RecvType::Connect(addr, data) => {
            let request: LayerRequest = postcard::from_bytes(&data)
                .map_err(|_e| new_io_error("Deseralize request friend failure"))?;

            match request {
                LayerRequest::Connect(proof) => {
                    let friend = load_friend(&layer.base, &mgid, &fgid)?;
                    if friend.is_none() {
                        let data = postcard::to_allocvec(&LayerResponse::Reject).unwrap_or(vec![]);
                        let msg = SendType::Result(0, addr, false, false, data);
                        results.layers.push((mgid, fgid, msg));
                        return Ok(results);
                    }
                    let f = friend.unwrap(); // safe.

                    // 1. check verify.
                    proof.verify(&fgid, &addr, &layer.addr)?;
                    // 2. online this group.
                    layer
                        .running_mut(&mgid)?
                        .check_add_online(fgid, Online::Direct(addr), f.id)?;
                    // 3. update remote addr. TODO
                    if f.addr != addr {
                        let db = chat_db(&layer.base, &mgid)?;
                        let _ = Friend::addr_update(&db, f.id, &addr);
                        drop(db);
                    }
                    // 4. online to UI.
                    results.rpcs.push(rpc::friend_online(mgid, f.id, addr));
                    // 5. connected.
                    let msg = conn_res_message(&layer, &mgid, addr).await?;
                    results.layers.push((mgid, fgid, msg));
                    layer.group.write().await.status(
                        &mgid,
                        StatusEvent::SessionFriendOnline(fgid),
                        &mut results,
                    )?;
                }
                LayerRequest::Friend(remote, remark) => {
                    let some_friend = load_friend(&layer.base, &mgid, &fgid)?;
                    if some_friend.is_none() {
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
                    }
                    let mut friend = some_friend.unwrap(); // safe checked.

                    // already friendship & update.
                    // 1. online this group.
                    layer.running_mut(&mgid)?.check_add_online(
                        fgid,
                        Online::Direct(addr),
                        friend.id,
                    )?;
                    // 2. update remote user.
                    friend.name = remote.name;
                    friend.addr = remote.addr;
                    let db = chat_db(&layer.base, &mgid)?;
                    friend.remote_update(&db)?;
                    drop(db);
                    write_avatar_sync(&layer.base, &mgid, &remote.id, remote.avatar)?;

                    // 3. online to UI.

                    // TODO UPDATE SESSION

                    results.rpcs.push(rpc::friend_info(mgid, &friend));
                    // 4. connected.
                    let msg = conn_agree_message(&mut layer, 0, &mgid, addr).await?;
                    results.layers.push((mgid, fgid, msg));
                    layer.group.write().await.status(
                        &mgid,
                        StatusEvent::SessionFriendOnline(fgid),
                        &mut results,
                    )?;
                }
            }
        }
        RecvType::Leave(addr) => {
            let group_pin = layer.group.clone();
            let mut group_lock = group_pin.write().await;
            for (mgid, running) in &mut layer.runnings {
                let peers = running.peer_leave(&addr);
                for (fgid, fid) in peers {
                    results.rpcs.push(rpc::friend_offline(*mgid, fid, &fgid));
                    results
                        .rpcs
                        .push(crate::apps::group_chat::rpc::group_offline(
                            *mgid, fid, &fgid,
                        ));

                    group_lock.status(
                        &mgid,
                        StatusEvent::SessionFriendOffline(fgid),
                        &mut results,
                    )?;
                }
            }
        }
        RecvType::Result(addr, is_ok, data) => {
            // check to close.
            if !is_ok {
                let db = chat_db(&layer.base, &mgid)?;
                if let Some(friend) = Friend::get_it(&db, &fgid)? {
                    if friend.contains_addr(&addr) {
                        results.rpcs.push(rpc::friend_close(mgid, friend.id));
                        friend.close(&db)?;
                    }
                }
                drop(db);

                let response: LayerResponse = postcard::from_bytes(&data)
                    .map_err(|_e| new_io_error("Deseralize result failure"))?;
                match response {
                    LayerResponse::Reject => {
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
                    _ => {}
                }

                return Ok(results);
            }

            let response: LayerResponse = postcard::from_bytes(&data)
                .map_err(|_e| new_io_error("Deseralize result failure"))?;

            match response {
                LayerResponse::Connect(proof) => {
                    // 1. check verify.
                    proof.verify(&fgid, &addr, &layer.addr)?;
                    // 2. check has this remove.
                    let some_friend = load_friend(&layer.base, &mgid, &fgid)?;
                    if some_friend.is_none() {
                        return Ok(results);
                    }
                    let fid = some_friend.unwrap().id; // safe.

                    // 3. online this group.
                    layer
                        .running_mut(&mgid)?
                        .check_add_online(fgid, Online::Direct(addr), fid)?;
                    // 4. update remote addr.
                    let db = chat_db(&layer.base, &mgid)?;
                    Friend::addr_update(&db, fid, &addr)?;
                    drop(db);
                    // 5. online to UI.
                    results.rpcs.push(rpc::friend_online(mgid, fid, addr));
                    layer.group.write().await.status(
                        &mgid,
                        StatusEvent::SessionFriendOnline(fgid),
                        &mut results,
                    )?;
                }
                LayerResponse::Agree(remote, proof) => {
                    // 1. check verify.
                    proof.verify(&fgid, &addr, &layer.addr)?;
                    if let Some(friend) = load_friend(&layer.base, &mgid, &fgid)? {
                        // already friendship.
                        layer.running_mut(&mgid)?.check_add_online(
                            fgid,
                            Online::Direct(addr),
                            friend.id,
                        )?;
                        results.rpcs.push(rpc::friend_online(mgid, friend.id, addr));
                        layer.group.write().await.status(
                            &mgid,
                            StatusEvent::SessionFriendOnline(fgid),
                            &mut results,
                        )?;
                    } else {
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
                        }
                        drop(db);
                    }

                    let data = postcard::to_allocvec(&LayerEvent::OnlinePing).unwrap_or(vec![]);
                    let msg = SendType::Event(0, addr, data);
                    results.layers.push((mgid, fgid, msg));
                }
                LayerResponse::Reject => {}
            }
        }
        RecvType::ResultConnect(addr, data) => {
            let response: LayerResponse = postcard::from_bytes(&data)
                .map_err(|_e| new_io_error("Deseralize result failure"))?;

            match response {
                LayerResponse::Connect(proof) => {
                    // 1. check verify.
                    proof.verify(&fgid, &addr, &layer.addr)?;
                    // 2. check has this remove.
                    let some_friend = load_friend(&layer.base, &mgid, &fgid)?;
                    if some_friend.is_none() {
                        return Ok(results);
                    }
                    let fid = some_friend.unwrap().id; // safe.

                    // 3. online this group.
                    layer
                        .running_mut(&mgid)?
                        .check_add_online(fgid, Online::Direct(addr), fid)?;
                    // 4. update remote addr.
                    let db = chat_db(&layer.base, &mgid)?;
                    Friend::addr_update(&db, fid, &addr)?;
                    drop(db);
                    // 5. online to UI.
                    results.rpcs.push(rpc::friend_online(mgid, fid, addr));
                    // 6. connected.
                    let msg = conn_res_message(&layer, &mgid, addr).await?;
                    results.layers.push((mgid, fgid, msg));
                    layer.group.write().await.status(
                        &mgid,
                        StatusEvent::SessionFriendOnline(fgid),
                        &mut results,
                    )?;
                }
                LayerResponse::Agree(remote, proof) => {
                    // 1. check verify.
                    proof.verify(&fgid, &addr, &layer.addr)?;
                    if let Some(friend) = load_friend(&layer.base, &mgid, &fgid)? {
                        // already friendship.
                        layer.running_mut(&mgid)?.check_add_online(
                            fgid,
                            Online::Direct(addr),
                            friend.id,
                        )?;
                        results.rpcs.push(rpc::friend_online(mgid, friend.id, addr));
                        layer.group.write().await.status(
                            &mgid,
                            StatusEvent::SessionFriendOnline(fgid),
                            &mut results,
                        )?;
                    } else {
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
                        }
                        drop(db);
                    }

                    let msg = conn_res_message(&layer, &mgid, addr).await?;
                    results.layers.push((mgid, fgid, msg));
                }
                LayerResponse::Reject => {
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

impl LayerEvent {
    pub async fn handle(
        fgid: GroupId,
        mgid: GroupId,
        layer: &mut Layer,
        addr: PeerAddr,
        bytes: Vec<u8>,
    ) -> Result<HandleResult> {
        let event: LayerEvent =
            postcard::from_bytes(&bytes).map_err(|_| new_io_error("serialize event error."))?;
        let fid = layer.get_running_remote_id(&mgid, &fgid)?;

        let mut results = HandleResult::new();

        match event {
            LayerEvent::Message(hash, m) => {
                let db = chat_db(&layer.base, &mgid)?;
                if !Message::exist(&db, &hash)? {
                    let msg = m.clone().handle(false, mgid, &layer.base, &db, fid, hash)?;
                    layer.group.write().await.broadcast(
                        &mgid,
                        InnerEvent::SessionMessageCreate(fgid, false, hash, m),
                        MESSAGE_TABLE_PATH,
                        msg.id,
                        &mut results,
                    )?;
                    results.rpcs.push(rpc::message_create(mgid, &msg));
                }
            }
            LayerEvent::Info(remote) => {
                let avatar = remote.avatar.clone();
                let db = chat_db(&layer.base, &mgid)?;
                let mut f = Friend::get_id(&db, fid)?.ok_or(new_io_error(""))?;
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
            LayerEvent::OnlinePing => {
                layer.group.write().await.status(
                    &mgid,
                    StatusEvent::SessionFriendOnline(fgid),
                    &mut results,
                )?;
                layer
                    .running_mut(&mgid)?
                    .check_add_online(fgid, Online::Direct(addr), fid)?;
                results.rpcs.push(rpc::friend_online(mgid, fid, addr));
                let data = postcard::to_allocvec(&LayerEvent::OnlinePong).unwrap_or(vec![]);
                let msg = SendType::Event(0, addr, data);
                results.layers.push((mgid, fgid, msg));
            }
            LayerEvent::OnlinePong => {
                layer.group.write().await.status(
                    &mgid,
                    StatusEvent::SessionFriendOnline(fgid),
                    &mut results,
                )?;
                layer
                    .running_mut(&mgid)?
                    .check_add_online(fgid, Online::Direct(addr), fid)?;
                results.rpcs.push(rpc::friend_online(mgid, fid, addr));
            }
            LayerEvent::Offline(_) => {
                layer.group.write().await.status(
                    &mgid,
                    StatusEvent::SessionFriendOffline(fgid),
                    &mut results,
                )?;
                layer.running_mut(&mgid)?.check_offline(&fgid, &addr);
                results.rpcs.push(rpc::friend_offline(mgid, fid, &fgid));
            }
            LayerEvent::Close => {
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
    ) -> std::result::Result<(Message, NetworkMessage), tdn::types::rpc::RpcError> {
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
                let old_name = file_path.file_name()?.to_str()?;
                let filename = write_file(base, &mgid, old_name, &bytes).await?;
                (NetworkMessage::File(filename.clone(), bytes), filename)
            }
            MessageType::Contact => {
                let cid: i64 = content.parse().map_err(|_e| new_io_error("id error"))?;
                let contact = Friend::get_id(&db, cid)??;
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
        };

        let mut msg = Message::new(&mgid, fid, true, m_type, raw, false);
        msg.insert(&db)?;

        // TODO UPDATE SESSION

        drop(db);
        Ok((msg, nm_type))
    }
}

#[inline]
fn load_friend(base: &PathBuf, mgid: &GroupId, fgid: &GroupId) -> Result<Option<Friend>> {
    let db = chat_db(base, mgid)?;
    Friend::get(&db, fgid)
}

pub(super) fn req_message(layer: &mut Layer, me: User, request: Request) -> SendType {
    // update delivery.
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (me.id, request.id));
    let req = LayerRequest::Friend(me, request.remark);
    let data = postcard::to_allocvec(&req).unwrap_or(vec![]);
    SendType::Connect(uid, request.addr, None, None, data)
}

pub(super) fn reject_message(
    layer: &mut Layer,
    tid: i64,
    addr: PeerAddr,
    me_id: GroupId,
) -> SendType {
    let data = postcard::to_allocvec(&LayerResponse::Reject).unwrap_or(vec![]);
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (me_id, tid));
    SendType::Result(uid, addr, false, false, data)
}

pub(super) fn event_message(
    layer: &mut Layer,
    tid: i64,
    me_id: GroupId,
    addr: PeerAddr,
    event: &LayerEvent,
) -> SendType {
    let data = postcard::to_allocvec(event).unwrap_or(vec![]);
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (me_id, tid));
    SendType::Event(uid, addr, data)
}

pub(crate) fn chat_conn(proof: Proof, addr: PeerAddr) -> SendType {
    let data = postcard::to_allocvec(&LayerRequest::Connect(proof)).unwrap_or(vec![]);
    SendType::Connect(0, addr, None, None, data)
}

async fn conn_res_message(layer: &Layer, mgid: &GroupId, addr: PeerAddr) -> Result<SendType> {
    let proof = layer.group.read().await.prove_addr(mgid, &addr)?;
    let data = postcard::to_allocvec(&LayerResponse::Connect(proof)).unwrap_or(vec![]);
    Ok(SendType::Result(0, addr, true, false, data))
}

async fn conn_agree_message(
    layer: &mut Layer,
    tid: i64,
    mgid: &GroupId,
    addr: PeerAddr,
) -> Result<SendType> {
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (*mgid, tid));
    let group_lock = layer.group.read().await;
    let proof = group_lock.prove_addr(mgid, &addr)?;
    let me = group_lock.clone_user(mgid)?;
    drop(group_lock);
    let data = postcard::to_allocvec(&LayerResponse::Agree(me, proof)).unwrap_or(vec![]);
    Ok(SendType::Result(uid, addr, true, false, data))
}

pub(super) fn rpc_agree_message(
    layer: &mut Layer,
    tid: i64,
    proof: Proof,
    me: User,
    mgid: &GroupId,
    addr: PeerAddr,
) -> Result<SendType> {
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (*mgid, tid));
    let data = postcard::to_allocvec(&LayerResponse::Agree(me, proof)).unwrap_or(vec![]);
    Ok(SendType::Result(uid, addr, true, false, data))
}

// maybe need if gid or addr in blocklist.
fn res_reject() -> Vec<u8> {
    postcard::to_allocvec(&LayerResponse::Reject).unwrap_or(vec![])
}
