use anyhow::Result;
use clap::Subcommand;

// Define the Persona and PersonaManager here
#[derive(Debug, Clone)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub focus_areas: Vec<String>,
    pub description: String,
    pub prompt_template: Option<String>,
}

impl Persona {
    pub fn new(
        id: String,
        name: String,
        focus_areas: Vec<String>,
        description: String,
        prompt_template: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            focus_areas,
            description,
            prompt_template,
        }
    }

    pub fn get_prompt(&self) -> String {
        if let Some(template) = &self.prompt_template {
            template.clone()
        } else {
            format!(
                "You are acting as a {}, focusing on {}. {}\n\nPlease provide your analysis based on this perspective.",
                self.name,
                self.focus_areas.join(", "),
                self.description
            )
        }
    }
}

pub struct PersonaManager {
    personas: std::collections::HashMap<String, Persona>,
}

impl PersonaManager {
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            personas: std::collections::HashMap::new(),
        };

        // Add default personas
        manager.add_persona(Persona::new(
            "qa-engineer".to_string(),
            "QA Engineer".to_string(),
            vec!["testing".to_string(), "quality".to_string(), "coverage".to_string()],
            "Focus on comprehensive test coverage and edge cases.".to_string(),
            None,
        ))?;

        manager.add_persona(Persona::new(
            "security-analyst".to_string(),
            "Security Analyst".to_string(),
            vec!["security".to_string(), "vulnerabilities".to_string(), "compliance".to_string()],
            "Focus on security vulnerabilities and compliance issues.".to_string(),
            None,
        ))?;

        manager.add_persona(Persona::new(
            "performance-engineer".to_string(),
            "Performance Engineer".to_string(),
            vec!["performance".to_string(), "optimization".to_string(), "scalability".to_string()],
            "Focus on performance implications and bottlenecks.".to_string(),
            None,
        ))?;

        Ok(manager)
    }

    pub fn add_persona(&mut self, persona: Persona) -> Result<()> {
        self.personas.insert(persona.id.clone(), persona);
        Ok(())
    }

    pub fn remove_persona(&mut self, id: &str) -> Result<()> {
        self.personas.remove(id);
        Ok(())
    }

    pub fn get_persona(&self, id: &str) -> Option<&Persona> {
        self.personas.get(id)
    }

    pub fn list_personas(&self) -> Vec<&Persona> {
        self.personas.values().collect()
    }

    pub fn get_prompt_for_personas(&self, personas: &[String]) -> Result<String> {
        let mut prompt = String::new();

        for persona_id in personas {
            if let Some(persona) = self.get_persona(persona_id) {
                prompt.push_str(&format!("# Persona: {}\n\n", persona.name));
                prompt.push_str(&persona.get_prompt());
                prompt.push_str("\n\n");
            }
        }

        Ok(prompt)
    }
}
use crate::cli::branding;

/// Persona CLI arguments
#[derive(Debug, clap::Args)]
pub struct PersonaArgs {
    /// Persona subcommand
    #[clap(subcommand)]
    pub command: PersonaCommand,
}

/// Persona subcommands
#[derive(Debug, Subcommand)]
pub enum PersonaCommand {
    /// Add a persona
    #[clap(name = "add")]
    Add {
        /// Persona ID
        #[clap(short, long)]
        id: String,

        /// Persona name
        #[clap(short, long)]
        name: String,

        /// Focus areas (comma-separated)
        #[clap(short, long)]
        focus: String,

        /// Persona description
        #[clap(short, long)]
        description: String,

        /// Prompt template
        #[clap(short, long)]
        template: Option<String>,
    },

    /// List personas
    #[clap(name = "list")]
    List,

    /// Remove a persona
    #[clap(name = "remove")]
    Remove {
        /// Persona ID
        #[clap(short, long)]
        id: String,
    },

    /// Show persona details
    #[clap(name = "show")]
    Show {
        /// Persona ID
        #[clap(short, long)]
        id: String,
    },
}

/// Handle persona commands
pub async fn handle_persona_command(args: &PersonaArgs) -> Result<()> {
    match &args.command {
        PersonaCommand::Add { id, name, focus, description, template } => {
            add_persona(id, name, focus, description, template.clone()).await
        },
        PersonaCommand::List => {
            list_personas().await
        },
        PersonaCommand::Remove { id } => {
            remove_persona(id).await
        },
        PersonaCommand::Show { id } => {
            show_persona(id).await
        },
    }
}

/// Add a persona
async fn add_persona(id: &str, name: &str, focus: &str, description: &str, template: Option<String>) -> Result<()> {
    let mut persona_manager = PersonaManager::new()?;

    let focus_areas = focus.split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let persona = Persona::new(
        id.to_string(),
        name.to_string(),
        focus_areas,
        description.to_string(),
        template,
    );

    persona_manager.add_persona(persona)?;

    branding::print_success(&format!("Persona '{}' added successfully", id));

    Ok(())
}

/// List personas
async fn list_personas() -> Result<()> {
    let persona_manager = PersonaManager::new()?;

    let personas = persona_manager.list_personas();

    if personas.is_empty() {
        println!("No personas found");
        return Ok(());
    }

    println!("Personas:");
    for persona in personas {
        println!("  ID: {}", persona.id);
        println!("    Name: {}", persona.name);
        println!("    Focus Areas: {}", persona.focus_areas.join(", "));
        println!("    Description: {}", persona.description);
        println!();
    }

    Ok(())
}

/// Remove a persona
async fn remove_persona(id: &str) -> Result<()> {
    let mut persona_manager = PersonaManager::new()?;

    persona_manager.remove_persona(id)?;

    branding::print_success(&format!("Persona '{}' removed successfully", id));

    Ok(())
}

/// Show persona details
async fn show_persona(id: &str) -> Result<()> {
    let persona_manager = PersonaManager::new()?;

    let persona = persona_manager.get_persona(id)
        .ok_or_else(|| anyhow::anyhow!("Persona not found: {}", id))?;

    println!("Persona: {}", persona.id);
    println!("Name: {}", persona.name);
    println!("Focus Areas: {}", persona.focus_areas.join(", "));
    println!("Description: {}", persona.description);

    if let Some(template) = &persona.prompt_template {
        println!();
        println!("Prompt Template:");
        println!("{}", template);
    }

    Ok(())
}
