use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn, info};

use crate::agent::traits::{Agent, AgentResponse, AgentStatus};
use crate::ci::github::{GitHubClient, PullRequestFile};
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

impl std::str::FromStr for PrFocus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "general" => Ok(PrFocus::General),
            "security" => Ok(PrFocus::Security),
            "performance" => Ok(PrFocus::Performance),
            "regression" => Ok(PrFocus::Regression),
            _ => Err(anyhow::anyhow!("Unknown PR focus: {}", s)),
        }
    }
}

impl PrFocus {
    /// Get the enhanced system prompt for this focus
    pub fn system_prompt(&self) -> String {
        match self {
            PrFocus::General => {
                "You are an expert code reviewer with deep knowledge of software development best practices. \
                Your task is to analyze the pull request and provide a comprehensive review that covers:

\
                1. Code Quality: Evaluate the code for readability, maintainability, and adherence to best practices
\
                2. Potential Bugs: Identify any logic errors, edge cases, or other issues that could cause bugs
\
                3. Testing Needs: Suggest areas that need more testing or specific test cases
\
                4. Design Patterns: Comment on the use of appropriate design patterns and architectural choices
\
                5. Documentation: Assess the quality and completeness of comments and documentation

\
                Provide specific, actionable feedback with examples where possible. Be thorough but constructive in your analysis.".to_string()
            },
            PrFocus::Security => {
                "You are a security expert specializing in code security reviews. \
                Your task is to analyze the pull request with a focus on security issues, including:

\
                1. Vulnerabilities: Identify potential security vulnerabilities such as injection attacks, authentication issues, etc.
\
                2. Insecure Coding Practices: Point out any practices that could lead to security issues
\
                3. Data Protection: Evaluate how sensitive data is handled and protected
\
                4. Input Validation: Check for proper validation of all inputs
\
                5. Security Controls: Assess the implementation of security controls and safeguards

\
                For each issue, explain the potential impact and suggest specific remediation steps. Prioritize issues by severity.".to_string()
            },
            PrFocus::Performance => {
                "You are a performance optimization specialist with expertise in identifying performance bottlenecks. \
                Your task is to analyze the pull request with a focus on performance issues, including:

\
                1. Algorithmic Efficiency: Evaluate the time and space complexity of algorithms
\
                2. Resource Usage: Identify inefficient use of memory, CPU, network, or disk resources
\
                3. Bottlenecks: Pinpoint potential performance bottlenecks
\
                4. Scalability: Assess how well the code will scale under increased load
\
                5. Optimization Opportunities: Suggest specific optimizations with expected improvements

\
                Provide concrete examples and benchmarking suggestions where appropriate. Quantify performance impacts when possible.".to_string()
            },
            PrFocus::Regression => {
                "You are a quality assurance expert specializing in regression analysis. \
                Your task is to analyze the pull request with a focus on potential regressions, including:

\
                1. Backward Compatibility: Identify changes that might break existing functionality
\
                2. API Changes: Evaluate changes to public interfaces and their impact
\
                3. Dependencies: Assess changes to dependencies and their potential effects
\
                4. Side Effects: Identify unintended side effects of changes
\
                5. Test Coverage: Evaluate whether existing tests adequately cover the changes

\
                For each potential regression, explain the impact and suggest mitigation strategies. Recommend specific regression tests.".to_string()
            },
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
    /// Create a new PR analysis agent with enhanced input validation
    pub async fn new(
        pr: String,
        focus: Option<String>,
        owner: String,
        repo: String,
        github_client: GitHubClient,
        llm_router: LlmRouter
    ) -> Result<Self> {
        // Validate PR input
        if pr.is_empty() {
            return Err(anyhow::anyhow!("PR number or URL cannot be empty"));
        }

        // Validate owner and repo
        if owner.is_empty() {
            return Err(anyhow::anyhow!("Repository owner cannot be empty"));
        }

        if repo.is_empty() {
            return Err(anyhow::anyhow!("Repository name cannot be empty"));
        }

        // Parse focus with better error handling
        let focus = match focus {
            Some(f) => {
                f.parse::<PrFocus>().context(format!("Invalid PR focus: '{}'. Supported values are: general, security, performance, regression", f))?
            },
            None => PrFocus::General,
        };

        // Create the agent
        let agent = Self {
            pr,
            focus,
            github_client,
            llm_router,
            owner,
            repo,
        };

        // Validate that we can extract a PR number
        agent.extract_pr_number()?;

        Ok(agent)
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

    /// Generate the prompt for the LLM with enhanced context
    async fn generate_prompt(&self, pr_info: &str, diff: &str, files: &[PullRequestFile]) -> Result<String> {
        // Try to initialize repository context
        let current_dir = std::env::current_dir()?;
        let repo_context: Option<crate::context::RepositoryContext> = match crate::context::RepositoryContext::new(&current_dir) {
            Ok(context) => Some(context),
            Err(e) => {
                debug!("Failed to initialize repository context: {}", e);
                None
            }
        };

        // Generate repository context if available
        let repo_context_str = if let Some(context) = &repo_context {
            format!("\n\nRepository Context:\n{}\n", context.generate_context(1000))
        } else {
            String::new()
        };

        // Generate file context for changed files
        let mut file_contexts = Vec::new();
        if let Some(mut context) = repo_context {
            for file in files.iter().take(3) { // Limit to 3 files to avoid too much context
                if let Ok(file_context) = context.generate_file_context(&file.filename, true, true) {
                    file_contexts.push(format!("File: {}\n{}", file.filename, file_context));
                }
            }
        }

        let file_contexts_str = if !file_contexts.is_empty() {
            format!("\n\nChanged Files Context:\n{}\n", file_contexts.join("\n\n"))
        } else {
            String::new()
        };

        // Build the enhanced prompt
        let prompt = format!(
            "Analyze the following pull request:\n\n{}{}{}\n\nDiff:\n```\n{}\n```\n\nProvide a detailed analysis focusing on code quality, potential bugs, and areas that might need more testing. Include specific recommendations for improvements if applicable.",
            pr_info, repo_context_str, file_contexts_str, diff
        );

        Ok(prompt)
    }
}

impl Agent for PrAnalyzeAgent {
    fn init(&mut self) -> Result<()> {
        // No initialization needed
        Ok(())
    }

    async fn execute(&self) -> Result<AgentResponse> {
        // Extract PR number
        let pr_number = match self.extract_pr_number() {
            Ok(num) => num,
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to extract PR number: {}", e),
                    data: None,
                });
            }
        };

        // Get PR information
        let pr_info = match self.github_client.get_pull_request(&self.owner, &self.repo, pr_number).await {
            Ok(info) => info,
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to get PR information: {}", e),
                    data: Some(serde_json::json!({
                        "pr_number": pr_number,
                        "error": format!("{}", e),
                    })),
                });
            }
        };

        // Get PR diff
        let diff = match self.github_client.get_pull_request_diff(&self.owner, &self.repo, pr_number).await {
            Ok(diff) => diff,
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to get PR diff: {}", e),
                    data: Some(serde_json::json!({
                        "pr_number": pr_number,
                        "pr_title": pr_info.title,
                        "error": format!("{}", e),
                    })),
                });
            }
        };

        // Get PR files
        let files = match self.github_client.get_pull_request_files(&self.owner, &self.repo, pr_number).await {
            Ok(files) => files,
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to get PR files: {}", e),
                    data: Some(serde_json::json!({
                        "pr_number": pr_number,
                        "pr_title": pr_info.title,
                        "error": format!("{}", e),
                    })),
                });
            }
        };

        // Generate file summary
        let file_summary = files.iter().map(|f| {
            format!("{} ({}, +{}, -{})", f.filename, f.status, f.additions, f.deletions)
        }).collect::<Vec<String>>().join("\n");

        // Generate the prompt with enhanced context
        let pr_info_str = format!(
            "Title: {}\nDescription: {}\n\nFiles Changed:\n{}",
            pr_info.title,
            pr_info.body.unwrap_or_default(),
            file_summary
        );

        info!("Generating enhanced prompt for PR analysis");
        let prompt = match self.generate_prompt(&pr_info_str, &diff, &files).await {
            Ok(p) => {
                info!("Successfully generated enhanced prompt with length: {}", p.len());
                debug!("Enhanced prompt: {}", p);
                p
            },
            Err(e) => {
                // Fall back to basic prompt if context generation fails
                warn!("Failed to generate enhanced prompt: {}", e);
                let basic_prompt = format!(
                    "Analyze the following pull request:\n\n{}\n\nDiff:\n```\n{}\n```",
                    pr_info_str, diff
                );
                info!("Using basic prompt with length: {}", basic_prompt.len());
                basic_prompt
            }
        };

        // Create the LLM request
        let model = self.llm_router.default_model().unwrap_or_else(|| "mistral".to_string());
        let request = LlmRequest::new(prompt, model)
            .with_system_message(self.focus.system_prompt());

        // Send the request to the LLM
        let response = match self.llm_router.send(request, Some("pr-analyze")).await {
            Ok(response) => response,
            Err(e) => {
                return Ok(AgentResponse {
                    status: AgentStatus::Error,
                    message: format!("Failed to get response from LLM: {}", e),
                    data: Some(serde_json::json!({
                        "pr_number": pr_number,
                        "pr_title": pr_info.title,
                        "focus": format!("{:?}", self.focus),
                        "files_changed": files.len(),
                        "error": format!("{}", e),
                    })),
                });
            }
        };

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
                "model": response.model,
                "provider": response.provider,
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
