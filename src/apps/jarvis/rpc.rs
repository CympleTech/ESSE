use rand::Rng;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::SendMessage,
    primitive::{HandleResult, Result},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};
use tdn_did::Language;
use tdn_storage::local::DStorage;
use tokio::sync::mpsc::Sender;

use chat_types::MessageType;

use crate::account::lang_from_i64;
use crate::apps::chat::raw_to_network_message;
use crate::rpc::RpcState;
use crate::storage::jarvis_db;
use crate::utils::answer::load_answer;

use super::models::Message;

async fn reply(
    sender: Sender<SendMessage>,
    db: DStorage,
    gid: GroupId,
    lang: Language,
    msg: Message,
) -> Result<()> {
    // analyse questions.
    let content = if msg.m_type == MessageType::String {
        if msg.content.ends_with("?") || msg.content.ends_with("？") {
            // answer book. ascill ? and SBC case.
            let answer = rand::thread_rng().gen_range(0..171);
            load_answer(&lang, answer)
        } else {
            msg.content
        }
    } else {
        // save
        msg.content
    };

    let mut reply = Message::new(msg.m_type, content, false);
    reply.insert(&db)?;

    let res = rpc_response(0, "jarvis-create", reply.to_rpc(), gid);
    sender.send(SendMessage::Rpc(0, res, true)).await?;
    Ok(())
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method(
        "jarvis-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let db = jarvis_db(state.layer.read().await.base(), &gid)?;
            let devices = Message::list(&db)?;
            db.close()?;
            let mut results = vec![];
            for device in devices {
                results.push(device.to_rpc());
            }
            Ok(HandleResult::rpc(json!(results)))
        },
    );

    handler.add_method(
        "jarvis-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let lang = lang_from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let m_type = MessageType::from_int(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let content = params[2].as_str().ok_or(RpcError::ParseError)?;

            let group_lock = state.group.read().await;
            let base = group_lock.base().clone();
            let sender = group_lock.sender();
            drop(group_lock);

            let (_, raw) = raw_to_network_message(&base, &gid, &m_type, content).await?;
            let mut msg = Message::new(m_type, raw, true);
            let db = jarvis_db(&base, &gid)?;
            msg.insert(&db)?;

            let results = HandleResult::rpc(msg.to_rpc());
            tokio::spawn(reply(sender, db, gid, lang, msg));

            Ok(results)
        },
    );

    handler.add_method(
        "jarvis-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let db = jarvis_db(state.layer.read().await.base(), &gid)?;
            Message::delete(&db, id)?;
            db.close()?;
            Ok(HandleResult::new())
        },
    );
}
