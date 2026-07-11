use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub update_available: bool,
}

/// Parse a version string, stripping a leading "v" prefix, into a semver
/// `Version` suitable for ordering comparisons.
fn parse_version(raw: &str) -> Option<semver::Version> {
    let stripped = raw.trim().trim_start_matches('v');
    semver::Version::parse(stripped).ok()
}

#[tauri::command]
pub async fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    let current_raw = voice_type::VERSION;
    let current = match parse_version(current_raw) {
        Some(v) => v,
        None => return Ok(None),
    };

    // Skip prerelease builds of the running app.
    if !current.pre.is_empty() {
        return Ok(None);
    }

    // Any network failure is treated as "check failed, no update info".
    let response = reqwest::Client::new()
        .get("https://api.github.com/repos/boring877/voice_type_rust/releases/latest")
        .header("User-Agent", "Voice-Type-Desktop")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok();

    let response = match response {
        Some(r) if r.status().is_success() => r,
        _ => return Ok(None),
    };

    let body: serde_json::Value = match response.json().await {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };

    let tag = body["tag_name"].as_str().unwrap_or("");
    let latest = match parse_version(tag) {
        Some(v) => v,
        None => return Ok(None),
    };

    // Skip prerelease releases.
    if !latest.pre.is_empty() {
        return Ok(None);
    }

    // An update is available only if latest is strictly greater than current.
    if latest <= current {
        return Ok(None);
    }

    let release_url = body["html_url"]
        .as_str()
        .unwrap_or("https://github.com/boring877/voice_type_rust/releases")
        .to_string();

    Ok(Some(UpdateInfo {
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        release_url,
        update_available: true,
    }))
}
