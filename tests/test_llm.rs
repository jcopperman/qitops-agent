use qitops_agent::llm::{LlmRequest, LlmResponse, MessageRole};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_request_creation() {
        // Test basic request creation
        let request = LlmRequest::new("Test content".to_string(), "test-model".to_string());
        
        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, MessageRole::User);
        assert_eq!(request.messages[0].content, "Test content");
        assert_eq!(request.max_tokens, 1024);
        assert_eq!(request.temperature, 0.7);
    }

    #[test]
    fn test_llm_request_with_system_message() {
        // Test request with system message
        let request = LlmRequest::new("Test content".to_string(), "test-model".to_string())
            .with_system_message("System instruction".to_string());
        
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(request.messages[0].content, "System instruction");
        assert_eq!(request.messages[1].role, MessageRole::User);
        assert_eq!(request.messages[1].content, "Test content");
    }

    #[test]
    fn test_llm_response_creation() {
        // Test response creation
        let response = LlmResponse::new(
            "Generated text".to_string(),
            "test-model".to_string(),
            "test-provider".to_string()
        );
        
        assert_eq!(response.text, "Generated text");
        assert_eq!(response.model, "test-model");
        assert_eq!(response.provider, "test-provider");
        assert_eq!(response.cached, false);
        assert!(response.tokens_used.is_none());
    }

    #[test]
    fn test_llm_response_with_tokens() {
        // Test response with tokens
        let response = LlmResponse::new(
            "Generated text".to_string(),
            "test-model".to_string(),
            "test-provider".to_string()
        ).with_tokens(100);
        
        assert_eq!(response.text, "Generated text");
        assert_eq!(response.tokens_used, Some(100));
    }

    #[test]
    fn test_llm_response_with_latency() {
        // Test response with latency
        let response = LlmResponse::new(
            "Generated text".to_string(),
            "test-model".to_string(),
            "test-provider".to_string()
        ).with_latency(150);
        
        assert_eq!(response.latency_ms, Some(150));
    }

    #[test]
    fn test_llm_response_with_cached() {
        // Test cached response
        let response = LlmResponse::new(
            "Generated text".to_string(),
            "test-model".to_string(),
            "test-provider".to_string()
        ).with_cached(true);
        
        assert_eq!(response.cached, true);
    }
}
