# QitOps Agent Environment Variables and Configuration Quick Start

This guide will help you quickly set up and use environment variables and configuration files with QitOps Agent.

## Setting Up Sources and Personas

### Step 1: Create Source Files

Create files that contain context for your project:

```bash
# Create a requirements file
mkdir -p docs
echo "# Project Requirements" > docs/requirements.md
echo "- User authentication must be secure" >> docs/requirements.md
echo "- API endpoints must be properly documented" >> docs/requirements.md

# Create a standards file
echo "# Coding Standards" > docs/standards.md
echo "- All code must follow the project's style guide" >> docs/standards.md
echo "- All code must have appropriate error handling" >> docs/standards.md
```

### Step 2: Set Up Environment Variables

```bash
# Windows (PowerShell)
# Set up sources
$env:QITOPS_SOURCE_REQUIREMENTS = "requirements:docs/requirements.md:Project requirements"
$env:QITOPS_SOURCE_STANDARDS = "standard:docs/standards.md:Coding standards"
$env:QITOPS_DEFAULT_SOURCES = "requirements,standards"

# Set up personas
$env:QITOPS_DEFAULT_PERSONAS = "security-analyst,qa-engineer"

# Linux/macOS
# Set up sources
export QITOPS_SOURCE_REQUIREMENTS="requirements:docs/requirements.md:Project requirements"
export QITOPS_SOURCE_STANDARDS="standard:docs/standards.md:Coding standards"
export QITOPS_DEFAULT_SOURCES="requirements,standards"

# Set up personas
export QITOPS_DEFAULT_PERSONAS="security-analyst,qa-engineer"
```

### Step 3: Use QitOps Agent with Environment Variables

```bash
# Generate test cases using default sources and personas
qitops run test-gen --path src/auth.js

# Analyze a PR using default sources and personas
qitops run pr-analyze --pr 123

# Assess risk using default sources and personas
qitops run risk --diff changes.diff
```

## Creating a Configuration File

### Step 1: Create the Configuration Directory

```bash
# Windows (PowerShell)
mkdir -p $env:APPDATA\qitops

# Linux/macOS
mkdir -p ~/.config/qitops
```

### Step 2: Create the Configuration File

```bash
# Windows (PowerShell)
$config = @"
{
  "commands": {
    "test_gen": {
      "default_format": "markdown",
      "default_sources": ["requirements", "standards"],
      "default_personas": ["qa-engineer"]
    },
    "pr_analyze": {
      "default_sources": ["requirements", "standards"],
      "default_personas": ["security-analyst"]
    },
    "risk": {
      "default_sources": ["requirements", "standards"],
      "default_personas": ["security-analyst"]
    }
  },
  "sources": {
    "paths": {
      "requirements": "docs/requirements.md",
      "standards": "docs/standards.md"
    }
  },
  "personas": {
    "default": "qa-engineer"
  }
}
"@
$config | Out-File -FilePath $env:APPDATA\qitops\config.json -Encoding utf8

# Linux/macOS
cat > ~/.config/qitops/config.json << 'EOF'
{
  "commands": {
    "test_gen": {
      "default_format": "markdown",
      "default_sources": ["requirements", "standards"],
      "default_personas": ["qa-engineer"]
    },
    "pr_analyze": {
      "default_sources": ["requirements", "standards"],
      "default_personas": ["security-analyst"]
    },
    "risk": {
      "default_sources": ["requirements", "standards"],
      "default_personas": ["security-analyst"]
    }
  },
  "sources": {
    "paths": {
      "requirements": "docs/requirements.md",
      "standards": "docs/standards.md"
    }
  },
  "personas": {
    "default": "qa-engineer"
  }
}
EOF
```

### Step 3: Use QitOps Agent with Configuration File

```bash
# Generate test cases using default sources and personas from config
qitops run test-gen --path src/auth.js

# Analyze a PR using default sources and personas from config
qitops run pr-analyze --pr 123

# Assess risk using default sources and personas from config
qitops run risk --diff changes.diff
```

## Verifying Configuration

### Check Environment Variables

```bash
# Windows (PowerShell)
$env:QITOPS_DEFAULT_SOURCES
$env:QITOPS_DEFAULT_PERSONAS

# Linux/macOS
echo $QITOPS_DEFAULT_SOURCES
echo $QITOPS_DEFAULT_PERSONAS
```

### Use Verbose Logging

```bash
# Enable verbose logging
qitops -v run test-gen --path src/auth.js
```

This will show detailed information about the sources and personas being used, making it easier to verify your configuration.

## Common Patterns

### Using Different Sources for Different Commands

```bash
# Test generation with requirements and standards
qitops run test-gen --path src/auth.js --sources requirements,standards

# Test data generation with data models
qitops run test-data --schema user-profile.json --sources data-models
```

### Using Different Personas for Different Commands

```bash
# PR analysis with security focus
qitops run pr-analyze --pr 123 --personas security-analyst

# Risk assessment with performance focus
qitops run risk --diff changes.diff --personas performance-engineer
```

### Combining Sources and Personas

```bash
# Test generation with requirements and standards, from a QA perspective
qitops run test-gen --path src/auth.js --sources requirements,standards --personas qa-engineer

# PR analysis with requirements and standards, from a security perspective
qitops run pr-analyze --pr 123 --sources requirements,standards --personas security-analyst
```

## Next Steps

- Read the [Environment Variables and Configuration Reference](ENV_CONFIG_REFERENCE.md) for detailed information
- Check out the [Practical Examples](PRACTICAL_EXAMPLES.md) for more advanced usage
- Learn about [Source Management in CI/CD](SOURCE_MANAGEMENT_CI_CD.md) for CI/CD integration
