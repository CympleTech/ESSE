use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::RecvType,
    primitive::{HandleResult, Result},
};
use tokio::sync::RwLock;

use domain_types::{LayerServerEvent, ServerEvent};

use crate::layer::Layer;

use super::models::{Name, Provider};
use super::rpc;

pub(crate) async fn handle(
    layer: &Arc<RwLock<Layer>>,
    ogid: GroupId,
    msg: RecvType,
) -> Result<HandleResult> {
    let mut results = HandleResult::new();

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
            let LayerServerEvent(event, _proof) = bincode::deserialize(&bytes)?;

            let db = layer.read().await.group.read().await.domain_db(&ogid)?;

            match event {
                ServerEvent::Status(name, support_request) => {
                    let mut provider = Provider::get_by_addr(&db, &addr)?;
                    provider.ok(&db, name, support_request)?;
                    results.rpcs.push(rpc::add_provider(ogid, &provider));
                }
                ServerEvent::Result(name, is_ok) => {
                    let provider = Provider::get_by_addr(&db, &addr)?;
                    let mut user = Name::get_by_name_provider(&db, &name, &provider.id)?;

                    if is_ok {
                        Name::active(&db, &user.id, true)?;
                        user.is_ok = true;
                        user.is_actived = true;
                        results.rpcs.push(rpc::register_success(ogid, &user));
                    } else {
                        user.delete(&db)?;
                        results.rpcs.push(rpc::register_failure(ogid, &name));
                    }
                }
                ServerEvent::Info(uname, ugid, uaddr, ubio, uavatar) => {
                    results.rpcs.push(rpc::search_result(
                        ogid, &uname, &ugid, &uaddr, &ubio, &uavatar,
                    ));
                }
                ServerEvent::None(uname) => {
                    results.rpcs.push(rpc::search_none(ogid, &uname));
                }
                ServerEvent::Actived(uname, is_actived) => {
                    let provider = Provider::get_by_addr(&db, &addr)?;
                    let name = Name::get_by_name_provider(&db, &uname, &provider.id)?;
                    Name::active(&db, &name.id, is_actived)?;

                    let ps = Provider::list(&db)?;
                    let names = Name::list(&db)?;
                    results.rpcs.push(rpc::domain_list(ogid, &ps, &names));
                }
                ServerEvent::Deleted(uname) => {
                    let provider = Provider::get_by_addr(&db, &addr)?;
                    let name = Name::get_by_name_provider(&db, &uname, &provider.id)?;
                    name.delete(&db)?;

                    let ps = Provider::list(&db)?;
                    let names = Name::list(&db)?;
                    results.rpcs.push(rpc::domain_list(ogid, &ps, &names));
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
