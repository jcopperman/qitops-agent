use anyhow::{Result, anyhow};
use clap::{Args, Subcommand};
use std::collections::HashMap;
use colored::Colorize;

use crate::llm::{ConfigManager, ProviderConfig, LlmRequest, LlmRouter, CacheConfig};
use crate::cli::branding;
use crate::cli::progress::ProgressIndicator;

/// LLM configuration and management commands
#[derive(Debug, Args)]
pub struct LlmArgs {
    #[clap(subcommand)]
    pub command: LlmCommand,
}

/// Cache management commands
#[derive(Debug, Subcommand)]
pub enum CacheCommand {
    /// Clear the cache
    #[clap(name = "clear")]
    Clear,

    /// Configure the cache
    #[clap(name = "config")]
    Config {
        /// Enable or disable the cache
        #[clap(long)]
        enabled: Option<bool>,

        /// Cache TTL in seconds
        #[clap(long)]
        ttl: Option<u64>,

        /// Enable or disable disk cache
        #[clap(long)]
        disk: Option<bool>,
    },

    /// Show cache status
    #[clap(name = "status")]
    Status,
}

/// LLM subcommands
#[derive(Debug, Subcommand)]
pub enum LlmCommand {
    /// List available LLM providers
    #[clap(name = "list")]
    List,

    /// Add a new LLM provider
    #[clap(name = "add")]
    Add {
        /// Provider type (openai, ollama, anthropic)
        #[clap(short = 'p', long)]
        provider: String,

        /// API key (if needed)
        #[clap(short = 'k', long)]
        api_key: Option<String>,

        /// API base URL (if custom)
        #[clap(short = 'b', long)]
        api_base: Option<String>,

        /// Default model to use
        #[clap(short = 'm', long)]
        model: String,
    },

    /// Remove an LLM provider
    #[clap(name = "remove")]
    Remove {
        /// Provider type to remove
        #[clap(short = 'p', long)]
        provider: String,
    },

    /// Set the default LLM provider
    #[clap(name = "default")]
    SetDefault {
        /// Provider type to set as default
        #[clap(short = 'p', long)]
        provider: String,
    },

    /// Set a task-specific LLM provider
    #[clap(name = "task")]
    SetTask {
        /// Task name
        #[clap(short = 't', long)]
        task: String,

        /// Provider type to use for this task
        #[clap(short = 'p', long)]
        provider: String,
    },

    /// Test an LLM provider
    #[clap(name = "test")]
    Test {
        /// Provider type to test
        #[clap(short = 'p', long)]
        provider: Option<String>,

        /// Prompt to send
        #[clap(short = 't', long)]
        prompt: String,

        /// Disable cache for this request
        #[clap(long)]
        no_cache: bool,
    },

    /// Manage the LLM cache
    #[clap(name = "cache")]
    Cache {
        /// Cache command
        #[clap(subcommand)]
        command: CacheCommand,
    },
}

/// Handle LLM commands
pub async fn handle_llm_command(args: &LlmArgs) -> Result<()> {
    match &args.command {
        LlmCommand::List => list_providers().await,
        LlmCommand::Add { provider, api_key, api_base, model } => {
            add_provider(provider, api_key.clone(), api_base.clone(), model).await
        },
        LlmCommand::Remove { provider } => remove_provider(provider).await,
        LlmCommand::SetDefault { provider } => set_default_provider(provider).await,
        LlmCommand::SetTask { task, provider } => set_task_provider(task, provider).await,
        LlmCommand::Test { provider, prompt, no_cache } => test_provider(provider.as_deref(), prompt, *no_cache).await,
        LlmCommand::Cache { command } => {
            match command {
                CacheCommand::Clear => clear_cache().await,
                CacheCommand::Config { enabled, ttl, disk } => configure_cache(*enabled, *ttl, *disk).await,
                CacheCommand::Status => show_cache_status().await,
            }
        },
    }
}

/// List available LLM providers
async fn list_providers() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let config = config_manager.get_config();

    branding::print_section("Available LLM providers");

    for provider in &config.providers {
        let default_marker = if provider.provider_type == config.default_provider {
            " (default)".bright_green().to_string()
        } else {
            "".to_string()
        };

        println!("- {}{}", provider.provider_type.bright_cyan(), default_marker);
        println!("  Model: {}", provider.default_model);
        if let Some(api_base) = &provider.api_base {
            println!("  API Base: {}", api_base);
        }
        if !provider.options.is_empty() {
            println!("  Options:");
            for (key, value) in &provider.options {
                println!("    {}: {}", key, value);
            }
        }
        println!();
    }

    if !config.task_providers.is_empty() {
        branding::print_section("Task-specific providers");
        for (task, provider) in &config.task_providers {
            println!("- {}: {}", task.bright_cyan(), provider);
        }
    }

    // Try to initialize the router and check which providers are actually available
    match LlmRouter::new(config.clone()).await {
        Ok(router) => {
            let available = router.available_providers().await;
            branding::print_section("Status");
            println!("Currently available providers: {}", available.join(", ").bright_green());
            println!("Default provider: {}", router.default_provider().bright_green());
        },
        Err(e) => {
            branding::print_warning(&format!("Could not initialize LLM router: {}", e));
        }
    }

    Ok(())
}

/// Add a new LLM provider
async fn add_provider(provider_type: &str, api_key: Option<String>, api_base: Option<String>, model: &str) -> Result<()> {
    let mut config_manager = ConfigManager::new()?;

    let provider_config = ProviderConfig {
        provider_type: provider_type.to_string(),
        api_key,
        api_base,
        default_model: model.to_string(),
        options: HashMap::new(),
    };

    match config_manager.add_provider(provider_config) {
        Ok(_) => {
            config_manager.save_config()?;
            branding::print_success(&format!("Added provider '{}' with model '{}'", provider_type, model));
            Ok(())
        },
        Err(e) => {
            branding::print_error(&format!("Failed to add provider: {}", e));
            Err(e)
        }
    }
}

/// Remove an LLM provider
async fn remove_provider(provider_type: &str) -> Result<()> {
    let mut config_manager = ConfigManager::new()?;

    match config_manager.remove_provider(provider_type) {
        Ok(_) => {
            config_manager.save_config()?;
            branding::print_success(&format!("Removed provider: {}", provider_type));
            Ok(())
        },
        Err(e) => {
            branding::print_error(&format!("Failed to remove provider: {}", e));
            Err(e)
        }
    }
}

/// Set the default LLM provider
async fn set_default_provider(provider_type: &str) -> Result<()> {
    let mut config_manager = ConfigManager::new()?;

    match config_manager.set_default_provider(provider_type.to_string()) {
        Ok(_) => {
            config_manager.save_config()?;
            branding::print_success(&format!("Set default provider to: {}", provider_type));
            Ok(())
        },
        Err(e) => {
            branding::print_error(&format!("Failed to set default provider: {}", e));
            Err(e)
        }
    }
}

/// Set a task-specific LLM provider
async fn set_task_provider(task: &str, provider_type: &str) -> Result<()> {
    let mut config_manager = ConfigManager::new()?;

    match config_manager.set_task_provider(task.to_string(), provider_type.to_string()) {
        Ok(_) => {
            config_manager.save_config()?;
            branding::print_success(&format!("Set provider for task '{}' to: {}", task, provider_type));
            Ok(())
        },
        Err(e) => {
            branding::print_error(&format!("Failed to set task provider: {}", e));
            Err(e)
        }
    }
}

/// Test an LLM provider
async fn test_provider(provider_type: Option<&str>, prompt: &str, no_cache: bool) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.get_config().clone();

    // If a specific provider is requested, only include that provider in the config
    if let Some(provider) = provider_type {
        branding::print_command_header(&format!("Testing {} Provider", provider));

        // Check if the provider exists
        if !config.providers.iter().any(|p| p.provider_type == provider) {
            branding::print_error(&format!("Provider '{}' not found in configuration", provider));
            return Err(anyhow!("Provider '{}' not found in configuration", provider));
        }

        // Keep only the requested provider
        config.providers.retain(|p| p.provider_type == provider);

        // Set it as the default
        config.default_provider = provider.to_string();
    } else {
        branding::print_command_header("Testing Default Provider");
    }

    // Get the default model for the provider
    let model = if let Some(provider) = provider_type {
        // Find the provider config
        let provider_config = config.providers.iter()
            .find(|p| p.provider_type == provider)
            .ok_or_else(|| anyhow!("Provider '{}' not found in configuration", provider))?;
        provider_config.default_model.clone()
    } else {
        // Find the default provider config
        let default_provider = &config.default_provider;
        let provider_config = config.providers.iter()
            .find(|p| p.provider_type == *default_provider)
            .ok_or_else(|| anyhow!("Default provider '{}' not found in configuration", default_provider))?;
        provider_config.default_model.clone()
    };

    // Create a simple request
    let mut request = LlmRequest::new(prompt.to_string(), model)
        .with_system_message("You are a helpful assistant for QA testing.".to_string());

    // Disable cache if requested
    if no_cache {
        request = request.with_cache(false);
    }

    // Show a spinner while waiting for the response
    let progress = ProgressIndicator::new("Initializing LLM router...");

    // Try to initialize the router with the filtered config
    match LlmRouter::new(config.clone()).await {
        Ok(router) => {
            // Update progress and send the request
            progress.update_message("Sending request to LLM...");
            match router.send(request.clone(), None).await {
                Ok(response) => {
                    // Finish progress indicator
                    progress.finish_with_message("Response received!");

                    branding::print_section("Response");
                    println!("From: {} (model: {})", response.provider.bright_cyan(), response.model.bright_cyan());
                    println!("{}", "---".bright_blue());
                    println!("{}", response.text);
                    println!("{}", "---".bright_blue());

                    if let Some(tokens) = response.tokens_used {
                        println!("Tokens used: {}", tokens.to_string().bright_yellow());
                    }

                    // Show cache status
                    if response.cached {
                        println!("Response was {} (from cache)", "cached".bright_green());
                    }
                },
                Err(e) => {
                    // Finish progress indicator
                    progress.finish();
                    branding::print_error(&format!("Failed to get response from LLM: {}", e));
                    return Err(anyhow!("Failed to get response from LLM: {}", e));
                }
            }
        },
        Err(e) => {
            // Finish progress indicator
            progress.finish();

            // If we're testing a specific provider and router initialization failed,
            // it's likely because other providers in the config are not properly configured.
            branding::print_error(&format!("Failed to initialize LLM router: {}", e));
            return Err(anyhow!("Failed to initialize LLM router: {}", e));
        }
    }

    Ok(())
}

/// Clear the LLM cache
async fn clear_cache() -> Result<()> {
    branding::print_command_header("Clearing LLM Cache");

    // Initialize the cache
    let progress = ProgressIndicator::new("Initializing cache...");
    let config_manager = ConfigManager::new()?;
    let config = config_manager.get_config().clone();

    if !config.cache.enabled {
        progress.finish();
        branding::print_warning("Cache is disabled in configuration");
        return Ok(());
    }

    match crate::llm::cache::ResponseCache::new(config.cache.ttl_seconds, config.cache.use_disk) {
        Ok(mut cache) => {
            progress.update_message("Clearing cache...");
            match cache.clear() {
                Ok(_) => {
                    progress.finish();
                    branding::print_success("Cache cleared successfully");
                    Ok(())
                },
                Err(e) => {
                    progress.finish();
                    branding::print_error(&format!("Failed to clear cache: {}", e));
                    Err(e)
                }
            }
        },
        Err(e) => {
            progress.finish();
            branding::print_error(&format!("Failed to initialize cache: {}", e));
            Err(e)
        }
    }
}

/// Configure the LLM cache
async fn configure_cache(enabled: Option<bool>, ttl: Option<u64>, disk: Option<bool>) -> Result<()> {
    branding::print_command_header("Configuring LLM Cache");

    let mut config_manager = ConfigManager::new()?;

    // Get a copy of the current configuration
    let mut config = config_manager.get_config().clone();

    // Update cache configuration
    if let Some(enabled) = enabled {
        config.cache.enabled = enabled;
    }

    if let Some(ttl) = ttl {
        config.cache.ttl_seconds = ttl;
    }

    if let Some(disk) = disk {
        config.cache.use_disk = disk;
    }

    // Update the configuration and save it
    *config_manager.get_config_mut() = config.clone();
    config_manager.save_config()?;

    branding::print_success("Cache configuration updated");

    // Show the new configuration
    println!("Cache enabled: {}", if config.cache.enabled { "yes".bright_green() } else { "no".bright_red() });
    println!("Cache TTL: {} seconds", config.cache.ttl_seconds.to_string().bright_yellow());
    println!("Disk cache: {}", if config.cache.use_disk { "yes".bright_green() } else { "no".bright_red() });

    Ok(())
}

/// Show the LLM cache status
async fn show_cache_status() -> Result<()> {
    branding::print_command_header("LLM Cache Status");

    let config_manager = ConfigManager::new()?;
    let config = config_manager.get_config().clone();

    // Show the configuration
    println!("Cache enabled: {}", if config.cache.enabled { "yes".bright_green() } else { "no".bright_red() });
    println!("Cache TTL: {} seconds", config.cache.ttl_seconds.to_string().bright_yellow());
    println!("Disk cache: {}", if config.cache.use_disk { "yes".bright_green() } else { "no".bright_red() });

    // Try to initialize the cache to check if it's working
    if config.cache.enabled {
        match crate::llm::cache::ResponseCache::new(config.cache.ttl_seconds, config.cache.use_disk) {
            Ok(_) => {
                println!("\nCache status: {}", "working".bright_green());
            },
            Err(e) => {
                println!("\nCache status: {}", "error".bright_red());
                println!("Error: {}", e);
            }
        }
    }

    Ok(())
}