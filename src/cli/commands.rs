use clap::{Parser, Subcommand};

use crate::cli::llm::LlmArgs;
use crate::cli::github::GitHubArgs;
use crate::cli::source::SourceArgs;
use crate::cli::persona::PersonaArgs;
use crate::cli::bot::BotArgs;

/// QitOps Agent CLI
#[derive(Debug, Parser)]
#[clap(name = "qitops", about = "QitOps Agent - An AI-powered QA Assistant", long_about = "QitOps Agent is an AI-powered QA Assistant that helps you improve software quality through automated analysis, testing, and risk assessment.")]
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

    /// Source management (add, list, remove, show sources)
    #[clap(name = "source", about = "Manage sources for context-aware generation")]
    Source(SourceArgs),

    /// Persona management (add, list, remove, show personas)
    #[clap(name = "persona", about = "Manage personas for context-aware generation")]
    Persona(PersonaArgs),

    /// QitOps Bot - Interactive assistant
    #[clap(name = "bot", about = "Interactive assistant for QitOps Agent")]
    Bot(BotArgs),

    /// Monitoring commands
    #[clap(name = "monitoring", about = "Monitoring and metrics for QitOps Agent")]
    Monitoring {
        /// Monitoring subcommand
        #[clap(subcommand)]
        command: MonitoringCommand,
    },

    /// Plugin management
    #[clap(name = "plugin", about = "Manage plugins for QitOps Agent")]
    Plugin {
        /// Plugin subcommand
        #[clap(subcommand)]
        command: PluginCommand,
    },

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

        /// Sources to use (comma-separated)
        #[clap(long)]
        sources: Option<String>,

        /// Personas to use (comma-separated)
        #[clap(long)]
        personas: Option<String>,
    },

    /// Analyze a pull request
    #[clap(name = "pr-analyze")]
    PrAnalyze {
        /// PR number or URL
        #[clap(short, long)]
        pr: String,

        /// Sources to use (comma-separated)
        #[clap(long)]
        sources: Option<String>,

        /// Personas to use (comma-separated)
        #[clap(long)]
        personas: Option<String>,
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

        /// Sources to use (comma-separated)
        #[clap(long)]
        sources: Option<String>,

        /// Personas to use (comma-separated)
        #[clap(long)]
        personas: Option<String>,
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

        /// Sources to use (comma-separated)
        #[clap(long)]
        sources: Option<String>,

        /// Personas to use (comma-separated)
        #[clap(long)]
        personas: Option<String>,
    },

    /// Start an interactive testing session
    #[clap(name = "session")]
    Session {
        /// Session name
        #[clap(short, long)]
        name: String,

        /// Application under test
        #[clap(short, long)]
        application: Option<String>,

        /// Session type (exploratory, regression, user-journey, performance, security)
        #[clap(short, long)]
        session_type: Option<String>,

        /// Test objectives (comma-separated)
        #[clap(short, long)]
        objectives: Option<String>,

        /// Sources to use (comma-separated)
        #[clap(long)]
        sources: Option<String>,

        /// Personas to use (comma-separated)
        #[clap(long)]
        personas: Option<String>,
    },
}

/// Monitoring commands
#[derive(Debug, Subcommand)]
pub enum MonitoringCommand {
    /// Start the monitoring server
    #[clap(name = "start")]
    Start {
        /// Host to bind the server to
        #[clap(long, default_value = "127.0.0.1")]
        host: String,

        /// Port to bind the server to
        #[clap(long, default_value = "9090")]
        port: u16,

        /// Start Docker monitoring stack
        #[clap(long)]
        docker: bool,
    },

    /// Stop the monitoring server
    #[clap(name = "stop")]
    Stop {
        /// Stop Docker monitoring stack
        #[clap(long)]
        docker: bool,
    },

    /// Show monitoring status
    #[clap(name = "status")]
    Status,

    /// Show monitoring metrics
    #[clap(name = "metrics")]
    Metrics,
}

/// Plugin commands
#[derive(Debug, Subcommand)]
pub enum PluginCommand {
    /// List all plugins
    #[clap(name = "list")]
    List,

    /// Show plugin details
    #[clap(name = "show")]
    Show {
        /// Plugin ID
        #[clap(name = "id")]
        id: String,
    },

    /// Execute a plugin
    #[clap(name = "exec")]
    Execute {
        /// Plugin ID
        #[clap(name = "id")]
        id: String,

        /// Arguments to pass to the plugin
        #[clap(name = "args")]
        args: Vec<String>,
    },

    /// Enable the example plugin
    #[clap(name = "enable-example")]
    EnableExample,

    /// Disable the example plugin
    #[clap(name = "disable-example")]
    DisableExample,
}
