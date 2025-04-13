use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    /// Parse a string into a test format
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(TestFormat::Markdown),
            "yaml" | "yml" => Ok(TestFormat::Yaml),
            "robot" => Ok(TestFormat::Robot),
            _ => Err(anyhow::anyhow!("Unknown test format: {}", s)),
        }
    }

    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            TestFormat::Markdown => "md",
            TestFormat::Yaml => "yaml",
            TestFormat::Robot => "robot",
        }
    }

    /// Get the system prompt for this format
    pub fn system_prompt(&self) -> String {
        match self {
            TestFormat::Markdown => "Generate test cases in Markdown format. Use proper Markdown formatting with headers, lists, and code blocks.".to_string(),
            TestFormat::Yaml => "Generate test cases in YAML format. Follow proper YAML syntax and indentation.".to_string(),
            TestFormat::Robot => "Generate test cases in Robot Framework format. Follow proper Robot Framework syntax with settings, variables, and keywords.".to_string(),
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
        format: &str,
        sources: Option<Vec<String>>,
        personas: Option<Vec<String>>,
        llm_router: LlmRouter
    ) -> Result<Self> {
        let format = TestFormat::from_str(format)?;

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

        fs::read_to_string(path).context(format!("Failed to read file: {}", self.path))
    }

    /// Generate the prompt for the LLM
    async fn generate_prompt(&self, source_code: &str) -> Result<String> {
        let mut prompt = format!(
            "Generate comprehensive test cases for the following code. Focus on edge cases, error handling, and important functionality.\n\nCode:\n```\n{}\n```",
            source_code
        );

        // Add sources if available
        if let Some(sources) = &self.sources {
            if !sources.is_empty() {
                let source_manager = crate::cli::source::SourceManager::new()?;
                let source_content = source_manager.get_content_for_sources(sources)?;

                if !source_content.is_empty() {
                    prompt.push_str("\n\nAdditional context from sources:\n");
                    prompt.push_str(&source_content);
                }
            }
        }

        // Add personas if available
        if let Some(personas) = &self.personas {
            if !personas.is_empty() {
                let persona_manager = crate::cli::persona::PersonaManager::new()?;
                let persona_prompt = persona_manager.get_prompt_for_personas(personas)?;

                if !persona_prompt.is_empty() {
                    prompt = format!("{}\n\n{}", persona_prompt, prompt);
                }
            }
        }

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
        // Read the source code
        let source_code = self.read_source_code()?;

        // Generate the prompt
        let prompt = self.generate_prompt(&source_code).await?;

        // Create the LLM request
        let model = self.llm_router.default_model().unwrap_or_else(|| "tinyllama".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message(self.format.system_prompt());

        // Send the request to the LLM
        let response = self.llm_router.send(request, Some("test-gen")).await?;

        // Save the test cases to a file
        let output_file = self.save_test_cases(&response.text)?;

        // Return the response
        Ok(AgentResponse {
            status: AgentStatus::Success,
            message: format!("Generated test cases saved to {}", output_file),
            data: Some(serde_json::json!({
                "output_file": output_file,
                "test_cases": response.text,
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
