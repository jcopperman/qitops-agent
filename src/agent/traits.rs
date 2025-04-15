use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Agent trait for defining common behavior across all QitOps agents
pub trait Agent {
    /// Initialize the agent with configuration
    #[allow(dead_code)]
    fn init(&mut self) -> Result<()>;

    /// Execute the agent's primary function
    #[allow(async_fn_in_trait)]
    async fn execute(&self) -> Result<AgentResponse>;

    /// Get the agent's name
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Get the agent's description
    #[allow(dead_code)]
    fn description(&self) -> &str;
}

/// Response from an agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// Status of the agent execution
    pub status: AgentStatus,

    /// Message from the agent
    pub message: String,

    /// Data returned by the agent
    pub data: Option<serde_json::Value>,
}

/// Status of an agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent execution succeeded
    Success,

    /// Agent execution failed
    Failure,

    /// Agent execution is in progress
    InProgress,

    /// Agent execution encountered an error
    Error,

    /// Agent execution produced a warning
    Warning,
}
