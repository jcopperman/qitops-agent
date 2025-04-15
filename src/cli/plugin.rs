use anyhow::Result;
use crate::cli::commands::PluginCommand;
use crate::cli::branding;
use crate::plugin;

/// Handle plugin commands
pub async fn handle_plugin_command(command: &PluginCommand) -> Result<()> {
    match command {
        PluginCommand::List => {
            // Load plugin state
            let enabled_plugins = match plugin::load_plugin_state() {
                Ok(plugins) => plugins,
                Err(e) => {
                    branding::print_error(&format!("Failed to load plugin state: {}", e));
                    return Ok(());
                }
            };

            // List all plugins
            let plugins = plugin::get_all_plugin_metadata()?;

            if plugins.is_empty() {
                println!("No plugins registered.");

                // Show available plugins
                println!("\nAvailable Plugins:");
                println!("{:-<60}", "");
                println!("ID: example");
                println!("Name: Example Plugin");
                println!("Status: {}", if enabled_plugins.contains(&"example".to_string()) { "Enabled" } else { "Disabled" });
                println!("Description: An example plugin for QitOps Agent");
                println!("{:-<60}", "");

                return Ok(());
            }

            println!("Registered Plugins:");
            println!("{:-<60}", "");

            for (id, metadata) in plugins {
                println!("ID: {}", id);
                println!("Name: {}", metadata.name);
                println!("Version: {}", metadata.version);
                println!("Status: {}", if enabled_plugins.contains(&id) { "Enabled" } else { "Disabled" });
                println!("Description: {}", metadata.description);
                println!("Author: {}", metadata.author);
                println!("{:-<60}", "");
            }
        }
        PluginCommand::Show { id } => {
            // Show plugin details
            match plugin::get_plugin_metadata(id)? {
                Some(metadata) => {
                    println!("Plugin Details:");
                    println!("{:-<60}", "");
                    println!("ID: {}", id);
                    println!("Name: {}", metadata.name);
                    println!("Version: {}", metadata.version);
                    println!("Description: {}", metadata.description);
                    println!("Author: {}", metadata.author);
                }
                None => {
                    branding::print_error(&format!("Plugin not found: {}", id));
                }
            }
        }
        PluginCommand::Execute { id, args } => {
            // Execute a plugin
            match plugin::get_plugin(id)? {
                Some(plugin) => {
                    match plugin.execute(args) {
                        Ok(result) => {
                            println!("Plugin execution result:");
                            println!("{}", result);
                        }
                        Err(e) => {
                            branding::print_error(&format!("Plugin execution failed: {}", e));
                        }
                    }
                }
                None => {
                    branding::print_error(&format!("Plugin not found: {}", id));
                }
            }
        }
        PluginCommand::EnableExample => {
            // Load current plugin state
            let mut enabled_plugins = match plugin::load_plugin_state() {
                Ok(plugins) => plugins,
                Err(e) => {
                    branding::print_error(&format!("Failed to load plugin state: {}", e));
                    return Ok(());
                }
            };

            // Check if the plugin is already enabled
            if enabled_plugins.contains(&"example".to_string()) {
                branding::print_error("Example plugin is already enabled");
                return Ok(());
            }

            // Enable the example plugin
            match plugin::register_example_plugin() {
                Ok(_) => {
                    // Add the plugin to the enabled plugins list
                    enabled_plugins.push("example".to_string());

                    // Save plugin state
                    if let Err(e) = plugin::save_plugin_state(&enabled_plugins) {
                        branding::print_error(&format!("Failed to save plugin state: {}", e));
                    }

                    branding::print_success("Example plugin enabled");
                    println!("You can now use the example plugin with: qitops plugin exec example [args]");
                }
                Err(e) => {
                    branding::print_error(&format!("Failed to enable example plugin: {}", e));
                }
            }
        }
        PluginCommand::DisableExample => {
            // Load current plugin state
            let mut enabled_plugins = match plugin::load_plugin_state() {
                Ok(plugins) => plugins,
                Err(e) => {
                    branding::print_error(&format!("Failed to load plugin state: {}", e));
                    return Ok(());
                }
            };

            // Check if the plugin is already disabled
            if !enabled_plugins.contains(&"example".to_string()) {
                branding::print_error("Example plugin is already disabled");
                return Ok(());
            }

            // Remove the plugin from the enabled plugins list
            enabled_plugins.retain(|id| id != "example");

            // Save plugin state
            if let Err(e) = plugin::save_plugin_state(&enabled_plugins) {
                branding::print_error(&format!("Failed to save plugin state: {}", e));
                return Ok(());
            }

            // Unregister the plugin if it's currently registered
            if let Ok(Some(_)) = plugin::get_plugin("example") {
                if let Err(e) = plugin::unregister_plugin("example") {
                    branding::print_error(&format!("Failed to unregister example plugin: {}", e));
                    return Ok(());
                }
            }

            branding::print_success("Example plugin disabled");
            println!("You can re-enable it with: qitops plugin enable-example");
        }
    }

    Ok(())
}
