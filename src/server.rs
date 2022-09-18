use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tdn::{
    prelude::*,
    types::{
        message::RpcSendMessage,
        primitives::{HandleResult, Result},
    },
};
use tdn_storage::local::DStorage;

use crate::account::Account;
use crate::apps::app_layer_handle;
use crate::global::Global;
use crate::group::group_handle;
use crate::migrate::{main_migrate, ACCOUNT_DB};
use crate::own::handle as own_handle;
use crate::primitives::network_seeds;
use crate::rpc::{init_rpc, inner_rpc, session_lost};

pub const DEFAULT_WS_ADDR: &'static str = "127.0.0.1:7366";

pub static RPC_WS_UID: OnceCell<u64> = OnceCell::new();

pub async fn start(db_path: PathBuf) -> Result<()> {
    let mut config = Config::default();
    config.db_path = Some(db_path.clone());
    config.p2p_allowlist.append(&mut network_seeds());
    config.rpc_ws = Some(DEFAULT_WS_ADDR.parse().unwrap());
    let config = Config::load_save(db_path.clone(), config).await?;

    info!("Config RPC HTTP : {:?}", config.rpc_http);
    info!("Config RPC WS   : {:?}", config.rpc_ws);
    info!(
        "Config P2P      : {} {:?}",
        config.p2p_peer.transport.to_str(),
        config.p2p_peer.socket
    );

    let rand_secret = config.secret.clone();
    main_migrate(&db_path, &hex::encode(&rand_secret))?;
    info!("Core storage path {:?}", db_path);

    let mut account_db_path = db_path.clone();
    account_db_path.push(ACCOUNT_DB);
    let account_db = DStorage::open(account_db_path, &hex::encode(&rand_secret))?;
    let accounts = Account::all(&account_db)?;
    account_db.close()?;
    let mut me: HashMap<PeerId, Account> = HashMap::new();
    for account in accounts {
        me.insert(account.pid, account);
    }

    let (_, _, p2p_config, rpc_config) = config.split();
    let (self_send, mut self_recv) = new_receive_channel();
    let rpc_send = start_rpc(rpc_config, self_send.clone()).await?;

    let global = Arc::new(Global::init(
        me,
        db_path,
        rand_secret,
        p2p_config,
        self_send,
        rpc_send,
    ));

    let rpc = init_rpc(global.clone());
    // //let mut group_rpcs: HashMap<u64, GroupId> = HashMap::new();
    let mut now_rpc_uid = 0;

    // running session remain task.
    tokio::spawn(session_remain(global.clone()));

    while let Some(message) = self_recv.recv().await {
        match message {
            ReceiveMessage::Own(o_msg) => {
                if let Ok(handle_result) = own_handle(o_msg, &global).await {
                    handle(handle_result, now_rpc_uid, true, &global).await;
                }
            }
            ReceiveMessage::Group(g_msg) => {
                if let Ok(handle_result) = group_handle(g_msg, &global).await {
                    handle(handle_result, now_rpc_uid, true, &global).await;
                }
            }
            ReceiveMessage::Layer(fgid, tgid, l_msg) => {
                if let Ok(handle_result) = app_layer_handle(fgid, tgid, l_msg, &global).await {
                    handle(handle_result, now_rpc_uid, true, &global).await;
                }
            }
            ReceiveMessage::Rpc(uid, params, is_ws) => {
                if !is_ws {
                    if inner_rpc(uid, params["method"].as_str().unwrap(), &global)
                        .await
                        .is_ok()
                    {
                        continue;
                    }
                }

                if now_rpc_uid != uid && is_ws {
                    let _ = RPC_WS_UID.set(uid);
                    now_rpc_uid = uid
                }

                if let Ok(handle_result) = rpc.handle(params).await {
                    handle(handle_result, uid, is_ws, &global).await;
                }
            }
            ReceiveMessage::NetworkLost => {
                global
                    .send(SendMessage::Network(NetworkType::NetworkReboot))
                    .await?;
            }
        }
    }

    Ok(())
}

async fn session_remain(global: Arc<Global>) -> Result<()> {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(120)).await;
        if let Some(uid) = RPC_WS_UID.get() {
            let mut rpcs = vec![];
            let mut addrs = vec![];

            // clear group connections.
            let mut group_lock = global.group.write().await;
            let mut closed = vec![];
            for (pid, session) in group_lock.sessions.iter_mut() {
                if session.clear() {
                    closed.push((*pid, session.sid));
                    addrs.push(*pid);
                }
            }
            for (pid, sid) in closed {
                group_lock.rm_online(&pid);
                rpcs.push(session_lost(&sid));
            }
            drop(group_lock);

            // clear layer connections.
            let mut layer_lock = global.layer.write().await;
            let mut closed = vec![];
            for (gcid, session) in layer_lock.groups.iter_mut() {
                if session.clear() {
                    closed.push((*gcid, session.sid));
                    for addr in &session.addrs {
                        addrs.push(*addr);
                    }
                }
            }
            for (gcid, sid) in closed {
                layer_lock.group_del(&gcid);
                rpcs.push(session_lost(&sid));
            }
            drop(layer_lock);

            for rpc in rpcs {
                let _ = global.send(SendMessage::Rpc(*uid, rpc, true)).await;
            }

            for addr in addrs {
                if global.group.read().await.is_online(&addr) {
                    continue;
                }

                if global.layer.read().await.is_addr_online(&addr) {
                    continue;
                }

                let _ = global
                    .send(SendMessage::Layer(
                        GroupId::default(),
                        SendType::Disconnect(addr),
                    ))
                    .await;
            }
        }
    }
}

#[inline]
async fn handle(handle_result: HandleResult, uid: u64, is_ws: bool, global: &Arc<Global>) {
    let HandleResult {
        mut owns,
        mut rpcs,
        mut layers,
        mut networks,
        mut groups,
    } = handle_result;

    loop {
        if rpcs.len() != 0 {
            let msg = rpcs.remove(0);
            global
                .rpc_send
                .send(RpcSendMessage(uid, msg, is_ws))
                .await
                .expect("TDN channel closed");
        } else {
            break;
        }
    }

    if let Ok(sender) = global.sender().await {
        loop {
            if owns.len() != 0 {
                let msg = owns.remove(0);
                sender
                    .send(SendMessage::Own(msg))
                    .await
                    .expect("TDN channel closed");
            } else {
                break;
            }
        }

        loop {
            if groups.len() != 0 {
                let msg = groups.remove(0);
                sender
                    .send(SendMessage::Group(msg))
                    .await
                    .expect("TDN channel closed");
            } else {
                break;
            }
        }

        loop {
            if layers.len() != 0 {
                let (tgid, msg) = layers.remove(0);
                sender
                    .send(SendMessage::Layer(tgid, msg))
                    .await
                    .expect("TDN channel closed");
            } else {
                break;
            }
        }

        // must last send, because it will has stop type.
        loop {
            if networks.len() != 0 {
                let msg = networks.remove(0);
                sender
                    .send(SendMessage::Network(msg))
                    .await
                    .expect("TDN channel closed");
            } else {
                break;
            }
        }
    }
}
