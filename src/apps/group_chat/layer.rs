use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tdn::types::{
    group::GroupId,
    message::{RecvType, SendType},
    primitive::{new_io_error, HandleResult, PeerAddr, Result},
};
use tokio::sync::RwLock;

use group_chat_types::{
    ConnectProof, Event, GroupType, JoinProof, LayerConnect, LayerEvent, LayerResult, PackedEvent,
};
use tdn_did::Proof;
use tdn_storage::local::DStorage;

use crate::apps::chat::Friend;
use crate::layer::{Layer, Online};
use crate::rpc::{session_connect, session_create, session_last, session_lost, session_suspend};
use crate::session::{connect_session, Session, SessionType};
use crate::storage::{
    chat_db, delete_avatar, group_chat_db, read_avatar, session_db, write_avatar, write_avatar_sync,
};

use super::models::{from_network_message, GroupChat, Member, Request};
use super::{add_layer, add_server_layer, rpc};

pub(crate) async fn handle(
    layer: &Arc<RwLock<Layer>>,
    fgid: GroupId, // when as client, `fgid` is GROUP_ID
    mgid: GroupId, // when as server, `mgid` is GROUP_ID
    is_server: bool,
    msg: RecvType,
) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Connect(addr, data) => {
            // only server handle it.
            if !is_server {
                let s = SendType::Result(0, addr, false, false, vec![]);
                add_server_layer(&mut results, fgid, s);
                return Ok(results);
            }

            let LayerConnect(gcd, connect) = bincode::deserialize(&data)
                .map_err(|_e| new_io_error("deserialize group chat connect failure"))?;

            let (ogid, height, id) = layer.read().await.running(&gcd)?.owner_height_id();

            match connect {
                ConnectProof::Common(_proof) => {
                    // check is member.
                    let db = group_chat_db(&layer.read().await.base, &ogid)?;

                    if let Ok(mid) = Member::get_id(&db, &id, &fgid) {
                        let res = LayerResult(gcd, height);
                        let data = bincode::serialize(&res).unwrap_or(vec![]);
                        let s = SendType::Result(0, addr, true, false, data);
                        add_server_layer(&mut results, fgid, s);

                        layer.write().await.running_mut(&gcd)?.check_add_online(
                            mgid,
                            Online::Direct(addr),
                            id,
                            mid,
                        )?;

                        let _ = Member::addr_update(&db, &id, &fgid, &addr);
                        results.rpcs.push(rpc::member_online(mgid, id, fgid, addr));

                        let new_data =
                            bincode::serialize(&LayerEvent::MemberOnline(gcd, fgid, addr))
                                .map_err(|_| new_io_error("serialize event error."))?;

                        for (mid, maddr) in layer.read().await.running(&gcd)?.onlines() {
                            let s = SendType::Event(0, *maddr, new_data.clone());
                            add_server_layer(&mut results, *mid, s);
                        }
                    } else {
                        let s = SendType::Result(0, addr, false, false, vec![]);
                        add_server_layer(&mut results, fgid, s);
                    }
                }
                ConnectProof::Zkp(_proof) => {
                    //
                }
            }
        }
        RecvType::Leave(_addr) => {
            // only server handle it.
            // TODO
        }
        RecvType::Result(addr, is_ok, data) => {
            // only client handle it.
            if !is_server && is_ok {
                let mut layer_lock = layer.write().await;
                handle_connect(mgid, addr, data, &mut layer_lock, &mut results)?;
            } else {
                let msg = SendType::Result(0, addr, false, false, vec![]);
                add_layer(&mut results, mgid, msg);
            }
        }
        RecvType::ResultConnect(addr, data) => {
            // only client handle it.
            if is_server {
                let msg = SendType::Result(0, addr, false, false, vec![]);
                add_layer(&mut results, mgid, msg);
            }

            let mut layer_lock = layer.write().await;
            if handle_connect(mgid, addr, data, &mut layer_lock, &mut results)? {
                let msg = SendType::Result(0, addr, true, false, vec![]);
                add_layer(&mut results, mgid, msg);
            }
        }
        RecvType::Event(addr, bytes) => {
            // server & client handle it.
            let event: LayerEvent =
                bincode::deserialize(&bytes).map_err(|_| new_io_error("serialize event error."))?;
            handle_event(fgid, mgid, is_server, addr, event, layer, &mut results).await?;
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

fn handle_connect(
    mgid: GroupId,
    addr: PeerAddr,
    data: Vec<u8>,
    layer: &mut Layer,
    results: &mut HandleResult,
) -> Result<bool> {
    // 0. deserialize result.
    let LayerResult(gcd, height) =
        bincode::deserialize(&data).map_err(|_e| new_io_error("Deseralize result failure"))?;

    // 1. check group.
    if let Some(group) = load_group(layer.base(), &mgid, &gcd)? {
        // 1.0 check address.
        if group.g_addr != addr {
            return Ok(false);
        }

        // 1.1 get session.
        let session_some =
            connect_session(layer.base(), &mgid, &SessionType::Group, &group.id, &addr)?;
        if session_some.is_none() {
            return Ok(false);
        }
        let sid = session_some.unwrap().id;

        // 1.2 online this group.
        layer
            .running_mut(&mgid)?
            .check_add_online(gcd, Online::Direct(addr), sid, group.id)?;

        // 1.3 online to UI.
        results.rpcs.push(session_connect(mgid, &sid, &addr));

        println!("will sync remote: {}, my: {}", height, group.height);
        // 1.4 sync group height.
        if group.height < height {
            add_layer(results, mgid, sync(gcd, addr, group.height));
        } else {
            // sync online members.
            add_layer(results, mgid, sync_online(gcd, addr));
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn handle_event(
    fgid: GroupId, // server use fgid is remote account.
    mgid: GroupId, // client user mgid is my account.
    is_server: bool,
    addr: PeerAddr,
    event: LayerEvent,
    layer: &Arc<RwLock<Layer>>,
    results: &mut HandleResult,
) -> Result<()> {
    println!("Got event.......");
    match event {
        LayerEvent::Offline(gcd) => {
            if is_server {
                // 1. check member online.
                if layer.write().await.remove_online(&gcd, &fgid).is_none() {
                    return Ok(());
                }

                // 2. offline this member.
                let (ogid, _, id) = layer.read().await.running(&gcd)?.owner_height_id();
                results.rpcs.push(rpc::member_offline(ogid, id, fgid));

                // 3. broadcast offline event.
                let new_data = bincode::serialize(&LayerEvent::MemberOffline(gcd, fgid))
                    .map_err(|_| new_io_error("serialize event error."))?;

                for (mid, maddr) in layer.read().await.running(&gcd)?.onlines() {
                    let s = SendType::Event(0, *maddr, new_data.clone());
                    add_layer(results, *mid, s);
                }
            } else {
                let mut layer_lock = layer.write().await;
                let (sid, _gid) = layer_lock.get_running_remote_id(&mgid, &gcd)?;
                layer_lock.running_mut(&mgid)?.check_offline(&gcd, &addr);
                drop(layer_lock);
                results.rpcs.push(session_lost(mgid, &sid));
            }
        }
        LayerEvent::Suspend(gcd) => {
            let mut layer_lock = layer.write().await;
            let (sid, _gid) = layer_lock.get_running_remote_id(&mgid, &gcd)?;
            if layer_lock.running_mut(&mgid)?.suspend(&gcd, false, true)? {
                results.rpcs.push(session_suspend(mgid, &sid));
            }
            drop(layer_lock);
        }
        LayerEvent::Actived(gcd) => {
            let mut layer_lock = layer.write().await;
            let (sid, _gid) = layer_lock.get_running_remote_id(&mgid, &gcd)?;
            let _ = layer_lock.running_mut(&mgid)?.active(&gcd, false);
            drop(layer_lock);
            results.rpcs.push(session_connect(mgid, &sid, &addr));
        }
        LayerEvent::CheckResult(ct, supported) => {
            println!("check: {:?}, supported: {:?}", ct, supported);
            results.rpcs.push(rpc::create_check(mgid, ct, supported))
        }
        LayerEvent::CreateResult(gcd, ok) => {
            println!("Create result: {}", ok);
            if ok {
                // get gc by gcd.
                let db = group_chat_db(layer.read().await.base(), &mgid)?;
                if let Some(mut gc) = GroupChat::get(&db, &gcd)? {
                    gc.ok(&db)?;
                    results.rpcs.push(rpc::create_result(mgid, gc.id, ok));

                    // ADD NEW SESSION.
                    let s_db = session_db(layer.read().await.base(), &mgid)?;
                    let mut session = gc.to_session();
                    session.insert(&s_db)?;
                    results.rpcs.push(session_create(mgid, &session));
                }
            }
        }
        LayerEvent::Agree(gcd, info) => {
            println!("Agree..........");
            let base = layer.read().await.base.clone();
            let db = group_chat_db(&base, &mgid)?;
            let (rid, key) = Request::over(&db, &gcd, true)?;

            // 1. add group chat.
            let mut group = GroupChat::from_info(key, info, 0, addr, &base, &mgid, true)?;
            group.insert(&db)?;

            // 2. ADD NEW SESSION.
            let s_db = session_db(&base, &mgid)?;
            let mut session = group.to_session();
            session.insert(&s_db)?;
            results.rpcs.push(session_create(mgid, &session));

            // 3. update UI.
            results
                .rpcs
                .push(rpc::request_handle(mgid, rid, true, false));
            results.rpcs.push(rpc::group_create(mgid, group));

            // 4. try connect.
            let proof = layer
                .read()
                .await
                .group
                .read()
                .await
                .prove_addr(&mgid, &addr)?;
            add_layer(results, mgid, group_chat_conn(proof, addr, gcd));
        }
        LayerEvent::Reject(gcd, efficacy) => {
            println!("Reject..........");
            let db = group_chat_db(layer.read().await.base(), &mgid)?;
            let (rid, _key) = Request::over(&db, &gcd, true)?;
            results
                .rpcs
                .push(rpc::request_handle(mgid, rid, false, efficacy));
        }
        LayerEvent::MemberOnline(gcd, mid, maddr) => {
            let (_sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;
            let db = group_chat_db(layer.read().await.base(), &mgid)?;
            let _ = Member::addr_update(&db, &gid, &mid, &maddr);
            results.rpcs.push(rpc::member_online(mgid, gid, mid, maddr));
        }
        LayerEvent::MemberOffline(gcd, mid) => {
            let (_sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;
            results.rpcs.push(rpc::member_offline(mgid, gid, mid));
        }
        LayerEvent::MemberOnlineSyncResult(gcd, onlines) => {
            let (_sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;
            for (mid, maddr) in onlines {
                results.rpcs.push(rpc::member_online(mgid, gid, mid, maddr));
            }
        }
        LayerEvent::Sync(gcd, height, event) => {
            let (_sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;

            println!("Sync: height: {}", height);
            let base = layer.read().await.base().clone();
            let db = group_chat_db(&base, &mgid)?;

            match event {
                Event::GroupInfo => {}
                Event::GroupTransfer => {}
                Event::GroupManagerAdd => {}
                Event::GroupManagerDel => {}
                Event::GroupClose => {}
                Event::MemberInfo(mid, maddr, mname, mavatar) => {
                    let id = Member::get_id(&db, &gid, &mid)?;
                    Member::update(&db, &id, &maddr, &mname)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&base, &mgid, &mid, mavatar)?;
                    }
                    results.rpcs.push(rpc::member_info(mgid, id, maddr, mname));
                }
                Event::MemberJoin(mid, maddr, mname, mavatar, mtime) => {
                    if Member::get_id(&db, &gid, &mid).is_err() {
                        let mut member = Member::new(gid, mid, maddr, mname, false, mtime);
                        member.insert(&db)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&base, &mgid, &mid, mavatar)?;
                        }
                        results.rpcs.push(rpc::member_join(mgid, member));
                    }
                }
                Event::MemberLeave(mid) => {
                    let id = Member::get_id(&db, &gid, &mid)?;
                    Member::leave(&db, &id)?;
                    // check mid is my chat friend. if not, delete avatar.
                    let s_db = chat_db(&base, &mgid)?;
                    if Friend::get(&s_db, &mid)?.is_none() {
                        let _ = delete_avatar(&base, &mgid, &mid).await;
                    }
                    results.rpcs.push(rpc::member_leave(mgid, id));
                }
                Event::MessageCreate(mid, nmsg, mtime) => {
                    println!("Sync: create message start");
                    let base = layer.read().await.base.clone();
                    let (msg, scontent) =
                        from_network_message(height, gid, mid, &mgid, nmsg, mtime, &base)?;
                    results.rpcs.push(rpc::message_create(mgid, &msg));
                    println!("Sync: create message ok");

                    // UPDATE SESSION.
                    let s_db = session_db(&base, &mgid)?;
                    if let Ok(id) = Session::last(
                        &s_db,
                        &gid,
                        &SessionType::Group,
                        &msg.datetime,
                        &scontent,
                        true,
                    ) {
                        results
                            .rpcs
                            .push(session_last(mgid, &id, &msg.datetime, &scontent, false));
                    }
                }
            }

            // save event.
            GroupChat::add_height(&db, gid, height)?;
        }
        LayerEvent::Packed(gcd, height, from, to, events) => {
            let (_sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;

            if to >= height {
                // when last packed sync, start sync online members.
                add_layer(results, mgid, sync_online(gcd, addr));
            }

            println!("Start handle sync packed... {}, {}, {}", height, from, to);
            let base = layer.read().await.base().clone();
            handle_sync(
                mgid, gid, gcd, addr, height, from, to, events, base, results,
            )?;
        }
        LayerEvent::RequestHandle(gcd, rgid, raddr, join_proof, rid, time) => {
            let (_sid, gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;

            match join_proof {
                JoinProof::Invite(i, _proof, mname, mavatar) => {
                    let mut req =
                        Request::new_by_remote(gid, rid, rgid, raddr, mname, i.to_hex(), time);
                    let base = layer.read().await.base().clone();
                    let db = group_chat_db(&base, &mgid)?;
                    req.insert(&db)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&base, &mgid, &rgid, mavatar)?;
                    }
                    results.rpcs.push(rpc::request_create(mgid, &req));
                }
                JoinProof::Zkp(_proof) => {
                    //
                }
                JoinProof::Open(..) => {} // nerver here.
            }
        }
        LayerEvent::RequestResult(gcd, rid, ok) => {
            let (_sid, _gid) = layer.read().await.get_running_remote_id(&mgid, &gcd)?;
            let db = group_chat_db(layer.read().await.base(), &mgid)?;
            let id = Request::over_rid(&db, &gcd, &rid, ok)?;
            results.rpcs.push(rpc::request_handle(mgid, id, ok, false));
        }
        LayerEvent::MemberOnlineSync(..) => {
            // TODO
        }
        LayerEvent::Request(gcd, join_proof) => {
            let (ogid, height, id) = layer.read().await.running(&gcd)?.owner_height_id();
            let base = layer.read().await.base.clone();
            let db = group_chat_db(&base, &ogid)?;
            let group = GroupChat::get_id(&db, &id)?.ok_or(new_io_error("missing group"))?;

            // 1. check account is online, if not online, nothing.
            match join_proof {
                JoinProof::Open(mname, mavatar) => {
                    // check is member.
                    if let Ok(mid) = Member::get_id(&db, &id, &fgid) {
                        let gavatar = read_avatar(&base, &ogid, &gcd).await?;
                        let group_info = group.to_group_info("".to_owned(), gavatar, vec![]);
                        let res = LayerEvent::Agree(gcd, group_info);
                        let d = bincode::serialize(&res).unwrap_or(vec![]);
                        let s = SendType::Event(0, addr, d);
                        add_server_layer(results, fgid, s);

                        return Ok(());
                    }

                    if group.g_type == GroupType::Open {
                        let start = SystemTime::now();
                        let datetime = start
                            .duration_since(UNIX_EPOCH)
                            .map(|s| s.as_secs())
                            .unwrap_or(0) as i64; // safe for all life.

                        let mut m = Member::new(id, fgid, addr, mname, false, datetime);
                        m.insert(&db)?;

                        // save avatar.
                        let _ = write_avatar(&base, &ogid, &fgid, &mavatar).await;

                        // add_height consensus.

                        // self.broadcast_join(&gcd, m, mavatar, results).await?;

                        // return join result.
                        let gavatar = read_avatar(&base, &ogid, &gcd).await?;
                        let group_info = group.to_group_info("".to_owned(), gavatar, vec![]);
                        let res = LayerEvent::Agree(gcd, group_info);
                        let d = bincode::serialize(&res).unwrap_or(vec![]);
                        let s = SendType::Event(0, addr, d);
                        add_server_layer(results, fgid, s);
                    } else {
                        // Self::reject(gcd, fmid, addr, false, results);
                    }
                }
                JoinProof::Invite(invite_gid, proof, mname, mavatar) => {
                    // check is member.
                    if let Ok(mid) = Member::get_id(&db, &id, &fgid) {
                        let gavatar = read_avatar(&base, &ogid, &gcd).await?;
                        let group_info = group.to_group_info("".to_owned(), gavatar, vec![]);
                        let res = LayerEvent::Agree(gcd, group_info);
                        let d = bincode::serialize(&res).unwrap_or(vec![]);
                        let s = SendType::Event(0, addr, d);
                        add_server_layer(results, fgid, s);
                        return Ok(());
                    }

                    // TODO check if request had or is blocked by manager.

                    // check if inviter is member.
                    if Member::get_id(&db, &id, &invite_gid).is_err() {
                        //Self::reject(gcd, fmid, addr, true, results);
                        return Ok(());
                    }

                    // TODO check proof.
                    // proof.verify(&invite_gid, &addr, &layer.addr)?;

                    // if group.is_need_agree {
                    //     if !Member::is_manager(fid, &invite_gid).await? {
                    //         let mut request = Request::new();
                    //         request.insert().await?;
                    //         self.broadcast_request(
                    //             &gcd,
                    //             request,
                    //             JoinProof::Invite(invite_gid, proof, mname, mavatar),
                    //             results,
                    //         );
                    //         return Ok(());
                    //     }
                    // }

                    //let mut m = Member::new(*fid, fmid, addr, mname, false);
                    //m.insert().await?;

                    // save avatar.
                    //let _ = write_avatar(&self.base, &gcd, &m.m_id, &mavatar).await;

                    //self.add_member(&gcd, fmid, addr);
                    //self.broadcast_join(&gcd, m, mavatar, results).await?;

                    // return join result.
                    //self.agree(gcd, fmid, addr, group, results).await?;
                }
                JoinProof::Zkp(_proof) => {
                    // TOOD zkp join.
                }
            }
        }
        LayerEvent::SyncReq(..) => {
            // TODO
        }
        LayerEvent::Check => {}      // nerver here.
        LayerEvent::Create(..) => {} // nerver here.
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
        bincode::serialize(&LayerConnect(gid, ConnectProof::Common(proof))).unwrap_or(vec![]);
    SendType::Connect(0, addr, None, None, data)
}

fn sync(gcd: GroupId, addr: PeerAddr, height: i64) -> SendType {
    println!("Send sync request...");
    let data = bincode::serialize(&LayerEvent::SyncReq(gcd, height + 1)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}

fn sync_online(gcd: GroupId, addr: PeerAddr) -> SendType {
    let data = bincode::serialize(&LayerEvent::MemberOnlineSync(gcd)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}

// fn broadcast_join(
//     gcd: &GroupId,
//     member: Member,
//     avatar: Vec<u8>,
//     results: &mut HandleResult,
// ) -> Result<()> {
//     println!("start broadcast join...");
//     let height = self
//         .add_height(gcd, &member.id, ConsensusType::MemberJoin)
//         .await?;

//     let datetime = member.datetime;
//     let event = Event::MemberJoin(
//         member.m_id,
//         member.m_addr,
//         member.m_name,
//         avatar,
//         member.datetime,
//     );

//     let new_data = bincode::serialize(&LayerEvent::Sync(*gcd, height, event)).unwrap_or(vec![]);

//     if let Some((members, _, _)) = self.groups.get(gcd) {
//         for (mid, maddr, _) in members {
//             let s = SendType::Event(0, *maddr, new_data.clone());
//             add_layer(results, *mid, s);
//         }
//     }
//     println!("over broadcast join...");

//     Ok(())
// }

fn handle_sync(
    mgid: GroupId,
    fid: i64,
    gcd: GroupId,
    addr: PeerAddr,
    height: i64,
    mut from: i64,
    to: i64,
    events: Vec<PackedEvent>,
    base: PathBuf,
    results: &mut HandleResult,
) -> Result<()> {
    let db = group_chat_db(&base, &mgid)?;

    let mut last_scontent: Option<(String, i64)> = None;

    for event in events {
        if let Ok(scontent) = handle_sync_event(&mgid, &fid, from, event, &base, &db, results) {
            last_scontent = scontent;
        }
        from += 1;
    }

    if to < height {
        add_layer(results, mgid, sync(gcd, addr, to + 1));
    }

    // update group chat height.
    GroupChat::add_height(&db, fid, to)?;

    // UPDATE SESSION.
    if let Some((sc, t)) = last_scontent {
        let s_db = session_db(&base, &mgid)?;
        if let Ok(id) = Session::last(&s_db, &fid, &SessionType::Group, &t, &sc, true) {
            results.rpcs.push(session_last(mgid, &id, &t, &sc, false));
        }
    }

    Ok(())
}

fn handle_sync_event(
    mgid: &GroupId,
    fid: &i64,
    height: i64,
    event: PackedEvent,
    base: &PathBuf,
    db: &DStorage,
    results: &mut HandleResult,
) -> Result<Option<(String, i64)>> {
    let scontent = match event {
        PackedEvent::GroupInfo => {
            // TODO
            None
        }
        PackedEvent::GroupTransfer => {
            // TODO
            None
        }
        PackedEvent::GroupManagerAdd => {
            // TODO
            None
        }
        PackedEvent::GroupManagerDel => {
            // TODO
            None
        }
        PackedEvent::GroupClose => {
            // TOOD
            None
        }
        PackedEvent::MemberInfo(_mid, _maddr, _mname, _mavatar) => {
            // TODO
            None
        }
        PackedEvent::MemberJoin(mid, maddr, mname, mavatar, mtime) => {
            if Member::get_id(db, fid, &mid).is_err() {
                if mavatar.len() > 0 {
                    write_avatar_sync(&base, &mgid, &mid, mavatar)?;
                }
                let mut member = Member::new(*fid, mid, maddr, mname, false, mtime);
                member.insert(&db)?;
                results.rpcs.push(rpc::member_join(*mgid, member));
            }
            None
        }
        PackedEvent::MemberLeave(_mid) => {
            // TODO
            None
        }
        PackedEvent::MessageCreate(mid, nmsg, time) => {
            let (msg, scontent) = from_network_message(height, *fid, mid, mgid, nmsg, time, base)?;
            results.rpcs.push(rpc::message_create(*mgid, &msg));
            Some((scontent, time))
        }
        PackedEvent::None => None,
    };

    Ok(scontent)
}
