use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::{RecvType, SendType},
    primitive::{HandleResult, Peer, PeerId, Result},
};
use tokio::sync::RwLock;

use group_types::{Event, LayerConnect, LayerEvent, LayerResult};
use tdn_did::Proof;

use crate::apps::chat::Friend;
use crate::layer::{Layer, Online};
use crate::rpc::{session_close, session_connect, session_last, session_lost, session_suspend};
use crate::session::{connect_session, Session, SessionType};
use crate::storage::{chat_db, delete_avatar, group_db, session_db, write_avatar_sync};

use super::models::{from_network_message, GroupChat, Member};
use super::{add_layer, add_server_layer, rpc};

// variable statement:
// gcd: Group Chat ID.
// fgid: where is event come from.
// ogid: my account ID. if server is group owner. if client is my.
// mgid: member account ID.
// id: Group Chat database Id.
// mid: member database Id.
pub(crate) async fn handle_server(
    layer: &Arc<RwLock<Layer>>,
    fgid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Connect(addr, data) => {
            let LayerConnect(gcd, _proof) = bincode::deserialize(&data)?;

            if handle_server_connect(layer, gcd, fgid, &addr, &mut results)
                .await
                .is_err()
            {
                let data = bincode::serialize(&gcd)?;
                let s = SendType::Result(0, addr, false, false, data);
                add_server_layer(&mut results, fgid, s);
            }
        }
        RecvType::Leave(_addr) => {
            // only server handle it. IMPORTANT !!! fgid IS mgid.
            // TODO
        }
        RecvType::Event(addr, bytes) => {
            println!("----------- DEBUG GROUP CHAT: SERVER GOT LAYER EVENT");
            let event: LayerEvent = bincode::deserialize(&bytes)?;
            handle_server_event(fgid, addr, event, layer, &mut results).await?;
            println!("----------- DEBUG GROUP CHAT: SERVER OVER LAYER EVENT");
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }
        RecvType::Result(..) => {}
        RecvType::ResultConnect(..) => {}
        RecvType::Delivery(..) => {}
    }

    Ok(results)
}

async fn handle_server_connect(
    layer: &Arc<RwLock<Layer>>,
    gcd: GroupId,
    fgid: GroupId,
    addr: &Peer,
    results: &mut HandleResult,
) -> Result<()> {
    let (ogid, height, id) = layer.read().await.running(&gcd)?.owner_height_id();
    // check is member.
    let db = group_db(&layer.read().await.base, &ogid)?;
    let mdid = Member::get_id(&db, &id, &fgid)?;

    let res = LayerResult(gcd, height);
    let data = bincode::serialize(&res).unwrap_or(vec![]);
    let s = SendType::Result(0, addr.clone(), true, false, data);
    add_server_layer(results, fgid, s);

    layer.write().await.running_mut(&gcd)?.check_add_online(
        fgid,
        Online::Direct(addr.id),
        id,
        mdid,
    )?;

    let _ = Member::addr_update(&db, &id, &fgid, &addr.id);
    results
        .rpcs
        .push(rpc::member_online(ogid, id, fgid, addr.id));

    let new_data = bincode::serialize(&LayerEvent::MemberOnline(gcd, fgid, addr.id))?;

    for (mid, maddr) in layer.read().await.running(&gcd)?.onlines() {
        let s = SendType::Event(0, *maddr, new_data.clone());
        add_server_layer(results, *mid, s);
    }
    Ok(())
}

pub(crate) async fn handle_peer(
    layer: &Arc<RwLock<Layer>>,
    ogid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Result(addr, is_ok, data) => {
            if is_ok {
                let mut layer_lock = layer.write().await;
                handle_connect(ogid, &addr, data, &mut layer_lock, &mut results)?;
            } else {
                // close the group chat.
                let gcd: GroupId = bincode::deserialize(&data)?;
                let base = layer.read().await.base().clone();
                let db = group_db(&base, &ogid)?;
                let group = GroupChat::close(&db, &gcd)?;
                let sid =
                    Session::close(&session_db(&base, &ogid)?, &group.id, &SessionType::Group)?;
                results.rpcs.push(session_close(ogid, &sid));
            }
        }
        RecvType::ResultConnect(addr, data) => {
            let mut layer_lock = layer.write().await;
            if handle_connect(ogid, &addr, data, &mut layer_lock, &mut results).is_err() {
                let msg = SendType::Result(0, addr, true, false, vec![]);
                add_layer(&mut results, ogid, msg);
            }
        }
        RecvType::Event(addr, bytes) => {
            println!("----------- DEBUG GROUP CHAT: PEER GOT LAYER EVENT");
            let event: LayerEvent = bincode::deserialize(&bytes)?;
            handle_peer_event(ogid, addr, event, layer, &mut results).await?;
            println!("----------- DEBUG GROUP CHAT: PEER OVER LAYER EVENT");
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {
            // TODO
        }
        _ => {
            error!("group chat peer handle layer nerver here")
        }
    }

    Ok(results)
}

fn handle_connect(
    ogid: GroupId,
    addr: &Peer,
    data: Vec<u8>,
    layer: &mut Layer,
    results: &mut HandleResult,
) -> Result<()> {
    // 0. deserialize result.
    let LayerResult(gcd, height) = bincode::deserialize(&data)?;

    // 1. check group.
    let db = group_db(layer.base(), &ogid)?;
    let group = GroupChat::get_id(&db, &gcd)?;

    // 1.0 check address.
    if group.g_addr != addr.id {
        return Err(anyhow!("invalid group chat address."));
    }

    // 1.1 get session.
    let session_some = connect_session(
        layer.base(),
        &ogid,
        &SessionType::Group,
        &group.id,
        &addr.id,
    )?;
    if session_some.is_none() {
        return Err(anyhow!("invalid group chat address."));
    }
    let sid = session_some.unwrap().id;

    // 1.2 online this group.
    layer
        .running_mut(&ogid)?
        .check_add_online(gcd, Online::Direct(addr.id), sid, group.id)?;

    // 1.3 online to UI.
    results.rpcs.push(session_connect(ogid, &sid, &addr.id));

    println!("will sync remote: {}, my: {}", height, group.height);
    // 1.4 sync group height.
    if group.height < height {
        add_layer(results, ogid, sync(gcd, addr.id, group.height));
    } else {
        // sync online members.
        add_layer(results, ogid, sync_online(gcd, addr.id));
    }

    Ok(())
}

// variable statement:
// gcd: Group Chat ID.
// fgid: where is event come from.
// ogid: my account ID. if server is group owner. if client is my.
// mgid: member account ID.
// id: Group Chat database Id.
// mdid: member database Id.
// sid: session Id.
async fn handle_server_event(
    fgid: GroupId,
    addr: PeerId,
    event: LayerEvent,
    layer: &Arc<RwLock<Layer>>,
    results: &mut HandleResult,
) -> Result<()> {
    let gcd = event.gcd();
    let base = layer.read().await.base().clone();
    let (ogid, height, id) = layer.read().await.running(gcd)?.owner_height_id();
    let db = group_db(&base, &ogid)?;

    match event {
        LayerEvent::Offline(gcd) => {
            // 1. check member online.
            if layer.write().await.remove_online(&gcd, &fgid).is_none() {
                return Ok(());
            }

            // 2. UI: offline the member.
            results.rpcs.push(rpc::member_offline(ogid, id, fgid));

            // 3. broadcast offline event.
            broadcast(&LayerEvent::MemberOffline(gcd, fgid), layer, &gcd, results).await?;
        }
        LayerEvent::Sync(gcd, _, event) => {
            match event {
                Event::MemberJoin(mgid, maddr, mname, mavatar) => {
                    let mdid_res = Member::get_id(&db, &id, &mgid);
                    let h = layer.write().await.running_mut(&gcd)?.increased();
                    let new_e = Event::MemberJoin(mgid, maddr, mname.clone(), mavatar.clone());

                    if let Ok(mdid) = mdid_res {
                        Member::update(&db, &h, &mdid, &maddr, &mname)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
                        }
                        results
                            .rpcs
                            .push(rpc::member_info(ogid, id, mdid, maddr, mname));
                    } else {
                        let mut member = Member::new(h, id, mgid, maddr, mname);
                        member.insert(&db)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
                        }
                        results.rpcs.push(rpc::member_join(ogid, &member));
                    }

                    // broadcast
                    GroupChat::add_height(&db, id, h)?;
                    broadcast(&LayerEvent::Sync(gcd, h, new_e), layer, &gcd, results).await?;
                }
                Event::MemberLeave(mgid) => {
                    let mdid = Member::get_id(&db, &id, &mgid)?;
                    let h = layer.write().await.running_mut(&gcd)?.increased();
                    Member::leave(&db, &mdid, &h)?;

                    // check mid is my chat friend. if not, delete avatar.
                    let s_db = chat_db(&base, &mgid)?;
                    if Friend::get(&s_db, &mgid)?.is_none() {
                        let _ = delete_avatar(&base, &ogid, &mgid).await;
                    }
                    results.rpcs.push(rpc::member_leave(ogid, id, mdid));

                    // broadcast
                    GroupChat::add_height(&db, id, h)?;
                    broadcast(&LayerEvent::Sync(gcd, h, event), layer, &gcd, results).await?;
                }
                Event::MessageCreate(mgid, nmsg, mtime) => {
                    println!("Sync: create message start");
                    let mdid = Member::get_id(&db, &id, &mgid)?;

                    let new_e = Event::MessageCreate(mgid, nmsg.clone(), mtime);
                    let new_h = layer.write().await.running_mut(&gcd)?.increased();
                    broadcast(&LayerEvent::Sync(gcd, new_h, new_e), layer, &gcd, results).await?;
                    GroupChat::add_height(&db, id, new_h)?;

                    let (msg, scontent) =
                        from_network_message(new_h, id, mgid, &ogid, nmsg, mtime, &base)?;
                    results.rpcs.push(rpc::message_create(ogid, &msg));
                    println!("Sync: create message ok");

                    // UPDATE SESSION.
                    let s_db = session_db(&base, &ogid)?;
                    if let Ok(sid) = Session::last(
                        &s_db,
                        &id,
                        &SessionType::Group,
                        &msg.datetime,
                        &scontent,
                        true,
                    ) {
                        results.rpcs.push(session_last(
                            ogid,
                            &sid,
                            &msg.datetime,
                            &scontent,
                            false,
                        ));
                    }
                }
                _ => error!("group server handle close nerver here"),
            }
        }
        LayerEvent::MemberOnlineSync(gcd) => {
            let onlines = layer
                .read()
                .await
                .running(&gcd)?
                .onlines()
                .iter()
                .map(|(g, a)| (**g, **a))
                .collect();
            let event = LayerEvent::MemberOnlineSyncResult(gcd, onlines);
            let data = bincode::serialize(&event).unwrap_or(vec![]);
            let s = SendType::Event(0, addr, data);
            add_server_layer(results, fgid, s);
        }
        LayerEvent::SyncReq(gcd, from) => {
            println!("Got sync request. height: {} from: {}", height, from);

            if height >= from {
                let to = if height - from > 100 {
                    from + 100
                } else {
                    height
                };

                // TODO

                // let packed = Consensus::pack(&db, &base, &gcd, &id, &from, &to).await?;
                // let event = LayerEvent::Packed(gcd, height, from, to, packed);

                //let packed_members = vec![];
                //let packed_messages = vec![];

                //let data = bincode::serialize(&event).unwrap_or(vec![]);
                //let s = SendType::Event(0, addr, data);
                //add_server_layer(results, fgid, s);
                //println!("Sended sync request results. from: {}, to: {}", from, to);
            }
        }
        LayerEvent::Suspend(..) => {}
        LayerEvent::Actived(..) => {}
        _ => error!("group server handle event nerver here"),
    }

    Ok(())
}

// variable statement:
// gcd: Group Chat ID.
// fgid: where is event come from.
// ogid: my account ID. if server is group owner. if client is my.
// mgid: member account ID.
// id: Group Chat database Id.
// mdid: member database Id.
// sid: session Id.
async fn handle_peer_event(
    ogid: GroupId,
    addr: PeerId,
    event: LayerEvent,
    layer: &Arc<RwLock<Layer>>,
    results: &mut HandleResult,
) -> Result<()> {
    let base = layer.read().await.base().clone();
    let gcd = event.gcd();
    let (sid, id) = layer.read().await.get_running_remote_id(&ogid, gcd)?;
    let db = group_db(&base, &ogid)?;

    match event {
        LayerEvent::Offline(gcd) => {
            // 1. offline group chat.
            layer
                .write()
                .await
                .running_mut(&ogid)?
                .check_offline(&gcd, &addr);

            // 2. UI: offline the session.
            results.rpcs.push(session_lost(ogid, &sid));
        }
        LayerEvent::Suspend(gcd) => {
            if layer
                .write()
                .await
                .running_mut(&ogid)?
                .suspend(&gcd, false, true)?
            {
                results.rpcs.push(session_suspend(ogid, &sid));
            }
        }
        LayerEvent::Actived(gcd) => {
            let _ = layer.write().await.running_mut(&ogid)?.active(&gcd, false);
            results.rpcs.push(session_connect(ogid, &sid, &addr));
        }
        LayerEvent::MemberOnline(_gcd, mgid, maddr) => {
            let _ = Member::addr_update(&db, &id, &mgid, &maddr);
            results.rpcs.push(rpc::member_online(ogid, id, mgid, maddr));
        }
        LayerEvent::MemberOffline(_gcd, mgid) => {
            results.rpcs.push(rpc::member_offline(ogid, id, mgid));
        }
        LayerEvent::MemberOnlineSyncResult(_gcd, onlines) => {
            for (mgid, maddr) in onlines {
                results.rpcs.push(rpc::member_online(ogid, id, mgid, maddr));
            }
        }
        LayerEvent::Sync(gcd, height, event) => {
            println!("Sync: handle height: {}", height);

            match event {
                Event::GroupTransfer(addr) => {
                    // TOOD transfer.
                }
                Event::GroupClose => {
                    let group = GroupChat::close(&db, &gcd)?;
                    let sid =
                        Session::close(&session_db(&base, &ogid)?, &group.id, &SessionType::Group)?;
                    results.rpcs.push(session_close(ogid, &sid));
                }
                Event::MemberJoin(mgid, maddr, mname, mavatar) => {
                    let mdid_res = Member::get_id(&db, &id, &mgid);
                    if let Ok(mdid) = mdid_res {
                        Member::update(&db, &height, &mdid, &maddr, &mname)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
                        }
                        results
                            .rpcs
                            .push(rpc::member_info(ogid, id, mdid, maddr, mname));
                    } else {
                        let mut member = Member::new(height, id, mgid, maddr, mname);
                        member.insert(&db)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
                        }
                        results.rpcs.push(rpc::member_join(ogid, &member));
                    }

                    // save consensus.
                    GroupChat::add_height(&db, id, height)?;
                }
                Event::MemberLeave(mgid) => {
                    let mdid = Member::get_id(&db, &id, &mgid)?;
                    Member::leave(&db, &height, &mdid)?;

                    // check mid is my chat friend. if not, delete avatar.
                    let s_db = chat_db(&base, &mgid)?;
                    if Friend::get(&s_db, &mgid)?.is_none() {
                        let _ = delete_avatar(&base, &ogid, &mgid).await;
                    }
                    results.rpcs.push(rpc::member_leave(ogid, id, mdid));

                    // save consensus.
                    GroupChat::add_height(&db, id, height)?;
                }
                Event::MessageCreate(mgid, nmsg, mtime) => {
                    println!("Sync: create message start");
                    let mdid = Member::get_id(&db, &id, &mgid)?;

                    let (msg, scontent) =
                        from_network_message(height, id, mgid, &ogid, nmsg, mtime, &base)?;
                    results.rpcs.push(rpc::message_create(ogid, &msg));

                    GroupChat::add_height(&db, id, height)?;
                    println!("Sync: create message ok");

                    // UPDATE SESSION.
                    let s_db = session_db(&base, &ogid)?;
                    if let Ok(sid) = Session::last(
                        &s_db,
                        &id,
                        &SessionType::Group,
                        &msg.datetime,
                        &scontent,
                        true,
                    ) {
                        results.rpcs.push(session_last(
                            ogid,
                            &sid,
                            &msg.datetime,
                            &scontent,
                            false,
                        ));
                    }
                }
            }
        }
        LayerEvent::SyncMember(gcd, height, from, to, adds, leaves) => {
            println!("Start handle sync packed... {}, {}, {}", height, from, to);
            // TODO
            // handle_sync(&db, ogid, id, gcd, addr, height, from, to, events, base, results)?;
            // update or leave.
        }
        LayerEvent::SyncMessage(gcd, height, mut from, to, adds) => {
            if to >= height {
                // when last packed sync, start sync online members.
                add_layer(results, ogid, sync_online(gcd, addr));
            }

            println!("Start handle sync packed... {}, {}, {}", height, from, to);
            let mut last_scontent = String::new();
            let mut last_time = 0;

            for (height, mgid, nm, time) in adds {
                let (msg, scontent) =
                    from_network_message(height, id, mgid, &ogid, nm, time, &base)?;
                results.rpcs.push(rpc::message_create(ogid, &msg));
                last_scontent = scontent;
                last_time = time;
                from += 1;
            }

            if to < height {
                add_layer(results, ogid, sync(gcd, addr, to + 1));
            }

            // update group chat height.
            GroupChat::add_height(&db, id, to)?;

            // UPDATE SESSION.
            if last_time > 1 {
                let s_db = session_db(&base, &ogid)?;
                if let Ok(sid) = Session::last(
                    &s_db,
                    &id,
                    &SessionType::Group,
                    &last_time,
                    &last_scontent,
                    true,
                ) {
                    results
                        .rpcs
                        .push(session_last(ogid, &sid, &last_time, &last_scontent, false));
                }
            }
        }
        _ => error!("group peer handle event nerver here"),
    }

    Ok(())
}

async fn broadcast(
    event: &LayerEvent,
    layer: &Arc<RwLock<Layer>>,
    gcd: &GroupId,
    results: &mut HandleResult,
) -> Result<()> {
    let new_data = bincode::serialize(&event)?;

    for (mgid, maddr) in layer.read().await.running(&gcd)?.onlines() {
        let s = SendType::Event(0, *maddr, new_data.clone());
        add_server_layer(results, *mgid, s);
        println!("--- DEBUG broadcast to: {:?}", mgid);
    }

    Ok(())
}

pub(crate) fn group_conn(proof: Proof, addr: Peer, gid: GroupId) -> SendType {
    let data = bincode::serialize(&LayerConnect(gid, proof)).unwrap_or(vec![]);
    SendType::Connect(0, addr, data)
}

fn sync(gcd: GroupId, addr: PeerId, height: i64) -> SendType {
    println!("Send sync request...");
    let data = bincode::serialize(&LayerEvent::SyncReq(gcd, height + 1)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}

fn sync_online(gcd: GroupId, addr: PeerId) -> SendType {
    let data = bincode::serialize(&LayerEvent::MemberOnlineSync(gcd)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}
