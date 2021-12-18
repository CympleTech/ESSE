mod layer;
mod models;

pub(crate) mod rpc;
pub(crate) use layer::handle;
pub(crate) use layer::LayerEvent;
pub(crate) use layer::{chat_conn, event_message};
pub(crate) use models::{Friend, Message, MessageType, NetworkMessage, Request};
pub(crate) use rpc::new_rpc_handler;
