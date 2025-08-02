use std::env;
use tempfile::TempDir;
use terminalai::{
    command_parser, command_validator, get_config_path, load_config, save_config, TerminalAIConfig,
};

/// Integration tests for the complete Terminal AI system
/// These tests verify that all components work together correctly
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_cp_ai_command_flow() {
        // Test the complete flow of cp_ai command validation and parsing

        // Valid copy commands should pass validation
        let valid_commands = vec![
            "copy all .txt files to backup folder",
            "cp documents to archive directory",
            "backup my photos to external drive",
            "duplicate project folder",
            "move old files to storage",
        ];

        for command in valid_commands {
            let result = command_validator::validate_cp_query(command);
            assert!(result.is_ok(), "Command should be valid: {command}");
        }

        // Load cp command definition
        let result = command_parser::load_command_definition("cp");
        assert!(result.is_ok());
        let (system_prompt, args_section) = result.unwrap();
        assert!(!system_prompt.is_empty());
        assert!(!args_section.is_empty());
    }

    #[test]
    fn test_grep_ai_command_flow() {
        // Test the complete flow of grep_ai command validation and parsing

        // Valid search commands should pass validation
        let valid_commands = vec![
            "search for TODO comments in code",
            "find error patterns in log files",
            "grep for configuration settings",
            "locate specific text in documents",
            "scan files for security issues",
        ];

        for command in valid_commands {
            let result = command_validator::validate_grep_query(command);
            assert!(result.is_ok(), "Command should be valid: {command}");
        }

        // Load grep command definition
        let result = command_parser::load_command_definition("grep");
        assert!(result.is_ok());
        let (system_prompt, args_section) = result.unwrap();
        assert!(!system_prompt.is_empty());
        assert!(!args_section.is_empty());
    }

    #[test]
    fn test_cross_tool_validation() {
        // Test that cp_ai rejects grep commands and vice versa

        // Search commands should fail in cp_ai
        let search_commands = vec![
            "search for files",
            "find error patterns",
            "grep configuration",
        ];

        for command in search_commands {
            let result = command_validator::validate_cp_query(command);
            assert!(
                result.is_err(),
                "Search command should fail in cp_ai: {command}"
            );

            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("search tools"));
            assert!(error_msg.contains("tai -p"));
        }

        // Copy commands should fail in grep_ai
        let copy_commands = vec![
            "copy files to backup",
            "cp documents folder",
            "backup my data",
        ];

        for command in copy_commands {
            let result = command_validator::validate_grep_query(command);
            assert!(
                result.is_err(),
                "Copy command should fail in grep_ai: {command}"
            );

            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("file copy tools"));
            assert!(error_msg.contains("tai -p"));
        }
    }

    #[test]
    fn test_config_lifecycle() {
        // Test the complete configuration lifecycle
        let temp_dir = TempDir::new().unwrap();

        // Set up temporary config directory
        let original_config_dir = env::var("XDG_CONFIG_HOME").ok();

        // Point config to our temp directory
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        // Test default configuration directly
        let default_config = TerminalAIConfig::default();
        assert_eq!(default_config.active_provider, "ollama");
        let default_provider = default_config.get_active_provider().unwrap();
        assert_eq!(
            default_provider.get_setting("url").unwrap(),
            "http://localhost:11434"
        );
        assert!(default_provider.timeout_seconds > 0); // Just verify positive timeout

        // Test saving a custom config
        let mut custom_config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        custom_config.update_provider(
            "ollama",
            terminalai::providers::ProviderConfig::new_ollama(
                "http://test:8080".to_string(),
                "test_model".to_string(),
                60,
            ),
        );

        let save_result = save_config(&custom_config);
        assert!(save_result.is_ok());

        // Test loading the saved config
        let loaded_config = load_config().unwrap();
        assert_eq!(loaded_config.active_provider, "ollama");
        let loaded_provider = loaded_config.get_active_provider().unwrap();
        assert_eq!(
            loaded_provider.get_setting("url").unwrap(),
            "http://test:8080"
        );
        assert_eq!(loaded_provider.get_setting("model").unwrap(), "test_model");
        assert_eq!(loaded_provider.timeout_seconds, 60);

        // Restore original environment
        if let Some(config_dir) = original_config_dir {
            env::set_var("XDG_CONFIG_HOME", config_dir);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    fn test_error_message_consistency() {
        // Test that error messages are consistent and helpful across tools

        // Test cp_ai error messages
        let cp_errors = vec![
            ("search for files", "search tools (grep, find)"),
            ("delete old files", "file deletion tools (rm)"),
            ("install package", "package management tools"),
        ];

        for (command, expected_tool) in cp_errors {
            let result = command_validator::validate_cp_query(command);
            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains(expected_tool));
            assert!(error_msg.contains("out of scope of cp_ai"));
            assert!(error_msg.contains("tai -p"));
        }

        // Test grep_ai error messages
        let grep_errors = vec![
            ("copy files", "file copy tools (cp, mv)"),
            ("remove files", "file deletion tools (rm)"),
            ("download package", "package management tools"),
        ];

        for (command, expected_tool) in grep_errors {
            let result = command_validator::validate_grep_query(command);
            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains(expected_tool));
            assert!(error_msg.contains("out of scope of grep_ai"));
            assert!(error_msg.contains("tai -p"));
        }
    }

    #[test]
    fn test_command_definitions_exist() {
        // Test that all expected command definitions can be loaded
        let commands = vec!["cp", "grep"];

        for command in commands {
            let result = command_parser::load_command_definition(command);
            assert!(
                result.is_ok(),
                "Failed to load command definition for: {command}"
            );

            let (system_prompt, args_section) = result.unwrap();
            assert!(
                !system_prompt.is_empty(),
                "System prompt should not be empty for: {command}"
            );
            assert!(
                !args_section.is_empty(),
                "Args section should not be empty for: {command}"
            );

            // Verify system prompt contains relevant keywords
            let system_prompt_lower = system_prompt.to_lowercase();
            match command {
                "cp" => {
                    assert!(
                        system_prompt_lower.contains("copy")
                            || system_prompt_lower.contains("cp")
                            || system_prompt_lower.contains("file"),
                        "cp system prompt should contain copy-related keywords"
                    );
                }
                "grep" => {
                    assert!(
                        system_prompt_lower.contains("search")
                            || system_prompt_lower.contains("grep")
                            || system_prompt_lower.contains("find"),
                        "grep system prompt should contain search-related keywords"
                    );
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_config_path_generation() {
        // Test that config path generation works correctly
        let result = get_config_path();
        assert!(result.is_ok());

        let path = result.unwrap();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("terminalai"));
        assert!(path_str.ends_with("config.json"));
    }

    #[test]
    fn test_realistic_user_scenarios() {
        // Test realistic user scenarios end-to-end

        // Scenario 1: User wants to backup Python files
        let backup_scenarios = vec![
            "copy all Python files to backup folder",
            "backup *.py files to external drive",
            "duplicate my python project",
        ];

        for scenario in backup_scenarios {
            let result = command_validator::validate_cp_query(scenario);
            assert!(
                result.is_ok(),
                "Backup scenario should be valid: {scenario}"
            );
        }

        // Scenario 2: User wants to search for patterns
        let search_scenarios = vec![
            "find all TODO comments in my code",
            "search for error messages in logs",
            "grep for configuration values",
        ];

        for scenario in search_scenarios {
            let result = command_validator::validate_grep_query(scenario);
            assert!(
                result.is_ok(),
                "Search scenario should be valid: {scenario}"
            );
        }

        // Scenario 3: User tries wrong tool for task
        // Test cp_ai rejecting non-copy commands
        let result = command_validator::validate_cp_query("search for errors");
        assert!(result.is_err(), "Search command should fail in cp_ai");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("tai -p"));
        assert!(error_msg.contains("search for errors"));

        // Test grep_ai rejecting non-search commands
        let result = command_validator::validate_grep_query("copy files to backup");
        assert!(result.is_err(), "Copy command should fail in grep_ai");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("tai -p"));
        assert!(error_msg.contains("copy files to backup"));
    }

    #[test]
    fn test_edge_cases() {
        // Test various edge cases

        // Empty commands
        assert!(command_validator::validate_cp_query("").is_err());
        assert!(command_validator::validate_grep_query("").is_err());

        // Whitespace only
        assert!(command_validator::validate_cp_query("   ").is_err());
        assert!(command_validator::validate_grep_query("   ").is_err());

        // Single word commands
        assert!(command_validator::validate_cp_query("copy").is_ok());
        assert!(command_validator::validate_grep_query("search").is_ok());

        // Mixed case
        assert!(command_validator::validate_cp_query("COPY FILES").is_ok());
        assert!(command_validator::validate_grep_query("SEARCH PATTERNS").is_ok());

        // Commands with multiple keywords
        assert!(command_validator::validate_cp_query("copy and backup files").is_ok());
        assert!(command_validator::validate_grep_query("search and find patterns").is_ok());
    }

    #[test]
    fn test_command_parser_edge_cases() {
        // Test command parser with edge cases

        // Unknown command
        let result = command_parser::load_command_definition("unknown");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unknown command: unknown"));

        // Case sensitivity
        let result = command_parser::load_command_definition("CP");
        assert!(result.is_err());

        let result = command_parser::load_command_definition("GREP");
        assert!(result.is_err());
    }

    #[test]
    fn test_integration_with_different_configs() {
        // Test that the system works with different configuration values
        let mut config1 = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config1.update_provider(
            "ollama",
            terminalai::providers::ProviderConfig::new_ollama(
                "http://localhost:11434".to_string(),
                "llama2".to_string(),
                30,
            ),
        );

        let mut config2 = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config2.update_provider(
            "ollama",
            terminalai::providers::ProviderConfig::new_ollama(
                "http://remote:8080".to_string(),
                "codellama".to_string(),
                60,
            ),
        );

        let mut config3 = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config3.update_provider(
            "ollama",
            terminalai::providers::ProviderConfig::new_ollama(
                "https://secure.example.com:443".to_string(),
                "custom-model".to_string(),
                120,
            ),
        );

        let configs = vec![config1, config2, config3];

        for config in &configs {
            // Test that config serializes/deserializes correctly
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: TerminalAIConfig = serde_json::from_str(&json).unwrap();

            assert_eq!(config.active_provider, deserialized.active_provider);
            let config_provider = config.get_active_provider().unwrap();
            let deserialized_provider = deserialized.get_active_provider().unwrap();

            assert_eq!(
                config_provider.get_setting("url").unwrap(),
                deserialized_provider.get_setting("url").unwrap()
            );
            assert_eq!(
                config_provider.get_setting("model").unwrap(),
                deserialized_provider.get_setting("model").unwrap()
            );
            assert_eq!(
                config_provider.timeout_seconds,
                deserialized_provider.timeout_seconds
            );
        }
    }
}
