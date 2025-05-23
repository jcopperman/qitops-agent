use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

/// LLM client error
#[derive(Debug, Error)]
pub enum LlmError {
    /// API error
    #[error("API error: {0}")]
    ApiError(String),

    /// Rate limit error
    #[error("Rate limit error: {0}")]
    RateLimitError(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Provider not available
    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Message role for chat models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
        }
    }
}

/// Chat message for LLM requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender
    pub role: MessageRole,

    /// Content of the message
    pub content: String,
}

/// LLM request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Messages to send to the LLM (for chat models)
    pub messages: Vec<ChatMessage>,

    /// Maximum number of tokens to generate
    pub max_tokens: usize,

    /// Temperature for generation
    pub temperature: f32,

    /// Model to use
    pub model: String,

    /// Top-p sampling
    #[serde(default = "default_top_p")]
    pub top_p: f32,

    /// Frequency penalty
    #[serde(default = "default_frequency_penalty")]
    pub frequency_penalty: f32,

    /// Presence penalty
    #[serde(default = "default_presence_penalty")]
    pub presence_penalty: f32,

    /// Stop sequences
    #[serde(default)]
    pub stop: Vec<String>,

    /// Whether to use cache
    #[serde(default = "default_use_cache")]
    pub use_cache: bool,

    /// Additional request options
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

/// Default top-p value
fn default_top_p() -> f32 {
    1.0
}

/// Default frequency penalty value
fn default_frequency_penalty() -> f32 {
    0.0
}

/// Default presence penalty value
fn default_presence_penalty() -> f32 {
    0.0
}

/// Default use cache value
fn default_use_cache() -> bool {
    true
}

impl LlmRequest {
    /// Create a new LLM request with a single user message
    pub fn new(content: String, model: String) -> Self {
        Self {
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content,
            }],
            max_tokens: 1024,
            temperature: 0.7,
            model,
            top_p: default_top_p(),
            frequency_penalty: default_frequency_penalty(),
            presence_penalty: default_presence_penalty(),
            stop: Vec::new(),
            use_cache: default_use_cache(),
            options: HashMap::new(),
        }
    }

    /// Add a system message at the beginning of the conversation
    pub fn with_system_message(mut self, content: String) -> Self {
        self.messages.insert(0, ChatMessage {
            role: MessageRole::System,
            content,
        });
        self
    }

    /// Set the maximum number of tokens to generate
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the temperature for generation
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set the top-p sampling
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }

    /// Set the frequency penalty
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = frequency_penalty;
        self
    }

    /// Set the presence penalty
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = presence_penalty;
        self
    }

    /// Add a stop sequence
    pub fn with_stop(mut self, stop: String) -> Self {
        self.stop.push(stop);
        self
    }

    /// Set whether to use cache
    pub fn with_cache(mut self, use_cache: bool) -> Self {
        self.use_cache = use_cache;
        self
    }

    /// Add additional context to the system message
    pub fn with_additional_context(mut self, context: String) -> Self {
        // Check if there's already a system message
        if let Some(system_message) = self.messages.iter_mut().find(|m| m.role == MessageRole::System) {
            // Append the context to the existing system message
            system_message.content = format!("{}

{}", system_message.content, context);
        } else {
            // Add a new system message with the context
            self.messages.insert(0, ChatMessage {
                role: MessageRole::System,
                content: context,
            });
        }
        self
    }

    /// Add an option
    pub fn with_option(mut self, key: &str, value: serde_json::Value) -> Self {
        self.options.insert(key.to_string(), value);
        self
    }
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Generated text
    pub text: String,

    /// Number of tokens used (if available)
    pub tokens_used: Option<usize>,

    /// Model used
    pub model: String,

    /// Provider that generated the response
    pub provider: String,

    /// Response timestamp
    #[serde(default = "default_timestamp")]
    pub timestamp: u64,

    /// Response latency in milliseconds
    pub latency_ms: Option<u64>,

    /// Whether the response was cached
    #[serde(default)]
    pub cached: bool,

    /// Additional response metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Default timestamp value
fn default_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl LlmResponse {
    /// Create a new LLM response
    pub fn new(text: String, model: String, provider: String) -> Self {
        Self {
            text,
            tokens_used: None,
            model,
            provider,
            timestamp: default_timestamp(),
            latency_ms: None,
            cached: false,
            metadata: HashMap::new(),
        }
    }

    /// Set the number of tokens used
    pub fn with_tokens(mut self, tokens: usize) -> Self {
        self.tokens_used = Some(tokens);
        self
    }

    /// Set the response latency
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    /// Mark the response as cached
    pub fn with_cached(mut self, cached: bool) -> Self {
        self.cached = cached;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type
    pub provider_type: String,

    /// API key (if needed)
    pub api_key: Option<String>,

    /// API base URL (if custom)
    pub api_base: Option<String>,

    /// Default model to use
    pub default_model: String,

    /// Additional provider-specific configuration
    #[serde(default)]
    pub options: HashMap<String, String>,
}

/// LLM router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// List of providers in priority order
    pub providers: Vec<ProviderConfig>,

    /// Default provider to use
    pub default_provider: String,

    /// Task-specific provider mappings
    #[serde(default)]
    pub task_providers: HashMap<String, String>,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether to enable caching
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,

    /// Whether to use disk cache
    #[serde(default = "default_cache_disk")]
    pub use_disk: bool,
}

/// Default cache enabled value
fn default_cache_enabled() -> bool {
    true
}

/// Default cache TTL value
fn default_cache_ttl() -> u64 {
    3600 // 1 hour
}

/// Default cache disk value
fn default_cache_disk() -> bool {
    true
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            ttl_seconds: default_cache_ttl(),
            use_disk: default_cache_disk(),
        }
    }
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            providers: vec![
                ProviderConfig {
                    provider_type: "ollama".to_string(),
                    api_key: None,
                    api_base: Some("http://localhost:11434".to_string()),
                    default_model: "mistral".to_string(),
                    options: HashMap::new(),
                },
                ProviderConfig {
                    provider_type: "openai".to_string(),
                    api_key: None,
                    api_base: None,
                    default_model: "gpt-3.5-turbo".to_string(),
                    options: HashMap::new(),
                },
            ],
            default_provider: "ollama".to_string(),
            task_providers: HashMap::new(),
            cache: CacheConfig::default(),
        }
    }
}

/// LLM client trait
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a request to the LLM
    async fn send(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Get the client name
    fn name(&self) -> &str;

    /// Check if the client is available
    async fn is_available(&self) -> bool;
}

// LLM client implementations are now in providers.rs

/// LLM router that manages multiple LLM clients
pub struct LlmRouter {
    clients: HashMap<String, Arc<dyn LlmClient>>,
    config: RouterConfig,
    default_client: String,
    cache: Option<Arc<Mutex<crate::llm::cache::ResponseCache>>>,
}

impl LlmRouter {
    /// Create a new LLM router with the given configuration
    pub async fn new(config: RouterConfig) -> Result<Self> {
        let mut clients = HashMap::new();
        let mut default_client = config.default_provider.clone();
        let mut any_client_available = false;

        // Initialize all providers
        for provider_config in &config.providers {
            // Try to initialize the provider
            let client_result = match provider_config.provider_type.as_str() {
                "openai" => crate::llm::providers::OpenAiClient::new(provider_config).map(|c| Arc::new(c) as Arc<dyn LlmClient>),
                "ollama" => crate::llm::providers::OllamaClient::new(provider_config).map(|c| Arc::new(c) as Arc<dyn LlmClient>),
                "anthropic" => crate::llm::providers::AnthropicClient::new(provider_config).map(|c| Arc::new(c) as Arc<dyn LlmClient>),
                _ => {
                    eprintln!("Warning: Unknown provider type: {}", provider_config.provider_type);
                    continue;
                }
            };

            // If initialization failed, log the error and continue
            if let Err(e) = &client_result {
                eprintln!("Warning: Failed to initialize {} client: {}", provider_config.provider_type, e);
                continue;
            }

            // Unwrap the client (safe because we checked for errors)
            let client = client_result.unwrap();
            let provider_name = client.name().to_string();
            clients.insert(provider_name.clone(), client.clone());

            // Check if this client is available
            if client.is_available().await {
                any_client_available = true;

                // If this is the default provider, or we haven't found an available client yet,
                // set this as the default
                if provider_name == config.default_provider || default_client.is_empty() {
                    default_client = provider_name;
                }
            }
        }

        if !any_client_available {
            return Err(anyhow!("No LLM providers are available"));
        }

        // Initialize cache if enabled
        let cache = if config.cache.enabled {
            match crate::llm::cache::ResponseCache::new(config.cache.ttl_seconds, config.cache.use_disk) {
                Ok(cache) => Some(Arc::new(Mutex::new(cache))),
                Err(e) => {
                    eprintln!("Warning: Failed to initialize cache: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            clients,
            config,
            default_client,
            cache,
        })
    }

    /// Send a request to the LLM using the appropriate client
    pub async fn send(&self, request: LlmRequest, task: Option<&str>) -> Result<LlmResponse> {
        // Determine which provider to use based on the task
        let provider = if let Some(task) = task {
            self.config.task_providers.get(task)
                .map(|s| s.as_str())
                .unwrap_or(&self.default_client)
        } else {
            &self.default_client
        };

        // Try to get the client
        let client = self.clients.get(provider)
            .ok_or_else(|| anyhow!("Provider not found: {}", provider))?;

        // Check cache if enabled and request allows caching
        if request.use_cache && self.cache.is_some() {
            if let Some(cache) = &self.cache {
                let cache_guard = cache.lock().await;
                if let Some(cached_response) = cache_guard.get(&request, provider) {
                    return Ok(cached_response.with_cached(true));
                }
            }
        }

        // Check if the client is available
        if !client.is_available().await {
            // If not, try to find an available client
            for (name, client) in &self.clients {
                if client.is_available().await {
                    let start_time = std::time::Instant::now();
                    let response = client.send(request.clone()).await?;
                    let latency = start_time.elapsed().as_millis() as u64;

                    // Add latency to response
                    let response = response.with_latency(latency);

                    return Ok(response);
                }
            }

            return Err(anyhow!("No LLM providers are available"));
        }

        // Measure latency
        let start_time = std::time::Instant::now();

        // Send the request
        let response = client.send(request.clone()).await?;

        // Calculate latency
        let latency = start_time.elapsed().as_millis() as u64;

        // Add latency to response
        let response = response.with_latency(latency);

        // Cache the response if caching is enabled
        if request.use_cache && self.cache.is_some() {
            if let Some(cache) = &self.cache {
                let mut cache_guard = cache.lock().await;
                let _ = cache_guard.put(&request, provider, response.clone());
            }
        }

        Ok(response)
    }

    /// Get the available providers
    pub async fn available_providers(&self) -> Vec<String> {
        let mut available = Vec::new();

        for (name, client) in &self.clients {
            if client.is_available().await {
                available.push(name.clone());
            }
        }

        available
    }

    /// Get the default provider
    pub fn default_provider(&self) -> &str {
        &self.default_client
    }

    /// Get the default model for a provider
    pub fn default_model_for_provider(&self, provider: &str) -> Option<String> {
        self.config.providers.iter()
            .find(|p| p.provider_type == provider)
            .map(|p| p.default_model.clone())
    }

    /// Get the default model for the default provider
    pub fn default_model(&self) -> Option<String> {
        self.default_model_for_provider(&self.default_client)
    }

    /// Get a client by provider name
    pub fn get_client(&self, provider: &str) -> Option<&Arc<dyn LlmClient>> {
        self.clients.get(provider)
    }
}
