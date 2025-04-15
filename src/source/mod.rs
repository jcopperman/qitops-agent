use anyhow::{Result, anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceType {
    /// Requirements document
    Requirements,

    /// Coding standard
    Standard,

    /// Test strategy
    TestStrategy,

    /// Bug history
    BugHistory,

    /// Documentation
    Documentation,

    /// Custom source type
    Custom(String),
}

impl std::str::FromStr for SourceType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "requirements" => Ok(SourceType::Requirements),
            "standard" => Ok(SourceType::Standard),
            "test-strategy" | "teststrategy" => Ok(SourceType::TestStrategy),
            "bug-history" | "bughistory" => Ok(SourceType::BugHistory),
            "documentation" | "docs" => Ok(SourceType::Documentation),
            _ => Ok(SourceType::Custom(s.to_string())),
        }
    }

}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Requirements => write!(f, "requirements"),
            SourceType::Standard => write!(f, "standard"),
            SourceType::TestStrategy => write!(f, "test-strategy"),
            SourceType::BugHistory => write!(f, "bug-history"),
            SourceType::Documentation => write!(f, "documentation"),
            SourceType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Source ID
    pub id: String,

    /// Source type
    pub source_type: SourceType,

    /// Source path
    pub path: PathBuf,

    /// Source description
    pub description: Option<String>,

    /// Source metadata
    pub metadata: HashMap<String, String>,
}

impl Source {
    /// Create a new source
    pub fn new(
        id: String,
        source_type: SourceType,
        path: PathBuf,
        description: Option<String>,
    ) -> Self {
        Self {
            id,
            source_type,
            path,
            description,
            metadata: HashMap::new(),
        }
    }

    /// Get source content
    pub fn get_content(&self) -> Result<String> {
        fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read source file: {}", self.path.display()))
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// Source manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SourceManagerConfig {
    /// Sources
    pub sources: HashMap<String, Source>,
}


/// Source manager
pub struct SourceManager {
    /// Sources
    sources: HashMap<String, Source>,

    /// Configuration path
    config_path: PathBuf,
}

impl SourceManager {
    /// Create a new source manager
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
        let config_path = config_dir.join("sources.json");

        // Load config if it exists, otherwise create default
        let config = if config_path.exists() {
            let config_str = fs::read_to_string(&config_path)
                .map_err(|e| anyhow!("Failed to read config file: {}", e))?;

            serde_json::from_str(&config_str)
                .map_err(|e| anyhow!("Failed to parse config file: {}", e))?
        } else {
            SourceManagerConfig::default()
        };

        // Create source manager
        let mut source_manager = Self {
            sources: config.sources,
            config_path,
        };

        // Check for environment variables
        source_manager.load_from_environment()?;

        Ok(source_manager)
    }

    /// Load sources from environment variables
    fn load_from_environment(&mut self) -> Result<()> {
        // Check for QITOPS_SOURCES environment variable
        if let Ok(sources_env) = std::env::var("QITOPS_SOURCES") {
            tracing::info!("Loading sources from QITOPS_SOURCES environment variable");
            // Format: "id1:type1:path1,id2:type2:path2"
            for source_str in sources_env.split(',') {
                let parts: Vec<&str> = source_str.split(':').collect();
                if parts.len() >= 3 {
                    let id = parts[0].trim().to_string();
                    let source_type = parts[1].trim().parse::<SourceType>()?;
                    let path = PathBuf::from(parts[2].trim());

                    // Optional description
                    let description = if parts.len() > 3 {
                        Some(parts[3].trim().to_string())
                    } else {
                        None
                    };

                    // Create and add the source
                    let source = Source::new(id.clone(), source_type.clone(), path.clone(), description.clone());
                    self.sources.insert(id.clone(), source);

                    tracing::info!("Added source from environment variable: id={}, type={}, path={}",
                        id, source_type.to_string(), path.display());
                } else {
                    tracing::warn!("Invalid source format in QITOPS_SOURCES: {}", source_str);
                }
            }

            // Save the updated configuration
            self.save_config()?;
        }

        // Check for individual source environment variables
        // Format: QITOPS_SOURCE_<ID>="type:path[:description]"
        let mut found_source_vars = false;
        for (key, value) in std::env::vars() {
            if key.starts_with("QITOPS_SOURCE_") {
                if !found_source_vars {
                    tracing::info!("Loading sources from individual environment variables");
                    found_source_vars = true;
                }

                let id = key.strip_prefix("QITOPS_SOURCE_").unwrap().to_lowercase();
                let parts: Vec<&str> = value.split(':').collect();

                if parts.len() >= 2 {
                    let source_type = parts[0].trim().parse::<SourceType>()?;
                    let path = PathBuf::from(parts[1].trim());

                    // Optional description
                    let description = if parts.len() > 2 {
                        Some(parts[2].trim().to_string())
                    } else {
                        None
                    };

                    // Create and add the source
                    let source = Source::new(id.clone(), source_type.clone(), path.clone(), description.clone());
                    self.sources.insert(id.clone(), source);

                    tracing::info!("Added source from environment variable {}: id={}, type={}, path={}",
                        key, id, source_type.to_string(), path.display());
                } else {
                    tracing::warn!("Invalid source format in {}: {}", key, value);
                }
            }
        }

        // Check for default sources environment variable
        if let Ok(default_sources) = std::env::var("QITOPS_DEFAULT_SOURCES") {
            tracing::info!("Found default sources in environment variable: {}", default_sources);
        }

        Ok(())
    }

    /// Add a source
    pub fn add_source(&mut self, source: Source) -> Result<()> {
        // Validate source path
        if !source.path.exists() {
            return Err(anyhow!("Source path does not exist: {}", source.path.display()));
        }

        // Add source
        self.sources.insert(source.id.clone(), source);

        // Save config
        self.save_config()
    }

    /// Get a source
    pub fn get_source(&self, id: &str) -> Option<&Source> {
        self.sources.get(id)
    }

    /// List sources
    pub fn list_sources(&self) -> Vec<&Source> {
        self.sources.values().collect()
    }

    /// Remove a source
    pub fn remove_source(&mut self, id: &str) -> Result<()> {
        if self.sources.remove(id).is_none() {
            return Err(anyhow!("Source not found: {}", id));
        }

        // Save config
        self.save_config()
    }

    /// Get content for sources
    pub fn get_content_for_sources(&self, ids: &[String]) -> Result<String> {
        let mut content = String::new();

        for id in ids {
            let source = self.get_source(id)
                .ok_or_else(|| anyhow!("Source not found: {}", id))?;

            let source_content = source.get_content()?;

            content.push_str(&format!("# Source: {} ({})\n\n", source.id, source.source_type.to_string()));
            content.push_str(&source_content);
            content.push_str("\n\n");
        }

        Ok(content)
    }

    /// Save config
    fn save_config(&self) -> Result<()> {
        let config = SourceManagerConfig {
            sources: self.sources.clone(),
        };

        let config_str = serde_json::to_string_pretty(&config)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

        fs::write(&self.config_path, config_str)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

        Ok(())
    }
}
