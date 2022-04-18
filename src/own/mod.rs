//use esse_primitives::id_to_str;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::EventId,
    message::{RecvType, SendType},
    primitives::{HandleResult, Peer, PeerId, PeerKey, Result},
};

use crate::account::{Account, User};
use crate::apps::device::rpc as device_rpc;
use crate::apps::device::Device;
use crate::global::Global;
//use crate::consensus::Event;
//use crate::event::{InnerEvent, StatusEvent, SyncEvent};
//use crate::layer::Layer;
//use crate::rpc;
use crate::storage::{account_db, account_init, consensus_db, wallet_db, write_avatar};
//use crate::utils::crypto::{decrypt, encrypt};
use crate::utils::device_status::{device_info, device_status as local_device_status};

/// ESSE own distributed accounts.
pub(crate) struct Own {
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
enum OwnConnect {
    /// Params: User, consensus height, event_id, remote_name, remote_info, other_devices addr.
    Create(User, u64, EventId, String, String, Vec<PeerId>),
    /// connected.
    Connect(u64, EventId),
}

/// Esse group's Event.
#[derive(Serialize, Deserialize)]
pub(crate) enum OwnEvent {
    /// Sync event.
    Event(u64, EventId, EventId),
    //Event(u64, EventId, EventId, InnerEvent),
    /// Sync infomations (name, info).
    Info(String, String),
    /// update device's name.
    DeviceUpdate(PeerId, String),
    /// device deleted.
    DeviceDelete(PeerId),
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
        RecvType::Connect(peer, _) => {
            let pid = global.pid().await;
            let db_key = global.own.read().await.db_key(&pid)?;
            let db = consensus_db(&global.base, &pid, &db_key)?;
            if let Ok(id) = global.own.write().await.online(&peer.id) {
                results.rpcs.push(device_rpc::device_online(id));
            } else {
                let aid = peer.id;
                let mut device = Device::new(peer);
                device.insert(&db)?;
                let (_id, name, info) = global.own.read().await.current_device()?;
                let own_event = OwnEvent::Info(name, info);
                let data = bincode::serialize(&own_event)?;
                let msg = SendType::Event(0, aid, data);
                results.owns.push(msg);
                results.rpcs.push(device_rpc::device_create(&device));
                global.own.write().await.add_device(device);
            };
        }
        RecvType::Leave(peer) => {
            if let Ok(id) = global.own.write().await.offline(&peer.id) {
                results.rpcs.push(device_rpc::device_offline(id));
            }
        }
        RecvType::Event(aid, bytes) => {
            let event: OwnEvent = bincode::deserialize(&bytes)?;
            return OwnEvent::handle(aid, event, global).await;
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            todo!();
            // TODO stream
        }
        _ => {
            warn!("own message nerver here!");
        }
    }

    Ok(results)
}

impl Own {
    pub fn init(accounts: HashMap<PeerId, Account>) -> Own {
        Own {
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

    pub fn online(&mut self, aid: &PeerId) -> Result<i64> {
        for device in self.distributes.iter_mut() {
            if &device.assist == aid {
                device.online = true;
                return Ok(device.id);
            }
        }
        Err(anyhow!("missing distribute device"))
    }

    pub fn offline(&mut self, aid: &PeerId) -> Result<i64> {
        for device in self.distributes.iter_mut() {
            if &device.assist == aid {
                device.online = false;
                return Ok(device.id);
            }
        }
        Err(anyhow!("missing distribute device"))
    }

    pub fn device_id(&self, aid: &PeerId) -> Result<i64> {
        for device in self.distributes.iter() {
            if &device.assist == aid {
                return Ok(device.id);
            }
        }
        Err(anyhow!("missing distribute device"))
    }

    pub fn add_device(&mut self, device: Device) {
        self.distributes.push(device);
    }

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

    /// reset group info when change account.
    pub fn reset(
        &mut self,
        pid: &PeerId,
        lock: &str,
        base: &PathBuf,
        secret: &[u8],
    ) -> Result<(u64, u64)> {
        let (keypair, pheight, oheight) = if let Some(u) = self.accounts.get_mut(pid) {
            let keypair = u.secret(secret, lock)?;
            u.cache_plainkey(secret, lock)?;
            (keypair, u.pub_height, u.own_height)
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
            Ok(User::info(
                u.pub_height,
                u.name.clone(),
                u.wallet.clone(),
                u.cloud.clone(),
                u.cloud_key.clone(),
                u.avatar.clone(),
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
        let (mut account, _sk, mut wallet) = Account::generate(
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
        let _key = account.plainkey();
        let _ = write_avatar(base, &account_id, &account_id, &account.avatar).await;
        self.accounts.insert(account.pid, account);

        let db_key = self.db_key(&account_id)?;
        let wallet_db = wallet_db(base, &account_id, &db_key)?;
        wallet.insert(&wallet_db)?;
        wallet_db.close()?;

        let (device_name, device_info) = device_info();
        let mut device = Device::new(Peer::peer(account_id));
        device.name = device_name;
        device.info = device_info;
        let device_db = consensus_db(base, &account_id, &db_key)?;
        device.insert(&device_db)?;
        device_db.close()?;

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

    pub fn current_device(&self) -> Result<(i64, String, String)> {
        if self.distributes.len() > 0 {
            Ok((
                self.distributes[0].id.clone(),
                self.distributes[0].name.clone(),
                self.distributes[0].info.clone(),
            ))
        } else {
            Err(anyhow!("no devices"))
        }
    }
}

impl OwnEvent {
    pub async fn handle(
        aid: PeerId,
        event: OwnEvent,
        global: &Arc<Global>,
    ) -> Result<HandleResult> {
        let pid = global.pid().await;
        let mut results = HandleResult::new();
        match event {
            OwnEvent::Info(name, info) => {
                let id = global.own.read().await.device_id(&aid)?;
                let db_key = global.own.read().await.db_key(&pid)?;
                let db = consensus_db(&global.base, &pid, &db_key)?;
                Device::update(&db, id, &name, &info)?;
            }
            OwnEvent::DeviceUpdate(_aid, _name) => {
                // TODO
            }
            OwnEvent::DeviceDelete(_aid) => {
                // TODO
            }
            OwnEvent::StatusRequest => {
                let uptime = global.own.read().await.uptime;
                let (cpu_n, mem_s, swap_s, disk_s, cpu_p, mem_p, swap_p, disk_p) =
                    local_device_status();
                let event = OwnEvent::StatusResponse(
                    cpu_n, mem_s, swap_s, disk_s, cpu_p, mem_p, swap_p, disk_p, uptime,
                );
                results
                    .owns
                    .push(SendType::Event(0, aid, bincode::serialize(&event)?))
            }
            OwnEvent::StatusResponse(
                cpu_n,
                mem_s,
                swap_s,
                disk_s,
                cpu_p,
                mem_p,
                swap_p,
                disk_p,
                uptime,
            ) => {
                let id = global.own.read().await.device_id(&aid)?;
                results.rpcs.push(device_rpc::device_status(
                    id, cpu_n, mem_s, swap_s, disk_s, cpu_p, mem_p, swap_p, disk_p, uptime,
                ));
            }
            OwnEvent::Event(_eheight, _eid, _pre) => {
                //inner_event.handle(group, pid, addr, eheight, eid, pre, &mut results, layer)?;
            }
            OwnEvent::SyncCheck(_ancestors, _hashes, _is_min) => {
                // println!("sync check: {:?}", ancestors);
                // let account = group.account(&pid)?;
                // if ancestors.len() == 0 || hashes.len() == 0 {
                //     return Ok(results);
                // }

                // // remote is new need it handle.
                // if hashes[0] == EventId::default() {
                //     return Ok(results);
                // }

                // let remote_height = ancestors.last().map(|v| *v).unwrap_or(0);
                // let remote_event = hashes.last().map(|v| *v).unwrap_or(EventId::default());
                // if account.own_height != remote_height || account.event != remote_event {
                //     // check ancestor and merge.
                //     let db = group.consensus_db(&pid)?;
                //     let ours = vec![];
                //     //let ours = crate::consensus::Event::get_assign_hash(&db, &ancestors)?;
                //     drop(db);

                //     if ours.len() == 0 {
                //         let event = OwnEvent::SyncRequest(1, remote_height);
                //         let data = bincode::serialize(&event).unwrap_or(vec![]);
                //         results.groups.push((pid, SendType::Event(0, addr, data)));
                //         return Ok(results);
                //     }

                //     let mut ancestor = 0u64;
                //     for i in 0..ancestors.len() {
                //         if hashes[i] != ours[i] {
                //             if i == 0 {
                //                 ancestor = ancestors[0];
                //                 break;
                //             }

                //             if ancestors[i - 1] == ancestors[i] + 1 {
                //                 ancestor = ancestors[i - 1];
                //             } else {
                //                 if is_min {
                //                     ancestor = ancestors[i - 1];
                //                 } else {
                //                     results.groups.push((
                //                         pid,
                //                         group.sync_message(
                //                             &pid,
                //                             addr,
                //                             ancestors[i - 1],
                //                             ancestors[i],
                //                         )?,
                //                     ));
                //                     return Ok(results);
                //                 }
                //             }

                //             break;
                //         }
                //     }

                //     if ancestor != 0 {
                //         let event = OwnEvent::SyncRequest(ancestor, remote_height);
                //         let data = bincode::serialize(&event).unwrap_or(vec![]);
                //         results.groups.push((pid, SendType::Event(0, addr, data)));
                //     } else {
                //         results.groups.push((
                //             pid,
                //             group.sync_message(&pid, addr, remote_height, account.own_height)?,
                //         ));
                //     }
                // }
            }
            OwnEvent::SyncRequest(_from, _to) => {
                //println!("====== DEBUG Sync Request: from: {} to {}", from, to);
                // every time sync MAX is 100.
                //let last_to = if to - from > 100 { to - 100 } else { to };
                // let sync_events = SyncEvent::sync(
                //     &group,
                //     &group.base,
                //     &pid,
                //     group.account(&pid)?,
                //     from,
                //     last_to,
                // )
                // .await?;
                // let event = OwnEvent::SyncResponse(from, last_to, to);
                // let data = bincode::serialize(&event).unwrap_or(vec![]);
                // results.groups.push((pid, SendType::Event(0, addr, data)));
            }
            OwnEvent::SyncResponse(_from, _last_to, _to) => {
                // println!(
                //     "====== DEBUG Sync Response: from: {} last {}, to {}",
                //     from, last_to, to
                // );
                // if last_to < to {
                //     let event = OwnEvent::SyncRequest(last_to + 1, to);
                //     let data = bincode::serialize(&event).unwrap_or(vec![]);
                //     results.groups.push((pid, SendType::Event(0, addr, data)));
                // }
                //SyncEvent::handle(pid, from, last_to, events, group, layer, &mut results, addr)?;
            }
        }

        Ok(results)
    }
}
