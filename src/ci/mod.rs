// CI/CD integration
pub mod github;
pub mod config;

// Re-export commonly used types
pub use github::GitHubClient;
pub use config::GitHubConfigManager;
