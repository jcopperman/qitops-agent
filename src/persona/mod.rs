use anyhow::{Result, anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Persona ID
    pub id: String,
    
    /// Persona name
    pub name: String,
    
    /// Focus areas
    pub focus_areas: Vec<String>,
    
    /// Persona description
    pub description: String,
    
    /// Prompt template
    pub prompt_template: Option<String>,
}

impl Persona {
    /// Create a new persona
    pub fn new(
        id: String,
        name: String,
        focus_areas: Vec<String>,
        description: String,
        prompt_template: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            focus_areas,
            description,
            prompt_template,
        }
    }
    
    /// Get prompt for persona
    pub fn get_prompt(&self) -> String {
        if let Some(template) = &self.prompt_template {
            return template.clone();
        }
        
        // Default prompt template
        format!(
            "You are acting as a {} with expertise in {}. {}\n\n",
            self.name,
            self.focus_areas.join(", "),
            self.description
        )
    }
}

/// Persona manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaManagerConfig {
    /// Personas
    pub personas: HashMap<String, Persona>,
}

impl Default for PersonaManagerConfig {
    fn default() -> Self {
        let mut personas = HashMap::new();
        
        // Add default personas
        personas.insert(
            "developer".to_string(),
            Persona::new(
                "developer".to_string(),
                "Software Developer".to_string(),
                vec!["code quality".to_string(), "maintainability".to_string(), "edge cases".to_string()],
                "Focus on code quality, maintainability, and edge cases.".to_string(),
                None,
            ),
        );
        
        personas.insert(
            "qa-engineer".to_string(),
            Persona::new(
                "qa-engineer".to_string(),
                "QA Engineer".to_string(),
                vec!["test coverage".to_string(), "regression testing".to_string(), "user scenarios".to_string()],
                "Focus on comprehensive test coverage and regression testing.".to_string(),
                None,
            ),
        );
        
        personas.insert(
            "security-analyst".to_string(),
            Persona::new(
                "security-analyst".to_string(),
                "Security Analyst".to_string(),
                vec!["security".to_string(), "vulnerabilities".to_string(), "compliance".to_string()],
                "Focus on security vulnerabilities and compliance issues.".to_string(),
                None,
            ),
        );
        
        personas.insert(
            "performance-engineer".to_string(),
            Persona::new(
                "performance-engineer".to_string(),
                "Performance Engineer".to_string(),
                vec!["performance".to_string(), "optimization".to_string(), "scalability".to_string()],
                "Focus on performance implications and bottlenecks.".to_string(),
                None,
            ),
        );
        
        Self {
            personas,
        }
    }
}

/// Persona manager
pub struct PersonaManager {
    /// Personas
    personas: HashMap<String, Persona>,
    
    /// Configuration path
    config_path: PathBuf,
}

impl PersonaManager {
    /// Create a new persona manager
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
        let config_path = config_dir.join("personas.json");
        
        // Load config if it exists, otherwise create default
        let config = if config_path.exists() {
            let config_str = fs::read_to_string(&config_path)
                .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
                
            serde_json::from_str(&config_str)
                .map_err(|e| anyhow!("Failed to parse config file: {}", e))?
        } else {
            let default_config = PersonaManagerConfig::default();
            
            // Save default config
            let config_str = serde_json::to_string_pretty(&default_config)
                .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
                
            fs::write(&config_path, config_str)
                .map_err(|e| anyhow!("Failed to write config file: {}", e))?;
                
            default_config
        };
        
        Ok(Self {
            personas: config.personas,
            config_path,
        })
    }
    
    /// Add a persona
    pub fn add_persona(&mut self, persona: Persona) -> Result<()> {
        // Add persona
        self.personas.insert(persona.id.clone(), persona);
        
        // Save config
        self.save_config()
    }
    
    /// Get a persona
    pub fn get_persona(&self, id: &str) -> Option<&Persona> {
        self.personas.get(id)
    }
    
    /// List personas
    pub fn list_personas(&self) -> Vec<&Persona> {
        self.personas.values().collect()
    }
    
    /// Remove a persona
    pub fn remove_persona(&mut self, id: &str) -> Result<()> {
        if self.personas.remove(id).is_none() {
            return Err(anyhow!("Persona not found: {}", id));
        }
        
        // Save config
        self.save_config()
    }
    
    /// Get prompt for personas
    pub fn get_prompt_for_personas(&self, ids: &[String]) -> Result<String> {
        let mut prompt = String::new();
        
        for id in ids {
            let persona = self.get_persona(id)
                .ok_or_else(|| anyhow!("Persona not found: {}", id))?;
                
            prompt.push_str(&persona.get_prompt());
        }
        
        Ok(prompt)
    }
    
    /// Save config
    fn save_config(&self) -> Result<()> {
        let config = PersonaManagerConfig {
            personas: self.personas.clone(),
        };
        
        let config_str = serde_json::to_string_pretty(&config)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
            
        fs::write(&self.config_path, config_str)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;
            
        Ok(())
    }
}
