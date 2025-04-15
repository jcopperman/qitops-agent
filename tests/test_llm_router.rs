use qitops_agent::llm::{LlmRouter, RouterConfig, ProviderConfig, LlmRequest};
use qitops_agent::llm::client::{MessageRole, CacheConfig};
use std::collections::HashMap;
use tokio_test::block_on;

#[test]
fn test_router_initialization() {
    // Create a basic config with Ollama
    let mut config = RouterConfig {
        providers: vec![
            ProviderConfig {
                provider_type: "ollama".to_string(),
                api_key: None,
                api_base: Some("http://localhost:11434".to_string()),
                default_model: "mistral".to_string(),
                options: HashMap::new(),
            }
        ],
        default_provider: "ollama".to_string(),
        cache: CacheConfig {
            enabled: true,
            ttl_seconds: 3600,
            use_disk: true,
        },
        task_providers: HashMap::new(),
    };

    // Test initialization with valid config
    let router_result = block_on(LlmRouter::new(config.clone()));

    // If Ollama is not running, this will fail, so we don't assert success
    if router_result.is_ok() {
        let router = router_result.unwrap();
        assert_eq!(router.default_provider(), "ollama");
        assert!(router.default_model_for_provider("ollama").is_some());
        assert_eq!(router.default_model_for_provider("ollama").unwrap(), "mistral");
    }

    // Test initialization with empty providers
    config.providers = vec![];
    let router_result = block_on(LlmRouter::new(config.clone()));
    assert!(router_result.is_err());

    // Test initialization with invalid provider
    config.providers = vec![
        ProviderConfig {
            provider_type: "unknown".to_string(),
            api_key: None,
            api_base: None,
            default_model: "unknown".to_string(),
            options: HashMap::new(),
        }
    ];
    let router_result = block_on(LlmRouter::new(config.clone()));
    assert!(router_result.is_err());
}

#[test]
fn test_llm_request_building() {
    // Create a basic request
    let request = LlmRequest::new("Test content".to_string(), "test-model".to_string());

    // Test basic properties
    assert_eq!(request.model, "test-model");
    assert_eq!(request.messages.len(), 1);
    assert_eq!(request.messages[0].role, MessageRole::User);
    assert_eq!(request.messages[0].content, "Test content");

    // Test with system message
    let request = request.with_system_message("System instruction".to_string());
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].role, MessageRole::System);
    assert_eq!(request.messages[0].content, "System instruction");

    // Test with temperature
    let request = request.with_temperature(0.5);
    assert_eq!(request.temperature, 0.5);

    // Test with max tokens
    let request = request.with_max_tokens(2000);
    assert_eq!(request.max_tokens, 2000);
}

#[test]
fn test_router_config_validation() {
    // Test OpenAI config without API key
    let config = RouterConfig {
        providers: vec![
            ProviderConfig {
                provider_type: "openai".to_string(),
                api_key: None,
                api_base: None,
                default_model: "gpt-4".to_string(),
                options: HashMap::new(),
            }
        ],
        default_provider: "openai".to_string(),
        cache: CacheConfig {
            enabled: false,
            ttl_seconds: 3600,
            use_disk: false,
        },
        task_providers: HashMap::new(),
    };

    let router_result = block_on(LlmRouter::new(config.clone()));
    assert!(router_result.is_err());

    // Test Ollama config without API base
    let config = RouterConfig {
        providers: vec![
            ProviderConfig {
                provider_type: "ollama".to_string(),
                api_key: None,
                api_base: None,
                default_model: "mistral".to_string(),
                options: HashMap::new(),
            }
        ],
        default_provider: "ollama".to_string(),
        cache: CacheConfig {
            enabled: false,
            ttl_seconds: 3600,
            use_disk: false,
        },
        task_providers: HashMap::new(),
    };

    let router_result = block_on(LlmRouter::new(config.clone()));
    assert!(router_result.is_err());
}
