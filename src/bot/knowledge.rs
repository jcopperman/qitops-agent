use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Command documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDoc {
    /// Command name
    pub name: String,
    
    /// Command description
    pub description: String,
    
    /// Command usage
    pub usage: String,
    
    /// Command examples
    pub examples: Vec<String>,
    
    /// Command options
    pub options: HashMap<String, String>,
}

/// Configuration documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDoc {
    /// Configuration file path
    pub file_path: String,
    
    /// Configuration sections
    pub sections: HashMap<String, String>,
    
    /// Configuration examples
    pub examples: Vec<String>,
}

/// FAQ entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaqEntry {
    /// Question
    pub question: String,
    
    /// Answer
    pub answer: String,
    
    /// Tags
    pub tags: Vec<String>,
}

/// Example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Example title
    pub title: String,
    
    /// Example description
    pub description: String,
    
    /// Example code
    pub code: String,
    
    /// Example tags
    pub tags: Vec<String>,
}

/// Knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Command documentation
    pub commands: HashMap<String, CommandDoc>,
    
    /// Configuration documentation
    pub config: ConfigDoc,
    
    /// FAQ
    pub faq: Vec<FaqEntry>,
    
    /// Examples
    pub examples: Vec<Example>,
}

impl KnowledgeBase {
    /// Load knowledge base from files
    pub fn load(path: &Path) -> Result<Self> {
        // Check if the path exists
        if !path.exists() {
            return Err(anyhow!("Knowledge base path does not exist: {}", path.display()));
        }
        
        // Load command documentation
        let commands_path = path.join("commands.json");
        let commands = if commands_path.exists() {
            let commands_str = fs::read_to_string(&commands_path)?;
            serde_json::from_str(&commands_str)?
        } else {
            HashMap::new()
        };
        
        // Load configuration documentation
        let config_path = path.join("config.json");
        let config = if config_path.exists() {
            let config_str = fs::read_to_string(&config_path)?;
            serde_json::from_str(&config_str)?
        } else {
            ConfigDoc {
                file_path: "~/.config/qitops/config.json".to_string(),
                sections: HashMap::new(),
                examples: Vec::new(),
            }
        };
        
        // Load FAQ
        let faq_path = path.join("faq.json");
        let faq = if faq_path.exists() {
            let faq_str = fs::read_to_string(&faq_path)?;
            serde_json::from_str(&faq_str)?
        } else {
            Vec::new()
        };
        
        // Load examples
        let examples_path = path.join("examples.json");
        let examples = if examples_path.exists() {
            let examples_str = fs::read_to_string(&examples_path)?;
            serde_json::from_str(&examples_str)?
        } else {
            Vec::new()
        };
        
        Ok(Self {
            commands,
            config,
            faq,
            examples,
        })
    }
    
    /// Get documentation for a command
    pub fn get_command_doc(&self, command: &str) -> Option<&CommandDoc> {
        self.commands.get(command)
    }
    
    /// Search for examples
    pub fn search_examples(&self, query: &str) -> Vec<&Example> {
        // Simple search implementation
        self.examples.iter()
            .filter(|example| example.description.contains(query) || example.tags.iter().any(|tag| tag.contains(query)))
            .collect()
    }
    
    /// Search for FAQ entries
    pub fn search_faq(&self, query: &str) -> Vec<&FaqEntry> {
        // Simple search implementation
        self.faq.iter()
            .filter(|entry| entry.question.contains(query) || entry.answer.contains(query) || entry.tags.iter().any(|tag| tag.contains(query)))
            .collect()
    }
    
    /// Get configuration documentation
    pub fn get_config_doc(&self) -> &ConfigDoc {
        &self.config
    }
}
