use rand::Rng;
use std::sync::Arc;
use tdn::types::{
    message::RpcSendMessage,
    primitives::{HandleResult, Result},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};
use tdn_did::Language;
use tdn_storage::local::DStorage;
use tokio::sync::mpsc::Sender;

use chat_types::MessageType;

use crate::account::lang_from_i64;
use crate::apps::chat::raw_to_network_message;
use crate::global::Global;
use crate::storage::jarvis_db;
use crate::utils::answer::load_answer;

use super::models::Message;

async fn reply(
    sender: Sender<RpcSendMessage>,
    db: DStorage,
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

    let res = rpc_response(0, "jarvis-create", reply.to_rpc());
    sender.send(RpcSendMessage(0, res, true)).await?;
    Ok(())
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<Global>) {
    handler.add_method(
        "jarvis-list",
        |_params: Vec<RpcParam>, state: Arc<Global>| async move {
            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = jarvis_db(&state.base, &pid, &db_key)?;
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
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let lang = lang_from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let m_type = MessageType::from_int(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let content = params[2].as_str().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = jarvis_db(&state.base, &pid, &db_key)?;

            let (_, raw) =
                raw_to_network_message(&pid, &state.base, &db_key, &m_type, content).await?;
            let mut msg = Message::new(m_type, raw, true);
            msg.insert(&db)?;

            let results = HandleResult::rpc(msg.to_rpc());
            tokio::spawn(reply(state.rpc_send.clone(), db, lang, msg));

            Ok(results)
        },
    );

    handler.add_method(
        "jarvis-delete",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let pid = state.pid().await;
            let db_key = state.group.read().await.db_key(&pid)?;
            let db = jarvis_db(&state.base, &pid, &db_key)?;
            Message::delete(&db, id)?;
            db.close()?;
            Ok(HandleResult::new())
        },
    );
}
