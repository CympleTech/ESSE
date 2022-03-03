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
//pub(crate) mod jarvis;
//pub(crate) mod dao;
//pub(crate) mod wallet;

pub(crate) fn app_rpc_inject(handler: &mut RpcHandler<Global>) {
    //device::new_rpc_handler(handler);
    chat::new_rpc_handler(handler);
    //jarvis::new_rpc_handler(handler);
    //domain::new_rpc_handler(handler);
    //file::new_rpc_handler(handler);
    //group::new_rpc_handler(handler);
    //wallet::new_rpc_handler(handler);
    //dao::new_rpc_handler(handler);
    //cloud::new_rpc_handler(handler);
}

#[allow(non_snake_case)]
pub(crate) async fn app_layer_handle(
    fgid: GroupId,
    msg: RecvType,
    global: &Arc<Global>,
) -> Result<HandleResult> {
    match fgid {
        CHAT_ID => chat::handle(msg, global).await,
        //CHAT_ID => chat::handle_peer(layer, mgid, msg).await,
        //(_, group::GROUP_ID) => group::handle_server(layer, fgid, msg).await,
        //(dao::GROUP_ID, _) => dao::handle(layer, fgid, mgid, false, msg).await,
        //(domain::GROUP_ID, _) => domain::handle(layer, mgid, msg).await,
        //(cloud::GROUP_ID, _) => cloud::handle(layer, mgid, msg).await,
        //_ => chat::handle(layer, fgid, mgid, msg).await,
        _ => match msg {
            RecvType::Leave(peer) => {
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
            _ => Err(anyhow!("nothing!")),
        },
    }
}
