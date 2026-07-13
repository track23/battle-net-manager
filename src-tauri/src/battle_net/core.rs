use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::config;
use super::models::{
    AccountInfo, GroupInfo, LegacyMigrationDataDirectory, LegacyMigrationState,
    SaveAccountResult, SwitchAccountResult,
};
use super::process;
use super::region::{
    infer_region_from_config, is_cross_region_switch, is_tagged_region,
    normalize_region, UNSET_REGION,
};
use super::session_state;

pub const DEFAULT_GROUP_ID: &str = "default";
const DEFAULT_GROUP_NAME: &str = "默认分组";

pub struct BattleNetCore {
    data_dir: PathBuf,
    accounts_json_path: PathBuf,
    groups_json_path: PathBuf,
    config_file_path: PathBuf,
    migration_state_path: PathBuf,
}

impl BattleNetCore {
    pub fn new() -> Self {
        // Battle.net stores its config under AppData\Roaming
        let app_data_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Battle.net");

        // Use AppData\Local\BattleNetManager\Data
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("BattleNetManager")
            .join("Data");

        let config_file_path = app_data_path.join("Battle.net.config");
        let accounts_json_path = data_dir.join("accounts.json");
        let groups_json_path = data_dir.join("groups.json");
        let migration_state_path = data_dir.join("migration-state.json");

        let core = Self {
            data_dir,
            accounts_json_path,
            groups_json_path,
            config_file_path,
            migration_state_path,
        };

        let _ = fs::create_dir_all(&core.data_dir);

        // Migrate legacy data if needed
        core.migrate_legacy_data_if_needed();

        core
    }

    // ─── Account CRUD ───────────────────────────────────────────────

    pub fn get_accounts(&self) -> Vec<AccountInfo> {
        if !self.accounts_json_path.exists() {
            return Vec::new();
        }
        match fs::read_to_string(&self.accounts_json_path) {
            Ok(json) => {
                let mut accounts: Vec<AccountInfo> =
                    serde_json::from_str(&json).unwrap_or_default();
                let groups = self.read_groups();
                if self.normalize_accounts(&mut accounts, &groups) {
                    self.save_accounts_inner(&accounts);
                }
                accounts
            }
            Err(_) => Vec::new(),
        }
    }

    fn save_accounts_inner(&self, accounts: &[AccountInfo]) -> bool {
        serde_json::to_string(accounts)
            .and_then(|json| fs::write(&self.accounts_json_path, json).map_err(serde_json::Error::io))
            .is_ok()
    }

    fn save_accounts(&self, accounts: &[AccountInfo]) -> bool {
        let mut accounts = accounts.to_vec();
        let groups = self.read_groups();
        self.normalize_accounts(&mut accounts, &groups);
        self.save_accounts_inner(&accounts)
    }

    fn save_accounts_with_groups(&self, accounts: &[AccountInfo], groups: &[GroupInfo]) -> bool {
        let mut accounts = accounts.to_vec();
        self.normalize_accounts(&mut accounts, groups);
        self.save_accounts_inner(&accounts)
    }

    fn get_accounts_with_groups(&self, groups: &[GroupInfo]) -> Vec<AccountInfo> {
        if !self.accounts_json_path.exists() {
            return Vec::new();
        }
        match fs::read_to_string(&self.accounts_json_path) {
            Ok(json) => {
                let mut accounts: Vec<AccountInfo> =
                    serde_json::from_str(&json).unwrap_or_default();
                self.normalize_accounts(&mut accounts, groups);
                accounts
            }
            Err(_) => Vec::new(),
        }
    }

    pub fn get_groups(&self) -> Vec<GroupInfo> {
        self.read_groups()
    }

    pub fn create_group(&self, name: &str) -> Option<GroupInfo> {
        let name = normalize_group_name(name);
        if name.is_empty() {
            return None;
        }

        let mut groups = self.read_groups();
        if let Some(existing) = groups
            .iter()
            .find(|g| g.name.eq_ignore_ascii_case(&name))
        {
            return Some(existing.clone());
        }

        let group = GroupInfo {
            id: uuid::Uuid::new_v4().to_string().replace('-', ""),
            name,
            created_at: chrono::Local::now(),
        };
        groups.push(group.clone());
        self.save_groups(&groups);
        Some(group)
    }

    pub fn rename_group(&self, id: &str, name: &str) -> bool {
        if id == DEFAULT_GROUP_ID {
            return false;
        }

        let name = normalize_group_name(name);
        if name.is_empty() {
            return false;
        }

        let mut groups = self.read_groups();
        if groups
            .iter()
            .any(|g| g.id != id && g.name.eq_ignore_ascii_case(&name))
        {
            return false;
        }

        if let Some(group) = groups.iter_mut().find(|g| g.id == id) {
            group.name = name;
            self.save_groups(&groups);
            true
        } else {
            false
        }
    }

    pub fn delete_group(&self, id: &str) -> bool {
        if id.is_empty() || id == DEFAULT_GROUP_ID {
            return false;
        }

        let mut groups = self.read_groups();
        let before_len = groups.len();
        groups.retain(|g| g.id != id);
        if groups.len() == before_len {
            return false;
        }
        self.save_groups(&groups);

        // Move accounts in deleted group to default
        if let Ok(json) = fs::read_to_string(&self.accounts_json_path) {
            let mut accounts: Vec<AccountInfo> =
                serde_json::from_str(&json).unwrap_or_default();
            let mut changed = false;
            for account in accounts.iter_mut().filter(|a| a.group_id == id) {
                account.group_id = String::new();
                changed = true;
            }
            if changed {
                self.save_accounts_with_groups(&accounts, &groups);
            }
        }

        true
    }

    pub fn move_account_to_group(&self, account_id: &str, group_id: &str) -> bool {
        let group_id = self.ensure_valid_group_id(group_id);
        let groups = self.read_groups();
        let mut accounts = self.get_accounts_with_groups(&groups);
        if let Some(account) = accounts.iter_mut().find(|a| a.id == account_id) {
            account.group_id = group_id;
            self.save_accounts_with_groups(&accounts, &groups);
            true
        } else {
            false
        }
    }

    pub fn update_account_info(
        &self,
        account_id: &str,
        remark: &str,
        battle_tag: &str,
        tags: &[String],
    ) -> bool {
        let groups = self.read_groups();
        let mut accounts = self.get_accounts_with_groups(&groups);
        if let Some(account) = accounts.iter_mut().find(|a| a.id == account_id) {
            account.remark = remark.trim().to_string();
            account.username = battle_tag.trim().to_string();
            account.tags = normalize_tags(tags);
            self.save_accounts_with_groups(&accounts, &groups);
            true
        } else {
            false
        }
    }

    /// Save the current Battle.net config as a new account with session state capture.
    pub async fn save_current_account_to_group(
        &self,
        remark: &str,
        battle_tag: &str,
        group_id: &str,
        tags: &[String],
    ) -> SaveAccountResult {
        if !self.config_file_path.exists() {
            return SaveAccountResult {
                success: false,
                session_state_saved: false,
                error: "missing_config".to_string(),
            };
        }

        let group_id = self.ensure_valid_group_id(group_id);
        let groups = self.read_groups();
        let mut accounts = self.get_accounts_with_groups(&groups);

        let region = infer_region_from_config(&self.config_file_path);
        let normalized_region = normalize_region(&region);

        let new_account = AccountInfo::new(
            remark.to_string(),
            battle_tag.to_string(),
            group_id,
            normalize_tags(tags),
            normalized_region.clone(),
        );

        let account_dir = self.data_dir.join(&new_account.id);
        if fs::create_dir_all(&account_dir).is_err() {
            return SaveAccountResult {
                success: false,
                session_state_saved: false,
                error: "create_dir_failed".to_string(),
            };
        }

        // Backup config to fallback
        let fallback_config = account_dir.join("Battle.net.config.fallback");
        let _ = fs::remove_file(&fallback_config);
        if fs::copy(&self.config_file_path, &fallback_config).is_err() {
            return SaveAccountResult {
                success: false,
                session_state_saved: false,
                error: "copy_config_failed".to_string(),
            };
        }

        let was_bnet_running = process::is_battle_net_client_running();

        // Gracefully close Battle.net for session state capture
        let prepare_succeeded = if was_bnet_running || process::is_battle_net_client_running() {
            self.prepare_session_capture().await
        } else {
            true // No need to close if not running
        };

        let should_relaunch =
            was_bnet_running && (prepare_succeeded || !process::is_battle_net_client_running());

        // Try to save session state
        let session_state_saved = if prepare_succeeded {
            // Refresh fallback config (may have changed during shutdown)
            if self.config_file_path.exists() {
                let _ = fs::copy(&self.config_file_path, &fallback_config);
            }

            session_state::save_session_snapshot(
                &self.config_file_path,
                &account_dir,
                &normalized_region,
                false,
            )
        } else {
            false
        };

        // If session state wasn't saved, use fallback config
        if !session_state_saved {
            let saved_config = account_dir.join("Battle.net.config");
            if fs::copy(&fallback_config, &saved_config).is_err() {
                let _ = fs::remove_dir_all(&account_dir);
                return SaveAccountResult {
                    success: false,
                    session_state_saved: false,
                    error: "copy_config_failed".to_string(),
                };
            }
        }

        accounts.push(new_account);
        self.save_accounts_with_groups(&accounts, &groups);
        let _ = fs::remove_file(&fallback_config);

        // Relaunch if needed
        if should_relaunch {
            process::launch_battle_net();
        }

        SaveAccountResult {
            success: true,
            session_state_saved,
            error: if session_state_saved {
                String::new()
            } else if prepare_succeeded {
                "missing_session_snapshot".to_string()
            } else {
                "client_exit_timeout".to_string()
            },
        }
    }

    /// Switch to a saved account with region awareness and session state management.
    pub async fn switch_account(&self, id: &str) -> SwitchAccountResult {
        let account_dir = self.data_dir.join(id);
        let saved_config = account_dir.join("Battle.net.config");

        if !saved_config.exists() {
            return SwitchAccountResult {
                success: false,
                requires_manual_launch: false,
                error: "missing_config".to_string(),
            };
        }

        let groups = self.read_groups();
        let accounts = self.get_accounts_with_groups(&groups);
        let target_account = accounts.iter().find(|a| a.id == id).cloned();
        let Some(target_account) = target_account else {
            return SwitchAccountResult {
                success: false,
                requires_manual_launch: false,
                error: "missing_account".to_string(),
            };
        };

        let target_region = normalize_region(&target_account.region);

        let is_untagged = !is_tagged_region(&target_region);

        // Tagged region: determine active account and region for advanced handling
        let (active_account, active_region, _is_cross_region, has_state, requires_clean_login,
             should_use_full_reset, should_refresh_leaving) = if is_untagged {
            // Untagged region: skip all session state / region logic
            (None, String::new(), false, false, false, false, false)
        } else {
            let active_account = accounts
                .iter()
                .max_by_key(|a| a.last_used)
                .cloned();

            let active_region = self.get_active_bnet_region(active_account.as_ref());
            let is_cross_region = is_cross_region_switch(&active_region, &target_region);
            let has_state = session_state::has_session_state(&account_dir);
            let requires_clean_login = is_cross_region && !has_state;
            let should_use_full_reset = has_state || requires_clean_login;
            let should_refresh_leaving =
                self.should_refresh_leaving_account(active_account.as_ref(), &target_account, &active_region);

            (active_account, active_region, is_cross_region, has_state, requires_clean_login,
             should_use_full_reset, should_refresh_leaving)
        };

        // Phase 1: Close Battle.net if needed
        if should_use_full_reset || should_refresh_leaving {
            // Gracefully close first (both paths need the client stopped).
            let capture_ok = self.prepare_session_capture().await;

            if should_refresh_leaving {
                if let Some(ref active) = active_account {
                    self.try_refresh_leaving_account(active, &active_region);
                }
            }

            if should_use_full_reset {
                if !capture_ok {
                    return SwitchAccountResult {
                        success: false,
                        requires_manual_launch: false,
                        error: "client_exit_timeout".to_string(),
                    };
                }
                self.clear_region_caches();
                super::registry::clear_auth_state();
            }
        } else {
            // Fast path: just kill and wait (no session state management).
            self.switch_fast_path().await;
        }

        // Phase 2: Restore config
        let _ = fs::remove_file(&self.config_file_path);

        if has_state {
            session_state::restore_session_state(&account_dir);
            let _ = fs::copy(&saved_config, &self.config_file_path);
        } else if requires_clean_login {
            config::write_cross_region_login_config(
                &saved_config,
                &self.config_file_path,
                &target_region,
            );
        } else {
            let _ = fs::copy(&saved_config, &self.config_file_path);
        }

        // Update last_used
        let mut accounts = self.get_accounts_with_groups(&groups);
        if let Some(acc) = accounts.iter_mut().find(|a| a.id == id) {
            acc.last_used = chrono::Local::now();
        }
        self.save_accounts_with_groups(&accounts, &groups);

        // Launch
        process::launch_battle_net();

        SwitchAccountResult {
            success: true,
            requires_manual_launch: false,
            error: if requires_clean_login {
                "missing_session_state".to_string()
            } else {
                String::new()
            },
        }
    }

    /// Synchronous wrapper for backward compatibility (commands.rs).
    pub fn switch_account_blocking(&self, id: &str) -> bool {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.switch_account(id)).success
    }

    /// Refresh session state for an existing account.
    pub async fn refresh_account_session_state(&self, id: &str) -> bool {
        if !self.config_file_path.exists() {
            return false;
        }

        let groups = self.read_groups();
        let accounts = self.get_accounts_with_groups(&groups);
        let Some(account) = accounts.iter().find(|a| a.id == id).cloned() else {
            return false;
        };

        let was_running = process::is_battle_net_client_running();
        let prepare_succeeded = self.prepare_session_capture().await;
        let should_relaunch =
            was_running && (prepare_succeeded || !process::is_battle_net_client_running());

        if !prepare_succeeded {
            return false;
        }

        if !self.config_file_path.exists() {
            return false;
        }

        let account_dir = self.data_dir.join(id);
        let _ = fs::create_dir_all(&account_dir);

        let saved = session_state::save_session_snapshot(
            &self.config_file_path,
            &account_dir,
            &account.region,
            false,
        );

        if !saved {
            return false;
        }

        let mut accounts = self.get_accounts_with_groups(&groups);
        if let Some(acc) = accounts.iter_mut().find(|a| a.id == id) {
            acc.last_used = chrono::Local::now();
        }
        self.save_accounts_with_groups(&accounts, &groups);

        if should_relaunch {
            process::launch_battle_net();
        }

        true
    }

    pub fn delete_account(&self, id: &str) -> bool {
        let mut accounts = self.get_accounts();
        accounts.retain(|a| a.id != id);
        let saved = self.save_accounts(&accounts);

        let account_dir = self.data_dir.join(id);
        if account_dir.exists() {
            let _ = fs::remove_dir_all(&account_dir);
        }

        saved
    }

    /// Add a new account with region awareness (defaults to CN).
    pub async fn add_new_account(&self) {
        let region = super::region::REGION_CN;

        // Gracefully close Battle.net
        self.prepare_new_account_switch().await;

        // Write minimal login config for the region
        config::write_new_account_login_config(&self.config_file_path, region);

        // Launch Battle.net
        process::launch_battle_net();
    }

    /// Compare the current live Battle.net.config's `SavedAccountNames` field
    /// with each saved account's config to find the active account.
    pub fn get_active_account_id(&self) -> Option<String> {
        let current_name = self.read_saved_account_names(&self.config_file_path)?;

        let accounts = self.get_accounts();
        for account in accounts {
            let saved_config_path = self.data_dir.join(&account.id).join("Battle.net.config");
            if let Some(saved_name) = self.read_saved_account_names(&saved_config_path) {
                if saved_name == current_name {
                    return Some(account.id);
                }
            }
        }

        None
    }

    fn read_saved_account_names(&self, path: &std::path::Path) -> Option<String> {
        let content = fs::read_to_string(path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("Client")
            .and_then(|c| c.get("SavedAccountNames"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    // ─── Async helpers ──────────────────────────────────────────────

    /// Gracefully close Battle.net and wait for it to exit.
    async fn prepare_session_capture(&self) -> bool {
        process::request_battle_net_close();

        let exited =
            process::ensure_cleanly_exited(Duration::from_secs(3), Duration::from_secs(30)).await;

        if !exited {
            return false;
        }

        // Kill remaining Agent processes
        let agent_pids = process::get_battle_net_processes();
        process::kill_processes(&agent_pids);
        tokio::time::sleep(Duration::from_millis(1000)).await;

        true
    }

    /// Gracefully close Battle.net + clear caches for new account creation.
    async fn prepare_new_account_switch(&self) {
        process::request_battle_net_close();

        let exited =
            process::ensure_cleanly_exited(Duration::from_secs(3), Duration::from_secs(30)).await;

        if !exited {
            // Force kill everything
            let pids = process::get_battle_net_processes();
            process::kill_processes(&pids);
        }

        // Kill remaining agent processes
        let all_pids = process::get_battle_net_processes();
        process::kill_processes(&all_pids);
        tokio::time::sleep(Duration::from_millis(1000)).await;

        self.clear_region_caches();
        super::registry::clear_auth_state();
    }

    /// Fast path: kill and wait (no session state management, no launch).
    /// The caller is responsible for copying config and launching Battle.net.
    async fn switch_fast_path(&self) {
        process::kill_battle_net().await;
    }

    /// Get the currently active Battle.net region by checking the live config.
    fn get_active_bnet_region(&self, tracked_active: Option<&AccountInfo>) -> String {
        let config_region = infer_region_from_config(&self.config_file_path);
        if is_tagged_region(&config_region) {
            return config_region;
        }

        tracked_active
            .map(|a| normalize_region(&a.region))
            .unwrap_or_else(|| UNSET_REGION.to_string())
    }

    /// Determine if we should refresh the leaving account's session state.
    fn should_refresh_leaving_account(
        &self,
        active_account: Option<&AccountInfo>,
        target_account: &AccountInfo,
        active_region: &str,
    ) -> bool {
        let Some(active) = active_account else {
            return false;
        };

        if active.id.eq_ignore_ascii_case(&target_account.id) {
            return false;
        }

        if !self.config_file_path.exists() {
            return false;
        }

        let active_region_norm = normalize_region(&active.region);
        if !is_tagged_region(&active_region_norm) {
            return false;
        }

        let current_config_region = infer_region_from_config(&self.config_file_path);
        if !is_tagged_region(&current_config_region) {
            return false;
        }

        if normalize_region(active_region) != current_config_region {
            return false;
        }

        active_region_norm == current_config_region
    }

    /// Try to refresh the leaving account's session state before switching.
    fn try_refresh_leaving_account(&self, active_account: &AccountInfo, active_region: &str) {
        let active_region_norm = normalize_region(&active_account.region);
        if !is_tagged_region(&active_region_norm)
            || active_region_norm != normalize_region(active_region)
        {
            return;
        }

        let account_dir = self.data_dir.join(&active_account.id);
        session_state::save_session_snapshot(
            &self.config_file_path,
            &account_dir,
            &active_region_norm,
            true,
        );
    }

    /// Clear Battle.net local state caches (Account, BrowserCaches, Cache, CachedData.db).
    /// Backs up before deleting.
    fn clear_region_caches(&self) {
        let local_bnet_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Battle.net");

        let backup_root = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("BattleNetManager")
            .join("BattleNetLocalStateBackups")
            .join(chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());

        backup_and_delete_dir(
            &local_bnet_dir.join("Account"),
            &backup_root.join("Account"),
        );
        let _ = try_delete_dir(&local_bnet_dir.join("BrowserCaches"));
        let _ = try_delete_dir(&local_bnet_dir.join("Cache"));
        let _ = fs::remove_file(local_bnet_dir.join("CachedData.db"));

        super::registry::prune_old_backups(
            &dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("BattleNetManager")
                .join("BattleNetLocalStateBackups"),
        );
    }

    // ─── Group helpers ──────────────────────────────────────────────

    fn read_groups(&self) -> Vec<GroupInfo> {
        match fs::read_to_string(&self.groups_json_path) {
            Ok(json) => {
                let mut groups: Vec<GroupInfo> =
                    serde_json::from_str(&json).unwrap_or_default();
                normalize_groups(&mut groups);
                groups
            }
            Err(_) => {
                vec![GroupInfo {
                    id: DEFAULT_GROUP_ID.to_string(),
                    name: DEFAULT_GROUP_NAME.to_string(),
                    created_at: chrono::Local::now(),
                }]
            }
        }
    }

    fn save_groups(&self, groups: &[GroupInfo]) -> bool {
        let mut groups = groups.to_vec();
        normalize_groups(&mut groups);
        serde_json::to_string(&groups)
            .and_then(|json| fs::write(&self.groups_json_path, json).map_err(serde_json::Error::io))
            .is_ok()
    }

    fn ensure_valid_group_id(&self, group_id: &str) -> String {
        if group_id.is_empty() {
            return String::new();
        }
        let groups = self.read_groups();
        if groups.iter().any(|g| g.id == group_id) {
            group_id.to_string()
        } else {
            String::new()
        }
    }

    fn normalize_accounts(&self, accounts: &mut [AccountInfo], groups: &[GroupInfo]) -> bool {
        let valid_ids: HashSet<&str> = groups.iter().map(|g| g.id.as_str()).collect();
        let mut changed = false;

        for account in accounts.iter_mut() {
            // Fix invalid group IDs
            if !account.group_id.is_empty() && !valid_ids.contains(account.group_id.as_str()) {
                account.group_id = String::new();
                changed = true;
            }

            // Normalize region (default to "cn" for old data)
            let normalized_region = normalize_region(&account.region);
            let effective_region = if normalized_region.is_empty() {
                super::region::REGION_CN.to_string()
            } else {
                normalized_region
            };
            if account.region != effective_region {
                account.region = effective_region;
                changed = true;
            }
        }

        changed
    }

    // ─── Legacy data migration ──────────────────────────────────────

    fn migrate_legacy_data_if_needed(&self) {
        let _ = fs::create_dir_all(&self.data_dir);

        let legacy_dirs = self.get_legacy_data_directories();
        if legacy_dirs.is_empty() {
            return;
        }

        let mut state = self.read_migration_state();
        let mut state_changed = false;

        for legacy_dir in legacy_dirs {
            // Skip if already migrated
            if self.has_migrated_data_directory(&state, &legacy_dir) {
                continue;
            }

            // Skip if data was already imported (fast path: avoid redundant I/O)
            if self.looks_like_data_directory_already_imported(&legacy_dir) {
                self.remember_migrated_data_directory(&mut state, &legacy_dir);
                state_changed = true;
                continue;
            }

            self.merge_legacy_data(&legacy_dir);
            self.remember_migrated_data_directory(&mut state, &legacy_dir);
            state_changed = true;
        }

        if state_changed {
            self.save_migration_state(&state);
        }
    }

    fn get_legacy_data_directories(&self) -> Vec<PathBuf> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let mut candidates = vec![
            exe_dir.join("Data"),
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Programs")
                .join("BattleNetManager")
                .join("Data"),
        ];

        // Check parent directory
        if let Some(parent) = exe_dir.parent() {
            candidates.push(parent.join("Data"));
        }

        // Check registry install location
        if let Some(install_path) = super::registry::get_battle_net_install_path() {
            candidates.push(PathBuf::from(&install_path).join("Data"));
        }

        let current_data = dunce::canonicalize(&self.data_dir)
            .unwrap_or_else(|_| self.data_dir.clone());

        candidates
            .into_iter()
            .filter(|p| p.exists())
            .filter(|p| {
                let canon = dunce::canonicalize(p).unwrap_or_else(|_| p.clone());
                canon != current_data
            })
            .collect()
    }

    fn merge_legacy_data(&self, legacy_dir: &PathBuf) {
        let mut accounts = self.read_accounts_from_file(&self.accounts_json_path);
        let mut account_ids: HashSet<String> =
            accounts.iter().map(|a| a.id.to_lowercase()).collect();
        let mut groups = self.read_groups();
        let mut group_ids: HashSet<String> = groups.iter().map(|g| g.id.to_lowercase()).collect();
        let mut groups_changed = false;
        let mut accounts_changed = false;

        // Merge groups
        let legacy_groups =
            self.read_groups_from_file(&legacy_dir.join("groups.json"));
        for legacy_group in legacy_groups {
            if legacy_group.id.is_empty() {
                continue;
            }
            if !group_ids.contains(&legacy_group.id.to_lowercase()) {
                group_ids.insert(legacy_group.id.to_lowercase());
                groups.push(legacy_group);
                groups_changed = true;
            }
        }

        if groups_changed {
            self.save_groups(&groups);
        }

        // Merge accounts
        let legacy_accounts =
            self.read_accounts_from_file(&legacy_dir.join("accounts.json"));
        for legacy_account in legacy_accounts {
            if legacy_account.id.is_empty() {
                continue;
            }

            if legacy_account.group_id.is_empty()
                || !group_ids.contains(&legacy_account.group_id.to_lowercase())
            {
                // Will be normalized later
            }

            if !account_ids.contains(&legacy_account.id.to_lowercase()) {
                account_ids.insert(legacy_account.id.to_lowercase());
                accounts.push(legacy_account);
                accounts_changed = true;

                // Copy account directory
                self.copy_account_directory(legacy_dir, &accounts.last().unwrap().id, false);
            }
        }

        // Recover orphaned account directories
        let mut recovered_index = 1;
        if let Ok(entries) = fs::read_dir(legacy_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else {
                    continue;
                };
                if account_ids.contains(&name.to_lowercase()) {
                    self.copy_account_directory(legacy_dir, &name, true);
                    continue;
                }
                if !path.join("Battle.net.config").exists() {
                    continue;
                }

                let last_used = fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .ok()
                    .map(chrono::DateTime::<chrono::Local>::from)
                    .unwrap_or_else(chrono::Local::now);

                accounts.push(AccountInfo {
                    id: name.clone(),
                    remark: format!("恢复账号 {}", recovered_index),
                    username: String::new(),
                    last_used,
                    group_id: String::new(),
                    tags: Vec::new(),
                    region: super::region::REGION_CN.to_string(),
                });
                account_ids.insert(name.to_lowercase());
                accounts_changed = true;
                recovered_index += 1;

                self.copy_account_directory(legacy_dir, &name, true);
            }
        }

        if accounts_changed {
            self.save_accounts(&accounts);
        }
    }

    fn copy_account_directory(&self, legacy_dir: &PathBuf, account_id: &str, overwrite: bool) {
        let source = legacy_dir.join(account_id);
        if !source.exists() {
            return;
        }
        let dest = self.data_dir.join(account_id);
        if overwrite && dest.exists() {
            let _ = fs::remove_dir_all(&dest);
        }
        copy_directory_best_effort(&source, &dest);
    }

    fn read_accounts_from_file(&self, path: &std::path::Path) -> Vec<AccountInfo> {
        fs::read_to_string(path)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    fn read_groups_from_file(&self, path: &std::path::Path) -> Vec<GroupInfo> {
        fs::read_to_string(path)
            .ok()
            .and_then(|json| {
                let mut groups: Vec<GroupInfo> = serde_json::from_str(&json).ok()?;
                normalize_groups(&mut groups);
                Some(groups)
            })
            .unwrap_or_default()
    }

    // ─── Migration state persistence ──────────────────────────────────

    fn read_migration_state(&self) -> LegacyMigrationState {
        if !self.migration_state_path.exists() {
            return LegacyMigrationState {
                data_directories: Vec::new(),
            };
        }
        fs::read_to_string(&self.migration_state_path)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_else(|| LegacyMigrationState {
                data_directories: Vec::new(),
            })
    }

    fn save_migration_state(&self, state: &LegacyMigrationState) {
        let _ = serde_json::to_string(state)
            .and_then(|json| {
                fs::write(&self.migration_state_path, json).map_err(serde_json::Error::io)
            });
    }

    fn has_migrated_data_directory(
        &self,
        state: &LegacyMigrationState,
        legacy_dir: &std::path::Path,
    ) -> bool {
        let normalized = normalize_path(legacy_dir);
        state.data_directories.iter().any(|item| {
            item.path.eq_ignore_ascii_case(&normalized)
        })
    }

    fn remember_migrated_data_directory(
        &self,
        state: &mut LegacyMigrationState,
        legacy_dir: &std::path::Path,
    ) {
        let normalized = normalize_path(legacy_dir);
        if let Some(existing) = state
            .data_directories
            .iter_mut()
            .find(|item| item.path.eq_ignore_ascii_case(&normalized))
        {
            existing.imported_at_utc = chrono::Utc::now();
        } else {
            state.data_directories.push(LegacyMigrationDataDirectory {
                path: normalized,
                prefer_legacy_data: false,
                imported_at_utc: chrono::Utc::now(),
            });
        }
    }

    fn looks_like_data_directory_already_imported(&self, legacy_dir: &std::path::Path) -> bool {
        let legacy_accounts_path = legacy_dir.join("accounts.json");
        if !legacy_accounts_path.exists() || !self.accounts_json_path.exists() {
            return false;
        }

        let legacy_account_ids = self.get_importable_account_ids(legacy_dir);
        if legacy_account_ids.is_empty() {
            return false;
        }

        let current_account_ids: HashSet<String> = self
            .read_accounts_from_file(&self.accounts_json_path)
            .iter()
            .map(|a| a.id.to_lowercase())
            .collect();

        if !legacy_account_ids
            .iter()
            .all(|id| current_account_ids.contains(&id.to_lowercase()))
        {
            return false;
        }

        // Check that current data is at least as new as legacy data
        let current_mtime = fs::metadata(&self.accounts_json_path)
            .and_then(|m| m.modified())
            .ok();
        let legacy_mtime = fs::metadata(&legacy_accounts_path)
            .and_then(|m| m.modified())
            .ok();

        if let (Some(current), Some(legacy)) = (current_mtime, legacy_mtime) {
            if current < legacy {
                return false;
            }
        } else {
            return false;
        }

        // Check that all account directories exist in current data
        legacy_account_ids.iter().all(|id| {
            self.data_dir.join(id).exists()
                && self.data_dir.join(id).join("Battle.net.config").exists()
        })
    }

    fn get_importable_account_ids(&self, legacy_dir: &std::path::Path) -> HashSet<String> {
        let mut ids: HashSet<String> = self
            .read_accounts_from_file(&legacy_dir.join("accounts.json"))
            .iter()
            .filter(|a| !a.id.is_empty())
            .map(|a| a.id.to_lowercase())
            .collect();

        if let Ok(entries) = fs::read_dir(legacy_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) {
                    if !name.is_empty() && path.join("Battle.net.config").exists() {
                        ids.insert(name.to_lowercase());
                    }
                }
            }
        }

        ids
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────

fn normalize_group_name(name: &str) -> String {
    name.trim().to_string()
}

fn normalize_tags(tags: &[String]) -> Vec<String> {
    tags.iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

fn normalize_groups(groups: &mut Vec<GroupInfo>) {
    groups.retain(|g| !g.id.is_empty());

    // Ensure default group exists and is first
    let default_pos = groups.iter().position(|g| g.id == DEFAULT_GROUP_ID);
    if let Some(pos) = default_pos {
        let default_group = groups.remove(pos);
        groups.insert(0, default_group);
    } else {
        groups.insert(
            0,
            GroupInfo {
                id: DEFAULT_GROUP_ID.to_string(),
                name: DEFAULT_GROUP_NAME.to_string(),
                created_at: chrono::Local::now(),
            },
        );
    }

    // Normalize names
    for group in groups.iter_mut() {
        group.name = normalize_group_name(&group.name);
        if group.name.is_empty() {
            group.name = "未命名分组".to_string();
        }
    }
}

fn backup_and_delete_dir(source: &std::path::Path, backup: &std::path::Path) {
    if !source.exists() {
        return;
    }

    if let Some(parent) = backup.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = copy_directory_best_effort(source, backup);
    let _ = fs::remove_dir_all(source);
}

fn copy_directory_best_effort(source: &std::path::Path, dest: &std::path::Path) -> Option<()> {
    if !source.exists() {
        return Some(());
    }
    fs::create_dir_all(dest).ok()?;

    for entry in fs::read_dir(source).ok()?.filter_map(|e| e.ok()) {
        let dest_path = dest.join(entry.file_name());
        if let Ok(ft) = entry.file_type() {
            if ft.is_dir() {
                let _ = copy_directory_best_effort(&entry.path(), &dest_path);
            } else if ft.is_file() {
                let _ = fs::copy(entry.path(), dest_path);
            }
        }
    }

    Some(())
}

fn try_delete_dir(path: &std::path::Path) -> Option<()> {
    if path.exists() {
        fs::remove_dir_all(path).ok()?;
    }
    Some(())
}

fn normalize_path(path: &std::path::Path) -> String {
    dunce::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}
