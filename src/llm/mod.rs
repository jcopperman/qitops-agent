// LLM integration
pub mod client;
pub mod config;
pub mod cache;
pub mod providers;

// Re-export commonly used types
pub use client::{LlmRequest, LlmRouter, ProviderConfig};
pub use config::ConfigManager;
