use anyhow::{Context, Result};
use clap::{Arg, Command};
use terminalai::{
    command_parser, command_validator, extract_and_execute_command, load_config,
    query_provider::QueryProvider,
};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("find_ai")
        .version("0.1.0")
        .author("Terminal AI Contributors")
        .about("AI-powered file and directory search operations")
        .arg(
            Arg::new("prompt")
                .help("Natural language description of what to find")
                .required(true)
                .index(1),
        )
        .get_matches();

    let prompt = matches.get_one::<String>("prompt").unwrap();

    // Keywords that indicate find/search operations
    let valid_keywords = [
        "find",
        "search",
        "locate",
        "look",
        "discover",
        "files",
        "directories",
        "folders",
        "path",
        "paths",
        "name",
        "pattern",
        "match",
        "filter",
        "contains",
        "size",
        "large",
        "small",
        "empty",
        "recent",
        "modified",
        "created",
        "accessed",
        "old",
        "new",
        "type",
        "extension",
        "executable",
        "hidden",
        "where",
        "which",
        "all",
        "any",
        "get",
        "show",
        "list",
        "scan",
        "browse",
        "explore",
    ];

    let invalid_keywords = [
        "copy",
        "cp",
        "duplicate",
        "backup",
        "move",
        "transfer",
        "delete",
        "remove",
        "rm",
        "kill",
        "destroy",
        "erase",
        "install",
        "download",
        "update",
        "upgrade",
        "configure",
        "edit",
        "modify",
        "change",
        "replace",
        "write",
        "create",
        "make",
        "mkdir",
        "touch",
        "new",
        "compile",
        "build",
        "deploy",
        "start",
        "stop",
        "restart",
    ];

    // Validate that this is a find-related query
    if let Err(e) = command_validator::validate_command_query(
        prompt,
        "find_ai",
        "file and directory search operations",
        &valid_keywords,
        &invalid_keywords,
    ) {
        eprintln!("❌ {e}");
        std::process::exit(1);
    }

    // Load configuration
    let config = load_config()?;

    // Load command definition
    let (system_prompt, _args_section) = command_parser::load_command_definition("find")?;

    // Create query provider
    let provider = QueryProvider::new(config).context("Failed to create query provider")?;

    println!("🔍 Processing your search request...\n");

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
            eprintln!("\n💡 Make sure your AI provider is configured correctly.");
            eprintln!("Run 'tai init' to set up your configuration.");
            std::process::exit(1);
        }
    }

    Ok(())
}
