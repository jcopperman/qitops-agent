use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::llm::client::{LlmClient, LlmRequest, LlmResponse, MessageRole, ProviderConfig};

/// OpenAI LLM client
pub struct OpenAiClient {
    api_key: String,
    api_base: String,
    http_client: HttpClient,
}

impl OpenAiClient {
    /// Create a new OpenAI client
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let api_key = config.api_key.clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .context("OpenAI API key not found in config or OPENAI_API_KEY environment variable")?;

        let api_base = config.api_base.clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        Ok(Self {
            api_key,
            api_base,
            http_client: HttpClient::new(),
        })
    }
    
    /// Build the OpenAI API request
    async fn build_request(&self, request: &LlmRequest) -> Result<serde_json::Value> {
        // Convert our messages to OpenAI format
        let messages: Vec<serde_json::Value> = request.messages.iter().map(|msg| {
            json!({
                "role": match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                },
                "content": msg.content
            })
        }).collect();
        
        // Build the request body
        let mut body = json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
            "top_p": request.top_p,
            "frequency_penalty": request.frequency_penalty,
            "presence_penalty": request.presence_penalty,
        });
        
        // Add stop sequences if any
        if !request.stop.is_empty() {
            body["stop"] = json!(request.stop);
        }
        
        // Add any additional options
        for (key, value) in &request.options {
            body[key] = value.clone();
        }
        
        Ok(body)
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn send(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Check if API key is available
        if self.api_key.is_empty() {
            return Err(anyhow!("OpenAI API key not found in config or OPENAI_API_KEY environment variable"));
        }
        
        // Build the request body
        let body = self.build_request(&request).await?;
        
        // Send the request to the OpenAI API
        let url = format!("{}/chat/completions", self.api_base);
        
        let response = self.http_client.post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to OpenAI API: {}", e))?;
            
        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Could not read error response".to_string());
                
            return match status.as_u16() {
                401 => Err(anyhow!("Authentication error: {}", error_text)),
                429 => Err(anyhow!("Rate limit exceeded: {}", error_text)),
                500..=599 => Err(anyhow!("OpenAI server error: {}", error_text)),
                _ => Err(anyhow!("OpenAI API error ({}): {}", status, error_text)),
            };
        }
        
        // Parse the response
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse OpenAI API response: {}", e))?;
            
        // Extract the response text
        let choices = response_json["choices"].as_array()
            .ok_or_else(|| anyhow!("Invalid response format: 'choices' field is missing or not an array"))?;
            
        if choices.is_empty() {
            return Err(anyhow!("No completions returned from OpenAI API"));
        }
        
        let message = &choices[0]["message"];
        let content = message["content"].as_str()
            .ok_or_else(|| anyhow!("Invalid response format: 'content' field is missing or not a string"))?;
            
        // Extract token usage
        let usage = response_json["usage"].as_object();
        let tokens_used = usage
            .and_then(|u| u.get("total_tokens"))
            .and_then(|t| t.as_u64())
            .map(|t| t as usize);
            
        // Extract model info
        let model = response_json["model"].as_str()
            .unwrap_or(&request.model)
            .to_string();
            
        // Create the response
        let mut llm_response = LlmResponse::new(
            content.to_string(),
            model,
            self.name().to_string()
        );
        
        if let Some(tokens) = tokens_used {
            llm_response = llm_response.with_tokens(tokens);
        }
        
        Ok(llm_response)
    }

    fn name(&self) -> &str {
        "openai"
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Anthropic LLM client
pub struct AnthropicClient {
    api_key: String,
    api_base: String,
    http_client: HttpClient,
}

impl AnthropicClient {
    /// Create a new Anthropic client
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let api_key = config.api_key.clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .context("Anthropic API key not found in config or ANTHROPIC_API_KEY environment variable")?;

        let api_base = config.api_base.clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());

        Ok(Self {
            api_key,
            api_base,
            http_client: HttpClient::new(),
        })
    }
    
    /// Build the Anthropic API request
    async fn build_request(&self, request: &LlmRequest) -> Result<serde_json::Value> {
        // Convert our messages to Anthropic format
        // Anthropic uses a different format than OpenAI
        let mut system_prompt = String::new();
        let mut messages = Vec::new();
        
        // Extract system message if present
        for msg in &request.messages {
            match msg.role {
                MessageRole::System => {
                    system_prompt = msg.content.clone();
                },
                _ => {
                    messages.push(json!({
                        "role": match msg.role {
                            MessageRole::User => "user",
                            MessageRole::Assistant => "assistant",
                            _ => "user", // Default to user for other roles
                        },
                        "content": msg.content
                    }));
                }
            }
        }
        
        // Build the request body
        let mut body = json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
            "top_p": request.top_p,
        });
        
        // Add system prompt if present
        if !system_prompt.is_empty() {
            body["system"] = json!(system_prompt);
        }
        
        // Add stop sequences if any
        if !request.stop.is_empty() {
            body["stop_sequences"] = json!(request.stop);
        }
        
        // Add any additional options
        for (key, value) in &request.options {
            body[key] = value.clone();
        }
        
        Ok(body)
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn send(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Check if API key is available
        if self.api_key.is_empty() {
            return Err(anyhow!("Anthropic API key not found in config or ANTHROPIC_API_KEY environment variable"));
        }
        
        // Build the request body
        let body = self.build_request(&request).await?;
        
        // Send the request to the Anthropic API
        let url = format!("{}/v1/messages", self.api_base);
        
        let response = self.http_client.post(&url)
            .header("Content-Type", "application/json")
            .header("X-API-Key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Anthropic API: {}", e))?;
            
        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Could not read error response".to_string());
                
            return match status.as_u16() {
                401 => Err(anyhow!("Authentication error: {}", error_text)),
                429 => Err(anyhow!("Rate limit exceeded: {}", error_text)),
                500..=599 => Err(anyhow!("Anthropic server error: {}", error_text)),
                _ => Err(anyhow!("Anthropic API error ({}): {}", status, error_text)),
            };
        }
        
        // Parse the response
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse Anthropic API response: {}", e))?;
            
        // Extract the response text
        let content = response_json["content"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|first| first["text"].as_str())
            .ok_or_else(|| anyhow!("Invalid response format: 'content' field is missing or not properly formatted"))?;
            
        // Extract token usage if available
        let tokens_used = response_json["usage"]["input_tokens"].as_u64()
            .and_then(|input| {
                response_json["usage"]["output_tokens"].as_u64().map(|output| input + output)
            })
            .map(|t| t as usize);
            
        // Extract model info
        let model = response_json["model"].as_str()
            .unwrap_or(&request.model)
            .to_string();
            
        // Create the response
        let mut llm_response = LlmResponse::new(
            content.to_string(),
            model,
            self.name().to_string()
        );
        
        if let Some(tokens) = tokens_used {
            llm_response = llm_response.with_tokens(tokens);
        }
        
        Ok(llm_response)
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Ollama LLM client
pub struct OllamaClient {
    api_base: String,
    http_client: HttpClient,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let api_base = config.api_base.clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string());

        Ok(Self {
            api_base,
            http_client: HttpClient::new(),
        })
    }
    
    /// Build the Ollama API request
    async fn build_request(&self, request: &LlmRequest) -> Result<serde_json::Value> {
        // Convert our messages to Ollama format
        let mut prompt = String::new();
        
        // Ollama uses a simple prompt format, so we need to convert our messages
        for msg in &request.messages {
            match msg.role {
                MessageRole::System => {
                    prompt.push_str(&format!("System: {}\n\n", msg.content));
                },
                MessageRole::User => {
                    prompt.push_str(&format!("User: {}\n\n", msg.content));
                },
                MessageRole::Assistant => {
                    prompt.push_str(&format!("Assistant: {}\n\n", msg.content));
                },
            }
        }
        
        // Build the request body
        let mut body = json!({
            "model": request.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": request.temperature,
                "top_p": request.top_p,
                "num_predict": request.max_tokens,
            }
        });
        
        // Add any additional options
        for (key, value) in &request.options {
            body["options"][key] = value.clone();
        }
        
        Ok(body)
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    async fn send(&self, request: LlmRequest) -> Result<LlmResponse> {
        // Build the request body
        let body = self.build_request(&request).await?;
        
        // Send the request to the Ollama API
        let url = format!("{}/api/generate", self.api_base);
        
        let response = self.http_client.post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Ollama API: {}", e))?;
            
        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Could not read error response".to_string());
                
            return Err(anyhow!("Ollama API error ({}): {}", status, error_text));
        }
        
        // Parse the response
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse Ollama API response: {}", e))?;
            
        // Extract the response text
        let content = response_json["response"].as_str()
            .ok_or_else(|| anyhow!("Invalid response format: 'response' field is missing or not a string"))?;
            
        // Extract token usage if available
        let tokens_used = response_json["eval_count"].as_u64()
            .map(|t| t as usize);
            
        // Create the response
        let mut llm_response = LlmResponse::new(
            content.to_string(),
            request.model,
            self.name().to_string()
        );
        
        if let Some(tokens) = tokens_used {
            llm_response = llm_response.with_tokens(tokens);
        }
        
        Ok(llm_response)
    }

    fn name(&self) -> &str {
        "ollama"
    }

    async fn is_available(&self) -> bool {
        // Check if Ollama is running by sending a simple request
        let url = format!("{}/api/version", self.api_base);
        
        match self.http_client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}
