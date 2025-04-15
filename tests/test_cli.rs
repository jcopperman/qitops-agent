use qitops_agent::cli::commands::{Cli, Command, RunCommand};
use qitops_agent::cli::llm::LlmCommand;
use qitops_agent::cli::github::GitHubCommand;
use clap::Parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_help() {
        let cli = Cli::try_parse_from(["qitops", "--help"]);
        assert!(cli.is_err()); // Help causes error in try_parse
    }

    #[test]
    fn test_cli_parsing_version() {
        let cli = Cli::try_parse_from(["qitops", "--version"]);
        assert!(cli.is_err()); // Version causes error in try_parse
    }

    #[test]
    fn test_cli_parsing_run_test_gen() {
        let cli = Cli::try_parse_from(["qitops", "run", "test-gen", "--path", "src/test.rs"]).unwrap();

        match cli.command {
            Command::Run { command } => {
                match command {
                    RunCommand::TestGen { path, format, .. } => {
                        assert_eq!(path, "src/test.rs");
                        assert_eq!(format, "markdown"); // Default value
                    },
                    _ => panic!("Expected TestGen command"),
                }
            },
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parsing_run_pr_analyze() {
        let cli = Cli::try_parse_from(["qitops", "run", "pr-analyze", "--pr", "123"]).unwrap();

        match cli.command {
            Command::Run { command } => {
                match command {
                    RunCommand::PrAnalyze { pr, sources, personas, .. } => {
                        assert_eq!(pr, "123");
                        assert!(sources.is_none()); // Default value
                        assert!(personas.is_none()); // Default value
                    },
                    _ => panic!("Expected PrAnalyze command"),
                }
            },
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parsing_run_risk() {
        let cli = Cli::try_parse_from(["qitops", "run", "risk", "--diff", "changes.diff"]).unwrap();

        match cli.command {
            Command::Run { command } => {
                match command {
                    RunCommand::Risk { diff, components, focus, sources, personas, .. } => {
                        assert_eq!(diff, "changes.diff");
                        assert!(components.is_none()); // Default value
                        assert!(focus.is_none()); // Default value
                        assert!(sources.is_none()); // Default value
                        assert!(personas.is_none()); // Default value
                    },
                    _ => panic!("Expected Risk command"),
                }
            },
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parsing_run_test_data() {
        let cli = Cli::try_parse_from(["qitops", "run", "test-data", "--schema", "user", "--count", "10"]).unwrap();

        match cli.command {
            Command::Run { command } => {
                match command {
                    RunCommand::TestData { schema, count, .. } => {
                        assert_eq!(schema, "user");
                        assert_eq!(count, 10);
                    },
                    _ => panic!("Expected TestData command"),
                }
            },
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parsing_llm_list() {
        let cli = Cli::try_parse_from(["qitops", "llm", "list"]).unwrap();

        match cli.command {
            Command::Llm(llm_args) => {
                if let LlmCommand::List = llm_args.command {
                    // Test passed
                } else {
                    panic!("Expected LlmCommand::List");
                }
            },
            _ => panic!("Expected Llm command"),
        }
    }

    #[test]
    fn test_cli_parsing_github_test() {
        let cli = Cli::try_parse_from(["qitops", "github", "test"]).unwrap();

        match cli.command {
            Command::GitHub(github_args) => {
                if let GitHubCommand::Test { .. } = github_args.command {
                    // Test passed
                } else {
                    panic!("Expected GitHubCommand::Test");
                }
            },
            _ => panic!("Expected GitHub command"),
        }
    }
}
