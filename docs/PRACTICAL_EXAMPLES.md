# QitOps Agent Practical Examples

This document provides practical examples of how to use QitOps Agent with environment variables and configuration files.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Using Environment Variables](#using-environment-variables)
- [Using Configuration Files](#using-configuration-files)
- [CI/CD Examples](#cicd-examples)
- [Advanced Scenarios](#advanced-scenarios)

## Basic Usage

### Generating Test Cases

```bash
# Generate test cases for a file
qitops run test-gen --path src/auth.js

# Generate test cases with specific format
qitops run test-gen --path src/auth.js --format yaml
```

### Analyzing Pull Requests

```bash
# Analyze a PR by number
qitops run pr-analyze --pr 123

# Analyze a PR by URL
qitops run pr-analyze --pr https://github.com/username/repo/pull/123
```

### Assessing Risk

```bash
# Assess risk from a PR
qitops run risk --diff https://github.com/username/repo/pull/123

# Assess risk from a local diff file
qitops run risk --diff changes.diff
```

### Generating Test Data

```bash
# Generate test data based on a schema
qitops run test-data --schema user-profile.json --count 10
```

## Using Environment Variables

### Setting Up Sources with Environment Variables

```bash
# Windows (PowerShell)
# Define multiple sources in a single environment variable
$env:QITOPS_SOURCES = "requirements:requirements:docs/requirements.md:Project requirements,standards:standard:docs/standards.md:Coding standards"

# Define individual sources
$env:QITOPS_SOURCE_REQUIREMENTS = "requirements:docs/requirements.md:Project requirements"
$env:QITOPS_SOURCE_STANDARDS = "standard:docs/standards.md:Coding standards"

# Set default sources
$env:QITOPS_DEFAULT_SOURCES = "requirements,standards"

# Linux/macOS
# Define multiple sources in a single environment variable
export QITOPS_SOURCES="requirements:requirements:docs/requirements.md:Project requirements,standards:standard:docs/standards.md:Coding standards"

# Define individual sources
export QITOPS_SOURCE_REQUIREMENTS="requirements:docs/requirements.md:Project requirements"
export QITOPS_SOURCE_STANDARDS="standard:docs/standards.md:Coding standards"

# Set default sources
export QITOPS_DEFAULT_SOURCES="requirements,standards"
```

### Setting Up Personas with Environment Variables

```bash
# Windows (PowerShell)
# Define multiple personas in a single environment variable
$env:QITOPS_PERSONAS = "security-analyst:Security Analyst:security;vulnerabilities;compliance:Focus on security vulnerabilities and compliance issues."

# Define individual personas
$env:QITOPS_PERSONA_SECURITY = "Security Analyst:security;vulnerabilities;compliance:Focus on security vulnerabilities and compliance issues."
$env:QITOPS_PERSONA_QA = "QA Engineer:testing;coverage;regression:Focus on comprehensive test coverage and regression testing."

# Set default personas
$env:QITOPS_DEFAULT_PERSONAS = "security-analyst,qa-engineer"

# Linux/macOS
# Define multiple personas in a single environment variable
export QITOPS_PERSONAS="security-analyst:Security Analyst:security;vulnerabilities;compliance:Focus on security vulnerabilities and compliance issues."

# Define individual personas
export QITOPS_PERSONA_SECURITY="Security Analyst:security;vulnerabilities;compliance:Focus on security vulnerabilities and compliance issues."
export QITOPS_PERSONA_QA="QA Engineer:testing;coverage;regression:Focus on comprehensive test coverage and regression testing."

# Set default personas
export QITOPS_DEFAULT_PERSONAS="security-analyst,qa-engineer"
```

### Using Environment Variables with Commands

```bash
# Windows (PowerShell)
$env:QITOPS_DEFAULT_SOURCES = "requirements,standards"
$env:QITOPS_DEFAULT_PERSONAS = "security-analyst"

# Generate test cases using default sources and personas
qitops run test-gen --path src/auth.js

# Linux/macOS
export QITOPS_DEFAULT_SOURCES="requirements,standards"
export QITOPS_DEFAULT_PERSONAS="security-analyst"

# Generate test cases using default sources and personas
qitops run test-gen --path src/auth.js
```

## Using Configuration Files

### Creating a Configuration File

Create a file at:
- Windows: `%APPDATA%\qitops\config.json`
- Linux/macOS: `~/.config/qitops/config.json`

```json
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
    },
    "test_data": {
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

### Using the Configuration File

```bash
# Generate test cases using default sources and personas from config
qitops run test-gen --path src/auth.js

# Analyze a PR using default sources and personas from config
qitops run pr-analyze --pr 123

# Assess risk using default sources and personas from config
qitops run risk --diff changes.diff

# Generate test data using default sources and personas from config
qitops run test-data --schema user-profile.json
```

## CI/CD Examples

### GitHub Actions

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
          
      - name: Set up sources and personas
        run: |
          # Set up sources
          export QITOPS_SOURCE_REQUIREMENTS="requirements:docs/requirements.md:Project requirements"
          export QITOPS_SOURCE_STANDARDS="standard:docs/standards.md:Coding standards"
          export QITOPS_DEFAULT_SOURCES="requirements,standards"
          
          # Set up personas
          export QITOPS_DEFAULT_PERSONAS="security-analyst"
          
      - name: Analyze PR
        run: |
          qitops run pr-analyze --pr ${{ github.event.pull_request.number }}
```

### GitLab CI

```yaml
stages:
  - analyze

variables:
  QITOPS_SOURCE_REQUIREMENTS: "requirements:docs/requirements.md:Project requirements"
  QITOPS_SOURCE_STANDARDS: "standard:docs/standards.md:Coding standards"
  QITOPS_DEFAULT_SOURCES: "requirements,standards"
  QITOPS_DEFAULT_PERSONAS: "security-analyst"

analyze-pr:
  stage: analyze
  image: rust:1.77
  script:
    - apt-get update && apt-get install -y git curl
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

### Jenkins Pipeline

```groovy
pipeline {
    agent {
        docker {
            image 'rust:1.77'
        }
    }
    
    environment {
        GITHUB_TOKEN = credentials('github-token')
        QITOPS_SOURCE_REQUIREMENTS = "requirements:docs/requirements.md:Project requirements"
        QITOPS_SOURCE_STANDARDS = "standard:docs/standards.md:Coding standards"
        QITOPS_DEFAULT_SOURCES = "requirements,standards"
        QITOPS_DEFAULT_PERSONAS = "security-analyst"
    }
    
    stages {
        stage('Setup') {
            steps {
                sh '''
                    git clone https://github.com/jcopperman/qitops-agent.git /tmp/qitops-agent
                    cd /tmp/qitops-agent
                    chmod +x install.sh
                    ./install.sh
                    export PATH="$HOME/.qitops/bin:$PATH"
                '''
            }
        }
        
        stage('Analyze PR') {
            when {
                expression { env.CHANGE_ID != null }
            }
            steps {
                sh '''
                    export PATH="$HOME/.qitops/bin:$PATH"
                    qitops run pr-analyze --pr $CHANGE_ID
                '''
            }
        }
    }
}
```

## Advanced Scenarios

### Combining Command-Line Arguments with Environment Variables

```bash
# Set default sources and personas
export QITOPS_DEFAULT_SOURCES="requirements,standards"
export QITOPS_DEFAULT_PERSONAS="security-analyst"

# Override default personas for a specific command
qitops run risk --diff changes.diff --personas performance-engineer
```

### Using Different Sources and Personas for Different Commands

```bash
# Set up environment variables
export QITOPS_SOURCE_REQUIREMENTS="requirements:docs/requirements.md:Project requirements"
export QITOPS_SOURCE_STANDARDS="standard:docs/standards.md:Coding standards"
export QITOPS_SOURCE_DATA_MODELS="documentation:docs/data-models.json:Data models documentation"

# Use different sources for different commands
qitops run test-gen --path src/auth.js --sources requirements,standards
qitops run test-data --schema user-profile.json --sources data-models
```

### Using Verbose Logging to Debug Configuration

```bash
# Enable verbose logging
qitops -v run test-gen --path src/auth.js
```

This will show detailed information about the sources and personas being used, making it easier to debug configuration issues.
