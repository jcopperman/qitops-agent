use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use tracing::info;

/// Save plugin state to a file
pub fn save_plugin_state(enabled_plugins: &[String]) -> Result<()> {
    // Create the plugin directory if it doesn't exist
    let plugin_dir = PathBuf::from("plugins");
    if !plugin_dir.exists() {
        fs::create_dir_all(&plugin_dir)?;
    }

    // Save the enabled plugins to a file
    let state_file = plugin_dir.join("state.json");
    let state_json = serde_json::to_string(enabled_plugins)?;
    fs::write(state_file, state_json)?;

    info!("Plugin state saved");

    Ok(())
}

/// Load plugin state from a file
pub fn load_plugin_state() -> Result<Vec<String>> {
    // Check if the plugin directory exists
    let plugin_dir = PathBuf::from("plugins");
    if !plugin_dir.exists() {
        return Ok(Vec::new());
    }

    // Check if the state file exists
    let state_file = plugin_dir.join("state.json");
    if !state_file.exists() {
        return Ok(Vec::new());
    }

    // Load the enabled plugins from the file
    let state_json = fs::read_to_string(state_file)?;
    let enabled_plugins: Vec<String> = serde_json::from_str(&state_json)?;

    info!("Plugin state loaded: {:?}", enabled_plugins);

    Ok(enabled_plugins)
}
