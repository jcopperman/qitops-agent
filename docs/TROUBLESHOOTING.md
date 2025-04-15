# QitOps Agent Troubleshooting Guide

This guide helps you troubleshoot common issues with QitOps Agent.

## Common Issues and Solutions

### File Access Permission Errors

**Error Message**: `Permission denied when reading file` or `Access is denied`

**Causes**:
- Running QitOps without sufficient permissions
- File is locked by another process
- File permissions are restrictive

**Solutions**:
1. **Run with Administrator Privileges**:
   - Windows: Right-click on Command Prompt or PowerShell and select "Run as administrator"
   - Linux/macOS: Use `sudo` before the command

2. **Check File Permissions**:
   - Windows: Right-click on the file, select Properties, and check the Security tab
   - Linux/macOS: Use `ls -l` to check permissions and `chmod` to modify them

3. **Use a Different File Path**:
   - Try using a file in a directory where you have full permissions
   - Avoid system directories or protected locations

4. **Close Other Applications**:
   - Make sure the file isn't open in another application

### GitHub Token Configuration Issues

**Error Message**: `GitHub token not found in config or GITHUB_TOKEN environment variable`

**Causes**:
- GitHub token not configured
- Token has expired or been revoked

**Solutions**:
1. **Configure GitHub Token**:
   ```bash
   qitops github config --token YOUR_GITHUB_TOKEN
   ```

2. **Set Environment Variable**:
   - Windows:
     ```
     set GITHUB_TOKEN=your_github_token
     ```
   - Linux/macOS:
     ```
     export GITHUB_TOKEN=your_github_token
     ```

3. **Verify Token Status**:
   - Check if your token is still valid in GitHub settings
   - Create a new token if necessary

4. **Check Configuration Status**:
   ```bash
   qitops github status
   ```

### LLM Configuration Issues

**Error Message**: `Failed to initialize LLM router` or `No LLM providers configured`

**Causes**:
- No LLM provider configured
- API key is invalid or missing
- Connection to LLM provider failed

**Solutions**:
1. **Configure OpenAI**:
   ```bash
   qitops llm add --provider openai --api-key YOUR_API_KEY
   ```

2. **Configure Ollama (Local)**:
   ```bash
   qitops llm add --provider ollama --api-base http://localhost:11434
   ```

3. **Set Default Provider**:
   ```bash
   qitops llm default --provider ollama
   ```

4. **Check LLM Status**:
   ```bash
   qitops llm list
   ```

### File Not Found Errors

**Error Message**: `File not found` or `Path is a directory, not a file`

**Causes**:
- File doesn't exist at the specified path
- Path is a directory, not a file
- Path contains typos or incorrect formatting

**Solutions**:
1. **Check File Existence**:
   - Verify that the file exists at the specified path
   - Use absolute paths instead of relative paths

2. **Check Path Format**:
   - Use forward slashes (`/`) even on Windows
   - Enclose paths with spaces in quotes

3. **Specify Correct Path Type**:
   - For directories, make sure to use the appropriate command
   - For files, make sure the file exists and is readable

## Advanced Troubleshooting

### Verbose Mode

Run QitOps with the `--verbose` flag to get more detailed error information:

```bash
qitops --verbose run test-gen --path src/test.rs
```

### Check Logs

QitOps logs are stored in:
- Windows: `%APPDATA%\qitops\logs`
- Linux/macOS: `~/.config/qitops/logs`

### Reset Configuration

If you're experiencing persistent issues, you can reset the configuration:

1. Delete the configuration directory:
   - Windows: `%APPDATA%\qitops`
   - Linux/macOS: `~/.config/qitops`

2. Reconfigure QitOps:
   ```bash
   qitops llm add --provider ollama --api-base http://localhost:11434
   qitops github config --token YOUR_GITHUB_TOKEN
   ```

## Getting Help

If you're still experiencing issues, please:

1. Check the [GitHub repository](https://github.com/jcopperman/qitops-agent) for known issues
2. Open a new issue with:
   - Detailed description of the problem
   - Steps to reproduce
   - Error messages
   - System information (OS, Rust version, etc.)
