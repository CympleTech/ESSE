use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
use crate::group::Group;
use crate::migrate::consensus::{FRIEND_TABLE_PATH, MESSAGE_TABLE_PATH, REQUEST_TABLE_PATH};
use crate::models::session::{Friend, Message, MessageType, NetworkMessage, Request};
use crate::rpc;
use crate::storage::{
    read_avatar, read_file, read_record, session_db, write_avatar_sync, write_file, write_image,
};

pub mod running;
use running::{Online, RunningAccount};

/// Esse layers.
pub(crate) struct Layer {
    /// account_gid => running_account.
    runnings: HashMap<GroupId, RunningAccount>,
    /// message delivery tracking. uuid, me_gid, db_id.
    delivery: HashMap<u64, (GroupId, i64)>,
    /// storage base path.
    base: PathBuf,
    /// self peer addr.
    addr: PeerAddr,
    /// group info.
    group: Arc<RwLock<Group>>,
}

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

impl Layer {
    pub async fn handle(
        &mut self,
        fgid: GroupId,
        mgid: GroupId,
        msg: RecvType,
    ) -> Result<HandleResult> {
        let mut results = HandleResult::new();

        // 1. check to account is online. if not online, nothing.
        if !self.runnings.contains_key(&mgid) {
            return Ok(results);
        }

        // 2. handle receive message by type.
        match msg {
            RecvType::Connect(addr, data) => {
                let request: LayerRequest = postcard::from_bytes(&data)
                    .map_err(|_e| new_io_error("Deseralize request friend failure"))?;

                match request {
                    LayerRequest::Connect(proof) => {
                        let fid = self.get_remote_id(&mgid, &fgid)?;
                        // 1. check verify.
                        proof.verify(&fgid, &addr, &self.addr)?;
                        // 2. online this group.
                        self.running_mut(&mgid)?
                            .check_add_online(fgid, Online::Direct(addr))?;
                        // 3. update remote addr. TODO
                        let db = session_db(&self.base, &mgid)?;
                        Friend::addr_update(&db, fid, &addr)?;
                        drop(db);
                        // 4. online to UI.
                        results.rpcs.push(rpc::friend_online(mgid, fid, addr));
                        // 5. connected.
                        let msg = self.conn_res_message(&mgid, addr).await?;
                        results.layers.push((mgid, fgid, msg));
                        self.group.write().await.status(
                            &mgid,
                            StatusEvent::SessionFriendOnline(fgid),
                            &mut results,
                        )?;
                    }
                    LayerRequest::Friend(remote, remark) => {
                        let some_fid = self.get_remote_id(&mgid, &fgid);
                        if some_fid.is_err() {
                            // check if exist request.
                            let db = session_db(&self.base, &mgid)?;
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
                            write_avatar_sync(
                                &self.base,
                                &mgid,
                                &request.gid,
                                remote.avatar.clone(),
                            )?;

                            self.group.write().await.broadcast(
                                &mgid,
                                InnerEvent::SessionRequestCreate(false, remote, remark),
                                REQUEST_TABLE_PATH,
                                request.id,
                                &mut results,
                            )?;

                            results.rpcs.push(rpc::request_create(mgid, &request));
                            return Ok(results);
                        }
                        let fid = some_fid.unwrap(); // safe checked.

                        // already friendship & update.
                        // 1. online this group.
                        self.running_mut(&mgid)?
                            .check_add_online(fgid, Online::Direct(addr))?;
                        // 2. update remote user.
                        let mut friend = self.update_friend(&mgid, fid, remote)?;
                        // 3. online to UI.
                        friend.online = true;
                        results.rpcs.push(rpc::friend_info(mgid, &friend));
                        // 4. connected.
                        let msg = self.conn_agree_message(0, &mgid, addr).await?;
                        results.layers.push((mgid, fgid, msg));
                        self.group.write().await.status(
                            &mgid,
                            StatusEvent::SessionFriendOnline(fgid),
                            &mut results,
                        )?;
                    }
                }
            }
            RecvType::Leave(addr) => {
                for (mgid, running) in &mut self.runnings {
                    let peers = running.peer_leave(&addr);
                    for (fgid, fid) in peers {
                        results.rpcs.push(rpc::friend_offline(*mgid, fid));
                        self.group.write().await.status(
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
                    let db = session_db(&self.base, &mgid)?;
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
                            let db = session_db(&self.base, &mgid)?;
                            if let Some(mut request) = Request::get(&db, &fgid)? {
                                self.group.write().await.broadcast(
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
                        proof.verify(&fgid, &addr, &self.addr)?;
                        // 2. check has this remove.
                        let fid = self.get_remote_id(&mgid, &fgid)?;
                        // 3. online this group.
                        self.running_mut(&mgid)?
                            .check_add_online(fgid, Online::Direct(addr))?;
                        // 4. update remote addr.
                        let db = session_db(&self.base, &mgid)?;
                        Friend::addr_update(&db, fid, &addr)?;
                        drop(db);
                        // 5. online to UI.
                        results.rpcs.push(rpc::friend_online(mgid, fid, addr));
                        self.group.write().await.status(
                            &mgid,
                            StatusEvent::SessionFriendOnline(fgid),
                            &mut results,
                        )?;
                    }
                    LayerResponse::Agree(remote, proof) => {
                        // 1. check verify.
                        proof.verify(&fgid, &addr, &self.addr)?;
                        if let Ok(fid) = self.get_remote_id(&mgid, &fgid) {
                            // already friendship.
                            self.running_mut(&mgid)?
                                .check_add_online(fgid, Online::Direct(addr))?;
                            results.rpcs.push(rpc::friend_online(mgid, fid, addr));
                            self.group.write().await.status(
                                &mgid,
                                StatusEvent::SessionFriendOnline(fgid),
                                &mut results,
                            )?;
                        } else {
                            // agree request for friend.
                            let db = session_db(&self.base, &mgid)?;
                            if let Some(mut request) = Request::get(&db, &remote.id)? {
                                self.group.write().await.broadcast(
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
                                write_avatar_sync(&self.base, &mgid, &remote.id, remote.avatar)?;
                                self.running_mut(&mgid)?.add_permissioned(fgid, friend.id);
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
                        proof.verify(&fgid, &addr, &self.addr)?;
                        // 2. check has this remove.
                        let fid = self.get_remote_id(&mgid, &fgid)?;
                        // 3. online this group.
                        self.running_mut(&mgid)?
                            .check_add_online(fgid, Online::Direct(addr))?;
                        // 4. update remote addr.
                        let db = session_db(&self.base, &mgid)?;
                        Friend::addr_update(&db, fid, &addr)?;
                        drop(db);
                        // 5. online to UI.
                        results.rpcs.push(rpc::friend_online(mgid, fid, addr));
                        // 6. connected.
                        let msg = self.conn_res_message(&mgid, addr).await?;
                        results.layers.push((mgid, fgid, msg));
                        self.group.write().await.status(
                            &mgid,
                            StatusEvent::SessionFriendOnline(fgid),
                            &mut results,
                        )?;
                    }
                    LayerResponse::Agree(remote, proof) => {
                        // 1. check verify.
                        proof.verify(&fgid, &addr, &self.addr)?;
                        if let Ok(fid) = self.get_remote_id(&mgid, &fgid) {
                            // already friendship.
                            self.running_mut(&mgid)?
                                .check_add_online(fgid, Online::Direct(addr))?;
                            results.rpcs.push(rpc::friend_online(mgid, fid, addr));
                            self.group.write().await.status(
                                &mgid,
                                StatusEvent::SessionFriendOnline(fgid),
                                &mut results,
                            )?;
                        } else {
                            // agree request for friend.
                            let db = session_db(&self.base, &mgid)?;
                            if let Some(mut request) = Request::get(&db, &remote.id)? {
                                self.group.write().await.broadcast(
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
                                write_avatar_sync(&self.base, &mgid, &remote.id, remote.avatar)?;
                                self.running_mut(&mgid)?.add_permissioned(fgid, friend.id);
                                results
                                    .rpcs
                                    .push(rpc::request_agree(mgid, request_id, &friend));
                            }
                            drop(db);
                        }

                        let msg = self.conn_res_message(&mgid, addr).await?;
                        results.layers.push((mgid, fgid, msg));
                    }
                    LayerResponse::Reject => {
                        let db = session_db(&self.base, &mgid)?;
                        if let Some(mut request) = Request::get(&db, &fgid)? {
                            self.group.write().await.broadcast(
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
                return LayerEvent::handle(fgid, mgid, self, addr, bytes).await;
            }
            RecvType::Stream(_uid, _stream, _bytes) => {
                // TODO stream
            }
            RecvType::Delivery(t, tid, is_ok) => {
                println!("delivery: tid: {}, is_ok: {}", tid, is_ok);
                // TODO maybe send failure need handle.
                if is_ok {
                    if let Some((gid, db_id)) = self.delivery.remove(&tid) {
                        let db = session_db(&self.base, &mgid)?;
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
}

impl Layer {
    pub async fn init(base: PathBuf, addr: PeerAddr, group: Arc<RwLock<Group>>) -> Result<Layer> {
        Ok(Layer {
            base,
            group,
            addr,
            runnings: HashMap::new(),
            delivery: HashMap::new(),
        })
    }

    pub fn base(&self) -> &PathBuf {
        &self.base
    }

    pub fn running(&self, gid: &GroupId) -> Result<&RunningAccount> {
        self.runnings.get(gid).ok_or(new_io_error("not online"))
    }

    pub fn running_mut(&mut self, gid: &GroupId) -> Result<&mut RunningAccount> {
        self.runnings.get_mut(gid).ok_or(new_io_error("not online"))
    }

    pub fn add_running(&mut self, gid: &GroupId) -> Result<()> {
        if !self.runnings.contains_key(gid) {
            self.runnings
                .insert(*gid, RunningAccount::init(&self.base, gid)?);
        }

        Ok(())
    }

    pub fn remove_running(&mut self, gid: &GroupId) -> HashMap<PeerAddr, GroupId> {
        // check close the stable connection.
        let mut addrs: HashMap<PeerAddr, GroupId> = HashMap::new();
        if let Some(running) = self.runnings.remove(gid) {
            for (addr, fgid) in running.remove_onlines() {
                addrs.insert(addr, fgid);
            }
        }

        let mut need_keep = vec![];
        for (_, running) in &self.runnings {
            for addr in addrs.keys() {
                if running.check_addr_online(addr) {
                    need_keep.push(*addr);
                }
            }
        }
        for i in need_keep {
            addrs.remove(&i);
        }

        addrs
    }

    pub fn remove_all_running(&mut self) -> HashMap<PeerAddr, GroupId> {
        let mut addrs: HashMap<PeerAddr, GroupId> = HashMap::new();
        for (_, running) in self.runnings.drain() {
            for (addr, fgid) in running.remove_onlines() {
                addrs.insert(addr, fgid);
            }
        }
        addrs
    }

    pub fn get_remote_id(&self, mgid: &GroupId, fgid: &GroupId) -> Result<i64> {
        self.running(mgid)?.get_permissioned(fgid)
    }

    pub fn all_friends(&self, gid: &GroupId) -> Result<Vec<Friend>> {
        let db = session_db(&self.base, &gid)?;
        let friends = Friend::all_ok(&db)?;
        drop(db);
        Ok(friends)
    }

    pub fn all_friends_with_online(&self, gid: &GroupId) -> Result<Vec<Friend>> {
        let db = session_db(&self.base, &gid)?;
        let mut friends = Friend::all(&db)?;
        drop(db);

        let keys: HashMap<GroupId, usize> = friends
            .iter()
            .enumerate()
            .map(|(i, f)| (f.gid, i))
            .collect();

        for fgid in self.running(gid)?.online_groups() {
            friends[keys[fgid]].online = true; // safe vec index.
        }

        Ok(friends)
    }

    pub fn update_friend(&self, gid: &GroupId, fid: i64, remote: User) -> Result<Friend> {
        let db = session_db(&self.base, &gid)?;
        if let Some(mut friend) = Friend::get_id(&db, fid)? {
            friend.name = remote.name;
            friend.addr = remote.addr;
            friend.remote_update(&db)?;
            drop(db);
            write_avatar_sync(&self.base, gid, &remote.id, remote.avatar)?;
            Ok(friend)
        } else {
            drop(db);
            Err(new_io_error("missing friend id"))
        }
    }

    pub fn remove_friend(&mut self, gid: &GroupId, fgid: &GroupId) -> Option<PeerAddr> {
        self.running_mut(gid).ok()?.remove_permissioned(fgid)
    }

    pub async fn all_friend_conns(&self) -> HashMap<GroupId, Vec<(GroupId, SendType)>> {
        let mut conns = HashMap::new();
        for mgid in self.runnings.keys() {
            if let Ok(friends) = self.all_friends(mgid) {
                let mut vecs = vec![];
                for friend in friends {
                    if let Ok(msg) = self.conn_req_message(&friend.gid, friend.addr).await {
                        vecs.push((friend.gid, msg));
                    }
                }
                conns.insert(*mgid, vecs);
            }
        }
        conns
    }

    pub fn is_online(&self, faddr: &PeerAddr) -> bool {
        for (_, running) in &self.runnings {
            running.check_addr_online(faddr);
        }
        return false;
    }

    pub fn req_message(&mut self, me: User, request: Request) -> SendType {
        // update delivery.
        let uid = self.delivery.len() as u64 + 1;
        self.delivery.insert(uid, (me.id, request.id));
        let req = LayerRequest::Friend(me, request.remark);
        let data = postcard::to_allocvec(&req).unwrap_or(vec![]);
        SendType::Connect(uid, request.addr, None, None, data)
    }

    pub fn reject_message(&mut self, tid: i64, addr: PeerAddr, me_id: GroupId) -> SendType {
        let data = postcard::to_allocvec(&LayerResponse::Reject).unwrap_or(vec![]);
        let uid = self.delivery.len() as u64 + 1;
        self.delivery.insert(uid, (me_id, tid));
        SendType::Result(uid, addr, false, false, data)
    }

    pub fn event_message(
        &mut self,
        tid: i64,
        me_id: GroupId,
        addr: PeerAddr,
        event: &LayerEvent,
    ) -> SendType {
        let data = postcard::to_allocvec(event).unwrap_or(vec![]);
        let uid = self.delivery.len() as u64 + 1;
        self.delivery.insert(uid, (me_id, tid));
        SendType::Event(uid, addr, data)
    }

    pub async fn conn_req_message(&self, mgid: &GroupId, addr: PeerAddr) -> Result<SendType> {
        let proof = self.group.read().await.prove_addr(mgid, &addr)?;
        let data = postcard::to_allocvec(&LayerRequest::Connect(proof)).unwrap_or(vec![]);
        Ok(SendType::Connect(0, addr, None, None, data))
    }

    pub async fn conn_res_message(&self, mgid: &GroupId, addr: PeerAddr) -> Result<SendType> {
        let proof = self.group.read().await.prove_addr(mgid, &addr)?;
        let data = postcard::to_allocvec(&LayerResponse::Connect(proof)).unwrap_or(vec![]);
        Ok(SendType::Result(0, addr, true, false, data))
    }

    pub async fn conn_agree_message(
        &mut self,
        tid: i64,
        mgid: &GroupId,
        addr: PeerAddr,
    ) -> Result<SendType> {
        let uid = self.delivery.len() as u64 + 1;
        self.delivery.insert(uid, (*mgid, tid));
        let group_lock = self.group.read().await;
        let proof = group_lock.prove_addr(mgid, &addr)?;
        let me = group_lock.clone_user(mgid)?;
        drop(group_lock);
        let data = postcard::to_allocvec(&LayerResponse::Agree(me, proof)).unwrap_or(vec![]);
        Ok(SendType::Result(uid, addr, true, false, data))
    }

    pub fn rpc_agree_message(
        &mut self,
        tid: i64,
        proof: Proof,
        me: User,
        mgid: &GroupId,
        addr: PeerAddr,
    ) -> Result<SendType> {
        let uid = self.delivery.len() as u64 + 1;
        self.delivery.insert(uid, (*mgid, tid));
        let data = postcard::to_allocvec(&LayerResponse::Agree(me, proof)).unwrap_or(vec![]);
        Ok(SendType::Result(uid, addr, true, false, data))
    }

    // maybe need if gid or addr in blocklist.
    pub fn _res_reject() -> Vec<u8> {
        postcard::to_allocvec(&LayerResponse::Reject).unwrap_or(vec![])
    }
}

/// Esse app's Event.
#[derive(Serialize, Deserialize)]
pub(crate) enum LayerEvent {
    /// receiver gid, sender gid, message.
    Message(EventId, NetworkMessage),
    /// receiver gid, sender user.
    Info(User),
    /// receiver gid, sender gid.
    OnlinePing,
    /// receiver gid, sender gid.
    OnlinePong,
    /// receiver gid, sender gid.
    Offline,
    /// close friendship.
    Close,
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
        let fid = layer.get_remote_id(&mgid, &fgid)?;

        let mut results = HandleResult::new();

        match event {
            LayerEvent::Message(hash, m) => {
                let db = session_db(&layer.base, &mgid)?;
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
                let f = layer.update_friend(&mgid, fid, remote)?;
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
                    .check_add_online(fgid, Online::Direct(addr))?;
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
                    .check_add_online(fgid, Online::Direct(addr))?;
                results.rpcs.push(rpc::friend_online(mgid, fid, addr));
            }
            LayerEvent::Offline => {
                layer.group.write().await.status(
                    &mgid,
                    StatusEvent::SessionFriendOffline(fgid),
                    &mut results,
                )?;
                layer.running_mut(&mgid)?.check_offline(&fgid, &addr);
                results.rpcs.push(rpc::friend_offline(mgid, fid));
            }
            LayerEvent::Close => {
                layer.group.write().await.broadcast(
                    &mgid,
                    InnerEvent::SessionFriendClose(fgid),
                    FRIEND_TABLE_PATH,
                    fid,
                    &mut results,
                )?;
                layer.remove_friend(&mgid, &fgid);
                let db = session_db(&layer.base, &mgid)?;
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
        let db = session_db(&base, &mgid)?;

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
        Friend::update_last_message(&db, fid, &msg, true)?;
        drop(db);
        Ok((msg, nm_type))
    }
}
