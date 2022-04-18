use esse_primitives::{id_from_str, id_to_str};
use group_types::{GroupChatId, LayerEvent as GroupLayerEvent, GROUP_CHAT_ID};
use std::net::SocketAddr;
use std::sync::Arc;
use tdn::{
    prelude::{new_send_channel, start_main},
    types::{
        message::{
            NetworkType, RpcSendMessage, SendMessage, SendType, StateRequest, StateResponse,
        },
        primitives::{HandleResult, Peer, PeerId, Result},
        rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
    },
};
use tdn_did::{generate_mnemonic, Count};

use crate::account::lang_from_i64;
use crate::apps::app_rpc_inject;
use crate::apps::group::{group_conn as group_chat_conn, GroupChat};
use crate::global::Global;
use crate::group::{group_conn, group_rpc, GroupEvent};
//use crate::event::InnerEvent;
use crate::session::{connect_session, Session, SessionType};
use crate::storage::{group_db, session_db};

pub(crate) fn init_rpc(global: Arc<Global>) -> RpcHandler<Global> {
    let mut handler = new_rpc_handler(global);

    // inject group rpcs
    group_rpc(&mut handler);

    // inject layers rpcs
    app_rpc_inject(&mut handler);
    handler
}

#[inline]
pub(crate) fn network_stable(peers: Vec<(PeerId, bool)>) -> RpcParam {
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
    rpc_response(0, "network-stable", json!(s_peers))
}

#[inline]
pub(crate) fn network_dht(peers: Vec<PeerId>) -> RpcParam {
    let s_peers: Vec<String> = peers.iter().map(|p| p.to_hex()).collect();
    rpc_response(0, "network-dht", json!(s_peers))
}

#[inline]
pub(crate) fn _account_update(pid: &PeerId, name: &str, avatar: String) -> RpcParam {
    rpc_response(0, "account-update", json!([id_to_str(pid), name, avatar]))
}

#[inline]
pub(crate) fn session_create(session: &Session) -> RpcParam {
    rpc_response(0, "session-create", session.to_rpc())
}

#[inline]
pub(crate) fn session_last(id: &i64, time: &i64, content: &str, readed: bool) -> RpcParam {
    rpc_response(0, "session-last", json!([id, time, content, readed]))
}

#[inline]
pub(crate) fn notice_menu(t: &SessionType) -> RpcParam {
    rpc_response(0, "notice-menu", json!([t.to_int()]))
}

#[inline]
pub(crate) fn session_update_name(id: &i64, name: &str) -> RpcParam {
    rpc_response(0, "session-update", json!([id, "", name, false]))
}

#[inline]
pub(crate) fn _session_update(id: &i64, addr: &PeerId, name: &str, is_top: bool) -> RpcParam {
    rpc_response(
        0,
        "session-update",
        json!([id, addr.to_hex(), name, is_top]),
    )
}

#[inline]
pub(crate) fn session_connect(id: &i64, addr: &PeerId) -> RpcParam {
    rpc_response(0, "session-connect", json!([id, addr.to_hex()]))
}

#[inline]
pub(crate) fn session_suspend(id: &i64) -> RpcParam {
    rpc_response(0, "session-suspend", json!([id]))
}

#[inline]
pub(crate) fn session_lost(id: &i64) -> RpcParam {
    rpc_response(0, "session-lost", json!([id]))
}

#[inline]
pub(crate) fn session_delete(id: &i64) -> RpcParam {
    rpc_response(0, "session-delete", json!([id]))
}

#[inline]
pub(crate) fn session_close(id: &i64) -> RpcParam {
    rpc_response(0, "session-close", json!([id]))
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
pub(crate) async fn inner_rpc(uid: u64, method: &str, global: &Arc<Global>) -> Result<()> {
    // Inner network default rpc method. only use in http-rpc.
    if method == "network-stable" || method == "network-dht" {
        let req = match method {
            "network-stable" => StateRequest::Stable,
            "network-dht" => StateRequest::DHT,
            _ => return Ok(()),
        };

        let (s, mut r) = tokio::sync::mpsc::channel::<StateResponse>(128);
        let _ = global
            .send(SendMessage::Network(NetworkType::NetworkState(req, s)))
            .await?;

        let param = match r.recv().await {
            Some(StateResponse::Stable(peers)) => network_stable(peers),
            Some(StateResponse::DHT(peers)) => network_dht(peers),
            Some(_) | None => {
                return Ok(());
            }
        };

        global
            .rpc_send
            .send(RpcSendMessage(uid, param, false))
            .await?;
        return Ok(());
    }

    Err(anyhow!("not found"))
}

fn new_rpc_handler(global: Arc<Global>) -> RpcHandler<Global> {
    let mut handler = RpcHandler::new_with_state(global);

    handler.add_method("echo", |params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method("add-bootstrap", |params: Vec<RpcParam>, _| async move {
        let socket = params[0].as_str().ok_or(RpcError::ParseError)?;
        let transport = params[1].as_str().ok_or(RpcError::ParseError)?;

        if let Ok(addr) = socket.parse::<SocketAddr>() {
            Ok(HandleResult::network(NetworkType::Connect(
                Peer::socket_transport(addr, transport),
            )))
        } else {
            Err(RpcError::InvalidRequest)
        }
    });

    handler.add_method("account-list", |_, state: Arc<Global>| async move {
        let mut accounts: Vec<Vec<String>> = vec![];
        let own_lock = state.own.read().await;
        for (pid, account) in own_lock.list_accounts().iter() {
            accounts.push(vec![
                id_to_str(pid),
                account.name.clone(),
                base64::encode(&account.avatar),
            ]);
        }
        drop(own_lock);

        Ok(HandleResult::rpc(json!(accounts)))
    });

    handler.add_method(
        "account-generate",
        |params: Vec<RpcParam>, _state: Arc<Global>| async move {
            let lang = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let language = lang_from_i64(lang);
            let words = generate_mnemonic(language, Count::Words12);
            Ok(HandleResult::rpc(json!([words])))
        },
    );

    handler.add_method(
        "account-create",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let lang = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let seed = params[1].as_str().ok_or(RpcError::ParseError)?;
            let pass = params[2].as_str().ok_or(RpcError::ParseError)?;

            let name = params[3].as_str().ok_or(RpcError::ParseError)?;
            let lock = params[4].as_str().ok_or(RpcError::ParseError)?;
            let avatar = params[5].as_str().ok_or(RpcError::ParseError)?;

            let avatar_bytes = base64::decode(avatar).unwrap_or(vec![]);
            let (_id, pid) = state
                .own
                .write()
                .await
                .add_account(
                    lang,
                    seed,
                    pass,
                    name,
                    lock,
                    avatar_bytes,
                    &state.base,
                    &state.secret,
                )
                .await?;

            Ok(HandleResult::rpc(json!(vec![id_to_str(&pid)])))
        },
    );

    handler.add_method(
        "account-restore",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let lang = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let seed = params[1].as_str().ok_or(RpcError::ParseError)?;
            let pass = params[2].as_str().ok_or(RpcError::ParseError)?;

            let name = params[3].as_str().ok_or(RpcError::ParseError)?;
            let lock = params[4].as_str().ok_or(RpcError::ParseError)?;

            let (_id, pid) = state
                .own
                .write()
                .await
                .add_account(
                    lang,
                    seed,
                    pass,
                    name,
                    lock,
                    vec![],
                    &state.base,
                    &state.secret,
                )
                .await?;

            // TODO auto search online account info.

            Ok(HandleResult::rpc(json!(vec![id_to_str(&pid)])))
        },
    );

    handler.add_method(
        "account-update",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?;
            let avatar = params[1].as_str().ok_or(RpcError::ParseError)?;

            let avatar_bytes = base64::decode(avatar).unwrap_or(vec![]);
            let pid = state.pid().await;

            let mut own_lock = state.own.write().await;
            own_lock.update_account(pid, name, avatar_bytes.clone(), &state.base, &state.secret)?;
            drop(own_lock);

            let results = HandleResult::new();

            // TODO broadcast to all devices.
            //let user = own_lock.clone_user(&pid)?;
            //own_lock.broadcast(&pid, &mut results)?;

            // TODO broadcast to all layers.
            //state.layer.read().await.broadcast(user, &mut results);

            Ok(results)
        },
    );

    handler.add_method(
        "account-pin-check",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let pid = id_from_str(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let lock = params[1].as_str().ok_or(RpcError::ParseError)?;
            let res = state.own.read().await.check_lock(&pid, lock);
            Ok(HandleResult::rpc(json!([res])))
        },
    );

    handler.add_method(
        "account-pin-change",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let old = params[0].as_str().ok_or(RpcError::ParseError)?;
            let new = params[1].as_str().ok_or(RpcError::ParseError)?;
            let pid = state.pid().await;
            let result = HandleResult::rpc(json!([new]));
            state
                .own
                .write()
                .await
                .pin(&pid, old, new, &state.base, &state.secret)?;
            Ok(result)
        },
    );

    handler.add_method(
        "account-mnemonic",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let lock = params[0].as_str().ok_or(RpcError::ParseError)?;
            let pid = state.pid().await;
            let mnemonic = state.own.read().await.mnemonic(&pid, lock, &state.secret)?;
            Ok(HandleResult::rpc(json!([mnemonic])))
        },
    );

    handler.add_method(
        "account-login",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let pid = id_from_str(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let me_lock = params[1].as_str().ok_or(RpcError::ParseError)?;

            let results = HandleResult::rpc(json!([id_to_str(&pid)]));

            let (tdn_send, tdn_recv) = new_send_channel();
            let running = state.reset(&pid, me_lock, tdn_send).await?;
            if running {
                return Ok(results);
            }

            // load all local services created by this account.
            let db_key = state.own.read().await.db_key(&pid)?;
            let group_db = group_db(&state.base, &pid, &db_key)?;
            let s_db = session_db(&state.base, &pid, &db_key)?;
            // 1. group chat.
            let group_chats = GroupChat::local(&group_db)?;
            let mut layer = state.layer.write().await;
            for g in group_chats {
                // 2. online group to self group onlines.
                if let Some(s) = connect_session(&s_db, &SessionType::Group, &g.id, &pid)? {
                    layer.group_add(g.gid, g.addr, s.id, g.id, g.height);
                }
            }
            drop(layer);

            let key = state.own.read().await.keypair();
            let peer_id = start_main(
                state.gids.clone(),
                state.p2p_config.clone(),
                state.self_send.clone(),
                tdn_recv,
                None,
                Some(key),
            )
            .await?;

            debug!("Account Logined: {}.", id_to_str(&peer_id));

            Ok(results)
        },
    );

    handler.add_method(
        "account-logout",
        |_params: Vec<RpcParam>, state: Arc<Global>| async move {
            let mut results = HandleResult::new();
            results.networks.push(NetworkType::NetworkStop);
            let pid = state.pid().await;
            debug!("Account Offline: {}.", id_to_str(&pid));
            state.clear().await;
            Ok(results)
        },
    );

    handler.add_method(
        "session-list",
        |_: Vec<RpcParam>, state: Arc<Global>| async move {
            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = session_db(&state.base, &pid, &db_key)?;
            Ok(HandleResult::rpc(session_list(Session::list(&db)?)))
        },
    );

    handler.add_method(
        "session-connect",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let remote = params[1].as_str().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = session_db(&state.base, &pid, &db_key)?;
            Session::readed(&db, &id)?;
            let s = Session::get(&db, &id)?;
            drop(db);

            let mut results = HandleResult::new();
            match s.s_type {
                SessionType::Chat => {
                    let remote_pid = id_from_str(remote)?;
                    if state.group.write().await.active(&remote_pid, true).is_ok() {
                        return Ok(HandleResult::rpc(json!([id, id_to_str(&remote_pid)])));
                    }
                    group_conn(remote_pid, &mut results);
                }
                SessionType::Group => {
                    let remote_gid: GroupChatId =
                        remote.parse().map_err(|_| RpcError::ParseError)?;
                    let online = state.layer.write().await.group_active(&remote_gid, true);
                    if let Some(addr) = online {
                        return Ok(HandleResult::rpc(json!([id, id_to_str(&addr)])));
                    }
                    group_chat_conn(s.addr, remote_gid, &mut results);
                }
                _ => {}
            }

            Ok(results)
        },
    );

    handler.add_method(
        "session-suspend",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let remote = params[1].as_str().ok_or(RpcError::ParseError)?;
            let must = params[2].as_bool().ok_or(RpcError::ParseError)?; // if need must suspend.

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = session_db(&state.base, &pid, &db_key)?;
            let s = Session::get(&db, &id)?;
            drop(db);

            let mut results = HandleResult::new();
            match s.s_type {
                SessionType::Chat => {
                    let rid = id_from_str(remote)?;
                    if state.group.write().await.suspend(&rid, true, must).is_ok() {
                        results.rpcs.push(json!([id]));
                    }
                    let data = bincode::serialize(&GroupEvent::Suspend)?;
                    results.groups.push(SendType::Event(0, rid, data));
                }
                SessionType::Group => {
                    let remote_gid: GroupChatId =
                        remote.parse().map_err(|_| RpcError::ParseError)?;
                    let mut layer_lock = state.layer.write().await;
                    if layer_lock.group_suspend(&remote_gid, true, must)?.is_some() {
                        results.rpcs.push(json!([id]));
                    }
                    drop(layer_lock);
                    let data = bincode::serialize(&GroupLayerEvent::Suspend(remote_gid))?;
                    let msg = SendType::Event(0, s.addr, data);
                    results.layers.push((GROUP_CHAT_ID, msg));
                }
                _ => {
                    return Ok(HandleResult::new()); // others has no online.
                }
            };

            Ok(results)
        },
    );

    handler.add_method(
        "session-readed",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = session_db(&state.base, &pid, &db_key)?;
            Session::readed(&db, &id)?;
            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "session-update",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let is_top = params[1].as_bool().ok_or(RpcError::ParseError)?;
            let is_close = params[2].as_bool().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = session_db(&state.base, &pid, &db_key)?;
            Session::update(&db, &id, is_top, is_close)?;
            Ok(HandleResult::new())
        },
    );

    handler
}
