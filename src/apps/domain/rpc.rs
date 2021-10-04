use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::{HandleResult, PeerAddr},
    rpc::{json, RpcError, RpcHandler, RpcParam},
};

use domain_types::PeerEvent;

use super::{
    add_layer,
    models::{Name, Provider},
};
use crate::{rpc::RpcState, storage::domain_db};

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
            let provider = PeerAddr::from_hex(params[0].as_str().ok_or(RpcError::ParseError)?)?;

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
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            Ok(HandleResult::rpc(json!(params)))
        },
    );

    handler.add_method(
        "domain-provider-remove",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            Ok(HandleResult::rpc(json!(params)))
        },
    );

    handler.add_method(
        "domain-register",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let _provider = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let _symbol = params[1].as_str().ok_or(RpcError::ParseError)?.to_string();
            let _bio = params[2].as_str().ok_or(RpcError::ParseError)?.to_string();

            let _me = state.group.read().await.clone_user(&gid)?;

            // Send to remote domain service.

            //

            Ok(HandleResult::rpc(json!(params)))
        },
    );

    handler.add_method(
        "domain-active",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            Ok(HandleResult::rpc(json!(params)))
        },
    );

    handler.add_method(
        "domain-remove",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            Ok(HandleResult::rpc(json!(params)))
        },
    );

    handler.add_method(
        "domain-search",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _name = params[0].as_str().ok_or(RpcError::ParseError)?;

            Ok(HandleResult::rpc(json!(params)))
        },
    );
}
