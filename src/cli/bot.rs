use anyhow::Result;
use clap::Subcommand;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::llm::{LlmRouter, LlmRequest, RouterConfig};
use crate::cli::branding;

// Define the QitOpsBot and BotConfig here
#[derive(Debug, Clone)]
pub struct BotConfig {
    /// System prompt
    pub system_prompt: String,

    /// Knowledge base path
    pub knowledge_base_path: Option<PathBuf>,

    /// Max history length
    #[allow(dead_code)]
    pub max_history_length: usize,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            knowledge_base_path: None,
            max_history_length: 10,
        }
    }
}

/// Default system prompt
const DEFAULT_SYSTEM_PROMPT: &str = r#"You are QitOps Bot, an assistant for the QitOps Agent toolchain.
Your purpose is to help users learn and use QitOps Agent effectively.

QitOps Agent is an AI-powered QA Assistant that helps improve software quality through automated analysis, testing, and risk assessment.

Key features of QitOps Agent:
1. Test case generation (qitops run test-gen)
2. Pull request analysis (qitops run pr-analyze)
3. Risk assessment (qitops run risk)
4. Test data generation (qitops run test-data)
5. Interactive testing sessions (qitops run session)

QitOps Agent also supports:
- Configurable LLM routing (qitops llm)
- GitHub integration (qitops github)
- Source management (qitops source)
- Persona management (qitops persona)

Be helpful, concise, and accurate. If you don't know something, say so.
Provide examples when appropriate.
"#;

/// Chat message
#[derive(Debug, Clone)]
pub enum ChatMessage {
    /// User message
    User(String),

    /// Bot message
    Bot(String),
}

pub struct QitOpsBot {
    /// LLM router
    llm_router: LlmRouter,

    /// Chat history
    chat_history: Vec<ChatMessage>,

    /// Bot configuration
    config: BotConfig,
}

impl QitOpsBot {
    /// Create a new QitOps Bot
    pub async fn new(llm_router: LlmRouter, config: Option<BotConfig>) -> Self {
        let config = config.unwrap_or_default();

        Self {
            llm_router,
            chat_history: Vec::new(),
            config,
        }
    }

    /// Start an interactive chat session
    pub async fn start_chat_session(&mut self) -> Result<()> {
        // Print welcome message
        branding::print_command_header("QitOps Bot");
        println!("Welcome to QitOps Bot! Type 'exit' or 'quit' to end the session.");
        println!();

        // Initial bot message
        let initial_message = "Hello! I'm the QitOps Bot. How can I help you with QitOps Agent today?";
        println!("{}: {}", branding::colorize("QitOps Bot", branding::Color::Green), initial_message);
        self.chat_history.push(ChatMessage::Bot(initial_message.to_string()));

        // Chat loop
        loop {
            // Get user input
            print!("{}: ", branding::colorize("You", branding::Color::Blue));
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            // Check for exit command
            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                println!("\n{}: Goodbye! Feel free to chat again if you need help with QitOps Agent.",
                    branding::colorize("QitOps Bot", branding::Color::Green));
                break;
            }

            // Process user message
            let response = self.process_message(input).await?;

            // Print bot response
            println!("{}: {}", branding::colorize("QitOps Bot", branding::Color::Green), response);
            println!();
        }

        Ok(())
    }

    /// Process a user message
    pub async fn process_message(&mut self, message: &str) -> Result<String> {
        // Add user message to chat history
        self.chat_history.push(ChatMessage::User(message.to_string()));

        // Create the LLM request
        let prompt = self.generate_prompt();
        let model = self.llm_router.default_model().unwrap_or_else(|| "mistral".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message(self.config.system_prompt.clone());

        // Send the request to the LLM
        let llm_response = self.llm_router.send(request, None).await?;

        // Extract the text from the response
        let response_text = llm_response.text;

        // Add bot response to chat history
        self.chat_history.push(ChatMessage::Bot(response_text.clone()));

        Ok(response_text)
    }

    /// Generate the prompt for the LLM
    fn generate_prompt(&self) -> String {
        // Convert chat history to a prompt
        let mut prompt = String::new();

        for message in &self.chat_history {
            match message {
                ChatMessage::User(text) => {
                    prompt.push_str(&format!("User: {}\n", text));
                },
                ChatMessage::Bot(text) => {
                    prompt.push_str(&format!("QitOps Bot: {}\n", text));
                },
            }
        }

        prompt
    }

    /// Execute a QitOps Agent command
    #[allow(dead_code)]
    pub async fn execute_command(&self, command: &str) -> Result<String> {
        // Parse the command
        let args = shlex::split(command).ok_or_else(|| anyhow::anyhow!("Failed to parse command"))?;

        // Create a new process
        let mut process = std::process::Command::new("qitops");
        process.args(&args);

        // Execute the command
        let output = process.output()?;

        // Return the output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !stderr.is_empty() {
            Ok(format!("Command output:\n{}\n\nErrors:\n{}", stdout, stderr))
        } else {
            Ok(format!("Command output:\n{}", stdout))
        }
    }
}

/// Bot CLI arguments
#[derive(Debug, clap::Args)]
pub struct BotArgs {
    /// Bot subcommand
    #[clap(subcommand)]
    pub command: BotCommand,
}

/// Bot subcommands
#[derive(Debug, Subcommand)]
pub enum BotCommand {
    /// Start a chat session with QitOps Bot
    #[clap(name = "chat")]
    Chat {
        /// System prompt file
        #[clap(short, long)]
        system_prompt: Option<String>,

        /// Knowledge base path
        #[clap(short, long)]
        knowledge_base: Option<String>,
    },
}

/// Handle bot commands
pub async fn handle_bot_command(args: &BotArgs) -> Result<()> {
    match &args.command {
        BotCommand::Chat { system_prompt, knowledge_base } => {
            chat(system_prompt, knowledge_base).await
        },
    }
}

/// Start a chat session with QitOps Bot
async fn chat(system_prompt: &Option<String>, knowledge_base: &Option<String>) -> Result<()> {
    // Initialize LLM router
    let llm_router = LlmRouter::new(RouterConfig::default()).await?;

    // Create bot configuration
    let mut config = BotConfig::default();

    // Load system prompt from file if provided
    if let Some(system_prompt_path) = system_prompt {
        let system_prompt_content = std::fs::read_to_string(system_prompt_path)?;
        config.system_prompt = system_prompt_content;
    }

    // Set knowledge base path if provided
    if let Some(kb_path) = knowledge_base {
        config.knowledge_base_path = Some(std::path::PathBuf::from(kb_path));
    }

    // Create QitOps Bot
    let mut bot = QitOpsBot::new(llm_router, Some(config)).await;

    // Start chat session
    bot.start_chat_session().await?;

    Ok(())
}
