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

        // Create persona manager
        let mut persona_manager = Self {
            personas: config.personas,
            config_path,
        };

        // Check for environment variables
        persona_manager.load_from_environment()?;

        Ok(persona_manager)
    }

    /// Load personas from environment variables
    fn load_from_environment(&mut self) -> Result<()> {
        // Check for QITOPS_PERSONAS environment variable
        if let Ok(personas_env) = std::env::var("QITOPS_PERSONAS") {
            tracing::info!("Loading personas from QITOPS_PERSONAS environment variable");
            // Format: "id1:name1:focus1:description1,id2:name2:focus2:description2"
            for persona_str in personas_env.split(',') {
                let parts: Vec<&str> = persona_str.split(':').collect();
                if parts.len() >= 4 {
                    let id = parts[0].trim().to_string();
                    let name = parts[1].trim().to_string();
                    let focus_areas = parts[2].trim().split(';').map(|s| s.trim().to_string()).collect::<Vec<String>>();
                    let description = parts[3].trim().to_string();

                    // Optional prompt template
                    let prompt_template = if parts.len() > 4 {
                        Some(parts[4].trim().to_string())
                    } else {
                        None
                    };

                    // Create and add the persona
                    let persona = Persona::new(id.clone(), name.clone(), focus_areas.clone(), description.clone(), prompt_template.clone());
                    self.personas.insert(id.clone(), persona);

                    tracing::info!("Added persona from environment variable: id={}, name={}, focus_areas={}",
                        id, name, focus_areas.join(", "));
                } else {
                    tracing::warn!("Invalid persona format in QITOPS_PERSONAS: {}", persona_str);
                }
            }

            // Save the updated configuration
            self.save_config()?;
        }

        // Check for individual persona environment variables
        // Format: QITOPS_PERSONA_<ID>="name:focus1;focus2:description[:prompt_template]"
        let mut found_persona_vars = false;
        for (key, value) in std::env::vars() {
            if key.starts_with("QITOPS_PERSONA_") {
                if !found_persona_vars {
                    tracing::info!("Loading personas from individual environment variables");
                    found_persona_vars = true;
                }

                let id = key.strip_prefix("QITOPS_PERSONA_").unwrap().to_lowercase();
                let parts: Vec<&str> = value.split(':').collect();

                if parts.len() >= 3 {
                    let name = parts[0].trim().to_string();
                    let focus_areas = parts[1].trim().split(';').map(|s| s.trim().to_string()).collect::<Vec<String>>();
                    let description = parts[2].trim().to_string();

                    // Optional prompt template
                    let prompt_template = if parts.len() > 3 {
                        Some(parts[3].trim().to_string())
                    } else {
                        None
                    };

                    // Create and add the persona
                    let persona = Persona::new(id.clone(), name.clone(), focus_areas.clone(), description.clone(), prompt_template.clone());
                    self.personas.insert(id.clone(), persona);

                    tracing::info!("Added persona from environment variable {}: id={}, name={}, focus_areas={}",
                        key, id, name, focus_areas.join(", "));
                } else {
                    tracing::warn!("Invalid persona format in {}: {}", key, value);
                }
            }
        }

        // Check for default persona environment variable
        if let Ok(default_persona) = std::env::var("QITOPS_DEFAULT_PERSONA") {
            // Ensure the persona exists
            if self.personas.contains_key(&default_persona) {
                // We don't need to do anything here, as the default persona is specified when using the personas
                tracing::info!("Default persona set to: {}", default_persona);
            } else {
                tracing::warn!("Default persona '{}' specified in environment variable QITOPS_DEFAULT_PERSONA does not exist", default_persona);
            }
        }

        // Check for default personas environment variable
        if let Ok(default_personas) = std::env::var("QITOPS_DEFAULT_PERSONAS") {
            tracing::info!("Found default personas in environment variable: {}", default_personas);

            // Validate that all personas exist
            for persona_id in default_personas.split(',').map(|s| s.trim()) {
                if !self.personas.contains_key(persona_id) {
                    tracing::warn!("Persona '{}' specified in QITOPS_DEFAULT_PERSONAS does not exist", persona_id);
                }
            }
        }

        Ok(())
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
