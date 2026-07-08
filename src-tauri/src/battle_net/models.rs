use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use super::region::REGION_CN;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Remark")]
    pub remark: String,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "LastUsed")]
    pub last_used: DateTime<Local>,
    #[serde(rename = "GroupId")]
    pub group_id: String,
    #[serde(rename = "Tags", default)]
    pub tags: Vec<String>,
    #[serde(rename = "Region", default = "default_region")]
    pub region: String,
}

fn default_region() -> String {
    REGION_CN.to_string()
}

impl AccountInfo {
    pub fn new(
        remark: String,
        username: String,
        group_id: String,
        tags: Vec<String>,
        region: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string().replace('-', ""),
            remark,
            username,
            last_used: Local::now(),
            group_id,
            tags,
            region,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "CreatedAt")]
    pub created_at: DateTime<Local>,
}

// ─── Result types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SaveAccountResult {
    pub success: bool,
    #[serde(rename = "sessionStateSaved")]
    pub session_state_saved: bool,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwitchAccountResult {
    pub success: bool,
    #[serde(rename = "requiresManualLaunch")]
    pub requires_manual_launch: bool,
    pub error: String,
}

// ─── Internal helper types ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryValueBackup {
    #[serde(rename = "KeyPath")]
    pub key_path: String,
    #[serde(rename = "ValueName")]
    pub value_name: String,
    #[serde(rename = "ValueKind")]
    pub value_kind: String,
    #[serde(rename = "Data")]
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStateMetadata {
    #[serde(rename = "CapturedAtUtc")]
    pub captured_at_utc: DateTime<chrono::Utc>,
}
