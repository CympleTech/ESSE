mod models;

pub(crate) mod rpc;
pub(crate) use models::{Friend, Message, MessageType, NetworkMessage, Request};
pub(crate) use rpc::new_rpc_handler;
