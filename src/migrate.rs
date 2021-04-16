use std::path::PathBuf;
use tdn_storage::local::DStorage;

pub mod consensus;

mod account;
mod file;
mod service;
mod session;

use account::ACCOUNT_VERSIONS;
use consensus::CONSENSUS_VERSIONS;
use file::FILE_VERSIONS;
use service::SERVICE_VERSIONS;
use session::SESSION_VERSIONS;

// Account's main database name.
pub(crate) const ACCOUNT_DB: &'static str = "account.db";

/// Account's consensus database name
pub(crate) const CONSENSUS_DB: &'static str = "consensus.db";

/// Account's session database name
pub(crate) const SESSION_DB: &'static str = "session.db";

/// Account's consensus database name
pub(crate) const FILE_DB: &'static str = "file.db";

/// Account's service database name
pub(crate) const SERVICE_DB: &'static str = "service.db";

pub(crate) fn main_migrate(path: &PathBuf) -> std::io::Result<()> {
    let mut db_path = path.clone();
    db_path.push(ACCOUNT_DB);

    if db_path.exists() {
        let db = DStorage::open(db_path)?;
        // 1. get current version.
        let mut matrix = db.query("select account_version, consensus_version, session_version, file_version, service_version from versions")?;
        let mut values = matrix.pop().unwrap();

        let current_service = values.pop().unwrap().as_i64() as usize;
        let current_file = values.pop().unwrap().as_i64() as usize;
        let current_session = values.pop().unwrap().as_i64() as usize;
        let current_consensus = values.pop().unwrap().as_i64() as usize;
        let current_account = values.pop().unwrap().as_i64() as usize;

        if current_account != ACCOUNT_VERSIONS.len() {
            // 2. migrate.
            for i in &ACCOUNT_VERSIONS[current_account..] {
                db.execute(i)?;
            }
            db.update(&format!(
                "UPDATE versions SET account_version = {}",
                ACCOUNT_VERSIONS.len()
            ))?;
        }

        if current_consensus != CONSENSUS_VERSIONS.len() {
            let mut matrix = db.query("select gid from accounts")?;
            while matrix.len() > 0 {
                let mut account_path = path.clone();
                account_path.push(matrix.pop().unwrap().pop().unwrap().as_str());
                account_path.push(CONSENSUS_DB);
                let account_db = DStorage::open(account_path)?;
                // migrate
                for i in &CONSENSUS_VERSIONS[current_consensus..] {
                    account_db.execute(i)?;
                }
                account_db.close()?;
            }

            db.update(&format!(
                "UPDATE versions SET consensus_version = {}",
                CONSENSUS_VERSIONS.len()
            ))?;
        }

        if current_session != SESSION_VERSIONS.len() {
            let mut matrix = db.query("select gid from accounts")?;
            while matrix.len() > 0 {
                let mut account_path = path.clone();
                account_path.push(matrix.pop().unwrap().pop().unwrap().as_str());
                account_path.push(SESSION_DB);
                let account_db = DStorage::open(account_path)?;
                // migrate
                for i in &SESSION_VERSIONS[current_session..] {
                    account_db.execute(i)?;
                }
                account_db.close()?;
            }

            db.update(&format!(
                "UPDATE versions SET session_version = {}",
                SESSION_VERSIONS.len()
            ))?;
        }

        if current_file != FILE_VERSIONS.len() {
            let mut matrix = db.query("select gid from accounts")?;
            while matrix.len() > 0 {
                let mut account_path = path.clone();
                account_path.push(matrix.pop().unwrap().pop().unwrap().as_str());
                account_path.push(FILE_DB);
                let account_db = DStorage::open(account_path)?;
                // migrate.
                for i in &FILE_VERSIONS[current_file..] {
                    account_db.execute(i)?;
                }
                account_db.close()?;
            }

            db.update(&format!(
                "UPDATE versions SET file_version = {}",
                FILE_VERSIONS.len()
            ))?;
        }

        if current_service != SERVICE_VERSIONS.len() {
            let mut matrix = db.query("select gid from accounts")?;
            while matrix.len() > 0 {
                let mut account_path = path.clone();
                account_path.push(matrix.pop().unwrap().pop().unwrap().as_str());
                account_path.push(SERVICE_DB);
                let account_db = DStorage::open(account_path)?;
                // 2. migrate.
                for i in &SERVICE_VERSIONS[current_service..] {
                    account_db.execute(i)?;
                }
                account_db.close()?;
            }

            db.update(&format!(
                "UPDATE versions SET service_version = {}",
                SERVICE_VERSIONS.len()
            ))?;
        }

        db.close()?;
    } else {
        let db = DStorage::open(db_path)?;
        // migrate all.
        for i in ACCOUNT_VERSIONS.iter() {
            db.execute(i)?;
        }
        db.insert(&format!(
            "INSERT INTO versions (account_version, consensus_version, session_version, file_version, service_version) VALUES ({}, {}, {}, {}, {})",
            ACCOUNT_VERSIONS.len(),
            CONSENSUS_VERSIONS.len(),
            SESSION_VERSIONS.len(),
            FILE_VERSIONS.len(),
            SERVICE_VERSIONS.len(),
        ))?;
        db.close()?;
    }

    Ok(())
}

pub(crate) fn consensus_migrate(path: &PathBuf) -> std::io::Result<()> {
    let mut db_path = path.clone();
    db_path.push(CONSENSUS_DB);
    let db = DStorage::open(db_path)?;
    for i in &CONSENSUS_VERSIONS {
        db.execute(i)?;
    }
    db.close()
}

pub(crate) fn session_migrate(path: &PathBuf) -> std::io::Result<()> {
    let mut db_path = path.clone();
    db_path.push(SESSION_DB);
    let db = DStorage::open(db_path)?;
    for i in &SESSION_VERSIONS {
        db.execute(i)?;
    }
    db.close()
}

pub(crate) fn file_migrate(path: &PathBuf) -> std::io::Result<()> {
    let mut db_path = path.clone();
    db_path.push(FILE_DB);
    let db = DStorage::open(db_path)?;
    for i in &FILE_VERSIONS {
        db.execute(i)?;
    }
    db.close()
}

pub(crate) fn service_migrate(path: &PathBuf) -> std::io::Result<()> {
    let mut db_path = path.clone();
    db_path.push(SERVICE_DB);
    let db = DStorage::open(db_path)?;
    for i in &SERVICE_VERSIONS {
        db.execute(i)?;
    }
    db.close()
}
