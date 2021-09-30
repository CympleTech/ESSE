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

            let db = domain_db(layer.read().await.base(), &ogid)?;

            match event {
                ServerEvent::Status(name, support_request) => {
                    println!(
                        "------ DEBUG DOMAIN SERVICE IS {}, request: {}",
                        name, support_request
                    );

                    let mut provider = Provider::get_by_addr(&db, &addr)?;
                    provider.ok(&db, name, support_request);

                    // TODO UI: add new provider.
                }
                ServerEvent::Result(name, is_ok) => {
                    let provider = Provider::get_by_addr(&db, &addr)?;
                    let mut name = Name::get_by_name_provider(&db, &name, &provider.id)?;

                    if is_ok {
                        //name.ok(&db);
                    } else {
                        //name.delete(&db);
                    }

                    // TODO UI: add new regsiter name.
                }
                ServerEvent::Info(_uname, _ugid, _uaddr, _ubio, _uavatar) => {
                    // TODO UI: show search result.
                }
                ServerEvent::None(_name) => {
                    // TODO UI: show search result.
                }
                ServerEvent::Response(_ugid, _uname, _is_ok) => {}
            }
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {
            // MAYBE
        }
    }

    Ok(results)
}
