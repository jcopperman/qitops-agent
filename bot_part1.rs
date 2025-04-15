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

        Self {
            llm_router,
            chat_history: Vec::new(),
            config,
            session_name,
            session_timestamp: timestamp,
        }
    }

    /// Start an interactive chat session
    pub async fn start_chat_session(&mut self) -> Result<()> {
        // Print welcome message
        branding::print_command_header("QitOps Bot");
        println!("Welcome to QitOps Bot! Type 'exit' or 'quit' to end the session.");
        println!();

        // Initial bot message
        let initial_message = "Hello! I'm the QitOps Bot. How can I help you with QitOps Agent today?\n\nType !help to see available commands.";
        println!("{}: {}", branding::colorize("QitOps Bot", branding::Color::Green), initial_message);
        self.chat_history.push(ChatMessage::Bot(initial_message.to_string()));

        // Save initial chat history
        let _ = self.save_chat_history();

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
