use std::path::PathBuf;
use std::sync::Arc;
use tdn::types::{
    primitives::HandleResult,
    rpc::{json, RpcError, RpcHandler, RpcParam},
};

use crate::global::Global;
use crate::storage::{copy_file, file_db, write_file};

use super::models::{File, RootDirectory};

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<Global>) {
    handler.add_method("dc-echo", |params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "dc-list",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let root = RootDirectory::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let parent = params[1].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = file_db(&state.base, &pid, &db_key)?;

            let files: Vec<RpcParam> = File::list(&db, &root, &parent)?
                .iter()
                .map(|p| p.to_rpc())
                .collect();

            Ok(HandleResult::rpc(json!(files)))
        },
    );

    handler.add_method(
        "dc-file-create",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let root = RootDirectory::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let parent = params[1].as_i64().ok_or(RpcError::ParseError)?;
            let name = params[2].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = file_db(&state.base, &pid, &db_key)?;

            // genereate new file.
            let mut file = File::generate(root, parent, name);
            file.insert(&db)?;

            // create file on disk.
            let _ = write_file(&state.base, &pid, &file.storage_name(), &[]).await?;
            Ok(HandleResult::rpc(file.to_rpc()))
        },
    );

    handler.add_method(
        "dc-file-upload",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let root = RootDirectory::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let parent = params[1].as_i64().ok_or(RpcError::ParseError)?;
            let path = params[2].as_str().ok_or(RpcError::ParseError)?;

            let file_path = PathBuf::from(path);
            let name = file_path
                .file_name()
                .ok_or(RpcError::ParseError)?
                .to_str()
                .ok_or(RpcError::ParseError)?
                .to_owned();

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = file_db(&state.base, &pid, &db_key)?;

            let mut file = File::generate(root, parent, name);
            file.insert(&db)?;
            copy_file(&file_path, &state.base, &pid, &file.storage_name()).await?;

            Ok(HandleResult::rpc(file.to_rpc()))
        },
    );

    handler.add_method(
        "dc-folder-create",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let root = RootDirectory::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let parent = params[1].as_i64().ok_or(RpcError::ParseError)?;
            let name = params[2].as_str().ok_or(RpcError::ParseError)?.to_owned();

            // create new folder.
            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = file_db(&state.base, &pid, &db_key)?;

            let mut file = File::generate(root, parent, name);
            file.insert(&db)?;

            Ok(HandleResult::rpc(file.to_rpc()))
        },
    );

    handler.add_method(
        "dc-file-update",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let root = RootDirectory::from_i64(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let parent = params[2].as_i64().ok_or(RpcError::ParseError)?;
            let name = params[3].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = file_db(&state.base, &pid, &db_key)?;

            let mut file = File::get(&db, &id)?;
            file.root = root;
            file.parent = parent;
            file.name = name;
            file.update(&db)?;

            Ok(HandleResult::rpc(file.to_rpc()))
        },
    );

    handler.add_method(
        "dc-file-star",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let starred = params[1].as_bool().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = file_db(&state.base, &pid, &db_key)?;

            File::star(&db, &id, starred)?;
            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "dc-file-trash",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = file_db(&state.base, &pid, &db_key)?;

            // TODO trash a directory.

            File::trash(&db, &id)?;
            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "dc-file-delete",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let pid = state.pid().await;
            let db_key = state.own.read().await.db_key(&pid)?;
            let db = file_db(&state.base, &pid, &db_key)?;

            // TODO deleted file & directory.

            File::delete(&db, &id)?;
            Ok(HandleResult::new())
        },
    );
}
