use chat_types::CHAT_ID;
use cloud_types::CLOUD_ID;
use dao_types::DAO_ID;
use domain_types::DOMAIN_ID;
use group_types::GROUP_CHAT_ID;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::RecvType,
    primitives::{HandleResult, Result},
    rpc::RpcHandler,
};

use crate::global::Global;
use crate::rpc::session_lost;

pub(crate) mod chat;
//pub(crate) mod cloud;
pub(crate) mod device;
//pub(crate) mod domain;
//pub(crate) mod file;
//pub(crate) mod group;
pub(crate) mod jarvis;
//pub(crate) mod dao;
//pub(crate) mod wallet;

pub(crate) fn app_rpc_inject(handler: &mut RpcHandler<Global>) {
    //device::new_rpc_handler(handler);
    chat::new_rpc_handler(handler);
    jarvis::new_rpc_handler(handler);
    //domain::new_rpc_handler(handler);
    //file::new_rpc_handler(handler);
    //group::new_rpc_handler(handler);
    //wallet::new_rpc_handler(handler);
    //dao::new_rpc_handler(handler);
    //cloud::new_rpc_handler(handler);
}

pub(crate) async fn app_layer_handle(
    fgid: GroupId,
    tgid: GroupId,
    msg: RecvType,
    global: &Arc<Global>,
) -> Result<HandleResult> {
    debug!("TODO GOT LAYER MESSAGE: ====== {} -> {} ===== ", fgid, tgid);
    match (fgid, tgid) {
        (CHAT_ID, 0) | (0, CHAT_ID) => chat::handle(msg, global).await,
        (GROUP_CHAT_ID, 0) => chat::handle(msg, global).await,
        (DAO_ID, 0) => chat::handle(msg, global).await,
        (DOMAIN_ID, 0) => chat::handle(msg, global).await,
        (CLOUD_ID, 0) => chat::handle(msg, global).await,
        _ => match msg {
            RecvType::Leave(peer) => {
                debug!("Peer leaved: {}", peer.id.to_hex());
                let mut results = HandleResult::new();
                let mut layer = global.layer.write().await;

                if let Some(session) = layer.chats.remove(&peer.id) {
                    results.rpcs.push(session_lost(&session.s_id));
                }

                let mut delete = vec![];
                for (gid, session) in &layer.groups {
                    if session.addr == peer.id {
                        delete.push(*gid);
                        results.rpcs.push(session_lost(&session.s_id));
                    }
                }

                for gid in delete {
                    let _ = layer.groups.remove(&gid);
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
