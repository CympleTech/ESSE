use std::path::PathBuf;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::{RecvType, SendType},
    primitive::{HandleResult, PeerAddr, Result},
};
use tokio::sync::RwLock;

use domain_types::{LayerPeerEvent, LayerServerEvent, ServerEvent};
use tdn_did::Proof;
use tdn_storage::local::DStorage;

use crate::layer::{Layer, Online};
use crate::storage::domain_db;

use super::models::{Name, Provider};
use super::{add_layer, rpc};

pub(crate) async fn handle(
    layer: &Arc<RwLock<Layer>>,
    ogid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    let results = HandleResult::new();

    match msg {
        RecvType::Connect(..)
        | RecvType::Leave(..)
        | RecvType::Result(..)
        | RecvType::ResultConnect(..)
        | RecvType::Stream(..) => {
            info!("domain message nerver to here.")
        }
        RecvType::Event(addr, bytes) => {
            // server & client handle it.
            let LayerServerEvent(event, proof) = bincode::deserialize(&bytes)?;

            match event {
                ServerEvent::Status => {
                    println!("------ DEBUG DOMAIN SERVICE IS OK");
                }
                ServerEvent::Result(_name, _is_ok) => {}
                ServerEvent::Info(_uname, _ugid, _uaddr, _ubio, _uavatar) => {}
                ServerEvent::None(_name) => {}
                ServerEvent::Response(_ugid, _uname, _is_ok) => {}
            }
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {
            // MAYBE
        }
    }

    Ok(results)
}
