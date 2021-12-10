use sysinfo::{DiskExt, ProcessorExt, System, SystemExt};

/// get this device status.
/// return: (cpu_num, memory space, swap space, disk space, cpu%, memory%, swap%, disk%)
/// use MB as default unit for memory, swap, disk and u32 MAX is 4096PB.
/// only 4-length number, max is 9999 (99.99%), min is 0 (0.00%)
pub(crate) fn device_status() -> (u32, u32, u32, u32, u16, u16, u16, u16) {
    let s = System::new_all();
    let cpu_n = s.physical_core_count().unwrap_or(0) as u32;
    let cpu = s.global_processor_info().cpu_usage();
    let cpu_p = (cpu * 100f32) as u16;

    let memory_t = (s.total_memory() / 1024) as u32; // MB
    let memory_u = (s.used_memory() / 1024) as f32;
    let memory_p = (memory_u / memory_t as f32 * 10000f32) as u16;

    let swap_t = (s.total_swap() / 1024) as u32;
    let swap_u = (s.used_swap() / 1024) as f32;
    let swap_p = (swap_u / swap_t as f32 * 10000f32) as u16;

    let mut disk_t = 0;
    let mut disk_a = 0;
    for disk in s.disks() {
        disk_t += disk.total_space();
        disk_a += disk.available_space();
    }
    let disk_t_n = (disk_t / 1048576) as u32;
    let disk_t_f = disk_t_n as f32;
    let disk_a_f = (disk_a / 1048576) as f32;
    let disk_p = ((disk_t_f - disk_a_f) / disk_t_f * 10000f32) as u16;

    (
        cpu_n, memory_t, swap_t, disk_t_n, cpu_p, memory_p, swap_p, disk_p,
    )
}
