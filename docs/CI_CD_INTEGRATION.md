# QitOps Agent CI/CD Integration Guide

This guide explains how to integrate QitOps Agent into your CI/CD pipelines.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation Methods](#installation-methods)
- [GitHub Actions Integration](#github-actions-integration)
- [GitLab CI Integration](#gitlab-ci-integration)
- [Jenkins Integration](#jenkins-integration)
- [CircleCI Integration](#circleci-integration)
- [Docker Integration](#docker-integration)
- [Configuration Options](#configuration-options)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Prerequisites

- Rust toolchain (1.70+)
- GitHub token with appropriate permissions
- LLM API keys (if using OpenAI or Anthropic)

## Installation Methods

### Direct Installation

```bash
git clone https://github.com/jcopperman/qitops-agent.git /tmp/qitops-agent
cd /tmp/qitops-agent
chmod +x install.sh
./install.sh
export PATH="$HOME/.qitops/bin:$PATH"
```

### Docker Installation

```bash
docker pull jcopperman/qitops-agent:latest
docker run -e GITHUB_TOKEN=your_token jcopperman/qitops-agent run pr-analyze --pr 123
```

## GitHub Actions Integration

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
        
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
          
      - name: Install QitOps Agent
        run: |
          git clone https://github.com/jcopperman/qitops-agent.git /tmp/qitops-agent
          cd /tmp/qitops-agent
          chmod +x install.sh
          ./install.sh
          echo "$HOME/.qitops/bin" >> $GITHUB_PATH
          
      - name: Configure QitOps GitHub Integration
        run: |
          qitops github config --token ${{ secrets.GITHUB_TOKEN }} --owner ${{ github.repository_owner }} --repo ${{ github.event.repository.name }}
        
      - name: Analyze PR
        run: |
          qitops run pr-analyze --pr ${{ github.event.pull_request.number }}
```

## GitLab CI Integration

Create a `.gitlab-ci.yml` file:

```yaml
stages:
  - test
  - analyze

variables:
  RUST_VERSION: "1.77"

.qitops_setup: &qitops_setup
  - apt-get update && apt-get install -y git curl
  - git clone https://github.com/jcopperman/qitops-agent.git /tmp/qitops-agent
  - cd /tmp/qitops-agent
  - chmod +x install.sh
  - ./install.sh
  - export PATH="$HOME/.qitops/bin:$PATH"
  - qitops github config --token $GITHUB_TOKEN --owner $CI_PROJECT_NAMESPACE --repo $CI_PROJECT_NAME

pr_analysis:
  stage: analyze
  image: rust:$RUST_VERSION
  script:
    - *qitops_setup
    - qitops run pr-analyze --pr $CI_MERGE_REQUEST_IID
  only:
    - merge_requests
```

## Jenkins Integration

Create a `Jenkinsfile`:

```groovy
pipeline {
    agent {
        docker {
            image 'rust:1.77'
        }
    }
    
    environment {
        GITHUB_TOKEN = credentials('github-token')
        OPENAI_API_KEY = credentials('openai-api-key')
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
        
        stage('PR Analysis') {
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

## CircleCI Integration

Create a `.circleci/config.yml` file:

```yaml
version: 2.1

jobs:
  analyze_pr:
    docker:
      - image: cimg/rust:1.77
    steps:
      - checkout
      - run:
          name: Install QitOps Agent
          command: |
            git clone https://github.com/jcopperman/qitops-agent.git /tmp/qitops-agent
            cd /tmp/qitops-agent
            chmod +x install.sh
            ./install.sh
            echo 'export PATH="$HOME/.qitops/bin:$PATH"' >> $BASH_ENV
            source $BASH_ENV
      - run:
          name: Analyze PR
          command: |
            PR_NUMBER=$(echo $CIRCLE_PULL_REQUEST | grep -o '[0-9]*$')
            if [ -n "$PR_NUMBER" ]; then
              qitops run pr-analyze --pr $PR_NUMBER
            fi
```

## Docker Integration

Create a `Dockerfile`:

```dockerfile
FROM rust:1.77-slim as builder

WORKDIR /usr/src/qitops-agent
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/qitops-agent/target/release/qitops /usr/local/bin/qitops

# Create config directory
RUN mkdir -p /root/.config/qitops

# Set environment variables
ENV GITHUB_TOKEN=""
ENV OPENAI_API_KEY=""

ENTRYPOINT ["qitops"]
CMD ["--help"]
```

## Configuration Options

QitOps Agent can be configured using a JSON configuration file:

```json
{
  "ci": {
    "output_format": "markdown",
    "save_output": true,
    "output_dir": "./qitops-reports",
    "comment_on_pr": true
  },
  "github": {
    "token_env_var": "GITHUB_TOKEN"
  },
  "llm": {
    "default_provider": "openai"
  }
}
```

## Best Practices

1. **Store Secrets Securely**: Use your CI/CD platform's secret management system.
2. **Cache Dependencies**: Cache Rust dependencies to speed up builds.
3. **Limit Scope**: Only run QitOps Agent on relevant events.
4. **Handle Output**: Save and publish QitOps Agent output as artifacts.
5. **Set Timeouts**: Set appropriate timeouts for LLM operations.
6. **Use Conditional Execution**: Skip analysis for certain file types or branches.

## Troubleshooting

### Common Issues

1. **Authentication Errors**: Ensure your GitHub token has the correct permissions.
2. **LLM API Errors**: Check your API keys and rate limits.
3. **Installation Failures**: Ensure Rust is installed correctly.
4. **Path Issues**: Make sure QitOps Agent is in your PATH.

### Debugging

Enable verbose logging with the `-v` flag:

```bash
qitops -v run pr-analyze --pr 123
```

Check the logs for error messages and troubleshooting information.
