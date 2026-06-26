use std::process::Command;
use std::thread;
use std::time::Duration;

use super::registry;

/// Kill all processes with the given name.
pub fn kill_process(name: &str) {
    // Use taskkill on Windows for reliable process termination
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", &format!("{}.exe", name)])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// Kill Battle.net and Agent processes, then wait for them to exit.
pub fn kill_battle_net() {
    kill_process("Battle.net");
    kill_process("Agent");
    thread::sleep(Duration::from_millis(1500));
}

/// Find and launch Battle.net.
pub fn launch_battle_net() {
    let exe_path = find_battle_net_exe();
    if let Some(path) = exe_path {
        let _ = Command::new(&path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

/// Find the Battle.net executable path.
fn find_battle_net_exe() -> Option<String> {
    // Try default installation path
    let default_path = r"C:\Program Files (x86)\Battle.net\Battle.net.exe";
    if std::path::Path::new(default_path).exists() {
        return Some(default_path.to_string());
    }

    // Try registry uninstall key
    if let Some(path) = registry::get_battle_net_install_path() {
        let exe = format!(r"{}\Battle.net.exe", path);
        if std::path::Path::new(&exe).exists() {
            return Some(exe);
        }
    }

    None
}
