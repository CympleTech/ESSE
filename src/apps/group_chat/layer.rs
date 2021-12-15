use std::path::PathBuf;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::{RecvType, SendType},
    primitive::{HandleResult, Peer, PeerId, Result},
};
use tokio::sync::RwLock;

use group_chat_types::{
    CheckType, ConnectProof, Event, GroupType, JoinProof, LayerConnect, LayerEvent, LayerResult,
    PackedEvent,
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

use super::models::{
    from_network_message, Consensus, ConsensusType, GroupChat, Member, Provider, Request,
};
use super::{add_layer, add_server_layer, rpc};

// variable statement:
// gcd: Group Chat ID.
// fgid: where is event come from.
// ogid: my account ID. if server is group owner. if client is my.
// mgid: member account ID.
// id: Group Chat database Id.
// mid: member database Id.
pub(crate) async fn handle(
    layer: &Arc<RwLock<Layer>>,
    fgid: GroupId, // when as client, `fgid` is GROUP_ID
    tgid: GroupId, // when as server, `tgid` is GROUP_ID
    is_server: bool,
    msg: RecvType,
) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Connect(addr, data) => {
            // only server handle it. IMPORTANT !!! fgid IS mgid.
            if !is_server {
                let s = SendType::Result(0, addr, false, false, vec![]);
                add_server_layer(&mut results, fgid, s);
                return Ok(results);
            }

            let LayerConnect(gcd, connect) = bincode::deserialize(&data)?;
            let (ogid, height, id) = layer.read().await.running(&gcd)?.owner_height_id();

            match connect {
                ConnectProof::Common(_proof) => {
                    // check is member.
                    let db = group_chat_db(&layer.read().await.base, &ogid)?;

                    if let Ok((mid, _)) = Member::get_id(&db, &id, &fgid) {
                        let res = LayerResult(gcd, height);
                        let data = bincode::serialize(&res).unwrap_or(vec![]);
                        let s = SendType::Result(0, addr.clone(), true, false, data);
                        add_server_layer(&mut results, fgid, s);

                        layer.write().await.running_mut(&gcd)?.check_add_online(
                            fgid,
                            Online::Direct(addr.id),
                            id,
                            mid,
                        )?;

                        let _ = Member::addr_update(&db, &id, &fgid, &addr.id);
                        results
                            .rpcs
                            .push(rpc::member_online(ogid, id, fgid, addr.id));

                        let new_data =
                            bincode::serialize(&LayerEvent::MemberOnline(gcd, fgid, addr.id))?;

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
            // only server handle it. IMPORTANT !!! fgid IS mgid.
            // TODO
        }
        RecvType::Result(addr, is_ok, data) => {
            // only cleint handle it. IMPORTANT !!! tgid IS ogid.
            if !is_server && is_ok {
                let mut layer_lock = layer.write().await;
                handle_connect(tgid, &addr, data, &mut layer_lock, &mut results)?;
            } else {
                let msg = SendType::Result(0, addr, false, false, vec![]);
                add_layer(&mut results, tgid, msg);
            }
        }
        RecvType::ResultConnect(addr, data) => {
            // only cleint handle it. IMPORTANT !!! tgid IS ogid.
            if is_server {
                let msg = SendType::Result(0, addr.clone(), false, false, vec![]);
                add_layer(&mut results, tgid, msg);
            }

            let mut layer_lock = layer.write().await;
            if handle_connect(tgid, &addr, data, &mut layer_lock, &mut results)? {
                let msg = SendType::Result(0, addr, true, false, vec![]);
                add_layer(&mut results, tgid, msg);
            }
        }
        RecvType::Event(addr, bytes) => {
            println!("----------- DEBUG GROUP CHAT: GOT LAYER EVENT");
            // server & client handle it.
            let event: LayerEvent = bincode::deserialize(&bytes)?;
            handle_event(fgid, tgid, is_server, addr, event, layer, &mut results).await?;
            println!("----------- DEBUG GROUP CHAT: OVER LAYER EVENT");
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
    ogid: GroupId,
    addr: &Peer,
    data: Vec<u8>,
    layer: &mut Layer,
    results: &mut HandleResult,
) -> Result<bool> {
    // 0. deserialize result.
    let LayerResult(gcd, height) = bincode::deserialize(&data)?;

    // 1. check group.
    let db = group_chat_db(layer.base(), &ogid)?;
    if let Some(group) = GroupChat::get(&db, &gcd)? {
        // 1.0 check address.
        if group.g_addr != addr.id {
            return Ok(false);
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
            return Ok(false);
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
        Ok(true)
    } else {
        Ok(false)
    }
}

// variable statement:
// gcd: Group Chat ID.
// fgid: where is event come from.
// ogid: my account ID. if server is group owner. if client is my.
// mgid: member account ID.
// id: Group Chat database Id.
// mid: member database Id.
// sid: session Id.
async fn handle_event(
    fgid: GroupId, // server use fgid is remote account.
    tgid: GroupId, // client user tgid is my account.
    is_server: bool,
    addr: PeerId,
    event: LayerEvent,
    layer: &Arc<RwLock<Layer>>,
    results: &mut HandleResult,
) -> Result<()> {
    println!("Got event.......is server: {:?}", is_server);
    let base = layer.read().await.base().clone();
    let (sid, db, id, height, ogid, fgid) = if let Some(gcd) = event.gcd() {
        if is_server {
            let (ogid, height, id) = layer.read().await.running(gcd)?.owner_height_id();
            println!("--- DEBUG server:--- online info ok");
            let db = group_chat_db(&base, &ogid)?;
            println!("--- DEBUG server:--- db ok");
            (0, db, id, height, ogid, fgid)
        } else {
            let (sid, id) = if event.need_online() {
                println!("--- DEBUG client: --- need online info start");
                let (sid, id) = layer.read().await.get_running_remote_id(&tgid, gcd)?;
                (sid, id)
            } else {
                println!("--- DEBUG client: --- not need online info");
                (0, 0)
            };
            println!("--- DEBUG client:--- online info ok");

            let db = group_chat_db(&base, &tgid)?;
            println!("--- DEBUG client:--- db ok");
            (sid, db, id, 0, tgid, *gcd)
        }
    } else {
        println!("--- DEBUG --- no group id");
        let db = group_chat_db(&base, &tgid)?;
        (0, db, 0, 0, tgid, fgid)
    };
    println!("Handle variable statement ok.");

    match event {
        LayerEvent::Offline(gcd) => {
            if is_server {
                // 1. check member online.
                if layer.write().await.remove_online(&gcd, &fgid).is_none() {
                    return Ok(());
                }

                // 2. UI: offline the member.
                results.rpcs.push(rpc::member_offline(ogid, id, fgid));

                // 3. broadcast offline event.
                broadcast(&LayerEvent::MemberOffline(gcd, fgid), layer, &gcd, results).await?;
            } else {
                // 1. offline group chat.
                layer
                    .write()
                    .await
                    .running_mut(&ogid)?
                    .check_offline(&gcd, &addr);

                // 2. UI: offline the session.
                results.rpcs.push(session_lost(ogid, &sid));
            }
        }
        LayerEvent::Suspend(gcd) => {
            // TODO server & client handle it.
            if is_server {
                //
            } else {
                if layer
                    .write()
                    .await
                    .running_mut(&ogid)?
                    .suspend(&gcd, false, true)?
                {
                    results.rpcs.push(session_suspend(ogid, &sid));
                }
            }
        }
        LayerEvent::Actived(gcd) => {
            // TODO if only client handle it ???
            let _ = layer.write().await.running_mut(&ogid)?.active(&gcd, false);
            results.rpcs.push(session_connect(ogid, &sid, &addr));
        }
        LayerEvent::CheckResult(ct, name, remain, supported) => {
            // only client handle it.
            println!("check: {:?}, supported: {:?}", ct, supported);
            let mut provider = Provider::get_by_addr(&db, &addr)?;
            let rpc_ui = match ct {
                CheckType::Allow => {
                    provider.update(&db, name, supported, remain)?;
                    rpc::provider_check(ogid, &provider)
                }
                CheckType::None => {
                    provider.update(&db, name, supported, 0)?;
                    rpc::provider_check(ogid, &provider)
                }
                CheckType::Suspend => {
                    provider.suspend(&db)?;
                    rpc::provider_check(ogid, &provider)
                }
                CheckType::Deny => {
                    Provider::delete(&db, &provider.id)?;
                    rpc::provider_delete(ogid, provider.id)
                }
            };
            results.rpcs.push(rpc_ui)
        }
        LayerEvent::CreateResult(gcd, ok) => {
            // only client handle it.
            println!("Create result: {}", ok);
            if ok {
                // get gc by gcd.
                if let Some(mut gc) = GroupChat::get(&db, &gcd)? {
                    gc.ok(&db)?;
                    results.rpcs.push(rpc::create_result(ogid, gc.id, ok));

                    // ADD NEW SESSION.
                    let s_db = session_db(&base, &ogid)?;
                    let mut session = gc.to_session();
                    session.insert(&s_db)?;
                    results.rpcs.push(session_create(ogid, &session));
                }
            }
        }
        LayerEvent::Agree(gcd, info) => {
            // only client handle it.
            println!("Agree..........");
            let (rid, key) = Request::over(&db, &gcd, true)?;

            // 1. add group chat.
            let mut group = GroupChat::from_info(key, info, 0, addr, &base, &ogid, true)?;
            group.insert(&db)?;

            // 2. ADD NEW SESSION.
            let s_db = session_db(&base, &ogid)?;
            let mut session = group.to_session();
            session.insert(&s_db)?;
            results.rpcs.push(session_create(ogid, &session));

            // 3. update UI.
            results
                .rpcs
                .push(rpc::request_handle(ogid, id, rid, true, false));
            results.rpcs.push(rpc::group_create(ogid, group));

            // 4. try connect.
            let proof = layer
                .read()
                .await
                .group
                .read()
                .await
                .prove_addr(&ogid, &addr)?;
            add_layer(results, ogid, group_chat_conn(proof, Peer::peer(addr), gcd));
        }
        LayerEvent::Reject(gcd, efficacy) => {
            // only client handle it.
            println!("Reject..........");
            let (rid, _key) = Request::over(&db, &gcd, true)?;
            results
                .rpcs
                .push(rpc::request_handle(ogid, id, rid, false, efficacy));
        }
        LayerEvent::MemberOnline(_gcd, mgid, maddr) => {
            // only client handle it.
            let _ = Member::addr_update(&db, &id, &mgid, &maddr);
            results.rpcs.push(rpc::member_online(ogid, id, mgid, maddr));
        }
        LayerEvent::MemberOffline(_gcd, mgid) => {
            // only client handle it.
            results.rpcs.push(rpc::member_offline(ogid, id, mgid));
        }
        LayerEvent::MemberOnlineSyncResult(_gcd, onlines) => {
            // only client handle it.
            for (mgid, maddr) in onlines {
                results.rpcs.push(rpc::member_online(ogid, id, mgid, maddr));
            }
        }
        LayerEvent::Sync(gcd, height, event) => {
            // all server & client handle it.
            println!("Sync: handle height: {}", height);

            match event {
                Event::GroupInfo => {}
                Event::GroupTransfer => {}
                Event::GroupManagerAdd => {}
                Event::GroupManagerDel => {}
                Event::GroupClose => {}
                Event::MemberJoin(mgid, maddr, mname, mavatar, mtime) => {
                    // only client handle it.
                    if Member::get_id(&db, &id, &mgid).is_err() {
                        let mut member = Member::new(id, mgid, maddr, mname, false, mtime);
                        member.insert(&db)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
                        }
                        results.rpcs.push(rpc::member_join(ogid, &member));
                    }

                    // save consensus.
                    GroupChat::add_height(&db, id, height)?;
                }
                Event::MemberInfo(mgid, maddr, mname, mavatar) => {
                    // TOOD server & client all handlt it.

                    let (mid, _) = Member::get_id(&db, &id, &mgid)?;
                    Member::update(&db, &mid, &maddr, &mname)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
                    }
                    results
                        .rpcs
                        .push(rpc::member_info(ogid, id, mid, maddr, mname));

                    // save consensus.
                    GroupChat::add_height(&db, id, height)?;
                }
                Event::MemberLeave(mgid) => {
                    // TODO server & client all handle it.

                    let (mid, _) = Member::get_id(&db, &id, &mgid)?;
                    Member::leave(&db, &mid)?;
                    // check mid is my chat friend. if not, delete avatar.
                    let s_db = chat_db(&base, &mgid)?;
                    if Friend::get(&s_db, &mgid)?.is_none() {
                        let _ = delete_avatar(&base, &ogid, &mgid).await;
                    }
                    results.rpcs.push(rpc::member_leave(ogid, id, mid));

                    // save consensus.
                    GroupChat::add_height(&db, id, height)?;
                }
                Event::MessageCreate(mgid, nmsg, mtime) => {
                    // server & client all handle it.
                    println!("Sync: create message start");
                    let (mid, _) = Member::get_id(&db, &id, &mgid)?;
                    let new_height = if is_server {
                        let new_e = Event::MessageCreate(mgid, nmsg.clone(), mtime);
                        let height = layer.write().await.running_mut(&gcd)?.increased();
                        Consensus::insert(&db, &id, &height, &mid, &ConsensusType::MessageCreate)?;
                        broadcast(&LayerEvent::Sync(gcd, height, new_e), layer, &gcd, results)
                            .await?;
                        height
                    } else {
                        height
                    };
                    GroupChat::add_height(&db, id, new_height)?;

                    let (msg, scontent) =
                        from_network_message(new_height, id, mgid, &ogid, nmsg, mtime, &base)?;
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
            }
        }
        LayerEvent::Packed(gcd, height, from, to, events) => {
            // only client handle it.
            if to >= height {
                // when last packed sync, start sync online members.
                add_layer(results, ogid, sync_online(gcd, addr));
            }

            println!("Start handle sync packed... {}, {}, {}", height, from, to);
            handle_sync(
                &db, ogid, id, gcd, addr, height, from, to, events, base, results,
            )?;
        }
        LayerEvent::RequestHandle(_gcd, rgid, raddr, join_proof, rid, time) => {
            // only client handle it.
            match join_proof {
                JoinProof::Invite(i, _proof, mname, mavatar) => {
                    let mut req =
                        Request::new_by_remote(id, rid, rgid, raddr, mname, i.to_hex(), time);
                    req.insert(&db)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&base, &ogid, &rgid, mavatar)?;
                    }
                    results.rpcs.push(rpc::request_create(ogid, &req));
                }
                JoinProof::Zkp(_proof) => {
                    //
                }
                JoinProof::Open(..) => {} // nerver here.
            }
        }
        LayerEvent::RequestResult(_gcd, rrid, ok) => {
            // only client handle it.
            let rid = Request::over_rid(&db, &id, &rrid, ok)?;
            results
                .rpcs
                .push(rpc::request_handle(ogid, id, rid, ok, false));
        }
        LayerEvent::MemberOnlineSync(gcd) => {
            // only server handle it.
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
        LayerEvent::Request(gcd, join_proof) => {
            // only server handle it.
            println!("----------- PRINTLN GROUP CHAT: GOT REQUEST: {}", id);
            let group = GroupChat::get_id(&db, &id)?.ok_or(anyhow!("missing group"))?;

            // 1. check account is online, if not online, nothing.
            match join_proof {
                JoinProof::Open(mname, mavatar) => {
                    // check is member.
                    if Member::get_id(&db, &id, &fgid).is_ok() {
                        let s = agree(&base, &ogid, &gcd, group, addr).await;
                        add_server_layer(results, fgid, s);
                        return Ok(());
                    }

                    if group.g_type == GroupType::Open {
                        let mut m = Member::new_notime(id, fgid, addr, mname, false);
                        m.insert(&db)?;
                        // save avatar.
                        let _ = write_avatar(&base, &ogid, &m.m_id, &mavatar).await;

                        // add consensuse and storage.
                        let height = layer.write().await.running_mut(&gcd)?.increased();
                        Consensus::insert(&db, &id, &height, &m.id, &ConsensusType::MemberJoin)?;
                        GroupChat::add_height(&db, id, height)?;

                        // UI: update.
                        results.rpcs.push(rpc::member_join(ogid, &m));

                        // broadcast join event.
                        let event =
                            Event::MemberJoin(m.m_id, m.m_addr, m.m_name, mavatar, m.datetime);
                        broadcast(&LayerEvent::Sync(gcd, height, event), layer, &gcd, results)
                            .await?;

                        // return join result.
                        let s = agree(&base, &ogid, &gcd, group, addr).await;
                        add_server_layer(results, fgid, s);
                    } else {
                        add_server_layer(results, fgid, reject(gcd, addr, false));
                    }
                }
                JoinProof::Invite(invite_gid, _proof, mname, mavatar) => {
                    println!("----------- PRINTLN GROUP CHAT: GOT REQUEST INVITE: {}", id);
                    // check is member.
                    if Member::get_id(&db, &id, &fgid).is_ok() {
                        let s = agree(&base, &ogid, &gcd, group, addr).await;
                        add_server_layer(results, fgid, s);
                        println!("----------- PRINTLN GROUP CHAT: GOT REQUEST HAD MEMBER");
                        return Ok(());
                    }

                    // TODO check if request had or is blocked by manager.

                    // check if inviter is member.
                    let inv_mid = Member::get_id(&db, &id, &invite_gid);
                    if inv_mid.is_err() {
                        add_server_layer(results, fgid, reject(gcd, addr, false));
                        println!("----------- PRINTLN GROUP CHAT: Inviter not exists");
                        return Ok(());
                    }

                    // TODO check proof.
                    //proof.verify(&invite_gid, &addr, &layer.addr)?;

                    if group.is_need_agree {
                        let (_inv_id, inv_is_manager) = inv_mid.unwrap();
                        // only server holder to handle this request.
                        if !inv_is_manager {
                            let mut req =
                                Request::new_by_server(id, fgid, addr, mname, invite_gid.to_hex());
                            req.insert(&db)?;
                            req.rid = req.id; // not need save. beacuse UI rpc will sended.
                            if mavatar.len() > 0 {
                                write_avatar_sync(&base, &ogid, &fgid, mavatar)?;
                            }
                            results.rpcs.push(rpc::request_create(ogid, &req));
                            return Ok(());
                        }
                    }

                    let mut m = Member::new_notime(id, fgid, addr, mname, false);
                    m.insert(&db)?;
                    // save avatar.
                    let _ = write_avatar(&base, &ogid, &m.m_id, &mavatar).await;

                    // add consensuse and storage.
                    let height = layer.write().await.running_mut(&gcd)?.increased();
                    Consensus::insert(&db, &id, &height, &m.id, &ConsensusType::MemberJoin)?;
                    GroupChat::add_height(&db, id, height)?;

                    // UI: update.
                    results.rpcs.push(rpc::member_join(ogid, &m));

                    // broadcast join event.
                    let event = Event::MemberJoin(m.m_id, m.m_addr, m.m_name, mavatar, m.datetime);
                    broadcast(&LayerEvent::Sync(gcd, height, event), layer, &gcd, results).await?;

                    // return join result.
                    let s = agree(&base, &ogid, &gcd, group, addr).await;
                    add_server_layer(results, fgid, s);
                    println!("----------- PRINTLN GROUP CHAT: GOT REQUEST INVITE OVER");
                }
                JoinProof::Zkp(_proof) => {
                    // TOOD zkp join.
                }
            }
        }
        LayerEvent::SyncReq(gcd, from) => {
            // only server handle it.
            println!("Got sync request. height: {} from: {}", height, from);

            if height >= from {
                let to = if height - from > 100 {
                    from + 100
                } else {
                    height
                };
                let packed = Consensus::pack(&db, &base, &gcd, &id, &from, &to).await?;
                let event = LayerEvent::Packed(gcd, height, from, to, packed);
                let data = bincode::serialize(&event).unwrap_or(vec![]);
                let s = SendType::Event(0, addr, data);
                add_server_layer(results, fgid, s);
                println!("Sended sync request results. from: {}, to: {}", from, to);
            }
        }
        LayerEvent::Check => {}      // nerver here.
        LayerEvent::Create(..) => {} // nerver here.
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

pub(crate) fn group_chat_conn(proof: Proof, addr: Peer, gid: GroupId) -> SendType {
    let data =
        bincode::serialize(&LayerConnect(gid, ConnectProof::Common(proof))).unwrap_or(vec![]);
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

async fn agree(
    base: &PathBuf,
    ogid: &GroupId,
    gcd: &GroupId,
    group: GroupChat,
    addr: PeerId,
) -> SendType {
    let gavatar = read_avatar(base, ogid, gcd).await.unwrap_or(vec![]);
    let group_info = group.to_group_info("".to_owned(), gavatar, vec![]);
    let res = LayerEvent::Agree(*gcd, group_info);
    let d = bincode::serialize(&res).unwrap_or(vec![]);
    SendType::Event(0, addr, d)
}

fn reject(gcd: GroupId, addr: PeerId, lost: bool) -> SendType {
    let d = bincode::serialize(&LayerEvent::Reject(gcd, lost)).unwrap_or(vec![]);
    SendType::Event(0, addr, d)
}

fn handle_sync(
    db: &DStorage,
    ogid: GroupId,
    id: i64,
    gcd: GroupId,
    addr: PeerId,
    height: i64,
    mut from: i64,
    to: i64,
    events: Vec<PackedEvent>,
    base: PathBuf,
    results: &mut HandleResult,
) -> Result<()> {
    let mut last_scontent: Option<(String, i64)> = None;

    for event in events {
        if let Ok(scontent) = handle_sync_event(&ogid, &id, from, event, &base, db, results) {
            last_scontent = scontent;
        }
        from += 1;
    }

    if to < height {
        add_layer(results, ogid, sync(gcd, addr, to + 1));
    }

    // update group chat height.
    GroupChat::add_height(db, id, to)?;

    // UPDATE SESSION.
    if let Some((sc, t)) = last_scontent {
        let s_db = session_db(&base, &ogid)?;
        if let Ok(sid) = Session::last(&s_db, &id, &SessionType::Group, &t, &sc, true) {
            results.rpcs.push(session_last(ogid, &sid, &t, &sc, false));
        }
    }

    Ok(())
}

fn handle_sync_event(
    ogid: &GroupId,
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
                    write_avatar_sync(&base, &ogid, &mid, mavatar)?;
                }
                let mut member = Member::new(*fid, mid, maddr, mname, false, mtime);
                member.insert(&db)?;
                results.rpcs.push(rpc::member_join(*ogid, &member));
            }
            None
        }
        PackedEvent::MemberLeave(_mid) => {
            // TODO
            None
        }
        PackedEvent::MessageCreate(mid, nmsg, time) => {
            let (msg, scontent) = from_network_message(height, *fid, mid, ogid, nmsg, time, base)?;
            results.rpcs.push(rpc::message_create(*ogid, &msg));
            Some((scontent, time))
        }
        PackedEvent::None => None,
    };

    Ok(scontent)
}
