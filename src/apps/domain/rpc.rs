use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::{HandleResult, PeerId},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};

use domain_types::PeerEvent;

use super::{
    add_layer,
    models::{Name, Provider},
};
use crate::{rpc::RpcState, storage::domain_db};

#[inline]
pub(crate) fn add_provider(mgid: GroupId, provider: &Provider) -> RpcParam {
    rpc_response(0, "domain-provider-add", json!(provider.to_rpc()), mgid)
}

#[inline]
pub(crate) fn register_success(mgid: GroupId, name: &Name) -> RpcParam {
    rpc_response(0, "domain-register-success", json!(name.to_rpc()), mgid)
}

#[inline]
pub(crate) fn register_failure(mgid: GroupId, name: &str) -> RpcParam {
    rpc_response(0, "domain-register-failure", json!([name]), mgid)
}

#[inline]
pub(crate) fn domain_list(mgid: GroupId, providers: &[Provider], names: &[Name]) -> RpcParam {
    let providers: Vec<RpcParam> = providers.iter().map(|p| p.to_rpc()).collect();
    let names: Vec<RpcParam> = names.iter().map(|p| p.to_rpc()).collect();
    rpc_response(0, "domain-list", json!([providers, names]), mgid)
}

#[inline]
pub(crate) fn search_result(
    mgid: GroupId,
    name: &str,
    gid: &GroupId,
    addr: &PeerId,
    bio: &str,
    avatar: &Vec<u8>,
) -> RpcParam {
    rpc_response(
        0,
        "domain-search",
        json!([
            name,
            gid.to_hex(),
            addr.to_hex(),
            bio,
            if avatar.len() > 0 {
                base64::encode(avatar)
            } else {
                "".to_owned()
            }
        ]),
        mgid,
    )
}

#[inline]
pub(crate) fn search_none(mgid: GroupId, name: &str) -> RpcParam {
    rpc_response(0, "domain-search", json!([name]), mgid)
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method(
        "domain-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let db = domain_db(state.layer.read().await.base(), &gid)?;

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
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let provider = PeerId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;

            let mut results = HandleResult::new();
            let db = domain_db(state.layer.read().await.base(), &gid)?;
            let mut p = Provider::prepare(provider);
            p.insert(&db)?;

            add_layer(&mut results, provider, PeerEvent::Check, gid)?;
            Ok(results)
        },
    );

    handler.add_method(
        "domain-provider-default",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let db = domain_db(state.layer.read().await.base(), &gid)?;
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
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let db = domain_db(state.layer.read().await.base(), &gid)?;
            let names = Name::get_by_provider(&db, &id)?;
            if names.len() == 0 {
                Provider::delete(&db, &id)?;
            }

            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "domain-register",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let provider = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let addr = PeerId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let name = params[2].as_str().ok_or(RpcError::ParseError)?.to_string();
            let bio = params[3].as_str().ok_or(RpcError::ParseError)?.to_string();

            let me = state.group.read().await.clone_user(&gid)?;

            // save to db.
            let mut results = HandleResult::new();
            let db = domain_db(state.layer.read().await.base(), &gid)?;
            let mut u = Name::prepare(name, bio, provider);
            u.insert(&db)?;

            // send to server.
            let event = PeerEvent::Register(u.name, u.bio, me.avatar);
            add_layer(&mut results, addr, event, gid)?;
            Ok(results)
        },
    );

    handler.add_method(
        "domain-active",
        |gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?.to_owned();
            let provider = PeerId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;
            let active = params[2].as_bool().ok_or(RpcError::ParseError)?;

            let mut results = HandleResult::new();
            let event = if active {
                PeerEvent::Active(name)
            } else {
                PeerEvent::Suspend(name)
            };
            add_layer(&mut results, provider, event, gid)?;

            Ok(results)
        },
    );

    handler.add_method(
        "domain-remove",
        |gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let name = params[0].as_str().ok_or(RpcError::ParseError)?.to_owned();
            let provider = PeerId::from_hex(params[1].as_str().ok_or(RpcError::ParseError)?)?;

            let mut results = HandleResult::new();
            let event = PeerEvent::Delete(name);
            add_layer(&mut results, provider, event, gid)?;

            Ok(results)
        },
    );

    handler.add_method(
        "domain-search",
        |gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let addr = PeerId::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;
            let name = params[1].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let mut results = HandleResult::new();

            // send to server.
            let event = PeerEvent::Search(name);
            add_layer(&mut results, addr, event, gid)?;
            Ok(results)
        },
    );
}
