use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    message::SendMessage,
    primitive::{HandleResult, Result},
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};
use tdn_did::{generate_btc_account, generate_eth_account, secp256k1::SecretKey};
use tdn_storage::local::DStorage;
use tokio::sync::mpsc::Sender;
use web3::{contract::Contract, signing::Key, types::Address as EthAddress, Web3};

use crate::{rpc::RpcState, storage::wallet_db, utils::crypto::encrypt};

use super::{
    models::{Address, ChainToken, Network, Token},
    ERC20_ABI, ERC721_ABI,
};

const WALLET_DEFAULT_PIN: &'static str = "walletissafe";

#[inline]
fn wallet_list(devices: Vec<Address>) -> RpcParam {
    let mut results = vec![];
    for wallet in devices {
        results.push(wallet.to_rpc());
    }
    json!(results)
}

#[inline]
fn res_balance(
    gid: GroupId,
    address: &str,
    network: &Network,
    balance: &str,
    token: Option<&Token>,
) -> RpcParam {
    if let Some(t) = token {
        rpc_response(
            0,
            "wallet-balance",
            json!([address, network.to_i64(), balance, t.to_rpc()]),
            gid,
        )
    } else {
        rpc_response(
            0,
            "wallet-balance",
            json!([address, network.to_i64(), balance]),
            gid,
        )
    }
}

async fn loop_token(
    sender: Sender<SendMessage>,
    db: DStorage,
    gid: GroupId,
    network: Network,
    address: String,
    c_token: Option<Token>,
) -> Result<()> {
    // loop get balance of all tokens.
    let node = network.node();
    let chain = network.chain();
    let tokens = Token::list(&db, &network)?;

    if let Some(token) = c_token {
        let balance = token_balance(&token.contract, &address, &node, &token.chain).await?;
        let res = res_balance(gid, &address, &network, &balance, Some(&token));
        sender.send(SendMessage::Rpc(0, res, true)).await?;
    } else {
        match chain {
            ChainToken::ETH => {
                let transport = web3::transports::Http::new(node)?;
                let web3 = Web3::new(transport);
                let balance = web3.eth().balance(address.parse()?, None).await?;
                let balance = balance.to_string();
                let _ = Address::update_balance(&db, &address, &network, &balance);
                let res = res_balance(gid, &address, &network, &balance, None);
                sender.send(SendMessage::Rpc(0, res, true)).await?;

                for token in tokens {
                    let balance =
                        token_balance(&token.contract, &address, &node, &token.chain).await?;
                    let res = res_balance(gid, &address, &network, &balance, Some(&token));
                    sender.send(SendMessage::Rpc(0, res, true)).await?;
                }
            }
            ChainToken::BTC => {
                // TODO
            }
            _ => panic!("nerver here!"),
        }
    }

    Ok(())
}

async fn token_check(
    sender: Sender<SendMessage>,
    db: DStorage,
    gid: GroupId,
    chain: ChainToken,
    network: Network,
    address: String,
    c_str: String,
) -> Result<()> {
    let account: EthAddress = address.parse()?;
    let addr: EthAddress = c_str.parse()?;

    let abi = match chain {
        ChainToken::ERC20 => ERC20_ABI,
        ChainToken::ERC721 => ERC721_ABI,
        _ => return Err(anyhow!("not supported")),
    };
    let node = network.node();
    let transport = web3::transports::Http::new(node)?;
    let web3 = Web3::new(transport);
    let contract = Contract::from_json(web3.eth(), addr, abi.as_bytes())?;

    let symbol: String = contract
        .query("symbol", (), None, Default::default(), None)
        .await?;

    let decimal: u64 = match chain {
        ChainToken::ERC20 => {
            contract
                .query("decimals", (), None, Default::default(), None)
                .await?
        }
        _ => 0,
    };

    let mut token = Token::new(chain, network, symbol, c_str, decimal as i64);
    token.insert(&db)?;

    let balance: web3::types::U256 = contract
        .query("balanceOf", (account,), None, Default::default(), None)
        .await?;
    let balance = balance.to_string();
    let res = res_balance(gid, &address, &network, &balance, Some(&token));
    sender.send(SendMessage::Rpc(0, res, true)).await?;

    Ok(())
}

async fn token_balance(
    c_str: &str,
    address: &str,
    node: &str,
    chain: &ChainToken,
) -> Result<String> {
    let addr: EthAddress = c_str.parse()?;
    let account: EthAddress = address.parse()?;
    let abi = match chain {
        ChainToken::ERC20 => ERC20_ABI,
        ChainToken::ERC721 => ERC721_ABI,
        _ => return Err(anyhow!("not supported")),
    };

    let transport = web3::transports::Http::new(node)?;
    let web3 = Web3::new(transport);
    let contract = Contract::from_json(web3.eth(), addr, abi.as_bytes())?;

    let balance: web3::types::U256 = contract
        .query("balanceOf", (account,), None, Default::default(), None)
        .await?;
    Ok(balance.to_string())
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
        "wallet-import",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let chain = ChainToken::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let secret = params[1].as_str().ok_or(RpcError::ParseError)?;

            let sk: SecretKey = secret.parse().or(Err(RpcError::ParseError))?;
            let addr = format!("{:?}", (&sk).address());

            let group_lock = state.group.read().await;
            let cbytes = encrypt(&group_lock.secret(), WALLET_DEFAULT_PIN, sk.as_ref())?;
            let db = wallet_db(group_lock.base(), &gid)?;
            drop(group_lock);

            let mut address = Address::import(chain, addr, cbytes);
            address.insert(&db)?;
            Ok(HandleResult::rpc(address.to_rpc()))
        },
    );

    handler.add_method(
        "wallet-balance",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let network = Network::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let address = params[1].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let group_lock = state.group.read().await;
            let db = wallet_db(group_lock.base(), &gid)?;
            let sender = group_lock.sender();
            drop(group_lock);

            let c_str = if params.len() == 4 {
                let cid = params[2].as_i64().ok_or(RpcError::ParseError)?;
                let token = Token::get(&db, &cid)?;
                Some(token)
            } else {
                None
            };

            tokio::spawn(loop_token(sender, db, gid, network, address, c_str));

            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "wallet-token-import",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let chain = ChainToken::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let network = Network::from_i64(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let address = params[2].as_str().ok_or(RpcError::ParseError)?.to_owned();
            let c_str = params[3].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let group_lock = state.group.read().await;
            let db = wallet_db(group_lock.base(), &gid)?;
            let sender = group_lock.sender();
            drop(group_lock);

            tokio::spawn(token_check(sender, db, gid, chain, network, address, c_str));

            Ok(HandleResult::new())
        },
    );
}
