#[macro_use]
extern crate log;

#[macro_use]
extern crate anyhow;

use std::ffi::CStr;
use std::os::raw::c_char;

mod account;
mod apps;
//mod consensus;
//mod event;
mod group;
mod layer;
mod migrate;
mod primitives;
mod rpc;
mod server;
//mod session;
mod global;
mod storage;
mod utils;

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
    let _ = rt.block_on(server::start(s_path));
}
