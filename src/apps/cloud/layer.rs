use cloud_types::LayerServerEvent;
use std::sync::Arc;
use tdn::types::{
    message::RecvType,
    primitives::{HandleResult, Result},
};

use crate::global::Global;

pub(crate) async fn handle(msg: RecvType, global: &Arc<Global>) -> Result<HandleResult> {
    let results = HandleResult::new();

    match msg {
        RecvType::Connect(..)
        | RecvType::Leave(..)
        | RecvType::Result(..)
        | RecvType::ResultConnect(..)
        | RecvType::Stream(..) => {
            info!("cloud message nerver to here.")
        }
        RecvType::Event(_addr, bytes) => {
            let LayerServerEvent(_event) = bincode::deserialize(&bytes)?;
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {
            // MAYBE
        }
    }

    Ok(results)
}
