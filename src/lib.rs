#[macro_use]
extern crate tracing;

#[macro_use]
extern crate anyhow;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;

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

const DEFAULT_LOG_FILE: &'static str = "esse.log.txt";

#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;

    use self::jni::objects::{JClass, JString};
    use self::jni::JNIEnv;
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn Java_com_esse_1core_esse_1core_RustCore_start(
        env: JNIEnv,
        _: JClass,
        java_pattern: JString,
    ) {
        start(
            env.get_string(java_pattern)
                .expect("invalid pattern string")
                .as_ptr(),
        );
    }
}

#[no_mangle]
pub extern "C" fn start(db_path: *const c_char) {
    let c_str = unsafe { CStr::from_ptr(db_path) };
    let s_path = c_str.to_str().unwrap_or("./tdn").to_owned();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _ = rt.block_on(async {
        let db_path = PathBuf::from(&s_path);
        if !db_path.exists() {
            tokio::fs::create_dir_all(&db_path).await.unwrap();
        }

        // init log file.
        let file_appender = tracing_appender::rolling::daily(&s_path, DEFAULT_LOG_FILE);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_level(true)
            .with_max_level(tracing::Level::INFO)
            .init();

        server::start(db_path).await.unwrap();
    });
}
