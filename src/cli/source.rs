use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

use crate::source::{SourceManager, Source, SourceType};
use crate::cli::branding;

/// Source CLI arguments
#[derive(Debug, clap::Args)]
pub struct SourceArgs {
    /// Source subcommand
    #[clap(subcommand)]
    pub command: SourceCommand,
}

/// Source subcommands
#[derive(Debug, Subcommand)]
pub enum SourceCommand {
    /// Add a source
    #[clap(name = "add")]
    Add {
        /// Source ID
        #[clap(short, long)]
        id: String,
        
        /// Source type (requirements, standard, test-strategy, bug-history, documentation, or custom)
        #[clap(short, long)]
        type_: String,
        
        /// Source path
        #[clap(short, long)]
        path: String,
        
        /// Source description
        #[clap(short, long)]
        description: Option<String>,
    },
    
    /// List sources
    #[clap(name = "list")]
    List,
    
    /// Remove a source
    #[clap(name = "remove")]
    Remove {
        /// Source ID
        #[clap(short, long)]
        id: String,
    },
    
    /// Show source content
    #[clap(name = "show")]
    Show {
        /// Source ID
        #[clap(short, long)]
        id: String,
    },
}

/// Handle source commands
pub async fn handle_source_command(args: &SourceArgs) -> Result<()> {
    match &args.command {
        SourceCommand::Add { id, type_, path, description } => {
            add_source(id, type_, path, description.clone()).await
        },
        SourceCommand::List => {
            list_sources().await
        },
        SourceCommand::Remove { id } => {
            remove_source(id).await
        },
        SourceCommand::Show { id } => {
            show_source(id).await
        },
    }
}

/// Add a source
async fn add_source(id: &str, type_: &str, path: &str, description: Option<String>) -> Result<()> {
    let mut source_manager = SourceManager::new()?;
    
    let source_type = SourceType::from_str(type_)?;
    let source_path = PathBuf::from(path);
    
    let source = Source::new(
        id.to_string(),
        source_type,
        source_path,
        description,
    );
    
    source_manager.add_source(source)?;
    
    branding::print_success(&format!("Source '{}' added successfully", id));
    
    Ok(())
}

/// List sources
async fn list_sources() -> Result<()> {
    let source_manager = SourceManager::new()?;
    
    let sources = source_manager.list_sources();
    
    if sources.is_empty() {
        println!("No sources found");
        return Ok(());
    }
    
    println!("Sources:");
    for source in sources {
        println!("  ID: {}", source.id);
        println!("    Type: {}", source.source_type.to_string());
        println!("    Path: {}", source.path.display());
        if let Some(description) = &source.description {
            println!("    Description: {}", description);
        }
        println!();
    }
    
    Ok(())
}

/// Remove a source
async fn remove_source(id: &str) -> Result<()> {
    let mut source_manager = SourceManager::new()?;
    
    source_manager.remove_source(id)?;
    
    branding::print_success(&format!("Source '{}' removed successfully", id));
    
    Ok(())
}

/// Show source content
async fn show_source(id: &str) -> Result<()> {
    let source_manager = SourceManager::new()?;
    
    let source = source_manager.get_source(id)
        .ok_or_else(|| anyhow::anyhow!("Source not found: {}", id))?;
        
    let content = source.get_content()?;
    
    println!("Source: {} ({})", source.id, source.source_type.to_string());
    if let Some(description) = &source.description {
        println!("Description: {}", description);
    }
    println!("Path: {}", source.path.display());
    println!();
    println!("{}", content);
    
    Ok(())
}
