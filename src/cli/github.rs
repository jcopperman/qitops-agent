use anyhow::Result;
use clap::Subcommand;

use crate::ci::{GitHubConfigManager, GitHubClient};
use crate::cli::branding;

/// GitHub CLI arguments
#[derive(Debug, clap::Args)]
pub struct GitHubArgs {
    /// GitHub subcommand
    #[clap(subcommand)]
    pub command: GitHubCommand,
}

/// GitHub subcommands
#[derive(Debug, Subcommand)]
pub enum GitHubCommand {
    /// Configure GitHub integration
    #[clap(name = "config")]
    Config {
        /// GitHub API token
        #[clap(short = 't', long)]
        token: Option<String>,
        
        /// GitHub API base URL (for GitHub Enterprise)
        #[clap(short = 'b', long)]
        api_base: Option<String>,
        
        /// Default repository owner
        #[clap(short = 'o', long)]
        owner: Option<String>,
        
        /// Default repository name
        #[clap(short = 'r', long)]
        repo: Option<String>,
    },
    
    /// Test GitHub integration
    #[clap(name = "test")]
    Test {
        /// Repository owner
        #[clap(short = 'o', long)]
        owner: Option<String>,
        
        /// Repository name
        #[clap(short = 'r', long)]
        repo: Option<String>,
    },
    
    /// Show GitHub configuration
    #[clap(name = "status")]
    Status,
}

/// Handle GitHub commands
pub async fn handle_github_command(args: &GitHubArgs) -> Result<()> {
    match &args.command {
        GitHubCommand::Config { token, api_base, owner, repo } => {
            configure_github(token.clone(), api_base.clone(), owner.clone(), repo.clone()).await
        },
        GitHubCommand::Test { owner, repo } => {
            test_github_integration(owner.clone(), repo.clone()).await
        },
        GitHubCommand::Status => {
            show_github_status().await
        },
    }
}

/// Configure GitHub integration
async fn configure_github(token: Option<String>, api_base: Option<String>, owner: Option<String>, repo: Option<String>) -> Result<()> {
    let mut config_manager = GitHubConfigManager::new()?;
    
    if let Some(token) = token {
        config_manager.set_token(token)?;
        branding::print_success("GitHub token configured");
    }
    
    if let Some(api_base) = api_base {
        config_manager.set_api_base(api_base)?;
        branding::print_success("GitHub API base URL configured");
    }
    
    if let Some(owner) = owner {
        config_manager.set_default_owner(owner)?;
        branding::print_success("Default repository owner configured");
    }
    
    if let Some(repo) = repo {
        config_manager.set_default_repo(repo)?;
        branding::print_success("Default repository name configured");
    }
    
    Ok(())
}

/// Test GitHub integration
async fn test_github_integration(owner: Option<String>, repo: Option<String>) -> Result<()> {
    let config_manager = GitHubConfigManager::new()?;
    
    // Get owner and repo from args or config
    let owner = owner
        .or_else(|| config_manager.get_default_owner())
        .ok_or_else(|| anyhow::anyhow!("Repository owner not specified"))?;
        
    let repo = repo
        .or_else(|| config_manager.get_default_repo())
        .ok_or_else(|| anyhow::anyhow!("Repository name not specified"))?;
    
    // Create GitHub client
    let github_client = GitHubClient::from_config(config_manager.get_config())?;
    
    // Test connection by getting repository info
    branding::print_info(&format!("Testing GitHub connection to {}/{}...", owner, repo));
    
    let repository = github_client.get_repository(&owner, &repo).await?;
    
    branding::print_success(&format!("Successfully connected to GitHub repository: {}", repository.name));
    println!("Repository information:");
    println!("  Name: {}", repository.name);
    println!("  Owner: {}", repository.owner);
    println!("  Default branch: {}", repository.default_branch);
    println!("  Private: {}", repository.private);
    if let Some(language) = &repository.language {
        println!("  Primary language: {}", language);
    }
    if let Some(description) = &repository.description {
        println!("  Description: {}", description);
    }
    
    // Get recent commits
    let commits = github_client.get_commits(&owner, &repo, Some(3)).await?;
    
    println!("\nRecent commits:");
    for commit in commits {
        println!("  {} - {}", &commit.sha[0..7], commit.message.lines().next().unwrap_or_default());
    }
    
    Ok(())
}

/// Show GitHub configuration status
async fn show_github_status() -> Result<()> {
    let config_manager = GitHubConfigManager::new()?;
    let config = config_manager.get_config();
    
    println!("GitHub Configuration:");
    
    // Check token
    if config.token.is_some() {
        branding::print_success("GitHub token: Configured");
    } else if std::env::var("GITHUB_TOKEN").is_ok() {
        branding::print_success("GitHub token: Using GITHUB_TOKEN environment variable");
    } else {
        branding::print_error("GitHub token: Not configured");
    }
    
    // Check API base URL
    if let Some(api_base) = &config.api_base {
        println!("GitHub API URL: {}", api_base);
    } else {
        println!("GitHub API URL: https://api.github.com (default)");
    }
    
    // Check default repository
    if let Some(owner) = &config.default_owner {
        if let Some(repo) = &config.default_repo {
            println!("Default repository: {}/{}", owner, repo);
        } else {
            println!("Default repository owner: {}", owner);
            branding::print_warning("Default repository name not configured");
        }
    } else {
        branding::print_warning("Default repository not configured");
    }
    
    Ok(())
}
