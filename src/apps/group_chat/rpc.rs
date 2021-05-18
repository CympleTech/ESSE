use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::SendType,
    primitive::{new_io_error, HandleResult, PeerAddr},
    rpc::{json, rpc_response, RpcHandler, RpcParam},
};
use tdn_did::Proof;

use group_chat_types::{CheckType, Event, GroupConnect, GroupType, JoinProof, LayerEvent};

use crate::apps::chat::MessageType;
use crate::rpc::RpcState;
use crate::storage::group_chat_db;

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
pub(crate) fn group_online(mgid: GroupId, gid: i64) -> RpcParam {
    rpc_response(0, "group-chat-online", json!([gid]), mgid)
}

#[inline]
pub(crate) fn group_offline(mgid: GroupId, fid: i64, gid: &GroupId) -> RpcParam {
    rpc_response(0, "group-chat-offline", json!([fid, gid.to_hex()]), mgid)
}

#[inline]
pub(crate) fn group_agree(mgid: GroupId, rid: i64, group: GroupChat) -> RpcParam {
    rpc_response(0, "group-chat-agree", json!([rid, group.to_rpc()]), mgid)
}

#[inline]
pub(crate) fn group_reject(mgid: GroupId, rid: i64) -> RpcParam {
    rpc_response(0, "group-chat-reject", json!([rid]), mgid)
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
pub(crate) fn member_offline(mgid: GroupId, gid: i64, mid: GroupId, maddr: PeerAddr) -> RpcParam {
    rpc_response(
        0,
        "group-chat-member-offline",
        json!([gid, mid.to_hex(), maddr.to_hex()]),
        mgid,
    )
}

#[inline]
pub(crate) fn message_create(mgid: GroupId, msg: Message) -> RpcParam {
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
            let mut groups = GroupChat::all(&db)?;
            drop(db);

            let gids: Vec<&GroupId> = groups.iter().map(|g| &g.g_id).collect();
            let onlines = layer_lock.merge_online(&gid, gids)?;
            for (index, online) in onlines.iter().enumerate() {
                groups[index].online = *online;
            }

            Ok(HandleResult::rpc(group_list(groups)))
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
            let data = postcard::to_allocvec(&GroupConnect::Check).unwrap_or(vec![]);
            let s = SendType::Connect(0, addr, None, None, data);
            add_layer(&mut results, gid, s);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let my_name = params[0].as_str()?.to_owned();
            let addr = PeerAddr::from_hex(params[1].as_str()?)?;
            let name = params[2].as_str()?.to_owned();
            let bio = params[3].as_str()?.to_owned();
            let need_agree = params[4].as_bool()?;
            let avatar = vec![];

            let db = group_chat_db(state.layer.read().await.base(), &gid)?;
            let mut gc = GroupChat::new(gid, GroupType::Common, addr, name, bio, need_agree);
            let _gcd = gc.g_id;

            // save db
            let me = state.group.read().await.clone_user(&gid)?;
            gc.insert(&db)?;
            Member::new(gc.id, gid, me.addr, me.name, true, gc.datetime).insert(&db)?;

            // TODO save avatar

            let mut results = HandleResult::new();
            // TODO add to rpcs.
            results.rpcs.push(json!(gc.to_rpc()));
            let info = gc.to_group_info(my_name, avatar);

            // TODO create proof.
            let proof: Proof = Default::default();

            let data = postcard::to_allocvec(&GroupConnect::Create(info, proof)).unwrap_or(vec![]);
            let s = SendType::Connect(0, addr, None, None, data);
            add_layer(&mut results, gid, s);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-join",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let gcd = GroupId::from_hex(params[0].as_str()?)?;
            let gaddr = PeerAddr::from_hex(params[1].as_str()?)?;
            let gname = params[2].as_str()?.to_owned();
            let gremark = params[3].as_str()?.to_owned();
            let gkey = params[4].as_str()?;
            let key = GroupChatKey::from_hex(gkey).unwrap_or(GroupChatKey::new(vec![]));

            let mut request = Request::new_by_me(gcd, gaddr, gname, gremark, key);
            let db = group_chat_db(state.layer.read().await.base(), &gid)?;
            request.insert(&db)?;
            drop(db);

            let mut results = HandleResult::rpc(request.to_rpc());
            let me = state.group.read().await.clone_user(&gid)?;
            let join_proof = JoinProof::Open(me.name, me.avatar);
            let data = postcard::to_allocvec(&GroupConnect::Join(request.gid, join_proof))
                .unwrap_or(vec![]);
            let s = SendType::Connect(0, request.addr, None, None, data);
            add_layer(&mut results, gid, s);
            Ok(results)
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
            let (nmsg, datetime) = to_network_message(m_type, m_content)?;
            let event = Event::MessageCreate(gid, nmsg, datetime);
            let data = postcard::to_allocvec(&LayerEvent::Sync(gcd, 0, event)).unwrap_or(vec![]);
            let msg = SendType::Event(0, addr, data);
            add_layer(&mut results, gid, msg);
            Ok(results)
        },
    );

    handler.add_method(
        "group-chat-readed",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let fid = params[0].as_i64()?;

            let db = group_chat_db(state.layer.read().await.base(), &gid)?;
            GroupChat::readed(&db, fid)?;
            drop(db);

            Ok(HandleResult::new())
        },
    );
}
