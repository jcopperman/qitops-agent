// QitOps Agent library

// Re-export modules
pub mod agent;
pub mod cli;
pub mod llm;
pub mod plugin;
pub mod ci;
pub mod source;
pub mod persona;
pub mod config;
pub mod bot;
pub mod update;
pub mod monitoring;
pub mod context;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
