use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use regex::Regex;
use base64::Engine;
use crate::ci::config::GitHubConfig;

/// GitHub API error
#[derive(Debug, Error)]
#[allow(dead_code)]
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

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository ID
    pub id: u64,

    /// Repository name
    pub name: String,

    /// Repository owner
    pub owner: String,

    /// Repository description
    pub description: Option<String>,

    /// Repository URL
    pub url: String,

    /// Repository default branch
    pub default_branch: String,

    /// Repository is private
    pub private: bool,

    /// Repository language
    pub language: Option<String>,

    /// Repository created at
    pub created_at: String,

    /// Repository updated at
    pub updated_at: String,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// Commit SHA
    pub sha: String,

    /// Commit message
    pub message: String,

    /// Commit author
    pub author: String,

    /// Commit author email
    pub author_email: Option<String>,

    /// Commit date
    pub date: String,
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
    #[allow(dead_code)]
    pub fn new(token: String) -> Self {
        Self {
            token,
            base_url: "https://api.github.com".to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Create a new GitHub client from config
    pub fn from_config(config: &GitHubConfig) -> Result<Self> {
        // Try to get token from config, then environment variable
        let token = match (config.token.clone(), std::env::var("GITHUB_TOKEN").ok()) {
            (Some(token), _) if !token.trim().is_empty() => token,
            (_, Some(token)) if !token.trim().is_empty() => token,
            _ => {
                return Err(anyhow!(
                    "GitHub token not found in config or GITHUB_TOKEN environment variable. \n\n\
                    To configure GitHub token, run: \n\
                    qitops github config --token <YOUR_GITHUB_TOKEN> \n\n\
                    Or set the GITHUB_TOKEN environment variable."
                ));
            }
        };

        let base_url = config.api_base.clone().unwrap_or_else(|| "https://api.github.com".to_string());

        Ok(Self {
            token,
            base_url,
            http_client: reqwest::Client::new(),
        })
    }

    /// Extract repository owner and name from a GitHub URL
    pub fn extract_repo_info(url: &str) -> Result<(String, String)> {
        // Match patterns like:
        // - https://github.com/owner/repo
        // - https://github.com/owner/repo.git
        // - https://github.com/owner/repo/pull/123
        // - git@github.com:owner/repo.git
        let patterns = [
            Regex::new(r"github\.com[/:]([^/]+)/([^/\.]+)(?:\.git)?(?:/.*)?$").unwrap(),
            Regex::new(r"github\.com[/:]([^/]+)/([^/\.]+)(?:\.git)?$").unwrap(),
        ];

        for pattern in &patterns {
            if let Some(captures) = pattern.captures(url) {
                if captures.len() >= 3 {
                    let owner = captures[1].to_string();
                    let repo = captures[2].to_string();
                    return Ok((owner, repo));
                }
            }
        }

        Err(anyhow!("Could not extract repository information from URL: {}", url))
    }

    /// Extract PR number from a GitHub PR URL or string
    pub fn extract_pr_number(pr_string: &str) -> Result<u64> {
        // Try to parse as a number first
        if let Ok(number) = pr_string.parse::<u64>() {
            return Ok(number);
        }

        // Try to extract from URL
        let pattern = Regex::new(r"github\.com/[^/]+/[^/]+/pull/(\d+)(?:/.*)?$").unwrap();
        if let Some(captures) = pattern.captures(pr_string) {
            if captures.len() >= 2 {
                let number = captures[1].parse::<u64>()
                    .map_err(|_| anyhow!("Failed to parse PR number from URL: {}", pr_string))?;
                return Ok(number);
            }
        }

        Err(anyhow!("Could not extract PR number from: {}", pr_string))
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
    #[allow(dead_code)]
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

    /// Get repository information
    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<Repository> {
        let url = format!("{}/repos/{}/{}", self.base_url, owner, repo);

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

        let repo_data: serde_json::Value = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;

        let repository = Repository {
            id: repo_data["id"].as_u64().unwrap_or_default(),
            name: repo_data["name"].as_str().unwrap_or_default().to_string(),
            owner: repo_data["owner"]["login"].as_str().unwrap_or_default().to_string(),
            description: repo_data["description"].as_str().map(|s| s.to_string()),
            url: repo_data["html_url"].as_str().unwrap_or_default().to_string(),
            default_branch: repo_data["default_branch"].as_str().unwrap_or_default().to_string(),
            private: repo_data["private"].as_bool().unwrap_or_default(),
            language: repo_data["language"].as_str().map(|s| s.to_string()),
            created_at: repo_data["created_at"].as_str().unwrap_or_default().to_string(),
            updated_at: repo_data["updated_at"].as_str().unwrap_or_default().to_string(),
        };

        Ok(repository)
    }

    /// Get recent commits for a repository
    pub async fn get_commits(&self, owner: &str, repo: &str, limit: Option<usize>) -> Result<Vec<Commit>> {
        let limit = limit.unwrap_or(10);
        let url = format!("{}/repos/{}/{}/commits?per_page={}", self.base_url, owner, repo, limit);

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

        let commits_data: Vec<serde_json::Value> = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;

        let mut commits = Vec::new();
        for commit_data in commits_data {
            let commit = Commit {
                sha: commit_data["sha"].as_str().unwrap_or_default().to_string(),
                message: commit_data["commit"]["message"].as_str().unwrap_or_default().to_string(),
                author: commit_data["commit"]["author"]["name"].as_str().unwrap_or_default().to_string(),
                author_email: commit_data["commit"]["author"]["email"].as_str().map(|s| s.to_string()),
                date: commit_data["commit"]["author"]["date"].as_str().unwrap_or_default().to_string(),
            };
            commits.push(commit);
        }

        Ok(commits)
    }

    /// Get file content from a repository
    #[allow(dead_code)]
    pub async fn get_file_content(&self, owner: &str, repo: &str, path: &str, branch: Option<&str>) -> Result<String> {
        let branch_param = branch.map(|b| format!("?ref={}", b)).unwrap_or_default();
        let url = format!("{}/repos/{}/{}/contents/{}{}",
            self.base_url, owner, repo, path, branch_param);

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

        let file_data: serde_json::Value = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;

        let content = file_data["content"].as_str()
            .ok_or_else(|| anyhow!("File content not found"))?;

        // GitHub returns base64 encoded content
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(content.replace("\n", ""))
            .map_err(|e| anyhow!("Failed to decode file content: {}", e))?;

        let content_str = String::from_utf8(decoded)
            .map_err(|e| anyhow!("Failed to convert file content to string: {}", e))?;

        Ok(content_str)
    }

    /// Create a comment on a pull request
    #[allow(dead_code)]
    pub async fn create_pull_request_comment(&self, owner: &str, repo: &str, number: u64, body: &str) -> Result<PullRequestComment> {
        let url = format!("{}/repos/{}/{}/issues/{}/comments", self.base_url, owner, repo, number);

        let payload = serde_json::json!({
            "body": body
        });

        let response = self.http_client.post(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "QitOps-Agent")
            .json(&payload)
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

        let comment_data: serde_json::Value = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse GitHub API response: {}", e))?;

        let comment = PullRequestComment {
            id: comment_data["id"].as_u64().unwrap_or_default(),
            body: comment_data["body"].as_str().unwrap_or_default().to_string(),
            user: comment_data["user"]["login"].as_str().unwrap_or_default().to_string(),
            created_at: comment_data["created_at"].as_str().unwrap_or_default().to_string(),
            updated_at: comment_data["updated_at"].as_str().unwrap_or_default().to_string(),
            path: None,
            line: None,
        };

        Ok(comment)
    }
}
