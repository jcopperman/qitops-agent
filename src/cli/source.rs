use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

// Use the Source and SourceManager from the source module
use crate::source::{Source, SourceType, SourceManager};
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

    /// Add metadata to a source
    #[clap(name = "add-metadata")]
    AddMetadata {
        /// Source ID
        #[clap(short, long)]
        id: String,

        /// Metadata key
        #[clap(short, long)]
        key: String,

        /// Metadata value
        #[clap(short, long)]
        value: String,
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
        SourceCommand::AddMetadata { id, key, value } => {
            add_metadata(id, key, value).await
        },
    }
}

/// Add a source
async fn add_source(id: &str, type_: &str, path: &str, description: Option<String>) -> Result<()> {
    let mut source_manager = SourceManager::new()?;

    let source_type = type_.parse::<SourceType>()?;
    let source_path = PathBuf::from(path);

    // Validate that the path exists
    if !source_path.exists() {
        return Err(anyhow::anyhow!("Source path does not exist: {}", path));
    }

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
    println!("{:-<60}", "");

    for source in sources {
        println!("ID: {}", source.id);
        println!("Type: {}", source.source_type);
        println!("Path: {}", source.path.display());
        if let Some(description) = &source.description {
            println!("Description: {}", description);
        }

        // Show metadata if available
        if !source.metadata.is_empty() {
            println!("Metadata:");
            for (key, value) in &source.metadata {
                println!("  {}: {}", key, value);
            }
        }

        println!("{:-<60}", "");
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

    println!("{:-<60}", "");
    println!("Source: {} ({})", source.id, source.source_type);
    if let Some(description) = &source.description {
        println!("Description: {}", description);
    }
    println!("Path: {}", source.path.display());

    // Show metadata if available
    if !source.metadata.is_empty() {
        println!("Metadata:");
        for (key, value) in &source.metadata {
            println!("  {}: {}", key, value);
        }
    }

    println!("{:-<60}", "");
    println!();
    println!("{}", content);

    Ok(())
}

/// Add metadata to a source
async fn add_metadata(id: &str, key: &str, value: &str) -> Result<()> {
    let mut source_manager = SourceManager::new()?;

    // Get the source
    let source = source_manager.get_source(id)
        .ok_or_else(|| anyhow::anyhow!("Source not found: {}", id))?;

    // Clone the source to modify it
    let mut source_clone = source.clone();

    // Add metadata
    source_clone.add_metadata(key.to_string(), value.to_string());

    // Update the source in the manager
    source_manager.add_source(source_clone)?;

    branding::print_success(&format!("Added metadata '{}={}' to source '{}'", key, value, id));

    Ok(())
}
