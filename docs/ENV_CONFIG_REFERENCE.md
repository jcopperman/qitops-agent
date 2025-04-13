# QitOps Agent Environment Variables and Configuration Reference

This document provides a quick reference for environment variables and configuration options in QitOps Agent.

## Environment Variables

### LLM Configuration

| Environment Variable | Description | Example |
|----------------------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key | `export OPENAI_API_KEY="sk-..."` |
| `ANTHROPIC_API_KEY` | Anthropic API key | `export ANTHROPIC_API_KEY="sk-ant-..."` |
| `OLLAMA_API_BASE` | Ollama API base URL | `export OLLAMA_API_BASE="http://localhost:11434"` |

### GitHub Configuration

| Environment Variable | Description | Example |
|----------------------|-------------|---------|
| `GITHUB_TOKEN` | GitHub API token | `export GITHUB_TOKEN="ghp_..."` |

### Sources Configuration

| Environment Variable | Description | Format | Example |
|----------------------|-------------|--------|---------|
| `QITOPS_SOURCES` | Multiple sources | `id1:type1:path1[:description1],id2:type2:path2[:description2]` | `export QITOPS_SOURCES="requirements:requirements:docs/requirements.md:Project requirements,standards:standard:docs/standards.md:Coding standards"` |
| `QITOPS_SOURCE_<ID>` | Single source | `type:path[:description]` | `export QITOPS_SOURCE_REQUIREMENTS="requirements:docs/requirements.md:Project requirements"` |
| `QITOPS_DEFAULT_SOURCES` | Default sources | `id1,id2,id3` | `export QITOPS_DEFAULT_SOURCES="requirements,standards"` |

### Personas Configuration

| Environment Variable | Description | Format | Example |
|----------------------|-------------|--------|---------|
| `QITOPS_PERSONAS` | Multiple personas | `id1:name1:focus1;focus2:description1[:prompt_template1],id2:name2:focus1;focus2:description2[:prompt_template2]` | `export QITOPS_PERSONAS="security-analyst:Security Analyst:security;vulnerabilities;compliance:Focus on security vulnerabilities and compliance issues."` |
| `QITOPS_PERSONA_<ID>` | Single persona | `name:focus1;focus2:description[:prompt_template]` | `export QITOPS_PERSONA_SECURITY="Security Analyst:security;vulnerabilities;compliance:Focus on security vulnerabilities and compliance issues."` |
| `QITOPS_DEFAULT_PERSONAS` | Default personas | `id1,id2,id3` | `export QITOPS_DEFAULT_PERSONAS="security-analyst"` |

## Configuration File

The configuration file is stored at:
- Windows: `%APPDATA%\qitops\config.json`
- Linux/macOS: `~/.config/qitops/config.json`

### Example Configuration

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

## Command-Line Usage with Sources and Personas

### Test Generation

```bash
# Using command-line arguments
qitops run test-gen --path src/module.rs --sources requirements,standards --personas qa-engineer

# Using default sources and personas from configuration
qitops run test-gen --path src/module.rs
```

### PR Analysis

```bash
# Using command-line arguments
qitops run pr-analyze --pr 123 --sources requirements,standards --personas security-analyst

# Using default sources and personas from configuration
qitops run pr-analyze --pr 123
```

### Risk Assessment

```bash
# Using command-line arguments
qitops run risk --diff changes.diff --sources requirements,standards --personas security-analyst

# Using default sources and personas from configuration
qitops run risk --diff changes.diff
```

### Test Data Generation

```bash
# Using command-line arguments
qitops run test-data --schema user-profile --sources data-models --personas qa-engineer

# Using default sources and personas from configuration
qitops run test-data --schema user-profile
```

## Source Management

```bash
# Add a source
qitops source add --id requirements --type requirements --path docs/requirements.md --description "Project requirements"

# List sources
qitops source list

# Show source content
qitops source show --id requirements

# Remove a source
qitops source remove --id requirements
```

## Persona Management

```bash
# Add a persona
qitops persona add --id security-analyst --name "Security Analyst" --focus "security,vulnerabilities,compliance" --description "Focus on security vulnerabilities and compliance issues."

# List personas
qitops persona list

# Show persona details
qitops persona show --id security-analyst

# Remove a persona
qitops persona remove --id security-analyst
```

## Configuration Precedence

QitOps Agent uses the following precedence order for configuration:

1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration files
4. Default values (lowest priority)
