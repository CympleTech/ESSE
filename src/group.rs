use esse_primitives::id_to_str;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::{EventId, GroupId},
    message::{RecvType, SendMessage, SendType},
    primitives::{HandleResult, Peer, PeerId, PeerKey, Result},
};
use tdn_storage::local::DStorage;
use tokio::sync::{mpsc::Sender, RwLock};

use crate::account::{Account, User};
use crate::global::Global;
//use crate::apps::device::rpc as device_rpc;
use crate::apps::device::Device;
//use crate::consensus::Event;
//use crate::event::{InnerEvent, StatusEvent, SyncEvent};
//use crate::layer::Layer;
//use crate::rpc;
use crate::storage::{account_db, account_init, consensus_db, write_avatar};
use crate::utils::crypto::{decrypt, encrypt};
use crate::utils::device_status::{device_info, device_status as local_device_status};

/// Esse group.
pub(crate) struct Group {
    /// all accounts.
    pub accounts: HashMap<PeerId, Account>,
    /// current account secret keypair.
    pub keypair: PeerKey,
    /// current account distribute connected devices.
    pub distributes: Vec<Device>,
    /// current account uptime
    pub uptime: u32,
}

/// Request for make distributed.
#[derive(Serialize, Deserialize)]
enum GroupConnect {
    /// Params: User, consensus height, event_id, remote_name, remote_info, other_devices addr.
    Create(User, u64, EventId, String, String, Vec<PeerId>),
    /// connected.
    Connect(u64, EventId),
}

/// Esse group's Event.
#[derive(Serialize, Deserialize)]
pub(crate) enum GroupEvent {
    /// Sync event.
    Event(u64, EventId, EventId),
    //Event(u64, EventId, EventId, InnerEvent),
    /// Sync infomations.
    Status,
    //Status(StatusEvent),
    /// device's info update.
    DeviceUpdate(String),
    /// device deleted.
    DeviceDelete,
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
    SyncResponse(u64, u64, u64),
    //SyncResponse(u64, u64, u64, Vec<SyncEvent>),
}

/// handle inner-group message.
pub(crate) async fn handle(msg: RecvType, global: &Arc<Global>) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Connect(peer, data) => {
            //self.hanlde_connect(&mut results, peer, data, true)?;
        }
        RecvType::Leave(peer) => {
            // check device leave.
            //if let Ok(id) = account.offline(&peer) {
            //results.rpcs.push(device_rpc::device_offline(peer.id, id));
            //}
        }
        RecvType::Result(peer, is_ok, data) => {
            if is_ok {
                //self.hanlde_connect(&mut results, peer, data, false)?;
            }
        }
        RecvType::ResultConnect(peer, data) => {
            //self.hanlde_connect(&mut results, peer, data, true)?;
        }
        RecvType::Event(addr, bytes) => {
            //let event: GroupEvent = bincode::deserialize(&bytes)?;
            //return GroupEvent::handle(self, event, pid, addr, uid).await;
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            todo!();
            // TODO stream
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {}
    }

    Ok(results)
}

//     fn hanlde_connect(
//         &mut self,
//         results: &mut HandleResult,
//         peer: Peer,
//         data: Vec<u8>,
//         is_connect: bool,
//     ) -> Result<()> {
//         let connect = bincode::deserialize(&data)?;
//         let pid = peer.id;

//         let (remote_height, remote_event, others) = match connect {
//             GroupConnect::Create(
//                 remote,
//                 remote_height,
//                 remote_event,
//                 device_name,
//                 device_info,
//                 others,
//             ) => {
//                 // check remote addr is receive addr.
//                 if remote.addr != pid {
//                     return Err(anyhow!("Address is invalid."));
//                 }

//                 if is_connect {
//                     results
//                         .groups
//                         .push((pid, self.agree_message(peer.clone())?));
//                 }

//                 // first init sync.
//                 if remote.avatar.len() > 0 {
//                     let account_db = self.account_db()?;
//                     if let Some(u) = self.accounts.get_mut(pid) {
//                         if u.avatar.len() == 0 {
//                             u.name = remote.name;
//                             u.avatar = remote.avatar;
//                             u.update(&account_db)?;
//                             account_db.close()?;
//                             results.rpcs.push(rpc::account_update(
//                                 *pid,
//                                 &u.name,
//                                 base64::encode(&u.avatar),
//                             ));
//                         }
//                     }
//                 }

//                 let db = self.consensus_db(pid)?;
//                 let running = self.runnings.get_mut(pid).unwrap(); // safe unwrap. checked.
//                 let mut new_addrs = vec![];
//                 for a in others {
//                     if a != peer_id && a != self.addr && !running.distributes.contains_key(&a) {
//                         new_addrs.push(a);
//                     }
//                 }

//                 if let Some(v) = running.distributes.get_mut(&peer_id) {
//                     v.2 = true;
//                     results.rpcs.push(device_rpc::device_online(*pid, v.1));
//                     (remote_height, remote_event, new_addrs)
//                 } else {
//                     let mut device = Device::new(device_name, device_info, peer_id);
//                     device.insert(&db)?;
//                     db.close()?;
//                     running
//                         .distributes
//                         .insert(peer_id, (addr.clone(), device.id, true));
//                     results.rpcs.push(device_rpc::device_create(*pid, &device));
//                     results
//                         .rpcs
//                         .push(device_rpc::device_online(*pid, device.id));
//                     (remote_height, remote_event, new_addrs)
//                 }
//             }
//             GroupConnect::Connect(remote_height, remote_event) => {
//                 if self
//                     .runnings
//                     .get(pid)
//                     .unwrap() // safe, checked
//                     .distributes
//                     .contains_key(&peer_id)
//                 {
//                     if is_connect {
//                         results.groups.push((*pid, self.connect_result(pid, addr)?));
//                     }
//                 } else {
//                     if is_connect {
//                         results.groups.push((*pid, self.create_message(pid, addr)?));
//                     }
//                     return Ok(());
//                 }

//                 let v = self.running_mut(pid)?;
//                 let did = v.add_online(&peer_id)?;
//                 results.rpcs.push(device_rpc::device_online(*pid, did));
//                 (remote_height, remote_event, vec![])
//             }
//         };

//         let account = self.account(pid)?;
//         if account.own_height != remote_height || account.event != remote_event {
//             results.groups.push((
//                 *pid,
//                 self.sync_message(pid, peer_id, 1, account.own_height)?,
//             ));
//         }

//         // connect to others.
//         for addr in others {
//             results
//                 .groups
//                 .push((*pid, self.create_message(pid, Peer::peer(addr))?));
//         }

//         Ok(())
//     }
// }

impl Group {
    pub fn init(accounts: HashMap<PeerId, Account>) -> Group {
        Group {
            accounts,
            keypair: PeerKey::default(),
            distributes: vec![],
            uptime: 0,
        }
    }

    pub fn keypair(&self) -> PeerKey {
        let bytes = self.keypair.to_db_bytes();
        PeerKey::from_db_bytes(&bytes).unwrap()
    }

    pub fn db_key(&self, pid: &PeerId) -> Result<String> {
        Ok(self.account(pid)?.plainkey())
    }

    // pub fn online(&mut self, peer: &Peer) -> Result<i64> {
    //     for i in self.distributes.iter_mut() {
    //         if &i.0 == peer {
    //             i.2 = true;
    //             return Ok(i.1);
    //         }
    //     }
    //     Err(anyhow!("missing distribute device"))
    // }

    // pub fn offline(&mut self, peer: &Peer) -> Result<i64> {
    //     for i in self.distributes.iter_mut() {
    //         if &i.0 == peer {
    //             i.2 = false;
    //             return Ok(i.1);
    //         }
    //     }
    //     Err(anyhow!("missing distribute device"))
    // }

    pub fn check_lock(&self, pid: &PeerId, lock: &str) -> bool {
        if let Some(account) = self.accounts.get(pid) {
            account.check_lock(lock).is_ok()
        } else {
            false
        }
    }

    pub fn account(&self, pid: &PeerId) -> Result<&Account> {
        if let Some(account) = self.accounts.get(pid) {
            Ok(account)
        } else {
            Err(anyhow!("account missing"))
        }
    }

    pub fn account_mut(&mut self, pid: &PeerId) -> Result<&mut Account> {
        if let Some(account) = self.accounts.get_mut(pid) {
            Ok(account)
        } else {
            Err(anyhow!("account missing"))
        }
    }

    //     pub fn running(&self, pid: &PeerId) -> Result<&RunningAccount> {
    //         if let Some(running) = self.runnings.get(pid) {
    //             Ok(running)
    //         } else {
    //             Err(anyhow!("user missing"))
    //         }
    //     }

    //     pub fn running_mut(&mut self, pid: &PeerId) -> Result<&mut RunningAccount> {
    //         if let Some(running) = self.runnings.get_mut(pid) {
    //             Ok(running)
    //         } else {
    //             Err(anyhow!("user missing"))
    //         }
    //     }

    //     pub fn prove_addr(&self, mpid: &PeerId, raddr: &PeerId) -> Result<Proof> {
    //         let running = self.running(mpid)?;
    //         Ok(Proof::prove(&running.keypair, &self.addr, raddr))
    //     }

    //     pub fn uptime(&self, pid: &PeerId) -> Result<u32> {
    //         self.running(pid).map(|v| v.uptime)
    //     }

    //     pub fn list_running_user(&self) -> Vec<PeerId> {
    //         self.runnings.keys().map(|d| *d).collect()
    //     }

    //     pub fn distribute_conns(&self, pid: &PeerId) -> Vec<SendType> {
    //         let mut vecs = vec![];
    //         if let Some(running) = &self.runnings.get(pid) {
    //             for (addr, (peer, _, _)) in &running.distributes {
    //                 if addr != &self.addr {
    //                     if let Ok(s) = self.connect_message(pid, peer.clone()) {
    //                         vecs.push(s);
    //                     }
    //                 }
    //             }
    //         }
    //         vecs
    //     }

    //     pub fn all_distribute_conns(&self) -> HashMap<PeerId, Vec<SendType>> {
    //         let mut conns = HashMap::new();
    //         for (mpid, running) in &self.runnings {
    //             let mut vecs = vec![];
    //             for (addr, (peer, _, _)) in &running.distributes {
    //                 if addr != &self.addr {
    //                     if let Ok(s) = self.connect_message(mpid, peer.clone()) {
    //                         vecs.push(s);
    //                     }
    //                 }
    //             }
    //             conns.insert(*mpid, vecs);
    //         }
    //         conns
    //     }

    //     pub fn online_devices(&self, pid: &PeerId, mut devices: Vec<Device>) -> Vec<Device> {
    //         if let Some(running) = self.runnings.get(pid) {
    //             for (addr, (_peer, _id, online)) in &running.distributes {
    //                 if *online {
    //                     for device in devices.iter_mut() {
    //                         if device.addr == *addr {
    //                             device.online = true;
    //                         }
    //                     }
    //                 }
    //             }
    //         }

    //         devices
    //     }

    //     pub fn remove_all_running(&mut self) -> HashMap<PeerId, ()> {
    //         let mut addrs: HashMap<PeerId, ()> = HashMap::new();
    //         for (_, running) in self.runnings.drain() {
    //             for (addr, (_peer, _id, online)) in running.distributes {
    //                 if addr != self.addr && online {
    //                     addrs.insert(addr, ());
    //                 }
    //             }
    //         }
    //         addrs
    //     }

    //     pub fn remove_running(&mut self, pid: &PeerId) -> HashMap<PeerId, ()> {
    //         // check close the stable connection.
    //         let mut addrs: HashMap<PeerId, ()> = HashMap::new();
    //         if let Some(running) = self.runnings.remove(pid) {
    //             for (addr, (_peer, _id, online)) in running.distributes {
    //                 if addr != self.addr && online {
    //                     addrs.insert(addr, ());
    //                 }
    //             }

    //             // check if other stable connection.
    //             for other_running in self.runnings.values() {
    //                 for (addr, (_peer, _id, online)) in &other_running.distributes {
    //                     if *online && addrs.contains_key(addr) {
    //                         addrs.remove(addr);
    //                     }
    //                 }
    //             }
    //         }

    //         addrs
    //     }

    /// reset group info when change account.
    pub fn reset(
        &mut self,
        pid: &PeerId,
        lock: &str,
        base: &PathBuf,
        secret: &[u8],
    ) -> Result<(u64, u64)> {
        let (keypair, pheight, oheight, key) = if let Some(u) = self.accounts.get_mut(pid) {
            let keypair = u.secret(secret, lock)?;
            u.cache_plainkey(secret, lock)?;
            (keypair, u.pub_height, u.own_height, u.plainkey())
        } else {
            return Err(anyhow!("user missing."));
        };

        self.keypair = keypair;

        let db = consensus_db(base, pid, &self.db_key(pid)?)?;
        self.distributes = Device::list(&db)?;
        db.close()?;

        let start = SystemTime::now();
        self.uptime = start
            .duration_since(UNIX_EPOCH)
            .map(|s| s.as_secs())
            .unwrap_or(0) as u32; // safe for all life.

        Ok((pheight, oheight))
    }

    pub fn clone_user(&self, pid: &PeerId) -> Result<User> {
        if let Some(u) = self.accounts.get(pid) {
            Ok(User::new(
                u.pid,
                u.name.clone(),
                u.avatar.clone(),
                u.wallet.clone(),
                u.pub_height,
            ))
        } else {
            Err(anyhow!("user missing."))
        }
    }

    pub fn list_accounts(&self) -> &HashMap<PeerId, Account> {
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
        base: &PathBuf,
        secret: &[u8],
    ) -> Result<(i64, PeerId)> {
        let account_index = self.accounts.len() as u32;
        let (mut account, sk) = Account::generate(
            account_index,
            secret,
            lang,
            seed,
            pass,
            name,
            lock,
            avatar_bytes,
        )?;
        let account_id = account.pid;

        if let Some(u) = self.accounts.get(&account_id) {
            return Ok((u.id, account_id));
        }

        account_init(base, &account.plainkey(), &account.pid).await?;

        let account_db = account_db(base, secret)?;
        account.insert(&account_db)?;
        account_db.close()?;
        let account_did = account.id;
        let key = account.plainkey();
        let _ = write_avatar(base, &account_id, &account_id, &account.avatar).await;
        self.accounts.insert(account.pid, account);

        let (device_name, device_info) = device_info();
        let mut device = Device::new(device_name, device_info, Peer::peer(account_id));
        let db = consensus_db(base, &account_id, &self.db_key(&account_id)?)?;
        device.insert(&db)?;
        db.close()?;

        Ok((account_did, account_id))
    }

    pub fn update_account(
        &mut self,
        pid: PeerId,
        name: &str,
        avatar: Vec<u8>,
        base: &PathBuf,
        secret: &[u8],
    ) -> Result<()> {
        let account_db = account_db(base, secret)?;
        let account = self.account_mut(&pid)?;
        account.name = name.to_owned();
        if avatar.len() > 0 {
            account.avatar = avatar;
        }
        account.pub_height = account.pub_height + 1;
        account.update_info(&account_db)?;
        account_db.close()
    }

    pub fn mnemonic(&self, pid: &PeerId, lock: &str, secret: &[u8]) -> Result<String> {
        let account = self.account(pid)?;
        account.mnemonic(secret, lock)
    }

    pub fn pin(
        &mut self,
        pid: &PeerId,
        lock: &str,
        new: &str,
        base: &PathBuf,
        secret: &[u8],
    ) -> Result<()> {
        let account_db = account_db(base, secret)?;
        let account = self.account_mut(pid)?;
        account.pin(secret, lock, new)?;
        account.update(&account_db)?;
        account_db.close()
    }

    pub fn device(&self) -> Result<&Device> {
        if self.distributes.len() > 0 {
            Ok(&self.distributes[0])
        } else {
            Err(anyhow!("no devices"))
        }
    }

    //     pub fn create_message(&self, pid: &PeerId, addr: Peer) -> Result<SendType> {
    //         let user = self.clone_user(pid)?;
    //         let account = self.account(pid)?;
    //         let height = account.own_height;
    //         let event = account.event;
    //         let proof = self.prove_addr(pid, &addr.id)?;
    //         let running = self.running(pid)?;

    //         Ok(SendType::Connect(
    //             0,
    //             addr,
    //             bincode::serialize(&GroupConnect::Create(
    //                 proof,
    //                 user,
    //                 height,
    //                 event,
    //                 running.device_name.clone(),
    //                 running.device_info.clone(),
    //                 running.distributes.keys().cloned().collect(),
    //             ))
    //             .unwrap_or(vec![]),
    //         ))
    //     }

    //     pub fn connect_message(&self, pid: &PeerId, addr: Peer) -> Result<SendType> {
    //         let account = self.account(pid)?;
    //         let height = account.own_height;
    //         let event = account.event;
    //         let data = bincode::serialize(&GroupConnect::Connect(height, event)).unwrap_or(vec![]);
    //         Ok(SendType::Connect(0, addr, data))
    //     }

    //     pub fn connect_result(&self, pid: &PeerId, addr: Peer) -> Result<SendType> {
    //         let account = self.account(pid)?;
    //         let height = account.own_height;
    //         let event = account.event;
    //         let data = bincode::serialize(&GroupConnect::Connect(height, event)).unwrap_or(vec![]);
    //         Ok(SendType::Result(0, addr, true, false, data))
    //     }

    //     pub fn agree_message(&self, pid: &PeerId, addr: Peer) -> Result<SendType> {
    //         let account = self.account(pid)?;
    //         let height = account.own_height;
    //         let event = account.event;
    //         let me = self.clone_user(pid)?;
    //         let proof = self.prove_addr(pid, &addr.id)?;
    //         let running = self.running(pid)?;

    //         Ok(SendType::Result(
    //             0,
    //             addr,
    //             true,
    //             false,
    //             bincode::serialize(&GroupConnect::Create(
    //                 proof,
    //                 me,
    //                 height,
    //                 event,
    //                 running.device_name.clone(),
    //                 running.device_info.clone(),
    //                 running.distributes.keys().cloned().collect(),
    //             ))
    //             .unwrap_or(vec![]),
    //         ))
    //     }

    //     fn ancestor(from: u64, to: u64) -> (Vec<u64>, bool) {
    //         let space = to - from;
    //         let step = space / 8;
    //         if step == 0 {
    //             ((from..to + 1).map(|i| i).collect(), true)
    //         } else {
    //             let mut vec: Vec<u64> = (1..8).map(|i| step * i + from).collect();
    //             vec.push(to);
    //             (vec, false)
    //         }
    //     }

    //     pub fn sync_message(&self, pid: &PeerId, addr: PeerId, from: u64, to: u64) -> Result<SendType> {
    //         let (ancestors, hashes, is_min) = if to >= from {
    //             let (ancestors, is_min) = Self::ancestor(from, to);
    //             let db = self.consensus_db(pid)?;
    //             let hashes = crate::consensus::Event::get_assign_hash(&db, &ancestors)?;
    //             db.close()?;
    //             (ancestors, hashes, is_min)
    //         } else {
    //             (vec![], vec![], true)
    //         };

    //         let event = GroupEvent::SyncCheck(ancestors, hashes, is_min);
    //         let data = bincode::serialize(&event).unwrap_or(vec![]);
    //         Ok(SendType::Event(0, addr, data))
    //     }

    //     pub fn event_message(&self, addr: PeerId, event: &GroupEvent) -> Result<SendType> {
    //         let data = bincode::serialize(event).unwrap_or(vec![]);
    //         Ok(SendType::Event(0, addr, data))
    //     }

    //     pub fn broadcast(
    //         &mut self,
    //         pid: &PeerId,
    //         event: InnerEvent,
    //         path: i64,
    //         row: i64,
    //         results: &mut HandleResult,
    //     ) -> Result<()> {
    //         let db = self.consensus_db(pid)?;
    //         let account_db = self.account_db()?;

    //         let account = self.account_mut(pid)?;
    //         let pre_event = account.event;
    //         let eheight = account.own_height + 1;
    //         let eid = event.generate_event_id();

    //         Event::merge(&db, eid, path, row, eheight)?;
    //         drop(db);

    //         account.update_consensus(&account_db, eheight, eid)?;
    //         account_db.close()?;
    //         drop(account);

    //         let e = GroupEvent::Event(eheight, eid, pre_event, event);
    //         let data = bincode::serialize(&e).unwrap_or(vec![]);
    //         let running = self.running(pid)?;
    //         for (addr, (_peer, _id, online)) in &running.distributes {
    //             if *online {
    //                 let msg = SendType::Event(0, *addr, data.clone());
    //                 results.groups.push((*pid, msg))
    //             }
    //         }
    //         Ok(())
    //     }

    //     pub fn _status(
    //         &mut self,
    //         pid: &PeerId,
    //         event: StatusEvent,
    //         results: &mut HandleResult,
    //     ) -> Result<()> {
    //         let running = self.running(pid)?;
    //         let data = bincode::serialize(&GroupEvent::Status(event)).unwrap_or(vec![]);
    //         for (addr, (_peer, _id, online)) in &running.distributes {
    //             if *online {
    //                 let msg = SendType::Event(0, *addr, data.clone());
    //                 results.groups.push((*pid, msg))
    //             }
    //         }
    //         Ok(())
    //     }
}

// impl GroupEvent {
//     pub async fn handle(
//         group: &mut Group,
//         event: GroupEvent,
//         pid: PeerId,
//         addr: PeerId,
//         //layer: &Arc<RwLock<Layer>>,
//         uid: u64,
//     ) -> Result<HandleResult> {
//         let mut results = HandleResult::new();
//         match event {
//             GroupEvent::DeviceUpdate(_at, _name) => {
//                 // TODO
//             }
//             GroupEvent::DeviceDelete(_at) => {
//                 // TODO
//             }
//             GroupEvent::DeviceOffline => {
//                 let v = group.running_mut(&pid)?;
//                 let did = v.offline(&addr)?;
//                 results.rpcs.push(device_rpc::device_offline(pid, did));
//             }
//             GroupEvent::StatusRequest => {
//                 let (cpu_n, mem_s, swap_s, disk_s, cpu_p, mem_p, swap_p, disk_p) =
//                     local_device_status();
//                 results.groups.push((
//                     pid,
//                     SendType::Event(
//                         0,
//                         addr,
//                         bincode::serialize(&GroupEvent::StatusResponse(
//                             cpu_n,
//                             mem_s,
//                             swap_s,
//                             disk_s,
//                             cpu_p,
//                             mem_p,
//                             swap_p,
//                             disk_p,
//                             group.uptime(&pid)?,
//                         ))
//                         .unwrap_or(vec![]),
//                     ),
//                 ))
//             }
//             GroupEvent::StatusResponse(
//                 cpu_n,
//                 mem_s,
//                 swap_s,
//                 disk_s,
//                 cpu_p,
//                 mem_p,
//                 swap_p,
//                 disk_p,
//                 uptime,
//             ) => results.rpcs.push(device_rpc::device_status(
//                 pid, cpu_n, mem_s, swap_s, disk_s, cpu_p, mem_p, swap_p, disk_p, uptime,
//             )),
//             GroupEvent::Event(eheight, eid, pre) => {
//                 //inner_event.handle(group, pid, addr, eheight, eid, pre, &mut results, layer)?;
//             }
//             GroupEvent::Status => {
//                 //status_event.handle(group, pid, addr, &mut results, layer, uid)?;
//             }
//             GroupEvent::SyncCheck(ancestors, hashes, is_min) => {
//                 println!("sync check: {:?}", ancestors);
//                 let account = group.account(&pid)?;
//                 if ancestors.len() == 0 || hashes.len() == 0 {
//                     return Ok(results);
//                 }

//                 // remote is new need it handle.
//                 if hashes[0] == EventId::default() {
//                     return Ok(results);
//                 }

//                 let remote_height = ancestors.last().map(|v| *v).unwrap_or(0);
//                 let remote_event = hashes.last().map(|v| *v).unwrap_or(EventId::default());
//                 if account.own_height != remote_height || account.event != remote_event {
//                     // check ancestor and merge.
//                     let db = group.consensus_db(&pid)?;
//                     let ours = vec![];
//                     //let ours = crate::consensus::Event::get_assign_hash(&db, &ancestors)?;
//                     drop(db);

//                     if ours.len() == 0 {
//                         let event = GroupEvent::SyncRequest(1, remote_height);
//                         let data = bincode::serialize(&event).unwrap_or(vec![]);
//                         results.groups.push((pid, SendType::Event(0, addr, data)));
//                         return Ok(results);
//                     }

//                     let mut ancestor = 0u64;
//                     for i in 0..ancestors.len() {
//                         if hashes[i] != ours[i] {
//                             if i == 0 {
//                                 ancestor = ancestors[0];
//                                 break;
//                             }

//                             if ancestors[i - 1] == ancestors[i] + 1 {
//                                 ancestor = ancestors[i - 1];
//                             } else {
//                                 if is_min {
//                                     ancestor = ancestors[i - 1];
//                                 } else {
//                                     results.groups.push((
//                                         pid,
//                                         group.sync_message(
//                                             &pid,
//                                             addr,
//                                             ancestors[i - 1],
//                                             ancestors[i],
//                                         )?,
//                                     ));
//                                     return Ok(results);
//                                 }
//                             }

//                             break;
//                         }
//                     }

//                     if ancestor != 0 {
//                         let event = GroupEvent::SyncRequest(ancestor, remote_height);
//                         let data = bincode::serialize(&event).unwrap_or(vec![]);
//                         results.groups.push((pid, SendType::Event(0, addr, data)));
//                     } else {
//                         results.groups.push((
//                             pid,
//                             group.sync_message(&pid, addr, remote_height, account.own_height)?,
//                         ));
//                     }
//                 }
//             }
//             GroupEvent::SyncRequest(from, to) => {
//                 println!("====== DEBUG Sync Request: from: {} to {}", from, to);
//                 // every time sync MAX is 100.
//                 let last_to = if to - from > 100 { to - 100 } else { to };
//                 // let sync_events = SyncEvent::sync(
//                 //     &group,
//                 //     &group.base,
//                 //     &pid,
//                 //     group.account(&pid)?,
//                 //     from,
//                 //     last_to,
//                 // )
//                 // .await?;
//                 let event = GroupEvent::SyncResponse(from, last_to, to);
//                 let data = bincode::serialize(&event).unwrap_or(vec![]);
//                 results.groups.push((pid, SendType::Event(0, addr, data)));
//             }
//             GroupEvent::SyncResponse(from, last_to, to) => {
//                 println!(
//                     "====== DEBUG Sync Response: from: {} last {}, to {}",
//                     from, last_to, to
//                 );
//                 if last_to < to {
//                     let event = GroupEvent::SyncRequest(last_to + 1, to);
//                     let data = bincode::serialize(&event).unwrap_or(vec![]);
//                     results.groups.push((pid, SendType::Event(0, addr, data)));
//                 }
//                 //SyncEvent::handle(pid, from, last_to, events, group, layer, &mut results, addr)?;
//             }
//         }

//         Ok(results)
//     }
// }
