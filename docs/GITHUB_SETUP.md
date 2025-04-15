# GitHub Integration Setup Guide

This guide explains how to set up GitHub integration for QitOps Agent.

## Creating a GitHub Personal Access Token

1. **Go to GitHub Settings**:
   - Click on your profile picture in the top right corner
   - Select "Settings"

2. **Access Developer Settings**:
   - Scroll down to the bottom of the sidebar
   - Click on "Developer settings"

3. **Generate a Personal Access Token**:
   - Click on "Personal access tokens"
   - Click on "Tokens (classic)" or "Fine-grained tokens" (recommended)
   - Click "Generate new token"

4. **Configure Token Permissions**:
   - Give your token a descriptive name (e.g., "QitOps Agent")
   - Set an expiration date (recommended: 90 days)
   - Select the following permissions:
     - For classic tokens: `repo`, `read:user`, `read:org`
     - For fine-grained tokens:
       - Repository access: Select repositories you want to analyze
       - Permissions:
         - Repository: `Contents` (Read-only)
         - Pull requests: `Read-only`
         - Issues: `Read-only`

5. **Generate Token**:
   - Click "Generate token"
   - **IMPORTANT**: Copy the token immediately! You won't be able to see it again.

## Configuring QitOps Agent with Your GitHub Token

### Option 1: Using the CLI Command

```bash
qitops github config --token YOUR_GITHUB_TOKEN
```

This will securely store your token in the QitOps configuration.

### Option 2: Using Environment Variables

Set the `GITHUB_TOKEN` environment variable:

**Windows (Command Prompt)**:
```
set GITHUB_TOKEN=your_github_token
```

**Windows (PowerShell)**:
```
$env:GITHUB_TOKEN = "your_github_token"
```

**Linux/macOS**:
```
export GITHUB_TOKEN=your_github_token
```

### Option 3: Setting Default Repository

If you frequently work with the same repository, you can set it as the default:

```bash
qitops github config --owner OWNER_NAME --repo REPOSITORY_NAME
```

For example:
```bash
qitops github config --owner jcopperman --repo qitops-agent
```

## Verifying GitHub Configuration

To verify that your GitHub token is configured correctly:

```bash
qitops github status
```

This will show your current GitHub configuration without revealing the full token.

## Testing GitHub Integration

To test that your GitHub integration is working:

```bash
qitops run pr-analyze --pr 123
```

Replace `123` with an actual PR number from your repository.

## Troubleshooting

### Token Not Found

If you see the error `GitHub token not found in config or GITHUB_TOKEN environment variable`:

1. Make sure you've configured the token using one of the methods above
2. Check that the token hasn't expired
3. Try setting the token again

### Permission Denied

If you see permission errors when accessing GitHub:

1. Check that your token has the correct permissions
2. Verify that the token is valid
3. Make sure you have access to the repository you're trying to analyze

### Repository Not Found

If you see `Repository not found` errors:

1. Check that you're using the correct owner and repository names
2. Verify that your token has access to the repository
3. Make sure the repository exists and is accessible to your account

## Security Considerations

- **Never** commit your GitHub token to version control
- Use the shortest expiration time that's practical for your workflow
- Consider using fine-grained tokens with minimal permissions
- Rotate your tokens regularly
- If you suspect your token has been compromised, revoke it immediately in GitHub settings
