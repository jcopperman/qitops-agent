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
const DEFAULT_SYSTEM_PROMPT: &str = "You are QitOps Bot, an AI assistant for QitOps Agent.

QitOps Agent is an AI-powered tool that assists in improving software quality through:
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
";

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

/// QitOps Bot
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

    /// Start a chat session
    pub async fn start_chat_session(&mut self) -> Result<()> {
        // Display welcome message
        println!("\n{}", branding::logo("QitOps Bot"));
        println!("\nWelcome to QitOps Bot! Type 'exit' or 'quit' to end the session.");

        // Add welcome message to chat history
        let welcome_message = "Hello! I'm the QitOps Bot. How can I help you with QitOps Agent today?\n\nType !help to see available commands.";
        self.chat_history.push(ChatMessage::Bot(welcome_message.to_string()));
        println!("\nQitOps Bot: {}", welcome_message);

        // Start chat loop
        loop {
            // Get user input
            print!("You: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            // Check if the user wants to exit
            if input.to_lowercase() == "exit" || input.to_lowercase() == "quit" {
                println!("\nQitOps Bot: Goodbye! Feel free to chat again if you need help with QitOps Agent.");
                break;
            }

            // Add user message to chat history
            self.chat_history.push(ChatMessage::User(input.to_string()));

            // Process the message
            let response = self.process_message(input).await?;
            println!("QitOps Bot: {}", response);
        }

        Ok(())
    }

    /// Process a message
    pub async fn process_message(&mut self, message: &str) -> Result<String> {
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
                let response = "Chat history cleared.".to_string();
                self.chat_history.push(ChatMessage::System(response.clone()));
                return Ok(response);
            }

            // Save history command
            if message == "!save" {
                match self.save_chat_history() {
                    Ok(path) => {
                        let response = format!("Chat history saved to: {}", path);
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
        }

        // Check if the message is a natural language command
        if let Some(command) = self.parse_natural_language_command(message).await? {
            let result = self.execute_command(&command).await?;
            let response = format!("I interpreted your request as the command: `{}`\n\nResult:\n```\n{}\n```", command, result);

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

    /// Parse a natural language command
    pub async fn parse_natural_language_command(&self, message: &str) -> Result<Option<String>> {
        // Check if the message looks like a command request
        let command_indicators = [
            "run", "execute", "start", "generate", "analyze", "test", "create",
            "show", "list", "add", "remove", "set", "config", "help", "check",
            "assess", "evaluate", "find", "search", "get", "make", "build", "setup"
        ];

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

        // Check for command indicators
        let has_command_indicator = command_indicators.iter().any(|&indicator| {
            message.to_lowercase().contains(&format!(" {} ", indicator)) ||
            message.to_lowercase().starts_with(&format!("{} ", indicator)) ||
            message.to_lowercase().contains(&format!("{} ", indicator))
        });

        // Check for command-specific indicators
        let has_specific_indicator = command_specific.iter().any(|&indicator| {
            message.to_lowercase().contains(indicator)
        });

        // If no indicators are found, it's probably not a command request
        if !has_command_indicator && !has_specific_indicator {
            return Ok(None);
        }

        // Create a detailed prompt for the LLM to parse the natural language request
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
}
