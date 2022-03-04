use chat_types::MessageType;
use group_types::{Event, GroupChatId, LayerConnect, LayerEvent, LayerResult, GROUP_CHAT_ID};
use std::sync::Arc;
use tdn::types::{
    message::{RecvType, SendType},
    primitives::{HandleResult, Peer, PeerId, Result},
};
use tdn_storage::local::DStorage;

use crate::apps::chat::Friend;
use crate::global::Global;
use crate::layer::Layer;
use crate::rpc::{
    session_close, session_connect, session_last, session_lost, session_suspend,
    session_update_name,
};
use crate::session::{connect_session, Session, SessionType};
use crate::storage::{delete_avatar, group_db, session_db, write_avatar_sync};

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
                let db_key = global.group.read().await.db_key(&pid)?;
                let db = group_db(&global.base, &pid, &db_key)?;
                let s_db = session_db(&global.base, &pid, &db_key)?;

                let group = GroupChat::close(&db, &gid, &peer.id)?;
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
        RecvType::Leave(..) => {}
        RecvType::Event(addr, bytes) => {
            let event: LayerEvent = bincode::deserialize(&bytes)?;
            debug!("----------- DEBUG GROUP CHAT: SERVER GOT LAYER EVENT");
            //handle_server_event(fgid, addr, event, layer, &mut results).await?;
            debug!("----------- DEBUG GROUP CHAT: SERVER OVER LAYER EVENT");

            debug!("----------- DEBUG GROUP CHAT: PEER GOT LAYER EVENT");
            //handle_peer_event(ogid, addr, event, layer, &mut results).await?;
            debug!("----------- DEBUG GROUP CHAT: PEER OVER LAYER EVENT");
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }

        RecvType::Delivery(..) => {}
    }

    Ok(results)
}

async fn handle_connect(
    global: &Arc<Global>,
    peer: &Peer,
    gid: GroupChatId,
    results: &mut HandleResult,
) -> Result<()> {
    let (height, sid, id) = global.layer.read().await.group(&gid)?.info();

    let pid = global.pid().await;
    let db_key = global.group.read().await.db_key(&pid)?;
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
    broadcast(&gid, global, &data, results).await;
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
    let db_key = global.group.read().await.db_key(&pid)?;
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

// async fn handle_server_event(
//     fgid: GroupId,
//     addr: PeerId,
//     event: LayerEvent,
//     layer: &Arc<RwLock<Layer>>,
//     results: &mut HandleResult,
// ) -> Result<()> {
//     let gcd = event.gcd();
//     let base = layer.read().await.base().clone();
//     let (ogid, height, id) = layer.read().await.running(gcd)?.owner_height_id();
//     let db = layer.read().await.group.read().await.group_db(&ogid)?;

//     match event {
//         LayerEvent::Offline(gcd) => {
//             // 1. check member online.
//             if layer.write().await.remove_online(&gcd, &fgid).is_none() {
//                 return Ok(());
//             }

//             // 2. UI: offline the member.
//             if let Ok(mid) = Member::get_id(&db, &id, &fgid) {
//                 results.rpcs.push(rpc::member_offline(ogid, id, mid));
//             }

//             // 3. broadcast offline event.
//             broadcast(&LayerEvent::MemberOffline(gcd, fgid), layer, &gcd, results).await?;
//         }
//         LayerEvent::GroupName(gcd, name) => {
//             // 1. update group name
//             let _ = GroupChat::update_name(&db, &id, &name)?;
//             // 2. UI: update
//             results.rpcs.push(rpc::group_name(ogid, &id, &name));
//             if let Ok(sid) = Session::update_name_by_id(
//                 &layer.read().await.group.read().await.session_db(&ogid)?,
//                 &id,
//                 &SessionType::Group,
//                 &name,
//             ) {
//                 results.rpcs.push(session_update_name(ogid, &sid, &name));
//             }
//             // 3. broadcast
//             broadcast(&LayerEvent::GroupName(gcd, name), layer, &gcd, results).await?;
//         }
//         LayerEvent::Sync(gcd, _, event) => {
//             match event {
//                 Event::MemberJoin(mgid, maddr, mname, mavatar) => {
//                     let mdid_res = Member::get_id(&db, &id, &mgid);
//                     let h = layer.write().await.running_mut(&gcd)?.increased();
//                     let new_e = Event::MemberJoin(mgid, maddr, mname.clone(), mavatar.clone());

//                     if let Ok(mdid) = mdid_res {
//                         Member::update(&db, &h, &mdid, &maddr, &mname)?;
//                         if mavatar.len() > 0 {
//                             write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
//                         }
//                         let mem = Member::info(mdid, id, mgid, maddr, mname);
//                         results.rpcs.push(rpc::member_join(ogid, &mem));
//                     } else {
//                         let mut member = Member::new(h, id, mgid, maddr, mname);
//                         member.insert(&db)?;
//                         if mavatar.len() > 0 {
//                             write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
//                         }
//                         results.rpcs.push(rpc::member_join(ogid, &member));
//                     }

//                     // broadcast
//                     GroupChat::add_height(&db, id, h)?;
//                     broadcast(&LayerEvent::Sync(gcd, h, new_e), layer, &gcd, results).await?;
//                 }
//                 Event::MemberLeave(mgid) => {
//                     let mdid = Member::get_id(&db, &id, &mgid)?;
//                     let h = layer.write().await.running_mut(&gcd)?.increased();
//                     Member::leave(&db, &mdid, &h)?;

//                     // check mid is my chat friend. if not, delete avatar.
//                     let s_db = &layer.read().await.group.read().await.chat_db(&ogid)?;
//                     if Friend::get_id(&s_db, &mgid).is_err() {
//                         let _ = delete_avatar(&base, &ogid, &mgid).await;
//                     }
//                     results.rpcs.push(rpc::member_leave(ogid, id, mdid));

//                     // broadcast
//                     GroupChat::add_height(&db, id, h)?;
//                     broadcast(&LayerEvent::Sync(gcd, h, event), layer, &gcd, results).await?;
//                 }
//                 Event::MessageCreate(mgid, nmsg, mtime) => {
//                     debug!("Sync: create message start");
//                     let _mdid = Member::get_id(&db, &id, &mgid)?;

//                     let new_e = Event::MessageCreate(mgid, nmsg.clone(), mtime);
//                     let new_h = layer.write().await.running_mut(&gcd)?.increased();
//                     broadcast(&LayerEvent::Sync(gcd, new_h, new_e), layer, &gcd, results).await?;
//                     GroupChat::add_height(&db, id, new_h)?;

//                     let msg = handle_network_message(
//                         &layer.read().await.group,
//                         new_h,
//                         id,
//                         mgid,
//                         &ogid,
//                         nmsg,
//                         mtime,
//                         &base,
//                         results,
//                     )
//                     .await?;
//                     results.rpcs.push(rpc::message_create(ogid, &msg));
//                     debug!("Sync: create message ok");

//                     // UPDATE SESSION.
//                     if let Ok(s_db) = layer.read().await.group.read().await.session_db(&ogid) {
//                         update_session(&s_db, &ogid, &id, &msg, results);
//                     }
//                 }
//             }
//         }
//         LayerEvent::MemberOnlineSync(gcd) => {
//             let onlines = layer
//                 .read()
//                 .await
//                 .running(&gcd)?
//                 .onlines()
//                 .iter()
//                 .map(|(g, a)| (**g, **a))
//                 .collect();
//             let event = LayerEvent::MemberOnlineSyncResult(gcd, onlines);
//             let data = bincode::serialize(&event).unwrap_or(vec![]);
//             let s = SendType::Event(0, addr, data);
//             add_server_layer(results, fgid, s);
//         }
//         LayerEvent::SyncReq(gcd, from) => {
//             debug!("Got sync request. height: {} from: {}", height, from);

//             if height >= from {
//                 let to = if height - from > 20 {
//                     from + 20
//                 } else {
//                     height
//                 };

//                 let (members, leaves) = Member::sync(&base, &ogid, &db, &id, &from, &to).await?;
//                 let messages = Message::sync(&base, &ogid, &db, &id, &from, &to).await?;
//                 let event = LayerEvent::SyncRes(gcd, height, from, to, members, leaves, messages);
//                 let data = bincode::serialize(&event).unwrap_or(vec![]);
//                 let s = SendType::Event(0, addr, data);
//                 add_server_layer(results, fgid, s);
//                 debug!("Sended sync request results. from: {}, to: {}", from, to);
//             }
//         }
//         LayerEvent::Suspend(..) => {}
//         LayerEvent::Actived(..) => {}
//         _ => error!("group server handle event nerver here"),
//     }

//     Ok(())
// }

// async fn handle_peer_event(
//     ogid: GroupId,
//     addr: PeerId,
//     event: LayerEvent,
//     layer: &Arc<RwLock<Layer>>,
//     results: &mut HandleResult,
// ) -> Result<()> {
//     let base = layer.read().await.base().clone();
//     let gcd = event.gcd();
//     let (sid, id) = layer.read().await.get_running_remote_id(&ogid, gcd)?;
//     let db = layer.read().await.group.read().await.group_db(&ogid)?;

//     match event {
//         LayerEvent::Offline(gcd) => {
//             // 1. offline group chat.
//             layer
//                 .write()
//                 .await
//                 .running_mut(&ogid)?
//                 .check_offline(&gcd, &addr);

//             // 2. UI: offline the session.
//             results.rpcs.push(session_lost(ogid, &sid));
//         }
//         LayerEvent::Suspend(gcd) => {
//             if layer
//                 .write()
//                 .await
//                 .running_mut(&ogid)?
//                 .suspend(&gcd, false, true)?
//             {
//                 results.rpcs.push(session_suspend(ogid, &sid));
//             }
//         }
//         LayerEvent::Actived(gcd) => {
//             let _ = layer.write().await.running_mut(&ogid)?.active(&gcd, false);
//             results.rpcs.push(session_connect(ogid, &sid, &addr));
//         }
//         LayerEvent::MemberOnline(_gcd, mgid, maddr) => {
//             if let Ok(mid) = Member::addr_update(&db, &id, &mgid, &maddr) {
//                 results.rpcs.push(rpc::member_online(ogid, id, mid, &maddr));
//             }
//         }
//         LayerEvent::MemberOffline(_gcd, mgid) => {
//             if let Ok(mid) = Member::get_id(&db, &id, &mgid) {
//                 results.rpcs.push(rpc::member_offline(ogid, id, mid));
//             }
//         }
//         LayerEvent::MemberOnlineSyncResult(_gcd, onlines) => {
//             for (mgid, maddr) in onlines {
//                 if let Ok(mid) = Member::addr_update(&db, &id, &mgid, &maddr) {
//                     results.rpcs.push(rpc::member_online(ogid, id, mid, &maddr));
//                 }
//             }
//         }
//         LayerEvent::GroupName(_gcd, name) => {
//             let _ = GroupChat::update_name(&db, &id, &name)?;
//             results.rpcs.push(rpc::group_name(ogid, &id, &name));
//             let _ = Session::update_name(
//                 &layer.read().await.group.read().await.session_db(&ogid)?,
//                 &sid,
//                 &name,
//             );
//             results.rpcs.push(session_update_name(ogid, &sid, &name));
//         }
//         LayerEvent::GroupClose(_gcd) => {
//             let group = GroupChat::close(&db, &gcd)?;
//             let sid = Session::close(
//                 &layer.read().await.group.read().await.session_db(&ogid)?,
//                 &group.id,
//                 &SessionType::Group,
//             )?;
//             results.rpcs.push(session_close(ogid, &sid));
//         }
//         LayerEvent::Sync(_gcd, height, event) => {
//             debug!("Sync: handle height: {}", height);

//             match event {
//                 Event::MemberJoin(mgid, maddr, mname, mavatar) => {
//                     let mdid_res = Member::get_id(&db, &id, &mgid);
//                     if let Ok(mdid) = mdid_res {
//                         Member::update(&db, &height, &mdid, &maddr, &mname)?;
//                         if mavatar.len() > 0 {
//                             write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
//                         }
//                         let mem = Member::info(mdid, id, mgid, maddr, mname);
//                         results.rpcs.push(rpc::member_join(ogid, &mem));
//                     } else {
//                         let mut member = Member::new(height, id, mgid, maddr, mname);
//                         member.insert(&db)?;
//                         if mavatar.len() > 0 {
//                             write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
//                         }
//                         results.rpcs.push(rpc::member_join(ogid, &member));
//                     }

//                     // save consensus.
//                     GroupChat::add_height(&db, id, height)?;
//                 }
//                 Event::MemberLeave(mgid) => {
//                     let mdid = Member::get_id(&db, &id, &mgid)?;
//                     Member::leave(&db, &height, &mdid)?;

//                     // check mid is my chat friend. if not, delete avatar.
//                     let s_db = &layer.read().await.group.read().await.chat_db(&ogid)?;
//                     if Friend::get_id(&s_db, &mgid).is_err() {
//                         let _ = delete_avatar(&base, &ogid, &mgid).await;
//                     }
//                     results.rpcs.push(rpc::member_leave(ogid, id, mdid));

//                     // save consensus.
//                     GroupChat::add_height(&db, id, height)?;
//                 }
//                 Event::MessageCreate(mgid, nmsg, mtime) => {
//                     debug!("Sync: create message start");
//                     let _mdid = Member::get_id(&db, &id, &mgid)?;

//                     let msg = handle_network_message(
//                         &layer.read().await.group,
//                         height,
//                         id,
//                         mgid,
//                         &ogid,
//                         nmsg,
//                         mtime,
//                         &base,
//                         results,
//                     )
//                     .await?;
//                     results.rpcs.push(rpc::message_create(ogid, &msg));

//                     GroupChat::add_height(&db, id, height)?;
//                     debug!("Sync: create message ok");

//                     // UPDATE SESSION.
//                     if let Ok(s_db) = layer.read().await.group.read().await.session_db(&ogid) {
//                         update_session(&s_db, &ogid, &id, &msg, results);
//                     }
//                 }
//             }
//         }
//         LayerEvent::SyncRes(gcd, height, from, to, adds, leaves, messages) => {
//             if to >= height {
//                 // when last packed sync, start sync online members.
//                 add_layer(results, ogid, sync_online(gcd, addr));
//             }

//             debug!("Start handle sync packed... {}, {}, {}", height, from, to);
//             let mut last_message = None;

//             for (height, mgid, maddr, mname, mavatar) in adds {
//                 let mdid_res = Member::get_id(&db, &id, &mgid);
//                 if let Ok(mdid) = mdid_res {
//                     Member::update(&db, &height, &mdid, &maddr, &mname)?;
//                     if mavatar.len() > 0 {
//                         write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
//                     }
//                     let mem = Member::info(mdid, id, mgid, maddr, mname);
//                     results.rpcs.push(rpc::member_join(ogid, &mem));
//                 } else {
//                     let mut member = Member::new(height, id, mgid, maddr, mname);
//                     member.insert(&db)?;
//                     if mavatar.len() > 0 {
//                         write_avatar_sync(&base, &ogid, &mgid, mavatar)?;
//                     }
//                     results.rpcs.push(rpc::member_join(ogid, &member));
//                 }
//             }

//             for (height, mgid) in leaves {
//                 if let Ok(mdid) = Member::get_id(&db, &id, &mgid) {
//                     Member::leave(&db, &height, &mdid)?;
//                     // check mid is my chat friend. if not, delete avatar.
//                     let s_db = &layer.read().await.group.read().await.chat_db(&ogid)?;
//                     if Friend::get_id(&s_db, &mgid).is_err() {
//                         let _ = delete_avatar(&base, &ogid, &mgid).await;
//                     }
//                     results.rpcs.push(rpc::member_leave(ogid, id, mdid));
//                 }
//             }

//             for (height, mgid, nm, time) in messages {
//                 if let Ok(msg) = handle_network_message(
//                     &layer.read().await.group,
//                     height,
//                     id,
//                     mgid,
//                     &ogid,
//                     nm,
//                     time,
//                     &base,
//                     results,
//                 )
//                 .await
//                 {
//                     results.rpcs.push(rpc::message_create(ogid, &msg));
//                     last_message = Some(msg);
//                 }
//             }

//             if to < height {
//                 add_layer(results, ogid, sync(gcd, addr, to + 1));
//             }

//             // update group chat height.
//             GroupChat::add_height(&db, id, to)?;

//             // UPDATE SESSION.
//             if let Some(msg) = last_message {
//                 if let Ok(s_db) = layer.read().await.group.read().await.session_db(&ogid) {
//                     update_session(&s_db, &ogid, &id, &msg, results);
//                 }
//             }
//             debug!("Over handle sync packed... {}, {}, {}", height, from, to);
//         }
//         _ => error!("group peer handle event nerver here"),
//     }

//     Ok(())
// }

pub(crate) async fn broadcast(
    gid: &GroupChatId,
    global: &Arc<Global>,
    event: &LayerEvent,
    results: &mut HandleResult,
) -> Result<()> {
    let new_data = bincode::serialize(event)?;

    for mpid in global.layer.read().await.group(gid)?.addrs.iter() {
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

pub(crate) fn group_conn(addr: Peer, gid: GroupChatId) -> SendType {
    let data = bincode::serialize(&LayerConnect(gid)).unwrap_or(vec![]);
    SendType::Connect(0, addr, data)
}

fn sync(gid: GroupChatId, addr: PeerId, height: i64) -> SendType {
    let data = bincode::serialize(&LayerEvent::SyncReq(gid, height + 1)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}

fn sync_online(gid: GroupChatId, addr: PeerId) -> SendType {
    let data = bincode::serialize(&LayerEvent::MemberOnlineSync(gid)).unwrap_or(vec![]);
    SendType::Event(0, addr, data)
}
