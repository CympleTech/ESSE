use simplelog::{CombinedLogger, Config as LogConfig, LevelFilter};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tdn::{
    prelude::*,
    smol::{
        channel::{SendError, Sender},
        io::Result,
        lock::RwLock,
    },
    types::primitive::HandleResult,
};

use crate::group::Group;
use crate::layer::Layer;
use crate::migrate::main_migrate;
use crate::models::account::Account;
use crate::primitives::network_seeds;
use crate::rpc::{inner_rpc, new_rpc_handler};
use crate::storage::account_db;

pub const DEFAULT_WS_ADDR: &'static str = "127.0.0.1:8080";
pub const DEFAULT_LOG_FILE: &'static str = "esse.log.txt";

pub async fn start(db_path: String) -> Result<()> {
    let db_path = PathBuf::from(db_path);
    if !db_path.exists() {
        tdn::smol::fs::create_dir_all(&db_path).await?;
    }

    init_log(db_path.clone());
    main_migrate(&db_path)?;
    info!("Core storage path {:?}", db_path);

    let mut config = Config::load_save(db_path.clone()).await;
    config.db_path = Some(db_path.clone());
    config.p2p_allowlist.append(&mut network_seeds());
    // use self sign to bootstrap peer.
    if config.rpc_ws.is_none() {
        // set default ws addr.
        config.rpc_ws = Some(DEFAULT_WS_ADDR.parse().unwrap());
    }

    info!("Config RPC HTTP : {:?}", config.rpc_addr);
    info!("Config RPC WS   : {:?}", config.rpc_ws);
    info!("Config P2P      : {:?}", config.p2p_addr);

    let rand_secret = config.secret.clone();

    let account_db = account_db(&db_path)?;
    let accounts = Account::all(&account_db)?;
    account_db.close()?;
    let mut me: HashMap<GroupId, Account> = HashMap::new();
    for account in accounts {
        me.insert(account.gid, account);
    }
    config.group_ids = me.keys().cloned().collect();

    let (peer_id, sender, recver) = start_with_config(config).await.unwrap();
    info!("Network Peer id : {}", peer_id.to_hex());

    let group = Arc::new(RwLock::new(
        Group::init(rand_secret, sender.clone(), peer_id, me, db_path.clone()).await?,
    ));
    let layer = Arc::new(RwLock::new(
        Layer::init(db_path, peer_id, group.clone()).await?,
    ));

    let rpc = new_rpc_handler(peer_id, group.clone(), layer.clone());
    //let mut group_rpcs: HashMap<u64, GroupId> = HashMap::new();
    let mut now_rpc_uid = 0;

    while let Ok(message) = recver.recv().await {
        match message {
            ReceiveMessage::Group(fgid, g_msg) => {
                if let Ok(handle_result) =
                    group.write().await.handle(fgid, g_msg, &layer, now_rpc_uid)
                {
                    handle(handle_result, now_rpc_uid, true, &sender).await;
                }
            }
            ReceiveMessage::Layer(fgid, tgid, l_msg) => {
                if let Ok(handle_result) = layer.write().await.handle(fgid, tgid, l_msg).await {
                    handle(handle_result, now_rpc_uid, true, &sender).await;
                }
            }
            ReceiveMessage::Rpc(uid, params, is_ws) => {
                if !is_ws {
                    if inner_rpc(uid, params["method"].as_str().unwrap(), &sender)
                        .await
                        .is_ok()
                    {
                        continue;
                    }
                }

                if now_rpc_uid != uid && is_ws {
                    now_rpc_uid = uid
                }

                if let Ok(handle_result) = rpc.handle(params).await {
                    handle(handle_result, uid, is_ws, &sender).await;
                }
            }
            ReceiveMessage::NetworkLost => {
                sender
                    .send(SendMessage::Network(NetworkType::NetworkReboot))
                    .await
                    .expect("TDN channel closed");
                let t_sender = sender.clone();
                let g_conns = group.read().await.all_distribute_conns();
                let l_conns = layer.read().await.all_friend_conns().await;
                tdn::smol::spawn(sleep_waiting_reboot(t_sender, g_conns, l_conns)).detach();
            }
        }
    }

    Ok(())
}

#[inline]
async fn sleep_waiting_reboot(
    sender: Sender<SendMessage>,
    groups: HashMap<GroupId, Vec<SendType>>,
    layers: HashMap<GroupId, Vec<(GroupId, SendType)>>,
) -> std::result::Result<(), SendError<SendMessage>> {
    tdn::smol::Timer::after(std::time::Duration::from_secs(10)).await;
    for (gid, conns) in groups {
        for conn in conns {
            sender.send(SendMessage::Group(gid, conn)).await?;
        }
    }

    for (fgid, conns) in layers {
        for (tgid, conn) in conns {
            sender.send(SendMessage::Layer(fgid, tgid, conn)).await?;
        }
    }

    Ok(())
}

#[inline]
async fn handle(handle_result: HandleResult, uid: u64, is_ws: bool, sender: &Sender<SendMessage>) {
    let HandleResult {
        mut rpcs,
        mut groups,
        mut layers,
        mut networks,
    } = handle_result;

    loop {
        if rpcs.len() != 0 {
            let msg = rpcs.remove(0);
            sender
                .send(SendMessage::Rpc(uid, msg, is_ws))
                .await
                .expect("TDN channel closed");
        } else {
            break;
        }
    }

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

    loop {
        if groups.len() != 0 {
            let (gid, msg) = groups.remove(0);
            sender
                .send(SendMessage::Group(gid, msg))
                .await
                .expect("TDN channel closed");
        } else {
            break;
        }
    }

    loop {
        if layers.len() != 0 {
            let (fgid, tgid, msg) = layers.remove(0);
            sender
                .send(SendMessage::Layer(fgid, tgid, msg))
                .await
                .expect("TDN channel closed");
        } else {
            break;
        }
    }
}

#[inline]
pub fn init_log(mut db_path: PathBuf) {
    db_path.push(DEFAULT_LOG_FILE);

    #[cfg(debug_assertions)]
    CombinedLogger::init(vec![simplelog::TermLogger::new(
        LevelFilter::Debug,
        LogConfig::default(),
        simplelog::TerminalMode::Mixed,
    )])
    .unwrap();

    #[cfg(not(debug_assertions))]
    CombinedLogger::init(vec![simplelog::WriteLogger::new(
        LevelFilter::Debug,
        LogConfig::default(),
        std::fs::File::create(db_path).unwrap(),
    )])
    .unwrap();
}
