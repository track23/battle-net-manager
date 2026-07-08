use std::fs;
use std::path::{Path, PathBuf};

use super::models::SessionStateMetadata;
use super::region::{infer_region_from_config, is_tagged_region, normalize_region};
use super::registry;

// ─── Directory paths ──────────────────────────────────────────────────

fn session_state_dir(account_dir: &Path) -> PathBuf {
    account_dir.join("BattleNetSessionState")
}

fn session_state_candidate_dir(account_dir: &Path) -> PathBuf {
    account_dir.join("BattleNetSessionStateCandidate")
}

fn session_state_backup_dir(account_dir: &Path) -> PathBuf {
    account_dir.join("BattleNetSessionStateBackup")
}

fn metadata_path_from_state_dir(state_dir: &Path) -> PathBuf {
    state_dir.join("metadata.json")
}

fn _metadata_path(account_dir: &Path) -> PathBuf {
    metadata_path_from_state_dir(&session_state_dir(account_dir))
}

// ─── Public API ───────────────────────────────────────────────────────

/// Check whether an account has a usable session state snapshot.
pub fn has_session_state(account_dir: &Path) -> bool {
    is_usable_state(&session_state_dir(account_dir))
}

/// Save a session snapshot for an account. Uses an atomic candidate → commit pattern.
/// If `require_exact_region` is true, the current Battle.net.config must match the
/// expected region before saving.
pub fn save_session_snapshot(
    config_path: &Path,
    account_dir: &Path,
    expected_region: &str,
    require_exact_region: bool,
) -> bool {
    if !config_path.exists() {
        return false;
    }

    let normalized_region = normalize_region(expected_region);
    if require_exact_region {
        let current_region = infer_region_from_config(config_path);
        if !is_tagged_region(&current_region) || current_region != normalized_region {
            return false;
        }
    }

    let candidate_dir = session_state_candidate_dir(account_dir);
    let candidate_config = account_dir.join("Battle.net.config.candidate");
    let saved_config = account_dir.join("Battle.net.config");

    let result = (|| -> Option<bool> {
        let _ = fs::create_dir_all(account_dir);
        let _ = try_delete_dir(&candidate_dir);
        let _ = fs::remove_file(&candidate_config);

        // Save state to candidate directory
        if !save_state_to_directory(&candidate_dir) {
            return Some(false);
        }

        // Validate the snapshot is usable
        if !is_usable_state(&candidate_dir) {
            let _ = try_delete_dir(&candidate_dir);
            return Some(false);
        }

        // Copy current config to candidate
        fs::copy(config_path, &candidate_config).ok()?;

        // Atomic commit
        if !commit_session_state(&candidate_dir, account_dir) {
            let _ = try_delete_dir(&candidate_dir);
            let _ = fs::remove_file(&candidate_config);
            return Some(false);
        }

        // Copy config to final location
        let _ = fs::copy(&candidate_config, &saved_config);
        let _ = fs::remove_file(&candidate_config);
        Some(true)
    })();

    match result {
        Some(success) => success,
        None => {
            let _ = try_delete_dir(&candidate_dir);
            let _ = fs::remove_file(account_dir.join("Battle.net.config.candidate"));
            false
        }
    }
}

/// Restore session state from a saved snapshot.
pub fn restore_session_state(account_dir: &Path) {
    let state_dir = session_state_dir(account_dir);
    if !state_dir.exists() {
        return;
    }

    let local_bnet_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Battle.net");

    let local_state_dir = state_dir.join("Local");

    // Restore local state directories
    restore_directory_snapshot(
        &local_state_dir.join("Account"),
        &local_bnet_dir.join("Account"),
    );
    restore_directory_snapshot(
        &local_state_dir.join("BrowserCaches"),
        &local_bnet_dir.join("BrowserCaches"),
    );
    restore_directory_snapshot(
        &local_state_dir.join("Cache"),
        &local_bnet_dir.join("Cache"),
    );
    restore_file_snapshot(
        &local_state_dir.join("CachedData.db"),
        &local_bnet_dir.join("CachedData.db"),
    );

    // Restore registry auth state
    registry::restore_auth_snapshot(&state_dir.join("Registry").join("registry-values.json"));
}

// ─── State validation ─────────────────────────────────────────────────

fn is_usable_state(state_dir: &Path) -> bool {
    if !state_dir.exists() {
        return false;
    }

    if !metadata_path_from_state_dir(state_dir).exists() {
        return false;
    }

    let registry_path = state_dir.join("Registry").join("registry-values.json");
    let local_state_dir = state_dir.join("Local");

    has_usable_registry_snapshot(&registry_path) && has_local_session_state(&local_state_dir)
}

fn has_usable_registry_snapshot(registry_path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(registry_path) else {
        return false;
    };
    let Ok(records) = serde_json::from_str::<Vec<super::models::RegistryValueBackup>>(&content)
    else {
        return false;
    };

    records.iter().any(|record| {
        !record.data.trim().is_empty()
            && (record
                .key_path
                .to_lowercase()
                .contains("\\unifiedauth")
                || record
                    .key_path
                    .to_lowercase()
                    .contains("\\identity")
                || record
                    .key_path
                    .to_lowercase()
                    .contains("\\encryptionkey")
                || record.value_name.eq_ignore_ascii_case("URI_TOKEN"))
    })
}

fn has_local_session_state(local_state_dir: &Path) -> bool {
    if has_any_file(&local_state_dir.join("Account")) {
        return true;
    }
    if has_any_file(&local_state_dir.join("BrowserCaches")) {
        return true;
    }
    if has_any_file(&local_state_dir.join("Cache")) {
        return true;
    }

    let cached_data = local_state_dir.join("CachedData.db");
    cached_data.exists()
        && fs::metadata(&cached_data)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
}

fn has_any_file(dir: &Path) -> bool {
    if !dir.exists() {
        return false;
    }
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| e.file_type().is_file())
}

// ─── Save state to directory ──────────────────────────────────────────

fn save_state_to_directory(state_dir: &Path) -> bool {
    let _ = try_delete_dir(state_dir);

    let result = (|| -> Option<()> {
        fs::create_dir_all(state_dir).ok()?;

        let local_bnet_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Battle.net");

        let local_state_dir = state_dir.join("Local");

        // Copy local Battle.net state
        copy_directory_best_effort(
            &local_bnet_dir.join("Account"),
            &local_state_dir.join("Account"),
        );
        copy_directory_best_effort(
            &local_bnet_dir.join("BrowserCaches"),
            &local_state_dir.join("BrowserCaches"),
        );
        copy_directory_best_effort(
            &local_bnet_dir.join("Cache"),
            &local_state_dir.join("Cache"),
        );
        copy_file_best_effort(
            &local_bnet_dir.join("CachedData.db"),
            &local_state_dir.join("CachedData.db"),
        );

        // Save registry auth snapshot
        registry::save_auth_snapshot(
            &state_dir.join("Registry").join("registry-values.json"),
        );

        // Write metadata
        let metadata = SessionStateMetadata {
            captured_at_utc: chrono::Utc::now(),
        };
        let metadata_json = serde_json::to_string(&metadata).ok().unwrap_or_default();
        let _ = fs::write(metadata_path_from_state_dir(state_dir), metadata_json);

        Some(())
    })();

    if result.is_none() {
        let _ = try_delete_dir(state_dir);
        return false;
    }

    true
}

/// Atomically move candidate_dir → account_dir/BattleNetSessionState.
/// Uses backup rotation for rollback on failure.
fn commit_session_state(candidate_dir: &Path, account_dir: &Path) -> bool {
    if !candidate_dir.exists() {
        return false;
    }

    let state_dir = session_state_dir(account_dir);
    let backup_dir = session_state_backup_dir(account_dir);

    let result = (|| -> Option<()> {
        let _ = try_delete_dir(&backup_dir);

        if state_dir.exists() {
            fs::rename(&state_dir, &backup_dir).ok()?;
        }

        fs::rename(candidate_dir, &state_dir).ok()?;
        Some(())
    })();

    if result.is_some() {
        return true;
    }

    // Rollback: restore backup if main state was lost
    if !state_dir.exists() && backup_dir.exists() {
        let _ = fs::rename(&backup_dir, &state_dir);
    }

    false
}

// ─── File operations ──────────────────────────────────────────────────

fn restore_directory_snapshot(source: &Path, destination: &Path) {
    let _ = try_delete_dir(destination);
    if source.exists() {
        copy_directory_best_effort(source, destination);
    }
}

fn restore_file_snapshot(source: &Path, destination: &Path) {
    let _ = fs::remove_file(destination);
    copy_file_best_effort(source, destination);
}

fn copy_directory_best_effort(source: &Path, destination: &Path) {
    let Ok(entries) = fs::read_dir(source) else {
        return;
    };

    let _ = fs::create_dir_all(destination);

    for entry in entries.filter_map(|e| e.ok()) {
        let dest_path = destination.join(entry.file_name());
        let file_type = entry.file_type();
        if let Ok(ft) = file_type {
            if ft.is_dir() {
                copy_directory_best_effort(&entry.path(), &dest_path);
            } else if ft.is_file() {
                copy_file_best_effort(&entry.path(), &dest_path);
            }
        }
    }
}

fn copy_file_best_effort(source: &Path, destination: &Path) {
    if !source.exists() {
        return;
    }
    if let Some(parent) = destination.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::copy(source, destination);
}

fn try_delete_dir(path: &Path) -> Option<()> {
    if path.exists() {
        fs::remove_dir_all(path).ok()?;
    }
    Some(())
}
