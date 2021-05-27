use async_lock::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tdn::{
    smol::channel::{SendError, Sender},
    types::{
        group::GroupId,
        message::{NetworkType, SendMessage, SendType, StateRequest, StateResponse},
        primitive::{new_io_error, HandleResult, PeerAddr, Result},
        rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
    },
};

use crate::apps::app_rpc_inject;
use crate::apps::chat::chat_conn;
use crate::apps::group_chat::{add_layer, group_chat_conn};
use crate::event::InnerEvent;
use crate::group::Group;
use crate::layer::{Layer, LayerEvent};
use crate::session::{Session, SessionType};
use crate::storage::session_db;

pub(crate) fn init_rpc(
    addr: PeerAddr,
    group: Arc<RwLock<Group>>,
    layer: Arc<RwLock<Layer>>,
) -> RpcHandler<RpcState> {
    let mut handler = new_rpc_handler(addr, group, layer);
    app_rpc_inject(&mut handler);
    handler
}

pub(crate) struct RpcState {
    pub group: Arc<RwLock<Group>>,
    pub layer: Arc<RwLock<Layer>>,
}

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
pub(crate) fn account_update(mgid: GroupId, name: &str, avatar: String) -> RpcParam {
    rpc_response(
        0,
        "account-update",
        json!([mgid.to_hex(), name, avatar]),
        mgid,
    )
}

#[inline]
pub(crate) fn session_create(mgid: GroupId, session: &Session) -> RpcParam {
    rpc_response(0, "session-create", session.to_rpc(), mgid)
}

#[inline]
pub(crate) fn session_last(
    mgid: GroupId,
    id: &i64,
    time: &i64,
    content: &str,
    readed: bool,
) -> RpcParam {
    rpc_response(0, "session-last", json!([id, time, content, readed]), mgid)
}

#[inline]
pub(crate) fn _session_update(
    mgid: GroupId,
    id: &i64,
    addr: &PeerAddr,
    name: &str,
    is_top: bool,
) -> RpcParam {
    rpc_response(
        0,
        "session-update",
        json!([id, addr.to_hex(), name, is_top]),
        mgid,
    )
}

#[inline]
pub(crate) fn session_connect(mgid: GroupId, id: &i64, addr: &PeerAddr) -> RpcParam {
    rpc_response(0, "session-connect", json!([id, addr.to_hex()]), mgid)
}

#[inline]
pub(crate) fn session_suspend(mgid: GroupId, id: &i64) -> RpcParam {
    rpc_response(0, "session-suspend", json!([id]), mgid)
}

#[inline]
pub(crate) fn session_lost(mgid: GroupId, id: &i64) -> RpcParam {
    rpc_response(0, "session-lost", json!([id]), mgid)
}

#[inline]
fn session_list(sessions: Vec<Session>) -> RpcParam {
    let mut results = vec![];
    for session in sessions {
        results.push(session.to_rpc());
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
) -> Result<()> {
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

fn new_rpc_handler(
    addr: PeerAddr,
    group: Arc<RwLock<Group>>,
    layer: Arc<RwLock<Layer>>,
) -> RpcHandler<RpcState> {
    let mut handler = RpcHandler::new(RpcState { group, layer });

    handler.add_method("echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method("account-system-info", move |_, _, _| async move {
        Ok(HandleResult::rpc(json!(vec![addr.to_hex()])))
    });

    handler.add_method(
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

    handler.add_method(
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

    handler.add_method(
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

    handler.add_method(
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

    handler.add_method(
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

    handler.add_method(
        "account-pin",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let old = params[0].as_str()?;
            let new = params[1].as_str()?;
            let result = HandleResult::rpc(json!([new]));
            state.group.write().await.pin(&gid, old, new)?;
            Ok(result)
        },
    );

    handler.add_method(
        "account-mnemonic",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let lock = params[0].as_str()?;

            let mnemonic = state.group.read().await.mnemonic(&gid, lock)?;
            Ok(HandleResult::rpc(json!([mnemonic])))
        },
    );

    handler.add_method(
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

    handler.add_method(
        "account-logout",
        |_gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let mut results = HandleResult::new();

            let group_lock = state.group.read().await;
            let layer_lock = state.layer.read().await;
            let keys = group_lock.list_running_user();

            for gid in keys {
                for (fgid, addr) in layer_lock.running(&gid)?.onlines() {
                    // send a event that is offline.
                    let data = postcard::to_allocvec(&LayerEvent::Offline(*fgid)).unwrap_or(vec![]);
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

    handler.add_method(
        "account-online",
        |_gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gid = GroupId::from_hex(params[0].as_str()?)?;

            let mut results = HandleResult::new();

            let group_lock = state.group.read().await;
            let devices = group_lock.distribute_conns(&gid);
            for device in devices {
                results.groups.push((gid, device));
            }

            drop(group_lock);
            debug!("Account Online: {}.", gid.to_hex());

            Ok(results)
        },
    );

    handler.add_method(
        "account-offline",
        |_gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gid = GroupId::from_hex(params[0].as_str()?)?;

            let mut results = HandleResult::new();
            let layer_lock = state.layer.read().await;
            for (fgid, addr) in layer_lock.running(&gid)?.onlines() {
                // send a event that is offline.
                let data = postcard::to_allocvec(&LayerEvent::Offline(*fgid)).unwrap_or(vec![]);
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

    handler.add_method(
        "session-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let db = session_db(state.layer.read().await.base(), &gid)?;
            Ok(HandleResult::rpc(session_list(Session::list(&db)?)))
        },
    );

    handler.add_method(
        "session-connect",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;
            let remote = GroupId::from_hex(params[1].as_str()?)?;

            let mut layer_lock = state.layer.write().await;
            let online = layer_lock.running_mut(&gid)?.active(&remote, true);
            drop(layer_lock);
            if let Some(addr) = online {
                return Ok(HandleResult::rpc(json!([id, addr.to_hex()])));
            }

            let group_lock = state.group.read().await;
            let db = session_db(group_lock.base(), &gid)?;
            let s = Session::get(&db, &id)?;
            drop(db);

            let mut results = HandleResult::new();
            match s.s_type {
                SessionType::Chat => {
                    let proof = group_lock.prove_addr(&gid, &s.addr)?;
                    results.layers.push((gid, s.gid, chat_conn(proof, s.addr)));
                }
                SessionType::Group => {
                    let proof = group_lock.prove_addr(&gid, &s.addr)?;
                    add_layer(&mut results, gid, group_chat_conn(proof, s.addr, s.gid));
                }
                _ => {}
            }
            Ok(results)
        },
    );

    handler.add_method(
        "session-suspend",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;
            let remote = GroupId::from_hex(params[1].as_str()?)?;

            let mut layer_lock = state.layer.write().await;
            let suspend = layer_lock.running_mut(&gid)?.suspend(&remote, true)?;
            drop(layer_lock);

            let mut results = HandleResult::new();
            if suspend {
                results.rpcs.push(json!([id]))
            }

            // let group_lock = state.group.read().await;
            // let db = session_db(group_lock.base(), &gid)?;
            // let s = Session::get(&db, &id)?;
            // drop(db);

            // match s.s_type {
            //     SessionType::Chat => {
            //         let proof = group_lock.prove_addr(&gid, &s.addr)?;
            //         results.layers.push((gid, s.gid, chat_conn(proof, s.addr)));
            //     }
            //     SessionType::Group => {
            //         let proof = group_lock.prove_addr(&gid, &s.addr)?;
            //         add_layer(&mut results, gid, group_chat_conn(proof, s.addr, s.gid));
            //     }
            //     _ => {}
            // }

            Ok(results)
        },
    );

    handler
}
