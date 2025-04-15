use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{info, debug, warn};
use crate::context::ContextProvider;
use crate::monitoring;

use crate::agent::traits::{Agent, AgentResponse, AgentStatus};
use crate::llm::{LlmRequest, LlmRouter};

/// Test case format
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TestFormat {
    /// Markdown format
    Markdown,
    /// YAML format
    Yaml,
    /// Robot Framework format
    Robot,
}

impl TestFormat {
    /// Parse test format from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(TestFormat::Markdown),
            "yaml" | "yml" => Ok(TestFormat::Yaml),
            "robot" => Ok(TestFormat::Robot),
            _ => Err(anyhow::anyhow!("Unknown test format: {}", s)),
        }
    }

    /// Get system prompt for test format
    pub fn system_prompt(&self) -> String {
        match self {
            TestFormat::Markdown => {
                "You are a test case generator. Generate comprehensive test cases for the given code. Focus on edge cases, error handling, and important functionality. Format the test cases in Markdown with clear sections for each test case, including description, inputs, expected outputs, and edge cases.".to_string()
            }
            TestFormat::Yaml => {
                "You are a test case generator. Generate comprehensive test cases for the given code. Focus on edge cases, error handling, and important functionality. Format the test cases in YAML with clear structure for each test case, including description, inputs, expected outputs, and edge cases.".to_string()
            }
            TestFormat::Robot => {
                "You are a test case generator. Generate comprehensive test cases for the given code. Focus on edge cases, error handling, and important functionality. Format the test cases in Robot Framework format with clear test cases, including documentation, setup, teardown, and test steps.".to_string()
            }
        }
    }

    /// Get file extension for test format
    pub fn extension(&self) -> String {
        match self {
            TestFormat::Markdown => "md".to_string(),
            TestFormat::Yaml => "yaml".to_string(),
            TestFormat::Robot => "robot".to_string(),
        }
    }
}

/// Test case generator agent
pub struct TestGenAgent {
    /// Path to the source code
    path: String,

    /// Output format
    format: TestFormat,

    /// Sources to use
    sources: Option<Vec<String>>,

    /// Personas to use
    personas: Option<Vec<String>>,

    /// LLM router
    llm_router: LlmRouter,
}

impl TestGenAgent {
    /// Create a new test case generator agent
    pub async fn new(
        path: String,
        format: String,
        sources: Option<Vec<String>>,
        personas: Option<Vec<String>>,
        llm_router: LlmRouter,
    ) -> Result<Self> {
        let format = TestFormat::from_str(&format)?;

        Ok(Self {
            path,
            format,
            sources,
            personas,
            llm_router,
        })
    }

    /// Read the source code
    fn read_source_code(&self) -> Result<String> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", self.path));
        }

        fs::read_to_string(path).context("Failed to read source code")
    }

    /// Generate the prompt for the LLM
    async fn generate_prompt(&self, source_code: &str) -> Result<String> {
        // Start a timer for monitoring
        let timer = monitoring::Timer::new("test_gen_prompt");

        let mut prompt = format!(
            "Generate comprehensive test cases for the following code. Focus on edge cases, error handling, and important functionality.\n\nCode:\n```\n{}\n```",
            source_code
        );

        // Add context from sources and personas
        let context_provider = ContextProvider::new()?;

        // Get sources (either from command line or defaults)
        let sources_vec: Vec<String>;
        let sources = if let Some(sources) = &self.sources {
            if !sources.is_empty() {
                Some(sources.as_slice())
            } else {
                // Try to get default sources
                sources_vec = context_provider.get_default_sources("test-gen")?;
                if !sources_vec.is_empty() {
                    Some(sources_vec.as_slice())
                } else {
                    None
                }
            }
        } else {
            // Try to get default sources
            sources_vec = context_provider.get_default_sources("test-gen")?;
            if !sources_vec.is_empty() {
                Some(sources_vec.as_slice())
            } else {
                None
            }
        };

        // Get personas (either from command line or defaults)
        let personas_vec: Vec<String>;
        let personas = if let Some(personas) = &self.personas {
            if !personas.is_empty() {
                Some(personas.as_slice())
            } else {
                // Try to get default personas
                personas_vec = context_provider.get_default_personas("test-gen")?;
                if !personas_vec.is_empty() {
                    Some(personas_vec.as_slice())
                } else {
                    None
                }
            }
        } else {
            // Try to get default personas
            personas_vec = context_provider.get_default_personas("test-gen")?;
            if !personas_vec.is_empty() {
                Some(personas_vec.as_slice())
            } else {
                None
            }
        };

        // Get context from sources and personas
        let context = context_provider.get_context(sources, personas)?;
        if !context.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&context);
        }

        // Stop the timer
        timer.stop();

        Ok(prompt)
    }

    /// Save the generated test cases to a file
    fn save_test_cases(&self, test_cases: &str) -> Result<String> {
        let path = Path::new(&self.path);
        let file_name = path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
            .to_string_lossy();

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let test_dir = parent.join("tests");

        // Create the test directory if it doesn't exist
        if !test_dir.exists() {
            fs::create_dir_all(&test_dir)?;
        }

        // Create the test file
        let test_file = test_dir.join(format!("test_{}.{}", file_name, self.format.extension()));
        fs::write(&test_file, test_cases)?;

        Ok(test_file.to_string_lossy().to_string())
    }
}

impl Agent for TestGenAgent {
    fn init(&mut self) -> Result<()> {
        // No initialization needed
        Ok(())
    }

    async fn execute(&self) -> Result<AgentResponse> {
        // Start a timer for monitoring
        let timer = monitoring::Timer::new("test_gen");
        monitoring::track_command("test-gen");

        // Read the source code
        let source_code = match self.read_source_code() {
            Ok(code) => code,
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to read source code: {}", e),
                    data: None,
                });
            }
        };

        // Generate the prompt
        info!("Generating enhanced prompt for test generation");
        let prompt = match self.generate_prompt(&source_code).await {
            Ok(prompt) => {
                info!("Successfully generated enhanced prompt with length: {}", prompt.len());
                debug!("Enhanced prompt: {}", prompt);
                prompt
            },
            Err(e) => {
                warn!("Failed to generate prompt: {}", e);
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to generate prompt: {}", e),
                    data: None,
                });
            }
        };

        // Create the LLM request
        let model = self.llm_router.default_model().unwrap_or_else(|| "mistral".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message(self.format.system_prompt());

        // Send the request to the LLM
        let response = match self.llm_router.send(request, Some("test-gen")).await {
            Ok(response) => response,
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to get response from LLM: {}", e),
                    data: None,
                });
            }
        };

        // Save the test cases to a file
        let output_file = match self.save_test_cases(&response.text) {
            Ok(file) => file,
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to save test cases: {}", e),
                    data: Some(serde_json::json!({
                        "test_cases": response.text,
                    })),
                });
            }
        };

        // Stop the timer
        timer.stop();

        // Return the response
        Ok(AgentResponse {
            status: AgentStatus::Success,
            message: format!("Generated test cases saved to {}", output_file),
            data: Some(serde_json::json!({
                "output_file": output_file,
                "test_cases": response.text,
                "model": response.model,
                "provider": response.provider,
                "format": format!("{:?}", self.format),
            })),
        })
    }

    fn name(&self) -> &str {
        "test-gen"
    }

    fn description(&self) -> &str {
        "Test case generator"
    }
}
