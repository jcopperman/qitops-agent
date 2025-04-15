use qitops_agent::cli::commands::{Cli, Command, RunCommand};
use clap::Parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_help() {
        let cli = Cli::try_parse_from(&["qitops", "--help"]);
        assert!(cli.is_err()); // Help causes error in try_parse
    }

    #[test]
    fn test_cli_parsing_version() {
        let cli = Cli::try_parse_from(&["qitops", "--version"]);
        assert!(cli.is_err()); // Version causes error in try_parse
    }

    #[test]
    fn test_cli_parsing_run_test_gen() {
        let cli = Cli::try_parse_from(&["qitops", "run", "test-gen", "--path", "src/test.rs"]).unwrap();
        
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
        let cli = Cli::try_parse_from(&["qitops", "run", "pr-analyze", "--pr", "123"]).unwrap();
        
        match cli.command {
            Command::Run { command } => {
                match command {
                    RunCommand::PrAnalyze { pr, focus, .. } => {
                        assert_eq!(pr, "123");
                        assert_eq!(focus, "general"); // Default value
                    },
                    _ => panic!("Expected PrAnalyze command"),
                }
            },
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parsing_run_risk() {
        let cli = Cli::try_parse_from(&["qitops", "run", "risk", "--diff", "changes.diff"]).unwrap();
        
        match cli.command {
            Command::Run { command } => {
                match command {
                    RunCommand::Risk { diff, components, focus_areas, .. } => {
                        assert_eq!(diff, "changes.diff");
                        assert!(components.is_empty()); // Default value
                        assert!(focus_areas.is_empty()); // Default value
                    },
                    _ => panic!("Expected Risk command"),
                }
            },
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_cli_parsing_run_test_data() {
        let cli = Cli::try_parse_from(&["qitops", "run", "test-data", "--schema", "user", "--count", "10"]).unwrap();
        
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
        let cli = Cli::try_parse_from(&["qitops", "llm", "list"]).unwrap();
        
        match cli.command {
            Command::Llm(llm_args) => {
                assert_eq!(llm_args.command, "list");
            },
            _ => panic!("Expected Llm command"),
        }
    }

    #[test]
    fn test_cli_parsing_github_test() {
        let cli = Cli::try_parse_from(&["qitops", "github", "test"]).unwrap();
        
        match cli.command {
            Command::GitHub(github_args) => {
                assert_eq!(github_args.command, "test");
            },
            _ => panic!("Expected GitHub command"),
        }
    }
}
