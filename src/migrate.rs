use std::path::PathBuf;
use tdn::types::primitive::Result;
use tdn_storage::local::DStorage;

pub mod consensus;

mod account;
mod chat;
mod cloud;
mod domain;
mod file;
mod group;
mod jarvis;
mod organization;
mod service;
mod session;
mod wallet;

use account::ACCOUNT_VERSIONS;
use chat::CHAT_VERSIONS;
use cloud::CLOUD_VERSIONS;
use consensus::CONSENSUS_VERSIONS;
use domain::DOMAIN_VERSIONS;
use file::FILE_VERSIONS;
use group::GROUP_VERSIONS;
use jarvis::JARVIS_VERSIONS;
use organization::ORGANIZATION_VERSIONS;
use service::SERVICE_VERSIONS;
use session::SESSION_VERSIONS;
use wallet::WALLET_VERSIONS;

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

/// Account's jarvis database name
pub(crate) const JARVIS_DB: &'static str = "jarvis.db";

/// Account's group chat database name
pub(crate) const GROUP_DB: &'static str = "group.db";

/// Account's organization database name
pub(crate) const ORGANIZATION_DB: &'static str = "organization.db";

/// Account's domain database name
pub(crate) const DOMAIN_DB: &'static str = "domain.db";

/// Account's wallet database name
pub(crate) const WALLET_DB: &'static str = "wallet.db";

/// Account's cloud database name
pub(crate) const CLOUD_DB: &'static str = "cloud.db";

pub(crate) fn main_migrate(path: &PathBuf) -> Result<()> {
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
                JARVIS_DB => JARVIS_VERSIONS.as_ref(),
                GROUP_DB => GROUP_VERSIONS.as_ref(),
                ORGANIZATION_DB => ORGANIZATION_VERSIONS.as_ref(),
                CHAT_DB => CHAT_VERSIONS.as_ref(),
                DOMAIN_DB => DOMAIN_VERSIONS.as_ref(),
                WALLET_DB => WALLET_VERSIONS.as_ref(),
                CLOUD_DB => CLOUD_VERSIONS.as_ref(),
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
            JARVIS_VERSIONS.len(),
            JARVIS_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            GROUP_VERSIONS.len(),
            GROUP_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            ORGANIZATION_VERSIONS.len(),
            ORGANIZATION_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            CHAT_VERSIONS.len(),
            CHAT_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            DOMAIN_VERSIONS.len(),
            DOMAIN_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            WALLET_VERSIONS.len(),
            WALLET_DB,
        ))?;

        db.update(&format!(
            "UPDATE migrates SET version = {} where db_name = '{}'",
            CLOUD_VERSIONS.len(),
            CLOUD_DB,
        ))?;

        db.close()?;
    }

    Ok(())
}

pub(crate) fn account_init_migrate(path: &PathBuf) -> Result<()> {
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
    db_path.push(JARVIS_DB);
    let db = DStorage::open(db_path)?;
    for i in &JARVIS_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(GROUP_DB);
    let db = DStorage::open(db_path)?;
    for i in &GROUP_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(ORGANIZATION_DB);
    let db = DStorage::open(db_path)?;
    for i in &ORGANIZATION_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(CHAT_DB);
    let db = DStorage::open(db_path)?;
    for i in &CHAT_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(DOMAIN_DB);
    let db = DStorage::open(db_path)?;
    for i in &DOMAIN_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(WALLET_DB);
    let db = DStorage::open(db_path)?;
    for i in &WALLET_VERSIONS {
        db.execute(i)?;
    }
    db.close()?;

    let mut db_path = path.clone();
    db_path.push(CLOUD_DB);
    let db = DStorage::open(db_path)?;
    for i in &CLOUD_VERSIONS {
        db.execute(i)?;
    }
    db.close()
}
