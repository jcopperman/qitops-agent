mod agent;
mod cli;
mod llm;
mod plugin;
mod ci;

use anyhow::Result;
use clap::Parser;
use cli::commands::{Cli, Command};
use cli::llm::handle_llm_command;
use cli::github::handle_github_command;
use cli::branding;
use cli::progress::ProgressIndicator;
use tracing::{info, error};
use tracing_subscriber;

use agent::{TestGenAgent, PrAnalyzeAgent, RiskAgent, TestDataAgent, AgentStatus};
use agent::traits::Agent;
use llm::{ConfigManager, LlmRouter};

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
        Command::TestGen { path, format } => {
            branding::print_command_header("Generating Test Cases");
            info!("Generating test cases for {} in {} format", path, format);

            // Initialize LLM router
            let progress = ProgressIndicator::new("Initializing LLM router...");
            let config_manager = ConfigManager::new()?;
            let router = LlmRouter::new(config_manager.get_config().clone()).await?;
            progress.finish();

            // Create and execute the test generation agent
            let progress = ProgressIndicator::new("Generating test cases...");
            let agent = TestGenAgent::new(path, &format, router).await?;
            let result = agent.execute().await?;
            progress.finish();

            match result.status {
                AgentStatus::Success => {
                    branding::print_success(&result.message);
                    if let Some(data) = result.data {
                        if let Some(output_file) = data.get("output_file") {
                            println!("Test cases saved to: {}", output_file);
                        }
                    }
                },
                _ => branding::print_error(&result.message),
            }
        }
        Command::PrAnalyze { pr } => {
            branding::print_command_header("Analyzing Pull Request");
            info!("Analyzing PR: {}", pr);

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
                            branding::print_info("Configure with: qitops-agent github config --owner <owner>");
                            anyhow::anyhow!("Default repository owner not configured")
                        })?;

                    let repo = github_config_manager.get_default_repo()
                        .ok_or_else(|| {
                            branding::print_error("Default repository name not configured");
                            branding::print_info("Configure with: qitops-agent github config --repo <repo>");
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
                    branding::print_info("Configure GitHub token with: qitops-agent github config --token <token>");
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
        Command::Risk { diff, components, focus } => {
            branding::print_command_header("Estimating Risk");
            info!("Estimating risk for diff: {}", diff);

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
                        if let Some(assessment) = data.get("assessment") {
                            println!("\nRisk Assessment:\n");
                            println!("{}", assessment);
                        }
                    }
                },
                _ => branding::print_error(&result.message),
            }
        }
        Command::TestData { schema, count } => {
            branding::print_command_header("Generating Test Data");
            info!("Generating {} test data records for schema: {}", count, schema);

            // Initialize LLM router
            let progress = ProgressIndicator::new("Initializing LLM router...");
            let config_manager = ConfigManager::new()?;
            let router = LlmRouter::new(config_manager.get_config().clone()).await?;
            progress.finish();

            // Create and execute the test data generation agent
            let progress = ProgressIndicator::new("Generating test data...");
            let agent = TestDataAgent::new(schema, count, Vec::new(), "json".to_string(), router).await?;
            let result = agent.execute().await?;
            progress.finish();

            match result.status {
                AgentStatus::Success => {
                    branding::print_success(&result.message);
                    if let Some(data) = result.data {
                        if let Some(output_file) = data.get("output_file") {
                            println!("Test data saved to: {}", output_file);
                        }
                    }
                },
                _ => branding::print_error(&result.message),
            }
        }
        Command::Session { name } => {
            branding::print_command_header("Starting Interactive Testing Session");
            info!("Starting interactive testing session: {}", name);
            // TODO: Implement interactive testing session
            branding::print_info("This feature is coming soon!");
        }
        Command::Llm(llm_args) => {
            branding::print_command_header("LLM Management");
            handle_llm_command(&llm_args).await?
        }
        Command::GitHub(github_args) => {
            branding::print_command_header("GitHub Integration");
            handle_github_command(&github_args).await?
        }
    }

    Ok(())
}
