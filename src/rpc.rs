use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::{NetworkType, SendMessage, SendType, StateRequest, StateResponse},
    primitive::{HandleResult, PeerAddr, Result},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};
use tokio::sync::{
    mpsc::{self, error::SendError, Sender},
    RwLock,
};

use crate::apps::app_rpc_inject;
use crate::apps::chat::chat_conn;
use crate::apps::group_chat::{add_layer, group_chat_conn, GroupChat};
use crate::event::InnerEvent;
use crate::group::Group;
use crate::layer::{Layer, LayerEvent};
use crate::session::{Session, SessionType};
use crate::storage::{group_chat_db, session_db};

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
pub(crate) fn session_delete(mgid: GroupId, id: &i64) -> RpcParam {
    rpc_response(0, "session-delete", json!([id]), mgid)
}

#[inline]
pub(crate) fn session_close(mgid: GroupId, id: &i64) -> RpcParam {
    rpc_response(0, "session-close", json!([id]), mgid)
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
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
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
pub(crate) async fn inner_rpc(uid: u64, method: &str, sender: &Sender<SendMessage>) -> Result<()> {
    // Inner network default rpc method. only use in http-rpc.
    if method == "network-stable" || method == "network-dht" {
        let req = match method {
            "network-stable" => StateRequest::Stable,
            "network-dht" => StateRequest::DHT,
            _ => return Ok(()),
        };

        let (s, mut r) = mpsc::channel::<StateResponse>(128);
        let _ = sender
            .send(SendMessage::Network(NetworkType::NetworkState(req, s)))
            .await
            .expect("TDN channel closed");

        let param = match r.recv().await {
            Some(StateResponse::Stable(peers)) => network_stable(peers),
            Some(StateResponse::DHT(peers)) => network_dht(peers),
            Some(_) | None => {
                return Ok(());
            }
        };

        sender
            .send(SendMessage::Rpc(uid, param, false))
            .await
            .expect("TDN channel closed");

        return Ok(());
    }

    Err(anyhow!("not found"))
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
            let socket = params[0].as_str().ok_or(RpcError::ParseError)?;
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
            let name = params[0].as_str().ok_or(RpcError::ParseError)?;
            let lock = params[1].as_str().ok_or(RpcError::ParseError)?;
            let seed = params[2].as_str().ok_or(RpcError::ParseError)?;
            let avatar = params[3].as_str().ok_or(RpcError::ParseError)?;
            let device_name = params[4].as_str().ok_or(RpcError::ParseError)?;
            let device_info = params[5].as_str().ok_or(RpcError::ParseError)?;
            let avatar_bytes = base64::decode(avatar).unwrap_or(vec![]);

            let (id, gid) = state
                .group
                .write()
                .await
                .add_account(name, seed, lock, avatar_bytes, device_name, device_info)
                .await?;
            state.layer.write().await.add_running(&gid, gid, id, 0)?;

            let mut results = HandleResult::rpc(json!(vec![gid.to_hex()]));
            results.networks.push(NetworkType::AddGroup(gid)); // add AddGroup to TDN.

            debug!("Account Logined: {}.", gid.to_hex());

            Ok(results)
        },
    );

    handler.add_method(
        "account-restore",
        |_gid, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?;
            let lock = params[1].as_str().ok_or(RpcError::ParseError)?;
            let seed = params[2].as_str().ok_or(RpcError::ParseError)?;
            let some_addr =
                PeerAddr::from_hex(params[3].as_str().ok_or(RpcError::ParseError)?).ok();
            let device_name = params[4].as_str().ok_or(RpcError::ParseError)?;
            let device_info = params[5].as_str().ok_or(RpcError::ParseError)?;

            let (id, gid) = state
                .group
                .write()
                .await
                .add_account(name, seed, lock, vec![], device_name, device_info)
                .await?;
            state.layer.write().await.add_running(&gid, gid, id, 0)?;

            let mut results = HandleResult::rpc(json!(vec![gid.to_hex()]));
            results.networks.push(NetworkType::AddGroup(gid)); // add AddGroup to TDN.

            debug!("Account Logined: {}.", gid.to_hex());

            if let Some(addr) = some_addr {
                let group_lock = state.group.read().await;
                let sender = group_lock.sender();
                let msg = group_lock.create_message(&gid, addr)?;
                drop(group_lock);
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let _ = sender.send(SendMessage::Group(gid, msg)).await;
                });
            }

            Ok(results)
        },
    );

    handler.add_method(
        "account-update",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?;
            let avatar = params[1].as_str().ok_or(RpcError::ParseError)?;

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
            let old = params[0].as_str().ok_or(RpcError::ParseError)?;
            let new = params[1].as_str().ok_or(RpcError::ParseError)?;
            let result = HandleResult::rpc(json!([new]));
            state.group.write().await.pin(&gid, old, new)?;
            Ok(result)
        },
    );

    handler.add_method(
        "account-mnemonic",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let lock = params[0].as_str().ok_or(RpcError::ParseError)?;

            let mnemonic = state.group.read().await.mnemonic(&gid, lock)?;
            Ok(HandleResult::rpc(json!([mnemonic])))
        },
    );

    handler.add_method(
        "account-login",
        |_gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gid = GroupId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let me_lock = params[1].as_str().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::rpc(json!([gid.to_hex()]));

            let id = state.group.write().await.add_running(&gid, me_lock)?;
            // add AddGroup to TDN.
            results.networks.push(NetworkType::AddGroup(gid));

            let mut layer_lock = state.layer.write().await;
            layer_lock.add_running(&gid, gid, id, 0)?; // TODO account current state height.

            // load all services layer created by this account.
            // 1. group chat.
            let group_db = group_chat_db(&layer_lock.base, &gid)?;
            let group_chats = GroupChat::all_local(&group_db, &gid)?;
            for (id, gcd, gheight) in group_chats {
                layer_lock.add_running(&gcd, gid, id, gheight)?;
                results.networks.push(NetworkType::AddGroup(gcd));
            }
            drop(layer_lock);

            debug!("Account Logined: {}.", gid.to_hex());

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
                    let data = bincode::serialize(&LayerEvent::Offline(*fgid))?;
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
            tokio::spawn(sleep_waiting_close_stable(sender, groups, layers));

            Ok(results)
        },
    );

    handler.add_method(
        "account-online",
        |_gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gid = GroupId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;

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
            let gid = GroupId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;

            let mut results = HandleResult::new();
            let layer_lock = state.layer.read().await;
            for (fgid, addr) in layer_lock.running(&gid)?.onlines() {
                // send a event that is offline.
                let data = bincode::serialize(&LayerEvent::Offline(*fgid))?;
                let msg = SendType::Event(0, *addr, data);
                results.layers.push((gid, *fgid, msg));
            }
            drop(layer_lock);

            let layers = state.layer.write().await.remove_running(&gid);
            let mut group_lock = state.group.write().await;
            let groups = group_lock.remove_running(&gid);
            let sender = group_lock.sender();
            drop(group_lock);
            tokio::spawn(sleep_waiting_close_stable(sender, groups, layers));

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
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let remote = GroupId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;

            let group_lock = state.group.read().await;
            let db = session_db(group_lock.base(), &gid)?;
            Session::readed(&db, &id)?;

            let mut layer_lock = state.layer.write().await;
            let online = layer_lock.running_mut(&gid)?.active(&remote, true);
            drop(layer_lock);
            if let Some(addr) = online {
                return Ok(HandleResult::rpc(json!([id, addr.to_hex()])));
            }

            let s = Session::get(&db, &id)?;
            drop(db);

            if s.addr == state.layer.read().await.addr {
                return Ok(HandleResult::rpc(json!([id, s.addr.to_hex()])));
            }

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
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let remote = GroupId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let must = params[2].as_bool().ok_or(RpcError::ParseError)?; // if need must suspend.

            let db = session_db(state.group.read().await.base(), &gid)?;
            let s = Session::get(&db, &id)?;
            drop(db);

            let msg = match s.s_type {
                SessionType::Chat | SessionType::Group => {
                    let event = LayerEvent::Suspend(s.gid);
                    let data = bincode::serialize(&event)?;
                    SendType::Event(0, s.addr, data)
                }
                _ => {
                    return Ok(HandleResult::new()); // others has no online.
                }
            };

            let mut layer_lock = state.layer.write().await;
            let suspend = layer_lock.running_mut(&gid)?.suspend(&remote, true, must)?;
            drop(layer_lock);

            let mut results = HandleResult::new();
            if suspend {
                results.rpcs.push(json!([id]))
            }

            match s.s_type {
                SessionType::Chat => {
                    results.layers.push((gid, s.gid, msg));
                }
                SessionType::Group => {
                    add_layer(&mut results, gid, msg);
                }
                _ => {}
            }

            Ok(results)
        },
    );

    handler.add_method(
        "session-readed",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let db = session_db(state.group.read().await.base(), &gid)?;
            Session::readed(&db, &id)?;
            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "session-update",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let is_top = params[1].as_bool().ok_or(RpcError::ParseError)?;
            let is_close = params[2].as_bool().ok_or(RpcError::ParseError)?;

            let db = session_db(state.group.read().await.base(), &gid)?;
            Session::update(&db, &id, is_top, is_close)?;
            Ok(HandleResult::new())
        },
    );

    handler
}
