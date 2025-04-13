# QitOps Bot Quick Start Guide

This guide will help you quickly get started with QitOps Bot, the interactive assistant for QitOps Agent.

## Installation

QitOps Bot is included with QitOps Agent. If you have QitOps Agent installed, you already have QitOps Bot.

If you haven't installed QitOps Agent yet, follow these steps:

```bash
# Clone the repository
git clone https://github.com/jcopperman/qitops-agent.git

# Navigate to the repository directory
cd qitops-agent

# Run the installation script
# On Linux/macOS
chmod +x install.sh
./install.sh

# On Windows (PowerShell)
.\install.ps1
```

## Starting QitOps Bot

To start a chat session with QitOps Bot, use the following command:

```bash
qitops bot chat
```

This will start an interactive chat session in your terminal.

## Basic Usage

### Asking Questions

You can ask QitOps Bot questions about QitOps Agent:

```
You: How do I generate test cases?
```

QitOps Bot will respond with information about the `test-gen` command, including its options and examples.

### Executing Commands

You can execute QitOps Agent commands directly from the chat using the `!exec` prefix:

```
You: !exec run test-gen --path src/auth.js
```

QitOps Bot will execute the command and show you the results.

### Ending the Session

To end the chat session, type `exit` or `quit`:

```
You: exit
```

## Advanced Usage

### Custom System Prompt

You can start QitOps Bot with a custom system prompt:

```bash
qitops bot chat --system-prompt custom-prompt.txt
```

Where `custom-prompt.txt` is a file containing your custom system prompt.

### Custom Knowledge Base

You can start QitOps Bot with a custom knowledge base:

```bash
qitops bot chat --knowledge-base knowledge-dir
```

Where `knowledge-dir` is a directory containing your custom knowledge base files.

## Example Workflow

Here's an example workflow using QitOps Bot:

1. Start QitOps Bot:
   ```bash
   qitops bot chat
   ```

2. Learn about PR analysis:
   ```
   You: What is PR analysis and how do I use it?
   ```

3. Configure GitHub integration:
   ```
   You: !exec github config --token YOUR_GITHUB_TOKEN --owner username --repo repository
   ```

4. Analyze a PR:
   ```
   You: !exec run pr-analyze --pr 123
   ```

5. Get help with troubleshooting:
   ```
   You: I'm getting an error when analyzing PRs. How can I debug it?
   ```

6. End the session:
   ```
   You: exit
   ```

## Tips for Effective Use

- Be specific in your questions
- Provide context when asking about errors
- Use `!exec` to try out commands
- Ask follow-up questions to dive deeper
- Use the `-v` flag with commands for verbose output

## Next Steps

- Read the [full QitOps Bot documentation](QITOPS_BOT.md)
- Explore the [QitOps Agent User Guide](USER_GUIDE.md)
- Check out the [Configuration Guide](CONFIGURATION.md) to customize QitOps Agent
