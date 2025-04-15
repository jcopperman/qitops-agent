use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

/// Tutorial step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    /// Step title
    pub title: String,
    /// Step content
    pub content: String,
    /// Example command (optional)
    pub example: Option<String>,
    /// Expected user action (optional)
    pub expected_action: Option<String>,
}

/// Tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    /// Tutorial ID
    pub id: String,
    /// Tutorial title
    pub title: String,
    /// Tutorial description
    pub description: String,
    /// Tutorial steps
    pub steps: Vec<TutorialStep>,
    /// Tutorial tags
    pub tags: Vec<String>,
    /// Tutorial difficulty level (beginner, intermediate, advanced)
    pub difficulty: String,
    /// Estimated time to complete (in minutes)
    pub estimated_time: u32,
}

/// Tutorial manager
#[derive(Debug)]
pub struct TutorialManager {
    /// Available tutorials
    tutorials: HashMap<String, Tutorial>,
    /// Tutorial directory
    tutorial_dir: PathBuf,
}

impl TutorialManager {
    /// Create a new tutorial manager
    pub fn new(tutorial_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            tutorials: HashMap::new(),
            tutorial_dir,
        };
        
        manager.load_tutorials()?;
        
        Ok(manager)
    }
    
    /// Load tutorials from the tutorial directory
    pub fn load_tutorials(&mut self) -> Result<()> {
        // Create the tutorial directory if it doesn't exist
        if !self.tutorial_dir.exists() {
            fs::create_dir_all(&self.tutorial_dir)?;
            
            // Create default tutorials
            self.create_default_tutorials()?;
        }
        
        // Load tutorials from the directory
        for entry in fs::read_dir(&self.tutorial_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let tutorial_json = fs::read_to_string(&path)?;
                let tutorial: Tutorial = serde_json::from_str(&tutorial_json)?;
                
                self.tutorials.insert(tutorial.id.clone(), tutorial);
            }
        }
        
        Ok(())
    }
    
    /// Create default tutorials
    fn create_default_tutorials(&self) -> Result<()> {
        // Create onboarding tutorial
        let onboarding = Tutorial {
            id: "onboarding".to_string(),
            title: "Getting Started with QitOps".to_string(),
            description: "Learn the basics of QitOps Agent and how to use it effectively.".to_string(),
            steps: vec![
                TutorialStep {
                    title: "Welcome to QitOps".to_string(),
                    content: "QitOps is an AI-powered QA Assistant that helps you generate test cases, analyze pull requests, assess risk, and more. This tutorial will guide you through the basics.".to_string(),
                    example: None,
                    expected_action: None,
                },
                TutorialStep {
                    title: "Understanding Commands".to_string(),
                    content: "QitOps Agent provides several commands that you can use. You can execute these commands directly or use natural language to describe what you want to do.".to_string(),
                    example: None,
                    expected_action: None,
                },
                TutorialStep {
                    title: "Generating Test Cases".to_string(),
                    content: "One of the most powerful features of QitOps is the ability to generate test cases for your code. Let's try generating test cases for a file.".to_string(),
                    example: Some("qitops run test-gen --path src/main.rs".to_string()),
                    expected_action: Some("Try generating test cases for a file using the example command or by saying 'Generate test cases for src/main.rs'".to_string()),
                },
                TutorialStep {
                    title: "Analyzing Pull Requests".to_string(),
                    content: "QitOps can analyze pull requests to identify potential issues, assess test coverage, and provide recommendations.".to_string(),
                    example: Some("qitops run pr-analyze --pr 123".to_string()),
                    expected_action: Some("Try analyzing a pull request using the example command or by saying 'Analyze pull request 123'".to_string()),
                },
                TutorialStep {
                    title: "Assessing Risk".to_string(),
                    content: "QitOps can assess the risk of code changes to help you prioritize testing efforts.".to_string(),
                    example: Some("qitops run risk --diff changes.diff".to_string()),
                    expected_action: Some("Try assessing risk using the example command or by saying 'Assess risk for changes.diff'".to_string()),
                },
                TutorialStep {
                    title: "Using the Bot".to_string(),
                    content: "The QitOps Bot provides a conversational interface to QitOps Agent. You can use commands like !help, !exec, and !history to interact with the bot.".to_string(),
                    example: Some("!help".to_string()),
                    expected_action: Some("Try using the !help command to see available bot commands".to_string()),
                },
                TutorialStep {
                    title: "Congratulations!".to_string(),
                    content: "You've completed the onboarding tutorial! You now know the basics of QitOps Agent and how to use it effectively. Type !tutorials to see more tutorials.".to_string(),
                    example: None,
                    expected_action: None,
                },
            ],
            tags: vec!["beginner".to_string(), "onboarding".to_string()],
            difficulty: "beginner".to_string(),
            estimated_time: 10,
        };
        
        // Create test generation tutorial
        let test_gen = Tutorial {
            id: "test-gen-workflow".to_string(),
            title: "Test Generation Workflow".to_string(),
            description: "Learn how to generate comprehensive test cases for your code using QitOps Agent.".to_string(),
            steps: vec![
                TutorialStep {
                    title: "Introduction to Test Generation".to_string(),
                    content: "QitOps Agent can generate comprehensive test cases for your code based on the code itself, requirements, and best practices.".to_string(),
                    example: None,
                    expected_action: None,
                },
                TutorialStep {
                    title: "Preparing Your Code".to_string(),
                    content: "Before generating test cases, make sure your code is well-documented with comments explaining the purpose and behavior of functions and classes.".to_string(),
                    example: None,
                    expected_action: None,
                },
                TutorialStep {
                    title: "Basic Test Generation".to_string(),
                    content: "Let's start with basic test generation for a file. The test-gen command requires a path to the file you want to generate tests for.".to_string(),
                    example: Some("qitops run test-gen --path src/main.rs".to_string()),
                    expected_action: Some("Try generating test cases for a file using the example command".to_string()),
                },
                TutorialStep {
                    title: "Customizing Test Format".to_string(),
                    content: "You can customize the format of the generated tests using the --format option. Supported formats include markdown, yaml, and robot.".to_string(),
                    example: Some("qitops run test-gen --path src/main.rs --format yaml".to_string()),
                    expected_action: Some("Try generating test cases in YAML format".to_string()),
                },
                TutorialStep {
                    title: "Using Sources".to_string(),
                    content: "You can specify sources to use for test generation using the --sources option. Sources provide additional context for test generation.".to_string(),
                    example: Some("qitops run test-gen --path src/main.rs --sources requirements,standards".to_string()),
                    expected_action: Some("Try generating test cases with specific sources".to_string()),
                },
                TutorialStep {
                    title: "Using Personas".to_string(),
                    content: "You can specify personas to use for test generation using the --personas option. Personas provide different perspectives on testing.".to_string(),
                    example: Some("qitops run test-gen --path src/main.rs --personas qa-engineer".to_string()),
                    expected_action: Some("Try generating test cases with a specific persona".to_string()),
                },
                TutorialStep {
                    title: "Reviewing Generated Tests".to_string(),
                    content: "After generating tests, review them to ensure they cover all important scenarios and edge cases. You may need to modify or add tests based on your specific requirements.".to_string(),
                    example: None,
                    expected_action: None,
                },
                TutorialStep {
                    title: "Saving and Implementing Tests".to_string(),
                    content: "Save the generated tests to a file and implement them in your test framework. QitOps generates test cases in a format that can be easily adapted to your testing framework.".to_string(),
                    example: None,
                    expected_action: None,
                },
                TutorialStep {
                    title: "Congratulations!".to_string(),
                    content: "You've completed the test generation workflow tutorial! You now know how to generate comprehensive test cases for your code using QitOps Agent.".to_string(),
                    example: None,
                    expected_action: None,
                },
            ],
            tags: vec!["testing".to_string(), "workflow".to_string()],
            difficulty: "beginner".to_string(),
            estimated_time: 15,
        };
        
        // Create PR analysis tutorial
        let pr_analysis = Tutorial {
            id: "pr-analysis-workflow".to_string(),
            title: "PR Analysis Workflow".to_string(),
            description: "Learn how to analyze pull requests for quality, risks, and test coverage using QitOps Agent.".to_string(),
            steps: vec![
                TutorialStep {
                    title: "Introduction to PR Analysis".to_string(),
                    content: "QitOps Agent can analyze pull requests to identify potential issues, assess test coverage, and provide recommendations.".to_string(),
                    example: None,
                    expected_action: None,
                },
                TutorialStep {
                    title: "Setting Up GitHub Integration".to_string(),
                    content: "Before analyzing pull requests, you need to set up GitHub integration. This allows QitOps to access your GitHub repositories.".to_string(),
                    example: Some("qitops github config --token YOUR_GITHUB_TOKEN".to_string()),
                    expected_action: Some("Set up GitHub integration using your GitHub token".to_string()),
                },
                TutorialStep {
                    title: "Basic PR Analysis".to_string(),
                    content: "Let's start with basic PR analysis. The pr-analyze command requires a PR number.".to_string(),
                    example: Some("qitops run pr-analyze --pr 123".to_string()),
                    expected_action: Some("Try analyzing a pull request using the example command".to_string()),
                },
                TutorialStep {
                    title: "Using Sources".to_string(),
                    content: "You can specify sources to use for PR analysis using the --sources option. Sources provide additional context for analysis.".to_string(),
                    example: Some("qitops run pr-analyze --pr 123 --sources requirements,standards".to_string()),
                    expected_action: Some("Try analyzing a pull request with specific sources".to_string()),
                },
                TutorialStep {
                    title: "Using Personas".to_string(),
                    content: "You can specify personas to use for PR analysis using the --personas option. Personas provide different perspectives on analysis.".to_string(),
                    example: Some("qitops run pr-analyze --pr 123 --personas qa-engineer".to_string()),
                    expected_action: Some("Try analyzing a pull request with a specific persona".to_string()),
                },
                TutorialStep {
                    title: "Reviewing Analysis Results".to_string(),
                    content: "After analyzing a pull request, review the results to identify potential issues and areas for improvement. The analysis includes code quality, test coverage, and potential risks.".to_string(),
                    example: None,
                    expected_action: None,
                },
                TutorialStep {
                    title: "Taking Action".to_string(),
                    content: "Based on the analysis results, take appropriate action to address any issues or concerns. This may include adding tests, refactoring code, or requesting changes from the PR author.".to_string(),
                    example: None,
                    expected_action: None,
                },
                TutorialStep {
                    title: "Congratulations!".to_string(),
                    content: "You've completed the PR analysis workflow tutorial! You now know how to analyze pull requests for quality, risks, and test coverage using QitOps Agent.".to_string(),
                    example: None,
                    expected_action: None,
                },
            ],
            tags: vec!["pr".to_string(), "workflow".to_string()],
            difficulty: "intermediate".to_string(),
            estimated_time: 20,
        };
        
        // Save tutorials to files
        let onboarding_path = self.tutorial_dir.join("onboarding.json");
        let test_gen_path = self.tutorial_dir.join("test-gen-workflow.json");
        let pr_analysis_path = self.tutorial_dir.join("pr-analysis-workflow.json");
        
        fs::write(onboarding_path, serde_json::to_string_pretty(&onboarding)?)?;
        fs::write(test_gen_path, serde_json::to_string_pretty(&test_gen)?)?;
        fs::write(pr_analysis_path, serde_json::to_string_pretty(&pr_analysis)?)?;
        
        Ok(())
    }
    
    /// Get a tutorial by ID
    pub fn get_tutorial(&self, id: &str) -> Option<&Tutorial> {
        self.tutorials.get(id)
    }
    
    /// Get all tutorials
    pub fn get_all_tutorials(&self) -> Vec<&Tutorial> {
        self.tutorials.values().collect()
    }
    
    /// Get tutorials by tag
    pub fn get_tutorials_by_tag(&self, tag: &str) -> Vec<&Tutorial> {
        self.tutorials.values()
            .filter(|t| t.tags.contains(&tag.to_string()))
            .collect()
    }
    
    /// Get tutorials by difficulty
    pub fn get_tutorials_by_difficulty(&self, difficulty: &str) -> Vec<&Tutorial> {
        self.tutorials.values()
            .filter(|t| t.difficulty == difficulty)
            .collect()
    }
    
    /// Format tutorial list as a string
    pub fn format_tutorial_list(&self, tutorials: Vec<&Tutorial>) -> String {
        let mut result = String::new();
        
        for (i, tutorial) in tutorials.iter().enumerate() {
            result.push_str(&format!("{}. {} ({})\n", i + 1, tutorial.title, tutorial.id));
            result.push_str(&format!("   {}\n", tutorial.description));
            result.push_str(&format!("   Difficulty: {} | Time: {} minutes | Tags: {}\n\n", 
                tutorial.difficulty, 
                tutorial.estimated_time,
                tutorial.tags.join(", ")));
        }
        
        result
    }
}

/// Tutorial session
#[derive(Debug)]
pub struct TutorialSession {
    /// Tutorial
    pub tutorial: Tutorial,
    /// Current step index
    pub current_step: usize,
    /// Session start time
    pub start_time: std::time::Instant,
    /// Completed steps
    pub completed_steps: Vec<usize>,
}

impl TutorialSession {
    /// Create a new tutorial session
    pub fn new(tutorial: Tutorial) -> Self {
        Self {
            tutorial,
            current_step: 0,
            start_time: std::time::Instant::now(),
            completed_steps: Vec::new(),
        }
    }
    
    /// Get the current step
    pub fn current_step(&self) -> Option<&TutorialStep> {
        self.tutorial.steps.get(self.current_step)
    }
    
    /// Move to the next step
    pub fn next_step(&mut self) -> Option<&TutorialStep> {
        if !self.completed_steps.contains(&self.current_step) {
            self.completed_steps.push(self.current_step);
        }
        
        self.current_step += 1;
        
        if self.current_step >= self.tutorial.steps.len() {
            self.current_step = self.tutorial.steps.len() - 1;
            return None;
        }
        
        self.current_step()
    }
    
    /// Move to the previous step
    pub fn previous_step(&mut self) -> Option<&TutorialStep> {
        if self.current_step > 0 {
            self.current_step -= 1;
        }
        
        self.current_step()
    }
    
    /// Check if the tutorial is completed
    pub fn is_completed(&self) -> bool {
        self.completed_steps.len() >= self.tutorial.steps.len()
    }
    
    /// Get the progress percentage
    pub fn progress_percentage(&self) -> f32 {
        let total_steps = self.tutorial.steps.len();
        let completed_steps = self.completed_steps.len();
        
        if total_steps == 0 {
            return 100.0;
        }
        
        (completed_steps as f32 / total_steps as f32) * 100.0
    }
    
    /// Format the current step as a string
    pub fn format_current_step(&self) -> String {
        let step = match self.current_step() {
            Some(step) => step,
            None => return "No current step".to_string(),
        };
        
        let mut result = String::new();
        
        result.push_str(&format!("Step {} of {}: {}\n\n", 
            self.current_step + 1, 
            self.tutorial.steps.len(),
            step.title));
        
        result.push_str(&format!("{}\n\n", step.content));
        
        if let Some(example) = &step.example {
            result.push_str(&format!("Example: {}\n\n", example));
        }
        
        if let Some(expected_action) = &step.expected_action {
            result.push_str(&format!("Action: {}\n\n", expected_action));
        }
        
        result.push_str(&format!("Progress: {:.1}%\n", self.progress_percentage()));
        result.push_str("Type !next to continue, !prev to go back, or !exit-tutorial to exit the tutorial.\n");
        
        result
    }
}
