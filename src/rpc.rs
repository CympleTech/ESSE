use async_lock::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tdn::{
    smol::channel::{SendError, Sender},
    types::{
        group::GroupId,
        message::{NetworkType, SendMessage, SendType, StateRequest, StateResponse},
        primitive::{new_io_error, HandleResult, PeerAddr},
        rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
    },
};
use tdn_did::user::User;

use crate::event::InnerEvent;
use crate::group::{Group, GroupEvent};
use crate::layer::{Layer, LayerEvent};
use crate::migrate::consensus::{FRIEND_TABLE_PATH, MESSAGE_TABLE_PATH, REQUEST_TABLE_PATH};
use crate::models::{
    device::Device,
    session::{Friend, Message, MessageType, Request},
};
use crate::storage::{consensus_db, delete_avatar, session_db};
use crate::utils::device_status::device_status as local_device_status;

#[inline]
pub(crate) fn network_stable(peers: Vec<(PeerAddr, bool)>) -> RpcParam {
    let s_peers: Vec<Vec<String>> = peers
        .iter()
        .map(|(p, is_d)| {
            let d = if *is_d {
                String::from("1")
            } else {
                String::from("0")
            };
            vec![p.to_hex(), d]
        })
        .collect();
    rpc_response(0, "network-stable", json!(s_peers), GroupId::default())
}

#[inline]
pub(crate) fn network_dht(peers: Vec<PeerAddr>) -> RpcParam {
    let s_peers: Vec<String> = peers.iter().map(|p| p.to_hex()).collect();
    rpc_response(0, "network-dht", json!(s_peers), GroupId::default())
}

#[inline]
pub(crate) fn network_seed(peers: Vec<SocketAddr>) -> RpcParam {
    let s_peers: Vec<String> = peers.iter().map(|p| p.to_string()).collect();
    rpc_response(0, "network-seed", json!(s_peers), GroupId::default())
}

#[inline]
pub(crate) fn friend_online(mgid: GroupId, fid: i64, addr: PeerAddr) -> RpcParam {
    rpc_response(0, "friend-online", json!([fid, addr.to_hex()]), mgid)
}

#[inline]
pub(crate) fn friend_offline(mgid: GroupId, fid: i64) -> RpcParam {
    rpc_response(0, "friend-offline", json!([fid]), mgid)
}

#[inline]
pub(crate) fn friend_info(mgid: GroupId, friend: &Friend) -> RpcParam {
    rpc_response(0, "friend-info", json!(friend.to_rpc()), mgid)
}

#[inline]
pub(crate) fn friend_update(mgid: GroupId, fid: i64, is_top: bool, remark: &str) -> RpcParam {
    rpc_response(0, "friend-update", json!([fid, is_top, remark]), mgid)
}

#[inline]
pub(crate) fn friend_close(mgid: GroupId, fid: i64) -> RpcParam {
    rpc_response(0, "friend-close", json!([fid]), mgid)
}

#[inline]
pub(crate) fn friend_delete(mgid: GroupId, fid: i64) -> RpcParam {
    rpc_response(0, "friend-delete", json!([fid]), mgid)
}

#[inline]
pub(crate) fn request_create(mgid: GroupId, req: &Request) -> RpcParam {
    rpc_response(0, "request-create", json!(req.to_rpc()), mgid)
}

#[inline]
pub(crate) fn request_delivery(mgid: GroupId, id: i64, is_d: bool) -> RpcParam {
    rpc_response(0, "request-delivery", json!([id, is_d]), mgid)
}

#[inline]
pub(crate) fn request_agree(mgid: GroupId, id: i64, friend: &Friend) -> RpcParam {
    rpc_response(0, "request-agree", json!([id, friend.to_rpc()]), mgid)
}

#[inline]
pub(crate) fn request_reject(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "request-reject", json!([id]), mgid)
}

#[inline]
pub(crate) fn request_delete(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "request-delete", json!([id]), mgid)
}

#[inline]
pub(crate) fn message_create(mgid: GroupId, msg: &Message) -> RpcParam {
    rpc_response(0, "message-create", json!(msg.to_rpc()), mgid)
}

#[inline]
pub(crate) fn message_delivery(mgid: GroupId, id: i64, is_d: bool) -> RpcParam {
    rpc_response(0, "message-delivery", json!([id, is_d]), mgid)
}

#[inline]
pub(crate) fn message_delete(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "message-delete", json!([id]), mgid)
}

#[inline]
pub(crate) fn device_create(mgid: GroupId, device: &Device) -> RpcParam {
    rpc_response(0, "device-create", json!(device.to_rpc()), mgid)
}

#[inline]
pub(crate) fn _device_remove(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "device-remove", json!([id]), mgid)
}

#[inline]
pub(crate) fn device_online(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "device-online", json!([id]), mgid)
}

#[inline]
pub(crate) fn device_offline(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "device-offline", json!([id]), mgid)
}

#[inline]
pub(crate) fn account_update(mgid: GroupId, name: &str, avatar: String) -> RpcParam {
    rpc_response(0, "account-update", json!([name, avatar]), mgid)
}

#[inline]
pub(crate) fn device_status(
    mgid: GroupId,
    cpu: u32,
    memory: u32,
    swap: u32,
    disk: u32,
    cpu_p: u16,
    memory_p: u16,
    swap_p: u16,
    disk_p: u16,
    uptime: u32,
) -> RpcParam {
    rpc_response(
        0,
        "device-status",
        json!([cpu, memory, swap, disk, cpu_p, memory_p, swap_p, disk_p, uptime]),
        mgid,
    )
}

#[inline]
fn friend_list(friends: Vec<Friend>) -> RpcParam {
    let mut results = vec![];
    for friend in friends {
        results.push(friend.to_rpc());
    }

    json!(results)
}

#[inline]
fn request_list(requests: Vec<Request>) -> RpcParam {
    let mut results = vec![];
    for request in requests {
        results.push(request.to_rpc());
    }
    json!(results)
}

#[inline]
fn message_list(messages: Vec<Message>) -> RpcParam {
    let mut results = vec![];
    for msg in messages {
        results.push(msg.to_rpc());
    }
    json!(results)
}

#[inline]
fn device_list(devices: Vec<Device>) -> RpcParam {
    let mut results = vec![];
    for device in devices {
        results.push(device.to_rpc());
    }
    json!(results)
}

#[inline]
pub(crate) async fn sleep_waiting_close_stable(
    sender: Sender<SendMessage>,
    groups: HashMap<PeerAddr, ()>,
    layers: HashMap<PeerAddr, GroupId>,
) -> std::result::Result<(), SendError<SendMessage>> {
    tdn::smol::Timer::after(std::time::Duration::from_secs(10)).await;
    for (addr, _) in groups {
        sender
            .send(SendMessage::Group(
                Default::default(),
                SendType::Disconnect(addr),
            ))
            .await?;
    }

    for (faddr, fgid) in layers {
        sender
            .send(SendMessage::Layer(
                Default::default(),
                fgid,
                SendType::Disconnect(faddr),
            ))
            .await?;
    }

    Ok(())
}

#[inline]
pub(crate) async fn inner_rpc(
    uid: u64,
    method: &str,
    sender: &async_channel::Sender<SendMessage>,
) -> Result<(), std::io::Error> {
    // Inner network default rpc method. only use in http-rpc.
    if method == "network-stable" || method == "network-dht" || method == "network-seed" {
        let req = match method {
            "network-stable" => StateRequest::Stable,
            "network-dht" => StateRequest::DHT,
            "network-seed" => StateRequest::Seed,
            _ => return Ok(()),
        };

        let (s, r) = async_channel::unbounded::<StateResponse>();
        let _ = sender
            .send(SendMessage::Network(NetworkType::NetworkState(req, s)))
            .await
            .expect("TDN channel closed");

        let param = match r.recv().await {
            Ok(StateResponse::Stable(peers)) => network_stable(peers),
            Ok(StateResponse::DHT(peers)) => network_dht(peers),
            Ok(StateResponse::Seed(seeds)) => network_seed(seeds),
            Err(_) => {
                return Ok(());
            }
        };

        sender
            .send(SendMessage::Rpc(uid, param, false))
            .await
            .expect("TDN channel closed");

        return Ok(());
    }

    Err(new_io_error("not found"))
}

pub(crate) struct RpcState {
    group: Arc<RwLock<Group>>,
    layer: Arc<RwLock<Layer>>,
}

#[inline]
pub(crate) fn new_rpc_handler(
    addr: PeerAddr,
    group: Arc<RwLock<Group>>,
    layer: Arc<RwLock<Layer>>,
) -> RpcHandler<RpcState> {
    let mut rpc_handler = RpcHandler::new(RpcState { group, layer });

    rpc_handler.add_method("echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    rpc_handler.add_method("system-info", move |_, _, _| async move {
        Ok(HandleResult::rpc(json!(vec![addr.to_hex()])))
    });

    rpc_handler.add_method(
        "add-bootstrap",
        |_gid, params: Vec<RpcParam>, _| async move {
            let socket = params[0].as_str()?;
            if let Ok(addr) = socket.parse::<SocketAddr>() {
                Ok(HandleResult::network(NetworkType::Connect(addr)))
            } else {
                Err(RpcError::InvalidRequest)
            }
        },
    );

    rpc_handler.add_method(
        "account-list",
        |_gid, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let mut users: Vec<Vec<String>> = vec![];
            let group_lock = state.group.read().await;
            for (gid, user) in group_lock.list_users().iter() {
                users.push(vec![
                    gid.to_hex(),
                    user.name.clone(),
                    user.lock.clone(),
                    base64::encode(&user.avatar),
                ]);
            }
            drop(group_lock);

            Ok(HandleResult::rpc(json!(users)))
        },
    );

    rpc_handler.add_method(
        "account-create",
        |_gid, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let name = params[0].as_str()?;
            let lock = params[1].as_str()?;
            let seed = params[2].as_str()?;
            let avatar = params[3].as_str()?;
            let device_name = params[4].as_str()?;
            let device_info = params[5].as_str()?;
            let avatar_bytes = base64::decode(avatar).unwrap_or(vec![]);

            let gid = state
                .group
                .write()
                .await
                .add_account(name, seed, lock, avatar_bytes, device_name, device_info)
                .await?;
            state.layer.write().await.add_running(&gid)?;

            let mut results = HandleResult::rpc(json!(vec![gid.to_hex()]));
            results.networks.push(NetworkType::AddGroup(gid)); // add AddGroup to TDN.

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "account-restore",
        |_gid, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let name = params[0].as_str()?;
            let lock = params[1].as_str()?;
            let seed = params[2].as_str()?;
            let some_addr = PeerAddr::from_hex(params[3].as_str()?).ok();
            let device_name = params[4].as_str()?;
            let device_info = params[5].as_str()?;

            let gid = state
                .group
                .write()
                .await
                .add_account(name, seed, lock, vec![], device_name, device_info)
                .await?;
            state.layer.write().await.add_running(&gid)?;

            let mut results = HandleResult::rpc(json!(vec![gid.to_hex()]));
            results.networks.push(NetworkType::AddGroup(gid)); // add AddGroup to TDN.

            if let Some(addr) = some_addr {
                let group_lock = state.group.read().await;
                let sender = group_lock.sender();
                let msg = group_lock.create_message(&gid, addr)?;
                drop(group_lock);
                tdn::smol::spawn(async move {
                    tdn::smol::Timer::after(std::time::Duration::from_secs(2)).await;
                    let _ = sender.send(SendMessage::Group(gid, msg)).await;
                })
                .detach();
            }

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "device-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let db = consensus_db(state.layer.read().await.base(), &gid)?;
            let devices = Device::all(&db)?;
            drop(db);
            let online_devices = state.group.read().await.online_devices(&gid, devices);
            Ok(HandleResult::rpc(device_list(online_devices)))
        },
    );

    rpc_handler.add_method(
        "device-status",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let addr = PeerAddr::from_hex(params[0].as_str()?)
                .map_err(|_e| new_io_error("PeerAddr invalid!"))?;

            let group_lock = state.group.read().await;
            if &addr == group_lock.addr() {
                let uptime = group_lock.uptime(&gid)?;
                let (cpu, memory, swap, disk, cpu_p, memory_p, swap_p, disk_p) =
                    local_device_status();
                return Ok(HandleResult::rpc(json!([
                    cpu, memory, swap, disk, cpu_p, memory_p, swap_p, disk_p, uptime
                ])));
            }
            drop(group_lock);

            let msg = state
                .group
                .write()
                .await
                .event_message(addr, &GroupEvent::StatusRequest)?;

            Ok(HandleResult::group(gid, msg))
        },
    );

    rpc_handler.add_method(
        "device-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let addr = PeerAddr::from_hex(params[0].as_str()?)
                .map_err(|_e| new_io_error("PeerAddr invalid!"))?;

            let msg = state.group.read().await.create_message(&gid, addr)?;
            Ok(HandleResult::group(gid, msg))
        },
    );

    rpc_handler.add_method(
        "device-connect",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let addr = PeerAddr::from_hex(params[0].as_str()?)
                .map_err(|_e| new_io_error("PeerAddr invalid!"))?;

            let msg = state.group.read().await.connect_message(&gid, addr)?;
            Ok(HandleResult::group(gid, msg))
        },
    );

    rpc_handler.add_method(
        "device-delete",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _id = params[0].as_i64()?;
            // TODO delete a device.
            Ok(HandleResult::new())
        },
    );

    rpc_handler.add_method(
        "account-update",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let name = params[0].as_str()?;
            let avatar = params[1].as_str()?;

            let avatar_bytes = base64::decode(avatar).unwrap_or(vec![]);

            let mut group_lock = state.group.write().await;
            group_lock.update_account(gid, name, avatar_bytes.clone())?;

            let mut results = HandleResult::new();
            group_lock.broadcast(
                &gid,
                InnerEvent::UserInfo(name.to_owned(), avatar_bytes),
                0,
                0,
                &mut results,
            )?;
            drop(group_lock);

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "account-pin",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let old = params[0].as_str()?;
            let new = params[1].as_str()?;
            let result = HandleResult::rpc(json!([new]));
            state.group.write().await.pin(&gid, old, new)?;
            Ok(result)
        },
    );

    rpc_handler.add_method(
        "account-mnemonic",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let lock = params[0].as_str()?;

            let mnemonic = state.group.read().await.mnemonic(&gid, lock)?;
            Ok(HandleResult::rpc(json!([mnemonic])))
        },
    );

    rpc_handler.add_method(
        "account-login",
        |_gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gid = GroupId::from_hex(params[0].as_str()?)?;
            let me_lock = params[1].as_str()?;

            state.group.write().await.add_running(&gid, me_lock)?;
            state.layer.write().await.add_running(&gid)?;

            let mut results = HandleResult::rpc(json!([gid.to_hex()]));

            debug!("Account Logined: {}.", gid.to_hex());
            // add AddGroup to TDN.
            results.networks.push(NetworkType::AddGroup(gid));

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "account-logout",
        |_gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let mut results = HandleResult::new();

            let group_lock = state.group.read().await;
            let layer_lock = state.layer.read().await;
            let keys = group_lock.list_running_user();

            for gid in keys {
                for (fgid, addr) in layer_lock.running(&gid)?.onlines() {
                    // send a event that is offline.
                    let data = postcard::to_allocvec(&LayerEvent::Offline).unwrap_or(vec![]);
                    let msg = SendType::Event(0, *addr, data);
                    results.layers.push((gid, *fgid, msg));
                }

                debug!("Account Offline: {}.", gid.to_hex());
                // add Remove Group to TDN.
                results.networks.push(NetworkType::DelGroup(gid));
            }
            drop(group_lock);
            drop(layer_lock);

            let mut layer_lock = state.layer.write().await;
            let layers = layer_lock.remove_all_running();
            drop(layer_lock);

            let mut group_lock = state.group.write().await;
            let groups = group_lock.remove_all_running();
            let sender = group_lock.sender();
            drop(group_lock);
            tdn::smol::spawn(sleep_waiting_close_stable(sender, groups, layers)).detach();

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "account-online",
        |_gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gid = GroupId::from_hex(params[0].as_str()?)?;

            let mut results = HandleResult::new();

            let layer_lock = state.layer.read().await;
            let friends = layer_lock.all_friends(&gid)?;
            for friend in friends {
                let msg = layer_lock.conn_req_message(&gid, friend.addr).await?;
                results.layers.push((gid, friend.gid, msg));
            }
            drop(layer_lock);

            let devices = state.group.read().await.distribute_conns(&gid);
            for device in devices {
                results.groups.push((gid, device));
            }

            debug!("Account Online: {}.", gid.to_hex());

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "account-offline",
        |_gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gid = GroupId::from_hex(params[0].as_str()?)?;

            let mut results = HandleResult::new();
            let layer_lock = state.layer.read().await;
            for (fgid, addr) in layer_lock.running(&gid)?.onlines() {
                // send a event that is offline.
                let data = postcard::to_allocvec(&LayerEvent::Offline).unwrap_or(vec![]);
                let msg = SendType::Event(0, *addr, data);
                results.layers.push((gid, *fgid, msg));
            }
            drop(layer_lock);

            let layers = state.layer.write().await.remove_running(&gid);
            let mut group_lock = state.group.write().await;
            let groups = group_lock.remove_running(&gid);
            let sender = group_lock.sender();
            drop(group_lock);
            tdn::smol::spawn(sleep_waiting_close_stable(sender, groups, layers)).detach();

            debug!("Account Offline: {}.", gid.to_hex());
            // add Remove Group to TDN.
            results.networks.push(NetworkType::DelGroup(gid));

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "friend-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let friends = state.layer.read().await.all_friends_with_online(&gid)?;
            Ok(HandleResult::rpc(friend_list(friends)))
        },
    );

    rpc_handler.add_method(
        "friend-update",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;
            let remark = params[1].as_str()?;
            let is_top = params[2].as_bool()?;

            let mut results = HandleResult::new();
            let db = session_db(state.layer.read().await.base(), &gid)?;
            let f = if let Some(mut f) = Friend::get_id(&db, id)? {
                f.is_top = is_top;
                f.remark = remark.to_owned();
                f.me_update(&db)?;
                f
            } else {
                return Ok(results);
            };
            drop(db);
            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionFriendUpdate(f.gid, f.is_top, f.remark),
                FRIEND_TABLE_PATH,
                f.id,
                &mut results,
            )?;
            Ok(results)
        },
    );

    rpc_handler.add_method(
        "friend-readed",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let fid = params[0].as_i64()?;

            let db = session_db(state.layer.read().await.base(), &gid)?;
            Friend::readed(&db, fid)?;
            drop(db);

            Ok(HandleResult::new())
        },
    );

    rpc_handler.add_method(
        "friend-close",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;

            let mut results = HandleResult::new();
            let mut layer_lock = state.layer.write().await;

            let db = session_db(layer_lock.base(), &gid)?;
            let friend = Friend::get_id(&db, id)??;
            friend.close(&db)?;
            drop(db);

            let online = layer_lock.remove_friend(&gid, &friend.gid);
            drop(layer_lock);

            if let Some(faddr) = online {
                let mut addrs: HashMap<PeerAddr, GroupId> = HashMap::new();
                addrs.insert(faddr, friend.gid);
                let sender = state.group.read().await.sender();
                tdn::smol::spawn(sleep_waiting_close_stable(sender, HashMap::new(), addrs))
                    .detach();
            }

            let data = postcard::to_allocvec(&LayerEvent::Close).unwrap_or(vec![]);
            let msg = SendType::Event(0, friend.addr, data);
            results.layers.push((gid, friend.gid, msg));

            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionFriendClose(friend.gid),
                FRIEND_TABLE_PATH,
                friend.id,
                &mut results,
            )?;

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "friend-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;

            let mut results = HandleResult::new();
            let mut layer_lock = state.layer.write().await;

            let db = session_db(layer_lock.base(), &gid)?;
            let friend = Friend::get_id(&db, id)??;
            friend.delete(&db)?;
            drop(db);

            let online = layer_lock.remove_friend(&gid, &friend.gid);
            delete_avatar(layer_lock.base(), &gid, &friend.gid).await?;
            drop(layer_lock);

            if let Some(faddr) = online {
                let mut addrs: HashMap<PeerAddr, GroupId> = HashMap::new();
                addrs.insert(faddr, friend.gid);
                let sender = state.group.read().await.sender();
                tdn::smol::spawn(sleep_waiting_close_stable(sender, HashMap::new(), addrs))
                    .detach();
            }

            let data = postcard::to_allocvec(&LayerEvent::Close).unwrap_or(vec![]);
            let msg = SendType::Event(0, friend.addr, data);
            results.layers.push((gid, friend.gid, msg));

            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionFriendDelete(friend.gid),
                FRIEND_TABLE_PATH,
                friend.id,
                &mut results,
            )?;

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "request-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let layer_lock = state.layer.read().await;
            let db = session_db(layer_lock.base(), &gid)?;
            drop(layer_lock);
            let requests = Request::all(&db)?;
            drop(db);
            Ok(HandleResult::rpc(request_list(requests)))
        },
    );

    rpc_handler.add_method(
        "request-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let remote_gid = GroupId::from_hex(params[0].as_str()?)?;
            let remote_addr = PeerAddr::from_hex(params[1].as_str()?)?;
            let remote_name = params[2].as_str()?.to_string();
            let remark = params[3].as_str()?.to_string();

            let mut request = Request::new(
                remote_gid,
                remote_addr,
                remote_name.clone(),
                remark.clone(),
                true,
                false,
            );

            let mut results = HandleResult::rpc(Default::default());
            let me = state.group.read().await.clone_user(&gid)?;

            let mut layer_lock = state.layer.write().await;
            let db = session_db(layer_lock.base(), &gid)?;
            if Friend::is_friend(&db, &request.gid)? {
                debug!("had friend.");
                drop(layer_lock);
                return Ok(results);
            }

            if let Some(req) = Request::get(&db, &request.gid)? {
                println!("Had this request.");
                req.delete(&db)?;
            }
            request.insert(&db)?;
            drop(db);

            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionRequestCreate(
                    true,
                    User::new(remote_gid, remote_addr, remote_name, vec![])?,
                    remark,
                ),
                REQUEST_TABLE_PATH,
                request.id,
                &mut results,
            )?;

            results
                .layers
                .push((gid, remote_gid, layer_lock.req_message(me, request)));

            drop(layer_lock);

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "request-agree",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;

            let mut group_lock = state.group.write().await;
            let me = group_lock.clone_user(&gid)?;
            let mut layer_lock = state.layer.write().await;
            let db = session_db(layer_lock.base(), &gid)?;
            let mut results = HandleResult::new();

            if let Some(mut request) = Request::get_id(&db, id)? {
                group_lock.broadcast(
                    &gid,
                    InnerEvent::SessionRequestHandle(request.gid, true, vec![]),
                    REQUEST_TABLE_PATH,
                    request.id,
                    &mut results,
                )?;
                request.is_ok = true;
                request.is_over = true;
                request.update(&db)?;

                let f = Friend::from_request(&db, request)?;
                layer_lock.running_mut(&gid)?.add_permissioned(f.gid, f.id);
                results.rpcs.push(json!([id, f.to_rpc()]));

                let proof = group_lock.prove_addr(&gid, &f.addr)?;
                let msg = layer_lock.rpc_agree_message(id, proof, me, &gid, f.addr)?;
                results.layers.push((gid, f.gid, msg));
            }
            db.close()?;
            drop(group_lock);
            drop(layer_lock);
            Ok(results)
        },
    );

    rpc_handler.add_method(
        "request-reject",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;

            let mut layer_lock = state.layer.write().await;
            let db = session_db(layer_lock.base(), &gid)?;
            let mut req = Request::get_id(&db, id)??;
            req.is_ok = false;
            req.is_over = true;
            req.update(&db)?;
            drop(db);
            let msg = layer_lock.reject_message(id, req.addr, gid);
            drop(layer_lock);

            let mut results = HandleResult::layer(gid, req.gid, msg);
            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionRequestHandle(req.gid, false, vec![]),
                REQUEST_TABLE_PATH,
                req.id,
                &mut results,
            )?;
            Ok(results)
        },
    );

    rpc_handler.add_method(
        "request-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;

            let layer_lock = state.layer.read().await;
            let db = session_db(layer_lock.base(), &gid)?;
            let base = layer_lock.base().clone();
            drop(layer_lock);
            let req = Request::get_id(&db, id)??;
            req.delete(&db)?;

            // delete avatar. check had friend.
            if Friend::get(&db, &req.gid)?.is_none() {
                delete_avatar(&base, &gid, &req.gid).await?;
            }
            drop(db);

            let mut results = HandleResult::new();
            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionRequestDelete(req.gid),
                REQUEST_TABLE_PATH,
                req.id,
                &mut results,
            )?;
            Ok(results)
        },
    );

    rpc_handler.add_method(
        "message-list",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let fid = params[0].as_i64()?;

            let layer_lock = state.layer.read().await;
            let db = session_db(layer_lock.base(), &gid)?;
            drop(layer_lock);

            Friend::readed(&db, fid)?;
            let messages = Message::get(&db, &fid)?;
            drop(db);
            Ok(HandleResult::rpc(message_list(messages)))
        },
    );

    rpc_handler.add_method(
        "message-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let fid = params[0].as_i64()?;
            let fgid = GroupId::from_hex(params[1].as_str()?)?;
            let m_type = MessageType::from_int(params[2].as_i64()?);
            let content = params[3].as_str()?.to_string();

            let mut layer_lock = state.layer.write().await;
            let base = layer_lock.base();
            let faddr = layer_lock.running(&gid)?.online(&fgid)?;

            let (msg, nw) = LayerEvent::from_message(base, gid, fid, m_type, content).await?;
            let event = LayerEvent::Message(msg.hash, nw);
            let s = layer_lock.event_message(msg.id, gid, faddr, &event);
            drop(layer_lock);

            let mut results = HandleResult::rpc(json!(msg.to_rpc()));
            results.layers.push((gid, fgid, s));

            match event {
                LayerEvent::Message(hash, nw) => {
                    state.group.write().await.broadcast(
                        &gid,
                        InnerEvent::SessionMessageCreate(fgid, true, hash, nw),
                        MESSAGE_TABLE_PATH,
                        msg.id,
                        &mut results,
                    )?;
                }
                _ => {}
            }

            Ok(results)
        },
    );

    rpc_handler.add_method(
        "message-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;

            let layer_lock = state.layer.read().await;
            let db = session_db(&layer_lock.base(), &gid)?;
            drop(layer_lock);

            let msg = Message::get_id(&db, id)??;
            msg.delete(&db)?;
            drop(db);
            let mut results = HandleResult::new();
            state.group.write().await.broadcast(
                &gid,
                InnerEvent::SessionMessageDelete(msg.hash),
                MESSAGE_TABLE_PATH,
                msg.id,
                &mut results,
            )?;
            Ok(results)
        },
    );

    rpc_handler.add_method(
        "files-folder",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _path = params[0].as_str()?;

            Ok(HandleResult::new())
        },
    );

    rpc_handler
}
