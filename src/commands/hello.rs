use anyhow::Result;

pub fn run(name: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("Executing hello command with name: {}", name);
    }

    println!("Hello, {}! 👋", name);

    if verbose {
        println!("Hello command completed successfully");
    }

    Ok(())
}
