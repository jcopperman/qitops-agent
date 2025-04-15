use qitops_agent::agent::traits::{AgentResponse, AgentStatus};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_response_creation() {
        // Test basic response creation
        let response = AgentResponse {
            status: AgentStatus::Success,
            message: "Test message".to_string(),
            data: Some(json!({"key": "value"})),
        };
        
        assert_eq!(response.status, AgentStatus::Success);
        assert_eq!(response.message, "Test message");
        assert!(response.data.is_some());
        
        if let Some(data) = response.data {
            assert_eq!(data["key"], "value");
        }
    }

    #[test]
    fn test_agent_response_without_data() {
        // Test response without data
        let response = AgentResponse {
            status: AgentStatus::Error,
            message: "Error message".to_string(),
            data: None,
        };
        
        assert_eq!(response.status, AgentStatus::Error);
        assert_eq!(response.message, "Error message");
        assert!(response.data.is_none());
    }

    #[test]
    fn test_agent_status_display() {
        // Test status display
        assert_eq!(format!("{}", AgentStatus::Success), "Success");
        assert_eq!(format!("{}", AgentStatus::Error), "Error");
        assert_eq!(format!("{}", AgentStatus::Warning), "Warning");
        assert_eq!(format!("{}", AgentStatus::InProgress), "In Progress");
    }
}
