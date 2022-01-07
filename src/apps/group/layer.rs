use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::{RecvType, SendType},
    primitive::{HandleResult, Peer, PeerId, Result},
};
use tokio::sync::RwLock;

use chat_types::MessageType;
use group_types::{Event, LayerConnect, LayerEvent, LayerResult};
use tdn_did::Proof;
use tdn_storage::local::DStorage;

use crate::apps::chat::Friend;
use crate::layer::{Layer, Online};
use crate::rpc::{
    session_close, session_connect, session_last, session_lost, session_suspend,
    session_update_name,
};
use crate::session::{connect_session, Session, SessionType};
use crate::storage::{delete_avatar, write_avatar_sync};

use super::models::{handle_network_message, GroupChat, Member, Message};
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
            debug!("----------- DEBUG GROUP CHAT: SERVER GOT LAYER EVENT");
            let event: LayerEvent = bincode::deserialize(&bytes)?;
            handle_server_event(fgid, addr, event, layer, &mut results).await?;
            debug!("----------- DEBUG GROUP CHAT: SERVER OVER LAYER EVENT");
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
    let db = layer.read().await.group.read().await.group_db(&ogid)?;
    let g = GroupChat::get(&db, &id)?;
    let mdid = Member::get_id(&db, &id, &fgid)?;

    let res = LayerResult(gcd, g.g_name, height);
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
        .push(rpc::member_online(ogid, id, mdid, &addr.id));

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
                handle_connect(ogid, &addr, data, &mut layer_lock, &mut results).await?;
            } else {
                // close the group chat.
                let gcd: GroupId = bincode::deserialize(&data)?;

                let layer_lock = layer.read().await;
                let group_lock = layer_lock.group.read().await;
                let db = group_lock.group_db(&ogid)?;
                let s_db = group_lock.session_db(&ogid)?;
                drop(group_lock);
                drop(layer_lock);

                let group = GroupChat::close(&db, &gcd)?;
                let sid = Session::close(&s_db, &group.id, &SessionType::Group)?;
                results.rpcs.push(session_close(ogid, &sid));
            }
        }
        RecvType::ResultConnect(addr, data) => {
            let mut layer_lock = layer.write().await;
            if handle_connect(ogid, &addr, data, &mut layer_lock, &mut results)
                .await
                .is_err()
            {
                let msg = SendType::Result(0, addr, true, false, vec![]);
                add_layer(&mut results, ogid, msg);
            }
        }
        RecvType::Event(addr, bytes) => {
            debug!("----------- DEBUG GROUP CHAT: PEER GOT LAYER EVENT");
            let event: LayerEvent = bincode::deserialize(&bytes)?;
            handle_peer_event(ogid, addr, event, layer, &mut results).await?;
            debug!("----------- DEBUG GROUP CHAT: PEER OVER LAYER EVENT");
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

async fn handle_connect(
    ogid: GroupId,
    addr: &Peer,
    data: Vec<u8>,
    layer: &mut Layer,
    results: &mut HandleResult,
) -> Result<()> {
    // 0. deserialize result.
    let LayerResult(gcd, gname, height) = bincode::deserialize(&data)?;

    // 1. check group.
    let db = layer.group.read().await.group_db(&ogid)?;
    let group = GroupChat::get_id(&db, &gcd)?;

    // 1.0 check address.
    if group.g_addr != addr.id {
        return Err(anyhow!("invalid group chat address."));
    }

    let _ = GroupChat::update_name(&db, &group.id, &gname);
    results.rpcs.push(rpc::group_name(ogid, &group.id, &gname));

    // 1.1 get session.
    let s_db = layer.group.read().await.session_db(&ogid)?;
    let session_some = connect_session(&s_db, &SessionType::Group, &group.id, &addr.id)?;
    if session_some.is_none() {
        return Err(anyhow!("invalid group chat address."));
    }
    let sid = session_some.unwrap().id;

    let _ = Session::update_name(&s_db, &sid, &gname);
    results.rpcs.push(session_update_name(ogid, &sid, &gname));

    // 1.2 online this group.
    layer
        .running_mut(&ogid)?
        .check_add_online(gcd, Online::Direct(addr.id), sid, group.id)?;

    // 1.3 online to UI.
    results.rpcs.push(session_connect(ogid, &sid, &addr.id));

    debug!("will sync remote: {}, my: {}", height, group.height);
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
    let db = layer.read().await.group.read().await.group_db(&ogid)?;

    match event {
        LayerEvent::Offline(gcd) => {
            // 1. check member online.
            if layer.write().await.remove_online(&gcd, &fgid).is_none() {
                return Ok(());
            }

            // 2. UI: offline the member.
            if let Ok(mid) = Member::get_id(&db, &id, &fgid) {
                results.rpcs.push(rpc::member_offline(ogid, id, mid));
            }

            // 3. broadcast offline event.
            broadcast(&LayerEvent::MemberOffline(gcd, fgid), layer, &gcd, results).await?;
        }
        LayerEvent::GroupName(gcd, name) => {
            // 1. update group name
            let _ = GroupChat::update_name(&db, &id, &name)?;
            // 2. UI: update
            results.rpcs.push(rpc::group_name(ogid, &id, &name));
            if let Ok(sid) = Session::update_name_by_id(
                &layer.read().await.group.read().await.session_db(&ogid)?,
                &id,
                &SessionType::Group,
                &name,
            ) {
                results.rpcs.push(session_update_name(ogid, &sid, &name));
            }
            // 3. broadcast
            broadcast(&LayerEvent::GroupName(gcd, name), layer, &gcd, results).await?;
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
                        let mem = Member::info(mdid, id, mgid, maddr, mname);
                        results.rpcs.push(rpc::member_join(ogid, &mem));
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
                    let s_db = &layer.read().await.group.read().await.chat_db(&ogid)?;
                    if Friend::get_id(&s_db, &mgid).is_err() {
                        let _ = delete_avatar(&base, &ogid, &mgid).await;
                    }
                    results.rpcs.push(rpc::member_leave(ogid, id, mdid));

                    // broadcast
                    GroupChat::add_height(&db, id, h)?;
                    broadcast(&LayerEvent::Sync(gcd, h, event), layer, &gcd, results).await?;
                }
                Event::MessageCreate(mgid, nmsg, mtime) => {
                    debug!("Sync: create message start");
                    let _mdid = Member::get_id(&db, &id, &mgid)?;

                    let new_e = Event::MessageCreate(mgid, nmsg.clone(), mtime);
                    let new_h = layer.write().await.running_mut(&gcd)?.increased();
                    broadcast(&LayerEvent::Sync(gcd, new_h, new_e), layer, &gcd, results).await?;
                    GroupChat::add_height(&db, id, new_h)?;

                    let msg = handle_network_message(
                        &layer.read().await.group,
                        new_h,
                        id,
                        mgid,
                        &ogid,
                        nmsg,
                        mtime,
                        &base,
                        results,
                    )
                    .await?;
                    results.rpcs.push(rpc::message_create(ogid, &msg));
                    debug!("Sync: create message ok");

                    // UPDATE SESSION.
                    if let Ok(s_db) = layer.read().await.group.read().await.session_db(&ogid) {
                        update_session(&s_db, &ogid, &id, &msg, results);
                    }
                }
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
            debug!("Got sync request. height: {} from: {}", height, from);

            if height >= from {
                let to = if height - from > 20 {
                    from + 20
                } else {
                    height
                };

                let (members, leaves) = Member::sync(&base, &ogid, &db, &id, &from, &to).await?;
                let messages = Message::sync(&base, &ogid, &db, &id, &from, &to).await?;
                let event = LayerEvent::SyncRes(gcd, height, from, to, members, leaves, messages);
                let data = bincode::serialize(&event).unwrap_or(vec![]);
                let s = SendType::Event(0, addr, data);
                add_server_layer(results, fgid, s);
                debug!("Sended sync request results. from: {}, to: {}", from, to);
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
    let db = layer.read().await.group.read().await.group_db(&ogid)?;

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
            if let Ok(mid) = Member::addr_update(&db, &id, &mgid, &maddr) {
                results.rpcs.push(rpc::member_online(ogid, id, mid, &maddr));
            }
        }
        LayerEvent::MemberOffline(_gcd, mgid) => {
            if let Ok(mid) = Member::get_id(&db, &id, &mgid) {
                results.rpcs.push(rpc::member_offline(ogid, id, mid));
            }
        }
        LayerEvent::MemberOnlineSyncResult(_gcd, onlines) => {
            for (mgid, maddr) in onlines {
                if let Ok(mid) = Member::addr_update(&db, &id, &mgid, &maddr) {
                    results.rpcs.push(rpc::member_online(ogid, id, mid, &maddr));
                }
            }
        }
        LayerEvent::GroupName(_gcd, name) => {
            let _ = GroupChat::update_name(&db, &id, &name)?;
            results.rpcs.push(rpc::group_name(ogid, &id, &name));
            let _ = Session::update_name(
                &layer.read().await.group.read().await.session_db(&ogid)?,
                &sid,
                &name,
            );
            results.rpcs.push(session_update_name(ogid, &sid, &name));
        }
        LayerEvent::GroupClose(_gcd) => {
            let group = GroupChat::close(&db, &gcd)?;
            let sid = Session::close(
                &layer.read().await.group.read().await.session_db(&ogid)?,
                &group.id,
                &SessionType::Group,
            )?;
            results.rpcs.push(session_close(ogid, &sid));
        }
        LayerEvent::Sync(_gcd, height, event) => {
            debug!("Sync: handle height: {}", height);

            match event {
                Event::MemberJoin(mgid, maddr, mname, mavatar) => {
                    let mdid_res = Member::get_id(&db, &id, &mgid);
                    if let Ok(mdid) = mdid_res {
                        Member::update(&db, &height, &mdid, &maddr, &mname)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
                        }
                        let mem = Member::info(mdid, id, mgid, maddr, mname);
                        results.rpcs.push(rpc::member_join(ogid, &mem));
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
                    let s_db = &layer.read().await.group.read().await.chat_db(&ogid)?;
                    if Friend::get_id(&s_db, &mgid).is_err() {
                        let _ = delete_avatar(&base, &ogid, &mgid).await;
                    }
                    results.rpcs.push(rpc::member_leave(ogid, id, mdid));

                    // save consensus.
                    GroupChat::add_height(&db, id, height)?;
                }
                Event::MessageCreate(mgid, nmsg, mtime) => {
                    debug!("Sync: create message start");
                    let _mdid = Member::get_id(&db, &id, &mgid)?;

                    let msg = handle_network_message(
                        &layer.read().await.group,
                        height,
                        id,
                        mgid,
                        &ogid,
                        nmsg,
                        mtime,
                        &base,
                        results,
                    )
                    .await?;
                    results.rpcs.push(rpc::message_create(ogid, &msg));

                    GroupChat::add_height(&db, id, height)?;
                    debug!("Sync: create message ok");

                    // UPDATE SESSION.
                    if let Ok(s_db) = layer.read().await.group.read().await.session_db(&ogid) {
                        update_session(&s_db, &ogid, &id, &msg, results);
                    }
                }
            }
        }
        LayerEvent::SyncRes(gcd, height, from, to, adds, leaves, messages) => {
            if to >= height {
                // when last packed sync, start sync online members.
                add_layer(results, ogid, sync_online(gcd, addr));
            }

            debug!("Start handle sync packed... {}, {}, {}", height, from, to);
            let mut last_message = None;

            for (height, mgid, maddr, mname, mavatar) in adds {
                let mdid_res = Member::get_id(&db, &id, &mgid);
                if let Ok(mdid) = mdid_res {
                    Member::update(&db, &height, &mdid, &maddr, &mname)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
                    }
                    let mem = Member::info(mdid, id, mgid, maddr, mname);
                    results.rpcs.push(rpc::member_join(ogid, &mem));
                } else {
                    let mut member = Member::new(height, id, mgid, maddr, mname);
                    member.insert(&db)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
                    }
                    results.rpcs.push(rpc::member_join(ogid, &member));
                }
            }

            for (height, mgid) in leaves {
                if let Ok(mdid) = Member::get_id(&db, &id, &mgid) {
                    Member::leave(&db, &height, &mdid)?;
                    // check mid is my chat friend. if not, delete avatar.
                    let s_db = &layer.read().await.group.read().await.chat_db(&ogid)?;
                    if Friend::get_id(&s_db, &mgid).is_err() {
                        let _ = delete_avatar(&base, &ogid, &mgid).await;
                    }
                    results.rpcs.push(rpc::member_leave(ogid, id, mdid));
                }
            }

            for (height, mgid, nm, time) in messages {
                if let Ok(msg) = handle_network_message(
                    &layer.read().await.group,
                    height,
                    id,
                    mgid,
                    &ogid,
                    nm,
                    time,
                    &base,
                    results,
                )
                .await
                {
                    results.rpcs.push(rpc::message_create(ogid, &msg));
                    last_message = Some(msg);
                }
            }

            if to < height {
                add_layer(results, ogid, sync(gcd, addr, to + 1));
            }

            // update group chat height.
            GroupChat::add_height(&db, id, to)?;

            // UPDATE SESSION.
            if let Some(msg) = last_message {
                if let Ok(s_db) = layer.read().await.group.read().await.session_db(&ogid) {
                    update_session(&s_db, &ogid, &id, &msg, results);
                }
            }
            debug!("Over handle sync packed... {}, {}, {}", height, from, to);
        }
        _ => error!("group peer handle event nerver here"),
    }

    Ok(())
}

pub(crate) async fn broadcast(
    event: &LayerEvent,
    layer: &Arc<RwLock<Layer>>,
    gcd: &GroupId,
    results: &mut HandleResult,
) -> Result<()> {
    let new_data = bincode::serialize(&event)?;

    for (mgid, maddr) in layer.read().await.running(&gcd)?.onlines() {
        let s = SendType::Event(0, *maddr, new_data.clone());
        add_server_layer(results, *mgid, s);
        debug!("--- DEBUG broadcast to: {:?}", mgid);
    }

    Ok(())
}

// UPDATE SESSION.
pub(crate) fn update_session(
    s_db: &DStorage,
    gid: &GroupId,
    id: &i64,
    msg: &Message,
    results: &mut HandleResult,
) {
    let scontent = match msg.m_type {
        MessageType::String => {
            format!("{}:{}", msg.m_type.to_int(), msg.content)
        }
        _ => format!("{}:", msg.m_type.to_int()),
    };

    if let Ok(sid) = Session::last(
        &s_db,
        id,
        &SessionType::Group,
        &msg.datetime,
        &scontent,
        true,
    ) {
        results
            .rpcs
            .push(session_last(*gid, &sid, &msg.datetime, &scontent, false));
    }
}

pub(crate) fn group_conn(proof: Proof, addr: Peer, gid: GroupId) -> SendType {
    let data = bincode::serialize(&LayerConnect(gid, proof)).unwrap_or(vec![]);
    SendType::Connect(0, addr, data)
}

fn sync(gcd: GroupId, addr: PeerId, height: i64) -> SendType {
    let data = bincode::serialize(&LayerEvent::SyncReq(gcd, height + 1)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}

fn sync_online(gcd: GroupId, addr: PeerId) -> SendType {
    let data = bincode::serialize(&LayerEvent::MemberOnlineSync(gcd)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}
