use esse_primitives::{id_from_str, id_to_str};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::{NetworkType, SendMessage, SendType, StateRequest, StateResponse},
    primitives::{HandleResult, Peer, PeerId, Result},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};
use tdn_did::{generate_mnemonic, Count};
use tokio::sync::{
    mpsc::{self, error::SendError, Sender},
    RwLock,
};

use crate::account::lang_from_i64;
use crate::apps::app_rpc_inject;
use crate::global::Global;
//use crate::apps::chat::chat_conn;
//use crate::apps::group::{add_layer, group_conn, GroupChat};
//use crate::event::InnerEvent;
use crate::group::Group;
use crate::layer::{Layer, LayerEvent, Online};
//use crate::session::{connect_session, Session, SessionType};

pub(crate) fn init_rpc(global: Arc<Global>) -> RpcHandler<Global> {
    let mut handler = new_rpc_handler(global);
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
pub(crate) fn account_update(pid: &PeerId, name: &str, avatar: String) -> RpcParam {
    rpc_response(0, "account-update", json!([id_to_str(pid), name, avatar]))
}

// #[inline]
// pub(crate) fn session_create(mgid: GroupId, session: &Session) -> RpcParam {
//     rpc_response(0, "session-create", session.to_rpc(), mgid)
// }

// #[inline]
// pub(crate) fn session_last(
//     mgid: GroupId,
//     id: &i64,
//     time: &i64,
//     content: &str,
//     readed: bool,
// ) -> RpcParam {
//     rpc_response(0, "session-last", json!([id, time, content, readed]), mgid)
// }

// #[inline]
// pub(crate) fn notice_menu(mgid: GroupId, t: &SessionType) -> RpcParam {
//     rpc_response(0, "notice-menu", json!([t.to_int()]), mgid)
// }

// #[inline]
// pub(crate) fn session_update_name(mgid: GroupId, id: &i64, name: &str) -> RpcParam {
//     rpc_response(0, "session-update", json!([id, "", name, false]), mgid)
// }

// #[inline]
// pub(crate) fn session_update(
//     mgid: GroupId,
//     id: &i64,
//     addr: &PeerId,
//     name: &str,
//     is_top: bool,
// ) -> RpcParam {
//     rpc_response(
//         0,
//         "session-update",
//         json!([id, addr.to_hex(), name, is_top]),
//         mgid,
//     )
// }

// #[inline]
// pub(crate) fn session_connect(mgid: GroupId, id: &i64, addr: &PeerId) -> RpcParam {
//     rpc_response(0, "session-connect", json!([id, addr.to_hex()]), mgid)
// }

// #[inline]
// pub(crate) fn session_suspend(mgid: GroupId, id: &i64) -> RpcParam {
//     rpc_response(0, "session-suspend", json!([id]), mgid)
// }

// #[inline]
// pub(crate) fn session_lost(mgid: GroupId, id: &i64) -> RpcParam {
//     rpc_response(0, "session-lost", json!([id]), mgid)
// }

// #[inline]
// pub(crate) fn session_delete(mgid: GroupId, id: &i64) -> RpcParam {
//     rpc_response(0, "session-delete", json!([id]), mgid)
// }

// #[inline]
// pub(crate) fn session_close(mgid: GroupId, id: &i64) -> RpcParam {
//     rpc_response(0, "session-close", json!([id]), mgid)
// }

// #[inline]
// fn session_list(sessions: Vec<Session>) -> RpcParam {
//     let mut results = vec![];
//     for session in sessions {
//         results.push(session.to_rpc());
//     }
//     json!(results)
// }

#[inline]
pub(crate) async fn sleep_waiting_close_stable(
    sender: Sender<SendMessage>,
    groups: HashMap<PeerId, ()>,
    layers: HashMap<PeerId, GroupId>,
) -> std::result::Result<(), SendError<SendMessage>> {
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    for (addr, _) in groups {
        sender
            .send(SendMessage::Group(SendType::Disconnect(addr)))
            .await?;
    }

    for (faddr, fgid) in layers {
        sender
            .send(SendMessage::Layer(fgid, SendType::Disconnect(faddr)))
            .await?;
    }

    Ok(())
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

        let (s, mut r) = mpsc::channel::<StateResponse>(128);
        let _ = global
            .sender
            .read()
            .await
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

        global
            .sender
            .read()
            .await
            .send(SendMessage::Rpc(uid, param, false))
            .await
            .expect("TDN channel closed");

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
        let group_lock = state.group.read().await;
        for (pid, account) in group_lock.list_accounts().iter() {
            accounts.push(vec![
                id_to_str(pid),
                account.name.clone(),
                base64::encode(&account.avatar),
            ]);
        }
        drop(group_lock);

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
                .group
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
                .group
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

            let mut group_lock = state.group.write().await;
            group_lock.update_account(
                pid,
                name,
                avatar_bytes.clone(),
                &state.base,
                &state.secret,
            )?;
            drop(group_lock);

            let results = HandleResult::new();

            // TODO broadcast to all devices.
            //let user = group_lock.clone_user(&pid)?;
            //group_lock.broadcast(&pid, &mut results)?;

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
            let res = state.group.read().await.check_lock(&pid, lock);
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
                .group
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
            let mnemonic = state
                .group
                .read()
                .await
                .mnemonic(&pid, lock, &state.secret)?;
            Ok(HandleResult::rpc(json!([mnemonic])))
        },
    );

    handler.add_method(
        "account-login",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let pid = id_from_str(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let me_lock = params[1].as_str().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::rpc(json!([id_to_str(&pid)]));

            let running = state.reset(&pid, me_lock).await?;
            if running {
                return Ok(results);
            }

            // TODO load all local services created by this account.
            // 1. group chat.
            // let self_addr = layer_lock.addr.clone();
            // let group_lock = state.group.read().await;
            // let group_db = group_lock.group_db(&ogid)?;
            // let s_db = group_lock.session_db(&ogid)?;
            // drop(group_lock);
            // let group_chats = GroupChat::local(&group_db)?;
            // for g in group_chats {
            //     layer_lock.add_running(&g.g_id, ogid, g.id, g.height)?;
            //     results.networks.push(NetworkType::AddGroup(g.g_id));

            //     // 2. online group to self group onlines.
            //     if let Some(session) =
            //         connect_session(&s_db, &SessionType::Group, &g.id, &self_addr)?
            //     {
            //         layer_lock.running_mut(&ogid)?.check_add_online(
            //             g.g_id,
            //             Online::Direct(self_addr),
            //             session.id,
            //             g.id,
            //         )?;
            //     }
            // }
            // drop(layer_lock);

            debug!("Account Logined: {}.", id_to_str(&pid));

            Ok(results)
        },
    );

    handler.add_method(
        "account-logout",
        |_params: Vec<RpcParam>, state: Arc<Global>| async move {
            let mut results = HandleResult::new();

            // TODO broadcast to inner-group.
            let group_lock = state.group.read().await;
            drop(group_lock);

            // TODO broadcast to layers.
            let layer_lock = state.layer.read().await;
            for (gid, sessions) in layer_lock.sessions.iter() {
                for (pid, _) in sessions {
                    // send a event that is offline.
                    let data = bincode::serialize(&LayerEvent::Offline(*gid))?;
                    let msg = SendType::Event(0, *pid, data);
                    results.layers.push((*gid, msg));
                }
            }
            drop(layer_lock);

            debug!("Account Offline: {}.", id_to_str(&state.pid().await));

            //let sender = state.sender.read().await.clone();
            state.clear().await;
            //tokio::spawn(sleep_waiting_close_stable(sender, groups, layers));

            Ok(results)
        },
    );

    // handler.add_method(
    //     "session-list",
    //     |gid: GroupId, _params: Vec<RpcParam>, state: Arc<Global>| async move {
    //         let db = state.group.read().await.session_db(&gid)?;
    //         Ok(HandleResult::rpc(session_list(Session::list(&db)?)))
    //     },
    // );

    // handler.add_method(
    //     "session-connect",
    //     |gid: GroupId, params: Vec<RpcParam>, state: Arc<Global>| async move {
    //         let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
    //         let remote = GroupId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;

    //         let group_lock = state.group.read().await;
    //         let db = group_lock.session_db(&gid)?;
    //         Session::readed(&db, &id)?;

    //         let mut layer_lock = state.layer.write().await;
    //         let online = layer_lock.running_mut(&gid)?.active(&remote, true);
    //         drop(layer_lock);
    //         if let Some(addr) = online {
    //             return Ok(HandleResult::rpc(json!([id, addr.to_hex()])));
    //         }

    //         let s = Session::get(&db, &id)?;
    //         drop(db);

    //         let mut results = HandleResult::new();
    //         match s.s_type {
    //             SessionType::Chat => {
    //                 let proof = group_lock.prove_addr(&gid, &s.addr)?;
    //                 results
    //                     .layers
    //                     .push((gid, s.gid, chat_conn(proof, Peer::peer(s.addr))));
    //             }
    //             SessionType::Group => {
    //                 let proof = group_lock.prove_addr(&gid, &s.addr)?;
    //                 add_layer(
    //                     &mut results,
    //                     gid,
    //                     group_conn(proof, Peer::peer(s.addr), s.gid),
    //                 );
    //             }
    //             _ => {}
    //         }
    //         Ok(results)
    //     },
    // );

    // handler.add_method(
    //     "session-suspend",
    //     |gid: GroupId, params: Vec<RpcParam>, state: Arc<Global>| async move {
    //         let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
    //         let remote = GroupId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
    //         let must = params[2].as_bool().ok_or(RpcError::ParseError)?; // if need must suspend.

    //         let db = state.group.read().await.session_db(&gid)?;
    //         let s = Session::get(&db, &id)?;
    //         drop(db);

    //         let msg = match s.s_type {
    //             SessionType::Chat | SessionType::Group => {
    //                 let event = LayerEvent::Suspend(s.gid);
    //                 let data = bincode::serialize(&event)?;
    //                 SendType::Event(0, s.addr, data)
    //             }
    //             _ => {
    //                 return Ok(HandleResult::new()); // others has no online.
    //             }
    //         };

    //         let mut layer_lock = state.layer.write().await;
    //         let suspend = layer_lock.running_mut(&gid)?.suspend(&remote, true, must)?;
    //         drop(layer_lock);

    //         let mut results = HandleResult::new();
    //         if suspend {
    //             results.rpcs.push(json!([id]))
    //         }

    //         match s.s_type {
    //             SessionType::Chat => {
    //                 results.layers.push((gid, s.gid, msg));
    //             }
    //             SessionType::Group => {
    //                 add_layer(&mut results, gid, msg);
    //             }
    //             _ => {}
    //         }

    //         Ok(results)
    //     },
    // );

    // handler.add_method(
    //     "session-readed",
    //     |gid: GroupId, params: Vec<RpcParam>, state: Arc<Global>| async move {
    //         let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
    //         let db = state.group.read().await.session_db(&gid)?;
    //         Session::readed(&db, &id)?;
    //         Ok(HandleResult::new())
    //     },
    // );

    // handler.add_method(
    //     "session-update",
    //     |gid: GroupId, params: Vec<RpcParam>, state: Arc<Global>| async move {
    //         let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
    //         let is_top = params[1].as_bool().ok_or(RpcError::ParseError)?;
    //         let is_close = params[2].as_bool().ok_or(RpcError::ParseError)?;

    //         let db = state.group.read().await.session_db(&gid)?;
    //         Session::update(&db, &id, is_top, is_close)?;
    //         Ok(HandleResult::new())
    //     },
    // );

    handler
}
