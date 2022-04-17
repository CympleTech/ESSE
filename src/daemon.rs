#[macro_use]
extern crate tracing;

#[macro_use]
extern crate anyhow;

use std::env::args;
use std::path::PathBuf;
use tracing_subscriber::{filter::LevelFilter, prelude::*};

mod account;
mod apps;
//mod consensus;
//mod event;
mod global;
mod group;
mod layer;
mod migrate;
mod own;
mod primitives;
mod rpc;
mod server;
mod session;
mod storage;
mod utils;

#[tokio::main]
async fn main() {
    let console_layer = console_subscriber::spawn();
    tracing_subscriber::registry()
        .with(console_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_level(true)
                .with_filter(LevelFilter::DEBUG),
        )
        .init();

    let db_path = PathBuf::from(&args().nth(1).unwrap_or("./.tdn".to_owned()));
    if !db_path.exists() {
        tokio::fs::create_dir_all(&db_path).await.unwrap();
    }

    server::start(db_path).await.unwrap();
}
