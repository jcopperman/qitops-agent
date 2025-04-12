use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::agent::traits::{Agent, AgentResponse, AgentStatus};
use crate::ci::github::GitHubClient;
use crate::llm::{LlmRequest, LlmRouter};

/// Risk level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical risk
    Critical,
}

/// Risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level
    pub overall_risk: RiskLevel,

    /// Risk breakdown by component
    pub component_risks: Vec<ComponentRisk>,

    /// Risk summary
    pub summary: String,

    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Component risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRisk {
    /// Component name
    pub component: String,

    /// Risk level
    pub risk_level: RiskLevel,

    /// Risk description
    pub description: String,
}

/// Risk assessment agent
pub struct RiskAgent {
    /// Path to the diff file or PR number
    diff_source: String,

    /// Components to focus on
    components: Vec<String>,

    /// Risk focus areas
    focus_areas: Vec<String>,

    /// GitHub client (if using PR)
    github_client: Option<GitHubClient>,

    /// LLM router
    llm_router: LlmRouter,

    /// Repository owner (if using PR)
    owner: Option<String>,

    /// Repository name (if using PR)
    repo: Option<String>,
}

impl RiskAgent {
    /// Create a new risk assessment agent for a diff file
    pub async fn new_from_diff(
        diff_path: String,
        components: Vec<String>,
        focus_areas: Vec<String>,
        llm_router: LlmRouter,
    ) -> Result<Self> {
        Ok(Self {
            diff_source: diff_path,
            components,
            focus_areas,
            github_client: None,
            llm_router,
            owner: None,
            repo: None,
        })
    }

    /// Create a new risk assessment agent for a PR
    pub async fn new_from_pr(
        pr: String,
        components: Vec<String>,
        focus_areas: Vec<String>,
        owner: String,
        repo: String,
        github_token: String,
        llm_router: LlmRouter,
    ) -> Result<Self> {
        let github_client = GitHubClient::new(github_token);

        Ok(Self {
            diff_source: pr,
            components,
            focus_areas,
            github_client: Some(github_client),
            llm_router,
            owner: Some(owner),
            repo: Some(repo),
        })
    }

    /// Read the diff from a file
    fn read_diff_file(&self) -> Result<String> {
        let path = Path::new(&self.diff_source);
        if !path.exists() {
            return Err(anyhow::anyhow!("Diff file not found: {}", self.diff_source));
        }

        fs::read_to_string(path).context(format!("Failed to read diff file: {}", self.diff_source))
    }

    /// Extract PR number from a PR string (number or URL)
    fn extract_pr_number(&self) -> Result<u64> {
        // If it's just a number, parse it directly
        if let Ok(num) = self.diff_source.parse::<u64>() {
            return Ok(num);
        }

        // If it's a URL, extract the number
        if self.diff_source.contains("github.com") && self.diff_source.contains("/pull/") {
            let parts: Vec<&str> = self.diff_source.split("/pull/").collect();
            if parts.len() >= 2 {
                let num_part = parts[1].split('/').next().unwrap_or(parts[1]);
                return num_part.parse::<u64>().context("Failed to parse PR number from URL");
            }
        }

        Err(anyhow::anyhow!("Invalid PR format: {}", self.diff_source))
    }

    /// Generate the prompt for the LLM
    fn generate_prompt(&self, diff: &str) -> String {
        let components_str = if self.components.is_empty() {
            "all components".to_string()
        } else {
            format!("the following components: {}", self.components.join(", "))
        };

        let focus_str = if self.focus_areas.is_empty() {
            "general risk factors".to_string()
        } else {
            format!("the following risk areas: {}", self.focus_areas.join(", "))
        };

        format!(
            "Assess the risk of the following code changes. Focus on {} and {}.\n\nDiff:\n```\n{}\n```\n\nProvide a risk assessment with an overall risk level (Low, Medium, High, or Critical), component-specific risks, a summary, and recommendations.",
            components_str, focus_str, diff
        )
    }

    /// Get the system prompt
    fn system_prompt(&self) -> String {
        "You are a risk assessment expert. Analyze code changes and provide a detailed risk assessment. Consider factors like complexity, scope of changes, critical components affected, potential for regressions, security implications, and performance impact. Provide your assessment in a structured format with an overall risk level, component-specific risks, a summary, and actionable recommendations.".to_string()
    }
}

impl Agent for RiskAgent {
    fn init(&mut self) -> Result<()> {
        // No initialization needed
        Ok(())
    }

    async fn execute(&self) -> Result<AgentResponse> {
        // Get the diff
        let diff = if self.github_client.is_some() {
            // Get diff from GitHub PR
            let pr_number = self.extract_pr_number()?;
            let owner = self.owner.as_ref().ok_or_else(|| anyhow::anyhow!("Repository owner not specified"))?;
            let repo = self.repo.as_ref().ok_or_else(|| anyhow::anyhow!("Repository name not specified"))?;

            self.github_client.as_ref().unwrap().get_pull_request_diff(owner, repo, pr_number).await?
        } else {
            // Read diff from file
            self.read_diff_file()?
        };

        // Generate the prompt
        let prompt = self.generate_prompt(&diff);

        // Create the LLM request
        let model = self.llm_router.default_model().unwrap_or_else(|| "tinyllama".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message(self.system_prompt());

        // Send the request to the LLM
        let response = self.llm_router.send(request, Some("risk")).await?;

        // Return the response
        Ok(AgentResponse {
            status: AgentStatus::Success,
            message: "Risk assessment completed".to_string(),
            data: Some(serde_json::json!({
                "assessment": response.text,
                "components": self.components,
                "focus_areas": self.focus_areas,
            })),
        })
    }

    fn name(&self) -> &str {
        "risk"
    }

    fn description(&self) -> &str {
        "Risk assessment agent"
    }
}
