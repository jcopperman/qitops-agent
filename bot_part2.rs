
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

    // Create QitOps Bot
    let mut bot = QitOpsBot::new(llm_router, Some(config)).await;

    // Start chat session
    bot.start_chat_session().await?;

    Ok(())
}
