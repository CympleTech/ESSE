use group_chat_types::{Event, LayerEvent};
use std::path::PathBuf;
use tdn::types::{
    group::GroupId,
    primitive::{HandleResult, Result},
};
use tdn_storage::local::DStorage;

use crate::apps::chat::Friend;
use crate::rpc::session_last;
use crate::session::{Session, SessionType};

use crate::storage::{chat_db, delete_avatar, session_db, write_avatar_sync};

use super::models::{from_network_message, Member};
use super::rpc;

pub async fn handle_event(
    db: DStorage,
    base: PathBuf,
    gid: i64,      // group chat database id.
    mgid: GroupId, // me account(group_id)
    event: LayerEvent,
    results: &mut HandleResult,
) -> Result<()> {
    match event {
        LayerEvent::Sync(_gcd, height, event) => {
            match event {
                Event::GroupInfo => {}
                Event::GroupTransfer => {}
                Event::GroupManagerAdd => {}
                Event::GroupManagerDel => {}
                Event::GroupClose => {}
                Event::MemberInfo(mid, maddr, mname, mavatar) => {
                    let id = Member::get_id(&db, &gid, &mid)?;
                    Member::update(&db, &id, &maddr, &mname)?;
                    if mavatar.len() > 0 {
                        write_avatar_sync(&base, &mgid, &mid, mavatar)?;
                    }
                    results.rpcs.push(rpc::member_info(mgid, id, maddr, mname));
                }
                Event::MemberJoin(mid, maddr, mname, mavatar, mtime) => {
                    if Member::get_id(&db, &gid, &mid).is_err() {
                        let mut member = Member::new(gid, mid, maddr, mname, false, mtime);
                        member.insert(&db)?;
                        if mavatar.len() > 0 {
                            write_avatar_sync(&base, &mgid, &mid, mavatar)?;
                        }
                        results.rpcs.push(rpc::member_join(mgid, member));
                    }
                }
                Event::MemberLeave(mid) => {
                    let id = Member::get_id(&db, &gid, &mid)?;
                    Member::leave(&db, &id)?;
                    // check mid is my chat friend. if not, delete avatar.
                    let s_db = chat_db(&base, &mgid)?;
                    if Friend::get(&s_db, &mid)?.is_none() {
                        let _ = delete_avatar(&base, &mgid, &mid).await;
                    }
                    results.rpcs.push(rpc::member_leave(mgid, id));
                }
                Event::MessageCreate(mid, nmsg, mtime) => {
                    println!("Sync: create message start");
                    let (msg, scontent) =
                        from_network_message(height, gid, mid, &mgid, nmsg, mtime, &base)?;
                    results.rpcs.push(rpc::message_create(mgid, &msg));
                    println!("Sync: create message ok");

                    // UPDATE SESSION.
                    let s_db = session_db(&base, &mgid)?;
                    if let Ok(id) = Session::last(
                        &s_db,
                        &gid,
                        &SessionType::Group,
                        &msg.datetime,
                        &scontent,
                        true,
                    ) {
                        results
                            .rpcs
                            .push(session_last(mgid, &id, &msg.datetime, &scontent, false));
                    }
                }
            }
        }
        _ => {} // TODO
    }

    Ok(())
}
