use std::sync::Arc;
use tdn::{
    smol::lock::RwLock,
    types::{
        group::GroupId,
        message::RecvType,
        primitive::{new_io_error, HandleResult, Result},
    },
};

use group_chat_types::GroupResult;
//use group_chat_types::{Event, GroupConnect, GroupEvent, GroupInfo, GroupResult, GroupType};

use crate::layer::Layer;
use crate::storage::group_chat_db;

use super::models::GroupChat;
use super::rpc;

pub(crate) async fn handle(
    layer: &Arc<RwLock<Layer>>,
    mgid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Connect(_addr, _data) => {
            // None.
        }
        RecvType::Leave(_addr) => {
            //
        }
        RecvType::Result(_addr, _is_ok, data) => {
            let res: GroupResult = postcard::from_bytes(&data)
                .map_err(|_e| new_io_error("Deseralize result failure"))?;
            match res {
                GroupResult::Check(ct, supported) => {
                    println!("check: {:?}, supported: {:?}", ct, supported);
                    results.rpcs.push(rpc::create_check(mgid, ct, supported))
                }
                GroupResult::Create(gcd, ok) => {
                    println!("Create result: {}", ok);
                    if ok {
                        // TODO get gc by gcd.
                        let db = group_chat_db(layer.read().await.base(), &mgid)?;
                        if let Some(mut gc) = GroupChat::get(&db, &gcd)? {
                            gc.ok(&db)?;
                            results.rpcs.push(rpc::create_result(mgid, gc.id, ok))
                        }
                    }
                }
                _ => {
                    //
                }
            }
        }
        RecvType::ResultConnect(_addr, data) => {
            let _res: GroupResult = postcard::from_bytes(&data)
                .map_err(|_e| new_io_error("Deseralize result failure"))?;
        }
        RecvType::Event(_addr, _bytes) => {
            //
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {
            //
        }
    }

    Ok(results)
}
