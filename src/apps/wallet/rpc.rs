use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::HandleResult,
    rpc::{json, RpcError, RpcHandler, RpcParam},
};
use tdn_did::generate_eth_account;
use web3::signing::Key;

use crate::rpc::RpcState;

use super::{models::ChainToken, ETH_NODE};

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("wallet-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "wallet-generate",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let lock = params[0].as_str().ok_or(RpcError::ParseError)?;

            let group_lock = state.group.read().await;
            let mnemonic = group_lock.mnemonic(&gid, lock)?;
            let account = group_lock.account(&gid)?;
            let lang = account.lang();
            let pass = account.pass.to_string();
            let account_index = account.index as u32;
            drop(group_lock);

            let pass = if pass.len() > 0 {
                Some(pass.as_ref())
            } else {
                None
            };
            let index = 0; // TOOD
            let sk = generate_eth_account(lang, &mnemonic, account_index, index, pass)?;
            let address = (&sk).address();
            println!("{:?}", address);

            Ok(HandleResult::rpc(json!([mnemonic])))
        },
    );

    handler.add_method(
        "wallet-balance",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let ctoken = ChainToken::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let address = params[0].as_str().ok_or(RpcError::ParseError)?;

            match ctoken {
                ChainToken::ETH => {
                    let transport = web3::transports::Http::new(ETH_NODE).unwrap();
                    let web3 = web3::Web3::new(transport);

                    let balance = web3
                        .eth()
                        .balance(address.parse().unwrap(), None)
                        .await
                        .unwrap();
                    println!("Balance of {:?}: {}", address, balance);
                }
                ChainToken::ERC20 => {}
                ChainToken::ERC721 => {}
                _ => {}
            }

            Ok(HandleResult::rpc(json!(params)))
        },
    );
}
