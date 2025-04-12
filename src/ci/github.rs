use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// GitHub API error
#[derive(Debug, Error)]
pub enum GitHubError {
    /// API error
    #[error("API error: {0}")]
    ApiError(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Rate limit error
    #[error("Rate limit error: {0}")]
    RateLimitError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// GitHub PR information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// PR number
    pub number: u64,

    /// PR title
    pub title: String,

    /// PR description
    pub body: Option<String>,

    /// PR author
    pub author: String,

    /// PR state (open, closed, merged)
    pub state: String,

    /// PR base branch
    pub base_branch: String,

    /// PR head branch
    pub head_branch: String,

    /// PR created at
    pub created_at: String,

    /// PR updated at
    pub updated_at: String,
}

/// GitHub PR file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestFile {
    /// File name
    pub filename: String,

    /// File status (added, modified, removed)
    pub status: String,

    /// Number of additions
    pub additions: u64,

    /// Number of deletions
    pub deletions: u64,

    /// Number of changes
    pub changes: u64,

    /// File content URL
    pub contents_url: String,

    /// File patch
    pub patch: Option<String>,
}

/// GitHub PR comment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestComment {
    /// Comment ID
    pub id: u64,

    /// Comment body
    pub body: String,

    /// Comment author
    pub user: String,

    /// Comment created at
    pub created_at: String,

    /// Comment updated at
    pub updated_at: String,

    /// Path to the file that was commented on
    pub path: Option<String>,

    /// Line number in the file that was commented on
    pub line: Option<u64>,
}

/// GitHub client
pub struct GitHubClient {
    /// API token
    token: String,

    /// API base URL
    base_url: String,

    /// HTTP client
    http_client: reqwest::Client,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new(token: String) -> Self {
        Self {
            token,
            base_url: "https://api.github.com".to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Get a pull request by number
    pub async fn get_pull_request(&self, owner: &str, repo: &str, number: u64) -> Result<PullRequest> {
        let url = format!("{}/repos/{}/{}/pulls/{}", self.base_url, owner, repo, number);

        let response = self.http_client.get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "QitOps-Agent")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to GitHub API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Could not read error response".to_string());

            return match status.as_u16() {
                401 => Err(anyhow!("Authentication error: {}", error_text)),
                403 => Err(anyhow!("Forbidden: {}", error_text)),
                404 => Err(anyhow!("Not found: {}", error_text)),
                422 => Err(anyhow!("Validation error: {}", error_text)),
                _ => Err(anyhow!("GitHub API error ({}): {}", status, error_text)),
            };
        }

        let pr_data: serde_json::Value = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;

        // Extract the relevant fields from the response
        let pr = PullRequest {
            number,
            title: pr_data["title"].as_str().unwrap_or_default().to_string(),
            body: pr_data["body"].as_str().map(|s| s.to_string()),
            author: pr_data["user"]["login"].as_str().unwrap_or_default().to_string(),
            state: pr_data["state"].as_str().unwrap_or_default().to_string(),
            base_branch: pr_data["base"]["ref"].as_str().unwrap_or_default().to_string(),
            head_branch: pr_data["head"]["ref"].as_str().unwrap_or_default().to_string(),
            created_at: pr_data["created_at"].as_str().unwrap_or_default().to_string(),
            updated_at: pr_data["updated_at"].as_str().unwrap_or_default().to_string(),
        };

        Ok(pr)
    }

    /// Get the diff for a pull request
    pub async fn get_pull_request_diff(&self, owner: &str, repo: &str, number: u64) -> Result<String> {
        let url = format!("{}/repos/{}/{}/pulls/{}", self.base_url, owner, repo, number);

        let response = self.http_client.get(&url)
            .header("Accept", "application/vnd.github.v3.diff")
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "QitOps-Agent")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to GitHub API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Could not read error response".to_string());

            return match status.as_u16() {
                401 => Err(anyhow!("Authentication error: {}", error_text)),
                403 => Err(anyhow!("Forbidden: {}", error_text)),
                404 => Err(anyhow!("Not found: {}", error_text)),
                422 => Err(anyhow!("Validation error: {}", error_text)),
                _ => Err(anyhow!("GitHub API error ({}): {}", status, error_text)),
            };
        }

        let diff = response.text()
            .await
            .map_err(|e| anyhow!("Failed to read GitHub API response: {}", e))?;

        Ok(diff)
    }

    /// Get pull request files
    pub async fn get_pull_request_files(&self, owner: &str, repo: &str, number: u64) -> Result<Vec<PullRequestFile>> {
        let url = format!("{}/repos/{}/{}/pulls/{}/files", self.base_url, owner, repo, number);

        let response = self.http_client.get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "QitOps-Agent")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to GitHub API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Could not read error response".to_string());

            return match status.as_u16() {
                401 => Err(anyhow!("Authentication error: {}", error_text)),
                403 => Err(anyhow!("Forbidden: {}", error_text)),
                404 => Err(anyhow!("Not found: {}", error_text)),
                422 => Err(anyhow!("Validation error: {}", error_text)),
                _ => Err(anyhow!("GitHub API error ({}): {}", status, error_text)),
            };
        }

        let files_data: Vec<serde_json::Value> = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;

        let mut files = Vec::new();
        for file_data in files_data {
            let file = PullRequestFile {
                filename: file_data["filename"].as_str().unwrap_or_default().to_string(),
                status: file_data["status"].as_str().unwrap_or_default().to_string(),
                additions: file_data["additions"].as_u64().unwrap_or_default(),
                deletions: file_data["deletions"].as_u64().unwrap_or_default(),
                changes: file_data["changes"].as_u64().unwrap_or_default(),
                contents_url: file_data["contents_url"].as_str().unwrap_or_default().to_string(),
                patch: file_data["patch"].as_str().map(|s| s.to_string()),
            };
            files.push(file);
        }

        Ok(files)
    }

    /// Get pull request comments
    pub async fn get_pull_request_comments(&self, owner: &str, repo: &str, number: u64) -> Result<Vec<PullRequestComment>> {
        let url = format!("{}/repos/{}/{}/pulls/{}/comments", self.base_url, owner, repo, number);

        let response = self.http_client.get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "QitOps-Agent")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to GitHub API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Could not read error response".to_string());

            return match status.as_u16() {
                401 => Err(anyhow!("Authentication error: {}", error_text)),
                403 => Err(anyhow!("Forbidden: {}", error_text)),
                404 => Err(anyhow!("Not found: {}", error_text)),
                422 => Err(anyhow!("Validation error: {}", error_text)),
                _ => Err(anyhow!("GitHub API error ({}): {}", status, error_text)),
            };
        }

        let comments_data: Vec<serde_json::Value> = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;

        let mut comments = Vec::new();
        for comment_data in comments_data {
            let comment = PullRequestComment {
                id: comment_data["id"].as_u64().unwrap_or_default(),
                body: comment_data["body"].as_str().unwrap_or_default().to_string(),
                user: comment_data["user"]["login"].as_str().unwrap_or_default().to_string(),
                created_at: comment_data["created_at"].as_str().unwrap_or_default().to_string(),
                updated_at: comment_data["updated_at"].as_str().unwrap_or_default().to_string(),
                path: comment_data["path"].as_str().map(|s| s.to_string()),
                line: comment_data["line"].as_u64(),
            };
            comments.push(comment);
        }

        Ok(comments)
    }
}
