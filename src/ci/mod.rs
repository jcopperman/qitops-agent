// CI/CD integration
pub mod github;
pub mod config;

// Re-export commonly used types
pub use github::{GitHubClient, PullRequest, PullRequestFile, PullRequestComment, Repository, Commit};
pub use config::{GitHubConfig, GitHubConfigManager};
