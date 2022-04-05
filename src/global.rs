use std::collections::HashMap;
use std::path::PathBuf;
use tdn::{
    prelude::{GroupId, P2pConfig, PeerId, PeerKey, ReceiveMessage, SendMessage},
    types::message::RpcSendMessage,
};
use tokio::{sync::mpsc::Sender, sync::RwLock};

use crate::account::Account;
use crate::layer::Layer;
use crate::own::Own;

/// global status.
pub(crate) struct Global {
    /// current running account.
    pub peer_id: RwLock<PeerId>,
    /// current account public height.
    pub peer_pub_height: RwLock<u64>,
    /// current account own height.
    pub peer_own_height: RwLock<u64>,
    /// current own.
    pub own: RwLock<Own>,
    /// current layer.
    pub layer: RwLock<Layer>,
    /// message delivery tracking. uuid, me_gid, db_id.
    pub _delivery: RwLock<HashMap<u64, (GroupId, i64)>>,
    /// storage base path.
    pub base: PathBuf,
    /// random secret seed.
    pub secret: [u8; 32],
    /// supported layers.
    pub gids: Vec<GroupId>,
    /// inner network params.
    pub p2p_config: P2pConfig,
    /// inner services channel sender.
    pub self_send: Sender<ReceiveMessage>,
    /// inner p2p network sender.
    pub p2p_send: RwLock<Option<Sender<SendMessage>>>,
    /// inner rpc channel sender.
    pub rpc_send: Sender<RpcSendMessage>,
}

impl Global {
    pub fn init(
        accounts: HashMap<PeerId, Account>,
        base: PathBuf,
        secret: [u8; 32],
        p2p_config: P2pConfig,
        self_send: Sender<ReceiveMessage>,
        rpc_send: Sender<RpcSendMessage>,
    ) -> Self {
        let gids = vec![0]; // ESSE DEFAULT IS 0

        Global {
            base,
            secret,
            p2p_config,
            self_send,
            rpc_send,
            gids,
            peer_id: RwLock::new(PeerId::default()),
            peer_pub_height: RwLock::new(0),
            peer_own_height: RwLock::new(0),
            own: RwLock::new(Own::init(accounts)),
            layer: RwLock::new(Layer::init()),
            p2p_send: RwLock::new(None),
            _delivery: RwLock::new(HashMap::new()),
        }
    }

    pub async fn pid(&self) -> PeerId {
        self.peer_id.read().await.clone()
    }

    pub async fn sender(&self) -> anyhow::Result<Sender<SendMessage>> {
        self.p2p_send
            .read()
            .await
            .clone()
            .ok_or(anyhow!("network lost!"))
    }

    pub async fn send(&self, msg: SendMessage) -> anyhow::Result<()> {
        if let Some(sender) = &*self.p2p_send.read().await {
            Ok(sender.send(msg).await?)
        } else {
            Err(anyhow!("network lost!"))
        }
    }

    pub async fn clear(&self) {
        *self.peer_id.write().await = PeerId::default();
        self.layer.write().await.clear();
    }

    pub async fn reset(
        &self,
        pid: &PeerId,
        lock: &str,
        send: Sender<SendMessage>,
    ) -> anyhow::Result<bool> {
        if *self.peer_id.read().await == *pid {
            return Ok(true);
        }

        let (pheight, oheight) =
            self.own
                .write()
                .await
                .reset(pid, lock, &self.base, &self.secret)?;
        self.layer.write().await.clear();

        *self.p2p_send.write().await = Some(send);
        *self.peer_id.write().await = *pid;
        *self.peer_pub_height.write().await = pheight;
        *self.peer_own_height.write().await = oheight;
        self._delivery.write().await.clear();

        Ok(false)
    }
}
