use qitops_agent::agent::{TestGenAgent, Agent, AgentStatus};
use qitops_agent::llm::{LlmRouter, RouterConfig};
use std::fs;
use std::path::Path;
use tokio_test::block_on;

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    // Mock the LlmRouter
    mock! {
        LlmRouter {}
        impl LlmRouter {
            pub fn default_model(&self) -> Option<String>;
            pub async fn send(&self, request: qitops_agent::llm::LlmRequest, task: Option<&str>) -> anyhow::Result<qitops_agent::llm::LlmResponse>;
            pub async fn available_providers(&self) -> Vec<String>;
            pub fn default_provider(&self) -> &str;
            pub fn default_model_for_provider(&self, provider: &str) -> Option<String>;
        }
    }

    #[test]
    fn test_test_gen_agent_creation() {
        // Create a mock LlmRouter
        let mut mock_router = MockLlmRouter::new();
        mock_router.expect_default_model()
            .returning(|| Some("test-model".to_string()));
        
        // Create a test file
        let test_dir = Path::new("test_files");
        if !test_dir.exists() {
            fs::create_dir_all(test_dir).unwrap();
        }
        let test_file = test_dir.join("test_sample.rs");
        fs::write(&test_file, "fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
        
        // Create the agent
        let agent = block_on(TestGenAgent::new(
            test_file.to_string_lossy().to_string(),
            "markdown",
            None,
            None,
            mock_router
        ));
        
        assert!(agent.is_ok());
        
        // Clean up
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_test_gen_agent_with_invalid_path() {
        // Create a mock LlmRouter
        let mut mock_router = MockLlmRouter::new();
        mock_router.expect_default_model()
            .returning(|| Some("test-model".to_string()));
        
        // Create the agent with an invalid path
        let agent = block_on(TestGenAgent::new(
            "non_existent_file.rs".to_string(),
            "markdown",
            None,
            None,
            mock_router
        ));
        
        assert!(agent.is_ok()); // Agent creation should succeed
        
        // But execution should fail
        let agent = agent.unwrap();
        let result = block_on(agent.execute());
        assert!(result.is_err());
    }
}
