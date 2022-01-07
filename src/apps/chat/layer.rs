use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tdn::types::{
    group::{EventId, GroupId},
    message::{RecvType, SendType},
    primitive::{DeliveryType, HandleResult, Peer, PeerId, Result},
};
use tdn_did::Proof;
use tdn_storage::local::DStorage;
use tokio::sync::RwLock;

use chat_types::{MessageType, NetworkMessage};

use crate::account::{Account, User};
use crate::event::InnerEvent;
use crate::group::Group;
use crate::layer::{Layer, Online};
use crate::migrate::consensus::{FRIEND_TABLE_PATH, MESSAGE_TABLE_PATH, REQUEST_TABLE_PATH};
use crate::rpc::{
    notice_menu, session_connect, session_create, session_last, session_lost, session_suspend,
    session_update_name,
};
use crate::session::{connect_session, Session, SessionType};
use crate::storage::write_avatar_sync;

use super::models::{handle_nmsg, raw_to_network_message, Friend, Message, Request};
use super::rpc;

/// Chat connect data structure.
/// params: Friend about me height, connect_proof.
#[derive(Serialize, Deserialize)]
pub struct LayerConnect(pub i64, pub Proof);

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
    /// params is name, remark, proof.
    Request(String, String, Proof),
    /// agree friendship request.
    /// params is gid.
    Agree(Proof),
    /// reject friendship request.
    Reject,
    /// receiver gid, sender gid, message.
    Message(EventId, NetworkMessage),
    /// request user info.
    InfoReq(i64),
    /// user full info.
    InfoRes(User),
    /// close friendship.
    Close,
}

pub(crate) async fn handle(
    arc_layer: &Arc<RwLock<Layer>>,
    fgid: GroupId,
    mgid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    debug!("---------DEBUG--------- GOT CHAT EVENT");
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
            if let Ok(height) =
                handle_connect(&mgid, &fgid, &addr, data, &mut layer, &mut results).await
            {
                let peer_id = addr.id;
                let proof = layer.group.read().await.prove_addr(&mgid, &addr.id)?;
                let data = bincode::serialize(&proof).unwrap_or(vec![]);
                let msg = SendType::Result(0, addr, true, false, data);
                results.layers.push((mgid, fgid, msg));

                let info = LayerEvent::InfoReq(height);
                let data = bincode::serialize(&info).unwrap_or(vec![]);
                let msg = SendType::Event(0, peer_id, data);
                results.layers.push((mgid, fgid, msg));
            } else {
                let msg = SendType::Result(0, addr, false, false, vec![]);
                results.layers.push((mgid, fgid, msg));
            }
        }
        RecvType::Result(addr, is_ok, data) => {
            // ESSE chat layer result date structure.
            if is_ok {
                if let Ok(height) =
                    handle_connect(&mgid, &fgid, &addr, data, &mut layer, &mut results).await
                {
                    let info = LayerEvent::InfoReq(height);
                    let data = bincode::serialize(&info).unwrap_or(vec![]);
                    let msg = SendType::Event(0, addr.id, data);
                    results.layers.push((mgid, fgid, msg));
                } else {
                    let msg = SendType::Result(0, addr, false, false, vec![]);
                    results.layers.push((mgid, fgid, msg));
                }
            } else {
                let db = layer.group.read().await.chat_db(&mgid)?;
                let friend = Friend::get_id(&db, &fgid)?;
                if friend.contains_addr(&addr.id) {
                    results.rpcs.push(rpc::friend_close(mgid, friend.id));
                    friend.close(&db)?;
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
            debug!("delivery: tid: {}, is_ok: {}", tid, is_ok);
            // TODO maybe send failure need handle.
            if is_ok {
                if let Some((gid, db_id)) = layer.delivery.remove(&tid) {
                    let db = layer.group.read().await.chat_db(&mgid)?;
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

async fn handle_connect(
    mgid: &GroupId,
    fgid: &GroupId,
    addr: &Peer,
    data: Vec<u8>,
    layer: &mut Layer,
    results: &mut HandleResult,
) -> Result<i64> {
    // 0. deserialize connect data.
    let proof: Proof = bincode::deserialize(&data)?;

    // 1. check verify.
    proof.verify(fgid, &addr.id, &layer.addr)?;

    // 2. check friendship.
    let friend = update_friend(&layer.group.read().await.chat_db(mgid)?, fgid, &addr.id);
    if friend.is_err() {
        return Err(anyhow!("not friend"));
    }
    let f = friend.unwrap(); // safe.

    // 3. get session.
    let session_some = connect_session(
        &layer.group.read().await.session_db(mgid)?,
        &SessionType::Chat,
        &f.id,
        &addr.id,
    )?;
    if session_some.is_none() {
        return Err(anyhow!("not friend"));
    }
    let sid = session_some.unwrap().id;

    // 4. active this session.
    layer
        .running_mut(mgid)?
        .check_add_online(*fgid, Online::Direct(addr.id), sid, f.id)?;

    // 5. session online to UI.
    results.rpcs.push(session_connect(*mgid, &sid, &addr.id));
    Ok(f.height)
}

impl LayerEvent {
    pub async fn handle(
        fgid: GroupId,
        mgid: GroupId,
        layer: &mut Layer,
        addr: PeerId,
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
            LayerEvent::Request(name, remark, proof) => {
                // 1. check verify.
                proof.verify(&fgid, &addr, &layer.addr)?;

                if load_friend(&layer.group.read().await.chat_db(&mgid)?, &fgid).is_err() {
                    // check if exist request.
                    let db = layer.group.read().await.chat_db(&mgid)?;
                    if let Ok(req) = Request::get_id(&db, &fgid) {
                        Request::delete(&db, &req.id)?; // delete the old request.
                        results.rpcs.push(rpc::request_delete(mgid, req.id));
                    }
                    let mut request = Request::new(fgid, addr, name, remark, false, true);
                    // save to db.
                    request.insert(&db)?;
                    drop(db);

                    results.rpcs.push(rpc::request_create(mgid, &request));
                    results.rpcs.push(notice_menu(mgid, &SessionType::Chat));
                    return Ok(results);
                } else {
                    let proof = layer.group.read().await.prove_addr(&mgid, &addr)?;
                    let msg = agree_message(proof, addr)?;
                    results.layers.push((mgid, fgid, msg));
                }
            }
            LayerEvent::Agree(proof) => {
                // 0. check verify.
                proof.verify(&fgid, &addr, &layer.addr)?;
                // 1. check friendship.
                if load_friend(&layer.group.read().await.chat_db(&mgid)?, &fgid).is_err() {
                    // 2. agree request for friend.
                    let db = layer.group.read().await.chat_db(&mgid)?;
                    if let Ok(mut r) = Request::get_id(&db, &fgid) {
                        r.is_over = true;
                        r.is_ok = true;
                        r.update(&db)?;
                        let friend = Friend::from_remote(&db, fgid, r.name, addr, "".to_owned())?;
                        results.rpcs.push(rpc::request_agree(mgid, r.id, &friend));

                        // ADD NEW SESSION.
                        let s_db = layer.group.read().await.session_db(&mgid)?;
                        let mut session = friend.to_session();
                        session.insert(&s_db)?;
                        results.rpcs.push(session_create(mgid, &session));
                    }
                    drop(db);
                }
            }
            LayerEvent::Reject => {
                let db = layer.group.read().await.chat_db(&mgid)?;
                if let Ok(mut request) = Request::get_id(&db, &fgid) {
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
                let db = layer.group.read().await.chat_db(&mgid)?;
                if !Message::exist(&db, &hash)? {
                    let msg = handle_nmsg(
                        &layer.group,
                        m.clone(),
                        false,
                        mgid,
                        &layer.base,
                        &db,
                        fid,
                        hash,
                        &mut results,
                    )
                    .await?;
                    layer.group.write().await.broadcast(
                        &mgid,
                        InnerEvent::SessionMessageCreate(fgid, false, hash, m),
                        MESSAGE_TABLE_PATH,
                        msg.id,
                        &mut results,
                    )?;
                    results.rpcs.push(rpc::message_create(mgid, &msg));

                    // UPDATE SESSION.
                    if let Ok(s_db) = layer.group.read().await.session_db(&mgid) {
                        update_session(&s_db, &mgid, &fid, &msg, &mut results);
                    }
                }
            }
            LayerEvent::InfoReq(height) => {
                // check sync remote height.
                if let Ok(account) = Account::get(&layer.group.read().await.account_db()?, &mgid) {
                    if account.pub_height > height {
                        let info = LayerEvent::InfoRes(User::info(
                            account.name,
                            account.wallet,
                            account.pub_height,
                            account.avatar,
                        ));
                        let data = bincode::serialize(&info).unwrap_or(vec![]);
                        let msg = SendType::Event(0, addr, data);
                        results.layers.push((mgid, fgid, msg));
                    }
                }
            }
            LayerEvent::InfoRes(remote) => {
                let (sid, fid) = layer.get_running_remote_id(&mgid, &fgid)?;
                let avatar = remote.avatar.clone();
                let db = layer.group.read().await.chat_db(&mgid)?;
                let mut f = Friend::get(&db, &fid)?;
                let name = remote.name.clone();
                f.name = remote.name;
                f.wallet = remote.wallet;
                f.height = remote.height;
                f.remote_update(&db)?;
                drop(db);
                write_avatar_sync(&layer.base, &mgid, &remote.id, remote.avatar)?;
                results.rpcs.push(rpc::friend_info(mgid, &f));
                let _ =
                    Session::update_name(&layer.group.read().await.session_db(&mgid)?, &sid, &name);
                results.rpcs.push(session_update_name(mgid, &sid, &name));

                layer.group.write().await.broadcast(
                    &mgid,
                    InnerEvent::SessionFriendInfo(f.gid, f.addr, f.name.clone(), avatar),
                    FRIEND_TABLE_PATH,
                    f.id,
                    &mut results,
                )?;
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
                let db = layer.group.read().await.chat_db(&mgid)?;
                Friend::id_close(&db, fid)?;
                drop(db);
                results.rpcs.push(rpc::friend_close(mgid, fid));
                if !layer.is_addr_online(&addr) {
                    results
                        .layers
                        .push((mgid, fgid, SendType::Disconnect(addr)))
                }
            }
        }

        Ok(results)
    }

    pub async fn from_message(
        group: &Arc<RwLock<Group>>,
        base: &PathBuf,
        mgid: GroupId,
        fid: i64,
        m_type: MessageType,
        content: &str,
    ) -> std::result::Result<(Message, NetworkMessage), tdn::types::rpc::RpcError> {
        // handle message's type.
        let db = group.read().await.chat_db(&mgid)?;
        let (nm_type, raw) = raw_to_network_message(group, base, &mgid, &m_type, content).await?;
        let mut msg = Message::new(&mgid, fid, true, m_type, raw, false);
        msg.insert(&db)?;
        Ok((msg, nm_type))
    }
}

#[inline]
fn load_friend(db: &DStorage, fgid: &GroupId) -> Result<Friend> {
    Friend::get_id(&db, fgid)
}

#[inline]
fn update_friend(db: &DStorage, fgid: &GroupId, addr: &PeerId) -> Result<Friend> {
    let friend = Friend::get_id(&db, fgid)?;
    if &friend.addr != addr {
        let _ = Friend::addr_update(&db, friend.id, addr);
    }
    Ok(friend)
}

pub(super) fn req_message(
    layer: &mut Layer,
    gid: GroupId,
    name: String,
    proof: Proof,
    request: Request,
) -> SendType {
    // update delivery.
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (gid, request.id));
    let req = LayerEvent::Request(name, request.remark, proof);
    let data = bincode::serialize(&req).unwrap_or(vec![]);
    SendType::Event(uid, request.addr, data)
}

pub(super) fn reject_message(layer: &mut Layer, tid: i64, addr: PeerId, gid: GroupId) -> SendType {
    let data = bincode::serialize(&LayerEvent::Reject).unwrap_or(vec![]);
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (gid, tid));
    SendType::Event(uid, addr, data)
}

pub(crate) fn event_message(
    layer: &mut Layer,
    tid: i64,
    me_id: GroupId,
    addr: PeerId,
    event: &LayerEvent,
) -> SendType {
    let data = bincode::serialize(event).unwrap_or(vec![]);
    let uid = layer.delivery.len() as u64 + 1;
    layer.delivery.insert(uid, (me_id, tid));
    SendType::Event(uid, addr, data)
}

pub(crate) fn chat_conn(proof: Proof, addr: Peer) -> SendType {
    let data = bincode::serialize(&proof).unwrap_or(vec![]);
    SendType::Connect(0, addr, data)
}

pub(super) fn agree_message(proof: Proof, addr: PeerId) -> Result<SendType> {
    let data = bincode::serialize(&LayerEvent::Agree(proof)).unwrap_or(vec![]);
    Ok(SendType::Event(0, addr, data))
}

// maybe need if gid or addr in blocklist.
fn _res_reject() -> Vec<u8> {
    bincode::serialize(&LayerEvent::Reject).unwrap_or(vec![])
}

// UPDATE SESSION.
pub(crate) fn update_session(
    s_db: &DStorage,
    gid: &GroupId,
    id: &i64,
    msg: &Message,
    results: &mut HandleResult,
) {
    let scontent = match msg.m_type {
        MessageType::String => {
            format!("{}:{}", msg.m_type.to_int(), msg.content)
        }
        _ => format!("{}:", msg.m_type.to_int()),
    };

    if let Ok(sid) = Session::last(
        &s_db,
        id,
        &SessionType::Chat,
        &msg.datetime,
        &scontent,
        true,
    ) {
        results
            .rpcs
            .push(session_last(*gid, &sid, &msg.datetime, &scontent, false));
    }
}
