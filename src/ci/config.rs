use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// GitHub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub API token
    pub token: Option<String>,
    
    /// GitHub API base URL (for GitHub Enterprise)
    pub api_base: Option<String>,
    
    /// Default repository owner
    pub default_owner: Option<String>,
    
    /// Default repository name
    pub default_repo: Option<String>,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: None,
            api_base: Some("https://api.github.com".to_string()),
            default_owner: None,
            default_repo: None,
        }
    }
}

/// GitHub configuration manager
pub struct GitHubConfigManager {
    /// Configuration file path
    config_path: PathBuf,
    
    /// Configuration
    config: GitHubConfig,
}

impl GitHubConfigManager {
    /// Create a new GitHub configuration manager
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
        let config_path = config_dir.join("github.json");
        
        // Load config if it exists, otherwise create default
        let config = if config_path.exists() {
            let config_str = fs::read_to_string(&config_path)
                .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
                
            serde_json::from_str(&config_str)
                .map_err(|e| anyhow!("Failed to parse config file: {}", e))?
        } else {
            GitHubConfig::default()
        };
        
        Ok(Self {
            config_path,
            config,
        })
    }
    
    /// Get the configuration
    pub fn get_config(&self) -> &GitHubConfig {
        &self.config
    }
    
    /// Set the GitHub token
    pub fn set_token(&mut self, token: String) -> Result<()> {
        self.config.token = Some(token);
        self.save_config()
    }
    
    /// Set the GitHub API base URL
    pub fn set_api_base(&mut self, api_base: String) -> Result<()> {
        self.config.api_base = Some(api_base);
        self.save_config()
    }
    
    /// Set the default repository owner
    pub fn set_default_owner(&mut self, owner: String) -> Result<()> {
        self.config.default_owner = Some(owner);
        self.save_config()
    }
    
    /// Set the default repository name
    pub fn set_default_repo(&mut self, repo: String) -> Result<()> {
        self.config.default_repo = Some(repo);
        self.save_config()
    }
    
    /// Save the configuration
    pub fn save_config(&self) -> Result<()> {
        let config_str = serde_json::to_string_pretty(&self.config)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
            
        fs::write(&self.config_path, config_str)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;
            
        Ok(())
    }
    
    /// Get the GitHub token
    pub fn get_token(&self) -> Option<String> {
        // First check the config
        if let Some(token) = &self.config.token {
            return Some(token.clone());
        }
        
        // Then check the environment variable
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            return Some(token);
        }
        
        None
    }
    
    /// Get the GitHub API base URL
    pub fn get_api_base(&self) -> String {
        self.config.api_base.clone().unwrap_or_else(|| "https://api.github.com".to_string())
    }
    
    /// Get the default repository owner
    pub fn get_default_owner(&self) -> Option<String> {
        self.config.default_owner.clone()
    }
    
    /// Get the default repository name
    pub fn get_default_repo(&self) -> Option<String> {
        self.config.default_repo.clone()
    }
}
