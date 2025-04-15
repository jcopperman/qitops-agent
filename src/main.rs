mod agent;
mod cli;
mod llm;
mod plugin;
mod ci;
mod source;
mod persona;
mod config;
mod bot;
mod update;
mod monitoring;
pub mod context;

use anyhow::Result;
use clap::Parser;
use cli::commands::{Cli, Command, RunCommand, MonitoringCommand};
use cli::llm::handle_llm_command;
use cli::github::handle_github_command;
use cli::source::handle_source_command;
use cli::persona::handle_persona_command;
use cli::bot::handle_bot_command;
use cli::branding;
use cli::progress::ProgressIndicator;
use tracing::info;
use colored::Colorize;
use tracing_subscriber;
use std::io::Write;

use agent::{TestGenAgent, PrAnalyzeAgent, RiskAgent, TestDataAgent, SessionAgent, AgentStatus};
use agent::traits::Agent;
use llm::{ConfigManager, LlmRouter};
use config::QitOpsConfigManager;
use monitoring::{init as init_monitoring, MonitoringConfig, track_command, Timer};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up error handling for the entire application
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            eprintln!("\n💥 Panic occurred in file '{}' at line {}", location.file(), location.line());
        } else {
            eprintln!("\n💥 Panic occurred but can't get location information");
        }

        if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("Error message: {}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Error message: {}", s);
        } else {
            eprintln!("Unknown error occurred");
        }

        eprintln!("\nPlease report this issue at: https://github.com/jcopperman/qitops-agent/issues\n");
    }));

    // Initialize logging with better formatting
    tracing_subscriber::fmt()
        .with_env_filter(if std::env::var("RUST_LOG").is_ok() {
            tracing_subscriber::EnvFilter::from_default_env()
        } else {
            tracing_subscriber::EnvFilter::new("qitops=info,warn")
        })
        .init();

    // Parse command line arguments
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Don't show error for --help or --version
            if err.kind() == clap::error::ErrorKind::DisplayHelp ||
               err.kind() == clap::error::ErrorKind::DisplayVersion {
                err.exit();
            }

            // For other errors, show a more user-friendly message
            eprintln!("\n{} {}", "✗".bright_red(), "Error parsing command line arguments:".red());
            eprintln!("{}", err);
            eprintln!("\nRun 'qitops --help' for usage information.\n");
            std::process::exit(1);
        }
    };

    // Display banner (unless help or version is requested)
    if std::env::args().len() > 1 && !std::env::args().any(|arg| arg == "-h" || arg == "--help" || arg == "-V" || arg == "--version") {
        branding::print_banner();
    }

    // Enable verbose logging if requested
    if cli.verbose {
        info!("Verbose logging enabled");
        // We don't need to reinitialize the subscriber, just log that verbose mode is enabled
    }

    // Check for updates in the background
    let update_check = tokio::spawn(update::check_for_updates());

    // Initialize monitoring service if enabled
    let monitoring_enabled = std::env::var("QITOPS_MONITORING_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true";
    let monitoring_host = std::env::var("QITOPS_MONITORING_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let monitoring_port = std::env::var("QITOPS_MONITORING_PORT").unwrap_or_else(|_| "9090".to_string())
        .parse::<u16>().unwrap_or(9090);
    let monitoring_interval = std::env::var("QITOPS_MONITORING_INTERVAL").unwrap_or_else(|_| "15".to_string())
        .parse::<u64>().unwrap_or(15);

    let monitoring_config = MonitoringConfig::new(
        monitoring_enabled,
        monitoring_host,
        monitoring_port,
        monitoring_interval
    );

    if monitoring_enabled {
        if let Err(e) = init_monitoring(monitoring_config.clone()).await {
            eprintln!("Warning: Failed to initialize monitoring service: {}", e);
        } else {
            info!("Monitoring service started on {}:{}", monitoring_config.host, monitoring_config.port);
            println!("Monitoring service started on {}:{}", monitoring_config.host, monitoring_config.port);
        }
    }

    // Execute the requested command
    let _command_result = match cli.command {
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
        Command::Bot(bot_args) => {
            branding::print_command_header("QitOps Bot");
            handle_bot_command(&bot_args).await?
        }
        Command::Monitoring { command } => {
            branding::print_command_header("QitOps Monitoring");
            handle_monitoring_command(command).await?
        }
        Command::Version => {
            println!("QitOps Agent v{}", env!("CARGO_PKG_VERSION"));
            println!("Developed by {}", env!("CARGO_PKG_AUTHORS"));
        }
    };

    // Check if an update is available
    if let Ok(update_result) = update_check.await {
        if let Ok(Some(update_info)) = update_result {
            update::print_update_info(&update_info);
        }
    }

    Ok(())
}

/// Handle run commands with enhanced error handling
async fn handle_run_command(command: RunCommand, verbose: bool) -> Result<()> {
    // Wrap the command execution in a function that provides better error handling
    let result = handle_run_command_inner(command, verbose).await;

    // Handle errors with user-friendly messages
    if let Err(e) = result {
        let error_message = format!("{}", e);

        // Categorize errors for better user feedback
        if error_message.contains("LLM") || error_message.contains("model") {
            branding::print_error("LLM configuration error");
            eprintln!("Error details: {}", error_message);
            eprintln!("\nTry configuring your LLM provider with: qitops llm add --provider <provider> --api-key <key>");
            eprintln!("Or use a local provider like Ollama: qitops llm add --provider ollama --api-base http://localhost:11434");
        } else if error_message.contains("GitHub") || error_message.contains("token") {
            branding::print_error("GitHub integration error");
            eprintln!("Error details: {}", error_message);
            eprintln!("\nTry configuring your GitHub token with: qitops github config --token <token>");
        } else if error_message.contains("File not found") || error_message.contains("path") {
            branding::print_error("File or path error");
            eprintln!("Error details: {}", error_message);
            eprintln!("\nPlease check that the specified file or path exists and is accessible.");
        } else if error_message.contains("Permission denied") || error_message.contains("Access is denied") {
            branding::print_error("File access permission error");
            eprintln!("Error details: {}", error_message);
            eprintln!("\nThis error occurs when QitOps doesn't have permission to access the specified file.");
            eprintln!("Try one of the following solutions:");
            eprintln!("  1. Run QitOps with administrator privileges");
            eprintln!("  2. Check the file permissions");
            eprintln!("  3. Use a file path that QitOps has permission to access");
            eprintln!("  4. Make sure the file exists and is readable");
        } else {
            branding::print_error("Command execution failed");
            eprintln!("Error details: {}", error_message);

            if verbose {
                // Print the full error chain in verbose mode
                let mut source = e.source();
                let mut depth = 0;
                while let Some(err) = source {
                    eprintln!("Caused by ({}): {}", depth, err);
                    source = err.source();
                    depth += 1;
                }
            } else {
                eprintln!("\nRun with --verbose for more detailed error information.");
            }
        }

        // Return the error to propagate it up
        return Err(e);
    }

    Ok(())
}

/// Internal implementation of run command handling
async fn handle_run_command_inner(command: RunCommand, _verbose: bool) -> Result<()> {
    // Create a timer to track command execution time if monitoring is enabled
    let monitoring_enabled = std::env::var("QITOPS_MONITORING_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true";

    let command_name = match &command {
        RunCommand::TestGen { .. } => "test-gen",
        RunCommand::PrAnalyze { .. } => "pr-analyze",
        RunCommand::Risk { .. } => "risk",
        RunCommand::TestData { .. } => "test-data",
        RunCommand::Session { .. } => "session",
    };

    // Track command execution if monitoring is enabled
    let timer = if monitoring_enabled {
        track_command(command_name);
        Some(Timer::new(command_name))
    } else {
        None
    };

    // Execute the command
    let _result = match command {
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
            let _sources_vec = if let Some(sources) = sources.clone() {
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

            let _personas_vec = if let Some(personas) = personas.clone() {
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
            let _sources_vec = if let Some(sources) = sources.clone() {
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

            let _personas_vec = if let Some(personas) = personas.clone() {
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

            let _personas_vec = if let Some(personas) = personas.clone() {
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
            let agent = TestDataAgent::new(schema, count, sources_vec, "json".to_string(), router).await?;
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
        RunCommand::Session { name, sources, personas, application, session_type, objectives } => {
            branding::print_command_header("Starting Interactive Testing Session");
            info!("Starting interactive testing session: {}", name);

            // Get QitOps configuration
            let qitops_config_manager = QitOpsConfigManager::new()?;

            // Parse sources and personas
            let sources_vec = if let Some(sources) = sources.clone() {
                // Use sources from command line
                info!("Using sources: {}", sources);
                Some(sources.split(',').map(|s| s.trim().to_string()).collect())
            } else {
                // Use default sources from configuration
                let default_sources = qitops_config_manager.get_default_sources("session");
                if !default_sources.is_empty() {
                    info!("Using default sources: {}", default_sources.join(", "));
                    Some(default_sources)
                } else {
                    None
                }
            };

            let personas_vec = if let Some(personas) = personas.clone() {
                // Use personas from command line
                info!("Using personas: {}", personas);
                Some(personas.split(',').map(|s| s.trim().to_string()).collect())
            } else {
                // Use default personas from configuration
                let default_personas = qitops_config_manager.get_default_personas("session");
                if !default_personas.is_empty() {
                    info!("Using default personas: {}", default_personas.join(", "));
                    Some(default_personas)
                } else {
                    None
                }
            };

            // Parse objectives
            let objectives_vec = if let Some(objectives) = objectives {
                objectives.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                Vec::new()
            };

            // Validate application
            let app = application.unwrap_or_else(|| "unknown".to_string());
            if app.is_empty() {
                branding::print_error("Application name cannot be empty");
                return Ok(());
            }

            // Initialize LLM router
            let progress = ProgressIndicator::new("Initializing LLM router...");
            let config_manager = ConfigManager::new()?;
            let router = LlmRouter::new(config_manager.get_config().clone()).await?;
            progress.finish();

            // Create and execute the session agent
            let progress = ProgressIndicator::new("Generating testing plan...");
            let mut agent = SessionAgent::new(
                name,
                session_type,
                app,
                objectives_vec,
                sources_vec,
                personas_vec,
                router.clone()
            ).await?;

            // Initialize the agent
            agent.init()?;

            // Execute the agent to get the initial plan
            let result = agent.execute().await?;
            progress.finish();

            match result.status {
                AgentStatus::Success => {
                    branding::print_success(&result.message);
                    if let Some(data) = result.data {
                        if let Some(plan) = data.get("plan") {
                            println!("{}", "\nTesting Plan:\n".bright_blue());
                            println!("{}", plan);
                            println!();
                        }
                    }

                    // Start interactive session
                    println!("{}", "\nInteractive Testing Session Started".bright_green());
                    println!("Type 'exit' or 'quit' to end the session.\n");

                    // Interactive loop
                    loop {
                        // Get user input
                        print!("{} ", "You:".bright_cyan());
                        std::io::stdout().flush().unwrap();
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        let input = input.trim();

                        // Check for exit command
                        if input.to_lowercase() == "exit" || input.to_lowercase() == "quit" {
                            break;
                        }

                        // Process the message
                        let progress = ProgressIndicator::new("Processing...");
                        match agent.process_message(input).await {
                            Ok(response) => {
                                progress.finish();
                                println!("{} {}", "QitOps:".bright_green(), response);
                            },
                            Err(e) => {
                                progress.finish();
                                branding::print_error(&format!("Error: {}", e));
                            }
                        }
                    }

                    // Save session history
                    match agent.save_session_history() {
                        Ok(file_path) => {
                            println!("{}", "\nSession ended. Thank you for using QitOps Agent!".bright_green());
                            println!("{} {}", "Session history saved to:".bright_blue(), file_path);
                        },
                        Err(e) => {
                            println!("{}", "\nSession ended. Thank you for using QitOps Agent!".bright_green());
                            branding::print_warning(&format!("Failed to save session history: {}", e));
                        }
                    };
                },
                _ => branding::print_error(&result.message),
            }
        }
    };

    // Stop the timer if monitoring is enabled
    if let Some(t) = timer {
        t.stop();
    }

    Ok(())
}

/// Handle monitoring commands
async fn handle_monitoring_command(command: MonitoringCommand) -> Result<()> {
    match command {
        MonitoringCommand::Start { host, port, docker } => {
            // Start the monitoring server
            let monitoring_config = MonitoringConfig::new(
                true,
                host.clone(),
                port,
                15
            );

            // Initialize the monitoring service
            if let Err(e) = init_monitoring(monitoring_config.clone()).await {
                branding::print_error(&format!("Failed to start monitoring server: {}", e));
                return Err(anyhow::anyhow!("Failed to start monitoring server: {}", e));
            }

            branding::print_success(&format!("Monitoring server started on {}:{}", host, port));
            println!("Access metrics at http://{}:{}/metrics", host, port);

            // Start Docker monitoring stack if requested
            if docker {
                start_docker_monitoring_stack().await?;
            }
        }
        MonitoringCommand::Stop { docker } => {
            // Stop the monitoring server
            if let Err(e) = monitoring::stop().await {
                branding::print_error(&format!("Failed to stop monitoring server: {}", e));
                return Err(anyhow::anyhow!("Failed to stop monitoring server: {}", e));
            }

            branding::print_success("Monitoring server stopped");

            // Stop Docker monitoring stack if requested
            if docker {
                stop_docker_monitoring_stack().await?;
            }
        }
        MonitoringCommand::Status => {
            // Check if monitoring is enabled
            let monitoring_enabled = std::env::var("QITOPS_MONITORING_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true";

            if monitoring_enabled {
                let monitoring_host = std::env::var("QITOPS_MONITORING_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
                let monitoring_port = std::env::var("QITOPS_MONITORING_PORT").unwrap_or_else(|_| "9090".to_string());

                branding::print_success("Monitoring is enabled");
                println!("Metrics available at: http://{}:{}/metrics", monitoring_host, monitoring_port);
            } else {
                branding::print_info("Monitoring is disabled");
                println!("Enable monitoring with: qitops monitoring start");
            }

            // Check if Docker monitoring stack is running
            check_docker_monitoring_stack().await?;
        }
    }

    Ok(())
}

/// Start the Docker monitoring stack
async fn start_docker_monitoring_stack() -> Result<()> {
    // Check if Docker is installed
    let docker_check = tokio::process::Command::new("docker")
        .arg("--version")
        .output()
        .await;

    if docker_check.is_err() {
        branding::print_error("Docker is not installed or not in PATH");
        return Err(anyhow::anyhow!("Docker is not installed or not in PATH"));
    }

    // Check if docker-compose is installed
    let compose_check = tokio::process::Command::new("docker-compose")
        .arg("--version")
        .output()
        .await;

    if compose_check.is_err() {
        branding::print_error("docker-compose is not installed or not in PATH");
        return Err(anyhow::anyhow!("docker-compose is not installed or not in PATH"));
    }

    // Start the Docker monitoring stack
    let progress = ProgressIndicator::new("Starting Docker monitoring stack...");

    let result = tokio::process::Command::new("docker-compose")
        .arg("-f")
        .arg("docker-compose-monitoring.yml")
        .arg("up")
        .arg("-d")
        .output()
        .await;

    progress.finish();

    match result {
        Ok(output) => {
            if output.status.success() {
                branding::print_success("Docker monitoring stack started");
                println!("Access Grafana at http://localhost:3000");
                println!("Default credentials: admin/qitops");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                branding::print_error(&format!("Failed to start Docker monitoring stack: {}", error));
                return Err(anyhow::anyhow!("Failed to start Docker monitoring stack: {}", error));
            }
        }
        Err(e) => {
            branding::print_error(&format!("Failed to start Docker monitoring stack: {}", e));
            return Err(anyhow::anyhow!("Failed to start Docker monitoring stack: {}", e));
        }
    }

    Ok(())
}

/// Stop the Docker monitoring stack
async fn stop_docker_monitoring_stack() -> Result<()> {
    // Stop the Docker monitoring stack
    let progress = ProgressIndicator::new("Stopping Docker monitoring stack...");

    let result = tokio::process::Command::new("docker-compose")
        .arg("-f")
        .arg("docker-compose-monitoring.yml")
        .arg("down")
        .output()
        .await;

    progress.finish();

    match result {
        Ok(output) => {
            if output.status.success() {
                branding::print_success("Docker monitoring stack stopped");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                branding::print_error(&format!("Failed to stop Docker monitoring stack: {}", error));
                return Err(anyhow::anyhow!("Failed to stop Docker monitoring stack: {}", error));
            }
        }
        Err(e) => {
            branding::print_error(&format!("Failed to stop Docker monitoring stack: {}", e));
            return Err(anyhow::anyhow!("Failed to stop Docker monitoring stack: {}", e));
        }
    }

    Ok(())
}

/// Check if the Docker monitoring stack is running
async fn check_docker_monitoring_stack() -> Result<()> {
    // Check if Docker is installed
    let docker_check = tokio::process::Command::new("docker")
        .arg("--version")
        .output()
        .await;

    if docker_check.is_err() {
        branding::print_warning("Docker is not installed or not in PATH");
        return Ok(());
    }

    // Check if the monitoring stack is running
    let result = tokio::process::Command::new("docker-compose")
        .arg("-f")
        .arg("docker-compose-monitoring.yml")
        .arg("ps")
        .arg("-q")
        .output()
        .await;

    match result {
        Ok(output) => {
            if !output.stdout.is_empty() {
                branding::print_success("Docker monitoring stack is running");
                println!("Access Grafana at http://localhost:3000");
                println!("Default credentials: admin/qitops");
            } else {
                branding::print_info("Docker monitoring stack is not running");
                println!("Start it with: qitops monitoring start --docker");
            }
        }
        Err(_) => {
            branding::print_warning("Could not check Docker monitoring stack status");
        }
    }

    Ok(())
}