use std::path::PathBuf;
use tdn_storage::local::DStorage;

pub mod consensus;

mod account;
mod chat;
mod file;
mod group_chat;
mod service;
mod session;

use account::ACCOUNT_VERSIONS;
use chat::CHAT_VERSIONS;
use consensus::CONSENSUS_VERSIONS;
use file::FILE_VERSIONS;
use group_chat::GROUP_CHAT_VERSIONS;
use service::SERVICE_VERSIONS;
use session::SESSION_VERSIONS;

use crate::apps::assistant::ASSISTANT_VERSIONS;

// Account's main database name.
pub(crate) const ACCOUNT_DB: &'static str = "account.db";

/// Account's consensus database name
pub(crate) const CONSENSUS_DB: &'static str = "consensus.db";

/// Account's session database name
pub(crate) const SESSION_DB: &'static str = "session.db";

/// Account's chat database name
pub(crate) const CHAT_DB: &'static str = "chat.db";

/// Account's consensus database name
pub(crate) const FILE_DB: &'static str = "file.db";

/// Account's service database name
pub(crate) const SERVICE_DB: &'static str = "service.db";

/// Account's assistant database name
pub(crate) const ASSISTANT_DB: &'static str = "assistant.db";

/// Account's assistant database name
pub(crate) const GROUP_CHAT_DB: &'static str = "group_chat.db";

pub(crate) fn main_migrate(path: &PathBuf) -> std::io::Result<()> {
    let mut db_path = path.clone();
    db_path.push(ACCOUNT_DB);

    if db_path.exists() {
        let db = DStorage::open(db_path)?;

        // 1. get current version.
        let first_matrix =
            db.query("SELECT name FROM sqlite_master WHERE type='table' AND name='migrates'")?;
        if first_matrix.len() == 0 {
            // 2. migrate.
            for i in &ACCOUNT_VERSIONS[1..] {
                db.execute(i)?;
            }

            db.update(&format!(
                "UPDATE migrates SET version = {} where db_name = '{}'",
                ACCOUNT_VERSIONS.len(),
                ACCOUNT_DB,
            ))?;
        }

        let mut account_matrix = db.query(&format!(
            "select version from migrates where db_name = '{}'",
            ACCOUNT_DB
        ))?;
        let account_version = account_matrix.pop().unwrap().pop().unwrap().as_i64() as usize;
        if account_version != ACCOUNT_VERSIONS.len() {
            // 2. migrate.
            for i in &ACCOUNT_VERSIONS[account_version..] {
                db.execute(i)?;
            }
            db.update(&format!(
                "UPDATE migrates SET version = {} where db_name = '{}'",
                ACCOUNT_VERSIONS.len(),
                ACCOUNT_DB,
            ))?;
        }

        let matrix = db.query("select db_name, version from migrates")?;
        for mut values in matrix {
            let db_version = values.pop().unwrap().as_i64() as usize;
            let db_name = values.pop().unwrap().as_string();

            let current_versions = match db_name.as_str() {
                ACCOUNT_DB => {
                    if db_version != ACCOUNT_VERSIONS.len() {
                        // 2. migrate.
                        for i in &ACCOUNT_VERSIONS[db_version..] {
                            db.execute(i)?;
                        }
                        db.update(&format!(
                            "UPDATE migrates SET version = {} where db_name = '{}'",
                            ACCOUNT_VERSIONS.len(),
                            db_name,
                        ))?;
                    }
                    continue;
                }
                CONSENSUS_DB => CONSENSUS_VERSIONS.as_ref(),
                SESSION_DB => SESSION_VERSIONS.as_ref(),
                FILE_DB => FILE_VERSIONS.as_ref(),
                SERVICE_DB => SERVICE_VERSIONS.as_ref(),
                ASSISTANT_DB => ASSISTANT_VERSIONS.as_ref(),
                GROUP_CHAT_DB => GROUP_CHAT_VERSIONS.as_ref(),
                CHAT_DB => CHAT_VERSIONS.as_ref(),
                _ => {
                    continue;
                }
            };

            if db_version != current_versions.len() {
                let mut matrix = db.query("select gid from accounts")?;
                while matrix.len() > 0 {
                    let mut account_path = path.clone();
                    account_path.push(matrix.pop().unwrap().pop().unwrap().as_str());
                    account_path.push(&db_name);
                    let account_db = DStorage::open(account_path)?;
                    // migrate
                    for i in &current_versions[db_version..] {
                        account_db.execute(i)?;
                    }
                    account_db.close()?;
                }

                db.update(&format!(
                    "UPDATE migrates SET version = {} where db_name = '{}'",
                    current_versions.len(),
                    db_name,
                ))?;
            }
        }

        db.close()?;
    } else {
        let db = DStorage::open(db_path)?;
        // migrate all.
        for i in ACCOUNT_VERSIONS.iter() {
            db.execute(i)?;
        }

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            ACCOUNT_VERSIONS.len(),
            ACCOUNT_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            CONSENSUS_VERSIONS.len(),
            CONSENSUS_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            SESSION_VERSIONS.len(),
            SESSION_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            FILE_VERSIONS.len(),
            FILE_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            SERVICE_VERSIONS.len(),
            SERVICE_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            ASSISTANT_VERSIONS.len(),
            ASSISTANT_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            GROUP_CHAT_VERSIONS.len(),
            GROUP_CHAT_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            CHAT_VERSIONS.len(),
            CHAT_DB,
        ))?;

        db.close()?;
    }

    Ok(())
}

pub(crate) fn account_init_migrate(path: &PathBuf) -> std::io::Result<()> {
    let mut db_path = path.clone();
    db_path.push(CONSENSUS_DB);
    let db = DStorage::open(db_path)?;
    for i in &CONSENSUS_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(SESSION_DB);
    let db = DStorage::open(db_path)?;
    for i in &SESSION_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(FILE_DB);
    let db = DStorage::open(db_path)?;
    for i in &FILE_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(SERVICE_DB);
    let db = DStorage::open(db_path)?;
    for i in &SERVICE_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(ASSISTANT_DB);
    let db = DStorage::open(db_path)?;
    for i in &ASSISTANT_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(GROUP_CHAT_DB);
    let db = DStorage::open(db_path)?;
    for i in &GROUP_CHAT_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(CHAT_DB);
    let db = DStorage::open(db_path)?;
    for i in &CHAT_VERSIONS {
        db.execute(i)?;
    }
    db.close()
}
