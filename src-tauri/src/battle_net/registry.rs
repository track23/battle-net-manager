use winreg::enums::*;
use winreg::RegKey;

const BNET_UNINSTALL_KEY: &str =
    r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Battle.net";

/// Get Battle.net install path from registry.
pub fn get_battle_net_install_path() -> Option<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey_with_flags(BNET_UNINSTALL_KEY, KEY_READ) {
        Ok(key) => key
            .get_value::<String, _>("InstallLocation")
            .ok()
            .filter(|s| !s.is_empty()),
        Err(_) => None,
    }
}
