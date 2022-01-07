use std::path::PathBuf;
use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::HandleResult,
    rpc::{json, RpcError, RpcHandler, RpcParam},
};

use crate::rpc::RpcState;
use crate::storage::{copy_file, write_file};

use super::models::{File, RootDirectory};

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("dc-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "dc-list",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let root = RootDirectory::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let parent = params[1].as_i64().ok_or(RpcError::ParseError)?;

            let db = state.group.read().await.file_db(&gid)?;
            let files: Vec<RpcParam> = File::list(&db, &root, &parent)?
                .iter()
                .map(|p| p.to_rpc())
                .collect();

            Ok(HandleResult::rpc(json!(files)))
        },
    );

    handler.add_method(
        "dc-file-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let root = RootDirectory::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let parent = params[1].as_i64().ok_or(RpcError::ParseError)?;
            let name = params[2].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let group_lock = state.group.read().await;
            let base = group_lock.base().clone();
            let db = group_lock.file_db(&gid)?;
            drop(group_lock);
            // genereate new file.
            let mut file = File::generate(root, parent, name);
            file.insert(&db)?;

            // create file on disk.
            let _ = write_file(&base, &gid, &file.storage_name(), &[]).await?;
            Ok(HandleResult::rpc(file.to_rpc()))
        },
    );

    handler.add_method(
        "dc-file-upload",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
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

            let group_lock = state.group.read().await;
            let base = group_lock.base().clone();
            let db = group_lock.file_db(&gid)?;
            drop(group_lock);
            let mut file = File::generate(root, parent, name);
            file.insert(&db)?;
            copy_file(&file_path, &base, &gid, &file.storage_name()).await?;

            Ok(HandleResult::rpc(file.to_rpc()))
        },
    );

    handler.add_method(
        "dc-folder-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let root = RootDirectory::from_i64(params[0].as_i64().ok_or(RpcError::ParseError)?);
            let parent = params[1].as_i64().ok_or(RpcError::ParseError)?;
            let name = params[2].as_str().ok_or(RpcError::ParseError)?.to_owned();

            // create new folder.
            let db = state.group.read().await.file_db(&gid)?;
            let mut file = File::generate(root, parent, name);
            file.insert(&db)?;

            Ok(HandleResult::rpc(file.to_rpc()))
        },
    );

    handler.add_method(
        "dc-file-update",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let root = RootDirectory::from_i64(params[1].as_i64().ok_or(RpcError::ParseError)?);
            let parent = params[2].as_i64().ok_or(RpcError::ParseError)?;
            let name = params[3].as_str().ok_or(RpcError::ParseError)?.to_owned();

            let db = state.group.read().await.file_db(&gid)?;
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
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            let starred = params[1].as_bool().ok_or(RpcError::ParseError)?;

            let db = state.group.read().await.file_db(&gid)?;
            File::star(&db, &id, starred)?;
            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "dc-file-trash",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            // TODO trash a directory.

            let db = state.group.read().await.file_db(&gid)?;
            File::trash(&db, &id)?;
            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "dc-file-delete",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            // TODO deleted file & directory.

            let db = state.group.read().await.file_db(&gid)?;
            File::delete(&db, &id)?;
            Ok(HandleResult::new())
        },
    );
}
