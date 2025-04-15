use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::bot::ChatMessage;

/// Chat session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    /// Session name
    pub name: String,
    
    /// Session timestamp
    pub timestamp: u64,
    
    /// Chat history
    pub history: Vec<ChatMessage>,
    
    /// System prompt
    pub system_prompt: String,
}

impl ChatSession {
    /// Create a new chat session
    pub fn new(name: String, system_prompt: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            name,
            timestamp,
            history: Vec::new(),
            system_prompt,
        }
    }
    
    /// Generate a default session name
    pub fn generate_default_name() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        format!("session_{}", timestamp)
    }
}
