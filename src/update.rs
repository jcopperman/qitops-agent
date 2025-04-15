use anyhow::Result;
use serde::Deserialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

/// GitHub release information
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    /// Release tag name (e.g., "v0.1.0")
    tag_name: String,

    /// Release name
    #[allow(dead_code)]
    name: String,

    /// Release URL
    html_url: String,

    /// Release body (description)
    body: String,

    /// Whether this is a prerelease
    #[allow(dead_code)]
    prerelease: bool,
}

/// Version check result
#[derive(Debug)]
pub struct VersionCheckResult {
    /// Current version
    pub current_version: String,

    /// Latest version
    pub latest_version: String,

    /// Whether an update is available
    #[allow(dead_code)]
    pub update_available: bool,

    /// Release URL
    pub release_url: String,

    /// Release notes
    pub release_notes: String,
}

/// Check for updates
pub async fn check_for_updates() -> Result<Option<VersionCheckResult>> {
    // Get the current version
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    // Check if we should skip the update check
    if std::env::var("QITOPS_SKIP_UPDATE_CHECK").is_ok() {
        info!("Update check skipped due to QITOPS_SKIP_UPDATE_CHECK environment variable");
        return Ok(None);
    }

    // Check if we've checked for updates recently
    if !should_check_for_updates()? {
        info!("Update check skipped (checked recently)");
        return Ok(None);
    }

    // Get the latest release from GitHub
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/jcopperman/qitops-agent/releases/latest")
        .header("User-Agent", format!("QitOps-Agent/{}", current_version))
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    // Check if the request was successful
    if !response.status().is_success() {
        warn!("Failed to check for updates: {}", response.status());
        return Ok(None);
    }

    // Parse the response
    let release: GitHubRelease = response.json().await?;

    // Update the last check time
    update_last_check_time()?;

    // Clean up the version strings
    let latest_version = release.tag_name.trim_start_matches('v').to_string();

    // Check if an update is available
    let update_available = latest_version != current_version;

    if update_available {
        info!("Update available: {} -> {}", current_version, latest_version);

        // Return the result
        Ok(Some(VersionCheckResult {
            current_version,
            latest_version,
            update_available,
            release_url: release.html_url,
            release_notes: release.body,
        }))
    } else {
        info!("No updates available");
        Ok(None)
    }
}

/// Check if we should check for updates
fn should_check_for_updates() -> Result<bool> {
    // Get the last check time
    let last_check = get_last_check_time()?;

    // If we've never checked, or it's been more than a day, check again
    match last_check {
        None => Ok(true),
        Some(time) => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let day_in_seconds = 24 * 60 * 60;

            Ok(now - time > day_in_seconds)
        }
    }
}

/// Get the last update check time
fn get_last_check_time() -> Result<Option<u64>> {
    let path = get_last_check_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let time_str = fs::read_to_string(path)?;
    let time = time_str.trim().parse::<u64>()?;

    Ok(Some(time))
}

/// Update the last check time
fn update_last_check_time() -> Result<()> {
    let path = get_last_check_path()?;

    // Create the directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Get the current time
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // Write the time to the file
    fs::write(path, now.to_string())?;

    Ok(())
}

/// Get the path to the last check time file
fn get_last_check_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("qitops");

    Ok(config_dir.join("last_update_check"))
}

/// Print update information
pub fn print_update_info(result: &VersionCheckResult) {
    println!("\n🚀 Update available: {} -> {}", result.current_version, result.latest_version);
    println!("Download: {}", result.release_url);
    println!("\nRelease notes:");

    // Print a shortened version of the release notes
    let notes = result.release_notes.lines()
        .take(5)
        .collect::<Vec<&str>>()
        .join("\n");

    println!("{}", notes);

    if result.release_notes.lines().count() > 5 {
        println!("...");
    }

    println!("\nTo update, run: git pull && cargo build --release\n");
}
