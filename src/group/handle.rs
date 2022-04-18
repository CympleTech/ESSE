use esse_primitives::MessageType;
use std::sync::Arc;
use tdn::types::{
    message::{RecvType, SendType},
    primitives::{DeliveryType, HandleResult, Peer, PeerId, Result},
};
use tdn_storage::local::DStorage;

use crate::account::{Account, User};
use crate::global::Global;
use crate::rpc::{
    notice_menu, session_connect, session_create, session_last, session_lost, session_suspend,
    session_update_name,
};
use crate::session::{connect_session, Session, SessionType};
use crate::storage::{account_db, chat_db, session_db, write_avatar_sync};

use super::rpc;
use super::{handle_nmsg, Friend, GroupEvent, Message, Request};

pub(crate) async fn group_handle(msg: RecvType, global: &Arc<Global>) -> Result<HandleResult> {
    debug!("---------DEBUG--------- GOT GROUP MESSAGE");
    let mut results = HandleResult::new();
    let pid = global.pid().await;

    match msg {
        RecvType::Connect(peer, _) | RecvType::ResultConnect(peer, _) => {
            // ESSE group connect date structure.
            if let Ok(height) = handle_connect(pid, &peer, global, &mut results).await {
                let peer_id = peer.id;
                let msg = SendType::Result(0, peer, true, false, vec![]);
                results.groups.push(msg);

                let info = GroupEvent::InfoReq(height);
                let data = bincode::serialize(&info).unwrap_or(vec![]);
                let msg = SendType::Event(0, peer_id, data);
                results.groups.push(msg);
            } else {
                let msg = SendType::Result(0, peer, false, false, vec![]);
                results.groups.push(msg);
            }
        }
        RecvType::Result(peer, is_ok, _) => {
            // ESSE group result date structure.
            if is_ok {
                if let Ok(height) = handle_connect(pid, &peer, global, &mut results).await {
                    let info = GroupEvent::InfoReq(height);
                    let data = bincode::serialize(&info).unwrap_or(vec![]);
                    let msg = SendType::Event(0, peer.id, data);
                    results.groups.push(msg);
                } else {
                    let msg = SendType::Result(0, peer, false, false, vec![]);
                    results.groups.push(msg);
                }
            } else {
                let db_key = global.own.read().await.db_key(&pid)?;
                let db = chat_db(&global.base, &pid, &db_key)?;
                let friend = Friend::get_id(&db, &peer.id)?;
                results.rpcs.push(rpc::friend_close(friend.id));
                friend.close(&db)?;
            }
        }
        RecvType::Event(fpid, bytes) => {
            return GroupEvent::handle(pid, fpid, global, bytes).await;
        }
        RecvType::Delivery(t, tid, is_ok) => {
            let mut group = global.group.write().await;
            let id = group.delivery.remove(&tid).ok_or(anyhow!("delivery err"))?;
            drop(group);
            let db_key = global.own.read().await.db_key(&pid)?;
            let db = chat_db(&global.base, &pid, &db_key)?;
            let resp = match t {
                DeliveryType::Event => {
                    Message::delivery(&db, id, is_ok)?;
                    rpc::message_delivery(id, is_ok)
                }
                DeliveryType::Connect => {
                    // request.
                    Request::delivery(&db, id, is_ok)?;
                    rpc::request_delivery(id, is_ok)
                }
                DeliveryType::Result => {
                    // response. TODO better for agree send.
                    Request::delivery(&db, id, is_ok)?;
                    rpc::request_delivery(id, is_ok)
                }
            };
            drop(db);
            results.rpcs.push(resp);
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }
        RecvType::Leave(peer) => {
            debug!("Peer leaved: {}", peer.id.to_hex());
            let mut group_lock = global.group.write().await;
            if let Ok((sid, _fid)) = group_lock.get(&peer.id) {
                results.rpcs.push(session_lost(&sid));
            }
            group_lock.rm_online(&peer.id);
            drop(group_lock);
        }
    }

    Ok(results)
}

async fn handle_connect(
    pid: PeerId,
    peer: &Peer,
    global: &Arc<Global>,
    results: &mut HandleResult,
) -> Result<u64> {
    let db_key = global.own.read().await.db_key(&pid)?;
    let db = chat_db(&global.base, &pid, &db_key)?;

    // 1. check friendship.
    let friend = Friend::get_id(&db, &peer.id);
    if friend.is_err() {
        return Err(anyhow!("not friend"));
    }
    let f = friend.unwrap(); // safe.

    // 2. get session.
    let s_db = session_db(&global.base, &pid, &db_key)?;
    let session_some = connect_session(&s_db, &SessionType::Chat, &f.id, &peer.id)?;
    if session_some.is_none() {
        return Err(anyhow!("not friend"));
    }
    let sid = session_some.unwrap().id;

    // 3. session online to UI.
    results.rpcs.push(session_connect(&sid, &peer.id));

    // 4. active this session.
    global.group.write().await.add(peer.id, sid, f.id, 0);

    Ok(f.height as u64)
}

impl GroupEvent {
    pub async fn handle(
        pid: PeerId,
        fpid: PeerId,
        global: &Arc<Global>,
        bytes: Vec<u8>,
    ) -> Result<HandleResult> {
        let event: GroupEvent = bincode::deserialize(&bytes)?;
        let mut results = HandleResult::new();

        match event {
            GroupEvent::Offline => {
                let mut group = global.group.write().await;
                let (sid, _fid) = group.get(&fpid)?;
                group.rm_online(&fpid);
                results.rpcs.push(session_lost(&sid));
            }
            GroupEvent::Suspend => {
                let mut group = global.group.write().await;
                let (sid, _fid) = group.get(&fpid)?;
                group.suspend(&fpid, false, false)?;
                results.rpcs.push(session_suspend(&sid));
            }
            GroupEvent::Actived => {
                let mut group = global.group.write().await;
                let (sid, _fid) = group.get(&fpid)?;
                group.active(&fpid, false)?;
                results.rpcs.push(session_connect(&sid, &fpid));
            }
            GroupEvent::Request(name, remark) => {
                let db_key = global.own.read().await.db_key(&pid)?;
                let db = chat_db(&global.base, &pid, &db_key)?;

                if Friend::get_id(&db, &fpid).is_err() {
                    // check if exist request.
                    if let Ok(req) = Request::get_id(&db, &fpid) {
                        Request::delete(&db, &req.id)?; // delete the old request.
                        results.rpcs.push(rpc::request_delete(req.id));
                    }
                    let mut request = Request::new(fpid, name, remark, false, true);
                    // save to db.
                    request.insert(&db)?;
                    drop(db);

                    results.rpcs.push(rpc::request_create(&request));
                    results.rpcs.push(notice_menu(&SessionType::Chat));
                    return Ok(results);
                } else {
                    let data = bincode::serialize(&GroupEvent::Agree).unwrap_or(vec![]);
                    let msg = SendType::Event(0, fpid, data);
                    results.groups.push(msg);
                }
            }
            GroupEvent::Agree => {
                let db_key = global.own.read().await.db_key(&pid)?;
                let db = chat_db(&global.base, &pid, &db_key)?;

                // 1. check friendship.
                if Friend::get_id(&db, &fpid).is_err() {
                    // 2. agree request for friend.
                    if let Ok(mut r) = Request::get_id(&db, &fpid) {
                        r.is_over = true;
                        r.is_ok = true;
                        r.update(&db)?;
                        let friend = Friend::from_remote(
                            &db,
                            fpid,
                            r.name,
                            "".to_owned(),
                            PeerId::default(),
                            [0u8; 32],
                        )?;
                        results.rpcs.push(rpc::request_agree(r.id, &friend));

                        // ADD NEW SESSION.
                        let s_db = session_db(&global.base, &pid, &db_key)?;
                        let mut session = friend.to_session();
                        session.insert(&s_db)?;
                        results.rpcs.push(session_create(&session));
                    }
                    drop(db);
                }
            }
            GroupEvent::Reject => {
                let db_key = global.own.read().await.db_key(&pid)?;
                let db = chat_db(&global.base, &pid, &db_key)?;

                if let Ok(mut request) = Request::get_id(&db, &fpid) {
                    request.is_over = true;
                    request.is_ok = false;
                    request.update(&db)?;
                    results.rpcs.push(rpc::request_reject(request.id));
                }
            }
            GroupEvent::Message(hash, m) => {
                let (_sid, fid) = global.group.read().await.get(&fpid)?;
                let db_key = global.own.read().await.db_key(&pid)?;
                let db = chat_db(&global.base, &pid, &db_key)?;

                if !Message::exist(&db, &hash)? {
                    let msg = handle_nmsg(
                        &pid,
                        &global.base,
                        &db_key,
                        m.clone(),
                        false,
                        &db,
                        fid,
                        hash,
                        &mut results,
                    )
                    .await?;
                    results.rpcs.push(rpc::message_create(&msg));

                    // UPDATE SESSION.
                    let s_db = session_db(&global.base, &pid, &db_key)?;
                    update_session(&s_db, &fid, &msg, &mut results);
                }
            }
            GroupEvent::InfoReq(height) => {
                // check sync remote height.
                let a_db = account_db(&global.base, &global.secret)?;
                let account = Account::get(&a_db, &pid)?;
                if account.pub_height > height {
                    let info = GroupEvent::InfoRes(User::info(
                        account.pub_height,
                        account.name,
                        account.wallet,
                        account.cloud,
                        account.cloud_key,
                        account.avatar,
                    ));
                    let data = bincode::serialize(&info).unwrap_or(vec![]);
                    let msg = SendType::Event(0, fpid, data);
                    results.groups.push(msg);
                }
            }
            GroupEvent::InfoRes(remote) => {
                let (sid, fid) = global.group.read().await.get(&fpid)?;
                let db_key = global.own.read().await.db_key(&pid)?;
                let db = chat_db(&global.base, &pid, &db_key)?;

                let mut f = Friend::get(&db, &fid)?;
                let name = remote.name.clone();
                f.name = remote.name;
                f.wallet = remote.wallet;
                f.height = remote.height as i64;
                f.cloud = remote.cloud;
                f.cloud_key = remote.cloud_key;
                f.remote_update(&db)?;
                drop(db);
                write_avatar_sync(&global.base, &pid, &f.pid, remote.avatar)?;
                results.rpcs.push(rpc::friend_info(&f));

                let s_db = session_db(&global.base, &pid, &db_key)?;
                let _ = Session::update_name(&s_db, &sid, &name);
                results.rpcs.push(session_update_name(&sid, &name));
            }
            GroupEvent::Close => {
                let mut group = global.group.write().await;
                group.rm_online(&fpid);
                let (_sid, fid) = group.get(&fpid)?;
                let keep = group.is_online(&fpid);
                drop(group);

                let db_key = global.own.read().await.db_key(&pid)?;
                let db = chat_db(&global.base, &pid, &db_key)?;

                Friend::id_close(&db, fid)?;
                drop(db);
                results.rpcs.push(rpc::friend_close(fid));
                if !keep {
                    results.groups.push(SendType::Disconnect(fpid))
                }
                // TODO close session
            }
        }

        Ok(results)
    }
}

pub(crate) fn group_conn(pid: PeerId, results: &mut HandleResult) {
    results
        .groups
        .push(SendType::Connect(0, Peer::peer(pid), vec![]));
}

// UPDATE SESSION.
pub(crate) fn update_session(s_db: &DStorage, id: &i64, msg: &Message, results: &mut HandleResult) {
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
            .push(session_last(&sid, &msg.datetime, &scontent, false));
    }
}
