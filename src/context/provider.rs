use anyhow::Result;

use crate::source::SourceManager;
use crate::persona::PersonaManager;

/// Context provider for LLM prompts
pub struct ContextProvider {
    /// Source manager
    source_manager: SourceManager,

    /// Persona manager
    persona_manager: PersonaManager,
}

impl ContextProvider {
    /// Create a new context provider
    pub fn new() -> Result<Self> {
        let source_manager = SourceManager::new()?;
        let persona_manager = PersonaManager::new()?;

        Ok(Self {
            source_manager,
            persona_manager,
        })
    }

    /// Get context from sources and personas
    pub fn get_context(&self, sources: Option<&[String]>, personas: Option<&[String]>) -> Result<String> {
        let mut context = String::new();

        // Add source content if available
        if let Some(source_ids) = sources {
            if !source_ids.is_empty() {
                let source_content = self.source_manager.get_content_for_sources(source_ids)?;
                if !source_content.is_empty() {
                    context.push_str("# Context from Sources\n\n");
                    context.push_str(&source_content);
                    context.push_str("\n\n");
                }
            }
        }

        // Add persona prompts if available
        if let Some(persona_ids) = personas {
            if !persona_ids.is_empty() {
                let persona_content = self.persona_manager.get_prompt_for_personas(persona_ids)?;
                if !persona_content.is_empty() {
                    context.push_str("# Persona Guidance\n\n");
                    context.push_str(&persona_content);
                    context.push_str("\n\n");
                }
            }
        }

        Ok(context)
    }

    /// Get default sources for a command
    pub fn get_default_sources(&self, _command: &str) -> Result<Vec<String>> {
        // Try to get from environment variable
        if let Ok(default_sources) = std::env::var("QITOPS_DEFAULT_SOURCES") {
            return Ok(default_sources.split(',')
                .map(|s| s.trim().to_string())
                .collect());
        }

        // No default sources
        Ok(Vec::new())
    }

    /// Get default personas for a command
    pub fn get_default_personas(&self, _command: &str) -> Result<Vec<String>> {
        // Try to get from environment variable
        if let Ok(default_personas) = std::env::var("QITOPS_DEFAULT_PERSONAS") {
            return Ok(default_personas.split(',')
                .map(|s| s.trim().to_string())
                .collect());
        }

        // No default personas
        Ok(Vec::new())
    }
}
