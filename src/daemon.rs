#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

use std::env::args;

mod account;
mod apps;
//mod consensus;
//mod event;
mod global;
mod group;
mod layer;
mod migrate;
mod primitives;
mod rpc;
mod server;
mod session;
mod storage;
mod utils;

#[tokio::main]
async fn main() {
    console_subscriber::init();

    let db_path = args().nth(1).unwrap_or("./.tdn".to_owned());

    if std::fs::metadata(&db_path).is_err() {
        std::fs::create_dir(&db_path).unwrap();
    }

    let _ = server::start(db_path).await;
}
