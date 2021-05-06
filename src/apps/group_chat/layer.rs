use tdn::types::{
    group::GroupId,
    message::{RecvType, SendType},
    primitive::{new_io_error, HandleResult, Result},
};

use group_chat_types::{Event, GroupConnect, GroupEvent, GroupInfo, GroupResult, GroupType};

pub(crate) fn handle(mgid: GroupId, msg: RecvType) -> Result<HandleResult> {
    let mut results = HandleResult::new();

    match msg {
        RecvType::Connect(addr, data) => {
            // None.
        }
        RecvType::Leave(addr) => {
            //
        }
        RecvType::Result(addr, is_ok, data) => {
            let res: GroupResult = postcard::from_bytes(&data)
                .map_err(|_e| new_io_error("Deseralize result failure"))?;
            match res {
                GroupResult::Check(is_ok, supported) => {
                    println!("check: {}, supported: {:?}", is_ok, supported);
                }
                _ => {
                    //
                }
            }
        }
        RecvType::ResultConnect(addr, data) => {
            let _res: GroupResult = postcard::from_bytes(&data)
                .map_err(|_e| new_io_error("Deseralize result failure"))?;
        }
        RecvType::Event(addr, bytes) => {
            //
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }
        RecvType::Delivery(t, tid, is_ok) => {
            //
        }
    }

    Ok(results)
}
