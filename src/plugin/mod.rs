// Plugin management
pub mod loader;
pub mod registry;
pub mod example;
pub mod persistence;

// Re-export registry functions
pub use registry::{unregister_plugin, get_plugin, get_plugin_metadata, get_all_plugin_metadata, init as init_plugins};

// Re-export example plugin
pub use example::register_example_plugin;

// Re-export persistence functions
pub use persistence::{save_plugin_state, load_plugin_state};
