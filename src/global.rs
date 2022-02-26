use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tdn::prelude::{GroupId, PeerId, SendMessage};
use tokio::{sync::mpsc::Sender, sync::RwLock};

use crate::account::Account;
use crate::group::Group;
use crate::layer::Layer;

/// global status.
pub(crate) struct Global {
    /// current running account.
    pub peer_id: RwLock<PeerId>,
    /// current account public height.
    pub peer_pub_height: RwLock<u64>,
    /// current account own height.
    pub peer_own_height: RwLock<u64>,
    /// current group.
    pub group: RwLock<Group>,
    /// current layer.
    pub layer: RwLock<Layer>,
    /// TDN network sender.
    pub sender: RwLock<Sender<SendMessage>>,
    /// message delivery tracking. uuid, me_gid, db_id.
    pub _delivery: RwLock<HashMap<u64, (GroupId, i64)>>,
    /// storage base path.
    pub base: PathBuf,
    /// random secret seed.
    pub secret: [u8; 32],
}

impl Global {
    pub fn init(
        accounts: HashMap<PeerId, Account>,
        tdn_send: Sender<SendMessage>,
        base: PathBuf,
        secret: [u8; 32],
    ) -> Self {
        Global {
            base,
            secret,
            peer_id: RwLock::new(PeerId::default()),
            peer_pub_height: RwLock::new(0),
            peer_own_height: RwLock::new(0),
            group: RwLock::new(Group::init(accounts)),
            layer: RwLock::new(Layer::init()),
            sender: RwLock::new(tdn_send),
            _delivery: RwLock::new(HashMap::new()),
        }
    }

    pub async fn pid(&self) -> PeerId {
        self.peer_id.read().await.clone()
    }

    pub async fn send(&self, msg: SendMessage) -> anyhow::Result<()> {
        self.sender
            .read()
            .await
            .send(msg)
            .await
            .map_err(|_e| anyhow!("network lost!"))
    }

    pub async fn clear(&self) {
        *self.peer_id.write().await = PeerId::default();
        self.layer.write().await.clear();
    }

    pub async fn reset(&self, pid: &PeerId, lock: &str) -> anyhow::Result<bool> {
        if *self.peer_id.read().await == *pid {
            return Ok(false);
        }

        let (pheight, oheight) =
            self.group
                .write()
                .await
                .reset(pid, lock, &self.base, &self.secret)?;
        self.layer.write().await.clear();

        *self.peer_id.write().await = *pid;
        *self.peer_pub_height.write().await = pheight;
        *self.peer_own_height.write().await = oheight;
        self._delivery.write().await.clear();

        // TODO change sender.

        Ok(true)
    }
}
