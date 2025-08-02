use anyhow::Result;
use clap::{Arg, Command};
use terminalai::{config, orchestrator};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("tai")
        .version("0.1.0")
        .author("Terminal AI Contributors")
        .about("AI-powered terminal commands")
        .arg(
            Arg::new("prompt")
                .short('p')
                .long("prompt")
                .help("Convert natural language query into terminal commands and execute them sequentially")
                .value_name("PROMPT")
        )
        .subcommand(
            Command::new("init")
                .about("Initialize Terminal AI configuration")
        )
        .get_matches();

    // Handle -p/--prompt flag for orchestration
    if let Some(prompt) = matches.get_one::<String>("prompt") {
        orchestrator::orchestrate_query(prompt).await?;
        return Ok(());
    }

    match matches.subcommand() {
        Some(("init", _)) => {
            config::init_config().await?;
        }
        _ => {
            println!("🤖 Terminal AI v0.1.0");
            println!();
            println!("Available commands:");
            println!("  tai init         - Initialize configuration");
            println!("  tai -p \"[query]\" - Convert query to commands and execute sequentially");
            println!("  cp_ai [prompt]           - AI-powered copy operations");
            println!("  grep_ai [prompt]         - AI-powered text search");
            println!("  find_ai [prompt]         - AI-powered file and directory search");
            println!("  resolve_ai -t [npm|python] -p [package@version] - AI-powered package dependency resolution");
            println!("  resolve_ai -f [dependency_file] - AI-powered dependency file resolution");
            println!();
            println!("Examples:");
            println!("  tai -p \"create a backup folder and copy all Python files to it\"");
            println!("  cp_ai \"copy all .txt files to documents folder\"");
            println!("  grep_ai \"find all error messages in log files\"");
            println!("  find_ai \"locate all Python files larger than 1MB\"");
            println!("  resolve_ai -t npm -p \"react@18.2.0\"");
            println!("  resolve_ai -t python -p \"requests==2.31.0\"");
            println!("  resolve_ai -f \"package.json\"");
            println!("  resolve_ai -f \"requirements.txt\"");
            println!();
            println!("Use --help with any command for more information.");
        }
    }

    Ok(())
}
