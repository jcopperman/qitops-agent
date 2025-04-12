# QitOps Agent User Guide

QitOps Agent is an AI-powered QA assistant that helps you improve software quality through automated analysis, testing, and risk assessment.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
- [Configuration](#configuration)
- [Use Cases](#use-cases)
- [Examples](#examples)
- [Customization](#customization)
- [Troubleshooting](#troubleshooting)

## Installation

### Windows

```powershell
# Clone the repository
git clone https://github.com/jcopperman/qitops-agent.git
cd qitops-agent

# Run the installation script
.\install.ps1
```

### Linux/macOS

```bash
# Clone the repository
git clone https://github.com/jcopperman/qitops-agent.git
cd qitops-agent

# Run the installation script
chmod +x install.sh
./install.sh
```

## Quick Start

After installation, you can use QitOps Agent with the following commands:

```bash
# Get help
qitops --help

# Generate test cases
qitops run test-gen --path src/user/auth.rs

# Analyze a pull request
qitops run pr-analyze --pr https://github.com/username/repo/pull/123

# Assess risk of changes
qitops run risk --diff https://github.com/username/repo/pull/123
```

## Command Reference

### Test Generation

Generate test cases for your code:

```bash
qitops run test-gen --path <file_or_directory> [options]

Options:
  --format <format>       Output format (markdown, yaml, robot) [default: markdown]
  --component <component> Component to focus on
  --coverage <level>      Coverage level (low, medium, high) [default: medium]
```

### PR Analysis

Analyze pull requests for potential issues:

```bash
qitops run pr-analyze --pr <pr_number_or_url> [options]

Options:
  --focus <areas>         Focus areas (comma-separated: security, performance, etc.)
```

### Risk Assessment

Assess the risk of code changes:

```bash
qitops run risk --diff <diff_file_or_pr> [options]

Options:
  --components <list>     Components to focus on (comma-separated)
  --focus <areas>         Focus areas (comma-separated: security, performance, etc.)
```

### Test Data Generation

Generate test data based on schemas:

```bash
qitops run test-data --schema <schema> [options]

Options:
  --count <number>        Number of records to generate [default: 10]
  --format <format>       Output format (json, csv, yaml) [default: json]
  --constraints <list>    Data constraints (comma-separated)
```

### LLM Management

Configure and manage LLM providers:

```bash
qitops llm list                                # List available providers
qitops llm add --provider <name> [options]     # Add a new provider
qitops llm default --provider <name>           # Set default provider
qitops llm test --provider <name> --prompt <text>  # Test a provider
```

### GitHub Integration

Configure GitHub integration:

```bash
qitops github config --token <token> [options]  # Configure GitHub token
qitops github status                            # Check GitHub configuration
qitops github test                              # Test GitHub connection
```

## Configuration

QitOps Agent can be configured using:

1. **Command-line arguments**: For one-time settings
2. **Configuration files**: For persistent settings
3. **Environment variables**: For sensitive information

### Configuration Files

QitOps Agent stores its configuration in:
- Windows: `%APPDATA%\qitops\config.json`
- Linux/macOS: `~/.config/qitops/config.json`

Example configuration:

```json
{
  "providers": [
    {
      "provider_type": "openai",
      "default_model": "gpt-4",
      "api_key": "YOUR_API_KEY"
    },
    {
      "provider_type": "ollama",
      "default_model": "mistral",
      "api_base": "http://localhost:11434"
    }
  ],
  "default_provider": "ollama",
  "task_providers": {
    "security": "openai",
    "performance": "ollama"
  }
}
```

### Environment Variables

You can use environment variables for sensitive information:

- `GITHUB_TOKEN`: GitHub API token
- `OPENAI_API_KEY`: OpenAI API key
- `ANTHROPIC_API_KEY`: Anthropic API key

## Use Cases

QitOps Agent can be used in various scenarios:

### 1. Developer Workflow

- Generate test cases for new features
- Assess risk before submitting PRs
- Generate test data for local testing

### 2. Code Review Process

- Analyze PRs for potential issues
- Identify security vulnerabilities
- Suggest improvements for code quality

### 3. CI/CD Pipeline

- Automate PR analysis in CI/CD
- Perform risk assessment before deployment
- Generate test cases for new features

### 4. QA Process

- Generate comprehensive test cases
- Create realistic test data
- Identify edge cases and potential issues

## Examples

### Example 1: Analyzing a PR

```bash
# Configure GitHub integration
qitops github config --token YOUR_GITHUB_TOKEN --owner username --repo repository

# Analyze a PR
qitops run pr-analyze --pr https://github.com/username/repo/pull/123
```

Output:
```
PR Analysis:
- 5 files changed (+120/-45 lines)
- Potential issues: 3 (1 high, 2 medium)
- Recommendations: 4

High Priority Issues:
1. Potential SQL injection vulnerability in user input handling
...
```

### Example 2: Generating Test Cases

```bash
# Generate test cases for authentication module
qitops run test-gen --path src/auth --format markdown --coverage high
```

Output:
```
Generated 15 test cases for authentication module:

## Test Case 1: Valid User Login
- **Description**: Test user login with valid credentials
- **Preconditions**: User exists in the system
- **Steps**:
  1. Enter valid username
  2. Enter valid password
  3. Click login button
- **Expected Result**: User is successfully logged in
...
```

### Example 3: Risk Assessment

```bash
# Assess risk of changes in a PR
qitops run risk --diff https://github.com/username/repo/pull/123 --focus security,performance
```

Output:
```
Risk Assessment:
- Overall Risk: Medium
- Security Risk: High
- Performance Risk: Low

Security Concerns:
1. User input is not properly sanitized in the login form
...
```

## Customization

QitOps Agent can be customized in several ways:

### 1. LLM Providers

You can configure multiple LLM providers:

```bash
# Add OpenAI provider
qitops llm add --provider openai --api-key YOUR_API_KEY --model gpt-4

# Add Ollama provider (local LLM)
qitops llm add --provider ollama --api-base http://localhost:11434 --model mistral

# Set default provider
qitops llm default --provider ollama
```

### 2. Task-Specific Providers

You can assign different providers to specific tasks:

```bash
# Use OpenAI for security analysis
qitops llm task --task security --provider openai

# Use Ollama for test generation
qitops llm task --task test-gen --provider ollama
```

### 3. Output Formats

You can customize output formats:

```bash
# Generate test cases in YAML format
qitops run test-gen --path src/auth --format yaml

# Generate test data in CSV format
qitops run test-data --schema user-profile --format csv
```

## Troubleshooting

### Common Issues

1. **Installation Issues**:
   - Ensure Rust is installed correctly
   - Check if the installation script has execute permissions

2. **GitHub Integration Issues**:
   - Verify your GitHub token has the correct permissions
   - Check if the repository owner and name are correct

3. **LLM Provider Issues**:
   - Verify your API keys are correct
   - Check if the LLM provider is accessible

### Verbose Logging

Enable verbose logging for troubleshooting:

```bash
qitops --verbose run pr-analyze --pr 123
```

### Getting Help

If you encounter issues, you can:
- Check the documentation
- Open an issue on GitHub
- Join the community Discord server
