use domain_types::LayerServerEvent;
use std::sync::Arc;
use tdn::types::{
    message::RecvType,
    primitives::{HandleResult, Result},
};

use crate::global::Global;
use crate::storage::domain_db;

use super::models::{Name, Provider};
use super::rpc;

pub(crate) async fn handle(msg: RecvType, global: &Arc<Global>) -> Result<HandleResult> {
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
            let event: LayerServerEvent = bincode::deserialize(&bytes)?;

            let pid = global.pid().await;
            let db_key = global.own.read().await.db_key(&pid)?;
            let db = domain_db(&global.base, &pid, &db_key)?;

            match event {
                LayerServerEvent::Status(name, support_request) => {
                    let mut provider = Provider::get_by_addr(&db, &addr)?;
                    provider.ok(&db, name, support_request)?;
                    results.rpcs.push(rpc::add_provider(&provider));
                }
                LayerServerEvent::Result(name, is_ok) => {
                    let provider = Provider::get_by_addr(&db, &addr)?;
                    let mut user = Name::get_by_name_provider(&db, &name, &provider.id)?;

                    if is_ok {
                        Name::active(&db, &user.id, true)?;
                        user.is_ok = true;
                        user.is_actived = true;
                        results.rpcs.push(rpc::register_success(&user));
                    } else {
                        user.delete(&db)?;
                        results.rpcs.push(rpc::register_failure(&name));
                    }
                }
                LayerServerEvent::Info(upid, uname, ubio, uavatar) => {
                    results
                        .rpcs
                        .push(rpc::search_result(&upid, &uname, &ubio, &uavatar));
                }
                LayerServerEvent::None(uname) => {
                    results.rpcs.push(rpc::search_none(&uname));
                }
                LayerServerEvent::Actived(uname, is_actived) => {
                    let provider = Provider::get_by_addr(&db, &addr)?;
                    let name = Name::get_by_name_provider(&db, &uname, &provider.id)?;
                    Name::active(&db, &name.id, is_actived)?;

                    let ps = Provider::list(&db)?;
                    let names = Name::list(&db)?;
                    results.rpcs.push(rpc::domain_list(&ps, &names));
                }
                LayerServerEvent::Deleted(uname) => {
                    let provider = Provider::get_by_addr(&db, &addr)?;
                    let name = Name::get_by_name_provider(&db, &uname, &provider.id)?;
                    name.delete(&db)?;

                    let ps = Provider::list(&db)?;
                    let names = Name::list(&db)?;
                    results.rpcs.push(rpc::domain_list(&ps, &names));
                }
                LayerServerEvent::Response(_ugid, _uname, _is_ok) => {}
            }
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {
            // MAYBE
        }
    }

    Ok(results)
}
