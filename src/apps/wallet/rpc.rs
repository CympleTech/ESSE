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
use web3::{
    contract::{tokens::Tokenize, Contract},
    signing::Key,
    transports::http::Http,
    types::{Address as EthAddress, Bytes, CallRequest, TransactionParameters, U256},
    Web3,
};

use crate::{rpc::RpcState, storage::wallet_db};

use super::{
    models::{Address, Balance, ChainToken, Network, Token},
    ERC20_ABI, ERC721_ABI,
};

#[inline]
fn wallet_list(wallets: Vec<Address>) -> RpcParam {
    let mut results = vec![];
    for wallet in wallets {
        results.push(wallet.to_rpc());
    }
    json!(results)
}

#[inline]
fn token_list(network: Network, tokens: Vec<Token>) -> RpcParam {
    let mut results = vec![];
    for token in tokens {
        results.push(token.to_rpc());
    }
    json!([network.to_i64(), results])
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
        let transport = Http::new(node)?;
        let web3 = Web3::new(transport);
        let balance = token_balance(&web3, &token.contract, &address, &token.chain).await?;
        let res = res_balance(gid, &address, &network, &balance, Some(&token));
        sender.send(SendMessage::Rpc(0, res, true)).await?;
    } else {
        match chain {
            ChainToken::ETH => {
                let transport = Http::new(node)?;
                let web3 = Web3::new(transport);
                let balance = web3.eth().balance(address.parse()?, None).await?;
                let balance = balance.to_string();
                let _ = Address::update_balance(&db, &address, &network, &balance);
                let res = res_balance(gid, &address, &network, &balance, None);
                sender.send(SendMessage::Rpc(0, res, true)).await?;

                for token in tokens {
                    //tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    let balance =
                        token_balance(&web3, &token.contract, &address, &token.chain).await?;
                    let res = res_balance(gid, &address, &network, &balance, Some(&token));

                    // update & clean balances.
                    // TODO

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
    let transport = Http::new(node)?;
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
        _ => 0, // NFT default no decimal.
    };

    let mut token = Token::new(chain, network, symbol, c_str, decimal as i64);
    token.insert(&db)?;

    let balance: U256 = contract
        .query("balanceOf", (account,), None, Default::default(), None)
        .await?;
    let balance = balance.to_string();
    let res = res_balance(gid, &address, &network, &balance, Some(&token));
    sender.send(SendMessage::Rpc(0, res, true)).await?;

    Ok(())
}

async fn token_balance(
    web3: &Web3<Http>,
    c_str: &str,
    address: &str,
    chain: &ChainToken,
) -> Result<String> {
    let addr: EthAddress = c_str.parse()?;
    let account: EthAddress = address.parse()?;

    let abi = match chain {
        ChainToken::ERC20 => ERC20_ABI,
        ChainToken::ERC721 => ERC721_ABI,
        _ => return Err(anyhow!("not supported")),
    };

    let contract = Contract::from_json(web3.eth(), addr, abi.as_bytes())?;

    let balance: U256 = contract
        .query("balanceOf", (account,), None, Default::default(), None)
        .await?;
    Ok(balance.to_string())
}

async fn token_transfer(
    from_str: &str,
    to_str: &str,
    amount_str: &str,
    c_str: &str,
    secret: &SecretKey,
    network: &Network,
    chain: &ChainToken,
) -> Result<String> {
    let from: EthAddress = from_str.parse()?;
    let to: EthAddress = to_str.parse()?;
    let amount = U256::from_dec_str(amount_str)?;

    let node = network.node();
    let transport = Http::new(node)?;
    let web3 = Web3::new(transport);

    let tx = match chain {
        ChainToken::ERC20 => {
            let addr: EthAddress = c_str.parse()?;
            let contract = Contract::from_json(web3.eth(), addr, ERC20_ABI.as_bytes())?;
            let fn_data = contract
                .abi()
                .function("transfer")
                .and_then(|function| function.encode_input(&(to, amount).into_tokens()))?;
            TransactionParameters {
                to: Some(addr),
                data: Bytes(fn_data),
                ..Default::default()
            }
        }
        ChainToken::ERC721 => {
            let addr: EthAddress = c_str.parse()?;
            let contract = Contract::from_json(web3.eth(), addr, ERC721_ABI.as_bytes())?;
            let fn_data = contract
                .abi()
                .function("safeTransferFrom")
                .and_then(|function| function.encode_input(&(from, to, amount).into_tokens()))?;
            TransactionParameters {
                to: Some(addr),
                data: Bytes(fn_data),
                ..Default::default()
            }
        }
        ChainToken::ETH => TransactionParameters {
            to: Some(to),
            value: amount,
            ..Default::default()
        },
        _ => return Err(anyhow!("not supported")),
    };

    let signed = web3.accounts().sign_transaction(tx, secret).await?;
    let result = web3
        .eth()
        .send_raw_transaction(signed.raw_transaction)
        .await?;
    Ok(format!("{:?}", result))
}

async fn token_gas(
    from_str: &str,
    to_str: &str,
    amount_str: &str,
    c_str: &str,
    network: &Network,
    chain: &ChainToken,
) -> Result<(String, String)> {
    let from: EthAddress = from_str.parse()?;
    let to: EthAddress = to_str.parse()?;
    let amount = U256::from_dec_str(amount_str)?;
    let node = network.node();

    let transport = Http::new(node)?;
    let web3 = Web3::new(transport);
    let price = web3.eth().gas_price().await?;

    let gas = match chain {
        ChainToken::ERC20 => {
            let addr: EthAddress = c_str.parse()?;
            let contract = Contract::from_json(web3.eth(), addr, ERC20_ABI.as_bytes())?;
            let gas = contract
                .estimate_gas("transfer", (to, amount), from, Default::default())
                .await?;
            gas * price
        }
        ChainToken::ERC721 => {
            let addr: EthAddress = c_str.parse()?;
            let contract = Contract::from_json(web3.eth(), addr, ERC721_ABI.as_bytes())?;
            let gas = contract
                .estimate_gas(
                    "safeTransferFrom",
                    (from, to, amount),
                    from,
                    Default::default(),
                )
                .await?;
            gas * price
        }
        ChainToken::ETH => {
            let tx = CallRequest {
                to: Some(to),
                value: Some(amount),
                ..Default::default()
            };
            let gas = web3.eth().estimate_gas(tx, None).await?;
            price * gas
        }
        _ => return Err(anyhow!("not supported")),
    };
    Ok((price.to_string(), gas.to_string()))
}

async fn nft_check(node: &str, c_str: &str, hash: &str) -> Result<String> {
    let addr: EthAddress = c_str.parse()?;
    let tokenid = if hash.starts_with("0x") {
        U256::from_str_radix(&hash, 16)?
    } else {
        U256::from_dec_str(&hash)?
    };
    let transport = Http::new(node)?;
    let web3 = Web3::new(transport);
    let contract = Contract::from_json(web3.eth(), addr, ERC721_ABI.as_bytes())?;

    let owner: EthAddress = contract
        .query("ownerOf", (tokenid,), None, Default::default(), None)
        .await?;

    Ok(format!("{:?}", owner))
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
                    Address::new(chain, index as i64, address, index == 0)
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
            let lock = params[2].as_str().ok_or(RpcError::ParseError)?;

            let sk: SecretKey = secret.parse().or(Err(RpcError::ParseError))?;
            let addr = format!("{:?}", (&sk).address());

            let group_lock = state.group.read().await;
            let cbytes = group_lock.encrypt(&gid, lock, sk.as_ref())?;
            let db = wallet_db(group_lock.base(), &gid)?;
            drop(group_lock);

            let mut address = Address::import(chain, addr, cbytes);
            address.insert(&db)?;
            Ok(HandleResult::rpc(address.to_rpc()))
        },
    );

    handler.add_method(
        "wallet-token",
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

            let tokens = Token::list(&db, &network)?;
            tokio::spawn(loop_token(sender, db, gid, network, address, c_str));
            Ok(HandleResult::rpc(token_list(network, tokens)))
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

    handler.add_method(
        "wallet-gas-price",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let chain = ChainToken::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let network = Network::from_i64(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let from = params[2].as_str().ok_or(RpcError::ParseError)?;
            let to = params[3].as_str().ok_or(RpcError::ParseError)?;
            let amount = params[4].as_str().ok_or(RpcError::ParseError)?;
            let c_str = params[5].as_str().ok_or(RpcError::ParseError)?;

            let (price, gas) = token_gas(from, to, amount, c_str, &network, &chain).await?;
            Ok(HandleResult::rpc(json!([price, gas])))
        },
    );

    handler.add_method(
        "wallet-transfer",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let chain = ChainToken::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let network = Network::from_i64(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let from = params[2].as_i64().ok_or(RpcError::ParseError)?;
            let to = params[3].as_str().ok_or(RpcError::ParseError)?;
            let amount = params[4].as_str().ok_or(RpcError::ParseError)?;
            let c_str = params[5].as_str().ok_or(RpcError::ParseError)?;
            let lock = params[6].as_str().ok_or(RpcError::ParseError)?;

            let group_lock = state.group.read().await;
            if !group_lock.check_lock(&gid, &lock) {
                return Err(RpcError::Custom("Lock is invalid!".to_owned()));
            }
            let db = wallet_db(group_lock.base(), &gid)?;
            let address = Address::get(&db, &from)?;

            let (mnemonic, pbytes) = if address.is_gen() {
                (group_lock.mnemonic(&gid, lock)?, vec![])
            } else {
                let pbytes = group_lock.decrypt(&gid, lock, address.secret.as_ref())?;
                (String::new(), pbytes)
            };
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

            let sk: SecretKey = if address.is_gen() {
                match chain {
                    ChainToken::ETH | ChainToken::ERC20 | ChainToken::ERC721 => {
                        generate_eth_account(
                            lang,
                            &mnemonic,
                            account_index,
                            address.index as u32,
                            pass,
                        )?
                    }
                    ChainToken::BTC => {
                        todo!();
                    }
                }
            } else {
                let sk = SecretKey::from_slice(&pbytes)
                    .or(Err(RpcError::Custom("Secret is invalid!".to_owned())))?;
                if format!("{:?}", (&sk).address()) != address.address {
                    return Err(RpcError::Custom("Secret is invalid!".to_owned()));
                }
                sk
            };

            let hash = token_transfer(&address.address, to, amount, c_str, &sk, &network, &chain)
                .await
                .map_err(|e| RpcError::Custom(format!("{:?}", e)))?;

            // NFT: delete old, add new if needed (between accounts).
            if let Ok(token) = Token::get_by_contract(&db, &network, c_str) {
                if token.chain == ChainToken::ERC721 {
                    let _ = Balance::delete_by_hash(&db, amount);

                    if let Ok(new) = Address::get_by_address(&db, to) {
                        let _ = Balance::add(&db, new.id, token.id, amount.to_owned());
                    }
                }
            }

            Ok(HandleResult::rpc(json!([
                from,
                network.to_i64(),
                [hash, to],
            ])))
        },
    );

    handler.add_method(
        "wallet-nft",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let address = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let token = params[1].as_i64().ok_or(RpcError::ParseError)?;

            let db = wallet_db(state.layer.read().await.base(), &gid)?;
            let nfts = Balance::list(&db, &address, &token)?;

            let mut results = vec![];
            for nft in nfts {
                results.push(nft.value);
            }
            Ok(HandleResult::rpc(json!([address, token, results])))
        },
    );

    handler.add_method(
        "wallet-nft-add",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let address = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let token = params[1].as_i64().ok_or(RpcError::ParseError)?;
            let hash = params[2].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let db = wallet_db(state.layer.read().await.base(), &gid)?;
            let t = Token::get(&db, &token)?;
            let a = Address::get(&db, &address)?;

            if t.chain != ChainToken::ERC721 {
                return Err(RpcError::Custom("token is not erc721".to_owned()));
            }

            let owner = nft_check(t.network.node(), &t.contract, &hash)
                .await
                .map_err(|e| RpcError::Custom(format!("{:?}", e)))?;

            if owner == a.address {
                let balance = Balance::add(&db, address, token, hash)?;
                Ok(HandleResult::rpc(json!([address, token, balance.value])))
            } else {
                Err(RpcError::Custom("address is not NFT owner".to_owned()))
            }
        },
    );

    handler.add_method(
        "wallet-main",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let db = wallet_db(state.layer.read().await.base(), &gid)?;
            Address::main(&db, &id)?;
            Ok(HandleResult::new())
        },
    );
}
