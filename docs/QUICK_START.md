# QitOps Agent Quick Start Guide

This guide will help you get started with QitOps Agent quickly and show you how to use its main features.

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

## First Steps

After installation, verify that QitOps Agent is working:

```bash
qitops --help
```

You should see the help message with available commands.

## Setting Up LLM Integration

QitOps Agent needs an LLM provider to function. You can use OpenAI, Anthropic, or Ollama (local LLM).

### Option 1: Using Ollama (Local LLM)

1. Install Ollama from [ollama.ai](https://ollama.ai)
2. Pull the Mistral model:
   ```bash
   ollama pull mistral
   ```
3. Configure QitOps to use Ollama:
   ```bash
   qitops llm add --provider ollama --api-base http://localhost:11434 --model mistral
   qitops llm default --provider ollama
   ```

### Option 2: Using OpenAI

1. Get an API key from [OpenAI](https://platform.openai.com)
2. Configure QitOps to use OpenAI:
   ```bash
   qitops llm add --provider openai --api-key YOUR_API_KEY --model gpt-4
   qitops llm default --provider openai
   ```

## Setting Up GitHub Integration

To analyze PRs and assess risks from GitHub:

```bash
# Configure GitHub integration
qitops github config --token YOUR_GITHUB_TOKEN --owner username --repo repository

# Test the connection
qitops github test
```

## Example Workflows

### Example 1: Generate Test Cases for a File

```bash
# Generate test cases for a specific file
qitops run test-gen --path src/user/auth.rs --format markdown
```

This will generate test cases for the authentication module in Markdown format.

### Example 2: Analyze a Pull Request

```bash
# Analyze a PR by URL
qitops run pr-analyze --pr https://github.com/username/repo/pull/123

# Or by PR number (if you've configured the default repository)
qitops run pr-analyze --pr 123
```

This will analyze the pull request for potential issues, code quality, and provide recommendations.

### Example 3: Assess Risk of Changes

```bash
# Assess risk from a PR
qitops run risk --diff https://github.com/username/repo/pull/123 --focus security,performance

# Or from a local diff file
qitops run risk --diff changes.diff --components auth,payment
```

This will assess the risk of the changes, focusing on security and performance aspects.

### Example 4: Generate Test Data

```bash
# Generate test data based on a schema
qitops run test-data --schema user-profile --count 10
```

This will generate 10 test data records based on the user profile schema.

## Common Command Patterns

### Test Generation Options

```bash
# Basic test generation
qitops run test-gen --path src/module.rs

# With high coverage
qitops run test-gen --path src/module.rs --coverage high

# For a specific component
qitops run test-gen --path src/module.rs --component authentication

# In YAML format
qitops run test-gen --path src/module.rs --format yaml
```

### PR Analysis Options

```bash
# Basic PR analysis
qitops run pr-analyze --pr 123

# With focus on security
qitops run pr-analyze --pr 123 --focus security

# With focus on multiple areas
qitops run pr-analyze --pr 123 --focus "security,performance,maintainability"
```

### Risk Assessment Options

```bash
# Basic risk assessment
qitops run risk --diff changes.diff

# With component focus
qitops run risk --diff changes.diff --components "auth,payment"

# With area focus
qitops run risk --diff changes.diff --focus "security,performance"
```

## Next Steps

- Read the full [User Guide](USER_GUIDE.md) for detailed information
- Check the [Configuration Guide](CONFIGURATION.md) for advanced configuration
- See the [CI/CD Integration Guide](CI_CD_INTEGRATION.md) for CI/CD setup
