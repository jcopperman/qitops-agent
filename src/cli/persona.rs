use anyhow::Result;
use clap::Subcommand;

use crate::persona::{PersonaManager, Persona};
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
