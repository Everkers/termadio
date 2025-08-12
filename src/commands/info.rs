use anyhow::Result;
use std::env;

pub fn run(verbose: bool) -> Result<()> {
    if verbose {
        println!("Executing info command");
    }

    println!("🖥️  System Information");
    println!("━━━━━━━━━━━━━━━━━━━━");

    // Operating System
    println!("OS: {}", env::consts::OS);
    println!("Architecture: {}", env::consts::ARCH);

    // Current directory
    if let Ok(current_dir) = env::current_dir() {
        println!("Current Directory: {}", current_dir.display());
    }

    // Environment variables (selected ones)
    if let Ok(user) = env::var("USER") {
        println!("User: {}", user);
    } else if let Ok(username) = env::var("USERNAME") {
        println!("User: {}", username);
    }

    if let Ok(shell) = env::var("SHELL") {
        println!("Shell: {}", shell);
    }

    if verbose {
        println!("\n🔧 Additional Details");
        println!("━━━━━━━━━━━━━━━━━━━━");
        println!("Target Family: {}", env::consts::FAMILY);
        println!("DLL Extension: {}", env::consts::DLL_EXTENSION);
        println!("EXE Extension: {}", env::consts::EXE_EXTENSION);
    }

    Ok(())
}
