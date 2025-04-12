# QitOps Agent CLI

<p align="center">
  <strong>AI-powered QA Assistant for Modern Development Teams</strong>
</p>

## Overview

QitOps Agent is a powerful CLI tool that leverages AI to enhance your QA and testing workflows. It represents the culmination of a vision: to reimagine Quality Assurance not as an afterthought or gatekeeper, but as an **embedded, intelligent, human-centered force for stability, trust, and creativity in software development**.

## Features

- **Test Case Generation**: Automatically generate comprehensive test cases for your code
- **Pull Request Analysis**: Get AI-powered insights on PRs to identify potential issues
- **Risk Assessment**: Estimate the risk of code changes to prioritize testing efforts
- **Test Data Generation**: Create realistic test data for your applications
- **Interactive Testing Sessions**: Get AI guidance during exploratory testing
- **Flexible LLM Integration**: Use local models (Ollama) or cloud providers (OpenAI, Anthropic)

## Installation

### From Source

#### Windows

```powershell
# Clone the repository
git clone https://github.com/jcopperman/qitops-agent.git
cd qitops-agent

# Run the installation script
.\install.ps1
```

#### Linux/macOS

```bash
# Clone the repository
git clone https://github.com/jcopperman/qitops-agent.git
cd qitops-agent

# Run the installation script
chmod +x install.sh
./install.sh
```

### Using Cargo

```bash
cargo install qitops-agent
```

## Usage

### Basic Commands

```bash
# Get help
qitops --help

# Generate test cases
qitops run test-gen --path src/user/auth.rs --format markdown

# Analyze a pull request
qitops run pr-analyze --pr 123

# Estimate risk of changes
qitops run risk --diff changes.diff

# Generate test data
qitops run test-data --schema user-profile --count 100

# Start an interactive testing session
qitops run session --name "Login Flow Test"
```

### LLM Management

QitOps Agent supports multiple LLM providers:

```bash
# List available providers
qitops llm list

# Add a new provider
qitops llm add --provider openai --api-key YOUR_API_KEY --model gpt-4

# Set default provider
qitops llm default --provider ollama

# Test a provider
qitops llm test --provider anthropic --prompt "Generate a test case for user authentication"
```

## Configuration

QitOps Agent stores its configuration in `~/.config/qitops/config.json` (Linux/macOS) or `%APPDATA%\qitops\config.json` (Windows).

### LLM Configuration

You can configure multiple LLM providers:

- **OpenAI**: Requires an API key
- **Anthropic**: Requires an API key
- **Ollama**: Local LLM, no API key required

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
      "default_model": "llama3",
      "api_base": "http://localhost:11434"
    }
  ],
  "default_provider": "ollama",
  "task_providers": {
    "security": "openai",
    "performance": "anthropic"
  }
}
```

### GitHub Configuration

QitOps Agent can integrate with GitHub for PR analysis and risk assessment. You can configure GitHub integration using the CLI:

```bash
# Configure GitHub token
qitops github config --token YOUR_GITHUB_TOKEN

# Configure default repository
qitops github config --owner username --repo repository

# Check GitHub configuration status
qitops github status

# Test GitHub connection
qitops github test
```

This configuration allows you to analyze PRs and assess risks directly from GitHub URLs or PR numbers.

## Real-World Testing Scenarios

QitOps Agent can be used in various real-world testing scenarios:

### 1. Automated Test Case Generation

Generate comprehensive test cases for your applications:

```bash
# Generate test cases for a specific feature or component
qitops run test-gen --component user-authentication --coverage high

# Generate test cases with specific focus areas
qitops run test-gen --focus edge-cases --component payment-processing
```

### 2. Pull Request Analysis

Integrate into your CI/CD pipeline:

```bash
# Analyze a pull request for potential issues
qitops run pr-analyze --pr https://github.com/username/repo/pull/123

# Analyze with specific focus on security concerns
qitops run pr-analyze --pr 123 --focus security
```

### 3. Risk Assessment

For critical changes, use the risk estimation feature:

```bash
# Estimate the risk of changes in a specific PR
qitops run risk --diff https://github.com/username/repo/pull/123 --components "payment,user-data"

# Estimate risk with specific focus
qitops run risk --diff 123 --focus "data-integrity,security"
```

### 4. Test Data Generation

Generate realistic test data for your applications:

```bash
# Generate test data for a specific schema
qitops run test-data --schema user-profile --count 100

# Generate test data with specific constraints
qitops run test-data --schema financial-transaction --constraints "amount<1000,currency=USD" --count 50
```

### 5. Interactive Testing Sessions

For exploratory testing:

```bash
# Start an interactive testing session
qitops run session --application web-app --focus "user-journey"

# Start a session with specific test objectives
qitops run session --application mobile-app --objectives "verify-payment-flow,test-error-handling"
```

## Development

### Project Structure

```
qitops-agent/
├── src/
│   ├── agent/       # Core agent functionality
│   ├── cli/         # CLI interface
│   ├── llm/         # LLM integration
│   ├── plugin/      # Plugin system
│   └── ci/          # CI integration
├── tests/           # Tests
└── docs/            # Documentation
```

### Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with verbose logging
cargo run -- --verbose llm list

# Or if installed
qitops --verbose llm list
```

## Contributing

Whether you're a prompt engineer, test automation expert, junior QA analyst, or just curious, you're welcome to contribute to QitOps Agent.

- Browse the [good first issues](https://github.com/jcopperman/qitops-agent/issues)
- Share ideas in [QitOps Discord](#) *(coming soon)*
- Follow [@jcopperman](https://github.com/jcopperman) for updates

This project thrives on contribution, curiosity, and experimentation.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  Made with ❤️ by the QitOps Team
</p>
