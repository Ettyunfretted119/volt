use serde::Serialize;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use sysinfo::{Pid, System};

#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub ram_mb: f64,
    pub cpu_percent: f32,
}

static SYS: std::sync::LazyLock<Mutex<System>> =
    std::sync::LazyLock::new(|| Mutex::new(System::new()));

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn get_system_stats() -> Result<SystemStats, String> {
    let pid = Pid::from_u32(std::process::id());
    let mut sys = SYS.lock().map_err(|e| format!("Lock error: {}", e))?;

    // First call: baseline refresh so next call can compute CPU delta.
    // Return early — RAM is accurate, CPU reads 0% (no prior baseline).
    // Next poll (5s later) will have a real delta for accurate CPU%.
    if !INITIALIZED.load(Ordering::Relaxed) {
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        INITIALIZED.store(true, Ordering::Relaxed);
        let stats = if let Some(process) = sys.process(pid) {
            SystemStats {
                ram_mb: process.memory() as f64 / 1_048_576.0,
                cpu_percent: 0.0,
            }
        } else {
            SystemStats { ram_mb: 0.0, cpu_percent: 0.0 }
        };
        return Ok(stats);
    }

    sys.refresh_processes(
        sysinfo::ProcessesToUpdate::Some(&[pid]),
        true,
    );

    if let Some(process) = sys.process(pid) {
        Ok(SystemStats {
            ram_mb: process.memory() as f64 / 1_048_576.0,
            cpu_percent: process.cpu_usage(),
        })
    } else {
        Ok(SystemStats {
            ram_mb: 0.0,
            cpu_percent: 0.0,
        })
    }
}
