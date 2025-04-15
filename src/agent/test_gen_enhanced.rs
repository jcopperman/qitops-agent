use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

use crate::agent::{Agent, AgentResponse, AgentStatus};
use crate::llm::{LlmRouter, LlmRequest};

/// Test case format
#[derive(Debug, Clone, Copy)]
pub enum TestFormat {
    /// Markdown format
    Markdown,
    /// Rust format
    Rust,
    /// Python format
    Python,
    /// JavaScript format
    JavaScript,
}

impl TestFormat {
    /// Get the file extension for the test format
    pub fn extension(&self) -> &str {
        match self {
            TestFormat::Markdown => "md",
            TestFormat::Rust => "rs",
            TestFormat::Python => "py",
            TestFormat::JavaScript => "js",
        }
    }

    /// Get the system prompt for the test format
    pub fn system_prompt(&self) -> String {
        match self {
            TestFormat::Markdown => "Generate test cases in Markdown format. Use proper Markdown formatting with headers, lists, and code blocks.".to_string(),
            TestFormat::Rust => "Generate Rust test cases using the #[test] attribute and the assert! macro family. Follow Rust testing best practices.".to_string(),
            TestFormat::Python => "Generate Python test cases using pytest or unittest. Follow Python testing best practices.".to_string(),
            TestFormat::JavaScript => "Generate JavaScript test cases using Jest or Mocha. Follow JavaScript testing best practices.".to_string(),
        }
    }
}

/// Test generation agent
pub struct TestGenAgent {
    /// Path to the source code
    path: String,

    /// Test format
    format: TestFormat,

    /// LLM router
    llm_router: LlmRouter,

    /// Sources to use for additional context
    sources: Option<Vec<String>>,

    /// Personas to use for the prompt
    personas: Option<Vec<String>>,
}

impl TestGenAgent {
    /// Create a new test generation agent
    pub fn new(
        path: String,
        format: TestFormat,
        llm_router: LlmRouter,
        sources: Option<Vec<String>>,
        personas: Option<Vec<String>>,
    ) -> Self {
        Self {
            path,
            format,
            llm_router,
            sources,
            personas,
        }
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

    /// Generate the prompt for the LLM with enhanced context
    async fn generate_prompt(&self, source_code: &str) -> Result<String> {
        // Try to initialize repository context
        let current_dir = std::env::current_dir()?;
        let mut repo_context_opt: Option<crate::context::RepositoryContext> = match crate::context::RepositoryContext::new(&current_dir) {
            Ok(context) => Some(context),
            Err(e) => {
                debug!("Failed to initialize repository context: {}", e);
                None
            }
        };

        // Start building the prompt
        let mut prompt_parts = Vec::new();

        // Add repository context if available
        if let Some(ref context) = repo_context_opt {
            let repo_context = context.generate_context(800);
            if !repo_context.is_empty() {
                prompt_parts.push(format!("Repository Context:\n{}\n", repo_context));
            }
        }

        // Add file context if available
        if let Some(ref mut context) = repo_context_opt {
            if let Ok(file_context) = context.generate_file_context(&self.path, true, true) {
                if !file_context.is_empty() {
                    prompt_parts.push(format!("File Context:\n{}\n", file_context));
                }
            }

            // Find imports and add their content for context
            if let Ok(imports) = context.find_imports(&self.path) {
                let mut import_contents = Vec::new();
                for import in imports.iter().take(3) { // Limit to 3 imports to avoid too much context
                    // Try to find the file that contains this import
                    let import_path = import.replace("::", "/") + ".rs";
                    if let Ok(content) = context.get_file_content(&import_path) {
                        import_contents.push(format!("Import: {}\n```\n{}\n```", import, content));
                    }
                }

                if !import_contents.is_empty() {
                    prompt_parts.push(format!("Related Code:\n{}\n", import_contents.join("\n\n")));
                }
            }
        }

        // Add the main code prompt
        prompt_parts.push(format!(
            "Generate comprehensive test cases for the following code. Focus on edge cases, error handling, and important functionality.\n\nCode:\n```\n{}\n```",
            source_code
        ));

        // Add sources if available
        if let Some(sources) = &self.sources {
            if !sources.is_empty() {
                let source_manager = crate::cli::source::SourceManager::new()?;
                let source_content = source_manager.get_content_for_sources(sources)?;

                if !source_content.is_empty() {
                    prompt_parts.push(format!("Additional context from sources:\n{}", source_content));
                }
            }
        }

        // Add personas if available
        if let Some(personas) = &self.personas {
            if !personas.is_empty() {
                let persona_manager = crate::cli::persona::PersonaManager::new()?;
                let persona_prompt = persona_manager.get_prompt_for_personas(personas)?;

                if !persona_prompt.is_empty() {
                    // Add persona prompt at the beginning
                    prompt_parts.insert(0, persona_prompt);
                }
            }
        }

        // Add specific instructions for test generation
        prompt_parts.push("Please generate comprehensive test cases that:\n\
1. Cover all functions and methods in the code\n\
2. Include tests for edge cases and error conditions\n\
3. Test both happy paths and failure scenarios\n\
4. Follow best practices for the language and testing framework\n\
5. Are well-organized and clearly documented".to_string());

        // Combine all parts into the final prompt
        let prompt = prompt_parts.join("\n\n");

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
