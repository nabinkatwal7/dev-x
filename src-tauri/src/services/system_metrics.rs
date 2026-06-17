use std::sync::Mutex;

use crate::error::AppError;
use crate::models::SystemMetrics;
use sysinfo::{Disks, Networks, System};

lazy_static::lazy_static! {
    static ref METRICS_SYS: Mutex<System> = Mutex::new(System::new());
}

fn sanitize_f64(v: f64) -> f64 {
    if v.is_nan() || v.is_infinite() { 0.0 } else { v }
}

pub fn get_metrics() -> Result<SystemMetrics, AppError> {
    let mut sys = METRICS_SYS.lock().map_err(|e| AppError::Internal(format!("lock: {}", e)))?;
    sys.refresh_memory();
    sys.refresh_cpu_specifics(sysinfo::CpuRefreshKind::everything());
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let total = sys.total_memory();
    let used = sys.used_memory();
    let mem_percent = if total > 0 { sanitize_f64((used as f64 / total as f64) * 100.0) } else { 0.0 };

    let cpu_percent = sanitize_f64(sys.global_cpu_usage() as f64);

    let networks = Networks::new_with_refreshed_list();
    let (mut rx, mut tx) = (0u64, 0u64);
    for (_, data) in &networks {
        rx += data.total_received();
        tx += data.total_transmitted();
    }

    let disks = Disks::new_with_refreshed_list();
    let (mut disk_used, mut disk_total) = (0u64, 0u64);
    for disk in &disks {
        disk_total += disk.total_space();
        disk_used += disk.total_space() - disk.available_space();
    }

    let uptime_secs = System::uptime();
    let process_count = sys.processes().len();

    let metrics = SystemMetrics {
        memory_used_gb: sanitize_f64((used as f64) / (1024.0 * 1024.0 * 1024.0)),
        memory_total_gb: sanitize_f64((total as f64) / (1024.0 * 1024.0 * 1024.0)),
        memory_percent: (mem_percent * 100.0).round() / 100.0,
        network_rx_bytes: rx,
        network_tx_bytes: tx,
        cpu_percent: (cpu_percent * 100.0).round() / 100.0,
        disk_used_gb: sanitize_f64((disk_used as f64) / (1024.0 * 1024.0 * 1024.0)),
        disk_total_gb: sanitize_f64((disk_total as f64) / (1024.0 * 1024.0 * 1024.0)),
        uptime_secs,
        process_count,
    };

    Ok(metrics)
}
