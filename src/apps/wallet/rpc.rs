use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::HandleResult,
    rpc::{json, RpcError, RpcHandler, RpcParam},
};
use tdn_did::{generate_btc_account, generate_eth_account};
use web3::signing::Key;

use crate::{rpc::RpcState, storage::wallet_db};

use super::{
    models::{Address, ChainToken},
    ETH_NODE,
};

#[inline]
fn wallet_list(devices: Vec<Address>) -> RpcParam {
    let mut results = vec![];
    for wallet in devices {
        results.push(wallet.to_rpc());
    }
    json!(results)
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("wallet-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "wallet-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let db = wallet_db(state.layer.read().await.base(), &gid)?;
            let addresses = Address::list(&db)?;
            Ok(HandleResult::rpc(wallet_list(addresses)))
        },
    );

    handler.add_method(
        "wallet-generate",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let chain = ChainToken::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let lock = params[1].as_str().ok_or(RpcError::ParseError)?;

            let group_lock = state.group.read().await;
            let mnemonic = group_lock.mnemonic(&gid, lock)?;
            let account = group_lock.account(&gid)?;
            let lang = account.lang();
            let pass = account.pass.to_string();
            let account_index = account.index as u32;
            let db = wallet_db(group_lock.base(), &gid)?;
            drop(group_lock);

            let pass = if pass.len() > 0 {
                Some(pass.as_ref())
            } else {
                None
            };

            let index = Address::next_index(&db, &chain)?;
            let mut address = match chain {
                ChainToken::ETH | ChainToken::ERC20 | ChainToken::ERC721 => {
                    let sk = generate_eth_account(lang, &mnemonic, account_index, index, pass)?;
                    let address = format!("{:?}", (&sk).address());
                    Address::new(chain, index as i64, address)
                }
                ChainToken::BTC => {
                    let _sk = generate_btc_account(lang, &mnemonic, account_index, index, pass)?;
                    todo!();
                }
            };

            address.insert(&db)?;
            Ok(HandleResult::rpc(address.to_rpc()))
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
