# QitOps Agent Demonstration Script

This script provides a step-by-step demonstration of QitOps Agent's capabilities. You can use it for presentations, training, or to explore the tool's features.

## Setup

Before starting the demonstration, ensure you have:

1. Installed QitOps Agent
2. Configured an LLM provider
3. Set up GitHub integration (optional for some demos)

```bash
# Verify installation
qitops --version

# Check LLM configuration
qitops llm list

# Check GitHub configuration (optional)
qitops github status
```

## Demo 1: Test Case Generation

This demo shows how QitOps Agent can generate test cases for your code.

### Step 1: Select a Code File

Choose a file from your project that you want to generate test cases for. For example, a user authentication module:

```bash
# View the file
cat src/auth/login.rs
```

### Step 2: Generate Test Cases

```bash
# Generate test cases in Markdown format
qitops run test-gen --path src/auth/login.rs --format markdown
```

### Step 3: Explore Different Options

```bash
# Generate test cases with high coverage
qitops run test-gen --path src/auth/login.rs --coverage high

# Generate test cases focusing on security
qitops run test-gen --path src/auth/login.rs --component security

# Generate test cases in YAML format
qitops run test-gen --path src/auth/login.rs --format yaml
```

### Step 4: Generate Test Cases for a Directory

```bash
# Generate test cases for an entire module
qitops run test-gen --path src/auth --format markdown
```

## Demo 2: Pull Request Analysis

This demo shows how QitOps Agent can analyze pull requests for potential issues.

### Step 1: Select a Pull Request

Choose a pull request from your GitHub repository:

```bash
# List recent PRs (requires GitHub integration)
qitops github list-prs
```

### Step 2: Analyze the Pull Request

```bash
# Analyze a PR by URL
qitops run pr-analyze --pr https://github.com/username/repo/pull/123

# Or by PR number (if you've configured the default repository)
qitops run pr-analyze --pr 123
```

### Step 3: Explore Different Focus Areas

```bash
# Focus on security
qitops run pr-analyze --pr 123 --focus security

# Focus on performance
qitops run pr-analyze --pr 123 --focus performance

# Focus on multiple areas
qitops run pr-analyze --pr 123 --focus "security,performance,maintainability"
```

## Demo 3: Risk Assessment

This demo shows how QitOps Agent can assess the risk of code changes.

### Step 1: Select Changes to Assess

You can use a PR, a diff file, or a branch:

```bash
# Create a diff file (optional)
git diff main..feature-branch > changes.diff
```

### Step 2: Assess Risk

```bash
# Assess risk from a PR
qitops run risk --diff https://github.com/username/repo/pull/123

# Or from a local diff file
qitops run risk --diff changes.diff
```

### Step 3: Explore Different Focus Areas

```bash
# Focus on specific components
qitops run risk --diff changes.diff --components "auth,payment"

# Focus on specific areas
qitops run risk --diff changes.diff --focus "security,performance"

# Combine both
qitops run risk --diff changes.diff --components "auth,payment" --focus "security,performance"
```

## Demo 4: Test Data Generation

This demo shows how QitOps Agent can generate test data based on schemas.

### Step 1: Define a Schema

Create a simple schema for test data generation:

```bash
# Create a schema file (optional)
cat > user-profile.json << EOF
{
  "name": "User Profile",
  "fields": [
    {"name": "id", "type": "uuid"},
    {"name": "username", "type": "string", "min_length": 3, "max_length": 20},
    {"name": "email", "type": "email"},
    {"name": "age", "type": "integer", "min": 18, "max": 100},
    {"name": "country", "type": "string", "enum": ["US", "UK", "CA", "AU"]}
  ]
}
EOF
```

### Step 2: Generate Test Data

```bash
# Generate test data based on the schema
qitops run test-data --schema user-profile --count 10
```

### Step 3: Explore Different Options

```bash
# Generate more records
qitops run test-data --schema user-profile --count 20

# Generate in CSV format
qitops run test-data --schema user-profile --format csv

# Generate with constraints
qitops run test-data --schema user-profile --constraints "age>21,country=US"
```

## Demo 5: LLM Provider Configuration

This demo shows how to configure different LLM providers.

### Step 1: List Current Providers

```bash
# List configured providers
qitops llm list
```

### Step 2: Add Different Providers

```bash
# Add OpenAI provider
qitops llm add --provider openai --api-key YOUR_API_KEY --model gpt-4

# Add Ollama provider
qitops llm add --provider ollama --api-base http://localhost:11434 --model mistral
```

### Step 3: Set Default Provider

```bash
# Set default provider
qitops llm default --provider ollama
```

### Step 4: Configure Task-Specific Providers

```bash
# Use OpenAI for security analysis
qitops llm task --task security --provider openai

# Use Ollama for test generation
qitops llm task --task test-gen --provider ollama
```

### Step 5: Test Providers

```bash
# Test OpenAI provider
qitops llm test --provider openai --prompt "Generate a test case for user authentication"

# Test Ollama provider
qitops llm test --provider ollama --prompt "Generate a test case for user authentication"
```

## Demo 6: GitHub Integration

This demo shows how to configure and use GitHub integration.

### Step 1: Configure GitHub Integration

```bash
# Configure GitHub token
qitops github config --token YOUR_GITHUB_TOKEN

# Configure default repository
qitops github config --owner username --repo repository
```

### Step 2: Test GitHub Connection

```bash
# Test GitHub connection
qitops github test
```

### Step 3: Use GitHub Integration

```bash
# Analyze a PR by URL
qitops run pr-analyze --pr https://github.com/username/repo/pull/123

# Analyze a PR by number
qitops run pr-analyze --pr 123

# Assess risk from a PR
qitops run risk --diff https://github.com/username/repo/pull/123
```

## Demo 7: CI/CD Integration

This demo shows how to use QitOps Agent in CI/CD pipelines.

### Step 1: Show GitHub Actions Workflow

```bash
# Show GitHub Actions workflow file
cat .github/workflows/qitops-pr-analysis.yml
```

### Step 2: Explain Key Components

Highlight the key components of the workflow:
- Installation of QitOps Agent
- Configuration of GitHub integration
- Running PR analysis
- Running risk assessment
- Posting results as comments

### Step 3: Show Docker Integration

```bash
# Show Dockerfile
cat Dockerfile

# Show docker-compose.yml
cat docker-compose.yml
```

### Step 4: Explain CI/CD Best Practices

Discuss best practices for using QitOps Agent in CI/CD:
- Storing secrets securely
- Caching for faster CI/CD runs
- Output handling
- Conditional execution

## Conclusion

This demonstration has shown the key features of QitOps Agent:

1. Test case generation
2. Pull request analysis
3. Risk assessment
4. Test data generation
5. LLM provider configuration
6. GitHub integration
7. CI/CD integration

QitOps Agent is highly configurable and can be adapted to various workflows and environments. It helps improve software quality by automating QA tasks and providing valuable insights.
