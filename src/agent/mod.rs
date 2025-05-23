// Agent trait system
pub mod traits;
pub mod test_gen;
pub mod pr_analyze;
pub mod risk;
pub mod test_data;

// Re-export commonly used types
pub use traits::{Agent, AgentResponse, AgentStatus};
pub use test_gen::TestGenAgent;
pub use pr_analyze::PrAnalyzeAgent;
pub use risk::RiskAgent;
pub use test_data::TestDataAgent;
