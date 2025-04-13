mod agent;
mod cli;
mod llm;
mod plugin;
mod ci;
mod source;
mod persona;
mod config;

use anyhow::Result;
use clap::Parser;
use cli::commands::{Cli, Command, RunCommand};
use cli::llm::handle_llm_command;
use cli::github::handle_github_command;
use cli::source::handle_source_command;
use cli::persona::handle_persona_command;
use cli::branding;
use cli::progress::ProgressIndicator;
use tracing::{info, error};
use tracing_subscriber;

use agent::{TestGenAgent, PrAnalyzeAgent, RiskAgent, TestDataAgent, AgentStatus};
use agent::traits::Agent;
use llm::{ConfigManager, LlmRouter};
use config::QitOpsConfigManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Display banner (unless help or version is requested)
    if std::env::args().len() > 1 && !std::env::args().any(|arg| arg == "-h" || arg == "--help" || arg == "-V" || arg == "--version") {
        branding::print_banner();
    }

    // Enable verbose logging if requested
    if cli.verbose {
        info!("Verbose logging enabled");
    }

    // Execute the requested command
    match cli.command {
        Command::Run { command } => {
            handle_run_command(command, cli.verbose).await?
        }
        Command::Llm(llm_args) => {
            branding::print_command_header("LLM Management");
            handle_llm_command(&llm_args).await?
        }
        Command::GitHub(github_args) => {
            branding::print_command_header("GitHub Integration");
            handle_github_command(&github_args).await?
        }
        Command::Source(source_args) => {
            branding::print_command_header("Source Management");
            handle_source_command(&source_args).await?
        }
        Command::Persona(persona_args) => {
            branding::print_command_header("Persona Management");
            handle_persona_command(&persona_args).await?
        }
        Command::Version => {
            println!("QitOps Agent v{}", env!("CARGO_PKG_VERSION"));
            println!("Developed by {}", env!("CARGO_PKG_AUTHORS"));
        }
    }

    Ok(())
}

async fn handle_run_command(command: RunCommand, verbose: bool) -> Result<()> {
    match command {
        RunCommand::TestGen { path, format, sources, personas } => {
            branding::print_command_header("Generating Test Cases");
            info!("Generating test cases for {} in {} format", path, format);

            if let Some(sources) = &sources {
                info!("Using sources: {}", sources);
            }

            if let Some(personas) = &personas {
                info!("Using personas: {}", personas);
            }

            // Initialize LLM router
            let progress = ProgressIndicator::new("Initializing LLM router...");
            let config_manager = ConfigManager::new()?;
            let router = LlmRouter::new(config_manager.get_config().clone()).await?;
            progress.finish();

            // Get QitOps configuration
            let qitops_config_manager = QitOpsConfigManager::new()?;

            // Parse sources and personas
            let sources_vec = if let Some(sources) = sources {
                // Use sources from command line
                Some(sources.split(',').map(|s| s.trim().to_string()).collect())
            } else {
                // Use default sources from configuration
                let default_sources = qitops_config_manager.get_default_sources("test-gen");
                if !default_sources.is_empty() {
                    info!("Using default sources: {}", default_sources.join(", "));
                    Some(default_sources)
                } else {
                    None
                }
            };

            let personas_vec = if let Some(personas) = personas {
                // Use personas from command line
                Some(personas.split(',').map(|s| s.trim().to_string()).collect())
            } else {
                // Use default personas from configuration
                let default_personas = qitops_config_manager.get_default_personas("test-gen");
                if !default_personas.is_empty() {
                    info!("Using default personas: {}", default_personas.join(", "));
                    Some(default_personas)
                } else {
                    None
                }
            };

            // Create and execute the test generation agent
            let progress = ProgressIndicator::new("Generating test cases...");
            let agent = TestGenAgent::new(path, &format, sources_vec, personas_vec, router).await?;
            let result = agent.execute().await?;
            progress.finish();

            match result.status {
                AgentStatus::Success => {
                    branding::print_success(&result.message);
                    if let Some(data) = result.data {
                        if let Some(test_cases) = data.get("test_cases") {
                            println!("\nTest Cases:\n");
                            println!("{}", test_cases);
                        }
                    }
                },
                _ => branding::print_error(&result.message),
            }
        }
        RunCommand::PrAnalyze { pr, sources, personas } => {
            branding::print_command_header("Analyzing Pull Request");
            info!("Analyzing PR: {}", pr);

            // Get QitOps configuration
            let qitops_config_manager = QitOpsConfigManager::new()?;

            // Parse sources and personas
            let sources_vec = if let Some(sources) = sources.clone() {
                // Use sources from command line
                info!("Using sources: {}", sources);
                sources.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                // Use default sources from configuration
                let default_sources = qitops_config_manager.get_default_sources("pr-analyze");
                if !default_sources.is_empty() {
                    info!("Using default sources: {}", default_sources.join(", "));
                    default_sources
                } else {
                    Vec::new()
                }
            };

            let personas_vec = if let Some(personas) = personas.clone() {
                // Use personas from command line
                info!("Using personas: {}", personas);
                personas.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                // Use default personas from configuration
                let default_personas = qitops_config_manager.get_default_personas("pr-analyze");
                if !default_personas.is_empty() {
                    info!("Using default personas: {}", default_personas.join(", "));
                    default_personas
                } else {
                    Vec::new()
                }
            };

            // Get GitHub configuration
            let github_config_manager = ci::GitHubConfigManager::new()?;

            // Try to extract repository information from PR URL
            let (owner, repo, pr_number) = match ci::GitHubClient::extract_repo_info(&pr) {
                Ok((owner, repo)) => {
                    // Try to extract PR number
                    let pr_number = match ci::GitHubClient::extract_pr_number(&pr) {
                        Ok(number) => number,
                        Err(_) => {
                            branding::print_error("Could not extract PR number from URL");
                            return Ok(());
                        }
                    };
                    (owner, repo, pr_number.to_string())
                },
                Err(_) => {
                    // If not a URL, use default repository and treat input as PR number
                    let owner = github_config_manager.get_default_owner()
                        .ok_or_else(|| {
                            branding::print_error("Default repository owner not configured");
                            branding::print_info("Configure with: qitops github config --owner <owner>");
                            anyhow::anyhow!("Default repository owner not configured")
                        })?;

                    let repo = github_config_manager.get_default_repo()
                        .ok_or_else(|| {
                            branding::print_error("Default repository name not configured");
                            branding::print_info("Configure with: qitops github config --repo <repo>");
                            anyhow::anyhow!("Default repository name not configured")
                        })?;

                    (owner, repo, pr.clone())
                }
            };

            // Create GitHub client
            let github_client = match ci::GitHubClient::from_config(github_config_manager.get_config()) {
                Ok(client) => client,
                Err(e) => {
                    branding::print_error(&format!("Failed to create GitHub client: {}", e));
                    branding::print_info("Configure GitHub token with: qitops github config --token <token>");
                    return Ok(());
                }
            };

            // Initialize LLM router
            let progress = ProgressIndicator::new("Initializing LLM router...");
            let config_manager = ConfigManager::new()?;
            let router = LlmRouter::new(config_manager.get_config().clone()).await?;
            progress.finish();

            // Create and execute the PR analysis agent
            let progress = ProgressIndicator::new("Analyzing pull request...");
            let agent = PrAnalyzeAgent::new(pr_number, None, owner, repo, github_client, router).await?;
            let result = agent.execute().await?;
            progress.finish();

            match result.status {
                AgentStatus::Success => {
                    branding::print_success(&result.message);
                    if let Some(data) = result.data {
                        if let Some(analysis) = data.get("analysis") {
                            println!("\nAnalysis:\n");
                            println!("{}", analysis);
                        }
                    }
                },
                _ => branding::print_error(&result.message),
            }
        }
        RunCommand::Risk { diff, components, focus, sources, personas } => {
            branding::print_command_header("Estimating Risk");
            info!("Estimating risk for diff: {}", diff);

            // Get QitOps configuration
            let qitops_config_manager = QitOpsConfigManager::new()?;

            // Parse sources and personas
            let sources_vec = if let Some(sources) = sources.clone() {
                // Use sources from command line
                info!("Using sources: {}", sources);
                sources.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                // Use default sources from configuration
                let default_sources = qitops_config_manager.get_default_sources("risk");
                if !default_sources.is_empty() {
                    info!("Using default sources: {}", default_sources.join(", "));
                    default_sources
                } else {
                    Vec::new()
                }
            };

            let personas_vec = if let Some(personas) = personas.clone() {
                // Use personas from command line
                info!("Using personas: {}", personas);
                personas.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                // Use default personas from configuration
                let default_personas = qitops_config_manager.get_default_personas("risk");
                if !default_personas.is_empty() {
                    info!("Using default personas: {}", default_personas.join(", "));
                    default_personas
                } else {
                    Vec::new()
                }
            };

            // Parse components and focus areas
            let components = components
                .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(Vec::new);

            let focus_areas = focus
                .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(Vec::new);

            if !components.is_empty() {
                info!("Components: {}", components.join(", "));
            }

            if !focus_areas.is_empty() {
                info!("Focus areas: {}", focus_areas.join(", "));
            }

            // Initialize LLM router
            let progress = ProgressIndicator::new("Initializing LLM router...");
            let config_manager = ConfigManager::new()?;
            let router = LlmRouter::new(config_manager.get_config().clone()).await?;
            progress.finish();

            // Check if diff is a file or a PR URL/number
            let agent = if diff.contains("github.com") || diff.contains("/") {
                // Try to extract repository information from PR URL
                let github_config_manager = ci::GitHubConfigManager::new()?;

                match ci::GitHubClient::extract_repo_info(&diff) {
                    Ok((owner, repo)) => {
                        // Try to extract PR number
                        match ci::GitHubClient::extract_pr_number(&diff) {
                            Ok(pr_number) => {
                                // Create GitHub client
                                match ci::GitHubClient::from_config(github_config_manager.get_config()) {
                                    Ok(github_client) => {
                                        branding::print_info(&format!("Analyzing PR #{} in {}/{}", pr_number, owner, repo));
                                        RiskAgent::new_from_pr(
                                            pr_number.to_string(),
                                            components,
                                            focus_areas,
                                            owner,
                                            repo,
                                            github_client,
                                            router
                                        ).await?
                                    },
                                    Err(e) => {
                                        branding::print_error(&format!("Failed to create GitHub client: {}", e));
                                        branding::print_info("Using diff as a file path instead");
                                        RiskAgent::new_from_diff(diff, components, focus_areas, router).await?
                                    }
                                }
                            },
                            Err(_) => {
                                branding::print_error("Could not extract PR number from URL");
                                branding::print_info("Using diff as a file path instead");
                                RiskAgent::new_from_diff(diff, components, focus_areas, router).await?
                            }
                        }
                    },
                    Err(_) => {
                        // If not a GitHub URL, treat as a file path
                        RiskAgent::new_from_diff(diff, components, focus_areas, router).await?
                    }
                }
            } else {
                // Try to parse as a PR number with default repository
                let github_config_manager = ci::GitHubConfigManager::new()?;

                if let (Some(owner), Some(repo)) = (github_config_manager.get_default_owner(), github_config_manager.get_default_repo()) {
                    if let Ok(pr_number) = diff.parse::<u64>() {
                        // Create GitHub client
                        match ci::GitHubClient::from_config(github_config_manager.get_config()) {
                            Ok(github_client) => {
                                branding::print_info(&format!("Analyzing PR #{} in {}/{}", pr_number, owner, repo));
                                RiskAgent::new_from_pr(
                                    pr_number.to_string(),
                                    components,
                                    focus_areas,
                                    owner,
                                    repo,
                                    github_client,
                                    router
                                ).await?
                            },
                            Err(_) => {
                                branding::print_info("Using diff as a file path");
                                RiskAgent::new_from_diff(diff, components, focus_areas, router).await?
                            }
                        }
                    } else {
                        // Not a PR number, treat as a file path
                        RiskAgent::new_from_diff(diff, components, focus_areas, router).await?
                    }
                } else {
                    // No default repository configured, treat as a file path
                    RiskAgent::new_from_diff(diff, components, focus_areas, router).await?
                }
            };

            // Execute the risk assessment agent
            let progress = ProgressIndicator::new("Estimating risk...");
            let result = agent.execute().await?;
            progress.finish();

            match result.status {
                AgentStatus::Success => {
                    branding::print_success(&result.message);
                    if let Some(data) = result.data {
                        if let Some(risk_assessment) = data.get("risk_assessment") {
                            println!("\nRisk Assessment:\n");
                            println!("{}", risk_assessment);
                        }
                    }
                },
                _ => branding::print_error(&result.message),
            }
        }
        RunCommand::TestData { schema, count, sources, personas } => {
            branding::print_command_header("Generating Test Data");
            info!("Generating {} test data records for schema: {}", count, schema);

            // Get QitOps configuration
            let qitops_config_manager = QitOpsConfigManager::new()?;

            // Parse sources and personas
            let sources_vec = if let Some(sources) = sources.clone() {
                // Use sources from command line
                info!("Using sources: {}", sources);
                sources.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                // Use default sources from configuration
                let default_sources = qitops_config_manager.get_default_sources("test-data");
                if !default_sources.is_empty() {
                    info!("Using default sources: {}", default_sources.join(", "));
                    default_sources
                } else {
                    Vec::new()
                }
            };

            let personas_vec = if let Some(personas) = personas.clone() {
                // Use personas from command line
                info!("Using personas: {}", personas);
                personas.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                // Use default personas from configuration
                let default_personas = qitops_config_manager.get_default_personas("test-data");
                if !default_personas.is_empty() {
                    info!("Using default personas: {}", default_personas.join(", "));
                    default_personas
                } else {
                    Vec::new()
                }
            };

            // Initialize LLM router
            let progress = ProgressIndicator::new("Initializing LLM router...");
            let config_manager = ConfigManager::new()?;
            let router = LlmRouter::new(config_manager.get_config().clone()).await?;
            progress.finish();

            // Create and execute the test data generation agent
            let progress = ProgressIndicator::new("Generating test data...");
            let agent = TestDataAgent::new(schema, count, sources_vec, personas_vec, "json".to_string(), router).await?;
            let result = agent.execute().await?;
            progress.finish();

            match result.status {
                AgentStatus::Success => {
                    branding::print_success(&result.message);
                    if let Some(data) = result.data {
                        if let Some(test_data) = data.get("test_data") {
                            println!("\nTest Data:\n");
                            println!("{}", test_data);
                        }
                    }
                },
                _ => branding::print_error(&result.message),
            }
        }
        RunCommand::Session { name, sources, personas } => {
            branding::print_command_header("Starting Interactive Testing Session");
            info!("Starting interactive testing session: {}", name);

            // Get QitOps configuration
            let qitops_config_manager = QitOpsConfigManager::new()?;

            // Parse sources and personas
            let sources_vec = if let Some(sources) = sources.clone() {
                // Use sources from command line
                info!("Using sources: {}", sources);
                sources.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                // Use default sources from configuration
                let default_sources = qitops_config_manager.get_default_sources("session");
                if !default_sources.is_empty() {
                    info!("Using default sources: {}", default_sources.join(", "));
                    default_sources
                } else {
                    Vec::new()
                }
            };

            let personas_vec = if let Some(personas) = personas.clone() {
                // Use personas from command line
                info!("Using personas: {}", personas);
                personas.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                // Use default personas from configuration
                let default_personas = qitops_config_manager.get_default_personas("session");
                if !default_personas.is_empty() {
                    info!("Using default personas: {}", default_personas.join(", "));
                    default_personas
                } else {
                    Vec::new()
                }
            };
            // TODO: Implement interactive testing session
            branding::print_info("This feature is coming soon!");
        }
    }

    Ok(())
}
