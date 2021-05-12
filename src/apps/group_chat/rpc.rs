use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::SendType,
    primitive::{new_io_error, HandleResult, PeerAddr},
    rpc::{json, rpc_response, RpcHandler, RpcParam},
};
use tdn_did::Proof;

use group_chat_types::{CheckType, GroupConnect, GroupType};

use crate::rpc::RpcState;
use crate::storage::group_chat_db;

use super::add_layer;
use super::models::{GroupChat, Member, Message};

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
            let addr = PeerAddr::from_hex(params[0].as_str()?)
                .map_err(|_e| new_io_error("PeerAddr invalid!"))?;
            println!("addr: {}", addr.to_hex());

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
            let addr = PeerAddr::from_hex(params[0].as_str()?)
                .map_err(|_e| new_io_error("PeerAddr invalid!"))?;
            let name = params[1].as_str()?.to_owned();
            let bio = params[2].as_str()?.to_owned();
            let need_agree = params[3].as_bool()?;
            let avatar = vec![];
            println!("Create: {}, {}, {}", name, bio, need_agree);

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
            let info = gc.to_group_info(avatar);

            // TODO create proof.
            let proof: Proof = Default::default();

            let data = postcard::to_allocvec(&GroupConnect::Create(info, proof)).unwrap_or(vec![]);
            let s = SendType::Connect(0, addr, None, None, data);
            add_layer(&mut results, gid, s);
            Ok(results)
        },
    );
}
