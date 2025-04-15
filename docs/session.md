# QitOps Agent - Interactive Testing Session

The QitOps Agent provides an interactive testing session feature that allows testers to have a conversation with an AI assistant to guide them through exploratory testing sessions.

## Overview

The interactive testing session feature is designed to help testers:

- Plan and structure their exploratory testing sessions
- Get guidance on what to test and how to test it
- Document their testing activities and findings
- Collaborate with developers and other stakeholders

## Usage

To start an interactive testing session, use the following command:

```bash
qitops run session --name "Session Name" --application "Application Name" [options]
```

### Required Parameters

- `--name`: The name of the testing session
- `--application`: The name of the application being tested

### Optional Parameters

- `--session-type`: The type of testing session (exploratory, regression, user-journey, performance, security)
- `--objectives`: Comma-separated list of testing objectives
- `--sources`: Comma-separated list of information sources to consider
- `--personas`: Comma-separated list of user personas to consider

## Session Types

The following session types are supported:

- **Exploratory**: Free-form testing to discover issues and learn about the application
- **Regression**: Testing to verify that previously working functionality still works correctly
- **User Journey**: Testing complete user journeys and end-to-end flows
- **Performance**: Testing the performance and scalability of the application
- **Security**: Testing for security vulnerabilities and compliance issues

## Example

```bash
qitops run session --name "Login Feature Test" --application "MyApp" --session-type exploratory --objectives "verify login flow, test error handling" --sources "documentation,code" --personas "new user,admin"
```

## Session History

After each session, a markdown file is created in the `sessions` directory with the session history. This file includes all the messages exchanged between the user and the QitOps Agent during the session.

The session history file is named using the session name, with spaces and special characters replaced by underscores.

Example: `sessions/Login_Feature_Test_session.md`

## Tips for Effective Sessions

1. **Be specific**: Provide clear objectives and context for your testing session
2. **Ask questions**: The QitOps Agent can provide guidance and suggestions
3. **Document findings**: Share your observations and findings during the session
4. **Use personas**: Consider different user perspectives by specifying personas
5. **Reference sources**: Include relevant documentation and code as sources

## Limitations

- The QitOps Agent does not have direct access to the application being tested
- The quality of guidance depends on the information provided by the user
- The session is text-based and cannot directly interact with the application
