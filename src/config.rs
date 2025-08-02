use crate::providers::{ProviderConfig, ProviderType};
use crate::save_config;
use anyhow::{Context, Result};
use std::io::{self, Write};

async fn trigger_local_setup(config: &ProviderConfig) -> Result<()> {
    // Create a temporary LocalProvider to trigger the setup
    let provider = crate::providers::LocalProvider::new(config.clone())?;

    // Install llama.cpp and download the model during setup
    let _ = provider.ensure_llama_cpp_installed()?;
    let _ = provider.ensure_model_downloaded().await?;

    Ok(())
}

pub async fn init_config() -> Result<()> {
    println!("🚀 Initializing Terminal AI configuration...\n");

    // Load existing config or create default
    let mut config = crate::load_config()?;

    // Select what to do: configure new provider or set active provider
    let action = select_action()?;
    println!();

    match action {
        ConfigAction::ConfigureProvider => {
            // Select provider to configure
            let provider_type = select_provider()?;
            println!();

            // Get timeout
            let timeout = get_timeout()?;

            // Configure provider-specific settings
            let provider_config = match provider_type {
                ProviderType::Ollama => configure_ollama(timeout)?,
                ProviderType::OpenAI => configure_openai(timeout)?,
                ProviderType::Claude => configure_claude(timeout)?,
                ProviderType::Gemini => configure_gemini(timeout)?,
                ProviderType::Local => configure_local(timeout)?,
            };

            // Determine provider name
            let provider_name = match provider_type {
                ProviderType::Ollama => "ollama",
                ProviderType::OpenAI => "openai",
                ProviderType::Claude => "claude",
                ProviderType::Gemini => "gemini",
                ProviderType::Local => "local",
            };

            // For local provider, trigger immediate setup (llama.cpp only)
            if provider_type == ProviderType::Local {
                println!("\n🚀 Starting local provider setup...");
                if let Err(e) = trigger_local_setup(&provider_config).await {
                    println!("⚠️  Warning: Failed to complete local setup: {e}");
                    println!("   You can retry by running any command with the local provider.");
                } else {
                    println!("✅ Local provider setup completed successfully!");
                    println!("📋 Both llama.cpp and model are now ready to use.");
                }
            }

            // Update provider in config
            config.update_provider(provider_name, provider_config);

            // Ask if user wants to set this as active provider
            if ask_set_active_provider(provider_name)? {
                config.set_active_provider(provider_name)?;
            }

            println!("\n✅ Provider {provider_name} configured successfully!");
        }
        ConfigAction::SetActiveProvider => {
            // Show available providers and let user select
            let provider_names = config.get_provider_names();
            let selected_provider = select_active_provider(&provider_names)?;
            config.set_active_provider(&selected_provider)?;

            println!("\n✅ Active provider set to: {selected_provider}");
        }
    }

    save_config(&config).context("Failed to save configuration")?;

    println!("🎯 Active provider: {}", config.active_provider);
    println!("📁 Config file location: {:?}", crate::get_config_path()?);

    Ok(())
}

#[derive(Debug)]
enum ConfigAction {
    ConfigureProvider,
    SetActiveProvider,
}

fn select_action() -> Result<ConfigAction> {
    println!("🔧 What would you like to do?");
    println!("1. Configure a provider");
    println!("2. Set active provider");

    loop {
        print!("\nEnter your choice [1-2]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => return Ok(ConfigAction::ConfigureProvider),
            "2" => return Ok(ConfigAction::SetActiveProvider),
            _ => println!("❌ Invalid choice. Please enter 1 or 2."),
        }
    }
}

fn ask_set_active_provider(provider_name: &str) -> Result<bool> {
    print!("🎯 Set {provider_name} as the active provider? [Y/n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(input.is_empty() || input == "y" || input == "yes")
}

fn select_active_provider(provider_names: &[String]) -> Result<String> {
    println!("📡 Available providers:");
    for (i, name) in provider_names.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }

    loop {
        print!("\nSelect provider [1-{}]: ", provider_names.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice >= 1 && choice <= provider_names.len() {
                return Ok(provider_names[choice - 1].clone());
            }
        }

        println!(
            "❌ Invalid choice. Please enter a number between 1 and {}.",
            provider_names.len()
        );
    }
}

fn select_provider() -> Result<ProviderType> {
    println!("📡 Select your AI provider:");
    println!("1. Ollama (Local)");
    println!("2. OpenAI (GPT-3.5/GPT-4)");
    println!("3. Claude (Anthropic)");
    println!("4. Gemini (Google)");
    println!("5. Local (llamacpp)");

    loop {
        print!("\nEnter your choice [1-5]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => return Ok(ProviderType::Ollama),
            "2" => return Ok(ProviderType::OpenAI),
            "3" => return Ok(ProviderType::Claude),
            "4" => return Ok(ProviderType::Gemini),
            "5" => return Ok(ProviderType::Local),
            _ => println!("❌ Invalid choice. Please enter 1, 2, 3, 4, or 5."),
        }
    }
}

fn get_timeout() -> Result<u64> {
    print!("⏱️  Request timeout in seconds [30]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(30)
    } else {
        input
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid timeout value. Please enter a number."))
    }
}

fn configure_ollama(timeout: u64) -> Result<ProviderConfig> {
    println!("\n🦙 Configuring Ollama...");

    print!("Ollama URL [http://localhost:11434]: ");
    io::stdout().flush()?;
    let mut url_input = String::new();
    io::stdin().read_line(&mut url_input)?;
    let url = if url_input.trim().is_empty() {
        "http://localhost:11434".to_string()
    } else {
        url_input.trim().to_string()
    };

    print!("Model name [llama2]: ");
    io::stdout().flush()?;
    let mut model_input = String::new();
    io::stdin().read_line(&mut model_input)?;
    let model = if model_input.trim().is_empty() {
        "llama2".to_string()
    } else {
        model_input.trim().to_string()
    };

    Ok(ProviderConfig::new_ollama(url, model, timeout))
}

fn configure_openai(timeout: u64) -> Result<ProviderConfig> {
    println!("\n🤖 Configuring OpenAI...");

    print!("OpenAI API Key: ");
    io::stdout().flush()?;
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    if api_key.is_empty() {
        return Err(anyhow::anyhow!("OpenAI API key is required"));
    }

    print!("Model [gpt-3.5-turbo]: ");
    io::stdout().flush()?;
    let mut model_input = String::new();
    io::stdin().read_line(&mut model_input)?;
    let model = if model_input.trim().is_empty() {
        "gpt-3.5-turbo".to_string()
    } else {
        model_input.trim().to_string()
    };

    Ok(ProviderConfig::new_openai(api_key, model, timeout))
}

fn configure_claude(timeout: u64) -> Result<ProviderConfig> {
    println!("\n🧠 Configuring Claude...");

    print!("Anthropic API Key: ");
    io::stdout().flush()?;
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    if api_key.is_empty() {
        return Err(anyhow::anyhow!("Anthropic API key is required"));
    }

    print!("Model [claude-3-sonnet-20240229]: ");
    io::stdout().flush()?;
    let mut model_input = String::new();
    io::stdin().read_line(&mut model_input)?;
    let model = if model_input.trim().is_empty() {
        "claude-3-sonnet-20240229".to_string()
    } else {
        model_input.trim().to_string()
    };

    Ok(ProviderConfig::new_claude(api_key, model, timeout))
}

fn configure_gemini(timeout: u64) -> Result<ProviderConfig> {
    println!("\n💎 Configuring Gemini...");

    print!("Google API Key: ");
    io::stdout().flush()?;
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    if api_key.is_empty() {
        return Err(anyhow::anyhow!("Google API key is required"));
    }

    print!("Model [gemini-pro]: ");
    io::stdout().flush()?;
    let mut model_input = String::new();
    io::stdin().read_line(&mut model_input)?;
    let model = if model_input.trim().is_empty() {
        "gemini-pro".to_string()
    } else {
        model_input.trim().to_string()
    };

    Ok(ProviderConfig::new_gemini(api_key, model, timeout))
}

fn configure_local(timeout: u64) -> Result<ProviderConfig> {
    println!("\n🏠 Configuring Local AI Provider...");
    println!("This will automatically install llama.cpp and download the specified model.");
    println!("The installation will be stored in ~/.terminalai/");

    print!("Press Enter to continue or Ctrl+C to cancel: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    print!("Hugging Face model path [Qwen2.5-Coder-1.5B]: ");
    io::stdout().flush()?;
    let mut model_input = String::new();
    io::stdin().read_line(&mut model_input)?;
    let model_path = if model_input.trim().is_empty() {
        "Qwen2.5-Coder-1.5B".to_string()
    } else {
        model_input.trim().to_string()
    };

    let mut config = ProviderConfig::new_local(timeout);
    config.settings.insert("model".to_string(), model_path);

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    // Note: Testing init_config fully is challenging because it reads from stdin
    // In a real testing environment, you would mock stdin or restructure the function
    // to accept input as a parameter. For now, we'll test the parts we can.

    #[test]
    fn test_default_config_is_ollama() {
        // Test that default config uses Ollama provider
        let default_config = crate::TerminalAIConfig::default();

        assert_eq!(default_config.active_provider, "ollama");
        let ollama_provider = default_config.get_active_provider().unwrap();
        assert_eq!(ollama_provider.provider_type, ProviderType::Ollama);
        assert_eq!(
            ollama_provider.get_setting("url").unwrap(),
            "http://localhost:11434"
        );
        assert_eq!(ollama_provider.get_setting("model").unwrap(), "llama2");
        assert_eq!(ollama_provider.timeout_seconds, 30);
    }

    // Test helper functions for provider configuration
    fn create_test_ollama_config(url: &str, model: &str, timeout: u64) -> crate::TerminalAIConfig {
        let mut config = crate::TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            ProviderConfig::new_ollama(url.to_string(), model.to_string(), timeout),
        );
        config
    }

    fn create_test_openai_config(
        api_key: &str,
        model: &str,
        timeout: u64,
    ) -> crate::TerminalAIConfig {
        let mut config = crate::TerminalAIConfig {
            active_provider: "openai".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "openai",
            ProviderConfig::new_openai(api_key.to_string(), model.to_string(), timeout),
        );
        config
    }

    #[test]
    fn test_ollama_provider_config() {
        // Test Ollama provider configuration
        let config = create_test_ollama_config("http://custom:9090", "custom_model", 60);

        assert_eq!(config.active_provider, "ollama");
        let provider = config.get_active_provider().unwrap();
        assert_eq!(provider.provider_type, ProviderType::Ollama);
        assert_eq!(provider.get_setting("url").unwrap(), "http://custom:9090");
        assert_eq!(provider.get_setting("model").unwrap(), "custom_model");
        assert_eq!(provider.timeout_seconds, 60);
    }

    #[test]
    fn test_openai_provider_config() {
        // Test OpenAI provider configuration
        let config = create_test_openai_config("sk-test-key", "gpt-4", 45);

        assert_eq!(config.active_provider, "openai");
        let provider = config.get_active_provider().unwrap();
        assert_eq!(provider.provider_type, ProviderType::OpenAI);
        assert_eq!(provider.get_setting("api_key").unwrap(), "sk-test-key");
        assert_eq!(provider.get_setting("model").unwrap(), "gpt-4");
        assert_eq!(provider.timeout_seconds, 45);
    }

    #[test]
    fn test_provider_config_new_ollama() {
        // Test Ollama provider configuration factory
        let provider = ProviderConfig::new_ollama(
            "http://custom:8080".to_string(),
            "codellama".to_string(),
            45,
        );

        assert_eq!(provider.provider_type, ProviderType::Ollama);
        assert_eq!(provider.get_setting("url").unwrap(), "http://custom:8080");
        assert_eq!(provider.get_setting("model").unwrap(), "codellama");
        assert_eq!(provider.timeout_seconds, 45);
    }

    #[test]
    fn test_provider_config_new_openai() {
        // Test OpenAI provider configuration factory
        let provider =
            ProviderConfig::new_openai("sk-test-key".to_string(), "gpt-3.5-turbo".to_string(), 60);

        assert_eq!(provider.provider_type, ProviderType::OpenAI);
        assert_eq!(provider.get_setting("api_key").unwrap(), "sk-test-key");
        assert_eq!(provider.get_setting("model").unwrap(), "gpt-3.5-turbo");
        assert_eq!(
            provider.get_setting("base_url").unwrap(),
            "https://api.openai.com/v1"
        );
        assert_eq!(provider.timeout_seconds, 60);
    }

    #[test]
    fn test_provider_config_new_claude() {
        // Test Claude provider configuration factory
        let provider = ProviderConfig::new_claude(
            "sk-ant-test-key".to_string(),
            "claude-3-sonnet".to_string(),
            90,
        );

        assert_eq!(provider.provider_type, ProviderType::Claude);
        assert_eq!(provider.get_setting("api_key").unwrap(), "sk-ant-test-key");
        assert_eq!(provider.get_setting("model").unwrap(), "claude-3-sonnet");
        assert_eq!(
            provider.get_setting("base_url").unwrap(),
            "https://api.anthropic.com"
        );
        assert_eq!(provider.timeout_seconds, 90);
    }

    #[test]
    fn test_provider_config_new_gemini() {
        // Test Gemini provider configuration factory
        let provider =
            ProviderConfig::new_gemini("google-api-key".to_string(), "gemini-pro".to_string(), 120);

        assert_eq!(provider.provider_type, ProviderType::Gemini);
        assert_eq!(provider.get_setting("api_key").unwrap(), "google-api-key");
        assert_eq!(provider.get_setting("model").unwrap(), "gemini-pro");
        assert_eq!(
            provider.get_setting("base_url").unwrap(),
            "https://generativelanguage.googleapis.com"
        );
        assert_eq!(provider.timeout_seconds, 120);
    }

    #[test]
    fn test_provider_config_get_setting_or_default() {
        // Test the get_setting_or_default method
        let provider = ProviderConfig::new_ollama(
            "http://custom:8080".to_string(),
            "custom_model".to_string(),
            60,
        );

        assert_eq!(
            provider.get_setting_or_default("url", "default"),
            "http://custom:8080"
        );
        assert_eq!(
            provider.get_setting_or_default("model", "default"),
            "custom_model"
        );
        assert_eq!(
            provider.get_setting_or_default("nonexistent", "default"),
            "default"
        );
    }

    // Integration test that actually uses the new config system
    #[test]
    fn test_config_save_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.json");

        let mut config = crate::TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            ProviderConfig::new_ollama(
                "http://test:8080".to_string(),
                "test_model".to_string(),
                45,
            ),
        );

        // Create the config content and save it (simulating save_config behavior)
        let config_content = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, config_content).unwrap();

        // Verify the file was created and contains expected content
        assert!(config_path.exists());

        let saved_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(saved_content.contains("http://test:8080"));
        assert!(saved_content.contains("test_model"));
        assert!(saved_content.contains("45"));
        assert!(saved_content.contains("Ollama"));

        // Verify we can load it back
        let loaded_config: crate::TerminalAIConfig = serde_json::from_str(&saved_content).unwrap();
        assert_eq!(loaded_config.active_provider, "ollama");
        let provider = loaded_config.get_active_provider().unwrap();
        assert_eq!(provider.get_setting("url").unwrap(), "http://test:8080");
        assert_eq!(provider.get_setting("model").unwrap(), "test_model");
        assert_eq!(provider.timeout_seconds, 45);
    }

    #[test]
    fn test_different_provider_types() {
        // Test creating configs with different provider types
        let ollama_config = create_test_ollama_config("http://localhost:11434", "llama2", 30);
        let openai_config = create_test_openai_config("sk-test", "gpt-3.5-turbo", 60);

        assert_eq!(ollama_config.active_provider, "ollama");
        assert_eq!(openai_config.active_provider, "openai");

        let ollama_provider = ollama_config.get_active_provider().unwrap();
        let openai_provider = openai_config.get_active_provider().unwrap();

        assert_eq!(ollama_provider.provider_type, ProviderType::Ollama);
        assert_eq!(openai_provider.provider_type, ProviderType::OpenAI);

        // Ensure they have different required settings
        assert!(ollama_provider.get_setting("url").is_some());
        assert!(ollama_provider.get_setting("api_key").is_none());

        assert!(openai_provider.get_setting("api_key").is_some());
        assert!(openai_provider.get_setting("url").is_none());
    }
}
