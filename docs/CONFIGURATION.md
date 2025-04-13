# QitOps Agent Configuration Guide

This guide explains how to configure QitOps Agent for your specific needs.

## Configuration Methods

QitOps Agent can be configured using:

1. **Command-line arguments**: For one-time settings
2. **Configuration files**: For persistent settings
3. **Environment variables**: For sensitive information

## Configuration Files

QitOps Agent stores its configuration in:
- Windows: `%APPDATA%\qitops\config.json`
- Linux/macOS: `~/.config/qitops/config.json`

GitHub configuration is stored in:
- Windows: `%APPDATA%\qitops\github.json`
- Linux/macOS: `~/.config/qitops/github.json`

## LLM Configuration

### Available LLM Providers

QitOps Agent supports the following LLM providers:

1. **OpenAI**: Cloud-based LLM (requires API key)
2. **Anthropic**: Cloud-based LLM (requires API key)
3. **Ollama**: Local LLM (no API key required)

### Configuring LLM Providers

#### Using Command Line

```bash
# Add OpenAI provider
qitops llm add --provider openai --api-key YOUR_API_KEY --model gpt-4

# Add Anthropic provider
qitops llm add --provider anthropic --api-key YOUR_API_KEY --model claude-3-opus-20240229

# Add Ollama provider
qitops llm add --provider ollama --api-base http://localhost:11434 --model mistral

# Set default provider
qitops llm default --provider ollama
```

#### Using Configuration File

You can manually edit the configuration file:

```json
{
  "providers": [
    {
      "provider_type": "openai",
      "default_model": "gpt-4",
      "api_key": "YOUR_API_KEY"
    },
    {
      "provider_type": "anthropic",
      "default_model": "claude-3-opus-20240229",
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
    "performance": "anthropic",
    "test-gen": "ollama"
  }
}
```

#### Using Environment Variables

You can set environment variables for sensitive information and configuration:

```bash
# Windows (PowerShell)
# LLM Configuration
$env:OPENAI_API_KEY = "your-api-key"
$env:ANTHROPIC_API_KEY = "your-api-key"
$env:OLLAMA_API_BASE = "http://localhost:11434"

# GitHub Configuration
$env:GITHUB_TOKEN = "your-github-token"

# Sources Configuration
$env:QITOPS_SOURCES = "requirements:requirements:docs/requirements.md:Project requirements,standards:standard:docs/standards.md:Coding standards"
$env:QITOPS_SOURCE_DATA_MODELS = "documentation:docs/data-models.json:Data models documentation"
$env:QITOPS_DEFAULT_SOURCES = "requirements,standards"

# Personas Configuration
$env:QITOPS_PERSONAS = "security-analyst:Security Analyst:security;vulnerabilities;compliance:Focus on security vulnerabilities and compliance issues."
$env:QITOPS_PERSONA_COMPLIANCE = "Compliance Officer:compliance;regulations;standards:Focus on regulatory compliance and standards adherence."
$env:QITOPS_DEFAULT_PERSONAS = "security-analyst"

# Linux/macOS
# LLM Configuration
export OPENAI_API_KEY="your-api-key"
export ANTHROPIC_API_KEY="your-api-key"
export OLLAMA_API_BASE="http://localhost:11434"

# GitHub Configuration
export GITHUB_TOKEN="your-github-token"

# Sources Configuration
export QITOPS_SOURCES="requirements:requirements:docs/requirements.md:Project requirements,standards:standard:docs/standards.md:Coding standards"
export QITOPS_SOURCE_DATA_MODELS="documentation:docs/data-models.json:Data models documentation"
export QITOPS_DEFAULT_SOURCES="requirements,standards"

# Personas Configuration
export QITOPS_PERSONAS="security-analyst:Security Analyst:security;vulnerabilities;compliance:Focus on security vulnerabilities and compliance issues."
export QITOPS_PERSONA_COMPLIANCE="Compliance Officer:compliance;regulations;standards:Focus on regulatory compliance and standards adherence."
export QITOPS_DEFAULT_PERSONAS="security-analyst"
```

##### Environment Variable Formats

**QITOPS_SOURCES**
Defines multiple sources in a single environment variable.
Format: `id1:type1:path1[:description1],id2:type2:path2[:description2]`

**QITOPS_SOURCE_<ID>**
Defines a single source with the specified ID.
Format: `type:path[:description]`

**QITOPS_DEFAULT_SOURCES**
Defines the default sources to use when no sources are specified.
Format: `id1,id2,id3`

**QITOPS_PERSONAS**
Defines multiple personas in a single environment variable.
Format: `id1:name1:focus1;focus2:description1[:prompt_template1],id2:name2:focus1;focus2:description2[:prompt_template2]`

**QITOPS_PERSONA_<ID>**
Defines a single persona with the specified ID.
Format: `name:focus1;focus2:description[:prompt_template]`

**QITOPS_DEFAULT_PERSONAS**
Defines the default personas to use when no personas are specified.
Format: `id1,id2,id3`

### Task-Specific LLM Providers

You can assign different LLM providers to specific tasks:

```bash
# Use OpenAI for security analysis
qitops llm task --task security --provider openai

# Use Anthropic for performance analysis
qitops llm task --task performance --provider anthropic

# Use Ollama for test generation
qitops llm task --task test-gen --provider ollama
```

## GitHub Configuration

### Configuring GitHub Integration

#### Using Command Line

```bash
# Configure GitHub token
qitops github config --token YOUR_GITHUB_TOKEN

# Configure default repository
qitops github config --owner username --repo repository

# Configure GitHub Enterprise API base URL
qitops github config --api-base https://github.example.com/api/v3
```

#### Using Environment Variables

```bash
# Windows (PowerShell)
$env:GITHUB_TOKEN = "your-github-token"

# Linux/macOS
export GITHUB_TOKEN="your-github-token"
```

## Command-Specific Configuration

### Test Generation Configuration

```bash
# Set default format
qitops run test-gen --path src/module.rs --format yaml

# Set coverage level
qitops run test-gen --path src/module.rs --coverage high

# Focus on specific component
qitops run test-gen --path src/module.rs --component authentication
```

### PR Analysis Configuration

```bash
# Focus on security
qitops run pr-analyze --pr 123 --focus security

# Focus on multiple areas
qitops run pr-analyze --pr 123 --focus "security,performance,maintainability"
```

### Risk Assessment Configuration

```bash
# Focus on specific components
qitops run risk --diff changes.diff --components "auth,payment"

# Focus on specific areas
qitops run risk --diff changes.diff --focus "security,performance"
```

### Test Data Generation Configuration

```bash
# Set number of records
qitops run test-data --schema user-profile --count 20

# Set output format
qitops run test-data --schema user-profile --format csv

# Set constraints
qitops run test-data --schema user-profile --constraints "age>18,country=US"
```

## Advanced Configuration

### Creating a Custom Configuration File

You can create a custom configuration file and use it with QitOps Agent:

```json
{
  "llm": {
    "default_provider": "openai",
    "providers": {
      "openai": {
        "api_key": "YOUR_API_KEY",
        "default_model": "gpt-4"
      },
      "ollama": {
        "api_base": "http://localhost:11434",
        "default_model": "mistral"
      }
    },
    "task_providers": {
      "security": "openai",
      "performance": "ollama",
      "test-gen": "ollama"
    }
  },
  "github": {
    "token": "YOUR_GITHUB_TOKEN",
    "default_owner": "username",
    "default_repo": "repository"
  },
  "commands": {
    "test_gen": {
      "default_format": "markdown",
      "default_coverage": "high",
      "default_sources": ["requirements", "standards"],
      "default_personas": ["qa-engineer"]
    },
    "pr_analyze": {
      "default_focus": ["security", "performance"],
      "default_sources": ["requirements", "standards"],
      "default_personas": ["security-analyst"]
    },
    "risk": {
      "default_components": ["auth", "payment"],
      "default_focus": ["security"],
      "default_sources": ["requirements", "standards"],
      "default_personas": ["security-analyst"]
    },
    "test_data": {
      "default_count": 10,
      "default_format": "json",
      "default_sources": ["data-models"],
      "default_personas": ["qa-engineer"]
    }
  },
  "sources": {
    "default": "requirements",
    "paths": {
      "requirements": "docs/requirements.md",
      "standards": "docs/standards.md",
      "data-models": "docs/data-models.json"
    }
  },
  "personas": {
    "default": "qa-engineer"
  }
}
```

#### Configuration File Structure

**commands**
Command-specific configuration, including default sources and personas for each command.

**sources**
Source configuration, including default source and paths to source files.

**personas**
Persona configuration, including default persona.

### Configuration Precedence

QitOps Agent uses the following precedence order for configuration:

1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration files
4. Default values (lowest priority)

## Configuration Examples

### Example 1: Development Environment

```json
{
  "llm": {
    "default_provider": "ollama",
    "providers": {
      "ollama": {
        "api_base": "http://localhost:11434",
        "default_model": "mistral"
      }
    }
  },
  "github": {
    "token": "YOUR_GITHUB_TOKEN",
    "default_owner": "username",
    "default_repo": "repository"
  }
}
```

### Example 2: CI/CD Environment

```json
{
  "llm": {
    "default_provider": "openai",
    "providers": {
      "openai": {
        "api_key": "YOUR_API_KEY",
        "default_model": "gpt-4"
      }
    }
  },
  "github": {
    "token": "YOUR_GITHUB_TOKEN"
  },
  "ci": {
    "output_format": "markdown",
    "save_output": true,
    "output_dir": "./qitops-reports",
    "comment_on_pr": true
  }
}
```

### Example 3: Enterprise Environment

```json
{
  "llm": {
    "default_provider": "anthropic",
    "providers": {
      "anthropic": {
        "api_key": "YOUR_API_KEY",
        "default_model": "claude-3-opus-20240229"
      }
    }
  },
  "github": {
    "token": "YOUR_GITHUB_TOKEN",
    "api_base": "https://github.example.com/api/v3"
  },
  "commands": {
    "risk": {
      "fail_on_high_risk": true,
      "risk_threshold": "medium"
    }
  }
}
```

## Troubleshooting Configuration Issues

### Common Configuration Issues

1. **Invalid API Keys**:
   - Check if your API keys are correct
   - Verify that your API keys have the necessary permissions

2. **GitHub Token Issues**:
   - Ensure your GitHub token has the correct scopes
   - Check if your token has expired

3. **Ollama Connection Issues**:
   - Verify that Ollama is running
   - Check if the API base URL is correct

### Debugging Configuration

Enable verbose logging to debug configuration issues:

```bash
qitops --verbose llm list
qitops --verbose github status
```

This will show detailed information about the configuration being used.
