use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::RecvType,
    primitive::{HandleResult, Result},
};
use tokio::sync::RwLock;

use cloud_types::LayerServerEvent;

use crate::layer::Layer;

pub(crate) async fn handle(
    _layer: &Arc<RwLock<Layer>>,
    _ogid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
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
            let LayerServerEvent(_event, _proof) = bincode::deserialize(&bytes)?;
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {
            // MAYBE
        }
    }

    Ok(results)
}
