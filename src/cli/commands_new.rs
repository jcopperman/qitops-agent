use clap::{Parser, Subcommand};

use crate::cli::llm::LlmArgs;
use crate::cli::github::GitHubArgs;

/// QitOps Agent CLI
#[derive(Debug, Parser)]
#[clap(name = "qitops", about = "QitOps Agent - An AI-powered QA Assistant")]
pub struct Cli {
    /// Enable verbose output
    #[clap(short, long)]
    pub verbose: bool,

    /// Subcommand to execute
    #[clap(subcommand)]
    pub command: Command,
}

/// CLI commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run QitOps commands
    #[clap(name = "run")]
    Run {
        /// Subcommand to run
        #[clap(subcommand)]
        command: RunCommand,
    },
    
    /// LLM configuration and management
    #[clap(name = "llm")]
    Llm(LlmArgs),

    /// GitHub integration
    #[clap(name = "github")]
    GitHub(GitHubArgs),
    
    /// Show version information
    #[clap(name = "version")]
    Version,
}

/// Run commands
#[derive(Debug, Subcommand)]
pub enum RunCommand {
    /// Generate test cases
    #[clap(name = "test-gen")]
    TestGen {
        /// Path to the source code
        #[clap(short, long)]
        path: String,

        /// Output format (markdown, yaml, robot)
        #[clap(short, long, default_value = "markdown")]
        format: String,
    },

    /// Analyze a pull request
    #[clap(name = "pr-analyze")]
    PrAnalyze {
        /// PR number or URL
        #[clap(short, long)]
        pr: String,
    },

    /// Estimate risk of changes
    #[clap(name = "risk")]
    Risk {
        /// Path to the diff file or PR URL/number
        #[clap(short, long)]
        diff: String,

        /// Components to focus on (comma-separated)
        #[clap(short, long)]
        components: Option<String>,

        /// Focus areas (comma-separated: security, performance, etc.)
        #[clap(short, long)]
        focus: Option<String>,
    },

    /// Generate test data
    #[clap(name = "test-data")]
    TestData {
        /// Schema definition
        #[clap(short, long)]
        schema: String,

        /// Number of records to generate
        #[clap(short, long, default_value = "10")]
        count: usize,
    },

    /// Start an interactive testing session
    #[clap(name = "session")]
    Session {
        /// Session name
        #[clap(short, long)]
        name: String,
    },
}
