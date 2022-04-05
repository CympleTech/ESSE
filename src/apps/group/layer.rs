use esse_primitives::MessageType;
use group_types::{Event, GroupChatId, LayerConnect, LayerEvent, LayerResult, GROUP_CHAT_ID};
use std::sync::Arc;
use tdn::types::{
    message::{RecvType, SendType},
    primitives::{HandleResult, Peer, PeerId, Result},
};
use tdn_storage::local::DStorage;

use crate::apps::chat::Friend;
use crate::global::Global;
use crate::rpc::{
    session_close, session_connect, session_last, session_lost, session_suspend,
    session_update_name,
};
use crate::session::{connect_session, Session, SessionType};
use crate::storage::{chat_db, delete_avatar, group_db, session_db, write_avatar_sync};

use super::models::{handle_network_message, GroupChat, Member, Message};
use super::rpc;

// variable statement:
// gid: Group Chat ID.
// pid: my account ID.
// mpid: member account ID.
// id: Group Chat database Id.
// sid: Group Chat Session Id.
// mid: member database Id.
pub(crate) async fn handle(msg: RecvType, global: &Arc<Global>) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Connect(peer, data) => {
            // SERVER
            let LayerConnect(gid) = bincode::deserialize(&data)?;
            if handle_connect(global, &peer, gid, &mut results)
                .await
                .is_err()
            {
                let data = bincode::serialize(&gid)?;
                let msg = SendType::Result(0, peer, false, false, data);
                results.layers.push((GROUP_CHAT_ID, msg));
            }
        }
        RecvType::Result(peer, is_ok, data) => {
            // PEER
            if is_ok {
                handle_result(global, &peer, data, &mut results).await?;
            } else {
                // close the group chat.
                let gid: GroupChatId = bincode::deserialize(&data)?;

                let pid = global.pid().await;
                let db_key = global.own.read().await.db_key(&pid)?;
                let db = group_db(&global.base, &pid, &db_key)?;
                let s_db = session_db(&global.base, &pid, &db_key)?;

                let group = GroupChat::close_id(&db, &gid, &peer.id)?;
                let sid = Session::close(&s_db, &group.id, &SessionType::Group)?;
                results.rpcs.push(session_close(&sid));
            }
        }
        RecvType::ResultConnect(peer, data) => {
            // PEER
            if handle_result(global, &peer, data, &mut results)
                .await
                .is_err()
            {
                let msg = SendType::Result(0, peer, true, false, vec![]);
                results.layers.push((GROUP_CHAT_ID, msg));
            }
        }
        RecvType::Event(addr, bytes) => {
            // PEER & SERVER
            let event: LayerEvent = bincode::deserialize(&bytes)?;
            handle_event(addr, event, global, &mut results).await?;
        }
        RecvType::Delivery(..) => {}
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }
        RecvType::Leave(..) => {} // nerver here.
    }

    Ok(results)
}

async fn handle_connect(
    global: &Arc<Global>,
    peer: &Peer,
    gid: GroupChatId,
    results: &mut HandleResult,
) -> Result<()> {
    let (height, _, id, _) = global.layer.read().await.group(&gid)?.info();

    let pid = global.pid().await;
    let db_key = global.own.read().await.db_key(&pid)?;
    let db = group_db(&global.base, &pid, &db_key)?;

    // check is member.
    let g = GroupChat::get(&db, &id)?;
    let mid = Member::get_id(&db, &id, &peer.id)?;

    let res = LayerResult(gid, g.name, height);
    let data = bincode::serialize(&res).unwrap_or(vec![]);
    let s = SendType::Result(0, peer.clone(), true, false, data);
    results.layers.push((GROUP_CHAT_ID, s));

    global.layer.write().await.group_add_member(&gid, peer.id);
    results.rpcs.push(rpc::member_online(id, mid));

    let data = LayerEvent::MemberOnline(gid, peer.id);
    broadcast(&gid, global, &data, results).await?;
    Ok(())
}

async fn handle_result(
    global: &Arc<Global>,
    peer: &Peer,
    data: Vec<u8>,
    results: &mut HandleResult,
) -> Result<()> {
    // 0. deserialize result.
    let LayerResult(gid, name, height) = bincode::deserialize(&data)?;

    let pid = global.pid().await;
    let db_key = global.own.read().await.db_key(&pid)?;
    let db = group_db(&global.base, &pid, &db_key)?;
    let s_db = session_db(&global.base, &pid, &db_key)?;

    // 1. check group.
    let group = GroupChat::get_id(&db, &gid, &peer.id)?;

    // 1.0 check address.
    if group.addr != peer.id {
        return Err(anyhow!("invalid group chat address."));
    }

    let _ = GroupChat::update_name(&db, &group.id, &name);
    results.rpcs.push(rpc::group_name(&group.id, &name));

    // 1.1 get session.
    let session_some = connect_session(&s_db, &SessionType::Group, &group.id, &peer.id)?;
    if session_some.is_none() {
        return Err(anyhow!("invalid group chat address."));
    }
    let sid = session_some.unwrap().id;

    let _ = Session::update_name(&s_db, &sid, &name);
    results.rpcs.push(session_update_name(&sid, &name));

    // 1.2 online this group.
    global
        .layer
        .write()
        .await
        .group_add(gid, peer.id, sid, group.id, height);

    // 1.3 online to UI.
    results.rpcs.push(session_connect(&sid, &peer.id));

    debug!("will sync remote: {}, my: {}", height, group.height);
    // 1.4 sync group height.
    if group.height < height {
        results
            .layers
            .push((GROUP_CHAT_ID, sync(gid, peer.id, group.height)));
    } else {
        // sync online members.
        results
            .layers
            .push((GROUP_CHAT_ID, sync_online(gid, peer.id)));
    }

    Ok(())
}

async fn handle_event(
    addr: PeerId,
    event: LayerEvent,
    global: &Arc<Global>,
    results: &mut HandleResult,
) -> Result<()> {
    let gid = event.gid();
    let (height, sid, id, gaddr) = global.layer.read().await.group(&gid)?.info();
    let pid = global.pid().await;
    let db_key = global.own.read().await.db_key(&pid)?;
    let db = group_db(&global.base, &pid, &db_key)?;
    let is_server = gaddr == pid;
    if !is_server && gaddr != addr {
        warn!("INVALID EVENT NOT FROM THE SERVER.");
        return Err(anyhow!("NOT THE SERVER EVENT"));
    }

    match event {
        LayerEvent::Offline(gid) => {
            // SERVER & PEER
            if is_server {
                // 1. check member online.
                if !global.layer.write().await.group_del_online(&gid, &addr) {
                    return Ok(());
                }

                // 2. UI: offline the member.
                if let Ok(mid) = Member::get_id(&db, &id, &addr) {
                    results.rpcs.push(rpc::member_offline(id, mid));
                }

                // 3. broadcast offline event.
                broadcast(&gid, global, &LayerEvent::MemberOffline(gid, addr), results).await?;
            } else {
                // 1. offline group chat.
                global.layer.write().await.group_del(&gid);

                // 2. UI: offline the session.
                results.rpcs.push(session_lost(&sid));
            }
        }
        LayerEvent::Suspend(gid) => {
            // PEER
            if global
                .layer
                .write()
                .await
                .group_mut(&gid)?
                .suspend(false, true)
                .is_some()
            {
                results.rpcs.push(session_suspend(&sid));
            }
        }
        LayerEvent::Actived(gid) => {
            // PEER
            let _ = global.layer.write().await.group_mut(&gid)?.active(false);
            results.rpcs.push(session_connect(&sid, &addr));
        }
        LayerEvent::MemberOnline(_gid, mpid) => {
            // PEER
            if let Ok(mid) = Member::get_id(&db, &id, &mpid) {
                results.rpcs.push(rpc::member_online(id, mid));
            }
        }
        LayerEvent::MemberOffline(_gid, mpid) => {
            // PEER
            if let Ok(mid) = Member::get_id(&db, &id, &mpid) {
                results.rpcs.push(rpc::member_offline(id, mid));
            }
        }
        LayerEvent::MemberOnlineSync(gid) => {
            // SERVER
            let onlines = global.layer.read().await.group(&gid)?.addrs.clone();
            let event = LayerEvent::MemberOnlineSyncResult(gid, onlines);
            let data = bincode::serialize(&event).unwrap_or(vec![]);
            let msg = SendType::Event(0, addr, data);
            results.layers.push((GROUP_CHAT_ID, msg));
        }
        LayerEvent::MemberOnlineSyncResult(_gid, onlines) => {
            // PEER
            for mpid in onlines {
                if let Ok(mid) = Member::get_id(&db, &id, &mpid) {
                    results.rpcs.push(rpc::member_online(id, mid));
                }
            }
        }
        LayerEvent::GroupName(gid, name) => {
            // SERVER & PEER
            // 1. update group name
            let _ = GroupChat::update_name(&db, &id, &name)?;

            // 2. UI: update
            results.rpcs.push(rpc::group_name(&id, &name));
            let s_db = session_db(&global.base, &pid, &db_key)?;
            let _ = Session::update_name(&s_db, &sid, &name);
            results.rpcs.push(session_update_name(&sid, &name));

            if is_server {
                // 3. broadcast
                broadcast(&gid, global, &LayerEvent::GroupName(gid, name), results).await?;
            }
        }
        LayerEvent::GroupClose(gid) => {
            // PEER
            let group = GroupChat::close(&db, &id)?;
            let s_db = session_db(&global.base, &pid, &db_key)?;
            let sid = Session::close(&s_db, &group.id, &SessionType::Group)?;
            results.rpcs.push(session_close(&sid));
        }
        LayerEvent::Sync(gid, height, event) => {
            // SERVER & PEER
            debug!("Sync: handle is_server: {} height: {} ", is_server, height);
            match event {
                Event::MemberJoin(mpid, mname, mavatar) => {
                    let mid_res = Member::get_id(&db, &id, &mpid);
                    let h = if is_server {
                        global.layer.write().await.group_mut(&gid)?.increased()
                    } else {
                        height
                    };

                    if let Ok(mid) = mid_res {
                        Member::update(&db, &h, &mid, &mname)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&global.base, &pid, &mpid, mavatar.clone())?;
                        }
                        let mem = Member::info(mid, id, mpid, mname.clone());
                        results.rpcs.push(rpc::member_join(&mem));
                    } else {
                        let mut member = Member::new(h, id, mpid, mname.clone());
                        member.insert(&db)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&global.base, &pid, &mpid, mavatar.clone())?;
                        }
                        results.rpcs.push(rpc::member_join(&member));
                    }

                    GroupChat::add_height(&db, id, h)?;
                    if is_server {
                        // broadcast
                        let new_e = Event::MemberJoin(mpid, mname, mavatar);
                        broadcast(&gid, global, &LayerEvent::Sync(gid, h, new_e), results).await?;
                    }
                }
                Event::MemberLeave(mpid) => {
                    let mid = Member::get_id(&db, &id, &mpid)?;
                    let h = if is_server {
                        global.layer.write().await.group_mut(&gid)?.increased()
                    } else {
                        height
                    };
                    Member::leave(&db, &mid, &h)?;

                    // check mid is my chat friend. if not, delete avatar.
                    let c_db = chat_db(&global.base, &pid, &db_key)?;
                    if Friend::get_id(&c_db, &mpid).is_err() {
                        let _ = delete_avatar(&global.base, &pid, &mpid).await;
                    }
                    results.rpcs.push(rpc::member_leave(id, mid));

                    // broadcast
                    GroupChat::add_height(&db, id, h)?;
                    if is_server {
                        broadcast(&gid, global, &LayerEvent::Sync(gid, h, event), results).await?;
                    }
                }
                Event::MessageCreate(mpid, nmsg, mtime) => {
                    debug!("Sync: create message start");
                    let _mid = Member::get_id(&db, &id, &mpid)?;
                    let h = if is_server {
                        global.layer.write().await.group_mut(&gid)?.increased()
                    } else {
                        height
                    };

                    let msg = handle_network_message(
                        &pid,
                        &global.base,
                        &db_key,
                        h,
                        id,
                        mpid,
                        nmsg.clone(),
                        mtime,
                        results,
                    )
                    .await?;
                    results.rpcs.push(rpc::message_create(&msg));
                    debug!("Sync: create message ok");

                    // UPDATE SESSION.
                    let s_db = session_db(&global.base, &pid, &db_key)?;
                    update_session(&s_db, &id, &msg, results);

                    GroupChat::add_height(&db, id, h)?;
                    if is_server {
                        let new_e = Event::MessageCreate(mpid, nmsg, mtime);
                        broadcast(&gid, global, &LayerEvent::Sync(gid, h, new_e), results).await?;
                    }
                }
            }
        }
        LayerEvent::SyncReq(gid, from) => {
            // SERVER
            debug!("Got sync request. height: {} from: {}", height, from);

            if height >= from {
                let to = if height - from > 20 {
                    from + 20
                } else {
                    height
                };

                let (members, leaves) =
                    Member::sync(&global.base, &pid, &db, &id, &from, &to).await?;
                let messages = Message::sync(&global.base, &pid, &db, &id, &from, &to).await?;
                let event = LayerEvent::SyncRes(gid, height, from, to, members, leaves, messages);
                let data = bincode::serialize(&event).unwrap_or(vec![]);
                let s = SendType::Event(0, addr, data);
                results.layers.push((GROUP_CHAT_ID, s));
                debug!("Sended sync request results. from: {}, to: {}", from, to);
            }
        }
        LayerEvent::SyncRes(gid, height, from, to, adds, leaves, messages) => {
            // PEER
            if to >= height {
                results.layers.push((GROUP_CHAT_ID, sync_online(gid, addr)));
                // when last packed sync, start sync online members.
            }

            debug!("Start handle sync packed... {}, {}, {}", height, from, to);
            let mut last_message = None;

            for (height, mpid, mname, mavatar) in adds {
                let mid_res = Member::get_id(&db, &id, &mpid);
                if let Ok(mid) = mid_res {
                    Member::update(&db, &height, &mid, &mname)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&global.base, &pid, &mpid, mavatar)?;
                    }
                    let mem = Member::info(mid, id, mpid, mname);
                    results.rpcs.push(rpc::member_join(&mem));
                } else {
                    let mut member = Member::new(height, id, mpid, mname);
                    member.insert(&db)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&global.base, &pid, &mpid, mavatar)?;
                    }
                    results.rpcs.push(rpc::member_join(&member));
                }
            }

            let c_db = chat_db(&global.base, &pid, &db_key)?;
            for (height, mpid) in leaves {
                if let Ok(mid) = Member::get_id(&db, &id, &mpid) {
                    Member::leave(&db, &height, &mid)?;
                    // check mid is my chat friend. if not, delete avatar.
                    if Friend::get_id(&c_db, &mpid).is_err() {
                        let _ = delete_avatar(&global.base, &pid, &mpid).await;
                    }
                    results.rpcs.push(rpc::member_leave(id, mid));
                }
            }

            for (height, mpid, nm, time) in messages {
                if let Ok(msg) = handle_network_message(
                    &pid,
                    &global.base,
                    &db_key,
                    height,
                    id,
                    mpid,
                    nm,
                    time,
                    results,
                )
                .await
                {
                    results.rpcs.push(rpc::message_create(&msg));
                    last_message = Some(msg);
                }
            }

            if to < height {
                results
                    .layers
                    .push((GROUP_CHAT_ID, sync(gid, addr, to + 1)));
            }

            // update group chat height.
            GroupChat::add_height(&db, id, to)?;

            // UPDATE SESSION.
            if let Some(msg) = last_message {
                let s_db = session_db(&global.base, &pid, &db_key)?;
                update_session(&s_db, &id, &msg, results);
            }
            debug!("Over handle sync packed... {}, {}, {}", height, from, to);
        }
    }

    Ok(())
}

pub(crate) async fn broadcast(
    gid: &GroupChatId,
    global: &Arc<Global>,
    event: &LayerEvent,
    results: &mut HandleResult,
) -> Result<()> {
    let new_data = bincode::serialize(event)?;

    for mpid in global.layer.read().await.group(gid)?.addrs.iter().skip(1) {
        let s = SendType::Event(0, *mpid, new_data.clone());
        results.layers.push((GROUP_CHAT_ID, s));
        debug!("--- DEBUG broadcast to: {:?}", mpid);
    }

    Ok(())
}

// UPDATE SESSION.
pub(crate) fn update_session(s_db: &DStorage, id: &i64, msg: &Message, results: &mut HandleResult) {
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
            .push(session_last(&sid, &msg.datetime, &scontent, false));
    }
}

pub(crate) fn group_conn(addr: PeerId, gid: GroupChatId, results: &mut HandleResult) {
    let data = bincode::serialize(&LayerConnect(gid)).unwrap_or(vec![]);
    let msg = SendType::Connect(0, Peer::peer(addr), data);
    results.layers.push((GROUP_CHAT_ID, msg));
}

fn sync(gid: GroupChatId, addr: PeerId, height: i64) -> SendType {
    let data = bincode::serialize(&LayerEvent::SyncReq(gid, height + 1)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}

fn sync_online(gid: GroupChatId, addr: PeerId) -> SendType {
    let data = bincode::serialize(&LayerEvent::MemberOnlineSync(gid)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}
