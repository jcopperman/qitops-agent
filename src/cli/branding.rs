use colored::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_banner() {
    let banner = r#"
  ██████╗ ██╗████████╗ ██████╗ ██████╗ ███████╗
 ██╔═══██╗██║╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
 ██║   ██║██║   ██║   ██║   ██║██████╔╝███████╗
 ██║▄▄ ██║██║   ██║   ██║   ██║██╔═══╝ ╚════██║
 ╚██████╔╝██║   ██║   ╚██████╔╝██║     ███████║
  ╚══▀▀═╝ ╚═╝   ╚═╝    ╚═════╝ ╚═╝     ╚══════╝
                                                "#;
    
    println!("{}", banner.bright_cyan());
    println!("{}", format!("QitOps Agent v{} - AI-powered QA Assistant", VERSION).cyan().bold());
    println!("{}", "Developed by QitOps Team".cyan());
    println!();
}

pub fn print_command_header(command: &str) {
    println!("\n{} {}\n", "▶".bright_cyan(), command.cyan().bold());
}

pub fn print_success(message: &str) {
    println!("\n{} {}\n", "✓".bright_green(), message.green());
}

pub fn print_warning(message: &str) {
    println!("\n{} {}\n", "⚠".yellow(), message.yellow());
}

pub fn print_error(message: &str) {
    eprintln!("\n{} {}\n", "✗".bright_red(), message.red());
}

pub fn print_info(message: &str) {
    println!("\n{} {}\n", "ℹ".bright_blue(), message.blue());
}

pub fn print_section(title: &str) {
    println!("\n{}", title.cyan().underline().bold());
    println!("{}\n", "─".repeat(title.len()).cyan());
}
