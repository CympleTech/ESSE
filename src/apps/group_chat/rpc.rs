use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::SendType,
    primitive::{HandleResult, PeerAddr},
    rpc::{json, rpc_response, RpcHandler, RpcParam},
};
use tdn_did::Proof;

use group_chat_types::{CheckType, Event, GroupType, JoinProof, LayerEvent};

use crate::apps::chat::{Friend, MessageType};
use crate::rpc::{session_close, session_delete, RpcState};
use crate::session::{Session, SessionType};
use crate::storage::{chat_db, group_chat_db, session_db, write_avatar};

use super::add_layer;
use super::models::{to_network_message, GroupChat, GroupChatKey, Member, Message, Request};

#[inline]
pub(crate) fn create_check(mgid: GroupId, ct: CheckType, supported: Vec<GroupType>) -> RpcParam {
    let s: Vec<u32> = supported.iter().map(|v| v.to_u32()).collect();
    rpc_response(0, "group-chat-check", json!([ct.to_u32(), s]), mgid)
}

#[inline]
pub(crate) fn create_result(mgid: GroupId, gid: i64, ok: bool) -> RpcParam {
    rpc_response(0, "group-chat-result", json!([gid, ok]), mgid)
}

#[inline]
pub(crate) fn group_create(mgid: GroupId, group: GroupChat) -> RpcParam {
    rpc_response(0, "group-chat-create", json!(group.to_rpc()), mgid)
}

#[inline]
pub(crate) fn request_create(mgid: GroupId, req: &Request) -> RpcParam {
    rpc_response(0, "group-chat-join", json!(req.to_rpc()), mgid)
}

#[inline]
pub(crate) fn request_handle(mgid: GroupId, id: i64, ok: bool, efficacy: bool) -> RpcParam {
    rpc_response(0, "group-chat-join-handle", json!([id, ok, efficacy]), mgid)
}

#[inline]
pub(crate) fn member_join(mgid: GroupId, member: Member) -> RpcParam {
    rpc_response(0, "group-chat-member-join", json!(member.to_rpc()), mgid)
}

#[inline]
pub(crate) fn member_info(mgid: GroupId, id: i64, addr: PeerAddr, name: String) -> RpcParam {
    rpc_response(
        0,
        "group-chat-member-info",
        json!([id, addr.to_hex(), name]),
        mgid,
    )
}

#[inline]
pub(crate) fn member_online(mgid: GroupId, gid: i64, mid: GroupId, maddr: PeerAddr) -> RpcParam {
    rpc_response(
        0,
        "group-chat-member-online",
        json!([gid, mid.to_hex(), maddr.to_hex()]),
        mgid,
    )
}

#[inline]
pub(crate) fn member_offline(mgid: GroupId, gid: i64, mid: GroupId) -> RpcParam {
    rpc_response(
        0,
        "group-chat-member-offline",
        json!([gid, mid.to_hex()]),
        mgid,
    )
}

#[inline]
pub(crate) fn message_create(mgid: GroupId, msg: &Message) -> RpcParam {
    rpc_response(0, "group-chat-message-create", json!(msg.to_rpc()), mgid)
}

#[inline]
fn group_list(groups: Vec<GroupChat>) -> RpcParam {
    let mut results = vec![];
    for group in groups {
        results.push(group.to_rpc());
    }

    json!(results)
}

#[inline]
fn request_list(requests: Vec<Request>) -> RpcParam {
    let mut results = vec![];
    for request in requests {
        results.push(request.to_rpc());
    }

    json!(results)
}

#[inline]
fn detail_list(members: Vec<Member>, messages: Vec<Message>) -> RpcParam {
    let mut member_results = vec![];
    for m in members {
        member_results.push(m.to_rpc());
    }

    let mut message_results = vec![];
    for msg in messages {
        message_results.push(msg.to_rpc());
    }

    json!([member_results, message_results])
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("group-chat-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "group-chat-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let layer_lock = state.layer.read().await;
            let db = group_chat_db(&layer_lock.base, &gid)?;
            Ok(HandleResult::rpc(group_list(GroupChat::all(&db)?)))
        },
    );

    handler.add_method(
        "group-chat-request-list",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let is_all = params[0].as_bool()?;
            let layer_lock = state.layer.read().await;
            let db = group_chat_db(&layer_lock.base, &gid)?;
            Ok(HandleResult::rpc(request_list(Request::list(&db, is_all)?)))
        },
    );

    handler.add_method(
        "group-chat-detail",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let g_did = params[0].as_i64()?;
            let db = group_chat_db(state.layer.read().await.base(), &gid)?;
            let members = Member::all(&db, &g_did)?;
            let messages = Message::all(&db, &g_did)?;
            Ok(HandleResult::rpc(detail_list(members, messages)))
        },
    );

    handler.add_method(
        "group-chat-check",
        |gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let addr = PeerAddr::from_hex(params[0].as_str()?)?;

            let mut results = HandleResult::new();
            let data = postcard::to_allocvec(&LayerEvent::Check).unwrap_or(vec![]);
            let s = SendType::Event(0, addr, data);
            add_layer(&mut results, gid, s);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gtype = GroupType::from_u32(params[0].as_i64()? as u32);
            let my_name = params[1].as_str()?.to_owned();
            let addr = PeerAddr::from_hex(params[2].as_str()?)?;
            let name = params[3].as_str()?.to_owned();
            let bio = params[4].as_str()?.to_owned();
            let need_agree = params[5].as_bool()?;
            let avatar = params[6].as_str()?;
            let avatar_bytes = base64::decode(avatar).unwrap_or(vec![]);

            let base = state.layer.read().await.base().clone();
            let db = group_chat_db(&base, &gid)?;
            let mut gc = GroupChat::new(gid, gtype, addr, name, bio, need_agree);
            let gcd = gc.g_id;

            // save db
            let me = state.group.read().await.clone_user(&gid)?;
            gc.insert(&db)?;
            Member::new(gc.id, gid, me.addr, me.name, true, gc.datetime).insert(&db)?;

            // save avatar
            let _ = write_avatar(&base, &gid, &gcd, &avatar_bytes).await;

            let mut results = HandleResult::new();
            // add to rpcs.
            results.rpcs.push(json!(gc.to_rpc()));
            let info = gc.to_group_info(my_name, avatar_bytes);

            // TODO create proof.
            let proof: Proof = Default::default();

            let data = postcard::to_allocvec(&LayerEvent::Create(info, proof)).unwrap_or(vec![]);
            let s = SendType::Event(0, addr, data);
            add_layer(&mut results, gid, s);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-resend",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;
            let mname = params[1].as_str()?.to_owned();

            let db = group_chat_db(state.layer.read().await.base(), &gid)?;
            let gc = GroupChat::get_id(&db, &id)??;
            drop(db);

            // TODO load avatar
            let avatar = vec![];
            let addr = gc.g_addr;
            let info = gc.to_group_info(mname, avatar);

            // TODO create proof.
            let proof: Proof = Default::default();

            let data = postcard::to_allocvec(&LayerEvent::Create(info, proof)).unwrap_or(vec![]);
            let s = SendType::Event(0, addr, data);
            let mut results = HandleResult::new();
            add_layer(&mut results, gid, s);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-join",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gtype = GroupType::from_u32(params[0].as_i64()? as u32);
            let gcd = GroupId::from_hex(params[1].as_str()?)?;
            let gaddr = PeerAddr::from_hex(params[2].as_str()?)?;
            let gname = params[3].as_str()?.to_owned();
            let gremark = params[4].as_str()?;
            let gproof = params[5].as_str()?;
            let proof = Proof::from_hex(gproof).unwrap_or(Proof::default());
            let gkey = params[6].as_str()?;
            let key = GroupChatKey::from_hex(gkey).unwrap_or(GroupChatKey::new(vec![]));

            let db = group_chat_db(state.layer.read().await.base(), &gid)?;
            if GroupChat::get(&db, &gcd)?.is_some() {
                debug!("Had joined this group.");
                return Ok(HandleResult::new()); // had join this group.
            }

            let mut results = HandleResult::new();
            // check request is exsit.
            if !Request::exist(&db, &gcd)? {
                let mut request = Request::new_by_me(gcd, gaddr, gname, gremark.to_owned(), key);
                request.insert(&db)?;
                results.rpcs.push(request.to_rpc());
            } else {
                debug!("Had request again.");
            }
            drop(db);

            let me = state.group.read().await.clone_user(&gid)?;
            let join_proof = match gtype {
                GroupType::Encrypted => {
                    // remark is inviter did.
                    let _fgid = GroupId::from_hex(gremark)?;
                    // TODO
                    JoinProof::Zkp(proof)
                }
                GroupType::Private => {
                    // remark is inviter did.
                    let fgid = GroupId::from_hex(gremark)?;
                    JoinProof::Invite(fgid, proof, me.name, me.avatar)
                }
                GroupType::Open => JoinProof::Open(me.name, me.avatar),
            };

            let data =
                postcard::to_allocvec(&LayerEvent::Request(gcd, join_proof)).unwrap_or(vec![]);
            let s = SendType::Event(0, gaddr, data);
            add_layer(&mut results, gid, s);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-invite",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;
            let gcd = GroupId::from_hex(params[1].as_str()?)?;
            let ids: Vec<i64> = params[2]
                .as_array()?
                .iter()
                .filter_map(|v| v.as_i64())
                .collect();

            //
            let group_lock = state.group.read().await;
            let base = group_lock.base().clone();

            let chat = chat_db(&base, &gid)?;
            let group_db = group_chat_db(&base, &gid)?;

            let mut invites = vec![];
            for id in ids {
                let friend = Friend::get_id(&chat, id)??;
                if Member::get_id(&group_db, &id, &friend.gid).is_err() {
                    let proof = group_lock.prove_addr(&gid, &friend.gid.into())?;
                    invites.push((friend.id, friend.gid, friend.addr, proof));
                }
            }
            drop(chat);

            let gc = GroupChat::get_id(&group_db, &id)??;
            let tmp_name = gc.g_name.replace(";", "-;");

            let mut results = HandleResult::new();
            let mut layer_lock = state.layer.write().await;
            for (fid, fgid, mut faddr, proof) in invites {
                let contact_values = format!(
                    "{};;{};;{};;{};;{};;{}",
                    gc.g_type.to_u32(),
                    gcd.to_hex(),
                    gc.g_addr.to_hex(),
                    tmp_name,
                    proof.to_hex(),
                    gc.key.to_hex(),
                );

                // check if encrypted group type. need online.
                if gc.g_type == GroupType::Encrypted {
                    if let Ok(addr) = layer_lock.running(&gid)?.online(&fgid) {
                        faddr = addr;
                    } else {
                        continue;
                    }
                }

                let (msg, nw, _) = crate::apps::chat::LayerEvent::from_message(
                    &base,
                    gid,
                    fid,
                    MessageType::Invite,
                    contact_values,
                )
                .await?;
                let event = crate::apps::chat::LayerEvent::Message(msg.hash, nw);
                let s =
                    crate::apps::chat::event_message(&mut layer_lock, msg.id, gid, faddr, &event);
                results.layers.push((gid, fgid, s));
            }
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-request-handle",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gcd = GroupId::from_hex(params[0].as_str()?)?;
            let id = params[1].as_i64()?;
            let rid = params[2].as_i64()?;
            let ok = params[3].as_bool()?;

            let db = group_chat_db(state.layer.read().await.base(), &gid)?;
            let gc = GroupChat::get_id(&db, &id)??;

            let mut results = HandleResult::new();
            let data =
                postcard::to_allocvec(&LayerEvent::RequestResult(gcd, rid, ok)).unwrap_or(vec![]);
            let s = SendType::Event(0, gc.g_addr, data);
            add_layer(&mut results, gid, s);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-member-update",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64()?;
            let is_block = params[1].as_bool()?;

            let db = group_chat_db(state.layer.read().await.base(), &gid)?;
            Member::block(&db, &id, is_block)?;
            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "group-chat-message-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gcd = GroupId::from_hex(params[0].as_str()?)?;
            let m_type = MessageType::from_int(params[1].as_i64()?);
            let m_content = params[2].as_str()?;

            let addr = state.layer.read().await.running(&gid)?.online(&gcd)?;

            let mut results = HandleResult::new();
            let base = state.group.read().await.base().clone();
            let (nmsg, datetime) = to_network_message(&base, &gid, m_type, m_content).await?;
            let event = Event::MessageCreate(gid, nmsg, datetime);
            let data = postcard::to_allocvec(&LayerEvent::Sync(gcd, 0, event)).unwrap_or(vec![]);
            let msg = SendType::Event(0, addr, data);
            add_layer(&mut results, gid, msg);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-close",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gcd = GroupId::from_hex(params[0].as_str()?)?;
            let id = params[1].as_i64()?;

            let addr = state.layer.write().await.remove_online(&gid, &gcd)?;

            let mut results = HandleResult::new();
            let base = state.layer.read().await.base().clone();
            let sid = Session::close(&session_db(&base, &gid)?, &id, &SessionType::Group)?;
            results.rpcs.push(session_close(gid, &sid));

            let db = group_chat_db(&base, &gid)?;
            GroupChat::close(&db, &id)?;

            let event = Event::MemberLeave(gid);
            let data = postcard::to_allocvec(&LayerEvent::Sync(gcd, 0, event)).unwrap_or(vec![]);
            let msg = SendType::Event(0, addr, data);
            add_layer(&mut results, gid, msg);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gcd = GroupId::from_hex(params[0].as_str()?)?;
            let id = params[1].as_i64()?;

            let mut results = HandleResult::new();
            let base = state.layer.read().await.base().clone();
            let sid = Session::delete(&session_db(&base, &gid)?, &id, &SessionType::Group)?;
            results.rpcs.push(session_delete(gid, &sid));
            let db = group_chat_db(&base, &gid)?;
            if GroupChat::delete(&db, &id)? {
                let addr = state.layer.write().await.remove_online(&gid, &gcd)?;
                let event = Event::MemberLeave(gid);
                let data =
                    postcard::to_allocvec(&LayerEvent::Sync(gcd, 0, event)).unwrap_or(vec![]);
                let msg = SendType::Event(0, addr, data);
                add_layer(&mut results, gid, msg);
            }
            Ok(results)
        },
    );
}
