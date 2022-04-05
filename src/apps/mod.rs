use cloud_types::CLOUD_ID;
use dao_types::DAO_ID;
use domain_types::DOMAIN_ID;
use group_types::{GroupChatId, GROUP_CHAT_ID};
use std::collections::HashMap;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::RecvType,
    primitives::{HandleResult, Result},
    rpc::RpcHandler,
};

use crate::global::Global;
use crate::rpc::session_lost;
use crate::storage::group_db;

pub(crate) mod cloud;
pub(crate) mod device;
pub(crate) mod domain;
pub(crate) mod file;
pub(crate) mod group;
pub(crate) mod jarvis;
pub(crate) mod wallet;
//pub(crate) mod dao;

pub(crate) fn app_rpc_inject(handler: &mut RpcHandler<Global>) {
    device::new_rpc_handler(handler);
    jarvis::new_rpc_handler(handler);
    domain::new_rpc_handler(handler);
    file::new_rpc_handler(handler);
    group::new_rpc_handler(handler);
    wallet::new_rpc_handler(handler);
    cloud::new_rpc_handler(handler);
    //dao::new_rpc_handler(handler);
}

pub(crate) async fn app_layer_handle(
    fgid: GroupId,
    tgid: GroupId,
    msg: RecvType,
    global: &Arc<Global>,
) -> Result<HandleResult> {
    debug!("TODO GOT LAYER MESSAGE: ====== {} -> {} ===== ", fgid, tgid);
    match (fgid, tgid) {
        (GROUP_CHAT_ID, 0) | (0, GROUP_CHAT_ID) => group::handle(msg, global).await,
        (DOMAIN_ID, 0) | (0, DOMAIN_ID) => domain::handle(msg, global).await,
        (CLOUD_ID, 0) | (0, CLOUD_ID) => cloud::handle(msg, global).await,
        (DAO_ID, 0) | (0, DAO_ID) => cloud::handle(msg, global).await, // TODO DAO
        _ => match msg {
            RecvType::Leave(peer) => {
                debug!("Peer leaved: {}", peer.id.to_hex());
                let mut results = HandleResult::new();
                let mut layer = global.layer.write().await;

                let mut delete: HashMap<GroupChatId, Vec<usize>> = HashMap::new();
                let pid = global.pid().await;
                let db_key = global.own.read().await.db_key(&pid)?;
                let db = group_db(&global.base, &pid, &db_key)?;

                for (gid, session) in &layer.groups {
                    for (index, addr) in session.addrs.iter().enumerate() {
                        if addr == &peer.id {
                            delete
                                .entry(*gid)
                                .and_modify(|f| f.push(index))
                                .or_insert(vec![index]);
                            if index == 0 {
                                results.rpcs.push(session_lost(&session.s_id));
                            } else {
                                if let Ok(mid) = group::Member::get_id(&db, &session.db_id, addr) {
                                    results
                                        .rpcs
                                        .push(group::rpc::member_offline(session.db_id, mid));
                                }
                            }
                        }
                    }
                }

                for (gid, mut indexs) in delete {
                    if indexs[0] == 0 {
                        let _ = layer.groups.remove(&gid);
                    } else {
                        indexs.reverse();
                        for i in indexs {
                            let _ = layer.group_del_member(&gid, i);
                        }
                    }
                }

                Ok(results)
            }
            _ => {
                warn!("LAYER MISSING: {:?}", msg);
                Err(anyhow!("nothing!"))
            }
        },
    }
}
