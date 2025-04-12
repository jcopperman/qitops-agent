mod agent;
mod cli;
mod llm;
mod plugin;
mod ci;

use anyhow::Result;
use clap::Parser;
use cli::commands::{Cli, Command};
use cli::llm::handle_llm_command;
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

            // Get GitHub token from environment
            let github_token = std::env::var("GITHUB_TOKEN")
                .unwrap_or_else(|_| {
                    branding::print_error("GITHUB_TOKEN environment variable not set");
                    String::new()
                });

            if github_token.is_empty() {
                branding::print_error("GitHub token is required for PR analysis");
                return Ok(());
            }

            // Get repository information
            // For now, we'll use hardcoded values, but in a real implementation,
            // we would extract this from the PR URL or use a configuration file
            let owner = "jcopperman".to_string();
            let repo = "qitops-agent".to_string();

            // Initialize LLM router
            let progress = ProgressIndicator::new("Initializing LLM router...");
            let config_manager = ConfigManager::new()?;
            let router = LlmRouter::new(config_manager.get_config().clone()).await?;
            progress.finish();

            // Create and execute the PR analysis agent
            let progress = ProgressIndicator::new("Analyzing pull request...");
            let agent = PrAnalyzeAgent::new(pr, None, owner, repo, github_token, router).await?;
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
        Command::Risk { diff } => {
            branding::print_command_header("Estimating Risk");
            info!("Estimating risk for diff: {}", diff);

            // Initialize LLM router
            let progress = ProgressIndicator::new("Initializing LLM router...");
            let config_manager = ConfigManager::new()?;
            let router = LlmRouter::new(config_manager.get_config().clone()).await?;
            progress.finish();

            // Create and execute the risk assessment agent
            let progress = ProgressIndicator::new("Estimating risk...");
            let agent = RiskAgent::new_from_diff(diff, Vec::new(), Vec::new(), router).await?;
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
    }

    Ok(())
}
