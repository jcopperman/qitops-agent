use anyhow::Result;
use qitops_agent::agent::{SessionAgent, AgentStatus};
use qitops_agent::agent::traits::Agent;
use qitops_agent::llm::{LlmRouter, ConfigManager, LlmConfig, LlmProvider};
use std::collections::HashMap;

#[tokio::test]
async fn test_session_agent_creation() -> Result<()> {
    // Create a mock LLM router
    let mut config = LlmConfig::default();
    config.providers.insert(
        "mock".to_string(),
        LlmProvider {
            provider_type: "mock".to_string(),
            api_key: None,
            api_base: None,
            model: Some("mock-model".to_string()),
            is_default: true,
        },
    );
    let router = LlmRouter::new(config).await?;

    // Create a session agent
    let agent = SessionAgent::new(
        "Test Session".to_string(),
        Some("exploratory".to_string()),
        "Test App".to_string(),
        vec!["Test objective 1".to_string(), "Test objective 2".to_string()],
        Some(vec!["documentation".to_string()]),
        Some(vec!["tester".to_string()]),
        router,
    ).await?;

    // Verify the agent was created successfully
    assert_eq!(agent.name(), "session");
    assert!(!agent.description().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_session_agent_with_empty_name() -> Result<()> {
    // Create a mock LLM router
    let mut config = LlmConfig::default();
    config.providers.insert(
        "mock".to_string(),
        LlmProvider {
            provider_type: "mock".to_string(),
            api_key: None,
            api_base: None,
            model: Some("mock-model".to_string()),
            is_default: true,
        },
    );
    let router = LlmRouter::new(config).await?;

    // Try to create a session agent with an empty name
    let result = SessionAgent::new(
        "".to_string(),
        Some("exploratory".to_string()),
        "Test App".to_string(),
        vec![],
        None,
        None,
        router,
    ).await;

    // Verify the agent creation failed
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("name cannot be empty"));

    Ok(())
}

#[tokio::test]
async fn test_session_agent_with_empty_application() -> Result<()> {
    // Create a mock LLM router
    let mut config = LlmConfig::default();
    config.providers.insert(
        "mock".to_string(),
        LlmProvider {
            provider_type: "mock".to_string(),
            api_key: None,
            api_base: None,
            model: Some("mock-model".to_string()),
            is_default: true,
        },
    );
    let router = LlmRouter::new(config).await?;

    // Try to create a session agent with an empty application
    let result = SessionAgent::new(
        "Test Session".to_string(),
        Some("exploratory".to_string()),
        "".to_string(),
        vec![],
        None,
        None,
        router,
    ).await;

    // Verify the agent creation failed
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("application name cannot be empty"));

    Ok(())
}

#[tokio::test]
async fn test_session_agent_with_invalid_session_type() -> Result<()> {
    // Create a mock LLM router
    let mut config = LlmConfig::default();
    config.providers.insert(
        "mock".to_string(),
        LlmProvider {
            provider_type: "mock".to_string(),
            api_key: None,
            api_base: None,
            model: Some("mock-model".to_string()),
            is_default: true,
        },
    );
    let router = LlmRouter::new(config).await?;

    // Try to create a session agent with an invalid session type
    let result = SessionAgent::new(
        "Test Session".to_string(),
        Some("invalid".to_string()),
        "Test App".to_string(),
        vec![],
        None,
        None,
        router,
    ).await;

    // Verify the agent creation failed
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown session type"));

    Ok(())
}

#[tokio::test]
async fn test_session_agent_init() -> Result<()> {
    // Create a mock LLM router
    let mut config = LlmConfig::default();
    config.providers.insert(
        "mock".to_string(),
        LlmProvider {
            provider_type: "mock".to_string(),
            api_key: None,
            api_base: None,
            model: Some("mock-model".to_string()),
            is_default: true,
        },
    );
    let router = LlmRouter::new(config).await?;

    // Create a session agent
    let mut agent = SessionAgent::new(
        "Test Session".to_string(),
        Some("exploratory".to_string()),
        "Test App".to_string(),
        vec!["Test objective".to_string()],
        None,
        None,
        router,
    ).await?;

    // Initialize the agent
    let result = agent.init();

    // Verify the agent was initialized successfully
    assert!(result.is_ok());

    Ok(())
}
