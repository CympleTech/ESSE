#[macro_use]
extern crate log;

use std::env::args;
use tdn::smol::{self, io::Result};

mod event;
mod group;
mod layer;
mod migrate;
mod models;
mod primitives;
mod rpc;
mod server;
mod storage;
mod utils;

fn main() -> Result<()> {
    let db_path = args().nth(1).unwrap_or("./.tdn".to_owned());

    if std::fs::metadata(&db_path).is_err() {
        std::fs::create_dir(&db_path).unwrap();
    }

    smol::block_on(server::start(db_path))
}
