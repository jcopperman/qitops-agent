use anyhow::{Result, anyhow};
use clap::Subcommand;
use std::io::{self, Write};
use std::path::PathBuf;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use crate::llm::{LlmRouter, LlmRequest, ConfigManager};
use crate::cli::branding;

// Define the QitOpsBot and BotConfig here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// System prompt
    pub system_prompt: String,

    /// Knowledge base path
    pub knowledge_base_path: Option<PathBuf>,

    /// Tutorial path
    pub tutorial_path: Option<PathBuf>,

    /// Max history length
    #[allow(dead_code)]
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

/// Chat session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    /// Session name
    pub name: String,

    /// Session timestamp
    pub timestamp: u64,

    /// Chat history
    pub history: Vec<ChatMessage>,

    /// System prompt
    pub system_prompt: String,
}

pub struct QitOpsBot {
    /// LLM router
    llm_router: LlmRouter,

    /// Chat history
    chat_history: Vec<ChatMessage>,

    /// Bot configuration
    config: BotConfig,

    /// Session name
    session_name: String,

    /// Session timestamp
    session_timestamp: u64,

    /// Tutorial manager
    tutorial_manager: Option<crate::bot::tutorial::TutorialManager>,

    /// Active tutorial session
    active_tutorial: Option<crate::bot::tutorial::TutorialSession>,

    /// First-time user flag
    is_first_time_user: bool,
}

impl QitOpsBot {
    /// Create a new QitOps Bot
    pub async fn new(llm_router: LlmRouter, config: Option<BotConfig>) -> Self {
        let config = config.unwrap_or_default();

        // Generate a timestamp for the session
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Generate a session name based on the timestamp
        let session_name = format!("session_{}", timestamp);

        // Initialize tutorial manager
        let tutorial_dir = config.tutorial_path.clone().unwrap_or_else(|| PathBuf::from("tutorials"));
        let tutorial_manager = match crate::bot::tutorial::TutorialManager::new(tutorial_dir) {
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
            session_name,
            session_timestamp: timestamp,
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

        // Save initial chat history
        let _ = self.save_chat_history();

        // Offer onboarding tutorial to first-time users
        if self.is_first_time_user && self.config.show_onboarding {
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

                            // Save chat history
                            let _ = self.save_chat_history();

                            return Ok(response);
                        }
                    }
                }
            }
        }

        // Check for special commands
        if message.starts_with("!") {
            // Command execution request
            if message.starts_with("!exec ") {
                let command = message.trim_start_matches("!exec ").trim();
                let result = self.execute_command(command).await?;
                let response = format!("I executed the command: `{}`\n\nResult:\n```\n{}\n```", command, result);

                // Add bot response to chat history
                self.chat_history.push(ChatMessage::Bot(response.clone()));

                // Save chat history
                let _ = self.save_chat_history();

                return Ok(response);
            }

            // History command
            if message == "!history" {
                let response = self.format_chat_history();
                return Ok(response);
            }

            // Clear history command
            if message == "!clear" {
                self.chat_history.clear();
                self.chat_history.push(ChatMessage::System("Chat history cleared.".to_string()));
                return Ok("Chat history cleared.".to_string());
            }

            // Save history command
            if message == "!save" {
                match self.save_chat_history() {
                    Ok(file_path) => {
                        let response = format!("Chat history saved to: {}", file_path);
                        self.chat_history.push(ChatMessage::System(response.clone()));
                        return Ok(response);
                    }
                    Err(e) => {
                        let response = format!("Failed to save chat history: {}", e);
                        self.chat_history.push(ChatMessage::System(response.clone()));
                        return Ok(response);
                    }
                }
            }

            // Load history command
            if message.starts_with("!load ") {
                let session_name = message.trim_start_matches("!load ").trim();
                match self.load_chat_history(session_name) {
                    Ok(_) => {
                        let response = format!("Loaded chat history from session: {}", session_name);
                        self.chat_history.push(ChatMessage::System(response.clone()));
                        return Ok(response);
                    }
                    Err(e) => {
                        let response = format!("Failed to load chat history: {}", e);
                        self.chat_history.push(ChatMessage::System(response.clone()));
                        return Ok(response);
                    }
                }
            }

            // List sessions command
            if message == "!sessions" {
                match Self::list_chat_sessions() {
                    Ok(sessions) => {
                        if sessions.is_empty() {
                            let response = "No saved chat sessions found.".to_string();
                            self.chat_history.push(ChatMessage::System(response.clone()));
                            return Ok(response);
                        } else {
                            let response = format!("Available chat sessions:\n{}\n\nUse !load <session_name> to load a session.",
                                sessions.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"));
                            self.chat_history.push(ChatMessage::System(response.clone()));
                            return Ok(response);
                        }
                    }
                    Err(e) => {
                        let response = format!("Failed to list chat sessions: {}", e);
                        self.chat_history.push(ChatMessage::System(response.clone()));
                        return Ok(response);
                    }
                }
            }

            // Help command
            if message == "!help" {
                let response = self.get_help_text();
                self.chat_history.push(ChatMessage::System(response.clone()));
                return Ok(response);
            }

            // Feedback command
            if message.starts_with("!feedback ") {
                let feedback = message.trim_start_matches("!feedback ").trim();
                let response = self.process_feedback(feedback).await?;
                self.chat_history.push(ChatMessage::System(response.clone()));
                return Ok(response);
            }

            // Tutorial commands
            if message == "!tutorial" {
                match self.list_tutorials() {
                    Ok(tutorials) => {
                        self.chat_history.push(ChatMessage::System(tutorials.clone()));
                        return Ok(tutorials);
                    }
                    Err(e) => {
                        let response = format!("Error listing tutorials: {}", e);
                        self.chat_history.push(ChatMessage::System(response.clone()));
                        return Ok(response);
                    }
                }
            } else if message.starts_with("!tutorial ") {
                let tutorial_id = message.trim_start_matches("!tutorial ").trim();
                match self.start_tutorial(tutorial_id).await {
                    Ok(_) => {
                        let response = format!("Started tutorial: {}", tutorial_id);
                        self.chat_history.push(ChatMessage::System(response.clone()));
                        return Ok(response);
                    }
                    Err(e) => {
                        let response = format!("Error starting tutorial: {}", e);
                        self.chat_history.push(ChatMessage::System(response.clone()));
                        return Ok(response);
                    }
                }
            }
        }

        // Check if the message is a help request for a specific command
        if message.to_lowercase().contains("how to") || message.to_lowercase().contains("help with") || message.to_lowercase().contains("explain") {
            if let Some(response) = self.provide_interactive_help(message).await? {
                // Add bot response to chat history
                self.chat_history.push(ChatMessage::Bot(response.clone()));

                // Save chat history
                let _ = self.save_chat_history();

                return Ok(response);
            }
        }

        // Check if the message is a natural language command
        if let Some(command) = self.parse_natural_language_command(message).await? {
            let result = self.execute_command(&command).await?;
            let response = format!("I interpreted your request as the command: `{}`\n\nResult:\n```\n{}\n```\n\nIf this wasn't what you intended, you can provide feedback with !feedback", command, result);

            // Add bot response to chat history
            self.chat_history.push(ChatMessage::Bot(response.clone()));

            // Save chat history
            let _ = self.save_chat_history();

            return Ok(response);
        }

        // Create the LLM request
        let prompt = self.generate_prompt();
        let model = self.llm_router.default_model().unwrap_or_else(|| "mistral".to_string());
        let mut request = LlmRequest::new(prompt, model)
            .with_system_message(self.config.system_prompt.clone());

        // Add knowledge base information if available
        if let Some(kb_path) = &self.config.knowledge_base_path {
            if let Ok(kb_info) = self.get_knowledge_base_info(message, kb_path) {
                if !kb_info.is_empty() {
                    request = request.with_additional_context(format!("Knowledge base information:\n{}\n", kb_info));
                }
            }
        }

        // Send the request to the LLM
        let llm_response = self.llm_router.send(request, None).await?;

        // Extract the text from the response
        let response_text = llm_response.text;

        // Add bot response to chat history
        self.chat_history.push(ChatMessage::Bot(response_text.clone()));

        // Save chat history
        let _ = self.save_chat_history();

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

    /// Save the chat history to a file
    pub fn save_chat_history(&self) -> Result<String> {
        // Create the chat sessions directory if it doesn't exist
        let sessions_dir = PathBuf::from("chat_sessions");
        if !sessions_dir.exists() {
            fs::create_dir_all(&sessions_dir)?;
        }

        // Create a chat session object
        let session = ChatSession {
            name: self.session_name.clone(),
            timestamp: self.session_timestamp,
            history: self.chat_history.clone(),
            system_prompt: self.config.system_prompt.clone(),
        };

        // Serialize the chat session
        let session_json = serde_json::to_string_pretty(&session)
            .map_err(|e| anyhow!("Failed to serialize chat session: {}", e))?;

        // Save the chat session to a file
        let file_path = sessions_dir.join(format!("{}.json", self.session_name));
        fs::write(&file_path, session_json)
            .map_err(|e| anyhow!("Failed to write chat session file: {}", e))?;

        Ok(file_path.to_string_lossy().to_string())
    }

    /// Load a chat session from a file
    pub fn load_chat_history(&mut self, session_name: &str) -> Result<()> {
        // Get the chat sessions directory
        let sessions_dir = PathBuf::from("chat_sessions");
        if !sessions_dir.exists() {
            return Err(anyhow!("No chat sessions found"));
        }

        // Get the session file path
        let file_path = sessions_dir.join(format!("{}.json", session_name));
        if !file_path.exists() {
            return Err(anyhow!("Chat session not found: {}", session_name));
        }

        // Read the session file
        let session_json = fs::read_to_string(&file_path)
            .map_err(|e| anyhow!("Failed to read chat session file: {}", e))?;

        // Deserialize the chat session
        let session: ChatSession = serde_json::from_str(&session_json)
            .map_err(|e| anyhow!("Failed to deserialize chat session: {}", e))?;

        // Update the bot with the session data
        self.chat_history = session.history;
        self.session_name = session.name;
        self.session_timestamp = session.timestamp;
        self.config.system_prompt = session.system_prompt;

        Ok(())
    }

    /// Format chat history as a string
    fn format_chat_history(&self) -> String {
        let mut history = String::new();
        history.push_str("Chat History:\n\n");

        for (i, message) in self.chat_history.iter().enumerate() {
            match message {
                ChatMessage::User(text) => {
                    history.push_str(&format!("[{}] User: {}\n", i + 1, text));
                },
                ChatMessage::Bot(text) => {
                    history.push_str(&format!("[{}] QitOps Bot: {}\n", i + 1, text));
                },
                ChatMessage::System(text) => {
                    history.push_str(&format!("[{}] System: {}\n", i + 1, text));
                },
            }
            history.push('\n');
        }

        history
    }

    /// Get help text
    fn get_help_text(&self) -> String {
        let mut help = String::new();
        help.push_str("QitOps Bot Commands:\n\n");
        help.push_str("!help - Show this help message\n");
        help.push_str("!exec <command> - Execute a QitOps Agent command\n");
        help.push_str("!history - Show chat history\n");
        help.push_str("!clear - Clear chat history\n");
        help.push_str("!save - Save chat history to a file\n");
        help.push_str("!sessions - List available chat sessions\n");
        help.push_str("!load <session_name> - Load a chat session\n");
        help.push_str("!feedback <message> - Provide feedback on command interpretation\n");
        help.push_str("!tutorial - List available tutorials\n");
        help.push_str("!tutorial <id> - Start a specific tutorial\n");

        // Add tutorial navigation commands if a tutorial is active
        if self.active_tutorial.is_some() {
            help.push_str("\nTutorial Navigation Commands:\n");
            help.push_str("!next - Move to the next tutorial step\n");
            help.push_str("!prev - Move to the previous tutorial step\n");
            help.push_str("!exit-tutorial - Exit the current tutorial\n");
        }

        help.push_str("\nYou can also use natural language to execute commands. For example:\n");
        help.push_str("- 'Generate test cases for src/main.rs'\n");
        help.push_str("- 'Analyze pull request 123'\n");
        help.push_str("- 'Assess risk for changes.diff'\n");
        help.push_str("\nIf the bot misinterprets your command, you can provide feedback:\n");
        help.push_str("- '!feedback That's not what I meant. I wanted to analyze PR 456, not generate tests.'\n");

        help
    }

    /// List available chat sessions
    pub fn list_chat_sessions() -> Result<Vec<String>> {
        // Get the chat sessions directory
        let sessions_dir = PathBuf::from("chat_sessions");
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        // Get all JSON files in the directory
        let mut sessions = Vec::new();
        for entry in fs::read_dir(sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(file_name) = path.file_stem() {
                    if let Some(session_name) = file_name.to_str() {
                        sessions.push(session_name.to_string());
                    }
                }
            }
        }

        // Sort sessions by name (which includes timestamp)
        sessions.sort();

        Ok(sessions)
    }

    /// Execute a QitOps Agent command with improved error handling
    pub async fn execute_command(&self, command: &str) -> Result<String> {
        // Parse the command
        let args = match shlex::split(command) {
            Some(args) => args,
            None => return Ok(format!("Failed to parse command: '{}'. Please check the syntax.", command)),
        };

        if args.is_empty() {
            return Ok("No command specified. Please provide a valid QitOps command.".to_string());
        }

        // Create a new process
        let mut process = std::process::Command::new("qitops");
        process.args(&args);

        // Execute the command
        let output = match process.output() {
            Ok(output) => output,
            Err(e) => {
                // Provide helpful suggestions based on the error
                let error_msg = e.to_string();
                let suggestion = if error_msg.contains("No such file or directory") {
                    "QitOps executable not found. Make sure QitOps is installed and in your PATH."
                } else if error_msg.contains("Permission denied") {
                    "Permission denied. Make sure you have the necessary permissions to run QitOps."
                } else {
                    "An error occurred while executing the command."
                };

                return Ok(format!("Error: {}\n\nSuggestion: {}", error_msg, suggestion));
            }
        };

        // Process the output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_status = output.status;

        // Check for common error patterns in stderr
        let mut error_suggestion = String::new();
        if !stderr.is_empty() {
            if stderr.contains("No such file or directory") && command.contains("--path") {
                error_suggestion = "\n\nSuggestion: The specified file or directory does not exist. Check the path and try again.".to_string();
            } else if stderr.contains("Permission denied") {
                error_suggestion = "\n\nSuggestion: Permission denied. Check file permissions or try running with elevated privileges.".to_string();
            } else if stderr.contains("Invalid value") || stderr.contains("required") {
                error_suggestion = "\n\nSuggestion: The command has invalid or missing parameters. Check the command syntax and try again.".to_string();
            } else if stderr.contains("API key") || stderr.contains("authentication") {
                error_suggestion = "\n\nSuggestion: Authentication failed. Check your API key or token and try again.".to_string();
            }
        }

        // Format the response based on exit status and output
        if exit_status.success() {
            if !stderr.is_empty() {
                Ok(format!("Command output:\n{}\n\nWarnings:\n{}{}", stdout, stderr, error_suggestion))
            } else {
                Ok(format!("Command output:\n{}", stdout))
            }
        } else {
            // Command failed, provide a more helpful error message
            let exit_code = exit_status.code().unwrap_or(-1);
            Ok(format!("Command failed with exit code {}:\n\nErrors:\n{}{}", exit_code, stderr, error_suggestion))
        }
    }

    /// Parse a natural language command
    /// Parse a natural language command
    pub async fn parse_natural_language_command(&self, message: &str) -> Result<Option<String>> {
        // Check if the message looks like a command request
        let command_indicators = [
            "run", "execute", "start", "generate", "analyze", "test", "create",
            "show", "list", "add", "remove", "set", "config", "help", "check",
            "assess", "evaluate", "find", "search", "get", "make", "build", "setup"
        ];

        let is_command_request = command_indicators.iter().any(|&indicator| {
            message.to_lowercase().contains(&format!(" {} ", indicator)) ||
            message.to_lowercase().starts_with(&format!("{} ", indicator)) ||
            message.to_lowercase().contains(&format!("{} ", indicator))
        });

        // Command-specific indicators
        let command_specific = [
            // test-gen indicators
            "test case", "test cases", "unit test", "generate test", "create test",
            // pr-analyze indicators
            "pull request", "pr", "analyze pr", "review pr", "check pr",
            // risk indicators
            "risk", "assess risk", "evaluate risk", "risk assessment",
            // test-data indicators
            "test data", "generate data", "sample data", "mock data",
            // session indicators
            "session", "testing session", "interactive session",
            // llm indicators
            "llm", "language model", "ai model", "model",
            // github indicators
            "github", "git", "repository", "repo",
            // source indicators
            "source", "context source", "knowledge source",
            // persona indicators
            "persona", "role", "perspective"
        ];

        // Check for command-specific indicators
        let has_specific_indicator = command_specific.iter().any(|&indicator| {
            message.to_lowercase().contains(indicator)
        });

        // If no indicators are found, it's probably not a command request
        if !is_command_request && !has_specific_indicator {
            return Ok(None);
        }

        // Create a prompt for the LLM to parse the natural language command
        let prompt = format!(
            "Convert the following natural language request into a QitOps Agent command.\n\n\
            Request: {}\n\n\
            Respond with ONLY the command, without any explanation or markdown formatting.\n\
            If you're not sure, respond with 'UNKNOWN'.\n\n\
            Available commands and their purposes:\n\
            1. Test Generation:\n\
               - qitops run test-gen --path <file_path> [--format <format>] [--sources <sources>] [--personas <personas>]\n\
               - Purpose: Generate test cases for source code files\n\
               - Example inputs: 'Generate tests for auth.js', 'Create unit tests for the user module'\n\
            2. PR Analysis:\n\
               - qitops run pr-analyze --pr <pr_number> [--sources <sources>] [--personas <personas>]\n\
               - Purpose: Analyze pull requests for quality, risks, and test coverage\n\
               - Example inputs: 'Analyze PR 123', 'Review pull request #456'\n\
            3. Risk Assessment:\n\
               - qitops run risk --diff <diff_path> [--components <components>] [--focus <focus_areas>]\n\
               - Purpose: Assess risk of code changes\n\
               - Example inputs: 'Assess risk for changes.diff', 'Evaluate risk in the payment module'\n\
            4. Test Data Generation:\n\
               - qitops run test-data --schema <schema> --count <count> [--format <format>]\n\
               - Purpose: Generate test data based on a schema\n\
               - Example inputs: 'Generate 10 user profiles', 'Create 50 sample transactions'\n\
            5. Testing Session:\n\
               - qitops run session --name <name> [--application <app>] [--focus <focus>]\n\
               - Purpose: Start an interactive testing session\n\
               - Example inputs: 'Start a testing session for login flow', 'Begin a test session for the API'\n\
            6. LLM Management:\n\
               - qitops llm list\n\
               - qitops llm add --provider <provider> --api-key <api_key> [--api-base <api_base>] [--model <model>]\n\
               - qitops llm remove --provider <provider>\n\
               - qitops llm set-default --provider <provider>\n\
               - qitops llm test [--provider <provider>] [--prompt <prompt>] [--no-cache]\n\
               - Purpose: Manage LLM providers and settings\n\
               - Example inputs: 'List available LLMs', 'Set OpenAI as default provider'\n\
            7. GitHub Integration:\n\
               - qitops github config --token <token> [--owner <owner>] [--repo <repo>]\n\
               - Purpose: Configure GitHub integration\n\
               - Example inputs: 'Setup GitHub integration', 'Configure GitHub with my token'\n\
            8. Source Management:\n\
               - qitops source list\n\
               - qitops source show --id <id>\n\
               - Purpose: Manage context sources\n\
               - Example inputs: 'Show available sources', 'Display source requirements'\n\
            9. Persona Management:\n\
               - qitops persona list\n\
               - qitops persona show --id <id>\n\
               - Purpose: Manage personas for context\n\
               - Example inputs: 'List available personas', 'Show the QA engineer persona'\n\
            Guidelines for parsing:\n\
            - For file paths, use the exact path mentioned or a reasonable default if not specified\n\
            - For PR numbers, extract the number from the request\n\
            - For formats, default to 'markdown' unless another format is specified\n\
            - For counts, use the number mentioned or a reasonable default (e.g., 10)\n\
            - For names, use the exact name mentioned or a reasonable default based on the context\n\
            - If multiple commands could apply, choose the most specific one\n\
            - If essential parameters are missing, make a reasonable guess based on the context\
            ",
            message
        );

        // Send the request to the LLM
        let model = self.llm_router.default_model().unwrap_or_else(|| "mistral".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message("You are a command parser for QitOps Agent. Your task is to convert natural language requests into valid QitOps Agent commands. Be precise and follow the format exactly. Only return the command itself without any explanation.".to_string());

        let llm_response = self.llm_router.send(request, None).await?;
        let command = llm_response.text.trim();

        // Check if the LLM couldn't parse the command
        if command == "UNKNOWN" || command.contains("I'm not sure") || command.contains("I don't know") {
            return Ok(None);
        }

        // Remove any markdown formatting
        let command = command.trim_start_matches("```").trim_end_matches("```").trim();
        let command = command.trim_start_matches("bash").trim();
        let command = command.trim_start_matches("qitops ").trim();

        // Log the parsed command for debugging
        tracing::debug!("Parsed command: {}", command);

        Ok(Some(command.to_string()))
    }

    /// Provide interactive help for complex commands
    pub async fn provide_interactive_help(&self, message: &str) -> Result<Option<String>> {
        // Extract the command or topic the user needs help with
        let command_indicators = [
            "test-gen", "pr-analyze", "risk", "test-data", "session",
            "llm", "github", "source", "persona", "bot"
        ];

        // Find which command the user is asking about
        let mut target_command = None;
        for &indicator in command_indicators.iter() {
            if message.to_lowercase().contains(indicator) {
                target_command = Some(indicator);
                break;
            }
        }

        // If no specific command was found, check for general topics
        if target_command.is_none() {
            let topic_indicators = [
                "test", "pr", "pull request", "risk", "data", "session",
                "llm", "model", "github", "source", "persona", "bot", "chat"
            ];

            for &indicator in topic_indicators.iter() {
                if message.to_lowercase().contains(indicator) {
                    // Map general topics to commands
                    target_command = match indicator {
                        "test" => Some("test-gen"),
                        "pr" | "pull request" => Some("pr-analyze"),
                        "data" => Some("test-data"),
                        "model" => Some("llm"),
                        "chat" => Some("bot"),
                        _ => Some(indicator),
                    };
                    break;
                }
            }
        }

        // If we still don't have a target command, return None
        let target_command = match target_command {
            Some(cmd) => cmd,
            None => return Ok(None),
        };

        // Get knowledge base information if available
        let mut kb_info = String::new();
        if let Some(kb_path) = &self.config.knowledge_base_path {
            if let Ok(info) = self.get_knowledge_base_info(&format!("help with {}", target_command), kb_path) {
                if !info.is_empty() {
                    kb_info = info;
                }
            }
        }

        // Create a prompt for the LLM to generate interactive help
        let prompt = format!(
            "The user is asking for help with the '{}' command or feature in QitOps Agent.\n\n\
            User message: {}\n\n\
            Knowledge base information:\n{}\n\n\
            Provide a detailed, step-by-step guide on how to use this command or feature.\n\
            Include:\n\
            1. A brief explanation of what the command does\n\
            2. The basic syntax and required parameters\n\
            3. Examples of common use cases\n\
            4. Tips for advanced usage\n\
            5. Common errors and how to fix them\n\
            Make the explanation conversational and easy to understand.\
            ",
            target_command, message, kb_info
        );

        // Send the request to the LLM
        let model = self.llm_router.default_model().unwrap_or_else(|| "mistral".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message("You are an AI assistant providing interactive help for QitOps Agent commands and features. Be detailed, clear, and helpful.".to_string());

        let llm_response = self.llm_router.send(request, None).await?;
        let help_text = llm_response.text.trim();

        Ok(Some(format!(
            "Here's help with the '{}' command:\n\n{}",
            target_command,
            help_text
        )))
    }

    /// Process user feedback on command interpretation
    pub async fn process_feedback(&self, feedback: &str) -> Result<String> {
        // Create a feedback file if it doesn't exist
        let feedback_dir = PathBuf::from("feedback");
        if !feedback_dir.exists() {
            fs::create_dir_all(&feedback_dir)?;
        }

        // Get the last few messages from the chat history to provide context
        let mut context = String::new();
        let history_len = self.chat_history.len();
        let start_idx = if history_len > 5 { history_len - 5 } else { 0 };

        for message in &self.chat_history[start_idx..] {
            match message {
                ChatMessage::User(text) => {
                    context.push_str(&format!("User: {}\n", text));
                },
                ChatMessage::Bot(text) => {
                    context.push_str(&format!("QitOps Bot: {}\n", text));
                },
                ChatMessage::System(_) => {}, // Skip system messages
            }
        }

        // Create a feedback entry
        let feedback_entry = serde_json::json!({
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            "context": context,
            "feedback": feedback,
        });

        // Save the feedback to a file
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let file_path = feedback_dir.join(format!("feedback_{}.json", timestamp));
        fs::write(&file_path, serde_json::to_string_pretty(&feedback_entry)?)?;

        // Analyze the feedback to improve future command parsing
        let prompt = format!(
            "Analyze the following user feedback about command interpretation and suggest improvements:\n\n\
            Context:\n{}\n\n\
            User Feedback: {}\n\n\
            Based on this feedback, what improvements could be made to the command parsing logic?\
            ",
            context, feedback
        );

        // Send the request to the LLM
        let model = self.llm_router.default_model().unwrap_or_else(|| "mistral".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message("You are an AI assistant helping to improve command parsing for QitOps Agent. Analyze user feedback and suggest concrete improvements.".to_string());

        let llm_response = self.llm_router.send(request, None).await?;
        let analysis = llm_response.text.trim();

        // Save the analysis to the feedback file
        let mut feedback_entry = serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&file_path)?)?;
        if let Some(obj) = feedback_entry.as_object_mut() {
            obj.insert("analysis".to_string(), serde_json::Value::String(analysis.to_string()));
            fs::write(&file_path, serde_json::to_string_pretty(&feedback_entry)?)?;
        }

        // Log the feedback and analysis
        tracing::info!("User feedback received: {}", feedback);
        tracing::info!("Feedback analysis: {}", analysis);

        Ok(format!("Thank you for your feedback! We'll use it to improve command interpretation. Your feedback has been saved to {}.", file_path.to_string_lossy()))
    }

    /// Get information from the knowledge base relevant to the user's message
    pub fn get_knowledge_base_info(&self, message: &str, kb_path: &PathBuf) -> Result<String> {
        use crate::bot::knowledge::KnowledgeBase;

        // Try to load the knowledge base
        let kb = match KnowledgeBase::load(kb_path) {
            Ok(kb) => kb,
            Err(e) => {
                tracing::warn!("Failed to load knowledge base: {}", e);
                return Ok(String::new());
            }
        };

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

        // Check for FAQ matches
        let faq_entries = kb.search_faq(message);
        if !faq_entries.is_empty() {
            kb_info.push_str("Relevant FAQs:\n");
            for entry in faq_entries.iter().take(3) {
                kb_info.push_str(&format!("Q: {}\n", entry.question));
                kb_info.push_str(&format!("A: {}\n\n", entry.answer));
            }
        }

        // Check for example matches
        let examples = kb.search_examples(message);
        if !examples.is_empty() {
            kb_info.push_str("Relevant Examples:\n");
            for example in examples.iter().take(2) {
                kb_info.push_str(&format!("Title: {}\n", example.title));
                kb_info.push_str(&format!("Description: {}\n", example.description));
                kb_info.push_str(&format!("Code: {}\n\n", example.code));
            }
        }

        Ok(kb_info)
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
        let session = crate::bot::tutorial::TutorialSession::new(tutorial);
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

        /// Tutorial path
        #[clap(short, long)]
        tutorial_path: Option<String>,

        /// Skip onboarding tutorial for first-time users
        #[clap(long)]
        skip_onboarding: bool,
    },

    /// List available tutorials
    #[clap(name = "tutorials")]
    ListTutorials {
        /// Tutorial path
        #[clap(short, long)]
        tutorial_path: Option<String>,
    },

    /// Start a specific tutorial
    #[clap(name = "tutorial")]
    StartTutorial {
        /// Tutorial ID
        #[clap(name = "id")]
        tutorial_id: String,

        /// Tutorial path
        #[clap(short, long)]
        tutorial_path: Option<String>,
    },
}

/// Handle bot commands
pub async fn handle_bot_command(args: &BotArgs) -> Result<()> {
    match &args.command {
        BotCommand::Chat { system_prompt, knowledge_base, tutorial_path, skip_onboarding } => {
            chat(system_prompt, knowledge_base, tutorial_path, *skip_onboarding).await
        },
        BotCommand::ListTutorials { tutorial_path } => {
            list_available_tutorials(tutorial_path).await
        },
        BotCommand::StartTutorial { tutorial_id, tutorial_path } => {
            start_tutorial(tutorial_id, tutorial_path).await
        },
    }
}

/// Start a chat session with QitOps Bot
async fn chat(system_prompt: &Option<String>, knowledge_base: &Option<String>, tutorial_path: &Option<String>, skip_onboarding: bool) -> Result<()> {
    // Initialize LLM router
    let progress = crate::cli::progress::ProgressIndicator::new("Initializing LLM router...");
    let config_manager = ConfigManager::new()?;
    let llm_router = LlmRouter::new(config_manager.get_config().clone()).await?;
    progress.finish();

    // Create bot configuration
    let mut config = BotConfig::default();

    // Load system prompt from file if provided
    if let Some(system_prompt_path) = system_prompt {
        let system_prompt_content = std::fs::read_to_string(system_prompt_path)?;
        config.system_prompt = system_prompt_content;
    }

    // Set knowledge base path if provided
    if let Some(kb_path) = knowledge_base {
        let kb_path_buf = std::path::PathBuf::from(kb_path);
        if kb_path_buf.exists() {
            config.knowledge_base_path = Some(kb_path_buf);
            println!("Using knowledge base from: {}", kb_path);
        } else {
            println!("Warning: Knowledge base path does not exist: {}", kb_path);
            println!("Continuing without knowledge base.");
        }
    }

    // Set tutorial path if provided
    if let Some(tutorial_path) = tutorial_path {
        let tutorial_path_buf = std::path::PathBuf::from(tutorial_path);
        if tutorial_path_buf.exists() {
            config.tutorial_path = Some(tutorial_path_buf);
            println!("Using tutorials from: {}", tutorial_path);
        } else {
            println!("Warning: Tutorial path does not exist: {}", tutorial_path);
            println!("Continuing with default tutorials.");
        }
    }

    // Set onboarding flag
    config.show_onboarding = !skip_onboarding;

    // Create QitOps Bot
    let mut bot = QitOpsBot::new(llm_router, Some(config)).await;

    // Start chat session
    bot.start_chat_session().await?;

    Ok(())
}

/// List available tutorials
async fn list_available_tutorials(tutorial_path: &Option<String>) -> Result<()> {
    // Initialize LLM router
    let progress = crate::cli::progress::ProgressIndicator::new("Initializing LLM router...");
    let config_manager = ConfigManager::new()?;
    let llm_router = LlmRouter::new(config_manager.get_config().clone()).await?;
    progress.finish();

    // Create bot configuration
    let mut config = BotConfig::default();

    // Set tutorial path if provided
    if let Some(tutorial_path) = tutorial_path {
        let tutorial_path_buf = std::path::PathBuf::from(tutorial_path);
        if tutorial_path_buf.exists() {
            config.tutorial_path = Some(tutorial_path_buf);
            println!("Using tutorials from: {}", tutorial_path);
        } else {
            println!("Warning: Tutorial path does not exist: {}", tutorial_path);
            println!("Continuing with default tutorials.");
        }
    }

    // Create QitOps Bot
    let bot = QitOpsBot::new(llm_router, Some(config)).await;

    // List tutorials
    match bot.list_tutorials() {
        Ok(tutorials) => {
            branding::print_command_header("Available Tutorials");
            println!("{}", tutorials);
        },
        Err(e) => {
            branding::print_error(&format!("Failed to list tutorials: {}", e));
        }
    }

    Ok(())
}

/// Start a specific tutorial
async fn start_tutorial(tutorial_id: &str, tutorial_path: &Option<String>) -> Result<()> {
    // Initialize LLM router
    let progress = crate::cli::progress::ProgressIndicator::new("Initializing LLM router...");
    let config_manager = ConfigManager::new()?;
    let llm_router = LlmRouter::new(config_manager.get_config().clone()).await?;
    progress.finish();

    // Create bot configuration
    let mut config = BotConfig::default();

    // Set tutorial path if provided
    if let Some(tutorial_path) = tutorial_path {
        let tutorial_path_buf = std::path::PathBuf::from(tutorial_path);
        if tutorial_path_buf.exists() {
            config.tutorial_path = Some(tutorial_path_buf);
            println!("Using tutorials from: {}", tutorial_path);
        } else {
            println!("Warning: Tutorial path does not exist: {}", tutorial_path);
            println!("Continuing with default tutorials.");
        }
    }

    // Create QitOps Bot
    let mut bot = QitOpsBot::new(llm_router, Some(config)).await;

    // Start tutorial
    match bot.start_tutorial(tutorial_id).await {
        Ok(_) => {
            // Start chat session to interact with the tutorial
            bot.start_chat_session().await?
        },
        Err(e) => {
            branding::print_error(&format!("Failed to start tutorial: {}", e));
        }
    }

    Ok(())
}
