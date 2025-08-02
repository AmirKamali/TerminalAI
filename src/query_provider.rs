use crate::providers::{create_provider, AIProvider};
use crate::TerminalAIConfig;
use anyhow::Result;

pub struct QueryProvider {
    provider: Box<dyn AIProvider>,
}

impl QueryProvider {
    pub fn new(config: TerminalAIConfig) -> Result<Self> {
        let active_provider_config = config.get_active_provider().ok_or_else(|| {
            anyhow::anyhow!(
                "Active provider '{}' not found in configuration",
                config.active_provider
            )
        })?;

        let provider = create_provider(active_provider_config)?;
        Ok(Self { provider })
    }

    pub async fn send_query(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        self.provider.send_query(system_prompt, user_prompt).await
    }

    pub fn provider_name(&self) -> &str {
        self.provider.provider_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;

    #[test]
    fn test_query_provider_new() {
        let mut config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            crate::providers::ProviderConfig::new_ollama(
                "http://localhost:11434".to_string(),
                "test_model".to_string(),
                30,
            ),
        );

        let provider = QueryProvider::new(config);

        // Verify provider was created successfully
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.provider_name(), "Ollama");
    }

    #[tokio::test]
    async fn test_send_query_success() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mut config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            crate::providers::ProviderConfig::new_ollama(mock_url, "test_model".to_string(), 30),
        );

        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"response": "Test AI response", "done": true}"#)
            .create_async()
            .await;

        let provider = QueryProvider::new(config).expect("Failed to create provider");
        let result = provider.send_query("System prompt", "User query").await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test AI response");
    }

    #[tokio::test]
    async fn test_send_query_server_error() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mut config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            crate::providers::ProviderConfig::new_ollama(mock_url, "test_model".to_string(), 30),
        );

        let mock = server
            .mock("POST", "/api/generate")
            .with_status(500)
            .with_body("Internal server error")
            .create_async()
            .await;

        let provider = QueryProvider::new(config).expect("Failed to create provider");
        let result = provider.send_query("System prompt", "User query").await;

        mock.assert_async().await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("500"));
    }

    #[tokio::test]
    async fn test_send_query_invalid_json_response() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mut config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            crate::providers::ProviderConfig::new_ollama(mock_url, "test_model".to_string(), 30),
        );

        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("invalid json response")
            .create_async()
            .await;

        let provider = QueryProvider::new(config).expect("Failed to create provider");
        let result = provider.send_query("System prompt", "User query").await;

        mock.assert_async().await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to parse Ollama response"));
    }

    #[tokio::test]
    async fn test_send_query_request_body() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mut config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            crate::providers::ProviderConfig::new_ollama(mock_url, "test_model".to_string(), 30),
        );

        // Verify the request body contains expected fields
        let mock = server
            .mock("POST", "/api/generate")
            .match_body(Matcher::JsonString(
                r#"{"model":"test_model","prompt":"System prompt\n\nUser Request: User query","stream":false}"#.to_string()
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"response": "AI response", "done": true}"#)
            .create_async()
            .await;

        let provider = QueryProvider::new(config).expect("Failed to create provider");
        let result = provider.send_query("System prompt", "User query").await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_query_prompt_combination() {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();

        let mut config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            crate::providers::ProviderConfig::new_ollama(mock_url, "test_model".to_string(), 30),
        );

        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"response": "Combined response", "done": true}"#)
            .create_async()
            .await;

        let provider = QueryProvider::new(config).expect("Failed to create provider");
        let result = provider
            .send_query("You are a helpful assistant.", "Help me with files")
            .await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Combined response");
    }

    #[tokio::test]
    async fn test_send_query_connection_error() {
        // Test with non-existent server
        let mut config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        config.update_provider(
            "ollama",
            crate::providers::ProviderConfig::new_ollama(
                "http://localhost:99999".to_string(), // Non-existent port
                "test_model".to_string(),
                1, // Short timeout
            ),
        );

        let provider = QueryProvider::new(config).expect("Failed to create provider");
        let result = provider.send_query("System prompt", "User query").await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to send request to Ollama"));
    }

    #[test]
    fn test_query_provider_with_different_provider_types() {
        // Test that QueryProvider can be created with different provider types
        let mut ollama_config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        ollama_config.update_provider(
            "ollama",
            crate::providers::ProviderConfig::new_ollama(
                "http://localhost:11434".to_string(),
                "llama2".to_string(),
                30,
            ),
        );

        let mut openai_config = TerminalAIConfig {
            active_provider: "openai".to_string(),
            ..Default::default()
        };
        openai_config.update_provider(
            "openai",
            crate::providers::ProviderConfig::new_openai(
                "sk-test-key".to_string(),
                "gpt-3.5-turbo".to_string(),
                30,
            ),
        );

        let ollama_provider = QueryProvider::new(ollama_config);
        let openai_provider = QueryProvider::new(openai_config);

        assert!(ollama_provider.is_ok());
        assert!(openai_provider.is_ok());

        assert_eq!(ollama_provider.unwrap().provider_name(), "Ollama");
        assert_eq!(openai_provider.unwrap().provider_name(), "OpenAI");
    }

    #[test]
    fn test_invalid_provider_config() {
        // Test that invalid provider configs are rejected
        let mut invalid_settings = std::collections::HashMap::new();
        // Missing required URL for Ollama
        invalid_settings.insert("model".to_string(), "test".to_string());

        let mut invalid_config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        invalid_config.update_provider(
            "ollama",
            crate::providers::ProviderConfig {
                provider_type: crate::providers::ProviderType::Ollama,
                timeout_seconds: 30,
                settings: invalid_settings,
            },
        );

        let result = QueryProvider::new(invalid_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_name_method() {
        // Test that different providers return correct names
        let mut ollama_config = TerminalAIConfig {
            active_provider: "ollama".to_string(),
            ..Default::default()
        };
        ollama_config.update_provider(
            "ollama",
            crate::providers::ProviderConfig::new_ollama(
                "http://localhost:11434".to_string(),
                "llama2".to_string(),
                30,
            ),
        );

        let mut claude_config = TerminalAIConfig {
            active_provider: "claude".to_string(),
            ..Default::default()
        };
        claude_config.update_provider(
            "claude",
            crate::providers::ProviderConfig::new_claude(
                "sk-ant-test".to_string(),
                "claude-3-sonnet".to_string(),
                30,
            ),
        );

        let mut gemini_config = TerminalAIConfig {
            active_provider: "gemini".to_string(),
            ..Default::default()
        };
        gemini_config.update_provider(
            "gemini",
            crate::providers::ProviderConfig::new_gemini(
                "google-api-key".to_string(),
                "gemini-pro".to_string(),
                30,
            ),
        );

        let ollama_provider = QueryProvider::new(ollama_config).expect("Failed to create provider");
        let claude_provider = QueryProvider::new(claude_config).expect("Failed to create provider");
        let gemini_provider = QueryProvider::new(gemini_config).expect("Failed to create provider");

        assert_eq!(ollama_provider.provider_name(), "Ollama");
        assert_eq!(claude_provider.provider_name(), "Claude");
        assert_eq!(gemini_provider.provider_name(), "Gemini");
    }
}
