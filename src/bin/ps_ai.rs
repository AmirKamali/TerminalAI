use anyhow::{Context, Result};
use clap::{Arg, Command};
use terminalai::{
    command_parser, command_validator, extract_and_execute_command, load_config,
    query_provider::QueryProvider,
};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("ps_ai")
        .version("0.1.0")
        .author("Terminal AI Contributors")
        .about("AI-powered process management operations")
        .arg(
            Arg::new("prompt")
                .help("Natural language description of the process operation")
                .required(true)
                .index(1),
        )
        .get_matches();

    let prompt = matches.get_one::<String>("prompt").unwrap();

    // Validate that this is a process-related query
    if let Err(e) = command_validator::validate_ps_query(prompt) {
        eprintln!("❌ {e}");
        std::process::exit(1);
    }

    // Load configuration
    let config = load_config()?;

    // Load command definition
    let (system_prompt, _args_section) = command_parser::load_command_definition("ps")?;

    // Create query provider
    let provider = QueryProvider::new(config).context("Failed to create query provider")?;

    println!("🤖 Processing your process management request...\n");

    // Send query to AI
    match provider.send_query(&system_prompt, prompt).await {
        Ok(response) => {
            // Extract and execute commands
            if let Err(e) = extract_and_execute_command(&response) {
                eprintln!("❌ Error executing commands: {e}");
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {e}");
            eprintln!("\n💡 Make sure Ollama is running and configured correctly.");
            eprintln!("Run 'tai init' to set up your configuration.");
            std::process::exit(1);
        }
    }

    Ok(())
}
