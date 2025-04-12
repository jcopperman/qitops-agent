# QitOps Agent Frequently Asked Questions

## General Questions

### What is QitOps Agent?

QitOps Agent is an AI-powered QA assistant that helps you improve software quality through automated analysis, testing, and risk assessment. It uses large language models (LLMs) to generate test cases, analyze pull requests, assess risk, and generate test data.

### What programming languages does QitOps Agent support?

QitOps Agent is language-agnostic and can work with any programming language. The LLMs it uses understand most popular programming languages, including but not limited to:
- JavaScript/TypeScript
- Python
- Java
- C#
- Rust
- Go
- Ruby
- PHP

### Is QitOps Agent free to use?

QitOps Agent itself is open-source and free to use. However, if you use cloud-based LLM providers like OpenAI or Anthropic, you will need to pay for their API usage. You can use Ollama with open-source models for free.

### Does QitOps Agent require internet access?

It depends on your LLM provider:
- If you use OpenAI or Anthropic, you need internet access
- If you use Ollama with local models, you don't need internet access
- For GitHub integration, you need internet access

## Installation Questions

### How do I install QitOps Agent?

You can install QitOps Agent using the provided installation scripts:

**Windows:**
```powershell
git clone https://github.com/jcopperman/qitops-agent.git
cd qitops-agent
.\install.ps1
```

**Linux/macOS:**
```bash
git clone https://github.com/jcopperman/qitops-agent.git
cd qitops-agent
chmod +x install.sh
./install.sh
```

### What are the system requirements?

- Operating System: Windows, macOS, or Linux
- Rust toolchain (1.70+) for building from source
- At least 1GB of RAM
- For Ollama: Additional RAM depending on the model size

### Can I install QitOps Agent without Rust?

Currently, you need Rust to build QitOps Agent from source. In the future, we plan to provide pre-built binaries for common platforms.

### How do I update QitOps Agent?

To update QitOps Agent, pull the latest changes from the repository and run the installation script again:

```bash
cd qitops-agent
git pull
./install.sh  # or .\install.ps1 on Windows
```

## Configuration Questions

### How do I configure QitOps Agent to use OpenAI?

```bash
qitops llm add --provider openai --api-key YOUR_API_KEY --model gpt-4
qitops llm default --provider openai
```

### How do I configure QitOps Agent to use Ollama?

First, install Ollama from [ollama.ai](https://ollama.ai) and pull a model:

```bash
ollama pull mistral
```

Then configure QitOps Agent:

```bash
qitops llm add --provider ollama --api-base http://localhost:11434 --model mistral
qitops llm default --provider ollama
```

### How do I configure GitHub integration?

```bash
qitops github config --token YOUR_GITHUB_TOKEN --owner username --repo repository
```

### Where are configuration files stored?

QitOps Agent stores its configuration in:
- Windows: `%APPDATA%\qitops\config.json`
- Linux/macOS: `~/.config/qitops\config.json`

### Can I use environment variables for configuration?

Yes, you can use environment variables for sensitive information:

```bash
# Windows (PowerShell)
$env:GITHUB_TOKEN = "your-github-token"
$env:OPENAI_API_KEY = "your-api-key"

# Linux/macOS
export GITHUB_TOKEN="your-github-token"
export OPENAI_API_KEY="your-api-key"
```

## Usage Questions

### How do I generate test cases?

```bash
# Basic test generation
qitops run test-gen --path src/module.rs

# With high coverage
qitops run test-gen --path src/module.rs --coverage high

# In YAML format
qitops run test-gen --path src/module.rs --format yaml
```

### How do I analyze a pull request?

```bash
# By PR URL
qitops run pr-analyze --pr https://github.com/username/repo/pull/123

# By PR number (if you've configured the default repository)
qitops run pr-analyze --pr 123

# With focus on security
qitops run pr-analyze --pr 123 --focus security
```

### How do I assess risk of changes?

```bash
# From a PR
qitops run risk --diff https://github.com/username/repo/pull/123

# From a local diff file
qitops run risk --diff changes.diff

# With focus on security
qitops run risk --diff changes.diff --focus security
```

### How do I generate test data?

```bash
# Basic test data generation
qitops run test-data --schema user-profile --count 10

# In CSV format
qitops run test-data --schema user-profile --format csv

# With constraints
qitops run test-data --schema user-profile --constraints "age>21,country=US"
```

### Can I use QitOps Agent in CI/CD pipelines?

Yes, QitOps Agent can be integrated into CI/CD pipelines. See the [CI/CD Integration Guide](CI_CD_INTEGRATION.md) for details.

## LLM Questions

### Which LLM provider should I use?

- **OpenAI**: Best for high-quality results, but requires an API key and internet access
- **Anthropic**: Good for detailed analysis, but requires an API key and internet access
- **Ollama**: Good for privacy and no API costs, but requires local resources

### How much do LLM providers cost?

- **OpenAI**: Varies by model, typically $0.01-$0.10 per 1K tokens
- **Anthropic**: Varies by model, typically $0.01-$0.15 per 1K tokens
- **Ollama**: Free (uses local resources)

### Can I use multiple LLM providers?

Yes, you can configure multiple LLM providers and assign them to specific tasks:

```bash
# Use OpenAI for security analysis
qitops llm task --task security --provider openai

# Use Ollama for test generation
qitops llm task --task test-gen --provider ollama
```

### How do I know which LLM provider is being used?

You can check the current configuration with:

```bash
qitops llm list
```

## GitHub Integration Questions

### What GitHub permissions does QitOps Agent need?

QitOps Agent needs the following GitHub permissions:
- `repo` scope for private repositories
- `public_repo` scope for public repositories

### Can I use QitOps Agent with GitHub Enterprise?

Yes, you can configure QitOps Agent to use GitHub Enterprise:

```bash
qitops github config --api-base https://github.example.com/api/v3
```

### Can I use QitOps Agent without GitHub integration?

Yes, you can use QitOps Agent without GitHub integration for:
- Test case generation
- Risk assessment from local diff files
- Test data generation

## CI/CD Integration Questions

### How do I integrate QitOps Agent with GitHub Actions?

Create a workflow file in `.github/workflows/qitops-pr-analysis.yml`:

```yaml
name: QitOps PR Analysis

on:
  pull_request:
    types: [opened, synchronize, reopened]

jobs:
  analyze-pr:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        
      - name: Install QitOps Agent
        run: |
          git clone https://github.com/jcopperman/qitops-agent.git /tmp/qitops-agent
          cd /tmp/qitops-agent
          chmod +x install.sh
          ./install.sh
          echo "$HOME/.qitops/bin" >> $GITHUB_PATH
          
      - name: Configure QitOps
        run: |
          qitops github config --token ${{ secrets.GITHUB_TOKEN }}
        
      - name: Analyze PR
        run: |
          qitops run pr-analyze --pr ${{ github.event.pull_request.number }}
```

### How do I integrate QitOps Agent with GitLab CI?

Create a `.gitlab-ci.yml` file:

```yaml
stages:
  - analyze

pr_analysis:
  stage: analyze
  image: rust:1.77
  script:
    - git clone https://github.com/jcopperman/qitops-agent.git /tmp/qitops-agent
    - cd /tmp/qitops-agent
    - chmod +x install.sh
    - ./install.sh
    - export PATH="$HOME/.qitops/bin:$PATH"
    - qitops github config --token $GITHUB_TOKEN
    - qitops run pr-analyze --pr $CI_MERGE_REQUEST_IID
  only:
    - merge_requests
```

### Can I use QitOps Agent with Jenkins?

Yes, you can use QitOps Agent with Jenkins. See the [CI/CD Integration Guide](CI_CD_INTEGRATION.md) for details.

## Troubleshooting Questions

### QitOps Agent is not found after installation

Make sure the installation directory is in your PATH:

```bash
# Windows (PowerShell)
$env:Path += ";$env:USERPROFILE\.qitops\bin"

# Linux/macOS
export PATH="$HOME/.qitops/bin:$PATH"
```

### I'm getting authentication errors with GitHub

Check that your GitHub token has the correct permissions and hasn't expired.

### I'm getting API errors with OpenAI/Anthropic

Check that your API key is correct and that you have sufficient credits.

### Ollama is not responding

Make sure Ollama is running:

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags
```

### How do I enable verbose logging?

Use the `--verbose` flag:

```bash
qitops --verbose run pr-analyze --pr 123
```

## Feature Questions

### Can QitOps Agent generate unit tests?

Yes, QitOps Agent can generate unit tests using the test-gen command:

```bash
qitops run test-gen --path src/module.rs --format code
```

### Can QitOps Agent analyze code quality?

Yes, QitOps Agent can analyze code quality as part of PR analysis:

```bash
qitops run pr-analyze --pr 123 --focus "code-quality,maintainability"
```

### Can QitOps Agent detect security vulnerabilities?

Yes, QitOps Agent can detect potential security vulnerabilities:

```bash
qitops run pr-analyze --pr 123 --focus security
qitops run risk --diff changes.diff --focus security
```

### Can QitOps Agent suggest improvements?

Yes, QitOps Agent can suggest improvements as part of PR analysis and risk assessment.

### Can QitOps Agent generate documentation?

This feature is planned for a future release.
