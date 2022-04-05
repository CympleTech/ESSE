use domain_types::{LayerPeerEvent, DOMAIN_ID};
use esse_primitives::id_to_str;
use std::sync::Arc;
use tdn::types::{
    message::SendType,
    primitives::{HandleResult, PeerId},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};

use crate::global::Global;
use crate::storage::domain_db;

use super::models::{Name, Provider};

#[inline]
pub(crate) fn add_provider(provider: &Provider) -> RpcParam {
    rpc_response(0, "domain-provider-add", json!(provider.to_rpc()))
}

#[inline]
pub(crate) fn register_success(name: &Name) -> RpcParam {
    rpc_response(0, "domain-register-success", json!(name.to_rpc()))
}

#[inline]
pub(crate) fn register_failure(name: &str) -> RpcParam {
    rpc_response(0, "domain-register-failure", json!([name]))
}

#[inline]
pub(crate) fn domain_list(providers: &[Provider], names: &[Name]) -> RpcParam {
    let providers: Vec<RpcParam> = providers.iter().map(|p| p.to_rpc()).collect();
    let names: Vec<RpcParam> = names.iter().map(|p| p.to_rpc()).collect();
    rpc_response(0, "domain-list", json!([providers, names]))
}

#[inline]
pub(crate) fn search_result(pid: &PeerId, name: &str, bio: &str, avatar: &Vec<u8>) -> RpcParam {
    rpc_response(
        0,
        "domain-search",
        json!([
            name,
            id_to_str(pid),
            bio,
            if avatar.len() > 0 {
                base64::encode(avatar)
            } else {
                "".to_owned()
            }
        ]),
    )
}

#[inline]
pub(crate) fn search_none(name: &str) -> RpcParam {
    rpc_response(0, "domain-search", json!([name]))
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<Global>) {
    handler.add_method(
        "domain-list",
        |_params: Vec<RpcParam>, state: Arc<Global>| async move {
            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = domain_db(&state.base, &pid, &db_key)?;

            // list providers.
            let providers: Vec<RpcParam> =
                Provider::list(&db)?.iter().map(|p| p.to_rpc()).collect();

            // list names.
            let names: Vec<RpcParam> = Name::list(&db)?.iter().map(|p| p.to_rpc()).collect();

            Ok(HandleResult::rpc(json!([providers, names])))
        },
    );

    handler.add_method(
        "domain-provider-add",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let provider = PeerId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;

            let mut results = HandleResult::new();
            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = domain_db(&state.base, &pid, &db_key)?;

            let mut p = Provider::prepare(provider);
            p.insert(&db)?;

            let data = bincode::serialize(&LayerPeerEvent::Check)?;
            let msg = SendType::Event(0, provider, data);
            results.layers.push((DOMAIN_ID, msg));
            Ok(results)
        },
    );

    handler.add_method(
        "domain-provider-default",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = domain_db(&state.base, &pid, &db_key)?;

            let provider = Provider::get(&db, &id)?;
            if let Ok(default) = Provider::get_default(&db) {
                if default.id == provider.id {
                    return Ok(HandleResult::new());
                }
                default.default(&db, false)?;
            }
            provider.default(&db, true)?;

            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "domain-provider-remove",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = domain_db(&state.base, &pid, &db_key)?;

            let names = Name::get_by_provider(&db, &id)?;
            if names.len() == 0 {
                Provider::delete(&db, &id)?;
            }

            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "domain-register",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let provider = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let addr = PeerId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let name = params[2].as_str().ok_or(RpcError::ParseError)?.to_string();
            let bio = params[3].as_str().ok_or(RpcError::ParseError)?.to_string();

            // save to db.
            let mut results = HandleResult::new();
            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = domain_db(&state.base, &pid, &db_key)?;

            let me = state.own.read().await.clone_user(&pid)?;

            let mut u = Name::prepare(name, bio, provider);
            u.insert(&db)?;

            // send to server.
            let data = bincode::serialize(&LayerPeerEvent::Register(u.name, u.bio, me.avatar))?;
            let msg = SendType::Event(0, addr, data);
            results.layers.push((DOMAIN_ID, msg));
            Ok(results)
        },
    );

    handler.add_method(
        "domain-active",
        |params: Vec<RpcParam>, _state: Arc<Global>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?.to_owned();
            let provider = PeerId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let active = params[2].as_bool().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let event = if active {
                LayerPeerEvent::Active(name)
            } else {
                LayerPeerEvent::Suspend(name)
            };

            let data = bincode::serialize(&event)?;
            let msg = SendType::Event(0, provider, data);
            results.layers.push((DOMAIN_ID, msg));
            Ok(results)
        },
    );

    handler.add_method(
        "domain-remove",
        |params: Vec<RpcParam>, _state: Arc<Global>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?.to_owned();
            let provider = PeerId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;

            let mut results = HandleResult::new();
            let event = LayerPeerEvent::Delete(name);
            let data = bincode::serialize(&event)?;
            let msg = SendType::Event(0, provider, data);
            results.layers.push((DOMAIN_ID, msg));
            Ok(results)
        },
    );

    handler.add_method(
        "domain-search",
        |params: Vec<RpcParam>, _state: Arc<Global>| async move {
            let addr = PeerId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let name = params[1].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let mut results = HandleResult::new();

            // send to server.
            let event = LayerPeerEvent::Search(name);
            let data = bincode::serialize(&event)?;
            let msg = SendType::Event(0, addr, data);
            results.layers.push((DOMAIN_ID, msg));
            Ok(results)
        },
    );
}
