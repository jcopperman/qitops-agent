use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;

pub mod knowledge;
use knowledge::KnowledgeBase;

pub mod tutorial;
use tutorial::{TutorialManager, TutorialSession};

use crate::llm::{LlmRouter, LlmRequest};
use crate::cli::branding;

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessage {
    /// User message
    User(String),

    /// Bot message
    Bot(String),

    /// System message
    System(String),
}

/// Bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// System prompt
    pub system_prompt: String,

    /// Knowledge base path
    pub knowledge_base_path: Option<PathBuf>,

    /// Tutorial path
    pub tutorial_path: Option<PathBuf>,

    /// Max history length
    pub max_history_length: usize,

    /// Show onboarding tutorial for first-time users
    pub show_onboarding: bool,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            knowledge_base_path: None,
            tutorial_path: Some(PathBuf::from("tutorials")),
            max_history_length: 10,
            show_onboarding: true,
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

/// QitOps Bot
pub struct QitOpsBot {
    /// LLM router
    llm_router: LlmRouter,

    /// Chat history
    chat_history: Vec<ChatMessage>,

    /// Bot configuration
    config: BotConfig,

    /// Knowledge base
    knowledge_base: Option<KnowledgeBase>,

    /// Tutorial manager
    tutorial_manager: Option<TutorialManager>,

    /// Active tutorial session
    active_tutorial: Option<TutorialSession>,

    /// First-time user flag
    is_first_time_user: bool,
}

impl QitOpsBot {
    /// Create a new QitOps Bot
    pub async fn new(llm_router: LlmRouter, config: Option<BotConfig>) -> Self {
        let config = config.unwrap_or_default();

        // Load knowledge base if path is provided
        let knowledge_base = if let Some(kb_path) = &config.knowledge_base_path {
            match KnowledgeBase::load(kb_path) {
                Ok(kb) => {
                    tracing::info!("Loaded knowledge base from {}", kb_path.display());
                    Some(kb)
                },
                Err(e) => {
                    tracing::warn!("Failed to load knowledge base: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize tutorial manager
        let tutorial_dir = PathBuf::from("tutorials");
        let tutorial_manager = match TutorialManager::new(tutorial_dir) {
            Ok(manager) => {
                tracing::info!("Initialized tutorial manager");
                Some(manager)
            },
            Err(e) => {
                tracing::warn!("Failed to initialize tutorial manager: {}", e);
                None
            }
        };

        // Check if this is a first-time user
        let is_first_time_user = !PathBuf::from("chat_sessions").exists();

        Self {
            llm_router,
            chat_history: Vec::new(),
            config,
            knowledge_base,
            tutorial_manager,
            active_tutorial: None,
            is_first_time_user,
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

        // Show help message
        let help_message = "Type !help to see available commands.";
        println!("{}", help_message);
        self.chat_history.push(ChatMessage::System(help_message.to_string()));

        // Offer onboarding tutorial to first-time users
        if self.is_first_time_user {
            println!();
            let onboarding_message = "It looks like this is your first time using QitOps Bot. Would you like to take a quick onboarding tutorial to learn the basics? (yes/no)";
            println!("{}: {}", branding::colorize("QitOps Bot", branding::Color::Green), onboarding_message);
            self.chat_history.push(ChatMessage::Bot(onboarding_message.to_string()));

            // Get user response
            print!("{}: ", branding::colorize("You", branding::Color::Blue));
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input == "yes" || input == "y" {
                // Start onboarding tutorial
                self.start_tutorial("onboarding").await?;
            } else {
                println!("{}: No problem! You can always start a tutorial later by typing !tutorial",
                    branding::colorize("QitOps Bot", branding::Color::Green));
            }
        }

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

        // Trim chat history if it's too long
        if self.chat_history.len() > self.config.max_history_length * 2 {
            let new_start = self.chat_history.len() - self.config.max_history_length * 2;
            self.chat_history = self.chat_history[new_start..].to_vec();
        }

        // Check if there's an active tutorial and process tutorial navigation commands
        if self.active_tutorial.is_some() {
            // Tutorial navigation commands
            if message == "!next" {
                if let Err(e) = self.next_tutorial_step() {
                    return Ok(format!("Error: {}", e));
                }
                return Ok("Moving to the next step.".to_string());
            } else if message == "!prev" {
                if let Err(e) = self.previous_tutorial_step() {
                    return Ok(format!("Error: {}", e));
                }
                return Ok("Moving to the previous step.".to_string());
            } else if message == "!exit-tutorial" {
                if let Err(e) = self.exit_tutorial() {
                    return Ok(format!("Error: {}", e));
                }
                return Ok("Tutorial exited.".to_string());
            }

            // Check if the message matches the expected action in the current tutorial step
            if let Some(session) = &self.active_tutorial {
                if let Some(step) = session.current_step() {
                    if let Some(expected_action) = &step.example {
                        if message.trim() == expected_action.trim() {
                            // User entered the expected command, execute it
                            let result = self.execute_command(message).await?;
                            let response = format!("I executed the command: `{}`\n\nResult:\n```\n{}\n```\n\nGreat job! Type !next to continue to the next step.", message, result);

                            // Add bot response to chat history
                            self.chat_history.push(ChatMessage::Bot(response.clone()));

                            return Ok(response);
                        }
                    }
                }
            }
        }

        // Check if the message is a help request
        if message == "!help" {
            let help_text = self.get_help_text();
            self.chat_history.push(ChatMessage::System(help_text.clone()));
            return Ok(help_text);
        }

        // Check if the message is a tutorial command
        if message == "!tutorial" {
            match self.list_tutorials() {
                Ok(tutorials) => {
                    self.chat_history.push(ChatMessage::System(tutorials.clone()));
                    return Ok(tutorials);
                }
                Err(e) => {
                    let error = format!("Error listing tutorials: {}", e);
                    self.chat_history.push(ChatMessage::System(error.clone()));
                    return Ok(error);
                }
            }
        } else if message.starts_with("!tutorial ") {
            let tutorial_id = message.trim_start_matches("!tutorial ").trim();
            match self.start_tutorial(tutorial_id).await {
                Ok(_) => return Ok(format!("Started tutorial: {}", tutorial_id)),
                Err(e) => {
                    let error = format!("Error starting tutorial: {}", e);
                    self.chat_history.push(ChatMessage::System(error.clone()));
                    return Ok(error);
                }
            }
        }

        // Check if the message is a command execution request
        if message.starts_with("!exec ") {
            let command = message.trim_start_matches("!exec ").trim();
            let result = self.execute_command(command).await?;
            let response = format!("I executed the command: `{}`\n\nResult:\n```\n{}\n```", command, result);

            // Add bot response to chat history
            self.chat_history.push(ChatMessage::Bot(response.clone()));

            return Ok(response);
        }

        // Create the LLM request
        let prompt = self.generate_prompt();
        let model = self.llm_router.default_model().unwrap_or_else(|| "mistral".to_string());
        let mut request = LlmRequest::new(prompt, model)
            .with_system_message(self.config.system_prompt.clone());

        // Add knowledge base information if available
        if let Some(kb) = &self.knowledge_base {
            // Try to find relevant information based on the user's message
            let mut kb_info = String::new();

            // Check for command-related questions
            for (cmd_name, cmd_doc) in &kb.commands {
                if message.to_lowercase().contains(&cmd_name.to_lowercase()) {
                    kb_info.push_str(&format!("Command: {}\n", cmd_name));
                    kb_info.push_str(&format!("Description: {}\n", cmd_doc.description));
                    kb_info.push_str(&format!("Usage: {}\n", cmd_doc.usage));
                    kb_info.push_str("Examples:\n");
                    for example in &cmd_doc.examples {
                        kb_info.push_str(&format!("- {}\n", example));
                    }
                    kb_info.push_str("Options:\n");
                    for (option, desc) in &cmd_doc.options {
                        kb_info.push_str(&format!("- {}: {}\n", option, desc));
                    }
                    kb_info.push('\n');
                }
            }

            // Check for FAQ-related questions
            let faq_entries = kb.search_faq(message);
            if !faq_entries.is_empty() {
                kb_info.push_str("Relevant FAQ entries:\n");
                for entry in faq_entries.iter().take(2) {
                    kb_info.push_str(&format!("Q: {}\n", entry.question));
                    kb_info.push_str(&format!("A: {}\n\n", entry.answer));
                }
            }

            // Check for example-related questions
            let examples = kb.search_examples(message);
            if !examples.is_empty() {
                kb_info.push_str("Relevant examples:\n");
                for example in examples.iter().take(3) {
                    kb_info.push_str(&format!("Title: {}\n", example.title));
                    kb_info.push_str(&format!("Description: {}\n", example.description));
                    kb_info.push_str(&format!("Code: {}\n\n", example.code));
                }
            }

            // Add knowledge base information to the request
            if !kb_info.is_empty() {
                request = request.with_additional_context(format!("Knowledge base information:\n{}\n", kb_info));
            }
        }

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
                ChatMessage::System(text) => {
                    prompt.push_str(&format!("System: {}\n", text));
                },
            }
        }

        prompt
    }

    /// Execute a QitOps Agent command
    pub async fn execute_command(&self, command: &str) -> Result<String> {
        // Parse the command
        let args = shlex::split(command).ok_or_else(|| anyhow!("Failed to parse command"))?;

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

    /// Start a tutorial
    pub async fn start_tutorial(&mut self, tutorial_id: &str) -> Result<()> {
        // Check if tutorial manager is available
        let tutorial_manager = match &self.tutorial_manager {
            Some(manager) => manager,
            None => return Err(anyhow!("Tutorial manager not available")),
        };

        // Get the tutorial
        let tutorial = match tutorial_manager.get_tutorial(tutorial_id) {
            Some(tutorial) => tutorial.clone(),
            None => return Err(anyhow!("Tutorial not found: {}", tutorial_id)),
        };

        // Create a new tutorial session
        let session = TutorialSession::new(tutorial);
        self.active_tutorial = Some(session);

        // Show the first step
        self.show_current_tutorial_step()
    }

    /// Show the current tutorial step
    pub fn show_current_tutorial_step(&self) -> Result<()> {
        // Check if there's an active tutorial
        let session = match &self.active_tutorial {
            Some(session) => session,
            None => return Err(anyhow!("No active tutorial")),
        };

        // Format and print the current step
        let step_text = session.format_current_step();
        println!("{}: {}\n", branding::colorize("Tutorial", branding::Color::Cyan), step_text);

        Ok(())
    }

    /// Move to the next tutorial step
    pub fn next_tutorial_step(&mut self) -> Result<()> {
        // Check if there's an active tutorial
        let session = match &mut self.active_tutorial {
            Some(session) => session,
            None => return Err(anyhow!("No active tutorial")),
        };

        // Move to the next step
        if session.next_step().is_none() {
            // Tutorial completed
            println!("{}: {}\n",
                branding::colorize("Tutorial", branding::Color::Cyan),
                "Congratulations! You've completed the tutorial.");

            // Clear the active tutorial
            self.active_tutorial = None;

            return Ok(());
        }

        // Show the current step
        self.show_current_tutorial_step()
    }

    /// Move to the previous tutorial step
    pub fn previous_tutorial_step(&mut self) -> Result<()> {
        // Check if there's an active tutorial
        let session = match &mut self.active_tutorial {
            Some(session) => session,
            None => return Err(anyhow!("No active tutorial")),
        };

        // Move to the previous step
        session.previous_step();

        // Show the current step
        self.show_current_tutorial_step()
    }

    /// Exit the current tutorial
    pub fn exit_tutorial(&mut self) -> Result<()> {
        // Check if there's an active tutorial
        if self.active_tutorial.is_none() {
            return Err(anyhow!("No active tutorial"));
        }

        // Clear the active tutorial
        self.active_tutorial = None;

        println!("{}: {}\n",
            branding::colorize("Tutorial", branding::Color::Cyan),
            "Tutorial exited. You can start another tutorial by typing !tutorial");

        Ok(())
    }

    /// List available tutorials
    pub fn list_tutorials(&self) -> Result<String> {
        // Check if tutorial manager is available
        let tutorial_manager = match &self.tutorial_manager {
            Some(manager) => manager,
            None => return Err(anyhow!("Tutorial manager not available")),
        };

        // Get all tutorials
        let tutorials = tutorial_manager.get_all_tutorials();

        if tutorials.is_empty() {
            return Ok("No tutorials available.".to_string());
        }

        // Format the tutorial list
        let mut result = String::new();
        result.push_str("Available Tutorials:\n\n");
        result.push_str(&tutorial_manager.format_tutorial_list(tutorials));
        result.push_str("\nTo start a tutorial, type !tutorial <id>\n");

        Ok(result)
    }

    /// Get help text
    pub fn get_help_text(&self) -> String {
        let mut help = String::new();
        help.push_str("QitOps Bot Commands:\n\n");
        help.push_str("!help - Show this help message\n");
        help.push_str("!exec <command> - Execute a QitOps Agent command\n");
        help.push_str("!tutorial - List available tutorials\n");
        help.push_str("!tutorial <id> - Start a specific tutorial\n");
        help.push_str("!next - Move to the next tutorial step\n");
        help.push_str("!prev - Move to the previous tutorial step\n");
        help.push_str("!exit-tutorial - Exit the current tutorial\n");

        help.push_str("\nYou can also use natural language to execute commands. For example:\n");
        help.push_str("- 'Generate test cases for src/main.rs'\n");
        help.push_str("- 'Analyze pull request 123'\n");
        help.push_str("- 'Assess risk for changes.diff'\n");

        help
    }
}
