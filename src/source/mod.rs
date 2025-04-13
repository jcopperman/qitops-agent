use anyhow::{Result, anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

impl SourceType {
    /// Parse source type from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "requirements" => Ok(SourceType::Requirements),
            "standard" => Ok(SourceType::Standard),
            "test-strategy" | "teststrategy" => Ok(SourceType::TestStrategy),
            "bug-history" | "bughistory" => Ok(SourceType::BugHistory),
            "documentation" | "docs" => Ok(SourceType::Documentation),
            _ => Ok(SourceType::Custom(s.to_string())),
        }
    }
    
    /// Convert source type to string
    pub fn to_string(&self) -> String {
        match self {
            SourceType::Requirements => "requirements".to_string(),
            SourceType::Standard => "standard".to_string(),
            SourceType::TestStrategy => "test-strategy".to_string(),
            SourceType::BugHistory => "bug-history".to_string(),
            SourceType::Documentation => "documentation".to_string(),
            SourceType::Custom(s) => s.clone(),
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
pub struct SourceManagerConfig {
    /// Sources
    pub sources: HashMap<String, Source>,
}

impl Default for SourceManagerConfig {
    fn default() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }
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
        
        Ok(Self {
            sources: config.sources,
            config_path,
        })
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
