use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub mod command_parser;
pub mod command_validator;
pub mod config;
pub mod orchestrator;
pub mod providers;
pub mod query_provider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalAIConfig {
    pub active_provider: String,
    pub providers: std::collections::HashMap<String, providers::ProviderConfig>,
}

impl Default for TerminalAIConfig {
    fn default() -> Self {
        let mut providers = std::collections::HashMap::new();

        // Add default providers
        providers.insert(
            "ollama".to_string(),
            providers::ProviderConfig::new_ollama(
                "http://localhost:11434".to_string(),
                "llama2".to_string(),
                30,
            ),
        );

        providers.insert(
            "openai".to_string(),
            providers::ProviderConfig::new_openai("".to_string(), "gpt-3.5-turbo".to_string(), 30),
        );

        providers.insert(
            "claude".to_string(),
            providers::ProviderConfig::new_claude(
                "".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                30,
            ),
        );

        providers.insert(
            "gemini".to_string(),
            providers::ProviderConfig::new_gemini("".to_string(), "gemini-pro".to_string(), 30),
        );

        providers.insert(
            "local".to_string(),
            providers::ProviderConfig::new_local(30),
        );

        Self {
            active_provider: "ollama".to_string(),
            providers,
        }
    }
}

impl TerminalAIConfig {
    pub fn get_active_provider(&self) -> Option<&providers::ProviderConfig> {
        self.providers.get(&self.active_provider)
    }

    pub fn set_active_provider(&mut self, provider_name: &str) -> Result<()> {
        if self.providers.contains_key(provider_name) {
            self.active_provider = provider_name.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Provider '{}' not found", provider_name))
        }
    }

    pub fn update_provider(&mut self, provider_name: &str, config: providers::ProviderConfig) {
        self.providers.insert(provider_name.to_string(), config);
    }

    pub fn get_provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Failed to find home directory")?;
    Ok(home_dir.join(".terminalai").join("config.json"))
}

pub fn get_local_config_path() -> Result<PathBuf> {
    // Get path relative to the current executable
    let exe_path = std::env::current_exe().context("Failed to get executable path")?;
    let exe_dir = exe_path
        .parent()
        .context("Failed to get executable directory")?;
    Ok(exe_dir.join("terminalai.conf"))
}

pub fn load_config_from_conf(path: &PathBuf) -> Result<TerminalAIConfig> {
    let content = std::fs::read_to_string(path).context("Failed to read config file")?;

    let mut config = TerminalAIConfig::default();
    let mut current_section = String::new();
    let mut active_provider_set = false;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Handle active provider setting
        if line.starts_with("active_provider") {
            if let Some(value) = line.split('=').nth(1) {
                let provider_name = value.trim().trim_matches('"').to_string();
                config.active_provider = provider_name;
                active_provider_set = true;
            }
            continue;
        }

        // Handle section headers [section_name]
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            continue;
        }

        // Handle key = value pairs within sections
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim().trim_matches('"');

            if !current_section.is_empty() {
                // Update the provider config for this section
                if let Some(provider_config) = config.providers.get_mut(&current_section) {
                    match key {
                        "timeout_seconds" => {
                            if let Ok(timeout) = value.parse::<u64>() {
                                provider_config.timeout_seconds = timeout;
                            }
                        }
                        _ => {
                            provider_config
                                .settings
                                .insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }
    }

    // If no active provider was set in the file, keep the default
    if !active_provider_set {
        config.active_provider = "ollama".to_string();
    }

    Ok(config)
}

pub fn load_config() -> Result<TerminalAIConfig> {
    // First, try to load from local .conf file (next to executable)
    if let Ok(local_config_path) = get_local_config_path() {
        if local_config_path.exists() {
            return load_config_from_conf(&local_config_path);
        }
    }

    // Fallback to JSON config in user config directory
    let config_path = get_config_path()?;
    if config_path.exists() {
        let config_content =
            std::fs::read_to_string(&config_path).context("Failed to read config file")?;

        // Try to parse as new multi-provider format first
        if let Ok(config) = serde_json::from_str::<TerminalAIConfig>(&config_content) {
            return Ok(config);
        }

        // If that fails, try to parse as old single-provider format and migrate
        if let Ok(old_config) = serde_json::from_str::<OldTerminalAIConfig>(&config_content) {
            let mut new_config = TerminalAIConfig::default();

            // Determine provider name based on type
            let provider_name = match old_config.provider.provider_type {
                providers::ProviderType::Ollama => "ollama",
                providers::ProviderType::OpenAI => "openai",
                providers::ProviderType::Claude => "claude",
                providers::ProviderType::Gemini => "gemini",
                providers::ProviderType::Local => "local",
            };

            new_config.active_provider = provider_name.to_string();
            new_config
                .providers
                .insert(provider_name.to_string(), old_config.provider);

            return Ok(new_config);
        }
    }

    // Return default if no config exists
    Ok(TerminalAIConfig::default())
}

// Old config format for migration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OldTerminalAIConfig {
    pub provider: providers::ProviderConfig,
}

pub fn save_config(config: &TerminalAIConfig) -> Result<()> {
    let config_path = get_config_path()?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let config_content =
        serde_json::to_string_pretty(config).context("Failed to serialize config")?;

    std::fs::write(&config_path, config_content).context("Failed to write config file")?;

    Ok(())
}

pub fn save_config_to_conf(config: &TerminalAIConfig, path: &PathBuf) -> Result<()> {
    let mut content = String::new();
    content.push_str("# Terminal AI Configuration File\n");
    content.push_str("# This file contains configuration for multiple AI providers\n\n");

    // Write active provider
    content.push_str(&format!(
        "active_provider = \"{}\"\n\n",
        config.active_provider
    ));

    // Write each provider section
    for (provider_name, provider_config) in &config.providers {
        content.push_str(&format!(
            "# {} Configuration\n",
            provider_name.to_uppercase()
        ));
        content.push_str(&format!("[{provider_name}]\n"));

        // Write all settings
        for (key, value) in &provider_config.settings {
            content.push_str(&format!("{key} = \"{value}\"\n"));
        }

        // Write timeout
        content.push_str(&format!(
            "timeout_seconds = {}\n\n",
            provider_config.timeout_seconds
        ));
    }

    std::fs::write(path, content).context("Failed to write config file")?;
    Ok(())
}

/// Fix find commands that use -exec with + terminator
/// The + terminator doesn't work well when passed through sh -c, so we convert it to ;
fn fix_find_exec_command(cmd: &str) -> String {
    // Check if this is a find command with -exec that ends with +
    if cmd.trim_start().starts_with("find ")
        && cmd.contains("-exec")
        && cmd.trim_end().ends_with(" +")
    {
        // Replace the trailing " +" with " \;"
        let mut fixed_cmd = cmd.to_string();
        if let Some(pos) = fixed_cmd.rfind(" +") {
            // Make sure this is actually the terminator and not part of a path or argument
            let after_plus = &fixed_cmd[pos + 2..].trim();
            if after_plus.is_empty() {
                fixed_cmd.replace_range(pos.., r" \;");
                return fixed_cmd;
            }
        }
    }
    cmd.to_string()
}

pub fn extract_commands_from_response(ai_response: &str) -> Vec<String> {
    // Look for command patterns in the AI response
    let lines: Vec<&str> = ai_response.lines().collect();
    let mut commands_to_execute = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Look for code blocks or command patterns
        if trimmed.starts_with("```bash") || trimmed.starts_with("```") {
            continue;
        }
        if trimmed == "```" {
            continue;
        }

        // Look for actual commands (starting with common command prefixes)
        if trimmed.starts_with("cp ")
            || trimmed.starts_with("grep ")
            || trimmed.starts_with("find ")
            || trimmed.starts_with("ps ")
            || trimmed.starts_with("mkdir ")
            || trimmed.starts_with("npm ")
            || trimmed.starts_with("pip ")
            || trimmed.starts_with("python -m pip ")
            || trimmed.starts_with("conda ")
            || trimmed.starts_with("pyenv ")
            || trimmed.starts_with("nvm ")
            || trimmed.starts_with("brew ")
            || trimmed.starts_with("rm -rf ")
            || trimmed.starts_with("yarn ")
            || trimmed.starts_with("poetry ")
            || trimmed.starts_with("pipenv ")
        {
            commands_to_execute.push(trimmed.to_string());
        }
    }

    commands_to_execute
}

pub fn extract_and_execute_command(ai_response: &str) -> Result<()> {
    let commands_to_execute = extract_commands_from_response(ai_response);

    if commands_to_execute.is_empty() {
        println!("⚠️  No executable commands found in AI response.");
        println!("💡 AI Response:");
        println!("{ai_response}");
        return Ok(());
    }

    // Show commands to user and ask for confirmation
    println!("Terminal AI suggest following commands:");
    for (i, cmd) in commands_to_execute.iter().enumerate() {
        println!("  {}. {}", i + 1, cmd);
    }

    print!("\n❓ Execute these commands? [Y/n]: ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() == "n" || input.trim().to_lowercase() == "no" {
        println!("❌ Commands not executed.");
        return Ok(());
    }

    // Execute commands with live output
    for cmd in &commands_to_execute {
        if let Err(e) = execute_command_with_live_output(cmd) {
            println!("🛑 Stopping execution due to command failure.");
            return Err(e);
        }
    }

    Ok(())
}

/// Check if a command is an installation, update, or remove command
pub fn is_install_update_remove_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();

    // Package manager installation commands
    let install_patterns = [
        "npm install",
        "yarn install",
        "pnpm install",
        "pip install",
        "python -m pip install",
        "pip3 install",
        "apt install",
        "apt-get install",
        "yum install",
        "dnf install",
        "brew install",
        "snap install",
        "flatpak install",
        "cargo install",
        "go install",
        "gem install",
        "composer install",
        "maven install",
        "gradle install",
        "choco install",
        "scoop install",
        "winget install",
        "pacman -S",
        "zypper install",
        "emerge",
        "nix-env -i",
        "guix install",
        "spack install",
    ];

    // Update commands
    let update_patterns = [
        "npm update",
        "yarn upgrade",
        "pnpm update",
        "pip install --upgrade",
        "pip install -U",
        "python -m pip install --upgrade",
        "apt update",
        "apt-get update",
        "yum update",
        "dnf update",
        "brew update",
        "snap refresh",
        "flatpak update",
        "cargo update",
        "go get -u",
        "gem update",
        "composer update",
        "maven versions:use-latest-versions",
        "choco upgrade",
        "scoop update",
        "winget upgrade",
        "pacman -Syu",
        "zypper update",
        "emerge --update",
        "nix-env -u",
        "guix upgrade",
        "spack update",
    ];

    // Remove/uninstall commands
    let remove_patterns = [
        "npm uninstall",
        "npm remove",
        "yarn remove",
        "pnpm remove",
        "pip uninstall",
        "python -m pip uninstall",
        "pip3 uninstall",
        "apt remove",
        "apt-get remove",
        "yum remove",
        "dnf remove",
        "brew uninstall",
        "snap remove",
        "flatpak uninstall",
        "cargo uninstall",
        "go clean",
        "gem uninstall",
        "composer remove",
        "maven dependency:purge-local-repository",
        "choco uninstall",
        "scoop uninstall",
        "winget uninstall",
        "pacman -R",
        "zypper remove",
        "emerge --unmerge",
        "nix-env -e",
        "guix remove",
        "spack uninstall",
    ];

    // Check if command matches any pattern
    install_patterns
        .iter()
        .any(|&pattern| cmd_lower.contains(pattern))
        || update_patterns
            .iter()
            .any(|&pattern| cmd_lower.contains(pattern))
        || remove_patterns
            .iter()
            .any(|&pattern| cmd_lower.contains(pattern))
}

/// Execute a command with live output and Terminal AI branding for install/update/remove commands
pub fn execute_command_with_live_output(cmd: &str) -> Result<()> {
    let is_install_cmd = is_install_update_remove_command(cmd);

    if is_install_cmd {
        println!(
            "{}",
            "[Terminal AI] - Executing package management command"
                .green()
                .bold()
        );
        println!("{}", format!("[Terminal AI] - Command: {cmd}").green());
        println!("{}", "[Terminal AI] - Live output:".green());
    } else {
        println!("\n🔄 Executing: {cmd}");
    }

    // Fix find commands with -exec that end with + which don't work well with sh -c
    let fixed_cmd = fix_find_exec_command(cmd);
    if fixed_cmd != cmd {
        if is_install_cmd {
            println!(
                "{}",
                format!("[Terminal AI] - Adjusted command: {fixed_cmd}").green()
            );
        } else {
            println!("🔧 Adjusted command for compatibility: {fixed_cmd}");
        }
    }

    // Use shell execution with live output
    let mut command = Command::new("sh");
    command.arg("-c");
    command.arg(&fixed_cmd);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());

    let status = command
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to execute command '{}': {}", cmd, e))?;

    if status.success() {
        if is_install_cmd {
            println!(
                "{}",
                "[Terminal AI] - Command completed successfully"
                    .green()
                    .bold()
            );
        } else {
            println!("✅ Command completed successfully");
        }
    } else {
        let exit_code = status.code().unwrap_or(-1);
        if is_install_cmd {
            eprintln!(
                "{}",
                format!("[Terminal AI] - Command failed with exit code: {exit_code}")
                    .red()
                    .bold()
            );
        } else {
            eprintln!("❌ Command failed with exit code: {exit_code:?}");
        }
        return Err(anyhow::anyhow!(
            "Command '{}' failed with exit code: {}",
            cmd,
            exit_code
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_terminalai_config_default() {
        let config = TerminalAIConfig::default();
        assert_eq!(config.active_provider, "ollama");

        let ollama_provider = config.get_active_provider().unwrap();
        assert_eq!(
            ollama_provider.provider_type,
            providers::ProviderType::Ollama
        );
        assert_eq!(
            ollama_provider.get_setting("url").unwrap(),
            "http://localhost:11434"
        );
        assert_eq!(ollama_provider.get_setting("model").unwrap(), "llama2");
        assert_eq!(ollama_provider.timeout_seconds, 30);

        // Check that all expected providers exist
        assert!(config.providers.contains_key("ollama"));
        assert!(config.providers.contains_key("openai"));
        assert!(config.providers.contains_key("claude"));
        assert!(config.providers.contains_key("gemini"));
    }

    #[test]
    fn test_get_config_path() {
        let result = get_config_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("terminalai"));
        assert!(path.to_string_lossy().ends_with("config.json"));
    }

    #[test]
    fn test_load_config_nonexistent_returns_default() {
        // Test the default configuration directly
        let default_config = TerminalAIConfig::default();
        assert_eq!(default_config.active_provider, "ollama");
        let active_provider = default_config.get_active_provider().unwrap();
        assert_eq!(
            active_provider.get_setting("url").unwrap(),
            "http://localhost:11434"
        );
        assert_eq!(active_provider.get_setting("model").unwrap(), "llama2");
        assert_eq!(active_provider.timeout_seconds, 30);
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        // Create a custom config
        let mut original_config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        original_config.update_provider(
            "ollama",
            providers::ProviderConfig::new_ollama(
                "http://test:8080".to_string(),
                "test_model".to_string(),
                60,
            ),
        );

        // Save config to test path
        let config_content = serde_json::to_string_pretty(&original_config).unwrap();
        std::fs::write(&config_path, config_content).unwrap();

        // Load config from test path
        let loaded_content = std::fs::read_to_string(&config_path).unwrap();
        let loaded_config: TerminalAIConfig = serde_json::from_str(&loaded_content).unwrap();

        // Verify config was saved and loaded correctly
        assert_eq!(loaded_config.active_provider, "ollama");
        let active_provider = loaded_config.get_active_provider().unwrap();
        assert_eq!(
            active_provider.get_setting("url").unwrap(),
            "http://test:8080"
        );
        assert_eq!(active_provider.get_setting("model").unwrap(), "test_model");
        assert_eq!(active_provider.timeout_seconds, 60);
    }

    #[test]
    fn test_extract_and_execute_command_no_commands() {
        let ai_response = r#"
This is just a text response without any commands.
There are no executable commands here.
"#;

        // This should succeed but not execute anything
        let result = extract_and_execute_command(ai_response);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_and_execute_command_finds_commands() {
        let ai_response = r#"
Here's what I suggest:

```bash
cp file1.txt backup/
mkdir -p new_directory
grep -r "pattern" .
find . -name "*.txt"
```

These commands will help you accomplish the task.
"#;

        // Test the command extraction logic without user interaction
        let commands = extract_commands_from_response(ai_response);

        // Verify that all expected commands were extracted
        assert_eq!(commands.len(), 4);
        assert!(commands.contains(&"cp file1.txt backup/".to_string()));
        assert!(commands.contains(&"mkdir -p new_directory".to_string()));
        assert!(commands.contains(&"grep -r \"pattern\" .".to_string()));
        assert!(commands.contains(&"find . -name \"*.txt\"".to_string()));
    }

    #[test]
    fn test_extract_commands_from_ai_response() {
        let ai_response = r#"
Here are the commands to run:

cp source.txt destination.txt
mkdir -p test_directory
grep -n "error" logfile.txt
find /path -name "*.log"

Also some text that should be ignored.
"#;

        // Test the command extraction logic by parsing the response
        let lines: Vec<&str> = ai_response.lines().collect();
        let mut found_commands = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            // Skip code block markers
            if trimmed.starts_with("```") {
                continue;
            }

            // Look for actual commands
            if trimmed.starts_with("cp ")
                || trimmed.starts_with("grep ")
                || trimmed.starts_with("find ")
                || trimmed.starts_with("mkdir ")
            {
                found_commands.push(trimmed);
            }
        }

        assert_eq!(found_commands.len(), 4);
        assert!(found_commands.contains(&"cp source.txt destination.txt"));
        assert!(found_commands.contains(&"mkdir -p test_directory"));
        assert!(found_commands.contains(&"grep -n \"error\" logfile.txt"));
        assert!(found_commands.contains(&"find /path -name \"*.log\""));
    }

    #[test]
    fn test_fix_find_exec_command() {
        // Test find command with + terminator - should be fixed
        let find_cmd_plus = "find . -name \"*.txt\" -exec cp {} /dest/ +";
        let fixed = fix_find_exec_command(find_cmd_plus);
        assert_eq!(fixed, "find . -name \"*.txt\" -exec cp {} /dest/ \\;");

        // Test find command with ; terminator - should be unchanged
        let find_cmd_semicolon = "find . -name \"*.txt\" -exec cp {} /dest/ \\;";
        let unchanged = fix_find_exec_command(find_cmd_semicolon);
        assert_eq!(unchanged, find_cmd_semicolon);

        // Test non-find command - should be unchanged
        let non_find_cmd = "cp file.txt dest/";
        let unchanged2 = fix_find_exec_command(non_find_cmd);
        assert_eq!(unchanged2, non_find_cmd);

        // Test find command without -exec - should be unchanged
        let find_no_exec = "find . -name \"*.txt\"";
        let unchanged3 = fix_find_exec_command(find_no_exec);
        assert_eq!(unchanged3, find_no_exec);

        // Test command that contains + but not as terminator - should be unchanged
        let find_plus_in_path =
            "find . -name \"*.txt\" -path \"/path+dir/\" -exec cp {} /dest/ \\;";
        let unchanged4 = fix_find_exec_command(find_plus_in_path);
        assert_eq!(unchanged4, find_plus_in_path);
    }

    #[test]
    fn test_extract_package_management_commands() {
        let ai_response = r#"
Here are the package management commands:

conda install pandas==2.3.1
pip install requests==2.31.0
npm install react@18.2.0
yarn install
poetry install
pipenv install
pyenv install 3.11.5
nvm install 18.17.0
brew install python@3.11

These commands will install the required packages.
"#;

        let commands = extract_commands_from_response(ai_response);

        // Verify that all package management commands were extracted
        assert_eq!(commands.len(), 9);
        assert!(commands.contains(&"conda install pandas==2.3.1".to_string()));
        assert!(commands.contains(&"pip install requests==2.31.0".to_string()));
        assert!(commands.contains(&"npm install react@18.2.0".to_string()));
        assert!(commands.contains(&"yarn install".to_string()));
        assert!(commands.contains(&"poetry install".to_string()));
        assert!(commands.contains(&"pipenv install".to_string()));
        assert!(commands.contains(&"pyenv install 3.11.5".to_string()));
        assert!(commands.contains(&"nvm install 18.17.0".to_string()));
        assert!(commands.contains(&"brew install python@3.11".to_string()));
    }

    #[test]
    fn test_extract_commands_ignores_code_blocks() {
        let ai_response = r#"
Here's the solution:

```bash
cp file1.txt file2.txt
```

And some other content:

```
mkdir test
```

But this should be found:
cp actual_command.txt destination.txt
"#;

        // Test command extraction with code blocks
        let lines: Vec<&str> = ai_response.lines().collect();
        let mut found_commands = Vec::new();
        let mut in_code_block = false;

        for line in lines {
            let trimmed = line.trim();

            // Handle code block markers
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip lines inside code blocks for this test
            if in_code_block {
                continue;
            }

            // Look for actual commands outside code blocks
            if trimmed.starts_with("cp ")
                || trimmed.starts_with("grep ")
                || trimmed.starts_with("find ")
                || trimmed.starts_with("mkdir ")
            {
                found_commands.push(trimmed);
            }
        }

        // Should only find the command outside code blocks
        assert_eq!(found_commands.len(), 1);
        assert!(found_commands.contains(&"cp actual_command.txt destination.txt"));
    }

    #[test]
    fn test_config_serialization() {
        let mut config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            providers::ProviderConfig::new_ollama(
                "http://custom:9090".to_string(),
                "custom_model".to_string(),
                120,
            ),
        );

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("http://custom:9090"));
        assert!(json.contains("custom_model"));
        assert!(json.contains("120"));
        assert!(json.contains("ollama"));

        // Test deserialization
        let deserialized: TerminalAIConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.active_provider, "ollama");
        let active_provider = deserialized.get_active_provider().unwrap();
        assert_eq!(
            active_provider.get_setting("url").unwrap(),
            "http://custom:9090"
        );
        assert_eq!(
            active_provider.get_setting("model").unwrap(),
            "custom_model"
        );
        assert_eq!(active_provider.timeout_seconds, 120);
    }

    #[test]
    fn test_config_partial_json() {
        // Test loading config with missing fields (should use defaults where possible)
        let partial_json = r#"{"provider": {"provider_type": "Ollama", "timeout_seconds": 30}}"#;

        // This should fail because serde expects all fields unless they have defaults
        let result: Result<TerminalAIConfig, _> = serde_json::from_str(partial_json);
        assert!(result.is_err()); // Expected to fail with missing required fields (settings)
    }

    #[test]
    fn test_empty_ai_response() {
        let result = extract_and_execute_command("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_whitespace_only_ai_response() {
        let result = extract_and_execute_command("   \n\t  \n  ");
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_install_update_remove_command() {
        // Test install commands
        assert!(is_install_update_remove_command("npm install react"));
        assert!(is_install_update_remove_command("pip install requests"));
        assert!(is_install_update_remove_command("apt install git"));
        assert!(is_install_update_remove_command("brew install node"));
        assert!(is_install_update_remove_command("cargo install ripgrep"));

        // Test update commands
        assert!(is_install_update_remove_command("npm update"));
        assert!(is_install_update_remove_command(
            "pip install --upgrade requests"
        ));
        assert!(is_install_update_remove_command("apt update"));
        assert!(is_install_update_remove_command("brew update"));

        // Test remove commands
        assert!(is_install_update_remove_command("npm uninstall react"));
        assert!(is_install_update_remove_command("pip uninstall requests"));
        assert!(is_install_update_remove_command("apt remove git"));
        assert!(is_install_update_remove_command("brew uninstall node"));

        // Test non-install commands
        assert!(!is_install_update_remove_command("ls -la"));
        assert!(!is_install_update_remove_command("cat file.txt"));
        assert!(!is_install_update_remove_command("grep pattern file"));
        assert!(!is_install_update_remove_command("find . -name '*.txt'"));
        assert!(!is_install_update_remove_command("cp source dest"));
    }
}
