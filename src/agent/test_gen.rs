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
    /// Create a new test case generator agent with input validation
    pub async fn new(
        path: String,
        format: &str,
        sources: Option<Vec<String>>,
        personas: Option<Vec<String>>,
        llm_router: LlmRouter
    ) -> Result<Self> {
        // Validate the format
        let format = TestFormat::from_str(format)
            .context(format!("Invalid test format: '{}'. Supported formats are: markdown, yaml, robot", format))?;

        // Validate the path exists
        let file_path = Path::new(&path);
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", path));
        }

        // Validate the path is readable
        match fs::metadata(file_path) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    return Err(anyhow::anyhow!("Path is a directory, not a file: {}", path));
                }
            },
            Err(e) => {
                return Err(anyhow::anyhow!("Cannot access file metadata for {}: {}", path, e));
            }
        }

        // Validate sources if provided
        if let Some(sources) = &sources {
            if sources.is_empty() {
                return Err(anyhow::anyhow!("Sources list is empty. Either provide valid sources or omit the parameter."));
            }
        }

        // Validate personas if provided
        if let Some(personas) = &personas {
            if personas.is_empty() {
                return Err(anyhow::anyhow!("Personas list is empty. Either provide valid personas or omit the parameter."));
            }
        }

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

        // Try to read the file with better error handling
        match fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(e) => {
                // Provide more specific error messages based on the error kind
                match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        Err(anyhow::anyhow!("Permission denied when reading file: {}. Try running with administrator privileges or check file permissions.", self.path))
                    },
                    std::io::ErrorKind::NotFound => {
                        Err(anyhow::anyhow!("File not found: {}", self.path))
                    },
                    _ => {
                        Err(anyhow::anyhow!("Failed to read file: {}. Error: {}", self.path, e))
                    }
                }
            }
        }
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
        let prompt = match self.generate_prompt(&source_code).await {
            Ok(prompt) => prompt,
            Err(e) => {
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
