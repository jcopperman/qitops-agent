use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;
use std::fs;

pub mod knowledge;
use knowledge::KnowledgeBase;

use crate::llm::{LlmRouter, LlmRequest};
use crate::cli::branding;

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessage {
    /// User message
    User(String),

    /// Bot message
    Bot(String),
}

/// Bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// System prompt
    pub system_prompt: String,

    /// Knowledge base path
    pub knowledge_base_path: Option<PathBuf>,

    /// Max history length
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

        Self {
            llm_router,
            chat_history: Vec::new(),
            config,
            knowledge_base,
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

        // Trim chat history if it's too long
        if self.chat_history.len() > self.config.max_history_length * 2 {
            let new_start = self.chat_history.len() - self.config.max_history_length * 2;
            self.chat_history = self.chat_history[new_start..].to_vec();
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
                    kb_info.push_str("\n");
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
}
