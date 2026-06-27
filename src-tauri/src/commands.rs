use crate::battle_net::BattleNetCore;
use serde::Serialize;
use tauri_plugin_updater::UpdaterExt;

// ─── Update info struct ──────────────────────────────────────────────

#[derive(Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    pub date: Option<String>,
}

// ─── Account & Group commands ──────────────────────────────────────────

#[tauri::command]
pub fn get_accounts(core: tauri::State<'_, BattleNetCore>) -> String {
    serde_json::to_string(&core.get_accounts()).unwrap_or_else(|_| "[]".to_string())
}

#[tauri::command]
pub fn get_groups(core: tauri::State<'_, BattleNetCore>) -> String {
    serde_json::to_string(&core.get_groups()).unwrap_or_else(|_| "[]".to_string())
}

#[tauri::command]
pub fn create_group(name: String, core: tauri::State<'_, BattleNetCore>) -> String {
    match core.create_group(&name) {
        Some(group) => serde_json::to_string(&group).unwrap_or_else(|_| "{}".to_string()),
        None => "null".to_string(),
    }
}

#[tauri::command]
pub fn rename_group(id: String, name: String, core: tauri::State<'_, BattleNetCore>) -> bool {
    core.rename_group(&id, &name)
}

#[tauri::command]
pub fn delete_group(id: String, core: tauri::State<'_, BattleNetCore>) -> bool {
    core.delete_group(&id)
}

#[tauri::command(rename_all = "camelCase")]
pub fn move_account_to_group(
    account_id: String,
    group_id: String,
    core: tauri::State<'_, BattleNetCore>,
) -> bool {
    core.move_account_to_group(&account_id, &group_id)
}

#[tauri::command(rename_all = "camelCase")]
pub fn update_account_info(
    account_id: String,
    remark: String,
    battle_tag: String,
    tags: Vec<String>,
    core: tauri::State<'_, BattleNetCore>,
) -> bool {
    core.update_account_info(&account_id, &remark, &battle_tag, &tags)
}

#[tauri::command(rename_all = "camelCase")]
pub fn save_current_account_to_group(
    remark: String,
    battle_tag: String,
    group_id: String,
    tags: Vec<String>,
    core: tauri::State<'_, BattleNetCore>,
) -> bool {
    core.save_current_account_to_group(&remark, &battle_tag, &group_id, &tags)
}

#[tauri::command]
pub fn switch_account(id: String, core: tauri::State<'_, BattleNetCore>) -> bool {
    core.switch_account(&id)
}

#[tauri::command]
pub fn delete_account(id: String, core: tauri::State<'_, BattleNetCore>) -> bool {
    core.delete_account(&id)
}

#[tauri::command]
pub fn add_new_account(core: tauri::State<'_, BattleNetCore>) {
    core.add_new_account();
}

#[tauri::command]
pub fn get_active_account_id(core: tauri::State<'_, BattleNetCore>) -> Option<String> {
    core.get_active_account_id()
}

// ─── External URL ─────────────────────────────────────────────────────

#[tauri::command]
pub fn open_external_url(url: String) -> bool {
    match url::Url::parse(&url) {
        Ok(uri) => {
            if (uri.scheme() == "https" || uri.scheme() == "http")
                && uri
                    .host_str()
                    .map(|h| h.eq_ignore_ascii_case("github.com"))
                    .unwrap_or(false)
            {
                opener::open(&url).is_ok()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

// ─── Window management commands ───────────────────────────────────────

#[tauri::command]
pub fn drag_window(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
pub fn minimize_app(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn close_app(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
pub fn show_window(window: tauri::Window) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

// ─── Updater commands ────────────────────────────────────────────────

#[tauri::command]
pub async fn check_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    match update {
        Some(update) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            notes: update.body.clone(),
            date: update.date.map(|d| d.to_string()),
        })),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    if let Some(update) = update {
        update
            .download_and_install(
                |chunk_length, content_length| {
                    // Progress callback - could emit events here
                    println!("downloaded {} of {:?}", chunk_length, content_length);
                },
                || {
                    // Download finished callback
                    println!("download finished");
                },
            )
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
