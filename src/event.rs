use esse_primitives::NetworkMessage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::{EventId, GroupId},
    message::{SendMessage, SendType},
    primitive::{HandleResult, PeerId, Result},
};
use tdn_storage::local::DStorage;
use tokio::sync::{mpsc::Sender, RwLock};

use crate::account::{Account, User};
use crate::consensus::Event as OldEvent;
use crate::group::GroupEvent;
use crate::layer::Layer;
use crate::migrate::consensus::{
    ACCOUNT_TABLE_PATH, FILE_TABLE_PATH, FRIEND_TABLE_PATH, MESSAGE_TABLE_PATH, REQUEST_TABLE_PATH,
};
use crate::own::{Own, OwnEvent};

use crate::apps::file::{FileDid, RootDirectory};
use crate::group::rpc as chat_rpc;
use crate::group::{from_model, Friend, Message, Request};
use crate::rpc;
use crate::storage::{delete_avatar_sync, read_avatar_sync, write_avatar_sync};

/// Online state synchronization.
#[derive(Serialize, Deserialize)]
pub(crate) enum State {
    Account,
    Session,
    ChatMessage,
    OwnMessage,
}

pub(crate) async fn _handle_state(
    _gid: GroupId,
    _addr: PeerId,
    state: State,
    _group: &Arc<RwLock<Own>>,
    _layer: &Arc<RwLock<Layer>>,
    _results: &mut HandleResult,
) -> Result<()> {
    match state {
        State::Account => {}
        State::Session => {}
        State::ChatMessage => {}
        State::OwnMessage => {}
    }
    Ok(())
}

/// Event that will update data.
#[derive(Serialize, Deserialize)]
pub(crate) enum InnerEvent {
    /// account info update.
    /// params: name, avatar.
    UserInfo(String, Vec<u8>),
    /// Session's request create.
    /// params: is_me, remote, remark.
    SessionRequestCreate(bool, User, String),
    /// Session's request agree or reject, if agree with avatar. friend's addr.
    SessionRequestHandle(GroupId, bool, Vec<u8>),
    /// Session's request delete.
    SessionRequestDelete(GroupId),
    /// Session's friend update by friend.
    /// params: f_gid, addr, name, avatar. (TODO addrs as list to store. keep others device)
    SessionFriendInfo(GroupId, PeerId, String, Vec<u8>),
    /// Session's friend update by me.
    /// params: f_gid, remark
    SessionFriendUpdate(GroupId, String),
    /// Session's friend close.
    /// params: f_gid.
    SessionFriendClose(GroupId),
    /// Sesson's friend delete.
    SessionFriendDelete(GroupId),
    /// Session's message create.
    /// params: f_gid, is_me, message_type,
    SessionMessageCreate(GroupId, bool, EventId, NetworkMessage),
    /// Session's message delete.
    SessionMessageDelete(EventId),
    /// create a file.
    /// params: file_id, file_parent_id, file_directory, file_name, file_desc, device_addr.
    FileCreate(FileDid, FileDid, RootDirectory, String, String, PeerId),
    /// update file info. file_id, file_name, file_desc.
    FileUpdate(FileDid, String, String),
    /// update file's parent id (move file to other directory).
    FileParent(FileDid, FileDid),
    /// backup file in new device.
    FileBackup(FileDid, PeerId),
    /// delete a file.
    FileDelete(FileDid),
}

/// Event that not update status. only change UI.
#[derive(Serialize, Deserialize)]
pub(crate) enum StatusEvent {
    /// Session's friend online.
    SessionFriendOnline(GroupId),
    /// Session's friend offline.
    SessionFriendOffline(GroupId),
}

/// event for sync models. use in sync consensus.
#[derive(Serialize, Deserialize)]
pub(crate) enum SyncEvent {
    /// account info.
    Account(EventId, String, Vec<u8>),
    AccountHad(EventId),
    /// eid, request_gid, addr, name, avatar, remark, is_me, is_ok
    Request(
        EventId,
        GroupId,
        PeerId,
        String,
        Vec<u8>,
        String,
        bool,
        bool,
        bool,
    ),
    RequestHad(EventId, GroupId),
    /// eid, friend_gid, addr, name, avatar, remark, is_closed
    Friend(EventId, GroupId, PeerId, String, Vec<u8>, String, bool),
    FriendHad(EventId, GroupId),
    /// eid, friend_gid, msg_id, is_me, message.
    Message(EventId, GroupId, EventId, bool, NetworkMessage),
    None,
}

impl InnerEvent {
    pub fn event_time(eid: &EventId) -> u128 {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&eid.0[0..16]);
        u128::from_le_bytes(bytes)
    }

    pub fn generate_event_id(&self) -> EventId {
        let mut bytes = [0u8; 32];
        let start = SystemTime::now();
        let datetime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_nanos())
            .unwrap_or(0);
        bytes[0..16].copy_from_slice(&datetime.to_le_bytes());
        let data = bincode::serialize(self).unwrap_or(vec![]);
        bytes[16..32].copy_from_slice(&blake3::hash(&data).as_bytes()[0..16]);
        EventId(bytes)
    }

    pub async fn direct_layer_session(
        sender: Sender<SendMessage>,
        layer: Arc<RwLock<Layer>>,
        gid: GroupId,
        fgid: GroupId,
        event: GroupEvent,
    ) -> Result<()> {
        let addr = layer.read().await.running(&gid)?.online_direct(&fgid)?;
        let data = bincode::serialize(&event).unwrap_or(vec![]);
        let msg = SendType::Event(0, addr, data);
        let _ = sender.send(SendMessage::Layer(gid, fgid, msg)).await;
        Ok(())
    }

    pub fn merge_event(
        db: &DStorage,
        addr: &PeerId,
        results: &mut HandleResult,
        our_height: u64,
        our_event: EventId,
        event_height: u64,
        event_id: EventId,
        need_sync: Option<GroupId>,
    ) -> Result<(u64, u64, EventId)> {
        if our_height + 1 < event_height {
            // TODO request for sync self_height + 1.
            if let Some(gid) = need_sync {
                let event = GroupEvent::SyncRequest(our_height + 1, event_height);
                let data = bincode::serialize(&event).unwrap_or(vec![]);
                results.groups.push((gid, SendType::Event(0, *addr, data)));
                return Ok((event_height, our_height, our_event));
            } else {
                return Ok((event_height, event_height, event_id));
            }
        }

        let mut merge_height = event_height;

        if our_height >= event_height {
            // load current event_hegiht.
            let events = OldEvent::get_nexts(db, event_height)?;
            for event in events {
                let our_event_time = Self::event_time(&event.hash);
                let remote_event_time = Self::event_time(&event_id);

                if our_event_time > remote_event_time {
                    break;
                }

                if our_event_time == remote_event_time {
                    let mut remote_bytes = [0u8; 16];
                    remote_bytes.copy_from_slice(&event_id.0[16..32]);
                    let remote_next = u128::from_le_bytes(remote_bytes);

                    let mut our_bytes = [0u8; 16];
                    our_bytes.copy_from_slice(&event.hash.0[16..32]);
                    let our_next = u128::from_le_bytes(our_bytes);

                    if remote_next > our_next {
                        break;
                    }
                }

                merge_height = event.height;
            }
        }

        Ok((merge_height, our_height + 1, our_event))
    }

    pub fn handle(
        self,
        group: &mut Group,
        gid: GroupId,
        addr: PeerId,
        eheight: u64,
        eid: EventId,
        pre_event: EventId,
        results: &mut HandleResult,
        layer: &Arc<RwLock<Layer>>,
    ) -> Result<()> {
        let account = group.account(&gid)?;
        let db = group.consensus_db(&gid)?;
        if OldEvent::contains_hash(&db, &eid)? {
            return Ok(());
        }

        let (merge_height, next_height, next_eid) =
            if account.own_height + 1 == eheight && account.event == pre_event {
                (eheight, eheight, eid)
            } else {
                Self::merge_event(
                    &db,
                    &addr,
                    results,
                    account.own_height,
                    account.event,
                    eheight,
                    eid,
                    Some(gid),
                )?
            };

        let (path, id) = match self {
            InnerEvent::UserInfo(name, avatar) => {
                results
                    .rpcs
                    .push(rpc::account_update(gid, &name, base64::encode(&avatar)));
                group.update_account(gid, &name, avatar)?;
                (ACCOUNT_TABLE_PATH, 0)
            }
            InnerEvent::SessionRequestCreate(is_me, remote, remark) => {
                let db = group.chat_db(&gid)?;
                // check if exist request.
                if Friend::get_id(&db, &remote.id).is_ok() {
                    return Ok(());
                }
                if let Ok(req) = Request::get_id(&db, &remote.id) {
                    Request::delete(&db, &req.id)?; // delete the old request.
                    results.rpcs.push(chat_rpc::request_delete(gid, req.id));
                }
                let mut request =
                    Request::new(remote.id, remote.addr, remote.name, remark, is_me, true);
                // save to db.
                request.insert(&db)?;
                drop(db);
                // save the avatar.
                write_avatar_sync(group.base(), &gid, &remote.id, remote.avatar)?;
                results.rpcs.push(chat_rpc::request_create(gid, &request));
                (REQUEST_TABLE_PATH, request.id)
            }
            InnerEvent::SessionRequestHandle(rgid, is_ok, avatar) => {
                let db = group.chat_db(&gid)?;
                if Friend::get_id(&db, &rgid).is_ok() {
                    return Ok(());
                }

                if let Ok(mut request) = Request::get_id(&db, &rgid) {
                    let rid = request.id;
                    request.is_over = true;
                    request.is_ok = is_ok;
                    request.update(&db)?;
                    if is_ok {
                        if avatar.len() > 0 {
                            write_avatar_sync(group.base(), &gid, &request.gid, avatar)?;
                        }
                        let friend = Friend::from_remote(
                            &db,
                            request.gid,
                            request.name,
                            request.addr,
                            "".to_owned(),
                        )?;
                        results
                            .rpcs
                            .push(chat_rpc::request_agree(gid, rid, &friend));
                    } else {
                        results.rpcs.push(chat_rpc::request_reject(gid, rid));
                    }
                    (REQUEST_TABLE_PATH, rid)
                } else {
                    return Ok(());
                }
            }
            InnerEvent::SessionRequestDelete(rgid) => {
                let db = group.chat_db(&gid)?;
                if let Ok(request) = Request::get_id(&db, &rgid) {
                    let rid = request.id;
                    Request::delete(&db, &request.id)?;
                    // delete avatar. check had friend.
                    if Friend::get_id(&db, &request.gid).is_err() {
                        delete_avatar_sync(group.base(), &gid, &request.gid)?;
                    }
                    results.rpcs.push(chat_rpc::request_delete(gid, rid));
                    (REQUEST_TABLE_PATH, rid)
                } else {
                    return Ok(());
                }
            }
            InnerEvent::SessionMessageCreate(rgid, is_me, hash, m) => {
                let db = group.chat_db(&gid)?;
                if Message::exist(&db, &hash)? {
                    return Ok(());
                }

                if let Ok(f) = Friend::get_id(&db, &rgid) {
                    if is_me {
                        let layer_lock = layer.clone();
                        let ggid = gid.clone();
                        let fgid = f.gid;
                        let sender = group.sender();
                        let layer_event = GroupEvent::Message(hash, m.clone());
                        tokio::spawn(InnerEvent::direct_layer_session(
                            sender,
                            layer_lock,
                            ggid,
                            fgid,
                            layer_event,
                        ));
                    }

                    // Need Refactor
                    // let msg =
                    //     handle_nmsg(group, m, is_me, gid, group.base(), &db, f.id, hash, results)
                    //         .await?;
                    // results.rpcs.push(chat_rpc::message_create(gid, &msg));
                    (MESSAGE_TABLE_PATH, 0)
                } else {
                    return Ok(());
                }
            }
            InnerEvent::SessionMessageDelete(hash) => {
                let db = group.chat_db(&gid)?;
                if let Ok(m) = Message::get_by_hash(&db, &hash) {
                    Message::delete(&db, &m.id)?;
                    results.rpcs.push(chat_rpc::message_delete(gid, m.id));
                    (MESSAGE_TABLE_PATH, m.id)
                } else {
                    return Ok(());
                }
            }
            InnerEvent::SessionFriendInfo(rgid, raddr, rname, ravatar) => {
                let db = group.chat_db(&gid)?;
                if let Ok(mut f) = Friend::get_id(&db, &rgid) {
                    f.addr = raddr;
                    f.name = rname;
                    f.remote_update(&db)?;
                    if ravatar.len() > 0 {
                        write_avatar_sync(group.base(), &gid, &rgid, ravatar)?;
                    }
                    results.rpcs.push(chat_rpc::friend_info(gid, &f));
                    (FRIEND_TABLE_PATH, f.id)
                } else {
                    return Ok(());
                }
            }
            InnerEvent::SessionFriendUpdate(rgid, remark) => {
                let db = group.chat_db(&gid)?;
                if let Ok(mut f) = Friend::get_id(&db, &rgid) {
                    f.remark = remark;
                    f.me_update(&db)?;
                    results
                        .rpcs
                        .push(chat_rpc::friend_update(gid, f.id, &f.remark));
                    (FRIEND_TABLE_PATH, f.id)
                } else {
                    return Ok(());
                }
            }
            InnerEvent::SessionFriendClose(rgid) => {
                let db = group.chat_db(&gid)?;
                if let Ok(f) = Friend::get_id(&db, &rgid) {
                    f.close(&db)?;
                    results.rpcs.push(chat_rpc::friend_close(gid, f.id));

                    let rfid = f.id;
                    let layer_lock = layer.clone();
                    let ggid = gid.clone();
                    let sender = group.sender();
                    tokio::spawn(async move {
                        let online = layer_lock.write().await.remove_online(&ggid, &f.gid);
                        if let Some(faddr) = online {
                            let mut addrs: HashMap<PeerId, GroupId> = HashMap::new();
                            addrs.insert(faddr, f.gid);
                            tokio::spawn(rpc::sleep_waiting_close_stable(
                                sender,
                                HashMap::new(),
                                addrs,
                            ));
                        }
                    });
                    (FRIEND_TABLE_PATH, rfid)
                } else {
                    return Ok(());
                }
            }
            InnerEvent::SessionFriendDelete(rgid) => {
                let db = group.chat_db(&gid)?;
                if let Ok(f) = Friend::get_id(&db, &rgid) {
                    Friend::delete(&db, &f.id)?;
                    results.rpcs.push(chat_rpc::friend_delete(gid, f.id));
                    delete_avatar_sync(group.base(), &gid, &f.gid)?;

                    let rfid = f.id;
                    let layer_lock = layer.clone();
                    let ggid = gid.clone();
                    let sender = group.sender();
                    tokio::spawn(async move {
                        let online = layer_lock.write().await.remove_online(&ggid, &f.gid);
                        if let Some(faddr) = online {
                            let mut addrs: HashMap<PeerId, GroupId> = HashMap::new();
                            addrs.insert(faddr, f.gid);
                            tokio::spawn(rpc::sleep_waiting_close_stable(
                                sender,
                                HashMap::new(),
                                addrs,
                            ));
                        }
                    });

                    (FRIEND_TABLE_PATH, rfid)
                } else {
                    return Ok(());
                }
            }
            InnerEvent::FileCreate(_fid, _fpid, _ftype, _fname, _fdesc, _faddr) => {
                // TOOD
                (FILE_TABLE_PATH, 0)
            }
            InnerEvent::FileUpdate(_fid, _fname, _fdesc) => {
                // TODO
                (FILE_TABLE_PATH, 0)
            }
            InnerEvent::FileParent(_fid, _fpid) => {
                // TODO
                (FILE_TABLE_PATH, 0)
            }
            InnerEvent::FileBackup(_fid, _faddr) => {
                // TODO
                (FILE_TABLE_PATH, 0)
            }
            InnerEvent::FileDelete(_fid) => {
                // TODO
                (FILE_TABLE_PATH, 0)
            }
        };

        OldEvent::merge(&db, eid, path, id, merge_height)?;
        drop(db);
        drop(layer);

        let account_db = group.account_db()?;
        let account = group.account_mut(&gid)?;
        account.update_consensus(&account_db, next_height, next_eid)?;
        account_db.close()?;

        Ok(())
    }
}

impl StatusEvent {
    pub fn handle(
        self,
        group: &mut Group,
        gid: GroupId,
        addr: PeerId,
        _results: &mut HandleResult,
        layer: &Arc<RwLock<Layer>>,
        _uid: u64,
    ) -> Result<()> {
        match self {
            StatusEvent::SessionFriendOnline(rgid) => {
                let db = group.chat_db(&gid)?;
                if let Ok(_f) = Friend::get_id(&db, &rgid) {
                    // TODO
                }
            }
            StatusEvent::SessionFriendOffline(rgid) => {
                let db = group.chat_db(&gid)?;
                if let Ok(f) = Friend::get_id(&db, &rgid) {
                    let layer_lock = layer.clone();
                    let rgid = f.gid;
                    let _rid = f.id;
                    let ggid = gid.clone();
                    let _sender = group.sender();
                    tokio::spawn(async move {
                        if let Ok(running) = layer_lock.write().await.running_mut(&ggid) {
                            if running.check_offline(&rgid, &addr) {
                                // TODO
                            }
                        }
                    });
                }
            }
        }
        Ok(())
    }
}

impl SyncEvent {
    pub async fn sync(
        group: &Group,
        base: &PathBuf,
        gid: &GroupId,
        account: &Account,
        from: u64,
        to: u64,
    ) -> Result<Vec<Self>> {
        let db = group.consensus_db(gid)?;
        let sql = format!(
            "SELECT id, hash, db_table, row from events WHERE id BETWEEN {} AND {}",
            from, to
        );
        let matrix = db.query(&sql)?;
        drop(db);
        let mut pre_keys: Vec<(i64, i64)> = vec![];
        let mut events: Vec<SyncEvent> = vec![];
        let mut next = from;
        for mut v in matrix {
            let row = v.pop().unwrap().as_i64(); // safe
            let path = v.pop().unwrap().as_i64(); // safe
            let hash = EventId::from_hex(v.pop().unwrap().as_str()).unwrap_or(EventId::default());
            let id = v.pop().unwrap().as_i64() as u64;
            if id != next {
                events.push(SyncEvent::None);
            }
            next += 1;

            match path {
                ACCOUNT_TABLE_PATH => {
                    if pre_keys.contains(&(path, row)) {
                        events.push(SyncEvent::AccountHad(hash));
                        continue;
                    } else {
                        pre_keys.push((path, row));
                    };

                    let name = account.name.clone();
                    let avatar = account.avatar.clone();
                    events.push(SyncEvent::Account(hash, name, avatar));
                }
                REQUEST_TABLE_PATH => {
                    let db = group.chat_db(gid)?;
                    let event = if let Ok(request) = Request::get(&db, &row) {
                        if pre_keys.contains(&(path, row)) {
                            events.push(SyncEvent::RequestHad(hash, request.gid));
                            continue;
                        } else {
                            pre_keys.push((path, row));
                        };

                        let avatar = if !request.is_ok {
                            vec![]
                        } else {
                            read_avatar_sync(base, gid, &request.gid)?
                        };

                        // request_gid, addr, name, avatar, remark, is_me, is_ok, is_delete
                        SyncEvent::Request(
                            hash,
                            request.gid,
                            request.addr,
                            request.name,
                            avatar,
                            request.remark,
                            request.is_me,
                            request.is_ok,
                            request.is_over,
                        )
                    } else {
                        SyncEvent::None
                    };

                    events.push(event);
                }
                FRIEND_TABLE_PATH => {
                    let db = group.chat_db(gid)?;
                    let event = if let Ok(friend) = Friend::get(&db, &row) {
                        if pre_keys.contains(&(path, row)) {
                            events.push(SyncEvent::FriendHad(hash, friend.gid));
                            continue;
                        } else {
                            pre_keys.push((path, row));
                        };

                        let avatar = if friend.is_closed {
                            vec![]
                        } else {
                            read_avatar_sync(base, gid, &friend.gid)?
                        };

                        SyncEvent::Friend(
                            hash,
                            friend.gid,
                            friend.addr,
                            friend.name,
                            avatar,
                            friend.remark,
                            friend.is_closed,
                        )
                    } else {
                        SyncEvent::None
                    };

                    events.push(event);
                }
                MESSAGE_TABLE_PATH => {
                    let db = group.chat_db(gid)?;
                    let event = if let Ok(msg) = Message::get(&db, &row) {
                        let fgid = if let Ok(f) = Friend::get(&db, &msg.fid) {
                            f.gid
                        } else {
                            GroupId::default()
                        };

                        // create
                        let mid = msg.hash;
                        let is_me = msg.is_me;
                        let nm = from_model(base, gid, msg).await?;
                        SyncEvent::Message(hash, fgid, mid, is_me, nm)
                    } else {
                        SyncEvent::None
                    };

                    events.push(event);
                }
                FILE_TABLE_PATH => {
                    //
                }
                _ => {}
            }
        }

        if events.len() as u64 != to + 1 - from {
            return Err(anyhow!("events number not matching."));
        }

        Ok(events)
    }

    pub fn handle(
        gid: GroupId,
        from: u64,
        to: u64,
        events: Vec<SyncEvent>,
        group: &mut Group,
        layer: &Arc<RwLock<Layer>>,
        results: &mut HandleResult,
        addr: PeerId,
    ) -> Result<()> {
        if events.len() as u64 != to + 1 - from {
            return Ok(());
        }
        let base = group.base().clone();
        let consensus_db = group.consensus_db(&gid)?;

        let mut next = from;
        for event in events {
            let height = next;
            next += 1;

            match &event {
                SyncEvent::Account(eid, ..)
                | SyncEvent::AccountHad(eid)
                | SyncEvent::Request(eid, ..)
                | SyncEvent::RequestHad(eid, ..)
                | SyncEvent::Friend(eid, ..)
                | SyncEvent::FriendHad(eid, ..)
                | SyncEvent::Message(eid, ..) => {
                    if OldEvent::contains_hash(&consensus_db, eid)? {
                        continue;
                    }
                }
                SyncEvent::None => {
                    continue;
                }
            }

            let (eid, path, id) = match event {
                SyncEvent::Account(eid, name, avatar) => {
                    results
                        .rpcs
                        .push(rpc::account_update(gid, &name, base64::encode(&avatar)));
                    group.update_account(gid, &name, avatar)?;
                    (eid, ACCOUNT_TABLE_PATH, 0)
                }
                SyncEvent::AccountHad(eid) => (eid, ACCOUNT_TABLE_PATH, 0),
                SyncEvent::Request(
                    eid,
                    rgid,
                    raddr,
                    rname,
                    avatar,
                    remark,
                    is_me,
                    is_ok,
                    is_over,
                ) => {
                    let chat_db = group.chat_db(&gid)?;
                    let request = if let Ok(mut req) = Request::get_id(&chat_db, &rgid) {
                        req.is_ok = is_ok;
                        req.is_over = is_over;
                        req.update(&chat_db)?;
                        req
                    } else {
                        let mut request = Request::new(rgid, raddr, rname, remark, is_me, true);
                        request.is_ok = is_ok;
                        request.is_over = is_over;

                        // save to db.
                        request.insert(&chat_db)?;
                        results.rpcs.push(chat_rpc::request_create(gid, &request));
                        request
                    };

                    let rid = request.id;
                    if is_ok {
                        if avatar.len() > 0 {
                            write_avatar_sync(&base, &gid, &request.gid, avatar)?;
                        }
                        let friend = Friend::from_remote(
                            &chat_db,
                            request.gid,
                            request.name,
                            request.addr,
                            "".to_owned(),
                        )?;
                        results
                            .rpcs
                            .push(chat_rpc::request_agree(gid, rid, &friend));
                    } else {
                        results.rpcs.push(chat_rpc::request_reject(gid, rid));
                    }
                    chat_db.close()?;

                    (eid, REQUEST_TABLE_PATH, rid)
                }
                SyncEvent::RequestHad(eid, rgid) => {
                    let chat_db = group.chat_db(&gid)?;
                    let id = if let Ok(req) = Request::get_id(&chat_db, &rgid) {
                        req.id
                    } else {
                        -1
                    };
                    (eid, REQUEST_TABLE_PATH, id)
                }
                SyncEvent::Friend(eid, fgid, faddr, fname, avatar, remark, is_closed) => {
                    let chat_db = group.chat_db(&gid)?;
                    let id = if let Ok(mut friend) = Friend::get_id(&chat_db, &fgid) {
                        friend.addr = faddr;
                        friend.name = fname;
                        friend.remark = remark;
                        friend.is_closed = is_closed;
                        friend.update(&chat_db)?;

                        if avatar.len() > 0 {
                            write_avatar_sync(&base, &gid, &friend.gid, avatar)?;
                        }

                        if friend.is_closed {
                            let layer_lock = layer.clone();
                            let ggid = gid.clone();
                            let fgid = friend.gid;
                            let sender = group.sender();
                            tokio::spawn(async move {
                                let online = layer_lock.write().await.remove_online(&ggid, &fgid);
                                if let Some(faddr) = online {
                                    let mut addrs: HashMap<PeerId, GroupId> = HashMap::new();
                                    addrs.insert(faddr, fgid);
                                    tokio::spawn(rpc::sleep_waiting_close_stable(
                                        sender,
                                        HashMap::new(),
                                        addrs,
                                    ));
                                }
                            });
                        }

                        results.rpcs.push(chat_rpc::friend_info(gid, &friend));

                        friend.id
                    } else {
                        -1
                    };
                    (eid, FRIEND_TABLE_PATH, id)
                }
                SyncEvent::FriendHad(eid, fgid) => {
                    let chat_db = group.chat_db(&gid)?;
                    let id = if let Ok(friend) = Friend::get_id(&chat_db, &fgid) {
                        friend.id
                    } else {
                        -1
                    };
                    (eid, FRIEND_TABLE_PATH, id)
                }
                SyncEvent::Message(eid, fgid, meid, _is_me, _m) => {
                    let chat_db = group.chat_db(&gid)?;
                    if Message::exist(&chat_db, &meid)? {
                        continue;
                    }

                    let id = if let Ok(_f) = Friend::get_id(&chat_db, &fgid) {
                        //let msg = handle_nmsg(m, is_me, gid, &base, &chat_db, f.id, eid, results)?;
                        //results.rpcs.push(chat_rpc::message_create(gid, &msg));
                        -1
                    } else {
                        -1
                    };

                    (eid, MESSAGE_TABLE_PATH, id)
                }
                SyncEvent::None => {
                    continue;
                }
            };

            let account_db = group.account_db()?;
            let account = group.account_mut(&gid)?;
            let (merge_height, next_height, next_eid) = InnerEvent::merge_event(
                &consensus_db,
                &addr,
                results,
                account.own_height,
                account.event,
                height,
                eid,
                None,
            )?;

            account.update_consensus(&account_db, next_height, next_eid)?;
            account_db.close()?;

            OldEvent::merge(&consensus_db, eid, path, id, merge_height)?;
        }

        consensus_db.close()?;
        Ok(())
    }
}
