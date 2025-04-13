# Source Management in CI/CD Environments

This guide explains how to manage sources in CI/CD environments for QitOps Agent.

## Table of Contents

- [Introduction](#introduction)
- [Source Types](#source-types)
- [Source Management Strategies](#source-management-strategies)
- [Environment Variables](#environment-variables)
- [Configuration Files](#configuration-files)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Introduction

Sources provide context for QitOps Agent's analysis, making it more relevant and accurate. In CI/CD environments, managing sources can be challenging, especially when the source files are not part of the repository.

This guide explains different strategies for managing sources in CI/CD environments.

## Source Types

QitOps Agent supports the following source types:

- **Requirements**: Project requirements and specifications
- **Standard**: Coding standards and guidelines
- **TestStrategy**: Test strategies and approaches
- **BugHistory**: History of bugs and issues
- **Documentation**: Project documentation
- **Custom**: Any other type of source

## Source Management Strategies

There are several strategies for managing sources in CI/CD environments:

### 1. Repository-Based Sources

Store source files in the repository and reference them in CI/CD pipelines.

**Pros**:
- Sources are version-controlled
- Sources are always available
- Sources can be reviewed and updated with the code

**Cons**:
- May clutter the repository
- May expose sensitive information

### 2. External Sources

Store source files in an external location (e.g., S3, Google Drive) and download them during CI/CD runs.

**Pros**:
- Keeps the repository clean
- Can be shared across multiple repositories
- Can be updated independently of the code

**Cons**:
- Requires additional setup
- May introduce dependencies on external services

### 3. Generated Sources

Generate source files during CI/CD runs based on templates or other sources.

**Pros**:
- Sources are always up-to-date
- Can be customized for each run
- No need to store source files

**Cons**:
- May be complex to set up
- May introduce inconsistencies

### 4. Environment Variable-Based Sources

Define sources using environment variables in CI/CD pipelines.

**Pros**:
- No need for source files
- Easy to configure in CI/CD pipelines
- Can be updated without changing the code

**Cons**:
- Limited to small sources
- May be difficult to maintain for complex sources

## Environment Variables

QitOps Agent supports the following environment variables for source management:

### QITOPS_SOURCES

Defines multiple sources in a single environment variable.

Format: `id1:type1:path1[:description1],id2:type2:path2[:description2]`

Example:
```bash
export QITOPS_SOURCES="requirements:requirements:docs/requirements.md:Project requirements,standards:standard:docs/standards.md:Coding standards"
```

### QITOPS_SOURCE_<ID>

Defines a single source with the specified ID.

Format: `type:path[:description]`

Example:
```bash
export QITOPS_SOURCE_REQUIREMENTS="requirements:docs/requirements.md:Project requirements"
export QITOPS_SOURCE_STANDARDS="standard:docs/standards.md:Coding standards"
```

### QITOPS_DEFAULT_SOURCES

Defines the default sources to use when no sources are specified.

Format: `id1,id2,id3`

Example:
```bash
export QITOPS_DEFAULT_SOURCES="requirements,standards"
```

## Configuration Files

QitOps Agent can also be configured using a JSON configuration file. In CI/CD environments, you can create this file during the pipeline run.

Example configuration file:
```json
{
  "commands": {
    "pr_analyze": {
      "default_sources": ["requirements", "standards"]
    },
    "risk": {
      "default_sources": ["requirements", "standards"]
    }
  },
  "sources": {
    "paths": {
      "requirements": "docs/requirements.md",
      "standards": "docs/standards.md"
    }
  }
}
```

## Examples

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
          
      - name: Configure Sources
        run: |
          # Add sources if they exist in the repository
          if [ -f "docs/requirements.md" ]; then
            qitops source add --id requirements --type requirements --path docs/requirements.md
          fi
          
          if [ -f "docs/standards.md" ]; then
            qitops source add --id standards --type standard --path docs/standards.md
          fi
          
          # Download external sources if needed
          if [ ! -f "docs/requirements.md" ]; then
            mkdir -p docs
            curl -o docs/requirements.md ${{ secrets.REQUIREMENTS_URL }}
            qitops source add --id requirements --type requirements --path docs/requirements.md
          fi
          
      - name: Analyze PR
        run: |
          qitops run pr-analyze --pr ${{ github.event.pull_request.number }} --sources requirements,standards
```

### GitLab CI

```yaml
stages:
  - analyze

variables:
  QITOPS_DEFAULT_SOURCES: "requirements,standards"

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
    
    # Configure sources
    - |
      if [ -f "docs/requirements.md" ]; then
        qitops source add --id requirements --type requirements --path docs/requirements.md
      else
        mkdir -p docs
        curl -o docs/requirements.md $REQUIREMENTS_URL
        qitops source add --id requirements --type requirements --path docs/requirements.md
      fi
      
      if [ -f "docs/standards.md" ]; then
        qitops source add --id standards --type standard --path docs/standards.md
      else
        mkdir -p docs
        curl -o docs/standards.md $STANDARDS_URL
        qitops source add --id standards --type standard --path docs/standards.md
      fi
    
    # Analyze PR
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
        QITOPS_DEFAULT_SOURCES = "requirements,standards"
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
        
        stage('Configure') {
            steps {
                sh '''
                    export PATH="$HOME/.qitops/bin:$PATH"
                    qitops github config --token $GITHUB_TOKEN
                    
                    # Configure sources
                    if [ -f "docs/requirements.md" ]; then
                        qitops source add --id requirements --type requirements --path docs/requirements.md
                    else
                        mkdir -p docs
                        curl -o docs/requirements.md $REQUIREMENTS_URL
                        qitops source add --id requirements --type requirements --path docs/requirements.md
                    fi
                    
                    if [ -f "docs/standards.md" ]; then
                        qitops source add --id standards --type standard --path docs/standards.md
                    else
                        mkdir -p docs
                        curl -o docs/standards.md $STANDARDS_URL
                        qitops source add --id standards --type standard --path docs/standards.md
                    fi
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

## Best Practices

1. **Version Control Sources**: When possible, store source files in version control to ensure consistency and traceability.

2. **Use Environment Variables for Sensitive Information**: If sources contain sensitive information, use environment variables to define them.

3. **Cache Sources**: If downloading sources from external locations, consider caching them to improve performance.

4. **Use Default Sources**: Configure default sources in the configuration file or environment variables to simplify command invocation.

5. **Validate Sources**: Before using sources, validate that they exist and are accessible.

6. **Document Sources**: Document the sources used in CI/CD pipelines to help others understand the context.

7. **Keep Sources Up-to-Date**: Regularly update sources to ensure they reflect the current state of the project.

## Troubleshooting

### Common Issues

1. **Source Not Found**: If a source is not found, check that the path is correct and the file exists.

2. **Invalid Source Type**: If a source type is invalid, check that it is one of the supported types.

3. **Environment Variables Not Set**: If environment variables are not set, check that they are defined in the CI/CD pipeline.

4. **Configuration File Not Found**: If the configuration file is not found, check that it is created during the CI/CD pipeline.

### Debugging

Enable verbose logging to debug source management issues:

```bash
qitops --verbose run pr-analyze --pr 123
```

This will show detailed information about the sources being used.
