use std::time::{Duration, Instant};

use super::registry;

// ─── Process enumeration ──────────────────────────────────────────────

/// Get PIDs of all Battle.net-related processes (Agent + Battle.net.*).
pub fn get_battle_net_processes() -> Vec<u32> {
    let mut pids = get_processes_by_name("Agent");
    pids.extend(get_processes_by_name("Battle.net"));
    pids
}

/// Get PIDs of Battle.net client processes only (Battle.net.*).
pub fn get_battle_net_client_processes() -> Vec<u32> {
    get_processes_by_name("Battle.net")
}

/// Check if any Battle.net client process is running.
pub fn is_battle_net_client_running() -> bool {
    !get_battle_net_client_processes().is_empty()
}

/// Get PIDs of processes matching an image name prefix.
/// Uses `tasklist` for reliable process enumeration on Windows.
fn get_processes_by_name(name_prefix: &str) -> Vec<u32> {
    let Ok(output) = std::process::Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
    else {
        return Vec::new();
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let prefix_lower = name_prefix.to_lowercase();

    stdout
        .lines()
        .filter_map(|line| {
            // CSV format: "Image Name","PID","Session Name","Session#","Mem Usage"
            let mut parts = line.split(',');
            let image = parts.next()?.trim_matches('"');
            if !image.to_lowercase().starts_with(&prefix_lower) {
                return None;
            }
            let pid_str = parts.next()?.trim_matches('"');
            pid_str.trim().parse::<u32>().ok()
        })
        .collect()
}

// ─── Graceful close ───────────────────────────────────────────────────

/// Ask Battle.net client to close gracefully by sending WM_CLOSE to its windows.
/// Uses PowerShell P/Invoke for reliable GUI app close.
pub fn request_battle_net_close() {
    let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class WinAPI {
    [DllImport("user32.dll")]
    public static extern IntPtr SendMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
}
"@

Get-Process | Where-Object { $_.ProcessName -like 'Battle.net*' } | ForEach-Object {
    if ($_.MainWindowHandle -ne [IntPtr]::Zero) {
        [WinAPI]::SendMessage($_.MainWindowHandle, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
    }
}
"#;

    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

// ─── Wait for exit ────────────────────────────────────────────────────

/// Wait for all Battle.net processes to exit. Requests graceful close periodically.
/// Returns true if all processes exited within the timeout.
pub async fn ensure_cleanly_exited(stable_delay: Duration, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let mut client_empty_since: Option<Instant> = None;

    loop {
        if Instant::now() >= deadline {
            return false;
        }

        let client_pids = get_battle_net_client_processes();
        if client_pids.is_empty() {
            let now = Instant::now();
            let since = client_empty_since.get_or_insert(now);
            if now.duration_since(*since) >= stable_delay {
                return true;
            }
        } else {
            client_empty_since = None;
            // Request graceful close again
            request_battle_net_close();
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

// ─── Kill processes ───────────────────────────────────────────────────

/// Kill specific processes by PID using taskkill.
pub fn kill_processes(pids: &[u32]) {
    for &pid in pids {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

/// Kill all Battle.net processes (client + agent), then wait briefly.
pub async fn kill_battle_net() {
    let pids = get_battle_net_processes();
    kill_processes(&pids);
    tokio::time::sleep(Duration::from_millis(1500)).await;
}

// ─── Launch ───────────────────────────────────────────────────────────

/// Find and launch Battle.net.
pub fn launch_battle_net() {
    if let Some(path) = find_battle_net_exe() {
        let _ = std::process::Command::new(&path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

/// Find the Battle.net executable path.
pub fn find_battle_net_exe() -> Option<String> {
    let default_path = r"C:\Program Files (x86)\Battle.net\Battle.net.exe";
    if std::path::Path::new(default_path).exists() {
        return Some(default_path.to_string());
    }

    if let Some(path) = registry::get_battle_net_install_path() {
        let exe = format!(r"{}\Battle.net.exe", path);
        if std::path::Path::new(&exe).exists() {
            return Some(exe);
        }
    }

    None
}
