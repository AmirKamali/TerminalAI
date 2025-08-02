use anyhow::{Context, Result};
use clap::{Arg, Command};
use terminalai::{
    command_parser, command_validator, extract_and_execute_command, load_config,
    query_provider::QueryProvider,
};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("template_ai")
        .version("0.1.0")
        .author("Terminal AI Contributors")
        .about("AI-powered [COMMAND_DESCRIPTION] operations")
        .arg(
            Arg::new("prompt")
                .help("Natural language description of the [COMMAND_TYPE] operation")
                .required(true)
                .index(1),
        )
        .get_matches();

    let prompt = matches.get_one::<String>("prompt").unwrap();

    // TODO: Replace with your specific validation keywords
    let valid_keywords = [
        // Add keywords that indicate valid operations for your command
        // Example: "search", "find", "locate" for search commands
        // Example: "copy", "cp", "duplicate" for copy commands
        "keyword1", "keyword2", "keyword3",
    ];

    let invalid_keywords = [
        // Add keywords that indicate operations outside your command's scope
        // Example: "delete", "remove", "install" for most commands
        "invalid1", "invalid2", "invalid3",
    ];

    // Validate that this is a [COMMAND_TYPE]-related query
    // Replace "template_ai", "[COMMAND_TYPE] operations", and the keyword arrays
    if let Err(e) = command_validator::validate_command_query(
        prompt,
        "template_ai",               // Replace with your command name
        "[COMMAND_TYPE] operations", // Replace with operation description
        &valid_keywords,
        &invalid_keywords,
    ) {
        eprintln!("❌ {e}");
        std::process::exit(1);
    }

    // Load configuration
    let config = load_config()?;

    // Load command definition
    // Replace "template" with your command name (should match cmd/[command].md filename)
    let (system_prompt, _args_section) = command_parser::load_command_definition("template")?;

    // Create query provider
    let provider = QueryProvider::new(config).context("Failed to create query provider")?;

    // Replace emoji and message with appropriate ones for your command
    println!("🤖 Processing your [COMMAND_TYPE] request...\n");

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

/*
TODO: To use this template for a new command:

1. Copy this file to src/bin/[your_command]_ai.rs
2. Replace all instances of:
   - "template_ai" with "[your_command]_ai"
   - "template" with "[your_command]"
   - "[COMMAND_DESCRIPTION]" with a brief description (e.g., "text search", "file copy")
   - "[COMMAND_TYPE]" with the operation type (e.g., "search", "copy", "format")
   - Update valid_keywords and invalid_keywords arrays with appropriate terms
   - Update the emoji and processing message
   - Create cmd/[your_command].md with system prompt and arguments documentation

3. Add your binary to Cargo.toml in the [[bin]] section:
   [[bin]]
   name = "[your_command]_ai"
   path = "src/bin/[your_command]_ai.rs"

4. Build and test:
   cargo build --bin [your_command]_ai
   cargo test

Example for a hypothetical "format_ai" command:
- File: src/bin/format_ai.rs
- Command definition: cmd/format.md
- Valid keywords: ["format", "beautify", "indent", "style", "pretty"]
- Invalid keywords: ["search", "copy", "delete", "install"]
- Processing message: "🎨 Processing your formatting request..."
*/
