# Contributing to QitOps Agent

Thank you for your interest in contributing to QitOps Agent! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Reporting Bugs

If you find a bug in the project, please create an issue on GitHub with the following information:

1. A clear, descriptive title
2. A detailed description of the issue
3. Steps to reproduce the bug
4. Expected behavior
5. Actual behavior
6. Screenshots (if applicable)
7. Environment information (OS, Rust version, etc.)

### Suggesting Enhancements

If you have an idea for an enhancement, please create an issue on GitHub with the following information:

1. A clear, descriptive title
2. A detailed description of the enhancement
3. The motivation behind the enhancement
4. Any potential implementation details

### Pull Requests

1. Fork the repository
2. Create a new branch for your feature or bug fix: `git checkout -b feature/your-feature-name` or `git checkout -b fix/your-bug-fix`
3. Make your changes
4. Run tests to ensure your changes don't break existing functionality: `cargo test`
5. Format your code: `cargo fmt`
6. Run linting: `cargo clippy`
7. Commit your changes with a descriptive message
8. Push to your fork: `git push origin feature/your-feature-name`
9. Submit a pull request to the `main` branch

## Development Setup

### Prerequisites

- Rust (latest stable version)
- Git
- An LLM provider (Ollama, OpenAI, or Anthropic)

### Setting Up the Development Environment

1. Clone the repository:
   ```bash
   git clone https://github.com/jcopperman/qitops-agent.git
   cd qitops-agent
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Run the application:
   ```bash
   cargo run -- --help
   ```

### Setting Up LLM Providers

#### Ollama (Recommended for Development)

1. Install Ollama from [ollama.ai](https://ollama.ai/download)
2. Pull a model: `ollama pull mistral`
3. Configure QitOps to use Ollama:
   ```bash
   cargo run -- llm add --provider ollama --api-base http://localhost:11434
   cargo run -- llm default --provider ollama
   ```

#### OpenAI

1. Get an API key from [OpenAI](https://platform.openai.com/)
2. Configure QitOps to use OpenAI:
   ```bash
   cargo run -- llm add --provider openai --api-key YOUR_API_KEY --model gpt-4
   cargo run -- llm default --provider openai
   ```

## Project Structure

- `src/` - Source code
  - `agent/` - Agent implementations (test-gen, pr-analyze, risk, test-data)
  - `llm/` - LLM client and provider implementations
  - `ci/` - CI/CD integration
  - `cli/` - Command-line interface
  - `config/` - Configuration management
  - `bot/` - QitOps Bot implementation
  - `source/` - Source management
  - `persona/` - Persona management
  - `update/` - Update checking mechanism
- `tests/` - Test files
- `docs/` - Documentation
- `.github/` - GitHub workflows and templates

## Coding Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format your code
- Use `cargo clippy` to catch common mistakes and improve your code
- Follow the naming conventions:
  - `snake_case` for variables, functions, and modules
  - `CamelCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

### Documentation

- Document all public functions, types, and modules
- Use Rust's documentation comments (`///` for functions and `//!` for modules)
- Include examples where appropriate
- Explain the purpose, parameters, and return values

### Error Handling

- Use `anyhow::Result` for functions that can fail
- Provide context for errors using `.context()`
- Handle errors gracefully and provide user-friendly error messages
- Use `thiserror` for defining custom error types

### General Guidelines

- Write clear, descriptive commit messages
- Add tests for new features
- Update documentation for changes
- Use meaningful variable and function names
- Keep functions small and focused
- Add comments for complex logic

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test --package qitops-agent --lib agent::test_gen

# Run a specific test
cargo test test_llm_request_creation

# Run tests with verbose output
cargo test -- --nocapture
```

### Writing Tests

- Place tests in a `tests` module within the same file or in a separate file in the `tests/` directory
- Use the `#[test]` attribute for test functions
- Use `assert!`, `assert_eq!`, and `assert_ne!` for assertions
- Use `#[should_panic]` for tests that should panic

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        let result = my_function();
        assert_eq!(result, expected_value);
    }

    #[test]
    #[should_panic]
    fn test_invalid_input() {
        my_function_with_invalid_input();
    }
}
```

### Test Guidelines

- Write unit tests for new functionality
- Ensure all tests pass before submitting a pull request
- Add integration tests for new features
- Test edge cases and error conditions
- Use mocks for external dependencies

## Documentation

- Update the README.md file for user-facing changes
- Add inline documentation for new functions and types
- Update the user guide for new features or changes to existing features

## Release Process

1. Update the version number in Cargo.toml
2. Update the CHANGELOG.md file with the changes since the last release
3. Create a pull request for the release
4. Once approved, merge the pull request
5. Create a new tag for the release: `git tag v0.1.0`
6. Push the tag: `git push origin v0.1.0`
7. Create a new release on GitHub with release notes
8. Publish to crates.io: `cargo publish`

## Getting Help

If you need help with contributing, please:

1. Check the documentation
2. Look for similar issues on GitHub
3. Ask for help in the issue you're working on
4. Join our community Discord server (coming soon)

Thank you for contributing to QitOps Agent!
