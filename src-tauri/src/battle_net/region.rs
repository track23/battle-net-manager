use std::path::Path;

use serde_json::Value as JsonValue;

pub const REGION_CN: &str = "cn";
pub const REGION_ASIA: &str = "asia";
pub const REGION_AMERICAS: &str = "americas";
pub const REGION_EUROPE: &str = "europe";
pub const UNSET_REGION: &str = "";
#[allow(dead_code)]
pub const DEFAULT_ACCOUNT_REGION: &str = REGION_ASIA;

pub struct RegionSettings {
    pub allowed_regions: &'static str,
    pub allowed_locales: &'static str,
    pub login_address: &'static str,
    pub login_region: &'static str,
    pub tassadar_host: &'static str,
}

pub fn normalize_region(region: &str) -> String {
    let lower = region.trim().to_lowercase();
    match lower.as_str() {
        "cn" => REGION_CN.to_string(),
        "asia" => REGION_ASIA.to_string(),
        "americas" => REGION_AMERICAS.to_string(),
        "europe" => REGION_EUROPE.to_string(),
        _ => UNSET_REGION.to_string(),
    }
}

pub fn is_tagged_region(region: &str) -> bool {
    !normalize_region(region).is_empty()
}

pub fn get_region_settings(region: &str) -> RegionSettings {
    match normalize_region(region).as_str() {
        REGION_CN => RegionSettings {
            allowed_regions: "CN",
            allowed_locales: "zhCN",
            login_address: "cn.actual.battlenet.com.cn",
            login_region: "CN",
            tassadar_host: "account.battlenet.com.cn",
        },
        REGION_AMERICAS => RegionSettings {
            allowed_regions: "US",
            allowed_locales: "",
            login_address: "us.actual.battle.net",
            login_region: "US",
            tassadar_host: "account.battle.net",
        },
        REGION_EUROPE => RegionSettings {
            allowed_regions: "EU",
            allowed_locales: "",
            login_address: "eu.actual.battle.net",
            login_region: "EU",
            tassadar_host: "account.battle.net",
        },
        // Default to KR/Asia
        _ => RegionSettings {
            allowed_regions: "KR",
            allowed_locales: "",
            login_address: "kr.actual.battle.net",
            login_region: "KR",
            tassadar_host: "account.battle.net",
        },
    }
}

/// Infer the active region from a Battle.net.config file.
/// Searches recursively for region-related fields, with priority:
/// AllowedRegions > LastLoginRegion/WebRegion > any other region field.
pub fn infer_region_from_config(path: &Path) -> String {
    let Ok(content) = std::fs::read_to_string(path) else {
        return UNSET_REGION.to_string();
    };
    let Ok(json) = serde_json::from_str::<JsonValue>(&content) else {
        return UNSET_REGION.to_string();
    };

    let mut region_values = Vec::new();
    collect_region_values(&json, &mut region_values);

    // Priority 1: AllowedRegions
    for (name, value) in &region_values {
        if name.eq_ignore_ascii_case("AllowedRegions") {
            let mapped = map_region_code(value);
            if is_tagged_region(&mapped) {
                return mapped;
            }
        }
    }

    // Priority 2: LastLoginRegion / WebRegion
    for (name, value) in &region_values {
        if name.eq_ignore_ascii_case("LastLoginRegion") || name.eq_ignore_ascii_case("WebRegion")
        {
            let mapped = map_region_code(value);
            if is_tagged_region(&mapped) {
                return mapped;
            }
        }
    }

    // Priority 3: any region field
    for (_name, value) in &region_values {
        let mapped = map_region_code(value);
        if is_tagged_region(&mapped) {
            return mapped;
        }
    }

    UNSET_REGION.to_string()
}

/// Map a Battle.net region code to our account region constant.
pub fn map_region_code(value: &str) -> String {
    let upper = value.trim().to_uppercase();
    match upper.as_str() {
        "CN" => REGION_CN.to_string(),
        "US" => REGION_AMERICAS.to_string(),
        "EU" => REGION_EUROPE.to_string(),
        "KR" | "TW" | "SG" => REGION_ASIA.to_string(),
        _ => UNSET_REGION.to_string(),
    }
}

pub fn is_cross_region_switch(active_region: &str, target_region: &str) -> bool {
    let target = normalize_region(target_region);
    if !is_tagged_region(&target) {
        return false;
    }

    let active = normalize_region(active_region);
    if !is_tagged_region(&active) {
        return true;
    }

    active != target
}

/// Recursively collect all JSON string values whose key name contains "Region".
fn collect_region_values(element: &JsonValue, values: &mut Vec<(String, String)>) {
    match element {
        JsonValue::Object(map) => {
            for (key, val) in map {
                if let JsonValue::String(s) = val {
                    if key.to_lowercase().contains("region") {
                        values.push((key.clone(), s.clone()));
                    }
                }
                collect_region_values(val, values);
            }
        }
        JsonValue::Array(arr) => {
            for item in arr {
                collect_region_values(item, values);
            }
        }
        _ => {}
    }
}
