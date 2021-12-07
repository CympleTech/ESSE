mod models;
mod rpc;

pub const ETH_NODE: &'static str = "https://mainnet.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161";

pub(crate) use rpc::new_rpc_handler;
