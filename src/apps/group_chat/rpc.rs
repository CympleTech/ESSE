use group_chat_types::{Event, GroupConnect, GroupEvent, GroupInfo, GroupResult, GroupType};
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::SendType,
    primitive::{new_io_error, HandleResult, PeerAddr},
    rpc::{json, rpc_response, RpcHandler, RpcParam},
};

//use crate::group::GroupEvent;
use super::add_layer;
use crate::rpc::RpcState;

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("group-chat-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

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
}
