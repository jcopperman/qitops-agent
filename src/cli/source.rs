use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

// Define the Source, SourceType, and SourceManager here
#[derive(Debug, Clone)]
pub enum SourceType {
    Requirements,
    Standard,
    Documentation,
    Custom(String),
}

impl std::str::FromStr for SourceType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "requirements" => Ok(SourceType::Requirements),
            "standard" => Ok(SourceType::Standard),
            "documentation" => Ok(SourceType::Documentation),
            _ => Ok(SourceType::Custom(s.to_string())),
        }
    }
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Requirements => write!(f, "requirements"),
            SourceType::Standard => write!(f, "standard"),
            SourceType::Documentation => write!(f, "documentation"),
            SourceType::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub id: String,
    pub source_type: SourceType,
    pub path: PathBuf,
    pub description: Option<String>,
}

impl Source {
    pub fn new(id: String, source_type: SourceType, path: PathBuf, description: Option<String>) -> Self {
        Self {
            id,
            source_type,
            path,
            description,
        }
    }

    pub fn get_content(&self) -> Result<String> {
        Ok(std::fs::read_to_string(&self.path)?)
    }
}

pub struct SourceManager {
    sources: std::collections::HashMap<String, Source>,
}

impl SourceManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sources: std::collections::HashMap::new(),
        })
    }

    pub fn add_source(&mut self, source: Source) -> Result<()> {
        self.sources.insert(source.id.clone(), source);
        Ok(())
    }

    pub fn remove_source(&mut self, id: &str) -> Result<()> {
        self.sources.remove(id);
        Ok(())
    }

    pub fn get_source(&self, id: &str) -> Option<&Source> {
        self.sources.get(id)
    }

    pub fn list_sources(&self) -> Vec<&Source> {
        self.sources.values().collect()
    }

    pub fn get_content_for_sources(&self, sources: &[String]) -> Result<String> {
        let mut content = String::new();

        for source_id in sources {
            if let Some(source) = self.get_source(source_id) {
                content.push_str(&format!("# Source: {} ({})\n\n", source_id, source.source_type));
                content.push_str(&source.get_content()?);
                content.push_str("\n\n");
            }
        }

        Ok(content)
    }
}
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

    let source_type = type_.parse::<SourceType>()?;
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
        println!("    Type: {}", source.source_type);
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

    println!("Source: {} ({})", source.id, source.source_type);
    if let Some(description) = &source.description {
        println!("Description: {}", description);
    }
    println!("Path: {}", source.path.display());
    println!();
    println!("{}", content);

    Ok(())
}
