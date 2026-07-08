use winreg::enums::*;
use winreg::RegKey;

use super::models::RegistryValueBackup;

const BNET_UNINSTALL_KEY: &str =
    r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Battle.net";

// Auth-related registry paths under HKCU
const REGISTRY_KEY_PATHS: &[(&str, Option<&str>)] = &[
    (
        r"Software\Blizzard Entertainment\Battle.net\UnifiedAuth",
        None,
    ),
    (
        r"Software\Blizzard Entertainment\Battle.net\Identity",
        None,
    ),
    (
        r"Software\Blizzard Entertainment\Battle.net\EncryptionKey",
        None,
    ),
    (
        r"Software\Blizzard Entertainment\Battle.net\Launch Options",
        Some("URI_TOKEN"),
    ),
];

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

// ─── Auth snapshot save/restore ───────────────────────────────────────

/// Save a snapshot of Battle.net auth registry values to a JSON file.
pub fn save_auth_snapshot(destination: &std::path::Path) {
    let mut records = Vec::new();

    for &(key_path, value_filter) in REGISTRY_KEY_PATHS {
        collect_registry_values(key_path, value_filter, &mut records);
    }

    if records.is_empty() {
        return;
    }

    if let Some(parent) = destination.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(json) = serde_json::to_string(&records) {
        let _ = std::fs::write(destination, json);
    }
}

/// Restore auth registry values from a snapshot file. Deletes existing values first.
pub fn restore_auth_snapshot(source: &std::path::Path) {
    let Ok(content) = std::fs::read_to_string(source) else {
        return;
    };
    let Ok(records) = serde_json::from_str::<Vec<RegistryValueBackup>>(&content) else {
        return;
    };

    // Delete existing values
    for &(key_path, value_filter) in REGISTRY_KEY_PATHS {
        delete_registry_values(key_path, value_filter);
    }

    // Restore
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for record in &records {
        if let Ok(key) = hkcu.create_subkey_with_flags(&record.key_path, KEY_WRITE) {
            let (subkey, _) = key;
            let kind = parse_value_kind(&record.value_kind);
            if let Some(value) = deserialize_registry_value(&record.data, kind) {
                let _ = subkey.set_raw_value(&record.value_name, &value);
            }
        }
    }
}

/// Backup and delete all auth registry values.
pub fn clear_auth_state() {
    let mut records = Vec::new();

    for &(key_path, value_filter) in REGISTRY_KEY_PATHS {
        backup_and_delete_registry_values(key_path, value_filter, &mut records);
    }

    if records.is_empty() {
        return;
    }

    // Save backup
    let backup_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("BattleNetManager")
        .join("BattleNetRegistryBackups")
        .join(chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());

    if let Ok(json) = serde_json::to_string(&records) {
        let _ = std::fs::create_dir_all(&backup_dir);
        let _ = std::fs::write(backup_dir.join("registry-values.json"), json);
    }

    prune_old_registry_backups();
}

// ─── Internal helpers ─────────────────────────────────────────────────

/// Read registry values from a key and add them to the records list.
fn collect_registry_values(
    key_path: &str,
    value_name_filter: Option<&str>,
    records: &mut Vec<RegistryValueBackup>,
) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey_with_flags(key_path, KEY_READ) else {
        return;
    };

    let names = match value_name_filter {
        Some(filter) => key
            .enum_values()
            .filter_map(|r| r.ok())
            .filter(|(name, _)| name.eq_ignore_ascii_case(filter))
            .map(|(name, _)| name)
            .collect::<Vec<_>>(),
        None => key
            .enum_values()
            .filter_map(|r| r.ok())
            .map(|(name, _)| name)
            .collect::<Vec<_>>(),
    };

    for name in names {
        if let Ok(value) = key.get_raw_value(&name) {
            records.push(RegistryValueBackup {
                key_path: key_path.to_string(),
                value_name: name,
                value_kind: format!("{:?}", value.vtype),
                data: serialize_raw_value(&value),
            });
        }
    }
}

/// Delete registry values from a key.
fn delete_registry_values(key_path: &str, value_name_filter: Option<&str>) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey_with_flags(key_path, KEY_ALL_ACCESS) else {
        return;
    };

    match value_name_filter {
        Some(filter) => {
            let names: Vec<String> = key
                .enum_values()
                .filter_map(|r| r.ok())
                .filter(|(name, _)| name.eq_ignore_ascii_case(filter))
                .map(|(name, _)| name)
                .collect();
            for name in names {
                let _ = key.delete_value(&name);
            }
        }
        None => {
            let names: Vec<String> = key
                .enum_values()
                .filter_map(|r| r.ok())
                .map(|(name, _)| name)
                .collect();
            for name in names {
                let _ = key.delete_value(&name);
            }
        }
    }
}

/// Backup and then delete registry values.
fn backup_and_delete_registry_values(
    key_path: &str,
    value_name_filter: Option<&str>,
    records: &mut Vec<RegistryValueBackup>,
) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey_with_flags(key_path, KEY_ALL_ACCESS) else {
        return;
    };

    let names = match value_name_filter {
        Some(filter) => key
            .enum_values()
            .filter_map(|r| r.ok())
            .filter(|(name, _)| name.eq_ignore_ascii_case(filter))
            .map(|(name, _)| name)
            .collect::<Vec<_>>(),
        None => key
            .enum_values()
            .filter_map(|r| r.ok())
            .map(|(name, _)| name)
            .collect::<Vec<_>>(),
    };

    for name in names {
        if let Ok(value) = key.get_raw_value(&name) {
            records.push(RegistryValueBackup {
                key_path: key_path.to_string(),
                value_name: name.clone(),
                value_kind: format!("{:?}", value.vtype),
                data: serialize_raw_value(&value),
            });
        }
        let _ = key.delete_value(&name);
    }
}

// ─── Value serialization ──────────────────────────────────────────────

fn serialize_raw_value(value: &winreg::RegValue) -> String {
    match value.vtype {
        REG_BINARY => base64_encode(&value.bytes),
        REG_DWORD => {
            if value.bytes.len() >= 4 {
                let val = u32::from_le_bytes([
                    value.bytes[0],
                    value.bytes[1],
                    value.bytes[2],
                    value.bytes[3],
                ]);
                val.to_string()
            } else {
                base64_encode(&value.bytes)
            }
        }
        REG_DWORD_BIG_ENDIAN => {
            if value.bytes.len() >= 4 {
                let val = u32::from_be_bytes([
                    value.bytes[0],
                    value.bytes[1],
                    value.bytes[2],
                    value.bytes[3],
                ]);
                val.to_string()
            } else {
                base64_encode(&value.bytes)
            }
        }
        REG_QWORD => {
            if value.bytes.len() >= 8 {
                let val = u64::from_le_bytes([
                    value.bytes[0],
                    value.bytes[1],
                    value.bytes[2],
                    value.bytes[3],
                    value.bytes[4],
                    value.bytes[5],
                    value.bytes[6],
                    value.bytes[7],
                ]);
                val.to_string()
            } else {
                base64_encode(&value.bytes)
            }
        }
        REG_SZ | REG_EXPAND_SZ => {
            // UTF-16LE string, strip null terminator
            let utf16: Vec<u16> = value
                .bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .take_while(|&c| c != 0)
                .collect();
            String::from_utf16_lossy(&utf16)
        }
        REG_MULTI_SZ => {
            let utf16: Vec<u16> = value
                .bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let strings: Vec<String> = String::from_utf16_lossy(&utf16)
                .split('\0')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            serde_json::to_string(&strings).unwrap_or_else(|_| "[]".to_string())
        }
        _ => base64_encode(&value.bytes),
    }
}

fn deserialize_registry_value(data: &str, kind: RegType) -> Option<winreg::RegValue> {
    let bytes = match kind {
        REG_BINARY => base64_decode(data),
        REG_DWORD => {
            let val: u32 = data.parse().ok()?;
            Some(val.to_le_bytes().to_vec())
        }
        REG_DWORD_BIG_ENDIAN => {
            let val: u32 = data.parse().ok()?;
            Some(val.to_be_bytes().to_vec())
        }
        REG_QWORD => {
            let val: u64 = data.parse().ok()?;
            Some(val.to_le_bytes().to_vec())
        }
        REG_SZ | REG_EXPAND_SZ => {
            let mut utf16: Vec<u16> = data.encode_utf16().collect();
            utf16.push(0); // null terminator
            let bytes: Vec<u8> = utf16.iter().flat_map(|c| c.to_le_bytes()).collect();
            Some(bytes)
        }
        REG_MULTI_SZ => {
            let strings: Vec<String> = serde_json::from_str(data).ok()?;
            let mut utf16: Vec<u16> = Vec::new();
            for s in &strings {
                utf16.extend(s.encode_utf16());
                utf16.push(0);
            }
            utf16.push(0); // final null terminator
            let bytes: Vec<u8> = utf16.iter().flat_map(|c| c.to_le_bytes()).collect();
            Some(bytes)
        }
        _ => base64_decode(data),
    };

    bytes.map(|b| winreg::RegValue {
        vtype: kind,
        bytes: b,
    })
}

fn parse_value_kind(kind_str: &str) -> RegType {
    match kind_str {
        "REG_BINARY" => REG_BINARY,
        "REG_DWORD" => REG_DWORD,
        "REG_DWORD_BIG_ENDIAN" => REG_DWORD_BIG_ENDIAN,
        "REG_QWORD" => REG_QWORD,
        "REG_SZ" => REG_SZ,
        "REG_EXPAND_SZ" => REG_EXPAND_SZ,
        "REG_MULTI_SZ" => REG_MULTI_SZ,
        _ => REG_SZ,
    }
}

// ─── Base64 helpers ───────────────────────────────────────────────────

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}

// ─── Backup pruning ──────────────────────────────────────────────────

fn prune_old_registry_backups() {
    let backups_root = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("BattleNetManager")
        .join("BattleNetRegistryBackups");

    prune_old_backups(&backups_root);
}

/// Keep only the 3 most recent subdirectories in `root`.
pub fn prune_old_backups(root: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    let mut dirs: Vec<(std::path::PathBuf, std::time::SystemTime)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let time = e.metadata().ok()?.created().ok()?;
            Some((e.path(), time))
        })
        .collect();

    dirs.sort_by(|a, b| b.1.cmp(&a.1)); // newest first

    for (path, _) in dirs.iter().skip(3) {
        let _ = std::fs::remove_dir_all(path);
    }
}
