use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// Default sources for the command
    #[serde(default)]
    pub default_sources: Vec<String>,

    /// Default personas for the command
    #[serde(default)]
    pub default_personas: Vec<String>,

    /// Other command-specific configuration
    #[serde(flatten)]
    pub other: serde_json::Value,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            default_sources: Vec::new(),
            default_personas: Vec::new(),
            other: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// Sources configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SourcesConfig {
    /// Default sources
    #[serde(default)]
    pub default: Option<String>,

    /// Source paths
    #[serde(default)]
    pub paths: HashMap<String, String>,
}


/// Personas configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct PersonasConfig {
    /// Default persona
    #[serde(default)]
    pub default: Option<String>,
}


/// QitOps configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QitOpsConfig {
    /// Command-specific configuration
    #[serde(default)]
    pub commands: HashMap<String, CommandConfig>,

    /// Sources configuration
    #[serde(default)]
    pub sources: SourcesConfig,

    /// Personas configuration
    #[serde(default)]
    pub personas: PersonasConfig,

    /// Other configuration
    #[serde(flatten)]
    pub other: serde_json::Value,
}

impl Default for QitOpsConfig {
    fn default() -> Self {
        Self {
            commands: HashMap::new(),
            sources: SourcesConfig::default(),
            personas: PersonasConfig::default(),
            other: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// QitOps configuration manager
pub struct QitOpsConfigManager {
    /// Configuration
    config: QitOpsConfig,

    /// Configuration path
    #[allow(dead_code)]
    config_path: PathBuf,
}

impl QitOpsConfigManager {
    /// Create a new QitOps configuration manager
    pub fn new() -> Result<Self> {
        // Get config directory
        let config_dir = if cfg!(windows) {
            let app_data = std::env::var("APPDATA")
                .map_err(|_| anyhow!("APPDATA environment variable not set"))?;
            PathBuf::from(app_data).join("qitops")
        } else {
            let home = std::env::var("HOME")
                .map_err(|_| anyhow!("HOME environment variable not set"))?;
            PathBuf::from(home).join(".config").join("qitops")
        };

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| anyhow!("Failed to create config directory: {}", e))?;
        }

        // Config file path
        let config_path = config_dir.join("config.json");

        // Load config if it exists, otherwise create default
        let config = if config_path.exists() {
            let config_str = fs::read_to_string(&config_path)
                .map_err(|e| anyhow!("Failed to read config file: {}", e))?;

            serde_json::from_str(&config_str)
                .map_err(|e| anyhow!("Failed to parse config file: {}", e))?
        } else {
            QitOpsConfig::default()
        };

        Ok(Self {
            config,
            config_path,
        })
    }

    /// Get the configuration
    #[allow(dead_code)]
    pub fn get_config(&self) -> &QitOpsConfig {
        &self.config
    }

    /// Get default sources for a command
    pub fn get_default_sources(&self, command: &str) -> Vec<String> {
        // Check command-specific default sources
        if let Some(command_config) = self.config.commands.get(command) {
            if !command_config.default_sources.is_empty() {
                return command_config.default_sources.clone();
            }
        }

        // Check global default sources
        if let Some(default_sources) = &self.config.sources.default {
            return default_sources.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        // Check environment variable
        if let Ok(default_sources) = std::env::var("QITOPS_DEFAULT_SOURCES") {
            return default_sources.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        Vec::new()
    }

    /// Get default personas for a command
    pub fn get_default_personas(&self, command: &str) -> Vec<String> {
        // Check command-specific default personas
        if let Some(command_config) = self.config.commands.get(command) {
            if !command_config.default_personas.is_empty() {
                return command_config.default_personas.clone();
            }
        }

        // Check global default persona
        if let Some(default_persona) = &self.config.personas.default {
            return vec![default_persona.clone()];
        }

        // Check environment variable
        if let Ok(default_personas) = std::env::var("QITOPS_DEFAULT_PERSONAS") {
            return default_personas.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        Vec::new()
    }

    /// Save the configuration
    #[allow(dead_code)]
    pub fn save_config(&self) -> Result<()> {
        let config_str = serde_json::to_string_pretty(&self.config)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        fs::write(&self.config_path, config_str)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }
}
