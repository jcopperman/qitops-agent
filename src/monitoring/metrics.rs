// Metrics definitions for QitOps Agent
//
// This module defines all the metrics that are collected by QitOps Agent.

use lazy_static::lazy_static;
use prometheus::{
    register_counter, register_gauge, register_histogram, Counter, Gauge, Histogram,
};

// System metrics
lazy_static! {
    /// System memory total (bytes)
    pub static ref SYSTEM_MEMORY_TOTAL: Gauge = register_gauge!(
        "qitops_system_memory_total_bytes",
        "Total system memory in bytes"
    )
    .unwrap();

    /// System memory free (bytes)
    pub static ref SYSTEM_MEMORY_FREE: Gauge = register_gauge!(
        "qitops_system_memory_free_bytes",
        "Free system memory in bytes"
    )
    .unwrap();

    /// System memory available (bytes)
    pub static ref SYSTEM_MEMORY_AVAILABLE: Gauge = register_gauge!(
        "qitops_system_memory_available_bytes",
        "Available system memory in bytes"
    )
    .unwrap();

    /// System memory buffers (bytes)
    pub static ref SYSTEM_MEMORY_BUFFERS: Gauge = register_gauge!(
        "qitops_system_memory_buffers_bytes",
        "System memory buffers in bytes"
    )
    .unwrap();

    /// System memory cached (bytes)
    pub static ref SYSTEM_MEMORY_CACHED: Gauge = register_gauge!(
        "qitops_system_memory_cached_bytes",
        "System memory cached in bytes"
    )
    .unwrap();

    /// System CPU load (1 minute average)
    pub static ref SYSTEM_CPU_LOAD_1M: Gauge = register_gauge!(
        "qitops_system_cpu_load_1m",
        "System CPU load (1 minute average)"
    )
    .unwrap();

    /// System CPU load (5 minute average)
    pub static ref SYSTEM_CPU_LOAD_5M: Gauge = register_gauge!(
        "qitops_system_cpu_load_5m",
        "System CPU load (5 minute average)"
    )
    .unwrap();

    /// System CPU load (15 minute average)
    pub static ref SYSTEM_CPU_LOAD_15M: Gauge = register_gauge!(
        "qitops_system_cpu_load_15m",
        "System CPU load (15 minute average)"
    )
    .unwrap();
}

// Process metrics
lazy_static! {
    /// Process CPU usage (percentage)
    pub static ref PROCESS_CPU_USAGE: Gauge = register_gauge!(
        "qitops_process_cpu_usage_percent",
        "Process CPU usage percentage"
    )
    .unwrap();

    /// Process memory usage (bytes)
    pub static ref PROCESS_MEMORY_USAGE: Gauge = register_gauge!(
        "qitops_process_memory_usage_bytes",
        "Process memory usage in bytes"
    )
    .unwrap();
}

// Command metrics
lazy_static! {
    /// Total number of commands executed
    pub static ref COMMAND_COUNTER: Counter = register_counter!(
        "qitops_commands_total",
        "Total number of commands executed"
    )
    .unwrap();

    /// Duration of command execution in seconds
    pub static ref COMMAND_DURATION: Histogram = register_histogram!(
        "qitops_command_duration_seconds",
        "Duration of command execution in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]
    )
    .unwrap();

    /// Total number of test-gen commands executed
    pub static ref TEST_GEN_COUNTER: Counter = register_counter!(
        "qitops_test_gen_commands_total",
        "Total number of test-gen commands executed"
    )
    .unwrap();

    /// Duration of test-gen command execution in seconds
    pub static ref TEST_GEN_DURATION: Histogram = register_histogram!(
        "qitops_test_gen_duration_seconds",
        "Duration of test-gen command execution in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]
    )
    .unwrap();

    /// Total number of pr-analyze commands executed
    pub static ref PR_ANALYZE_COUNTER: Counter = register_counter!(
        "qitops_pr_analyze_commands_total",
        "Total number of pr-analyze commands executed"
    )
    .unwrap();

    /// Duration of pr-analyze command execution in seconds
    pub static ref PR_ANALYZE_DURATION: Histogram = register_histogram!(
        "qitops_pr_analyze_duration_seconds",
        "Duration of pr-analyze command execution in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]
    )
    .unwrap();

    /// Total number of risk commands executed
    pub static ref RISK_COUNTER: Counter = register_counter!(
        "qitops_risk_commands_total",
        "Total number of risk commands executed"
    )
    .unwrap();

    /// Duration of risk command execution in seconds
    pub static ref RISK_DURATION: Histogram = register_histogram!(
        "qitops_risk_duration_seconds",
        "Duration of risk command execution in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]
    )
    .unwrap();

    /// Total number of test-data commands executed
    pub static ref TEST_DATA_COUNTER: Counter = register_counter!(
        "qitops_test_data_commands_total",
        "Total number of test-data commands executed"
    )
    .unwrap();

    /// Duration of test-data command execution in seconds
    pub static ref TEST_DATA_DURATION: Histogram = register_histogram!(
        "qitops_test_data_duration_seconds",
        "Duration of test-data command execution in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]
    )
    .unwrap();

    /// Total number of session commands executed
    pub static ref SESSION_COUNTER: Counter = register_counter!(
        "qitops_session_commands_total",
        "Total number of session commands executed"
    )
    .unwrap();

    /// Duration of session command execution in seconds
    pub static ref SESSION_DURATION: Histogram = register_histogram!(
        "qitops_session_duration_seconds",
        "Duration of session command execution in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0, 3600.0]
    )
    .unwrap();
}

// LLM metrics
lazy_static! {
    /// Total number of LLM requests
    pub static ref LLM_REQUEST_COUNTER: Counter = register_counter!(
        "qitops_llm_requests_total",
        "Total number of LLM requests"
    )
    .unwrap();

    /// Duration of LLM requests in seconds
    pub static ref LLM_REQUEST_DURATION: Histogram = register_histogram!(
        "qitops_llm_request_duration_seconds",
        "Duration of LLM requests in seconds",
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]
    )
    .unwrap();

    /// Total number of tokens used in LLM requests
    pub static ref LLM_TOKEN_USAGE: Counter = register_counter!(
        "qitops_llm_tokens_total",
        "Total number of tokens used in LLM requests"
    )
    .unwrap();

    /// Total number of OpenAI requests
    pub static ref LLM_OPENAI_REQUEST_COUNTER: Counter = register_counter!(
        "qitops_llm_openai_requests_total",
        "Total number of OpenAI requests"
    )
    .unwrap();

    /// Total number of tokens used in OpenAI requests
    pub static ref LLM_OPENAI_TOKEN_USAGE: Counter = register_counter!(
        "qitops_llm_openai_tokens_total",
        "Total number of tokens used in OpenAI requests"
    )
    .unwrap();

    /// Total number of Ollama requests
    pub static ref LLM_OLLAMA_REQUEST_COUNTER: Counter = register_counter!(
        "qitops_llm_ollama_requests_total",
        "Total number of Ollama requests"
    )
    .unwrap();

    /// Total number of tokens used in Ollama requests
    pub static ref LLM_OLLAMA_TOKEN_USAGE: Counter = register_counter!(
        "qitops_llm_ollama_tokens_total",
        "Total number of tokens used in Ollama requests"
    )
    .unwrap();

    /// Total number of Anthropic requests
    pub static ref LLM_ANTHROPIC_REQUEST_COUNTER: Counter = register_counter!(
        "qitops_llm_anthropic_requests_total",
        "Total number of Anthropic requests"
    )
    .unwrap();

    /// Total number of tokens used in Anthropic requests
    pub static ref LLM_ANTHROPIC_TOKEN_USAGE: Counter = register_counter!(
        "qitops_llm_anthropic_tokens_total",
        "Total number of tokens used in Anthropic requests"
    )
    .unwrap();
}

// Error metrics
lazy_static! {
    /// Total number of errors
    pub static ref ERROR_COUNTER: Counter = register_counter!(
        "qitops_errors_total",
        "Total number of errors"
    )
    .unwrap();

    /// Total number of LLM errors
    pub static ref LLM_ERROR_COUNTER: Counter = register_counter!(
        "qitops_llm_errors_total",
        "Total number of LLM errors"
    )
    .unwrap();

    /// Total number of GitHub errors
    pub static ref GITHUB_ERROR_COUNTER: Counter = register_counter!(
        "qitops_github_errors_total",
        "Total number of GitHub errors"
    )
    .unwrap();

    /// Total number of agent errors
    pub static ref AGENT_ERROR_COUNTER: Counter = register_counter!(
        "qitops_agent_errors_total",
        "Total number of agent errors"
    )
    .unwrap();
}

// Cache metrics
lazy_static! {
    /// Total number of cache hits
    pub static ref CACHE_HIT_COUNTER: Counter = register_counter!(
        "qitops_cache_hits_total",
        "Total number of cache hits"
    )
    .unwrap();

    /// Total number of cache misses
    pub static ref CACHE_MISS_COUNTER: Counter = register_counter!(
        "qitops_cache_misses_total",
        "Total number of cache misses"
    )
    .unwrap();
}

// Session metrics
lazy_static! {
    /// Total number of session messages
    pub static ref SESSION_MESSAGE_COUNTER: Counter = register_counter!(
        "qitops_session_messages_total",
        "Total number of session messages"
    )
    .unwrap();

    /// Total number of user messages in sessions
    pub static ref SESSION_USER_MESSAGE_COUNTER: Counter = register_counter!(
        "qitops_session_user_messages_total",
        "Total number of user messages in sessions"
    )
    .unwrap();

    /// Total number of agent messages in sessions
    pub static ref SESSION_AGENT_MESSAGE_COUNTER: Counter = register_counter!(
        "qitops_session_agent_messages_total",
        "Total number of agent messages in sessions"
    )
    .unwrap();
}
