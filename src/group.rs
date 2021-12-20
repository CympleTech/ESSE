use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tdn::types::{
    group::{EventId, GroupId},
    message::{RecvType, SendMessage, SendType},
    primitive::{HandleResult, Peer, PeerId, Result},
};
use tdn_did::Proof;
use tokio::sync::{mpsc::Sender, RwLock};

use crate::account::{Account, User};
use crate::apps::device::rpc as device_rpc;
use crate::apps::device::Device;
use crate::consensus::Event;
use crate::event::{InnerEvent, StatusEvent, SyncEvent};
use crate::layer::Layer;
use crate::rpc;
use crate::storage::{account_db, account_init, consensus_db, write_avatar};
use crate::utils::crypto::{decrypt, encrypt};
use crate::utils::device_status::{device_info, device_status as local_device_status};

pub(crate) mod running;

use running::RunningAccount;

/// Esse group.
pub(crate) struct Group {
    /// storage base path.
    base: PathBuf,
    /// random secret seed.
    secret: [u8; 32],
    /// TDN network sender.
    sender: Sender<SendMessage>,
    /// current address.
    addr: PeerId,
    /// all accounts.
    accounts: HashMap<GroupId, Account>,
    /// distributed devices.
    runnings: HashMap<GroupId, RunningAccount>,
}

/// Request for make distributed.
#[derive(Serialize, Deserialize)]
enum GroupConnect {
    /// Params: User, consensus height, event_id, remote_name, remote_info, other_devices addr.
    Create(Proof, User, u64, EventId, String, String, Vec<PeerId>),
    /// connected.
    Connect(u64, EventId),
}

/// Esse group's Event.
#[derive(Serialize, Deserialize)]
pub(crate) enum GroupEvent {
    /// Sync event.
    Event(u64, EventId, EventId, InnerEvent),
    /// Sync infomations.
    Status(StatusEvent),
    /// device's info update.
    DeviceUpdate(PeerId, String),
    /// device deleted.
    DeviceDelete(PeerId),
    /// offline.
    DeviceOffline,
    /// Device status request.
    StatusRequest,
    /// Device status response.
    /// (cpu_num, memory_space, swap_space, disk_space, cpu%, memory%, swap%, disk%, uptime).
    StatusResponse(u32, u32, u32, u32, u16, u16, u16, u16, u32),
    /// check consensus stable.
    SyncCheck(Vec<u64>, Vec<EventId>, bool),
    /// Sync height from..to request.
    SyncRequest(u64, u64),
    /// Sync height from..last_to, to, response.
    SyncResponse(u64, u64, u64, Vec<SyncEvent>),
}

impl Group {
    pub async fn handle(
        &mut self,
        gid: GroupId,
        msg: RecvType,
        layer: &Arc<RwLock<Layer>>,
        uid: u64,
    ) -> Result<HandleResult> {
        let mut results = HandleResult::new();

        // 1. check account is online, if not online, nothing.
        if !self.runnings.contains_key(&gid) {
            return Ok(results);
        }

        match msg {
            RecvType::Connect(addr, data) => {
                self.hanlde_connect(&mut results, &gid, addr, data, true)?;
            }
            RecvType::Leave(addr) => {
                for (_, account) in &mut self.runnings {
                    if let Some(device) = account.distributes.get_mut(&addr) {
                        device.2 = false;
                        results.rpcs.push(device_rpc::device_offline(gid, device.1));
                    }
                }
            }
            RecvType::Result(addr, is_ok, data) => {
                if is_ok {
                    self.hanlde_connect(&mut results, &gid, addr, data, false)?;
                }
            }
            RecvType::ResultConnect(addr, data) => {
                self.hanlde_connect(&mut results, &gid, addr, data, true)?;
            }
            RecvType::Event(addr, bytes) => {
                let event: GroupEvent = bincode::deserialize(&bytes)?;
                return GroupEvent::handle(self, event, gid, addr, layer, uid).await;
            }
            RecvType::Stream(_uid, _stream, _bytes) => {
                todo!();
                // TODO stream
            }
            RecvType::Delivery(_t, _tid, _is_ok) => {}
        }

        Ok(results)
    }

    fn hanlde_connect(
        &mut self,
        results: &mut HandleResult,
        gid: &GroupId,
        addr: Peer,
        data: Vec<u8>,
        is_connect: bool,
    ) -> Result<()> {
        let connect = bincode::deserialize(&data)?;
        let peer_id = addr.id;

        let (remote_height, remote_event, others) = match connect {
            GroupConnect::Create(
                proof,
                remote,
                remote_height,
                remote_event,
                device_name,
                device_info,
                others,
            ) => {
                // check remote addr is receive addr.
                if remote.addr != peer_id {
                    return Err(anyhow!("Address is invalid."));
                }
                proof.verify(gid, &peer_id, &self.addr)?;
                if is_connect {
                    results
                        .groups
                        .push((*gid, self.agree_message(gid, addr.clone())?));
                }

                // first init sync.
                if remote.avatar.len() > 0 {
                    if let Some(u) = self.accounts.get_mut(gid) {
                        if u.avatar.len() == 0 {
                            u.name = remote.name;
                            u.avatar = remote.avatar;
                            let account_db = account_db(&self.base)?;
                            u.update(&account_db)?;
                            account_db.close()?;
                            results.rpcs.push(rpc::account_update(
                                *gid,
                                &u.name,
                                base64::encode(&u.avatar),
                            ));
                        }
                    }
                }

                let running = self.runnings.get_mut(gid).unwrap(); // safe unwrap. checked.
                let mut new_addrs = vec![];
                for a in others {
                    if a != peer_id && a != self.addr && !running.distributes.contains_key(&a) {
                        new_addrs.push(a);
                    }
                }

                if let Some(v) = running.distributes.get_mut(&peer_id) {
                    v.2 = true;
                    results.rpcs.push(device_rpc::device_online(*gid, v.1));
                    (remote_height, remote_event, new_addrs)
                } else {
                    let mut device = Device::new(device_name, device_info, peer_id);
                    let db = consensus_db(&self.base, gid)?;
                    device.insert(&db)?;
                    db.close()?;
                    running
                        .distributes
                        .insert(peer_id, (addr.clone(), device.id, true));
                    results.rpcs.push(device_rpc::device_create(*gid, &device));
                    results
                        .rpcs
                        .push(device_rpc::device_online(*gid, device.id));
                    (remote_height, remote_event, new_addrs)
                }
            }
            GroupConnect::Connect(remote_height, remote_event) => {
                if self
                    .runnings
                    .get(gid)
                    .unwrap() // safe, checked
                    .distributes
                    .contains_key(&peer_id)
                {
                    if is_connect {
                        results.groups.push((*gid, self.connect_result(gid, addr)?));
                    }
                } else {
                    if is_connect {
                        results.groups.push((*gid, self.create_message(gid, addr)?));
                    }
                    return Ok(());
                }

                let v = self.running_mut(gid)?;
                let did = v.add_online(&peer_id)?;
                results.rpcs.push(device_rpc::device_online(*gid, did));
                (remote_height, remote_event, vec![])
            }
        };

        let account = self.account(gid)?;
        if account.height != remote_height || account.event != remote_event {
            results
                .groups
                .push((*gid, self.sync_message(gid, peer_id, 1, account.height)?));
        }

        // connect to others.
        for addr in others {
            results
                .groups
                .push((*gid, self.create_message(gid, Peer::peer(addr))?));
        }

        Ok(())
    }
}

impl Group {
    pub async fn init(
        secret: [u8; 32],
        sender: Sender<SendMessage>,
        addr: PeerId,
        accounts: HashMap<GroupId, Account>,
        base: PathBuf,
    ) -> Result<Group> {
        Ok(Group {
            secret,
            sender,
            addr,
            accounts,
            base,
            runnings: HashMap::new(),
        })
    }

    pub fn addr(&self) -> &PeerId {
        &self.addr
    }

    pub fn base(&self) -> &PathBuf {
        &self.base
    }

    pub fn sender(&self) -> Sender<SendMessage> {
        self.sender.clone()
    }

    pub fn check_lock(&self, gid: &GroupId, lock: &str) -> bool {
        if let Some(account) = self.accounts.get(gid) {
            account.check_lock(&self.secret, lock).is_ok()
        } else {
            false
        }
    }

    pub fn account(&self, gid: &GroupId) -> Result<&Account> {
        if let Some(account) = self.accounts.get(gid) {
            Ok(account)
        } else {
            Err(anyhow!("user missing"))
        }
    }

    pub fn account_mut(&mut self, gid: &GroupId) -> Result<&mut Account> {
        if let Some(account) = self.accounts.get_mut(gid) {
            Ok(account)
        } else {
            Err(anyhow!("user missing"))
        }
    }

    pub fn running(&self, gid: &GroupId) -> Result<&RunningAccount> {
        if let Some(running) = self.runnings.get(gid) {
            Ok(running)
        } else {
            Err(anyhow!("user missing"))
        }
    }

    pub fn running_mut(&mut self, gid: &GroupId) -> Result<&mut RunningAccount> {
        if let Some(running) = self.runnings.get_mut(gid) {
            Ok(running)
        } else {
            Err(anyhow!("user missing"))
        }
    }

    pub fn prove_addr(&self, mgid: &GroupId, raddr: &PeerId) -> Result<Proof> {
        let running = self.running(mgid)?;
        Ok(Proof::prove(&running.keypair, &self.addr, raddr))
    }

    pub fn uptime(&self, gid: &GroupId) -> Result<u32> {
        self.running(gid).map(|v| v.uptime)
    }

    pub fn list_running_user(&self) -> Vec<GroupId> {
        self.runnings.keys().map(|d| *d).collect()
    }

    pub fn distribute_conns(&self, gid: &GroupId) -> Vec<SendType> {
        let mut vecs = vec![];
        if let Some(running) = &self.runnings.get(gid) {
            for (addr, (peer, _, _)) in &running.distributes {
                if addr != &self.addr {
                    if let Ok(s) = self.connect_message(gid, peer.clone()) {
                        vecs.push(s);
                    }
                }
            }
        }
        vecs
    }

    pub fn all_distribute_conns(&self) -> HashMap<GroupId, Vec<SendType>> {
        let mut conns = HashMap::new();
        for (mgid, running) in &self.runnings {
            let mut vecs = vec![];
            for (addr, (peer, _, _)) in &running.distributes {
                if addr != &self.addr {
                    if let Ok(s) = self.connect_message(mgid, peer.clone()) {
                        vecs.push(s);
                    }
                }
            }
            conns.insert(*mgid, vecs);
        }
        conns
    }

    pub fn online_devices(&self, gid: &GroupId, mut devices: Vec<Device>) -> Vec<Device> {
        if let Some(running) = self.runnings.get(gid) {
            for (addr, (_peer, _id, online)) in &running.distributes {
                if *online {
                    for device in devices.iter_mut() {
                        if device.addr == *addr {
                            device.online = true;
                        }
                    }
                }
            }
        }

        devices
    }

    pub fn remove_all_running(&mut self) -> HashMap<PeerId, ()> {
        let mut addrs: HashMap<PeerId, ()> = HashMap::new();
        for (_, running) in self.runnings.drain() {
            for (addr, (_peer, _id, online)) in running.distributes {
                if addr != self.addr && online {
                    addrs.insert(addr, ());
                }
            }
        }
        addrs
    }

    pub fn remove_running(&mut self, gid: &GroupId) -> HashMap<PeerId, ()> {
        // check close the stable connection.
        let mut addrs: HashMap<PeerId, ()> = HashMap::new();
        if let Some(running) = self.runnings.remove(gid) {
            for (addr, (_peer, _id, online)) in running.distributes {
                if addr != self.addr && online {
                    addrs.insert(addr, ());
                }
            }

            // check if other stable connection.
            for other_running in self.runnings.values() {
                for (addr, (_peer, _id, online)) in &other_running.distributes {
                    if *online && addrs.contains_key(addr) {
                        addrs.remove(addr);
                    }
                }
            }
        }

        addrs
    }

    pub fn add_running(&mut self, gid: &GroupId, lock: &str) -> Result<(i64, bool)> {
        if let Some(u) = self.accounts.get(gid) {
            let keypair = u.secret(&self.secret, lock)?;
            if !self.runnings.contains_key(gid) {
                // load devices to runnings.
                let running = RunningAccount::init(keypair, &self.base, gid)?;
                self.runnings.insert(gid.clone(), running);
                Ok((u.id, false))
            } else {
                Ok((u.id, true))
            }
        } else {
            Err(anyhow!("user missing."))
        }
    }

    pub fn clone_user(&self, gid: &GroupId) -> Result<User> {
        if let Some(u) = self.accounts.get(gid) {
            Ok(User::simple(
                u.gid,
                self.addr,
                u.name.clone(),
                u.avatar.clone(),
            ))
        } else {
            Err(anyhow!("user missing."))
        }
    }

    pub fn list_users(&self) -> &HashMap<GroupId, Account> {
        &self.accounts
    }

    pub async fn add_account(
        &mut self,
        lang: i64,
        seed: &str,
        pass: &str,
        name: &str,
        lock: &str,
        avatar_bytes: Vec<u8>,
    ) -> Result<(i64, GroupId)> {
        let account_index = self.accounts.len() as u32;
        let (mut account, sk) = Account::generate(
            account_index,
            &self.secret,
            lang,
            seed,
            pass,
            name,
            lock,
            avatar_bytes,
        )?;
        let account_id = account.gid;

        if let Some(u) = self.accounts.get(&account_id) {
            let running = RunningAccount::init(sk, &self.base, &account_id)?;
            self.runnings.insert(account_id, running);
            return Ok((u.id, account_id));
        }

        account_init(&self.base, &account.gid).await?;

        let account_db = account_db(&self.base)?;
        account.insert(&account_db)?;
        account_db.close()?;
        let account_did = account.id;
        let _ = write_avatar(&self.base, &account_id, &account_id, &account.avatar).await;
        self.accounts.insert(account.gid, account);

        let (device_name, device_info) = device_info();
        let mut device = Device::new(device_name, device_info, self.addr);
        let db = consensus_db(&self.base, &account_id)?;
        device.insert(&db)?;
        db.close()?;

        self.runnings.insert(
            account_id,
            RunningAccount::init(sk, &self.base, &account_id)?,
        );

        Ok((account_did, account_id))
    }

    pub fn update_account(&mut self, gid: GroupId, name: &str, avatar: Vec<u8>) -> Result<()> {
        let account_db = account_db(&self.base)?;
        let account = self.account_mut(&gid)?;
        account.name = name.to_owned();
        if avatar.len() > 0 {
            account.avatar = avatar;
        }
        account.update_info(&account_db)?;
        account_db.close()
    }

    pub fn mnemonic(&self, gid: &GroupId, lock: &str) -> Result<String> {
        if let Some(u) = self.accounts.get(gid) {
            u.mnemonic(&self.secret, lock)
        } else {
            Err(anyhow!("user missing."))
        }
    }

    pub fn pin(&mut self, gid: &GroupId, lock: &str, new: &str) -> Result<()> {
        if let Some(u) = self.accounts.get_mut(gid) {
            u.pin(&self.secret, lock, new)?;
            let account_db = account_db(&self.base)?;
            u.update(&account_db)?;
            account_db.close()
        } else {
            Err(anyhow!("user missing."))
        }
    }

    pub fn encrypt(&self, gid: &GroupId, lock: &str, bytes: &[u8]) -> Result<Vec<u8>> {
        let ckey = &self.account(gid)?.encrypt;
        encrypt(&self.secret, lock, ckey, bytes)
    }
    pub fn decrypt(&self, gid: &GroupId, lock: &str, bytes: &[u8]) -> Result<Vec<u8>> {
        let ckey = &self.account(gid)?.encrypt;
        decrypt(&self.secret, lock, ckey, bytes)
    }

    pub fn create_message(&self, gid: &GroupId, addr: Peer) -> Result<SendType> {
        let user = self.clone_user(gid)?;
        let account = self.account(gid)?;
        let height = account.height;
        let event = account.event;
        let proof = self.prove_addr(gid, &addr.id)?;
        let running = self.running(gid)?;

        Ok(SendType::Connect(
            0,
            addr,
            bincode::serialize(&GroupConnect::Create(
                proof,
                user,
                height,
                event,
                running.device_name.clone(),
                running.device_info.clone(),
                running.distributes.keys().cloned().collect(),
            ))
            .unwrap_or(vec![]),
        ))
    }

    pub fn connect_message(&self, gid: &GroupId, addr: Peer) -> Result<SendType> {
        let account = self.account(gid)?;
        let height = account.height;
        let event = account.event;
        let data = bincode::serialize(&GroupConnect::Connect(height, event)).unwrap_or(vec![]);
        Ok(SendType::Connect(0, addr, data))
    }

    pub fn connect_result(&self, gid: &GroupId, addr: Peer) -> Result<SendType> {
        let account = self.account(gid)?;
        let height = account.height;
        let event = account.event;
        let data = bincode::serialize(&GroupConnect::Connect(height, event)).unwrap_or(vec![]);
        Ok(SendType::Result(0, addr, true, false, data))
    }

    pub fn agree_message(&self, gid: &GroupId, addr: Peer) -> Result<SendType> {
        let account = self.account(gid)?;
        let height = account.height;
        let event = account.event;
        let me = self.clone_user(gid)?;
        let proof = self.prove_addr(gid, &addr.id)?;
        let running = self.running(gid)?;

        Ok(SendType::Result(
            0,
            addr,
            true,
            false,
            bincode::serialize(&GroupConnect::Create(
                proof,
                me,
                height,
                event,
                running.device_name.clone(),
                running.device_info.clone(),
                running.distributes.keys().cloned().collect(),
            ))
            .unwrap_or(vec![]),
        ))
    }

    fn ancestor(from: u64, to: u64) -> (Vec<u64>, bool) {
        let space = to - from;
        let step = space / 8;
        if step == 0 {
            ((from..to + 1).map(|i| i).collect(), true)
        } else {
            let mut vec: Vec<u64> = (1..8).map(|i| step * i + from).collect();
            vec.push(to);
            (vec, false)
        }
    }

    pub fn sync_message(
        &self,
        gid: &GroupId,
        addr: PeerId,
        from: u64,
        to: u64,
    ) -> Result<SendType> {
        let (ancestors, hashes, is_min) = if to >= from {
            let (ancestors, is_min) = Self::ancestor(from, to);
            let db = consensus_db(&self.base, gid)?;
            let hashes = crate::consensus::Event::get_assign_hash(&db, &ancestors)?;
            db.close()?;
            (ancestors, hashes, is_min)
        } else {
            (vec![], vec![], true)
        };

        let event = GroupEvent::SyncCheck(ancestors, hashes, is_min);
        let data = bincode::serialize(&event).unwrap_or(vec![]);
        Ok(SendType::Event(0, addr, data))
    }

    pub fn event_message(&self, addr: PeerId, event: &GroupEvent) -> Result<SendType> {
        let data = bincode::serialize(event).unwrap_or(vec![]);
        Ok(SendType::Event(0, addr, data))
    }

    pub fn broadcast(
        &mut self,
        gid: &GroupId,
        event: InnerEvent,
        path: i64,
        row: i64,
        results: &mut HandleResult,
    ) -> Result<()> {
        let base = self.base.clone();

        let account = self.account_mut(gid)?;
        let pre_event = account.event;
        let eheight = account.height + 1;
        let eid = event.generate_event_id();

        let db = consensus_db(&base, gid)?;
        Event::merge(&db, eid, path, row, eheight)?;
        drop(db);
        let account_db = account_db(&base)?;
        account.update_consensus(&account_db, eheight, eid)?;
        account_db.close()?;
        drop(account);

        let e = GroupEvent::Event(eheight, eid, pre_event, event);
        let data = bincode::serialize(&e).unwrap_or(vec![]);
        let running = self.running(gid)?;
        for (addr, (_peer, _id, online)) in &running.distributes {
            if *online {
                let msg = SendType::Event(0, *addr, data.clone());
                results.groups.push((*gid, msg))
            }
        }
        Ok(())
    }

    pub fn _status(
        &mut self,
        gid: &GroupId,
        event: StatusEvent,
        results: &mut HandleResult,
    ) -> Result<()> {
        let running = self.running(gid)?;
        let data = bincode::serialize(&GroupEvent::Status(event)).unwrap_or(vec![]);
        for (addr, (_peer, _id, online)) in &running.distributes {
            if *online {
                let msg = SendType::Event(0, *addr, data.clone());
                results.groups.push((*gid, msg))
            }
        }
        Ok(())
    }
}

impl GroupEvent {
    pub async fn handle(
        group: &mut Group,
        event: GroupEvent,
        gid: GroupId,
        addr: PeerId,
        layer: &Arc<RwLock<Layer>>,
        uid: u64,
    ) -> Result<HandleResult> {
        let mut results = HandleResult::new();
        match event {
            GroupEvent::DeviceUpdate(_at, _name) => {
                // TODO
            }
            GroupEvent::DeviceDelete(_at) => {
                // TODO
            }
            GroupEvent::DeviceOffline => {
                let v = group.running_mut(&gid)?;
                let did = v.offline(&addr)?;
                results.rpcs.push(device_rpc::device_offline(gid, did));
            }
            GroupEvent::StatusRequest => {
                let (cpu_n, mem_s, swap_s, disk_s, cpu_p, mem_p, swap_p, disk_p) =
                    local_device_status();
                results.groups.push((
                    gid,
                    SendType::Event(
                        0,
                        addr,
                        bincode::serialize(&GroupEvent::StatusResponse(
                            cpu_n,
                            mem_s,
                            swap_s,
                            disk_s,
                            cpu_p,
                            mem_p,
                            swap_p,
                            disk_p,
                            group.uptime(&gid)?,
                        ))
                        .unwrap_or(vec![]),
                    ),
                ))
            }
            GroupEvent::StatusResponse(
                cpu_n,
                mem_s,
                swap_s,
                disk_s,
                cpu_p,
                mem_p,
                swap_p,
                disk_p,
                uptime,
            ) => results.rpcs.push(device_rpc::device_status(
                gid, cpu_n, mem_s, swap_s, disk_s, cpu_p, mem_p, swap_p, disk_p, uptime,
            )),
            GroupEvent::Event(eheight, eid, pre, inner_event) => {
                inner_event.handle(group, gid, addr, eheight, eid, pre, &mut results, layer)?;
            }
            GroupEvent::Status(status_event) => {
                status_event.handle(group, gid, addr, &mut results, layer, uid)?;
            }
            GroupEvent::SyncCheck(ancestors, hashes, is_min) => {
                println!("sync check: {:?}", ancestors);
                let account = group.account(&gid)?;
                if ancestors.len() == 0 || hashes.len() == 0 {
                    return Ok(results);
                }

                // remote is new need it handle.
                if hashes[0] == EventId::default() {
                    return Ok(results);
                }

                let remote_height = ancestors.last().map(|v| *v).unwrap_or(0);
                let remote_event = hashes.last().map(|v| *v).unwrap_or(EventId::default());
                if account.height != remote_height || account.event != remote_event {
                    // check ancestor and merge.
                    let db = consensus_db(&group.base, &gid)?;
                    let ours = crate::consensus::Event::get_assign_hash(&db, &ancestors)?;
                    drop(db);

                    if ours.len() == 0 {
                        let event = GroupEvent::SyncRequest(1, remote_height);
                        let data = bincode::serialize(&event).unwrap_or(vec![]);
                        results.groups.push((gid, SendType::Event(0, addr, data)));
                        return Ok(results);
                    }

                    let mut ancestor = 0u64;
                    for i in 0..ancestors.len() {
                        if hashes[i] != ours[i] {
                            if i == 0 {
                                ancestor = ancestors[0];
                                break;
                            }

                            if ancestors[i - 1] == ancestors[i] + 1 {
                                ancestor = ancestors[i - 1];
                            } else {
                                if is_min {
                                    ancestor = ancestors[i - 1];
                                } else {
                                    results.groups.push((
                                        gid,
                                        group.sync_message(
                                            &gid,
                                            addr,
                                            ancestors[i - 1],
                                            ancestors[i],
                                        )?,
                                    ));
                                    return Ok(results);
                                }
                            }

                            break;
                        }
                    }

                    if ancestor != 0 {
                        let event = GroupEvent::SyncRequest(ancestor, remote_height);
                        let data = bincode::serialize(&event).unwrap_or(vec![]);
                        results.groups.push((gid, SendType::Event(0, addr, data)));
                    } else {
                        results.groups.push((
                            gid,
                            group.sync_message(&gid, addr, remote_height, account.height)?,
                        ));
                    }
                }
            }
            GroupEvent::SyncRequest(from, to) => {
                println!("====== DEBUG Sync Request: from: {} to {}", from, to);
                // every time sync MAX is 100.
                let last_to = if to - from > 100 { to - 100 } else { to };
                let sync_events =
                    SyncEvent::sync(&group.base, &gid, group.account(&gid)?, from, last_to).await?;
                let event = GroupEvent::SyncResponse(from, last_to, to, sync_events);
                let data = bincode::serialize(&event).unwrap_or(vec![]);
                results.groups.push((gid, SendType::Event(0, addr, data)));
            }
            GroupEvent::SyncResponse(from, last_to, to, events) => {
                println!(
                    "====== DEBUG Sync Response: from: {} last {}, to {}",
                    from, last_to, to
                );
                if last_to < to {
                    let event = GroupEvent::SyncRequest(last_to + 1, to);
                    let data = bincode::serialize(&event).unwrap_or(vec![]);
                    results.groups.push((gid, SendType::Event(0, addr, data)));
                }
                SyncEvent::handle(gid, from, last_to, events, group, layer, &mut results, addr)?;
            }
        }

        Ok(results)
    }
}
