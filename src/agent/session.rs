use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::agent::traits::{Agent, AgentResponse, AgentStatus};
use crate::llm::{LlmRequest, LlmRouter};

/// Session type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SessionType {
    /// Exploratory testing
    Exploratory,
    /// Regression testing
    Regression,
    /// User journey testing
    UserJourney,
    /// Performance testing
    Performance,
    /// Security testing
    Security,
}

impl std::str::FromStr for SessionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "exploratory" => Ok(SessionType::Exploratory),
            "regression" => Ok(SessionType::Regression),
            "user-journey" | "userjourney" | "user_journey" => Ok(SessionType::UserJourney),
            "performance" => Ok(SessionType::Performance),
            "security" => Ok(SessionType::Security),
            _ => Err(anyhow::anyhow!("Unknown session type: {}", s)),
        }
    }
}

impl SessionType {

    /// Get the system prompt for this session type
    pub fn system_prompt(&self) -> String {
        match self {
            SessionType::Exploratory => "You are an exploratory testing expert. Guide the user through an exploratory testing session, helping them discover and document issues in their application.".to_string(),
            SessionType::Regression => "You are a regression testing expert. Guide the user through a regression testing session, helping them verify that previously working functionality still works correctly.".to_string(),
            SessionType::UserJourney => "You are a user journey testing expert. Guide the user through testing complete user journeys, ensuring that end-to-end flows work correctly.".to_string(),
            SessionType::Performance => "You are a performance testing expert. Guide the user through performance testing, helping them identify and document performance issues.".to_string(),
            SessionType::Security => "You are a security testing expert. Guide the user through security testing, helping them identify and document security vulnerabilities.".to_string(),
        }
    }
}

/// Interactive testing session agent
#[derive(Debug)]
pub struct SessionAgent {
    /// Session name
    name: String,

    /// Session type
    session_type: SessionType,

    /// Application under test
    application: String,

    /// Test objectives
    objectives: Vec<String>,

    /// Sources to use
    sources: Option<Vec<String>>,

    /// Personas to use
    personas: Option<Vec<String>>,

    /// LLM router
    llm_router: LlmRouter,

    /// Session history
    history: Vec<SessionMessage>,
}

/// Session message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionMessage {
    /// User message
    User(String),
    /// Agent message
    Agent(String),
    /// System message
    System(String),
}

impl SessionAgent {
    /// Create a new interactive testing session agent
    pub async fn new(
        name: String,
        session_type: Option<String>,
        application: String,
        objectives: Vec<String>,
        sources: Option<Vec<String>>,
        personas: Option<Vec<String>>,
        llm_router: LlmRouter,
    ) -> Result<Self> {
        // Validate inputs
        if name.is_empty() {
            return Err(anyhow::anyhow!("Session name cannot be empty"));
        }

        if application.is_empty() {
            return Err(anyhow::anyhow!("application name cannot be empty"));
        }

        // Parse session type
        let session_type = match session_type {
            Some(t) => t.parse::<SessionType>()?,
            None => SessionType::Exploratory,
        };

        // Create the agent
        Ok(Self {
            name,
            session_type,
            application,
            objectives,
            sources,
            personas,
            llm_router,
            history: Vec::new(),
        })
    }

    /// Generate the prompt for the LLM
    fn generate_prompt(&self) -> String {
        let objectives_str = if self.objectives.is_empty() {
            "general testing".to_string()
        } else {
            format!("the following objectives: {}", self.objectives.join(", "))
        };

        // Add sources if available
        let sources_str = if let Some(sources) = &self.sources {
            if !sources.is_empty() {
                format!("\n\nConsider the following sources of information: {}.", sources.join(", "))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Add personas if available
        let personas_str = if let Some(personas) = &self.personas {
            if !personas.is_empty() {
                format!("\n\nConsider the following user personas: {}.", personas.join(", "))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        format!(
            "You are guiding a testing session for the application '{}' with {}. The session name is '{}'.\n\nProvide a structured testing plan with specific test scenarios, expected results, and areas to focus on.{}{}",
            self.application, objectives_str, self.name, sources_str, personas_str
        )
    }

    /// Add a message to the session history
    pub fn add_message(&mut self, message: SessionMessage) {
        self.history.push(message);
    }

    /// Process a user message
    pub async fn process_message(&mut self, message: &str) -> Result<String> {
        // Add user message to history
        self.add_message(SessionMessage::User(message.to_string()));

        // Create the prompt from the session history
        let mut prompt = String::new();
        for msg in &self.history {
            match msg {
                SessionMessage::User(text) => {
                    prompt.push_str(&format!("User: {}\n", text));
                },
                SessionMessage::Agent(text) => {
                    prompt.push_str(&format!("QitOps Agent: {}\n", text));
                },
                SessionMessage::System(text) => {
                    prompt.push_str(&format!("System: {}\n", text));
                },
            }
        }

        // Get a valid provider
        let (_provider, model) = match self.llm_router.get_valid_provider(None).await {
            Ok((provider, model)) => (provider, model),
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get a valid LLM provider: {}", e));
            }
        };

        // Create the LLM request
        let request = LlmRequest::new(prompt, model)
            .with_system_message(self.session_type.system_prompt());

        // Send the request to the LLM
        let response = match self.llm_router.send(request, Some("session")).await {
            Ok(response) => response,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get response from LLM: {}", e));
            }
        };

        // Add agent response to history
        let response_text = response.text;
        self.add_message(SessionMessage::Agent(response_text.clone()));

        Ok(response_text)
    }

    /// Save the session history to a file
    pub fn save_session_history(&self) -> Result<String> {
        // Create the output directory if it doesn't exist
        let output_dir = Path::new("sessions");
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }

        // Create a sanitized session name for the file
        let session_name = self.name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|', ' '], "_");

        // Create the output file
        let output_file = output_dir.join(format!("{}_session.md", session_name));

        // Format the session history as markdown
        let mut markdown = format!("# Testing Session: {}\n\n", self.name);
        markdown.push_str(&format!("## Application: {}\n\n", self.application));

        if !self.objectives.is_empty() {
            markdown.push_str("## Objectives\n\n");
            for objective in &self.objectives {
                markdown.push_str(&format!("- {}\n", objective));
            }
            markdown.push('\n');
        }

        markdown.push_str("## Session History\n\n");
        for msg in &self.history {
            match msg {
                SessionMessage::User(text) => {
                    markdown.push_str(&format!("**User**: {}\n\n", text));
                },
                SessionMessage::Agent(text) => {
                    markdown.push_str(&format!("**QitOps Agent**: {}\n\n", text));
                },
                SessionMessage::System(text) => {
                    markdown.push_str(&format!("**System**: {}\n\n", text));
                },
            }
        }

        // Write the markdown to the file
        fs::write(&output_file, markdown)?;

        Ok(output_file.to_string_lossy().to_string())
    }
}

impl Agent for SessionAgent {
    fn init(&mut self) -> Result<()> {
        // Add initial system message
        self.add_message(SessionMessage::System(format!(
            "Starting a new {} testing session for application '{}'.",
            format!("{:?}", self.session_type).to_lowercase(),
            self.application
        )));

        Ok(())
    }

    async fn execute(&self) -> Result<AgentResponse> {
        // Generate the prompt
        let prompt = self.generate_prompt();

        // Get a valid provider
        let (_provider, model) = match self.llm_router.get_valid_provider(None).await {
            Ok((provider, model)) => (provider, model),
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to get a valid LLM provider: {}", e),
                    data: Some(serde_json::json!({
                        "session_name": self.name,
                        "application": self.application,
                        "objectives": self.objectives,
                        "error": format!("{}", e),
                    })),
                });
            }
        };

        // Create the LLM request
        let request = LlmRequest::new(prompt, model)
            .with_system_message(self.session_type.system_prompt());

        // Send the request to the LLM
        let response = match self.llm_router.send(request, Some("session")).await {
            Ok(response) => response,
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to get response from LLM: {}", e),
                    data: Some(serde_json::json!({
                        "session_name": self.name,
                        "application": self.application,
                        "objectives": self.objectives,
                        "error": format!("{}", e),
                    })),
                });
            }
        };

        // Return the response
        Ok(AgentResponse {
            status: AgentStatus::Success,
            message: format!("Testing session plan generated for '{}'", self.name),
            data: Some(serde_json::json!({
                "session_name": self.name,
                "application": self.application,
                "objectives": self.objectives,
                "session_type": format!("{:?}", self.session_type),
                "plan": response.text,
                "model": response.model,
                "provider": response.provider,
            })),
        })
    }

    fn name(&self) -> &str {
        "session"
    }

    fn description(&self) -> &str {
        "Interactive testing session"
    }
}
