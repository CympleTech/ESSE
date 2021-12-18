use std::path::PathBuf;
use tdn::types::{group::GroupId, primitive::Result};
use tdn_storage::local::{DStorage, DsValue};

use group_types::PackedEvent;

use super::{to_network_message, Member, Message};

pub(crate) enum ConsensusType {
    GroupInfo,
    GroupTransfer,
    GroupManagerAdd,
    GroupManagerDel,
    GroupClose,
    MemberInfo,
    MemberJoin,
    MemberLeave,
    MessageCreate,
    None,
}

impl ConsensusType {
    fn to_i64(&self) -> i64 {
        match self {
            ConsensusType::None => 0,
            ConsensusType::GroupInfo => 1,
            ConsensusType::GroupTransfer => 2,
            ConsensusType::GroupManagerAdd => 3,
            ConsensusType::GroupManagerDel => 4,
            ConsensusType::GroupClose => 5,
            ConsensusType::MemberInfo => 6,
            ConsensusType::MemberJoin => 7,
            ConsensusType::MemberLeave => 8,
            ConsensusType::MessageCreate => 9,
        }
    }

    fn from_i64(a: i64) -> Self {
        match a {
            1 => ConsensusType::GroupInfo,
            2 => ConsensusType::GroupTransfer,
            3 => ConsensusType::GroupManagerAdd,
            4 => ConsensusType::GroupManagerDel,
            5 => ConsensusType::GroupClose,
            6 => ConsensusType::MemberInfo,
            7 => ConsensusType::MemberJoin,
            8 => ConsensusType::MemberLeave,
            9 => ConsensusType::MessageCreate,
            _ => ConsensusType::None,
        }
    }
}

/// Group Chat Consensus.
pub(crate) struct Consensus {
    /// db auto-increment id.
    _id: i64,
    /// group's db id.
    _fid: i64,
    /// group's height.
    _height: i64,
    /// consensus type.
    ctype: ConsensusType,
    /// consensus point value db id.
    cid: i64,
}

impl Consensus {
    fn from_values(mut v: Vec<DsValue>) -> Consensus {
        Consensus {
            cid: v.pop().unwrap().as_i64(),
            ctype: ConsensusType::from_i64(v.pop().unwrap().as_i64()),
            _height: v.pop().unwrap().as_i64(),
            _fid: v.pop().unwrap().as_i64(),
            _id: v.pop().unwrap().as_i64(),
        }
    }

    pub async fn pack(
        db: &DStorage,
        base: &PathBuf,
        gcd: &GroupId,
        fid: &i64,
        from: &i64,
        to: &i64,
    ) -> Result<Vec<PackedEvent>> {
        let matrix = db.query(&format!(
            "SELECT id, fid, height, ctype, cid FROM consensus WHERE fid = {} AND height BETWEEN {} AND {}", fid, from, to))?;

        let mut packed = vec![];
        let mut consensuses = vec![];
        for res in matrix {
            consensuses.push(Consensus::from_values(res));
        }

        for consensus in consensuses {
            match consensus.ctype {
                ConsensusType::GroupInfo => {
                    //
                }
                ConsensusType::GroupTransfer => {
                    //
                }
                ConsensusType::GroupManagerAdd => {
                    //
                }
                ConsensusType::GroupManagerDel => {
                    //
                }
                ConsensusType::GroupClose => {
                    //
                }
                ConsensusType::MemberInfo => {
                    //
                }
                ConsensusType::MemberJoin => {
                    let m = Member::get(db, &consensus.cid)?;
                    // TODO load member avatar.
                    let mavatar = vec![];
                    packed.push(PackedEvent::MemberJoin(
                        m.m_id, m.m_addr, m.m_name, mavatar, m.datetime,
                    ))
                }
                ConsensusType::MemberLeave => {
                    //
                }
                ConsensusType::MessageCreate => {
                    let m = Message::get(db, &consensus.cid)?;
                    let datetime = m.datetime;
                    let mem = Member::get(db, &m.mid)?;
                    let (nmsg, _) = to_network_message(base, gcd, m.m_type, &m.content).await?;
                    packed.push(PackedEvent::MessageCreate(mem.m_id, nmsg, datetime))
                }
                ConsensusType::None => {
                    // None
                }
            }
        }

        Ok(packed)
    }

    pub fn insert(
        db: &DStorage,
        fid: &i64,
        height: &i64,
        cid: &i64,
        ctype: &ConsensusType,
    ) -> Result<()> {
        let mut unique_check = db.query(&format!(
            "SELECT id from consensus WHERE fid = {} AND height = {}",
            fid, height
        ))?;

        if unique_check.len() > 0 {
            let id = unique_check.pop().unwrap().pop().unwrap().as_i64();
            let _ = db.query(&format!(
                "UPDATE consensus SET ctype = {}, cid = {} WHERE id = {}",
                ctype.to_i64(),
                cid,
                id
            ))?;
        } else {
            let _ = db.query(&format!(
                "INSERT INTO consensus ( fid, height, ctype, cid ) VALUES ( {}, {}, {}, {} )",
                fid,
                height,
                ctype.to_i64(),
                cid
            ))?;
        }

        Ok(())
    }
}
