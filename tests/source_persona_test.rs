use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

use qitops::source::{Source, SourceManager, SourceType};
use qitops::persona::{Persona, PersonaManager};
use qitops::config::QitOpsConfigManager;

#[tokio::test]
async fn test_source_manager() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();
    
    // Create a test source file
    let source_path = temp_path.join("requirements.md");
    fs::write(&source_path, "# Project Requirements\n\n- Requirement 1\n- Requirement 2")?;
    
    // Set up environment variables for testing
    env::set_var("HOME", temp_path.to_str().unwrap());
    env::set_var("APPDATA", temp_path.to_str().unwrap());
    
    // Create a source manager
    let mut source_manager = SourceManager::new()?;
    
    // Add a source
    let source = Source::new(
        "requirements".to_string(),
        SourceType::Requirements,
        source_path.clone(),
        Some("Project requirements".to_string()),
    );
    
    source_manager.add_source(source)?;
    
    // Get the source
    let retrieved_source = source_manager.get_source("requirements")
        .ok_or_else(|| anyhow::anyhow!("Source not found"))?;
        
    assert_eq!(retrieved_source.id, "requirements");
    assert_eq!(retrieved_source.source_type, SourceType::Requirements);
    assert_eq!(retrieved_source.path, source_path);
    assert_eq!(retrieved_source.description, Some("Project requirements".to_string()));
    
    // Get source content
    let content = retrieved_source.get_content()?;
    assert_eq!(content, "# Project Requirements\n\n- Requirement 1\n- Requirement 2");
    
    // List sources
    let sources = source_manager.list_sources();
    assert_eq!(sources.len(), 1);
    
    // Remove the source
    source_manager.remove_source("requirements")?;
    
    // Check that the source was removed
    let sources = source_manager.list_sources();
    assert_eq!(sources.len(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_persona_manager() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();
    
    // Set up environment variables for testing
    env::set_var("HOME", temp_path.to_str().unwrap());
    env::set_var("APPDATA", temp_path.to_str().unwrap());
    
    // Create a persona manager
    let mut persona_manager = PersonaManager::new()?;
    
    // Add a persona
    let persona = Persona::new(
        "security-analyst".to_string(),
        "Security Analyst".to_string(),
        vec!["security".to_string(), "vulnerabilities".to_string(), "compliance".to_string()],
        "Focus on security vulnerabilities and compliance issues.".to_string(),
        None,
    );
    
    persona_manager.add_persona(persona)?;
    
    // Get the persona
    let retrieved_persona = persona_manager.get_persona("security-analyst")
        .ok_or_else(|| anyhow::anyhow!("Persona not found"))?;
        
    assert_eq!(retrieved_persona.id, "security-analyst");
    assert_eq!(retrieved_persona.name, "Security Analyst");
    assert_eq!(retrieved_persona.focus_areas, vec!["security", "vulnerabilities", "compliance"]);
    assert_eq!(retrieved_persona.description, "Focus on security vulnerabilities and compliance issues.");
    
    // Get prompt for persona
    let prompt = retrieved_persona.get_prompt();
    assert!(prompt.contains("Security Analyst"));
    assert!(prompt.contains("security, vulnerabilities, compliance"));
    
    // List personas
    let personas = persona_manager.list_personas();
    assert!(personas.len() > 0); // There are default personas
    
    // Remove the persona
    persona_manager.remove_persona("security-analyst")?;
    
    // Check that the persona was removed
    let retrieved_persona = persona_manager.get_persona("security-analyst");
    assert!(retrieved_persona.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_config_manager() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();
    
    // Set up environment variables for testing
    env::set_var("HOME", temp_path.to_str().unwrap());
    env::set_var("APPDATA", temp_path.to_str().unwrap());
    
    // Create a config manager
    let config_manager = QitOpsConfigManager::new()?;
    
    // Get default sources for a command
    let default_sources = config_manager.get_default_sources("test-gen");
    
    // Get default personas for a command
    let default_personas = config_manager.get_default_personas("test-gen");
    
    // Set environment variables for default sources and personas
    env::set_var("QITOPS_DEFAULT_SOURCES", "requirements,standards");
    env::set_var("QITOPS_DEFAULT_PERSONAS", "security-analyst");
    
    // Create a new config manager to pick up the environment variables
    let config_manager = QitOpsConfigManager::new()?;
    
    // Get default sources for a command
    let default_sources = config_manager.get_default_sources("test-gen");
    assert_eq!(default_sources, vec!["requirements", "standards"]);
    
    // Get default personas for a command
    let default_personas = config_manager.get_default_personas("test-gen");
    assert_eq!(default_personas, vec!["security-analyst"]);
    
    Ok(())
}

#[tokio::test]
async fn test_environment_variables() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();
    
    // Set up environment variables for testing
    env::set_var("HOME", temp_path.to_str().unwrap());
    env::set_var("APPDATA", temp_path.to_str().unwrap());
    
    // Set environment variables for sources
    env::set_var("QITOPS_SOURCES", "requirements:requirements:docs/requirements.md:Project requirements,standards:standard:docs/standards.md:Coding standards");
    
    // Set environment variables for personas
    env::set_var("QITOPS_PERSONAS", "security-analyst:Security Analyst:security;vulnerabilities;compliance:Focus on security vulnerabilities and compliance issues.");
    
    // Create a source manager
    let source_manager = SourceManager::new()?;
    
    // Check that the sources were loaded from environment variables
    let sources = source_manager.list_sources();
    assert!(sources.len() >= 2);
    
    let requirements_source = source_manager.get_source("requirements");
    assert!(requirements_source.is_some());
    
    let standards_source = source_manager.get_source("standards");
    assert!(standards_source.is_some());
    
    // Create a persona manager
    let persona_manager = PersonaManager::new()?;
    
    // Check that the personas were loaded from environment variables
    let security_analyst = persona_manager.get_persona("security-analyst");
    assert!(security_analyst.is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_source_content_for_sources() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();
    
    // Create test source files
    let requirements_path = temp_path.join("requirements.md");
    fs::write(&requirements_path, "# Project Requirements\n\n- Requirement 1\n- Requirement 2")?;
    
    let standards_path = temp_path.join("standards.md");
    fs::write(&standards_path, "# Coding Standards\n\n- Standard 1\n- Standard 2")?;
    
    // Set up environment variables for testing
    env::set_var("HOME", temp_path.to_str().unwrap());
    env::set_var("APPDATA", temp_path.to_str().unwrap());
    
    // Create a source manager
    let mut source_manager = SourceManager::new()?;
    
    // Add sources
    let requirements_source = Source::new(
        "requirements".to_string(),
        SourceType::Requirements,
        requirements_path.clone(),
        Some("Project requirements".to_string()),
    );
    
    let standards_source = Source::new(
        "standards".to_string(),
        SourceType::Standard,
        standards_path.clone(),
        Some("Coding standards".to_string()),
    );
    
    source_manager.add_source(requirements_source)?;
    source_manager.add_source(standards_source)?;
    
    // Get content for sources
    let content = source_manager.get_content_for_sources(&["requirements".to_string(), "standards".to_string()])?;
    
    // Check that the content contains both sources
    assert!(content.contains("# Source: requirements (requirements)"));
    assert!(content.contains("# Project Requirements"));
    assert!(content.contains("# Source: standards (standard)"));
    assert!(content.contains("# Coding Standards"));
    
    Ok(())
}

#[tokio::test]
async fn test_prompt_for_personas() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();
    
    // Set up environment variables for testing
    env::set_var("HOME", temp_path.to_str().unwrap());
    env::set_var("APPDATA", temp_path.to_str().unwrap());
    
    // Create a persona manager
    let mut persona_manager = PersonaManager::new()?;
    
    // Add personas
    let security_analyst = Persona::new(
        "security-analyst".to_string(),
        "Security Analyst".to_string(),
        vec!["security".to_string(), "vulnerabilities".to_string(), "compliance".to_string()],
        "Focus on security vulnerabilities and compliance issues.".to_string(),
        None,
    );
    
    let performance_engineer = Persona::new(
        "performance-engineer".to_string(),
        "Performance Engineer".to_string(),
        vec!["performance".to_string(), "optimization".to_string(), "scalability".to_string()],
        "Focus on performance implications and bottlenecks.".to_string(),
        None,
    );
    
    persona_manager.add_persona(security_analyst)?;
    persona_manager.add_persona(performance_engineer)?;
    
    // Get prompt for personas
    let prompt = persona_manager.get_prompt_for_personas(&["security-analyst".to_string(), "performance-engineer".to_string()])?;
    
    // Check that the prompt contains both personas
    assert!(prompt.contains("Security Analyst"));
    assert!(prompt.contains("security, vulnerabilities, compliance"));
    assert!(prompt.contains("Performance Engineer"));
    assert!(prompt.contains("performance, optimization, scalability"));
    
    Ok(())
}
