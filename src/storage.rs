use image::{load_from_memory, DynamicImage, GenericImageView};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

use tdn::types::{group::GroupId, primitive::Result};
use tdn_storage::local::DStorage;

use crate::migrate::{
    account_init_migrate, ACCOUNT_DB, CHAT_DB, CLOUD_DB, CONSENSUS_DB, DAO_DB, DOMAIN_DB, FILE_DB,
    GROUP_DB, JARVIS_DB, SERVICE_DB, SESSION_DB, WALLET_DB,
};

const FILES_DIR: &'static str = "files";
const IMAGE_DIR: &'static str = "images";
const THUMB_DIR: &'static str = "thumbs";
const EMOJI_DIR: &'static str = "emojis";
const RECORD_DIR: &'static str = "records";
const AVATAR_DIR: &'static str = "avatars";

pub(crate) async fn init_local_files(base: &PathBuf) -> Result<()> {
    let mut files_path = base.clone();
    files_path.push(FILES_DIR);
    if !files_path.exists() {
        fs::create_dir_all(files_path).await?;
    }
    let mut image_path = base.clone();
    image_path.push(IMAGE_DIR);
    if !image_path.exists() {
        fs::create_dir_all(image_path).await?;
    }
    let mut thumb_path = base.clone();
    thumb_path.push(THUMB_DIR);
    if !thumb_path.exists() {
        fs::create_dir_all(thumb_path).await?;
    }
    let mut emoji_path = base.clone();
    emoji_path.push(EMOJI_DIR);
    if !emoji_path.exists() {
        fs::create_dir_all(emoji_path).await?;
    }
    let mut record_path = base.clone();
    record_path.push(RECORD_DIR);
    if !record_path.exists() {
        fs::create_dir_all(record_path).await?;
    }
    let mut avatar_path = base.clone();
    avatar_path.push(AVATAR_DIR);
    if !avatar_path.exists() {
        fs::create_dir_all(avatar_path).await?;
    }
    Ok(())
}

pub(crate) async fn read_file(base: &PathBuf) -> Result<Vec<u8>> {
    Ok(fs::read(base).await?)
}

pub(crate) async fn copy_file(
    target: &PathBuf,
    base: &PathBuf,
    gid: &GroupId,
    name: &str,
) -> Result<()> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(FILES_DIR);
    path.push(name);
    fs::copy(target, path).await?;
    Ok(())
}

pub(crate) async fn write_file(
    base: &PathBuf,
    gid: &GroupId,
    name: &str,
    bytes: &[u8],
) -> Result<String> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(FILES_DIR);
    path.push(name);
    fs::write(path, bytes).await?;
    Ok(name.to_owned())
}

pub(crate) fn write_file_sync(
    base: &PathBuf,
    gid: &GroupId,
    name: &str,
    bytes: Vec<u8>,
) -> Result<String> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(FILES_DIR);
    path.push(name);
    tokio::spawn(async move { fs::write(path, bytes).await });

    Ok(name.to_owned())
}

pub(crate) async fn read_db_file(base: &PathBuf, gid: &GroupId, name: &str) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(FILES_DIR);
    path.push(name);
    if path.exists() {
        Ok(fs::read(path).await?)
    } else {
        Ok(vec![])
    }
}

pub(crate) async fn read_image(base: &PathBuf, gid: &GroupId, name: &str) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(IMAGE_DIR);
    path.push(name);
    if path.exists() {
        Ok(fs::read(path).await?)
    } else {
        Ok(vec![])
    }
}

#[inline]
fn image_name() -> String {
    let mut name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
    name.push_str(".png");
    name
}

#[inline]
fn image_thumb(bytes: &[u8]) -> Result<DynamicImage> {
    // thumbnail image. 120*800
    let img = load_from_memory(&bytes)?;
    let (x, _) = img.dimensions();
    if x > 100 {
        Ok(img.thumbnail(120, 800))
    } else {
        Ok(img)
    }
}

pub(crate) fn write_image_sync(base: &PathBuf, gid: &GroupId, bytes: Vec<u8>) -> Result<String> {
    let mut path = base.clone();
    path.push(gid.to_hex());

    let thumb = image_thumb(&bytes)?;
    let name = image_name();

    let mut thumb_path = path.clone();
    thumb_path.push(THUMB_DIR);
    thumb_path.push(name.clone());
    tokio::spawn(async move {
        let _ = thumb.save(thumb_path);
    });

    path.push(IMAGE_DIR);
    path.push(name.clone());
    tokio::spawn(async move { fs::write(path, bytes).await });

    Ok(name)
}

pub(crate) async fn write_image(base: &PathBuf, gid: &GroupId, bytes: &[u8]) -> Result<String> {
    let mut path = base.clone();
    path.push(gid.to_hex());

    let thumb = image_thumb(bytes)?;
    let name = image_name();

    let mut thumb_path = path.clone();
    thumb_path.push(THUMB_DIR);
    thumb_path.push(name.clone());
    tokio::spawn(async move {
        let _ = thumb.save(thumb_path);
    });

    path.push(IMAGE_DIR);
    path.push(name.clone());
    fs::write(path, bytes).await?;

    Ok(name)
}

#[inline]
fn avatar_png(gid: &GroupId) -> String {
    let mut gs = gid.to_hex();
    gs.push_str(".png");
    gs
}

pub(crate) async fn read_avatar(
    base: &PathBuf,
    gid: &GroupId,
    remote: &GroupId,
) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    if path.exists() {
        Ok(fs::read(path).await?)
    } else {
        Ok(vec![])
    }
}

pub(crate) fn read_avatar_sync(base: &PathBuf, gid: &GroupId, remote: &GroupId) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    if path.exists() {
        Ok(std::fs::read(path)?)
    } else {
        Ok(vec![])
    }
}

pub(crate) async fn write_avatar(
    base: &PathBuf,
    gid: &GroupId,
    remote: &GroupId,
    bytes: &[u8],
) -> Result<()> {
    if bytes.len() < 1 {
        return Ok(());
    }
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    Ok(fs::write(path, bytes).await?)
}

pub(crate) fn write_avatar_sync(
    base: &PathBuf,
    gid: &GroupId,
    remote: &GroupId,
    bytes: Vec<u8>,
) -> Result<()> {
    if bytes.len() < 1 {
        return Ok(());
    }
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    tokio::spawn(async move { fs::write(path, bytes).await });
    Ok(())
}

pub(crate) async fn delete_avatar(base: &PathBuf, gid: &GroupId, remote: &GroupId) -> Result<()> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    if path.exists() {
        Ok(fs::remove_file(path).await?)
    } else {
        Ok(())
    }
}

pub(crate) fn delete_avatar_sync(base: &PathBuf, gid: &GroupId, remote: &GroupId) -> Result<()> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(AVATAR_DIR);
    path.push(avatar_png(remote));
    if path.exists() {
        tokio::spawn(async move { fs::remove_file(path).await });
    }
    Ok(())
}

pub(crate) async fn read_record(base: &PathBuf, gid: &GroupId, name: &str) -> Result<Vec<u8>> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(RECORD_DIR);
    path.push(name);
    if path.exists() {
        Ok(fs::read(path).await?)
    } else {
        Ok(vec![])
    }
}

pub(crate) fn write_record_sync(
    base: &PathBuf,
    gid: &GroupId,
    t: u32,
    bytes: Vec<u8>,
) -> Result<String> {
    let start = SystemTime::now();
    let datetime = start
        .duration_since(UNIX_EPOCH)
        .map(|s| s.as_millis())
        .unwrap_or(0u128);

    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(RECORD_DIR);
    path.push(format!("{}.m4a", datetime));
    tokio::spawn(async move { fs::write(path, bytes).await });

    Ok(format!("{}_{}.m4a", t, datetime))
}

pub(crate) async fn _delete_record(base: &PathBuf, gid: &GroupId, name: &str) -> Result<()> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(RECORD_DIR);
    path.push(name);
    Ok(fs::remove_file(path).await?)
}

pub(crate) fn _write_emoji(base: &PathBuf, gid: &GroupId) -> Result<()> {
    let mut path = base.clone();
    path.push(gid.to_hex());
    path.push(EMOJI_DIR);
    Ok(())
}

#[inline]
pub(crate) fn account_db(base: &PathBuf) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(ACCOUNT_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn consensus_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(CONSENSUS_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn session_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(SESSION_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn chat_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(CHAT_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn file_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(FILE_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn _service_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(SERVICE_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn jarvis_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(JARVIS_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn group_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(GROUP_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn dao_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(DAO_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn domain_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(DOMAIN_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn wallet_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(WALLET_DB);
    DStorage::open(db_path)
}

#[inline]
pub(crate) fn cloud_db(base: &PathBuf, gid: &GroupId) -> Result<DStorage> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    db_path.push(CLOUD_DB);
    DStorage::open(db_path)
}

/// account independent db and storage directory.
pub(crate) async fn account_init(base: &PathBuf, gid: &GroupId) -> Result<()> {
    let mut db_path = base.clone();
    db_path.push(gid.to_hex());
    init_local_files(&db_path).await?;

    // Inner Database.
    account_init_migrate(&db_path)
}
