use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

use crate::agent::traits::{Agent, AgentResponse, AgentStatus};
use crate::ci::github::GitHubClient;
use crate::llm::{LlmRequest, LlmRouter};

/// PR analysis focus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrFocus {
    /// General analysis
    General,
    /// Security analysis
    Security,
    /// Performance analysis
    Performance,
    /// Regression analysis
    Regression,
}

impl PrFocus {
    /// Parse a string into a PR focus
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "general" => Ok(PrFocus::General),
            "security" => Ok(PrFocus::Security),
            "performance" => Ok(PrFocus::Performance),
            "regression" => Ok(PrFocus::Regression),
            _ => Err(anyhow::anyhow!("Unknown PR focus: {}", s)),
        }
    }

    /// Get the system prompt for this focus
    pub fn system_prompt(&self) -> String {
        match self {
            PrFocus::General => "Analyze the pull request diff and provide a general analysis of the changes. Focus on code quality, potential bugs, and areas that might need more testing.".to_string(),
            PrFocus::Security => "Analyze the pull request diff with a focus on security issues. Look for potential vulnerabilities, insecure coding practices, and security risks.".to_string(),
            PrFocus::Performance => "Analyze the pull request diff with a focus on performance issues. Look for inefficient code, potential bottlenecks, and areas that might impact performance.".to_string(),
            PrFocus::Regression => "Analyze the pull request diff with a focus on potential regressions. Look for changes that might break existing functionality or introduce compatibility issues.".to_string(),
        }
    }
}

/// PR analysis agent
pub struct PrAnalyzeAgent {
    /// PR number or URL
    pr: String,

    /// PR focus
    focus: PrFocus,

    /// GitHub client
    github_client: GitHubClient,

    /// LLM router
    llm_router: LlmRouter,

    /// Repository owner
    owner: String,

    /// Repository name
    repo: String,
}

impl PrAnalyzeAgent {
    /// Create a new PR analysis agent
    pub async fn new(
        pr: String,
        focus: Option<String>,
        owner: String,
        repo: String,
        github_client: GitHubClient,
        llm_router: LlmRouter
    ) -> Result<Self> {
        let focus = match focus {
            Some(f) => PrFocus::from_str(&f)?,
            None => PrFocus::General,
        };

        Ok(Self {
            pr,
            focus,
            github_client,
            llm_router,
            owner,
            repo,
        })
    }

    /// Extract PR number from a PR string (number or URL)
    fn extract_pr_number(&self) -> Result<u64> {
        // If it's just a number, parse it directly
        if let Ok(num) = self.pr.parse::<u64>() {
            return Ok(num);
        }

        // If it's a URL, extract the number
        if self.pr.contains("github.com") && self.pr.contains("/pull/") {
            let parts: Vec<&str> = self.pr.split("/pull/").collect();
            if parts.len() >= 2 {
                let num_part = parts[1].split('/').next().unwrap_or(parts[1]);
                return num_part.parse::<u64>().context("Failed to parse PR number from URL");
            }
        }

        Err(anyhow::anyhow!("Invalid PR format: {}", self.pr))
    }

    /// Generate the prompt for the LLM
    fn generate_prompt(&self, pr_info: &str, diff: &str) -> String {
        format!(
            "Analyze the following pull request:\n\n{}\n\nDiff:\n```\n{}\n```",
            pr_info, diff
        )
    }
}

impl Agent for PrAnalyzeAgent {
    fn init(&mut self) -> Result<()> {
        // No initialization needed
        Ok(())
    }

    async fn execute(&self) -> Result<AgentResponse> {
        // Extract PR number
        let pr_number = self.extract_pr_number()?;

        // Get PR information
        let pr_info = self.github_client.get_pull_request(&self.owner, &self.repo, pr_number).await?;

        // Get PR diff
        let diff = self.github_client.get_pull_request_diff(&self.owner, &self.repo, pr_number).await?;

        // Get PR files
        let files = self.github_client.get_pull_request_files(&self.owner, &self.repo, pr_number).await?;

        // Generate file summary
        let file_summary = files.iter().map(|f| {
            format!("{} ({}, +{}, -{})", f.filename, f.status, f.additions, f.deletions)
        }).collect::<Vec<String>>().join("\n");

        // Generate the prompt
        let prompt = self.generate_prompt(
            &format!(
                "Title: {}\nDescription: {}\n\nFiles Changed:\n{}",
                pr_info.title,
                pr_info.body.unwrap_or_default(),
                file_summary
            ),
            &diff
        );

        // Create the LLM request
        let model = self.llm_router.default_model().unwrap_or_else(|| "tinyllama".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message(self.focus.system_prompt());

        // Send the request to the LLM
        let response = self.llm_router.send(request, Some("pr-analyze")).await?;

        // Return the response
        Ok(AgentResponse {
            status: AgentStatus::Success,
            message: format!("PR analysis completed for PR #{}", pr_number),
            data: Some(serde_json::json!({
                "pr_number": pr_number,
                "pr_title": pr_info.title,
                "analysis": response.text,
                "focus": format!("{:?}", self.focus),
                "files_changed": files.len(),
            })),
        })
    }

    fn name(&self) -> &str {
        "pr-analyze"
    }

    fn description(&self) -> &str {
        "Pull request analyzer"
    }
}
