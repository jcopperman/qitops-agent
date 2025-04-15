        pub async fn parse_natural_language_command(&self, message: &str) -> Result<Option<String>> {
            // Check if the message looks like a command request
            let command_indicators = [
                "run", "execute", "start", "generate", "analyze", "test", "create",
                "show", "list", "add", "remove", "set", "config", "help", "check",
                "assess", "evaluate", "find", "search", "get", "make", "build", "setup"
            ];
    
            // Command-specific indicators
            let command_specific = [
                // test-gen indicators
                "test case", "test cases", "unit test", "generate test", "create test",
                // pr-analyze indicators
                "pull request", "pr", "analyze pr", "review pr", "check pr",
                // risk indicators
                "risk", "assess risk", "evaluate risk", "risk assessment",
                // test-data indicators
                "test data", "generate data", "sample data", "mock data",
                // session indicators
                "session", "testing session", "interactive session",
                // llm indicators
                "llm", "language model", "ai model", "model",
                // github indicators
                "github", "git", "repository", "repo",
                // source indicators
                "source", "context source", "knowledge source",
                // persona indicators
                "persona", "role", "perspective"
            ];
    
            // Check for command indicators
            let has_command_indicator = command_indicators.iter().any(|&indicator| {
                message.to_lowercase().contains(&format!(" {} ", indicator)) ||
                message.to_lowercase().starts_with(&format!("{} ", indicator)) ||
                message.to_lowercase().contains(&format!("{} ", indicator))
            });
    
            // Check for command-specific indicators
            let has_specific_indicator = command_specific.iter().any(|&indicator| {
                message.to_lowercase().contains(indicator)
            });
    
            // If no indicators are found, it's probably not a command request
            if !has_command_indicator && !has_specific_indicator {
                return Ok(None);
            }
    
            // Create a detailed prompt for the LLM to parse the natural language request
            let prompt = format!(
                "Convert the following natural language request into a QitOps Agent command.\n\n\
                Request: {}\n\n\
                Respond with ONLY the command, without any explanation or markdown formatting.\n\
                If you're not sure, respond with 'UNKNOWN'.\n\n\
                Available commands and their purposes:\n\
                1. Test Generation:\n\
                   - qitops run test-gen --path <file_path> [--format <format>] [--sources <sources>] [--personas <personas>]\n\
                   - Purpose: Generate test cases for source code files\n\
                   - Example inputs: 'Generate tests for auth.js', 'Create unit tests for the user module'\n\
                2. PR Analysis:\n\
                   - qitops run pr-analyze --pr <pr_number> [--sources <sources>] [--personas <personas>]\n\
                   - Purpose: Analyze pull requests for quality, risks, and test coverage\n\
                   - Example inputs: 'Analyze PR 123', 'Review pull request #456'\n\
                3. Risk Assessment:\n\
                   - qitops run risk --diff <diff_path> [--components <components>] [--focus <focus_areas>]\n\
                   - Purpose: Assess risk of code changes\n\
                   - Example inputs: 'Assess risk for changes.diff', 'Evaluate risk in the payment module'\n\
                4. Test Data Generation:\n\
                   - qitops run test-data --schema <schema> --count <count> [--format <format>]\n\
                   - Purpose: Generate test data based on a schema\n\
                   - Example inputs: 'Generate 10 user profiles', 'Create 50 sample transactions'\n\
                5. Testing Session:\n\
                   - qitops run session --name <name> [--application <app>] [--focus <focus>]\n\
                   - Purpose: Start an interactive testing session\n\
                   - Example inputs: 'Start a testing session for login flow', 'Begin a test session for the API'\n\
                6. LLM Management:\n\
                   - qitops llm list\n\
                   - qitops llm add --provider <provider> --api-key <api_key> [--api-base <api_base>] [--model <model>]\n\
                   - qitops llm remove --provider <provider>\n\
                   - qitops llm set-default --provider <provider>\n\
                   - qitops llm test [--provider <provider>] [--prompt <prompt>] [--no-cache]\n\
                   - Purpose: Manage LLM providers and settings\n\
                   - Example inputs: 'List available LLMs', 'Set OpenAI as default provider'\n\
                7. GitHub Integration:\n\
                   - qitops github config --token <token> [--owner <owner>] [--repo <repo>]\n\
                   - Purpose: Configure GitHub integration\n\
                   - Example inputs: 'Setup GitHub integration', 'Configure GitHub with my token'\n\
                8. Source Management:\n\
                   - qitops source list\n\
                   - qitops source show --id <id>\n\
                   - Purpose: Manage context sources\n\
                   - Example inputs: 'Show available sources', 'Display source requirements'\n\
                9. Persona Management:\n\
                   - qitops persona list\n\
                   - qitops persona show --id <id>\n\
                   - Purpose: Manage personas for context\n\
                   - Example inputs: 'List available personas', 'Show the QA engineer persona'\n\
                Guidelines for parsing:\n\
                - For file paths, use the exact path mentioned or a reasonable default if not specified\n\
                - For PR numbers, extract the number from the request\n\
                - For formats, default to 'markdown' unless another format is specified\n\
                - For counts, use the number mentioned or a reasonable default (e.g., 10)\n\
                - For names, use the exact name mentioned or a reasonable default based on the context\n\
                - If multiple commands could apply, choose the most specific one\n\
                - If essential parameters are missing, make a reasonable guess based on the context\
                ",
                message
            );
    
            // Send the request to the LLM
            let model = self.llm_router.default_model().unwrap_or_else(|| "mistral".to_string());
            let request = LlmRequest::new(prompt, model)
                .with_system_message("You are a command parser for QitOps Agent. Your task is to convert natural language requests into valid QitOps Agent commands. Be precise and follow the format exactly. Only return the command itself without any explanation.".to_string());
    
            let llm_response = self.llm_router.send(request, None).await?;
            let command = llm_response.text.trim();
    
            // Check if the LLM couldn't parse the command
            if command == "UNKNOWN" || command.contains("I'm not sure") || command.contains("I don't know") {
                return Ok(None);
            }
    
            // Remove any markdown formatting
            let command = command.trim_start_matches("```").trim_end_matches("```").trim();
            let command = command.trim_start_matches("bash").trim();
            let command = command.trim_start_matches("qitops ").trim();
    
            // Log the parsed command for debugging
            tracing::debug!("Parsed command: {}", command);
    
            Ok(Some(command.to_string()))
        }
