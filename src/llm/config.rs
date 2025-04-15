use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};

use crate::llm::client::RouterConfig;

/// Configuration manager for LLM router
pub struct ConfigManager {
    config_path: PathBuf,
    config: RouterConfig,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_config(&config_path)?;

        Ok(Self {
            config_path,
            config,
        })
    }

    /// Create a new configuration manager with a custom config path
    #[allow(dead_code)]
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_path = path.as_ref().to_path_buf();
        let config = Self::load_config(&config_path)?;

        Ok(Self {
            config_path,
            config,
        })
    }

    /// Get the default configuration path
    fn get_config_path() -> Result<PathBuf> {
        // Try to find the config in the following locations:
        // 1. Current directory
        // 2. User's home directory

        // Check current directory
        let current_dir = std::env::current_dir()?;
        let current_dir_config = current_dir.join("qitops-config.json");
        if current_dir_config.exists() {
            return Ok(current_dir_config);
        }

        // Check home directory
        if let Some(home_dir) = dirs::home_dir() {
            let home_config_dir = home_dir.join(".qitops");
            let home_config = home_config_dir.join("config.json");
            if home_config.exists() {
                return Ok(home_config);
            }

            // Create the config directory if it doesn't exist
            if !home_config_dir.exists() {
                fs::create_dir_all(&home_config_dir)?;
            }

            // Return the home config path even if it doesn't exist yet
            return Ok(home_config);
        }

        // Fallback to current directory
        Ok(current_dir_config)
    }

    /// Load the configuration from the given path
    fn load_config(path: &Path) -> Result<RouterConfig> {
        if path.exists() {
            let config_str = fs::read_to_string(path)
                .context(format!("Failed to read config file: {}", path.display()))?;

            let config: RouterConfig = serde_json::from_str(&config_str)
                .context(format!("Failed to parse config file: {}", path.display()))?;

            Ok(config)
        } else {
            // Return default config if the file doesn't exist
            Ok(RouterConfig::default())
        }
    }

    /// Save the configuration to the given path
    pub fn save_config(&self) -> Result<()> {
        let config_str = serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize config")?;

        fs::write(&self.config_path, config_str)
            .context(format!("Failed to write config file: {}", self.config_path.display()))?;

        Ok(())
    }

    /// Get the configuration
    pub fn get_config(&self) -> &RouterConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn get_config_mut(&mut self) -> &mut RouterConfig {
        &mut self.config
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, provider: String) -> Result<()> {
        // Check if the provider exists
        if !self.config.providers.iter().any(|p| p.provider_type == provider) {
            return Err(anyhow::anyhow!("Provider not found: {}", provider));
        }

        self.config.default_provider = provider;
        Ok(())
    }

    /// Add a provider
    pub fn add_provider(&mut self, provider: crate::llm::client::ProviderConfig) -> Result<()> {
        // Check if the provider already exists
        if self.config.providers.iter().any(|p| p.provider_type == provider.provider_type) {
            return Err(anyhow::anyhow!("Provider already exists: {}", provider.provider_type));
        }

        self.config.providers.push(provider);
        Ok(())
    }

    /// Remove a provider
    pub fn remove_provider(&mut self, provider_type: &str) -> Result<()> {
        // Check if the provider exists
        if !self.config.providers.iter().any(|p| p.provider_type == provider_type) {
            return Err(anyhow::anyhow!("Provider not found: {}", provider_type));
        }

        // Check if it's the default provider
        if self.config.default_provider == provider_type {
            return Err(anyhow::anyhow!("Cannot remove the default provider"));
        }

        self.config.providers.retain(|p| p.provider_type != provider_type);

        // Remove any task mappings to this provider
        self.config.task_providers.retain(|_, v| v != provider_type);

        Ok(())
    }

    /// Set a task provider mapping
    pub fn set_task_provider(&mut self, task: String, provider: String) -> Result<()> {
        // Check if the provider exists
        if !self.config.providers.iter().any(|p| p.provider_type == provider) {
            return Err(anyhow::anyhow!("Provider not found: {}", provider));
        }

        self.config.task_providers.insert(task, provider);
        Ok(())
    }

    /// Remove a task provider mapping
    #[allow(dead_code)]
    pub fn remove_task_provider(&mut self, task: &str) -> Result<()> {
        if !self.config.task_providers.contains_key(task) {
            return Err(anyhow::anyhow!("Task mapping not found: {}", task));
        }

        self.config.task_providers.remove(task);
        Ok(())
    }
}
