use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use super::models::{AccountInfo, GroupInfo};
use super::process;

pub const DEFAULT_GROUP_ID: &str = "default";

pub struct BattleNetCore {
    #[allow(dead_code)]
    app_data_path: PathBuf,
    data_dir: PathBuf,
    accounts_json_path: PathBuf,
    groups_json_path: PathBuf,
    config_file_path: PathBuf,
}

impl BattleNetCore {
    pub fn new() -> Self {
        // Battle.net stores its config under AppData\Roaming
        let app_data_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Battle.net");

        // Use AppData\Local\BattleNetManager\Data so the directory is always
        // writable (exe_dir may be read-only when installed to Program Files).
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("BattleNetManager")
            .join("Data");

        let config_file_path = app_data_path.join("Battle.net.config");
        let accounts_json_path = data_dir.join("accounts.json");
        let groups_json_path = data_dir.join("groups.json");

        let core = Self {
            app_data_path,
            data_dir,
            accounts_json_path,
            groups_json_path,
            config_file_path,
        };

        let _ = fs::create_dir_all(&core.data_dir);

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

    /// save_accounts variant that reuses an already-loaded groups list.
    fn save_accounts_with_groups(&self, accounts: &[AccountInfo], groups: &[GroupInfo]) -> bool {
        let mut accounts = accounts.to_vec();
        self.normalize_accounts(&mut accounts, groups);
        self.save_accounts_inner(&accounts)
    }

    /// Load accounts from disk, normalized against already-loaded groups.
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
        let accounts_json = fs::read_to_string(&self.accounts_json_path);
        if let Ok(json) = accounts_json {
            let mut accounts: Vec<AccountInfo> =
                serde_json::from_str(&json).unwrap_or_default();
            let mut changed = false;
            for account in accounts.iter_mut().filter(|a| a.group_id == id) {
                account.group_id = String::new();
                changed = true;
            }
            if changed {
                // groups already loaded above, reuse it
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

    pub fn update_account_info(&self, account_id: &str, remark: &str, battle_tag: &str, tags: &[String]) -> bool {
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

    pub fn save_current_account_to_group(
        &self,
        remark: &str,
        battle_tag: &str,
        group_id: &str,
        tags: &[String],
    ) -> bool {
        if !self.config_file_path.exists() {
            return false;
        }

        let group_id = self.ensure_valid_group_id(group_id);
        let groups = self.read_groups();
        let mut accounts = self.get_accounts_with_groups(&groups);
        let new_account = AccountInfo::new(
            remark.to_string(),
            battle_tag.to_string(),
            group_id,
            normalize_tags(tags),
        );

        let account_dir = self.data_dir.join(&new_account.id);
        if fs::create_dir_all(&account_dir).is_err() {
            return false;
        }
        if fs::copy(
            &self.config_file_path,
            account_dir.join("Battle.net.config"),
        )
        .is_err()
        {
            return false;
        }

        accounts.push(new_account);
        self.save_accounts_with_groups(&accounts, &groups);
        true
    }

    pub fn switch_account(&self, id: &str) -> bool {
        let saved_config = self.data_dir.join(id).join("Battle.net.config");
        if !saved_config.exists() {
            return false;
        }

        process::kill_battle_net();

        // fs::copy overwrites the destination on Windows, no need to delete first
        if fs::copy(&saved_config, &self.config_file_path).is_err() {
            return false;
        }

        // Update LastUsed
        let groups = self.read_groups();
        let mut accounts = self.get_accounts_with_groups(&groups);
        if let Some(acc) = accounts.iter_mut().find(|a| a.id == id) {
            acc.last_used = chrono::Local::now();
            self.save_accounts_with_groups(&accounts, &groups);
        }

        process::launch_battle_net();
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

    pub fn add_new_account(&self) {
        process::kill_battle_net();

        if self.config_file_path.exists() {
            let _ = fs::remove_file(&self.config_file_path);
        }

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

    // ─── Group helpers ──────────────────────────────────────────────

    fn read_groups(&self) -> Vec<GroupInfo> {
        match fs::read_to_string(&self.groups_json_path) {
            Ok(json) => {
                let mut groups: Vec<GroupInfo> =
                    serde_json::from_str(&json).unwrap_or_default();
                normalize_groups(&mut groups);
                groups
            }
            Err(_) => Vec::new(),
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
            if !account.group_id.is_empty() && !valid_ids.contains(account.group_id.as_str()) {
                account.group_id = String::new();
                changed = true;
            }
        }

        changed
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

    // Normalize names
    for group in groups.iter_mut() {
        group.name = normalize_group_name(&group.name);
        if group.name.is_empty() {
            group.name = "未命名分组".to_string();
        }
    }
}

