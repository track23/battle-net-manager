use std::fs;
use std::path::Path;

use serde_json::{json, Map, Value as JsonValue};

use super::region::{get_region_settings, normalize_region, UNSET_REGION};

/// Create a minimal Battle.net login config for the given region and write it
/// to `config_path`. Used when adding a new account.
pub fn write_new_account_login_config(config_path: &Path, region: &str) {
    let region = normalize_region(region);
    if region == UNSET_REGION {
        return;
    }

    let settings = get_region_settings(&region);
    let config = json!({
        "Client": {
            "LoginSettings": {
                "AllowedRegions": settings.allowed_regions,
                "AllowedLocales": settings.allowed_locales
            }
        },
        "Services": {
            "LastLoginAddress": settings.login_address,
            "LastLoginRegion": settings.login_region,
            "LastLoginTassadar": settings.tassadar_host
        }
    });

    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(config_path, config.to_string());
}

/// Write a cross-region login config: reads the source config, strips
/// authentication-related fields, and applies region settings for the target
/// region. Falls back to a plain copy on any error.
pub fn write_cross_region_login_config(source: &Path, destination: &Path, target_region: &str) {
    let result = (|| -> Option<()> {
        let content = fs::read_to_string(source).ok()?;
        let mut root: JsonValue = serde_json::from_str(&content).ok()?;

        // Strip auth-related fields from Client section
        if let Some(client) = root.get_mut("Client").and_then(|v| v.as_object_mut()) {
            client.remove("AutoLogin");
            client.remove("AutoLoginCN");
            client.remove("RememberAccountName");
            client.remove("SavedAccountNames");
        }

        let settings = get_region_settings(target_region);

        // Apply region settings to root
        apply_region_settings(&mut root, &settings, true);

        // Apply to all nested sections (e.g. per-game overrides)
        let keys: Vec<String> = root
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        for key in keys {
            if let Some(section) = root.get_mut(&key).and_then(|v| v.as_object_mut()) {
                let mut section_val = JsonValue::Object(section.clone());
                apply_region_settings(&mut section_val, &settings, false);
                if let Some(obj) = section_val.as_object() {
                    *section = obj.clone();
                }
            }
        }

        let output = serde_json::to_string_pretty(&root).ok()?;
        fs::write(destination, output).ok()?;
        Some(())
    })();

    if result.is_none() {
        // Fallback: plain copy
        let _ = fs::copy(source, destination);
    }
}

fn apply_region_settings(
    section: &mut JsonValue,
    settings: &super::region::RegionSettings,
    create_missing: bool,
) {
    // Apply LoginSettings
    let login_settings = section
        .get_mut("Client")
        .and_then(|c| c.get_mut("LoginSettings"));

    if login_settings.is_some() || create_missing {
        // Ensure Client object exists
        if section.get("Client").is_none() && create_missing {
            section["Client"] = JsonValue::Object(Map::new());
        }
        if let Some(client) = section.get_mut("Client").and_then(|v| v.as_object_mut()) {
            if client.get("LoginSettings").is_none() && create_missing {
                client.insert(
                    "LoginSettings".to_string(),
                    JsonValue::Object(Map::new()),
                );
            }
            if let Some(ls) = client
                .get_mut("LoginSettings")
                .and_then(|v| v.as_object_mut())
            {
                ls.insert(
                    "AllowedRegions".to_string(),
                    json!(settings.allowed_regions),
                );
                ls.insert(
                    "AllowedLocales".to_string(),
                    json!(settings.allowed_locales),
                );
            }
        }
    }

    // Apply Services
    if section.get("Services").is_none() && create_missing {
        section["Services"] = JsonValue::Object(Map::new());
    }
    if let Some(services) = section.get_mut("Services").and_then(|v| v.as_object_mut()) {
        services.insert(
            "LastLoginAddress".to_string(),
            json!(settings.login_address),
        );
        services.insert(
            "LastLoginRegion".to_string(),
            json!(settings.login_region),
        );
        services.insert(
            "LastLoginTassadar".to_string(),
            json!(settings.tassadar_host),
        );
    }
}
