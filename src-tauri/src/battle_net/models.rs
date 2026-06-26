use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

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
}

impl AccountInfo {
    pub fn new(remark: String, username: String, group_id: String, tags: Vec<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string().replace('-', ""),
            remark,
            username,
            last_used: Local::now(),
            group_id,
            tags,
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
