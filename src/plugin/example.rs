use std::sync::Arc;
use anyhow::Result;

use super::loader::{Plugin, PluginMetadata};
use super::registry::register_plugin;

/// Example plugin for QitOps Agent
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn execute(&self, args: &[String]) -> Result<String> {
        Ok(format!("Example plugin executed with args: {:?}", args))
    }
}

/// Register the example plugin
pub fn register_example_plugin() -> Result<()> {
    // Create plugin metadata
    let metadata = PluginMetadata {
        id: "example".to_string(),
        name: "Example Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "An example plugin for QitOps Agent".to_string(),
        author: "QitOps Team".to_string(),
    };
    
    // Create plugin instance
    let plugin = Arc::new(ExamplePlugin);
    
    // Register the plugin
    register_plugin(plugin, metadata)
}
