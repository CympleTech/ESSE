use std::path::PathBuf;
use std::sync::Arc;
use tdn::{
    smol::lock::RwLock,
    types::{
        group::GroupId,
        message::{RecvType, SendType},
        primitive::{new_io_error, HandleResult, PeerAddr, Result},
    },
};

use group_chat_types::{Event, GroupConnect, GroupResult, JoinProof, LayerEvent};
use tdn_did::Proof;

use crate::layer::{Layer, Online};
use crate::storage::group_chat_db;

use super::models::GroupChat;
use super::{add_layer, rpc};

pub(crate) async fn handle(
    layer: &Arc<RwLock<Layer>>,
    mgid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Connect(..) => {} // Never to here.
        RecvType::Leave(_addr) => {
            //
        }
        RecvType::Result(addr, _is_ok, data) => {
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
                GroupResult::Join(gcd, ok, height) => {
                    println!("Got join result: {}", ok);
                    if ok {
                        let base = layer.read().await.base.clone();
                        if let Some(group) = load_group(&base, &mgid, &gcd)? {
                            let mut layer_lock = layer.write().await;
                            // 1. check address.
                            if group.g_addr != addr {
                                return Ok(results);
                            }
                            // 2. online this group.
                            layer_lock.running_mut(&mgid)?.check_add_online(
                                gcd,
                                Online::Direct(addr),
                                group.id,
                            )?;
                            // 3. online to UI.
                            results.rpcs.push(rpc::group_online(mgid, group.id));

                            // 5. sync group height.
                            let db = group_chat_db(&base, &mgid)?;
                            let my_height = GroupChat::get_height(&db, &group.id)?;
                            drop(db);

                            if my_height < height {
                                // TOOD
                            }
                        } else {
                            let msg = SendType::Result(0, addr, false, false, vec![]);
                            add_layer(&mut results, mgid, msg);
                            return Ok(results);
                        }
                    }
                }
                GroupResult::Waiting(_gcd) => {
                    // TODO waiting
                }
                GroupResult::Agree(_gcd, _group_info, _height) => {
                    // TOOD
                }
                GroupResult::Reject(_gcd) => {
                    // TOOD
                }
            }
        }
        RecvType::ResultConnect(_addr, data) => {
            let _res: GroupResult = postcard::from_bytes(&data)
                .map_err(|_e| new_io_error("Deseralize result failure"))?;
        }
        RecvType::Event(addr, bytes) => {
            let event: LayerEvent =
                postcard::from_bytes(&bytes).map_err(|_| new_io_error("serialize event error."))?;
            handle_event(mgid, addr, event, layer, &mut results).await?;
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

async fn handle_event(
    mgid: GroupId,
    addr: PeerAddr,
    event: LayerEvent,
    layer: &Arc<RwLock<Layer>>,
    results: &mut HandleResult,
) -> Result<()> {
    let gid = match event {
        LayerEvent::Offline(gcd)
        | LayerEvent::OnlinePing(gcd)
        | LayerEvent::OnlinePong(gcd)
        | LayerEvent::MemberOnline(gcd, ..)
        | LayerEvent::MemberOffline(gcd, ..)
        | LayerEvent::Sync(gcd, ..) => layer.read().await.get_running_remote_id(&mgid, &gcd)?,
    };

    match event {
        LayerEvent::Offline(_) => {
            results.rpcs.push(rpc::group_offline(mgid, gid));
        }
        LayerEvent::OnlinePing(gcd) => {
            results.rpcs.push(rpc::group_online(mgid, gid));
            let data = postcard::to_allocvec(&LayerEvent::OnlinePong(gcd)).unwrap_or(vec![]);
            let msg = SendType::Event(0, addr, data);
            add_layer(results, mgid, msg);
        }

        LayerEvent::OnlinePong(_) => {
            results.rpcs.push(rpc::group_online(mgid, gid));
        }
        LayerEvent::Sync(_gcd, _, event) => {
            match event {
                Event::Message => {
                    //
                }
                Event::GroupUpdate => {
                    //
                }
                Event::GroupTransfer => {
                    //
                }
                Event::UserInfo => {
                    //
                }
                Event::Close => {
                    //
                }
            }

            // save event.

            // update to UI.
        }
        LayerEvent::MemberOnline(_, mid, maddr) => {
            results.rpcs.push(rpc::member_online(mgid, gid, mid, maddr));
        }
        LayerEvent::MemberOffline(_, mid, ma) => {
            results.rpcs.push(rpc::member_offline(mgid, gid, mid, ma));
        }
    }

    Ok(())
}

#[inline]
fn load_group(base: &PathBuf, mgid: &GroupId, gcd: &GroupId) -> Result<Option<GroupChat>> {
    let db = group_chat_db(base, mgid)?;
    GroupChat::get(&db, gcd)
}

pub(crate) fn group_chat_conn(proof: Proof, addr: PeerAddr, gid: GroupId) -> SendType {
    let data =
        postcard::to_allocvec(&GroupConnect::Join(gid, JoinProof::Had(proof))).unwrap_or(vec![]);
    SendType::Connect(0, addr, None, None, data)
}
