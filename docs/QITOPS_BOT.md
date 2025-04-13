# QitOps Bot

QitOps Bot is an interactive assistant for the QitOps Agent toolchain. It helps users learn and use QitOps Agent effectively through a simple chat interface.

## Features

- **Interactive Chat Interface**: Chat with QitOps Bot directly from the command line
- **Command Assistance**: Get help with QitOps Agent commands and options
- **Knowledge Base**: Access documentation, FAQs, and examples
- **Command Execution**: Execute QitOps Agent commands directly from the chat
- **Contextual Help**: Get help specific to your situation

## Usage

### Starting a Chat Session

```bash
# Start a basic chat session
qitops bot chat

# Start a chat session with a custom system prompt
qitops bot chat --system-prompt custom-prompt.txt

# Start a chat session with a custom knowledge base
qitops bot chat --knowledge-base knowledge-dir
```

### Chat Commands

Once in a chat session, you can use the following commands:

- `exit` or `quit`: End the chat session
- `!exec <command>`: Execute a QitOps Agent command (e.g., `!exec run test-gen --path src/auth.js`)

### Example Conversations

### Getting Help with Commands

```
You: How do I generate test cases?

QitOps Bot: To generate test cases, you can use the `test-gen` command:

qitops run test-gen --path <file_or_directory> [options]

Options:
- --path: Path to the source code (required)
- --format: Output format (markdown, yaml, robot) [default: markdown]
- --sources: Sources to use (comma-separated)
- --personas: Personas to use (comma-separated)

Example:
qitops run test-gen --path src/auth.js

Would you like to know more about any of these options?
```

### Executing Commands

```
You: !exec run test-gen --path src/auth.js --format yaml

QitOps Bot: I executed the command: `run test-gen --path src/auth.js --format yaml`

Result:
```
QitOps Agent v0.1.0 - AI-powered QA Assistant
Developed by QitOps Team

▶ Generating Test Cases

Generated test cases saved to src/tests/test_auth.js.yaml
```
```

### Getting Help with Configuration

```
You: How do I set up environment variables for sources?

QitOps Bot: To set up environment variables for sources, you can use the following format:

For Windows (PowerShell):
$env:QITOPS_SOURCES = "requirements:requirements:docs/requirements.md:Project requirements,standards:standard:docs/standards.md:Coding standards"

For Linux/macOS:
export QITOPS_SOURCES="requirements:requirements:docs/requirements.md:Project requirements,standards:standard:docs/standards.md:Coding standards"

You can also set up individual sources:

For Windows (PowerShell):
$env:QITOPS_SOURCE_REQUIREMENTS = "requirements:docs/requirements.md:Project requirements"

For Linux/macOS:
export QITOPS_SOURCE_REQUIREMENTS="requirements:docs/requirements.md:Project requirements"

Would you like to know more about environment variables for QitOps Agent?
```

### Learning About PR Analysis

```
You: What is PR analysis and how do I use it?

QitOps Bot: PR analysis is a feature of QitOps Agent that analyzes pull requests for potential issues, including:

- Security vulnerabilities
- Performance concerns
- Code quality issues
- Compliance with requirements and standards
- Test coverage gaps

To analyze a PR, you can use the `pr-analyze` command:

qitops run pr-analyze --pr <pr_number_or_url> [options]

Examples:

1. Analyze a PR by number:
   qitops run pr-analyze --pr 123

2. Analyze a PR by URL:
   qitops run pr-analyze --pr https://github.com/username/repo/pull/123

3. Analyze with specific personas:
   qitops run pr-analyze --pr 123 --personas security-analyst

The analysis results will include potential issues, recommendations, and suggested improvements.
```

### Getting Help with Source Management

```
You: How do I manage sources in QitOps?

QitOps Bot: In QitOps Agent, sources provide context for AI-powered generation. You can manage sources using the `source` command:

1. Add a source:
   qitops source add --id requirements --type requirements --path docs/requirements.md --description "Project requirements"

2. List all sources:
   qitops source list

3. Show source content:
   qitops source show --id requirements

4. Remove a source:
   qitops source remove --id requirements

Sources can be used with various commands by specifying the `--sources` option:

qitops run test-gen --path src/auth.js --sources requirements,standards

You can also set default sources using environment variables or configuration files as we discussed earlier.

Would you like to know more about source types or how to use sources effectively?
```

### Troubleshooting Issues

```
You: I'm getting an error "Source not found: requirements" when running test-gen

QitOps Bot: This error occurs when QitOps Agent can't find a source with the ID "requirements". Here are some steps to troubleshoot:

1. Check if the source exists by listing all sources:
   qitops source list

2. If the source doesn't exist, add it:
   qitops source add --id requirements --type requirements --path docs/requirements.md

3. Make sure the source file exists at the specified path:
   ls docs/requirements.md

4. If you're using environment variables, check that they're set correctly:
   echo $QITOPS_SOURCES  # Linux/macOS
   echo $env:QITOPS_SOURCES  # Windows PowerShell

5. Try running the command with verbose output to see more details:
   qitops -v run test-gen --path src/auth.js --sources requirements

Let me know if you need help with any of these steps!
```

### Learning About CI/CD Integration

```
You: How can I integrate QitOps with my CI/CD pipeline?

QitOps Bot: QitOps Agent can be integrated into your CI/CD pipeline to automate quality assurance tasks. Here's how to do it:

1. Install QitOps Agent in your CI/CD environment:
   ```bash
   git clone https://github.com/jcopperman/qitops-agent.git
   cd qitops-agent
   chmod +x install.sh
   ./install.sh
   ```

2. Configure QitOps Agent with environment variables:
   ```bash
   export GITHUB_TOKEN="your-github-token"
   export QITOPS_SOURCES="requirements:requirements:docs/requirements.md:Project requirements"
   export QITOPS_DEFAULT_SOURCES="requirements"
   export QITOPS_DEFAULT_PERSONAS="security-analyst"
   ```

3. Add QitOps Agent commands to your CI/CD workflow.

Here's an example GitHub Actions workflow:

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

Would you like examples for other CI/CD platforms or more details on specific integration scenarios?
```

## Customization

### Custom System Prompt

You can create a custom system prompt file to customize the behavior of QitOps Bot:

```
You are QitOps Bot, an assistant for the QitOps Agent toolchain.
Your purpose is to help users learn and use QitOps Agent effectively.

QitOps Agent is an AI-powered QA Assistant that helps improve software quality through automated analysis, testing, and risk assessment.

Key features of QitOps Agent:
1. Test case generation (qitops run test-gen)
2. Pull request analysis (qitops run pr-analyze)
3. Risk assessment (qitops run risk)
4. Test data generation (qitops run test-data)
5. Interactive testing sessions (qitops run session)

Be helpful, concise, and accurate. If you don't know something, say so.
Provide examples when appropriate.
```

### Custom Knowledge Base

You can create a custom knowledge base to provide QitOps Bot with additional information:

1. Create a directory for the knowledge base
2. Create the following files in the directory:
   - `commands.json`: Command documentation
   - `config.json`: Configuration documentation
   - `faq.json`: Frequently asked questions
   - `examples.json`: Examples of QitOps Agent usage

## Implementation Details

QitOps Bot is implemented as a module in the QitOps Agent codebase:

- `src/bot/mod.rs`: Main bot implementation
- `src/bot/knowledge.rs`: Knowledge base implementation
- `src/cli/bot.rs`: CLI interface for the bot

QitOps Bot uses the same LLM integration infrastructure as the rest of QitOps Agent, so it can use any configured LLM provider (OpenAI, Anthropic, Ollama, etc.).

## Best Practices

### Effective Questioning

- **Be specific**: Ask clear, specific questions to get the most helpful responses
- **Provide context**: Include relevant details about your project or environment
- **One topic at a time**: Focus on one topic per question for clearer answers
- **Follow-up questions**: Ask follow-up questions to dive deeper into a topic

### Command Execution

- **Start with simple commands**: Begin with basic commands before trying complex ones
- **Review before executing**: Check the command syntax before using `!exec`
- **Use verbose mode**: Add `-v` to commands for more detailed output
- **Save outputs**: Copy important command outputs for future reference

### Knowledge Base Utilization

- **Explore commands**: Ask about specific commands to learn their options
- **Learn from examples**: Ask for examples of how to use features
- **Troubleshooting**: Describe errors in detail when seeking help
- **CI/CD integration**: Ask for specific CI/CD platform examples

## Future Improvements

- Web-based chat interface
- Integration with chat platforms (Slack, Discord, etc.)
- More advanced knowledge base with semantic search
- Support for multi-turn conversations with context
- Support for code generation and explanation
- Integration with project-specific documentation
- Personalized learning paths for new users
