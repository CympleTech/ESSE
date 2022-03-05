use std::sync::Arc;
use tdn::types::{
    primitives::HandleResult,
    rpc::{json, rpc_response, RpcError, RpcHandler, RpcParam},
};

use crate::global::Global;
//use crate::group::GroupEvent;
use crate::utils::device_status::device_status as local_device_status;

use super::Device;

#[inline]
pub(crate) fn device_create(device: &Device) -> RpcParam {
    rpc_response(0, "device-create", json!(device.to_rpc()))
}

#[inline]
pub(crate) fn _device_remove(id: i64) -> RpcParam {
    rpc_response(0, "device-remove", json!([id]))
}

#[inline]
pub(crate) fn device_online(id: i64) -> RpcParam {
    rpc_response(0, "device-online", json!([id]))
}

#[inline]
pub(crate) fn device_offline(id: i64) -> RpcParam {
    rpc_response(0, "device-offline", json!([id]))
}

#[inline]
pub(crate) fn device_status(
    cpu: u32,
    memory: u32,
    swap: u32,
    disk: u32,
    cpu_p: u16,
    memory_p: u16,
    swap_p: u16,
    disk_p: u16,
    uptime: u32,
) -> RpcParam {
    rpc_response(
        0,
        "device-status",
        json!([cpu, memory, swap, disk, cpu_p, memory_p, swap_p, disk_p, uptime]),
    )
}

#[inline]
fn device_list(devices: &[Device]) -> RpcParam {
    let mut results = vec![];
    for device in devices {
        results.push(device.to_rpc());
    }
    json!(results)
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<Global>) {
    handler.add_method("device-echo", |params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "device-list",
        |_params: Vec<RpcParam>, state: Arc<Global>| async move {
            let devices = &state.group.read().await.distributes;
            Ok(HandleResult::rpc(device_list(devices)))
        },
    );

    handler.add_method(
        "device-status",
        |params: Vec<RpcParam>, state: Arc<Global>| async move {
            let id = params[0].as_i64().ok_or(RpcError::ParseError)?;

            let group_lock = state.group.read().await;
            if id == group_lock.device()?.id {
                let uptime = group_lock.uptime;
                let (cpu, memory, swap, disk, cpu_p, memory_p, swap_p, disk_p) =
                    local_device_status();
                return Ok(HandleResult::rpc(json!([
                    cpu, memory, swap, disk, cpu_p, memory_p, swap_p, disk_p, uptime
                ])));
            }
            drop(group_lock);

            //let msg = state.group.write().await.event_message(addr, &GroupEvent::StatusRequest)?;
            //Ok(HandleResult::group(msg))
            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "device-search",
        |_params: Vec<RpcParam>, state: Arc<Global>| async move {
            //let msg = state.group.read().await.create_message(&gid, Peer::peer(addr))?;
            //Ok(HandleResult::group(gid, msg))
            Ok(HandleResult::new())
        },
    );

    handler.add_method(
        "device-delete",
        |params: Vec<RpcParam>, _state: Arc<Global>| async move {
            let _id = params[0].as_i64().ok_or(RpcError::ParseError)?;
            // TODO delete a device.
            Ok(HandleResult::new())
        },
    );
}
