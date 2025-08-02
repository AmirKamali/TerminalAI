use crate::{load_config, query_provider::QueryProvider};
use anyhow::{Context, Result};

pub async fn orchestrate_query(prompt: &str) -> Result<()> {
    println!("🧠 Analyzing your request: {prompt}\n");

    // Load configuration
    let config = load_config()?;
    let provider = QueryProvider::new(config).context("Failed to create query provider")?;

    // System prompt for query orchestration
    let orchestration_prompt = r#"
You are a terminal command orchestrator. Your job is to analyze user requests and break them down into specific terminal commands that can be executed safely.

Convert user requests into actual shell commands that accomplish the task. Focus on common, safe operations like:
- File operations: cp, mv, mkdir, rm (with caution), ls, find
- Text operations: grep, cat, echo, sort, uniq
- Archive operations: tar, gzip, zip, unzip
- System info: ps, df, du, whoami, pwd
- Network: curl, wget (for safe downloads)

Respond with a list of specific shell commands to execute, one per line, starting each line with "COMMAND: " followed by the command.

Example:
User: "backup all python files to a new folder and then find all TODO comments in them"
Response:
COMMAND: mkdir -p backup_python
COMMAND: find . -name "*.py" -exec cp {} backup_python/ \;
COMMAND: grep -r "TODO" backup_python/

Be specific, safe, and use standard UNIX commands. Avoid destructive operations without explicit confirmation.
Do not include the example commands in your response - only provide commands for the specific user request.
"#;

    // Get orchestration plan from AI
    let orchestration_response = provider
        .send_query(orchestration_prompt, prompt)
        .await
        .context("Failed to get orchestration plan from AI")?;

    println!("📋 Execution Plan:\n{orchestration_response}\n");

    // Parse the orchestration response for commands
    let commands = parse_orchestration_response(&orchestration_response)?;

    if commands.is_empty() {
        println!("⚠️  No specific commands could be generated from your request.");
        println!("💡 Try being more specific about what operations you want to perform.");
        return Ok(());
    }

    // Show commands and ask for confirmation
    println!("🤖 Commands to execute:");
    for (i, cmd) in commands.iter().enumerate() {
        println!("  {}. {}", i + 1, cmd);
    }

    print!("\n❓ Execute these commands in sequence? [Y/n]: ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() == "n" || input.trim().to_lowercase() == "no" {
        println!("❌ Commands not executed.");
        return Ok(());
    }

    // Execute commands in sequence
    for (i, cmd) in commands.iter().enumerate() {
        println!("\n🔄 Step {}: Executing: {}", i + 1, cmd);
        println!("{}", "=".repeat(60));

        let result = execute_shell_command(cmd).await;

        match result {
            Ok(_) => println!("✅ Step {} completed successfully (exit code: 0)\n", i + 1),
            Err(e) => {
                eprintln!("❌ Step {} failed: {}\n", i + 1, e);
                eprintln!("🛑 Stopping execution due to non-zero exit code.");
                return Err(e);
            }
        }
    }

    println!("🎉 Orchestration complete!");
    Ok(())
}

fn parse_orchestration_response(response: &str) -> Result<Vec<String>> {
    let mut commands = Vec::new();

    for line in response.lines() {
        let line = line.trim();
        if line.starts_with("COMMAND:") {
            let command = line.strip_prefix("COMMAND:").unwrap().trim();

            // Basic validation - ensure command is not empty and doesn't contain potentially dangerous patterns
            if !command.is_empty() && is_safe_command(command) {
                commands.push(command.to_string());
            }
        }
    }

    Ok(commands)
}

fn is_safe_command(command: &str) -> bool {
    // Basic safety checks - reject obviously dangerous patterns
    let dangerous_patterns = [
        "rm -rf /",
        "dd if=",
        "mkfs.",
        "fdisk",
        "chmod 777",
        "sudo rm",
        ">/dev/",
    ];

    for pattern in &dangerous_patterns {
        if command.contains(pattern) {
            return false;
        }
    }

    // Additional check for common safe command prefixes
    let safe_prefixes = [
        "ls", "find", "grep", "cat", "echo", "pwd", "whoami", "mkdir", "cp", "mv", "tar", "gzip",
        "gunzip", "zip", "unzip", "sort", "uniq", "wc", "head", "tail", "ps", "df", "du", "curl",
        "wget", "git",
    ];

    for prefix in &safe_prefixes {
        if command.starts_with(prefix) {
            return true;
        }
    }

    // Allow other commands but log them for review
    println!("⚠️  Allowing command that may need review: {command}");
    true
}

async fn execute_shell_command(cmd: &str) -> Result<()> {
    use colored::*;
    use std::process::Stdio;
    use tokio::process::Command;

    let is_install_cmd = crate::is_install_update_remove_command(cmd);

    if is_install_cmd {
        println!(
            "{}",
            "[Terminal AI] - Executing package management command"
                .green()
                .bold()
        );
        println!("{}", format!("[Terminal AI] - Command: {cmd}").green());
        println!("{}", "[Terminal AI] - Live output:".green());
    }

    // Use shell to execute the command for proper handling of pipes, redirects, etc.
    let mut command = Command::new("sh");
    command.arg("-c");
    command.arg(cmd);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    let output = command
        .output()
        .await
        .context(format!("Failed to execute command: {cmd}"))?;

    // Check exit code - must be 0 to continue
    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        if is_install_cmd {
            eprintln!(
                "{}",
                format!("[Terminal AI] - Command failed with exit code: {exit_code}")
                    .red()
                    .bold()
            );
        }
        return Err(anyhow::anyhow!(
            "Command '{}' failed with exit code: {}",
            cmd,
            exit_code
        ));
    } else if is_install_cmd {
        println!(
            "{}",
            "[Terminal AI] - Command completed successfully"
                .green()
                .bold()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_orchestration_response_valid() {
        let response = r#"
Here's what I recommend:

COMMAND: ls -la
COMMAND: cp file1.txt backup/
COMMAND: mkdir -p new_directory
COMMAND: grep -r "pattern" logs/

These commands should accomplish your task.
"#;

        let result = parse_orchestration_response(response);
        assert!(result.is_ok());

        let commands = result.unwrap();
        assert_eq!(commands.len(), 4);
        assert!(commands.contains(&"ls -la".to_string()));
        assert!(commands.contains(&"cp file1.txt backup/".to_string()));
        assert!(commands.contains(&"mkdir -p new_directory".to_string()));
        assert!(commands.contains(&"grep -r \"pattern\" logs/".to_string()));
    }

    #[test]
    fn test_parse_orchestration_response_no_commands() {
        let response = r#"
This is just a text response without any COMMAND: markers.
No executable commands here.
"#;

        let result = parse_orchestration_response(response);
        assert!(result.is_ok());

        let commands = result.unwrap();
        assert!(commands.is_empty());
    }

    #[test]
    fn test_parse_orchestration_response_mixed_content() {
        let response = r#"
Here's the analysis:

Some text before commands.

COMMAND: find . -name "*.txt"
More text here.
COMMAND: cat file.txt

And some text after.
"#;

        let result = parse_orchestration_response(response);
        assert!(result.is_ok());

        let commands = result.unwrap();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"find . -name \"*.txt\"".to_string()));
        assert!(commands.contains(&"cat file.txt".to_string()));
    }

    #[test]
    fn test_parse_orchestration_response_empty_commands() {
        let response = r#"
COMMAND: 
COMMAND:    
COMMAND: ls -la
"#;

        let result = parse_orchestration_response(response);
        assert!(result.is_ok());

        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert!(commands.contains(&"ls -la".to_string()));
    }

    #[test]
    fn test_is_safe_command_safe_commands() {
        // Test safe commands that should be allowed
        assert!(is_safe_command("ls -la"));
        assert!(is_safe_command("find . -name '*.txt'"));
        assert!(is_safe_command("grep -r pattern ."));
        assert!(is_safe_command("cat file.txt"));
        assert!(is_safe_command("echo 'hello world'"));
        assert!(is_safe_command("pwd"));
        assert!(is_safe_command("whoami"));
        assert!(is_safe_command("mkdir -p new_dir"));
        assert!(is_safe_command("cp source.txt dest.txt"));
        assert!(is_safe_command("mv old.txt new.txt"));
        assert!(is_safe_command("tar -czf archive.tar.gz files/"));
        assert!(is_safe_command("gzip file.txt"));
        assert!(is_safe_command("gunzip file.gz"));
        assert!(is_safe_command("zip archive.zip files/"));
        assert!(is_safe_command("unzip archive.zip"));
        assert!(is_safe_command("sort file.txt"));
        assert!(is_safe_command("uniq file.txt"));
        assert!(is_safe_command("wc -l file.txt"));
        assert!(is_safe_command("head -10 file.txt"));
        assert!(is_safe_command("tail -f logfile.txt"));
        assert!(is_safe_command("ps aux"));
        assert!(is_safe_command("df -h"));
        assert!(is_safe_command("du -sh folder/"));
        assert!(is_safe_command("curl -s https://example.com"));
        assert!(is_safe_command("wget https://example.com/file.txt"));
        assert!(is_safe_command("git status"));
    }

    #[test]
    fn test_is_safe_command_dangerous_commands() {
        // Test dangerous commands that should be rejected
        assert!(!is_safe_command("rm -rf /"));
        assert!(!is_safe_command("dd if=/dev/zero of=/dev/sda"));
        assert!(!is_safe_command("mkfs.ext4 /dev/sda1"));
        assert!(!is_safe_command("fdisk /dev/sda"));
        assert!(!is_safe_command("chmod 777 /etc/passwd"));
        assert!(!is_safe_command("sudo rm -rf /home"));
        assert!(!is_safe_command("echo 'data' >/dev/sda")); // Fixed spacing
    }

    #[test]
    fn test_is_safe_command_edge_cases() {
        // Test commands that contain dangerous patterns but might be legitimate
        assert!(!is_safe_command("rm -rf / # this is a comment"));
        assert!(!is_safe_command("backup_script.sh && rm -rf /tmp"));
        assert!(!is_safe_command("ls -la && dd if=/dev/random"));

        // Test empty command
        assert!(is_safe_command(""));

        // Test commands that don't match safe prefixes (should be allowed with warning)
        assert!(is_safe_command("custom_script.sh"));
        assert!(is_safe_command("python script.py"));
        assert!(is_safe_command("node server.js"));
    }

    #[test]
    fn test_is_safe_command_whitespace_handling() {
        // Test commands with various whitespace patterns
        assert!(is_safe_command("  ls -la  "));
        assert!(is_safe_command("\tgrep pattern file\t"));
        assert!(is_safe_command("find . -name '*.txt'"));
    }

    #[test]
    fn test_parse_orchestration_response_whitespace_in_commands() {
        let response = r#"
COMMAND:   ls -la   
COMMAND:	grep pattern file	
COMMAND: find . -name "*.txt"
"#;

        let result = parse_orchestration_response(response);
        assert!(result.is_ok());

        let commands = result.unwrap();
        assert_eq!(commands.len(), 3);
        // Commands should be trimmed
        assert!(commands.contains(&"ls -la".to_string()));
        assert!(commands.contains(&"grep pattern file".to_string()));
        assert!(commands.contains(&"find . -name \"*.txt\"".to_string()));
    }

    #[test]
    fn test_parse_orchestration_response_case_sensitive() {
        let response = r#"
COMMAND: ls -la
Command: this should not be parsed
command: this should not be parsed either
COMMAND: echo "this should be parsed"
"#;

        let result = parse_orchestration_response(response);
        assert!(result.is_ok());

        let commands = result.unwrap();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"ls -la".to_string()));
        assert!(commands.contains(&"echo \"this should be parsed\"".to_string()));
    }

    #[test]
    fn test_parse_orchestration_response_dangerous_commands_filtered() {
        let response = r#"
COMMAND: ls -la
COMMAND: rm -rf /
COMMAND: find . -name "*.txt"
COMMAND: dd if=/dev/zero of=/dev/sda
COMMAND: echo "safe command"
"#;

        let result = parse_orchestration_response(response);
        assert!(result.is_ok());

        let commands = result.unwrap();
        // Should only contain safe commands
        assert_eq!(commands.len(), 3);
        assert!(commands.contains(&"ls -la".to_string()));
        assert!(commands.contains(&"find . -name \"*.txt\"".to_string()));
        assert!(commands.contains(&"echo \"safe command\"".to_string()));
        // Dangerous commands should be filtered out
        assert!(!commands.contains(&"rm -rf /".to_string()));
        assert!(!commands.contains(&"dd if=/dev/zero of=/dev/sda".to_string()));
    }

    #[test]
    fn test_is_safe_command_partial_dangerous_patterns() {
        // Test commands that contain partial dangerous patterns
        assert!(is_safe_command("grep 'rm -rf' logfile.txt")); // Should be allowed - searching for pattern
        assert!(!is_safe_command("echo 'dont run: dd if=/dev/zero'")); // Should be rejected - contains dangerous dd pattern
        assert!(!is_safe_command("rm -rf /tmp && ls")); // Should be rejected - contains dangerous rm pattern
        assert!(!is_safe_command("backup && dd if=/dev/sda1")); // Should be rejected - contains dangerous dd pattern
    }

    #[test]
    fn test_orchestration_integration() {
        // Test a realistic orchestration response
        let response = r#"
I'll help you backup your Python files and search for TODOs.

Here's my plan:

COMMAND: mkdir -p python_backup
COMMAND: find . -name "*.py" -type f -exec cp {} python_backup/ \;
COMMAND: ls -la python_backup/
COMMAND: grep -r "TODO" python_backup/

This will create a backup directory, copy all Python files, list them, and find TODO comments.
"#;

        let result = parse_orchestration_response(response);
        assert!(result.is_ok());

        let commands = result.unwrap();
        assert_eq!(commands.len(), 4);
        assert!(commands.contains(&"mkdir -p python_backup".to_string()));
        assert!(commands
            .contains(&"find . -name \"*.py\" -type f -exec cp {} python_backup/ \\;".to_string()));
        assert!(commands.contains(&"ls -la python_backup/".to_string()));
        assert!(commands.contains(&"grep -r \"TODO\" python_backup/".to_string()));

        // Verify all commands are safe
        for command in &commands {
            assert!(
                is_safe_command(command),
                "Command should be safe: {command}"
            );
        }
    }
}
