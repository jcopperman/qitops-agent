use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use tracing::info;

use super::loader::{Plugin, PluginMetadata};

/// Plugin registry for QitOps Agent
pub struct PluginRegistry {
    /// Registered plugins
    plugins: HashMap<String, Arc<dyn Plugin>>,
    /// Plugin metadata
    metadata: HashMap<String, PluginMetadata>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Arc<dyn Plugin>, metadata: PluginMetadata) -> Result<()> {
        let id = metadata.id.clone();

        // Check if plugin is already registered
        if self.plugins.contains_key(&id) {
            return Err(anyhow!("Plugin with ID '{}' is already registered", id));
        }

        // Register the plugin
        info!("Registering plugin: {} v{}", metadata.name, metadata.version);
        self.plugins.insert(id.clone(), plugin);
        self.metadata.insert(id, metadata);

        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister(&mut self, id: &str) -> Result<()> {
        // Check if plugin is registered
        if !self.plugins.contains_key(id) {
            return Err(anyhow!("Plugin with ID '{}' is not registered", id));
        }

        // Get plugin metadata
        let metadata = self.metadata.get(id).unwrap();

        // Unregister the plugin
        info!("Unregistering plugin: {} v{}", metadata.name, metadata.version);
        self.plugins.remove(id);
        self.metadata.remove(id);

        Ok(())
    }

    /// Get a plugin by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(id).cloned()
    }

    /// Get plugin metadata by ID
    pub fn get_metadata(&self, id: &str) -> Option<&PluginMetadata> {
        self.metadata.get(id)
    }

    /// Get all plugin metadata
    pub fn get_all_metadata(&self) -> Vec<(String, &PluginMetadata)> {
        self.metadata.iter().map(|(id, metadata)| (id.clone(), metadata)).collect()
    }
}

/// Global plugin registry
pub static PLUGIN_REGISTRY: once_cell::sync::Lazy<Arc<RwLock<PluginRegistry>>> =
    once_cell::sync::Lazy::new(|| {
        Arc::new(RwLock::new(PluginRegistry::new()))
    });

/// Register a plugin in the global registry
pub fn register_plugin(plugin: Arc<dyn Plugin>, metadata: PluginMetadata) -> Result<()> {
    let mut registry = PLUGIN_REGISTRY.write().map_err(|e| anyhow!("Failed to acquire write lock on plugin registry: {}", e))?;
    registry.register(plugin, metadata)
}

/// Unregister a plugin from the global registry
pub fn unregister_plugin(id: &str) -> Result<()> {
    let mut registry = PLUGIN_REGISTRY.write().map_err(|e| anyhow!("Failed to acquire write lock on plugin registry: {}", e))?;
    registry.unregister(id)
}

/// Get a plugin from the global registry
pub fn get_plugin(id: &str) -> Result<Option<Arc<dyn Plugin>>> {
    let registry = PLUGIN_REGISTRY.read().map_err(|e| anyhow!("Failed to acquire read lock on plugin registry: {}", e))?;
    Ok(registry.get(id))
}

/// Get plugin metadata from the global registry
pub fn get_plugin_metadata(id: &str) -> Result<Option<PluginMetadata>> {
    let registry = PLUGIN_REGISTRY.read().map_err(|e| anyhow!("Failed to acquire read lock on plugin registry: {}", e))?;
    Ok(registry.get_metadata(id).cloned())
}

/// Get all plugin metadata from the global registry
pub fn get_all_plugin_metadata() -> Result<Vec<(String, PluginMetadata)>> {
    let registry = PLUGIN_REGISTRY.read().map_err(|e| anyhow!("Failed to acquire read lock on plugin registry: {}", e))?;
    Ok(registry.get_all_metadata().into_iter().map(|(id, metadata)| (id, metadata.clone())).collect())
}

/// Initialize the plugin system
pub fn init() -> Result<()> {
    info!("Initializing plugin system");

    // Create the plugin registry
    let registry = PLUGIN_REGISTRY.read().map_err(|e| anyhow!("Failed to acquire read lock on plugin registry: {}", e))?;

    let plugin_count = registry.plugins.len();
    info!("Plugin system initialized with {} plugins", plugin_count);

    Ok(())
}
