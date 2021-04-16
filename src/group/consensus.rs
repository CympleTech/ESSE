struct RunningAccount {
    /// secret keypair.
    keypair: Keypair,
    /// device's name.
    device_name: String,
    /// device's info.
    device_info: String,
    /// distribute connected devices.
    distributes: HashMap<PeerAddr, (i64, bool)>,
    /// uptime
    uptime: u32,
    /// pool to store consensus event.
    pool: HashMap<EventId, i64>,
    /// voting block.
    vote_block: BlockId,
    /// voting confirm.
    vote_confirm: Vec<PeerAddr>,
    /// next block miner.
    next_miner: PeerAdr,
}

pub(crate) enum ConsensusEvent {
    /// Sync block request.
    BlockRequest(u64),
    /// Sync block response.
    BlockResponse(BlockFullData),
    /// Sync pool event request.
    PoolRequest(EventId),
    /// Sync pool event response.
    PoolResponse(EventData),
    /// Sync event request.
    EventRequest(EventId),
    /// Sync event response.
    EventResponse(EventData),
    /// Consensus: mined block and next miner.
    Mine(BlockData, GroupId),
    /// Consensus: confirm block.
    MineConfirm(u64),
    /// miner lost, need new.
    MinerNew(PeerAddr),
    /// confirm miner is lost.
    MinerNewConfirm(bool),
    /// Consensus: Miner lost, re-vote.
    Vote(u64, GroupId),
    /// Consensus: Re-vote confirm.
    VoteConfirm(u64, GroupId),
}

pub(crate) async fn group_mine(group: Arc<RwLock<Group>>, gid: GroupId) -> Result<()> {
    // 10-minutes
    let events = group.write().await.runnings.get(&gid)?.mine();
    if events.len() > 0 {
        // create block.

        // save block.

        // update consensus status.

        // broadcast block.
    }
}

/// current height.
pub(crate) async fn sync_req(group: &mut Group) -> Result<u64> {
    todo!()
}

/// current height. blocks, pool events.
pub(crate) async fn sync_res(group: &mut Group) -> Result<(u64, Vec<BlockId>, Vec<EventId>)> {
    todo!()
}

/// after connect & sync block
pub(crate) fn sync_handle(group: &mut Group, height: u64) -> Result<()> {
    let my_height = 0;

    if my_height < height {
        // create block sync request with block_height.
        my_height + 1;
    } else if my_height == height {
        // sync pools
    }

    todo!();
}
