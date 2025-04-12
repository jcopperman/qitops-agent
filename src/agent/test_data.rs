use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::agent::traits::{Agent, AgentResponse, AgentStatus};
use crate::llm::{LlmRequest, LlmRouter};

/// Test data generator agent
pub struct TestDataAgent {
    /// Schema definition
    schema: String,

    /// Number of records to generate
    count: usize,

    /// Constraints for the generated data
    constraints: Vec<String>,

    /// Output format (json, csv, yaml)
    format: String,

    /// LLM router
    llm_router: LlmRouter,
}

impl TestDataAgent {
    /// Create a new test data generator agent
    pub async fn new(
        schema: String,
        count: usize,
        constraints: Vec<String>,
        format: String,
        llm_router: LlmRouter,
    ) -> Result<Self> {
        Ok(Self {
            schema,
            count,
            constraints,
            format,
            llm_router,
        })
    }

    /// Generate the prompt for the LLM
    fn generate_prompt(&self) -> String {
        let constraints_str = if self.constraints.is_empty() {
            "".to_string()
        } else {
            format!("\n\nApply the following constraints: {}", self.constraints.join(", "))
        };

        format!(
            "Generate {} test data records for the following schema: {}{}\n\nProvide the data in {} format.",
            self.count, self.schema, constraints_str, self.format
        )
    }

    /// Get the system prompt
    fn system_prompt(&self) -> String {
        format!(
            "You are a test data generator. Generate realistic and diverse test data based on the provided schema. Ensure the data is valid and follows the specified constraints. Provide the data in {} format.",
            self.format
        )
    }

    /// Save the generated test data to a file
    fn save_test_data(&self, test_data: &str) -> Result<String> {
        // Create the output directory if it doesn't exist
        let output_dir = Path::new("test_data");
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }

        // Create a sanitized schema name for the file
        let schema_name = self.schema.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|', ' '], "_");

        // Create the output file
        let output_file = output_dir.join(format!("{}_data.{}", schema_name, self.format.to_lowercase()));
        fs::write(&output_file, test_data)?;

        Ok(output_file.to_string_lossy().to_string())
    }
}

impl Agent for TestDataAgent {
    fn init(&mut self) -> Result<()> {
        // No initialization needed
        Ok(())
    }

    async fn execute(&self) -> Result<AgentResponse> {
        // Generate the prompt
        let prompt = self.generate_prompt();

        // Create the LLM request
        let model = self.llm_router.default_model().unwrap_or_else(|| "tinyllama".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message(self.system_prompt());

        // Send the request to the LLM
        let response = self.llm_router.send(request, Some("test-data")).await?;

        // Save the test data to a file
        let output_file = self.save_test_data(&response.text)?;

        // Return the response
        Ok(AgentResponse {
            status: AgentStatus::Success,
            message: format!("Generated {} test data records for schema: {}", self.count, self.schema),
            data: Some(serde_json::json!({
                "output_file": output_file,
                "schema": self.schema,
                "count": self.count,
                "constraints": self.constraints,
            })),
        })
    }

    fn name(&self) -> &str {
        "test-data"
    }

    fn description(&self) -> &str {
        "Test data generator"
    }
}
