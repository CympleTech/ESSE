use tdn::types::{
    group::GroupId,
    message::RecvType,
    primitive::{new_io_error, HandleResult, Result},
};

use group_chat_types::GroupResult;
//use group_chat_types::{Event, GroupConnect, GroupEvent, GroupInfo, GroupResult, GroupType};

pub(crate) fn handle(_mgid: GroupId, msg: RecvType) -> Result<HandleResult> {
    let results = HandleResult::new();

    match msg {
        RecvType::Connect(_addr, _data) => {
            // None.
        }
        RecvType::Leave(_addr) => {
            //
        }
        RecvType::Result(_addr, _is_ok, data) => {
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
        RecvType::ResultConnect(_addr, data) => {
            let _res: GroupResult = postcard::from_bytes(&data)
                .map_err(|_e| new_io_error("Deseralize result failure"))?;
        }
        RecvType::Event(_addr, _bytes) => {
            //
        }
        RecvType::Stream(_uid, _stream, _bytes) => {
            // TODO stream
        }
        RecvType::Delivery(_t, _tid, _is_ok) => {
            //
        }
    }

    Ok(results)
}
