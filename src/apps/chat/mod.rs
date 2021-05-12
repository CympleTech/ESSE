mod layer;
mod models;

pub(crate) mod rpc;
pub(crate) use layer::chat_conn;
pub(crate) use layer::handle as layer_handle;
pub(crate) use layer::LayerEvent;
pub(crate) use models::{Friend, Message, MessageType, NetworkMessage, Request};
pub(crate) use rpc::new_rpc_handler;
