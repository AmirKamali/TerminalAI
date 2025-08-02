use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for AI providers that can generate responses from prompts
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn send_query(&self, system_prompt: &str, user_prompt: &str) -> Result<String>;
    fn provider_name(&self) -> &'static str;
    fn validate_config(&self) -> Result<()>;
}

/// Enum for different AI provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    Ollama,
    OpenAI,
    Claude,
    Gemini,
    Local,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::Ollama => write!(f, "Ollama"),
            ProviderType::OpenAI => write!(f, "OpenAI"),
            ProviderType::Claude => write!(f, "Claude (Anthropic)"),
            ProviderType::Gemini => write!(f, "Gemini (Google)"),
            ProviderType::Local => write!(f, "Local (llamacpp)"),
        }
    }
}

/// Configuration for different AI providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub timeout_seconds: u64,
    pub settings: HashMap<String, String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        let mut settings = HashMap::new();
        settings.insert("url".to_string(), "http://localhost:11434".to_string());
        settings.insert("model".to_string(), "llama2".to_string());

        Self {
            provider_type: ProviderType::Ollama,
            timeout_seconds: 30,
            settings,
        }
    }
}

impl ProviderConfig {
    pub fn new_ollama(url: String, model: String, timeout_seconds: u64) -> Self {
        let mut settings = HashMap::new();
        settings.insert("url".to_string(), url);
        settings.insert("model".to_string(), model);

        Self {
            provider_type: ProviderType::Ollama,
            timeout_seconds,
            settings,
        }
    }

    pub fn new_openai(api_key: String, model: String, timeout_seconds: u64) -> Self {
        let mut settings = HashMap::new();
        settings.insert("api_key".to_string(), api_key);
        settings.insert("model".to_string(), model);
        settings.insert(
            "base_url".to_string(),
            "https://api.openai.com/v1".to_string(),
        );

        Self {
            provider_type: ProviderType::OpenAI,
            timeout_seconds,
            settings,
        }
    }

    pub fn new_claude(api_key: String, model: String, timeout_seconds: u64) -> Self {
        let mut settings = HashMap::new();
        settings.insert("api_key".to_string(), api_key);
        settings.insert("model".to_string(), model);
        settings.insert(
            "base_url".to_string(),
            "https://api.anthropic.com".to_string(),
        );

        Self {
            provider_type: ProviderType::Claude,
            timeout_seconds,
            settings,
        }
    }

    pub fn new_gemini(api_key: String, model: String, timeout_seconds: u64) -> Self {
        let mut settings = HashMap::new();
        settings.insert("api_key".to_string(), api_key);
        settings.insert("model".to_string(), model);
        settings.insert(
            "base_url".to_string(),
            "https://generativelanguage.googleapis.com".to_string(),
        );

        Self {
            provider_type: ProviderType::Gemini,
            timeout_seconds,
            settings,
        }
    }

    pub fn new_local(timeout_seconds: u64) -> Self {
        let mut settings = HashMap::new();
        settings.insert("model".to_string(), "Qwen2.5-Coder-1.5B".to_string());
        settings.insert("llama_cpp_path".to_string(), "".to_string());
        settings.insert("model_path".to_string(), "".to_string());

        Self {
            provider_type: ProviderType::Local,
            timeout_seconds,
            settings,
        }
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_setting_or_default(&self, key: &str, default: &str) -> String {
        self.settings
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }
}

/// Factory function to create the appropriate provider based on configuration
pub fn create_provider(config: &ProviderConfig) -> Result<Box<dyn AIProvider>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_seconds))
        .build()
        .context("Failed to create HTTP client")?;

    match config.provider_type {
        ProviderType::Ollama => {
            let provider = OllamaProvider::new(config.clone(), client)?;
            Ok(Box::new(provider))
        }
        ProviderType::OpenAI => {
            let provider = OpenAIProvider::new(config.clone(), client)?;
            Ok(Box::new(provider))
        }
        ProviderType::Claude => {
            let provider = ClaudeProvider::new(config.clone(), client)?;
            Ok(Box::new(provider))
        }
        ProviderType::Gemini => {
            let provider = GeminiProvider::new(config.clone(), client)?;
            Ok(Box::new(provider))
        }
        ProviderType::Local => {
            let provider = LocalProvider::new(config.clone())?;
            Ok(Box::new(provider))
        }
    }
}

// Ollama Provider Implementation
pub struct OllamaProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig, client: reqwest::Client) -> Result<Self> {
        let provider = Self { config, client };
        provider.validate_config()?;
        Ok(provider)
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn send_query(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let combined_prompt = format!("{system_prompt}\n\nUser Request: {user_prompt}");

        let request = OllamaRequest {
            model: self.config.get_setting_or_default("model", "llama2"),
            prompt: combined_prompt,
            stream: false,
        };

        let url = format!(
            "{}/api/generate",
            self.config
                .get_setting_or_default("url", "http://localhost:11434")
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Ollama request failed with status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(ollama_response.response)
    }

    fn provider_name(&self) -> &'static str {
        "Ollama"
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.get_setting("url").is_none() {
            return Err(anyhow::anyhow!("Ollama URL is required"));
        }
        if self.config.get_setting("model").is_none() {
            return Err(anyhow::anyhow!("Ollama model is required"));
        }
        Ok(())
    }
}

// OpenAI Provider Implementation
pub struct OpenAIProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig, client: reqwest::Client) -> Result<Self> {
        let provider = Self { config, client };
        provider.validate_config()?;
        Ok(provider)
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn send_query(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ];

        let request = OpenAIRequest {
            model: self.config.get_setting_or_default("model", "gpt-3.5-turbo"),
            messages,
            max_tokens: 1000,
            temperature: 0.1,
        };

        let api_key = self
            .config
            .get_setting("api_key")
            .context("OpenAI API key not found in configuration")?;

        let url = format!(
            "{}/chat/completions",
            self.config
                .get_setting_or_default("base_url", "https://api.openai.com/v1")
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "OpenAI request failed with status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        openai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .context("No response from OpenAI")
    }

    fn provider_name(&self) -> &'static str {
        "OpenAI"
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.get_setting("api_key").is_none() {
            return Err(anyhow::anyhow!("OpenAI API key is required"));
        }
        if self.config.get_setting("model").is_none() {
            return Err(anyhow::anyhow!("OpenAI model is required"));
        }
        Ok(())
    }
}

// Claude Provider Implementation
pub struct ClaudeProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    system: String,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

impl ClaudeProvider {
    pub fn new(config: ProviderConfig, client: reqwest::Client) -> Result<Self> {
        let provider = Self { config, client };
        provider.validate_config()?;
        Ok(provider)
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    async fn send_query(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        }];

        let request = ClaudeRequest {
            model: self
                .config
                .get_setting_or_default("model", "claude-3-sonnet-20240229"),
            max_tokens: 1000,
            messages,
            system: system_prompt.to_string(),
        };

        let api_key = self
            .config
            .get_setting("api_key")
            .context("Claude API key not found in configuration")?;

        let url = format!(
            "{}/v1/messages",
            self.config
                .get_setting_or_default("base_url", "https://api.anthropic.com")
        );

        let response = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Claude")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Claude request failed with status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .context("Failed to parse Claude response")?;

        claude_response
            .content
            .first()
            .map(|content| content.text.clone())
            .context("No response from Claude")
    }

    fn provider_name(&self) -> &'static str {
        "Claude"
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.get_setting("api_key").is_none() {
            return Err(anyhow::anyhow!("Claude API key is required"));
        }
        if self.config.get_setting("model").is_none() {
            return Err(anyhow::anyhow!("Claude model is required"));
        }
        Ok(())
    }
}

// Gemini Provider Implementation
pub struct GeminiProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    role: String,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: String,
}

impl GeminiProvider {
    pub fn new(config: ProviderConfig, client: reqwest::Client) -> Result<Self> {
        let provider = Self { config, client };
        provider.validate_config()?;
        Ok(provider)
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn send_query(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let combined_prompt = format!("{system_prompt}\n\nUser Request: {user_prompt}");

        let contents = vec![GeminiContent {
            parts: vec![GeminiPart {
                text: combined_prompt,
            }],
            role: "user".to_string(),
        }];

        let request = GeminiRequest {
            contents,
            generation_config: GeminiGenerationConfig {
                temperature: 0.1,
                max_output_tokens: 1000,
            },
        };

        let api_key = self
            .config
            .get_setting("api_key")
            .context("Gemini API key not found in configuration")?;

        let model = self.config.get_setting_or_default("model", "gemini-pro");
        let base_url = self
            .config
            .get_setting_or_default("base_url", "https://generativelanguage.googleapis.com");
        let url = format!("{base_url}/v1/models/{model}:generateContent?key={api_key}");

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Gemini")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Gemini request failed with status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .context("Failed to parse Gemini response")?;

        gemini_response
            .candidates
            .first()
            .and_then(|candidate| candidate.content.parts.first())
            .map(|part| part.text.clone())
            .context("No response from Gemini")
    }

    fn provider_name(&self) -> &'static str {
        "Gemini"
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.get_setting("api_key").is_none() {
            return Err(anyhow::anyhow!("Gemini API key is required"));
        }
        if self.config.get_setting("model").is_none() {
            return Err(anyhow::anyhow!("Gemini model is required"));
        }
        Ok(())
    }
}

// Local Provider Implementation
pub struct LocalProvider {
    config: ProviderConfig,
}

impl LocalProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let provider = Self { config };
        provider.validate_config()?;
        Ok(provider)
    }

    fn detect_os() -> &'static str {
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        }
    }

    fn get_llama_cpp_download_url_fixed() -> Result<String> {
        let os = Self::detect_os();
        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            return Err(anyhow::anyhow!("Unsupported architecture"));
        };

        // Use the correct ggml-org repository URLs with latest release
        let url = match (os, arch) {
            ("windows", "x64") => "https://github.com/ggml-org/llama.cpp/releases/download/b6075/llama-b6075-bin-win-x64.zip",
            ("windows", "arm64") => "https://github.com/ggml-org/llama.cpp/releases/download/b6075/llama-b6075-bin-win-arm64.zip",
            ("macos", "x64") => "https://github.com/ggml-org/llama.cpp/releases/download/b6075/llama-b6075-bin-macos-x64.zip",
            ("macos", "arm64") => "https://github.com/ggml-org/llama.cpp/releases/download/b6075/llama-b6075-bin-macos-arm64.zip",
            ("linux", "x64") => "https://github.com/ggml-org/llama.cpp/releases/download/b6075/llama-b6075-bin-linux-x64.tar.gz",
            ("linux", "arm64") => "https://github.com/ggml-org/llama.cpp/releases/download/b6075/llama-b6075-bin-linux-arm64.tar.gz",
            _ => return Err(anyhow::anyhow!("Unsupported platform: {os}-{arch}")),
        };

        Ok(url.to_string())
    }

    fn find_executable_recursively(
        dir: &std::path::Path,
        executable_name: &str,
    ) -> Result<Option<std::path::PathBuf>> {
        for entry in std::fs::read_dir(dir).context("Failed to read directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if entry.file_type()?.is_file()
                && path.file_name().unwrap_or_default() == executable_name
            {
                return Ok(Some(path));
            } else if entry.file_type()?.is_dir() {
                // Recursively search subdirectories
                if let Some(found) = Self::find_executable_recursively(&path, executable_name)? {
                    return Ok(Some(found));
                }
            }
        }
        Ok(None)
    }

    fn list_directory_contents(dir: &std::path::Path, depth: usize) -> Result<()> {
        let indent = "  ".repeat(depth);
        for entry in std::fs::read_dir(dir).context("Failed to read directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if entry.file_type()?.is_dir() {
                println!("{indent}{name}/");
                if depth < 2 {
                    // Limit recursion depth
                    Self::list_directory_contents(&path, depth + 1)?;
                }
            } else {
                println!("{indent}{name}");
            }
        }
        Ok(())
    }

    fn make_executables_executable(dir: &std::path::Path) -> Result<()> {
        for entry in std::fs::read_dir(dir).context("Failed to read directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if entry.file_type()?.is_file() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();

                // Check if this looks like an executable (common llama.cpp executable names)
                if name.starts_with("llama-")
                    || name == "main"
                    || name.contains("server")
                    || name.contains("cli")
                {
                    println!("🔧 Making executable: {}", path.display());

                    // Use chmod +x to make the file executable
                    let chmod_output = std::process::Command::new("chmod")
                        .arg("+x")
                        .arg(&path)
                        .output()
                        .context("Failed to run chmod command")?;

                    if !chmod_output.status.success() {
                        let stderr = String::from_utf8_lossy(&chmod_output.stderr);
                        println!("⚠️  chmod warning for {}: {}", path.display(), stderr);
                    }
                }
            } else if entry.file_type()?.is_dir() {
                // Recursively process subdirectories
                Self::make_executables_executable(&path)?;
            }
        }
        Ok(())
    }

    // Note: Model download URLs are no longer used since Hugging Face requires authentication
    // Users need to download models manually or use Ollama

    pub fn ensure_llama_cpp_installed(&self) -> Result<String> {
        let os = Self::detect_os();
        println!("🔍 Detected OS: {os}");

        // Check if llama.cpp is already installed
        let llama_cpp_path = self.config.get_setting("llama_cpp_path");
        if let Some(path) = llama_cpp_path {
            if !path.is_empty() && std::path::Path::new(path).exists() {
                println!("✅ llama.cpp already installed at: {path}");
                return Ok(path.clone());
            }
        }

        // Check if llama.cpp exists in the default installation directory
        let home_dir = dirs::home_dir().context("Failed to find home directory")?;
        let install_dir = home_dir.join(".terminalai").join("llama_cpp");
        let executable_names = if os == "windows" {
            vec!["llama-cli.exe", "main.exe", "llama-server.exe"]
        } else {
            vec!["llama-cli", "main", "llama-server"]
        };

        // Check if any executable exists in the default location
        for name in &executable_names {
            let possible_paths = vec![
                install_dir.join(name),
                install_dir.join("bin").join(name),
                install_dir.join("build").join("bin").join(name),
            ];

            for path in possible_paths {
                if path.exists() {
                    println!("✅ llama.cpp already installed at: {}", path.display());
                    return Ok(path.to_string_lossy().to_string());
                }
            }
        }

        println!("📥 Installing llama.cpp...");

        // Create installation directory
        let home_dir = dirs::home_dir().context("Failed to find home directory")?;
        let install_dir = home_dir.join(".terminalai").join("llama_cpp");
        std::fs::create_dir_all(&install_dir).context("Failed to create installation directory")?;

        // Download llama.cpp
        let download_url = Self::get_llama_cpp_download_url_fixed()?;
        println!("📥 Downloading llama.cpp from: {download_url}");

        let response =
            reqwest::blocking::get(&download_url).context("Failed to download llama.cpp")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download llama.cpp: {}",
                response.status()
            ));
        }

        let archive_data = response.bytes().context("Failed to read download data")?;

        // Extract archive
        println!("📦 Extracting llama.cpp...");
        if download_url.ends_with(".zip") {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&archive_data))
                .context("Failed to read zip archive")?;

            println!("📋 Archive contains {} files", archive.len());
            for i in 0..archive.len() {
                let mut file = archive
                    .by_index(i)
                    .context("Failed to access archive file")?;
                let outpath = install_dir.join(file.name());

                if file.name().ends_with('/') {
                    std::fs::create_dir_all(&outpath).context("Failed to create directory")?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p)
                                .context("Failed to create parent directory")?;
                        }
                    }
                    let mut outfile =
                        std::fs::File::create(&outpath).context("Failed to create file")?;
                    std::io::copy(&mut file, &mut outfile).context("Failed to write file")?;
                }
            }
        } else {
            // Handle tar.gz
            let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(
                std::io::Cursor::new(&archive_data),
            ));
            archive
                .unpack(&install_dir)
                .context("Failed to extract tar.gz archive")?;
        }

        // List extracted contents for debugging
        println!("📁 Extracted contents:");
        Self::list_directory_contents(&install_dir, 0)?;

        // Make executables executable on Unix systems (Linux and macOS)
        if os != "windows" {
            println!("🔧 Setting executable permissions on Unix system...");
            Self::make_executables_executable(&install_dir)?;
        }

        // Find the main executable - try multiple possible locations and names
        let executable_names = if os == "windows" {
            vec!["llama-cli.exe", "main.exe", "llama-server.exe"]
        } else {
            vec!["llama-cli", "main", "llama-server"]
        };

        // Try different possible paths
        let mut possible_paths = Vec::new();
        for name in &executable_names {
            possible_paths.extend(vec![
                install_dir.join(name),
                install_dir.join("bin").join(name),
                install_dir.join("build").join("bin").join(name),
                install_dir.join("llama-b6075-bin-macos-arm64").join(name),
                install_dir.join("llama-b6075-bin-macos-x64").join(name),
                install_dir.join("llama-b6075-bin-linux-x64").join(name),
                install_dir.join("llama-b6075-bin-linux-arm64").join(name),
                install_dir.join("llama-b6075-bin-win-x64").join(name),
                install_dir.join("llama-b6075-bin-win-arm64").join(name),
            ]);
        }

        // Also search recursively in subdirectories
        let mut found_path = None;
        for path in &possible_paths {
            if path.exists() {
                found_path = Some(path.clone());
                break;
            }
        }

        // If not found in predefined paths, search recursively
        if found_path.is_none() {
            for name in &executable_names {
                if let Some(path) = Self::find_executable_recursively(&install_dir, name)? {
                    found_path = Some(path);
                    break;
                }
            }
        }

        let executable_path = found_path.ok_or_else(|| {
            anyhow::anyhow!(
                "Could not find llama.cpp executable in {}",
                install_dir.display()
            )
        })?;

        println!(
            "✅ llama.cpp installed successfully at: {}",
            executable_path.display()
        );
        Ok(executable_path.to_string_lossy().to_string())
    }

    pub fn get_existing_model_path(&self) -> Result<String> {
        let model_path = self.config.get_setting("model_path");
        if let Some(path) = model_path {
            if !path.is_empty() && std::path::Path::new(path).exists() {
                return Ok(path.clone());
            }
        }

        // Get model name from configuration
        let model_name = self
            .config
            .get_setting_or_default("model", "Qwen2.5-Coder-1.5B");

        // Create model directory path
        let home_dir = dirs::home_dir().context("Failed to find home directory")?;
        let model_dir = home_dir.join(".terminalai").join("models");

        // Determine model filename based on model name
        let model_filename = match model_name.as_str() {
            "Qwen2.5-Coder-1.5B" => "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-3B" => "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-7B" => "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-14B" => "qwen2.5-coder-14b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-32B" => "qwen2.5-coder-32b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-72B" => "qwen2.5-coder-72b-instruct-q4_k_m.gguf",
            "Phi-3.5-Mini" => "phi-3.5-mini-4k-instruct.Q4_K_M.gguf",
            "Phi-3.5-Mini-128K" => "phi-3.5-mini-128k-instruct.Q4_K_M.gguf",
            "CodeLlama-3.8B" => "codellama-3.8b-instruct.Q4_K_M.gguf",
            "CodeLlama-7B" => "codellama-7b-instruct.Q4_K_M.gguf",
            _ => "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf", // Default fallback to Qwen2.5-Coder-1.5B
        };

        let model_path = model_dir.join(model_filename);

        if model_path.exists() {
            return Ok(model_path.to_string_lossy().to_string());
        }

        Err(anyhow::anyhow!(
            "No existing model found at: {}",
            model_path.display()
        ))
    }

    pub fn get_model_path(&self) -> Result<String> {
        let model_path = self.config.get_setting("model_path");
        if let Some(path) = model_path {
            if !path.is_empty() && std::path::Path::new(path).exists() {
                println!("✅ Using model at: {path}");
                return Ok(path.clone());
            }
        }

        // Get model name from configuration
        let model_name = self
            .config
            .get_setting_or_default("model", "Qwen2.5-Coder-1.5B");

        // Create model directory
        let home_dir = dirs::home_dir().context("Failed to find home directory")?;
        let model_dir = home_dir.join(".terminalai").join("models");
        std::fs::create_dir_all(&model_dir).context("Failed to create model directory")?;

        // Determine model filename based on model name
        let model_filename = match model_name.as_str() {
            "Qwen2.5-Coder-1.5B" => "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-3B" => "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-7B" => "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-14B" => "qwen2.5-coder-14b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-32B" => "qwen2.5-coder-32b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-72B" => "qwen2.5-coder-72b-instruct-q4_k_m.gguf",
            "Phi-3.5-Mini" => "phi-3.5-mini-4k-instruct.Q4_K_M.gguf",
            "Phi-3.5-Mini-128K" => "phi-3.5-mini-128k-instruct.Q4_K_M.gguf",
            "CodeLlama-3.8B" => "codellama-3.8b-instruct.Q4_K_M.gguf",
            "CodeLlama-7B" => "codellama-7b-instruct.Q4_K_M.gguf",
            _ => "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf", // Default fallback to Qwen2.5-Coder-1.5B
        };

        let model_path = model_dir.join(model_filename);

        if model_path.exists() {
            println!("✅ Using model at: {}", model_path.display());
            return Ok(model_path.to_string_lossy().to_string());
        }

        // Model doesn't exist - try to download using git clone
        println!("⚠️  Model not found: {}", model_path.display());
        println!("📁 Looking for model in folder: {}", model_dir.display());
        println!("🚀 Attempting to download model using git clone...");

        // Check if git-lfs is installed and install if needed
        let lfs_check = std::process::Command::new("git")
            .arg("lfs")
            .arg("version")
            .output();

        if lfs_check.is_err() {
            println!("❌ Git LFS is not installed. Attempting to install automatically...");

            let os = Self::detect_os();
            let install_result = match os {
                "macos" => {
                    println!("📥 Installing git-lfs using Homebrew...");
                    std::process::Command::new("brew")
                        .arg("install")
                        .arg("git-lfs")
                        .output()
                }
                "linux" => {
                    println!("📥 Installing git-lfs using apt-get...");
                    std::process::Command::new("sudo")
                        .arg("apt-get")
                        .arg("update")
                        .output()
                        .and_then(|_| {
                            std::process::Command::new("sudo")
                                .arg("apt-get")
                                .arg("install")
                                .arg("-y")
                                .arg("git-lfs")
                                .output()
                        })
                }
                "windows" => {
                    println!("📥 Installing git-lfs using winget...");
                    std::process::Command::new("winget")
                        .arg("install")
                        .arg("Git.GitLFS")
                        .output()
                }
                _ => {
                    println!("❌ Unsupported OS for automatic git-lfs installation.");
                    println!("📋 Please install git-lfs manually:");
                    println!();
                    println!("macOS (using Homebrew):");
                    println!("   brew install git-lfs");
                    println!();
                    println!("Ubuntu/Debian:");
                    println!("   sudo apt-get install git-lfs");
                    println!();
                    println!("Windows:");
                    println!("   winget install Git.GitLFS");
                    println!();
                    println!("Or visit: https://git-lfs.com");
                    println!();
                    return Err(anyhow::anyhow!("Git LFS is not installed and automatic installation failed. Please install it manually."));
                }
            };

            match install_result {
                Ok(output) if output.status.success() => {
                    println!("✅ Git LFS installed successfully!");
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ Failed to install git-lfs automatically: {stderr}");
                    println!("📋 Please install git-lfs manually:");
                    println!();
                    println!("macOS (using Homebrew):");
                    println!("   brew install git-lfs");
                    println!();
                    println!("Ubuntu/Debian:");
                    println!("   sudo apt-get install git-lfs");
                    println!();
                    println!("Windows:");
                    println!("   winget install Git.GitLFS");
                    println!();
                    println!("Or visit: https://git-lfs.com");
                    println!();
                    return Err(anyhow::anyhow!(
                        "Failed to install git-lfs automatically. Please install it manually."
                    ));
                }
                Err(e) => {
                    println!("❌ Failed to run git-lfs installer: {e}");
                    println!("📋 Please install git-lfs manually:");
                    println!();
                    println!("macOS (using Homebrew):");
                    println!("   brew install git-lfs");
                    println!();
                    println!("Ubuntu/Debian:");
                    println!("   sudo apt-get install git-lfs");
                    println!();
                    println!("Windows:");
                    println!("   winget install Git.GitLFS");
                    println!();
                    println!("Or visit: https://git-lfs.com");
                    println!();
                    return Err(anyhow::anyhow!("Failed to run git-lfs installer: {}", e));
                }
            }
        }

        // Initialize git-lfs globally
        println!("🔧 Initializing Git LFS...");
        let lfs_init_global = std::process::Command::new("git")
            .arg("lfs")
            .arg("install")
            .output();

        if lfs_init_global.is_err() {
            println!("⚠️  Warning: Failed to initialize git-lfs globally");
        }

        // Get the model repository URL
        let model_repo = match model_name.as_str() {
            "Qwen2.5-Coder-1.5B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF",
            "Qwen2.5-Coder-3B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
            "Qwen2.5-Coder-7B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
            "Qwen2.5-Coder-14B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-14B-Instruct-GGUF",
            "Qwen2.5-Coder-32B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-32B-Instruct-GGUF",
            "Qwen2.5-Coder-72B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-72B-Instruct-GGUF",
            "Phi-3.5-Mini" => "https://huggingface.co/TheBloke/Phi-3.5-Mini-4K-Instruct-GGUF",
            "Phi-3.5-Mini-128K" => {
                "https://huggingface.co/TheBloke/Phi-3.5-Mini-128K-Instruct-GGUF"
            }
            "CodeLlama-3.8B" => "https://huggingface.co/TheBloke/CodeLlama-3.8B-Instruct-GGUF",
            "CodeLlama-7B" => "https://huggingface.co/TheBloke/CodeLlama-7B-Instruct-GGUF",
            _ => "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF", // Default fallback
        };

        // Create a temporary directory for cloning
        let temp_dir = model_dir.join(format!("temp_{}", model_name.replace('/', "_")));
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)
                .context("Failed to remove existing temp directory")?;
        }

        // Clone the repository
        println!("📥 Cloning repository: {model_repo}");
        let clone_output = std::process::Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--filter=blob:none")
            .arg("--sparse")
            .arg(model_repo)
            .arg(&temp_dir)
            .output()
            .context("Failed to run git clone")?;

        if !clone_output.status.success() {
            let stderr = String::from_utf8_lossy(&clone_output.stderr);
            println!("❌ Git clone failed: {stderr}");
            println!("📋 Please download the model manually:");
            println!();
            println!("1. Visit: {model_repo}");
            println!("2. Download: {model_filename}");
            println!("3. Copy the downloaded file to: {}", model_dir.display());
            println!();
            println!("Or use Ollama to download the model:");
            println!("   ollama pull qwen2.5-coder:1.5b");
            println!();
            return Err(anyhow::anyhow!(
                "Git clone failed. Please download the model manually or use Ollama."
            ));
        }

        // Initialize Git LFS
        println!("🔧 Initializing Git LFS...");
        let lfs_init_output = std::process::Command::new("git")
            .arg("lfs")
            .arg("install")
            .current_dir(&temp_dir)
            .output()
            .context("Failed to run git lfs install")?;

        if !lfs_init_output.status.success() {
            let stderr = String::from_utf8_lossy(&lfs_init_output.stderr);
            println!("⚠️  Git LFS install warning: {stderr}");
            // Continue anyway as LFS might already be installed
        }

        // Sparse checkout the specific model file
        println!("📥 Downloading model file: {model_filename}");
        let sparse_output = std::process::Command::new("git")
            .arg("sparse-checkout")
            .arg("set")
            .arg(model_filename)
            .current_dir(&temp_dir)
            .output()
            .context("Failed to run git sparse-checkout")?;

        if !sparse_output.status.success() {
            let stderr = String::from_utf8_lossy(&sparse_output.stderr);
            println!("❌ Sparse checkout failed: {stderr}");
            return Err(anyhow::anyhow!("Git sparse-checkout failed: {}", stderr));
        }

        let checkout_output = std::process::Command::new("git")
            .arg("checkout")
            .current_dir(&temp_dir)
            .output()
            .context("Failed to run git checkout")?;

        if !checkout_output.status.success() {
            let stderr = String::from_utf8_lossy(&checkout_output.stderr);
            println!("❌ Git checkout failed: {stderr}");
            return Err(anyhow::anyhow!("Git checkout failed: {}", stderr));
        }

        // Pull LFS files
        println!("📥 Pulling LFS files...");
        let lfs_pull_output = std::process::Command::new("git")
            .arg("lfs")
            .arg("pull")
            .current_dir(&temp_dir)
            .output()
            .context("Failed to run git lfs pull")?;

        if !lfs_pull_output.status.success() {
            let stderr = String::from_utf8_lossy(&lfs_pull_output.stderr);
            println!("⚠️  Git LFS pull warning: {stderr}");
            // Continue anyway as the file might already be downloaded
        }

        // Move the model file to the models directory
        let source_path = temp_dir.join(model_filename);
        if !source_path.exists() {
            println!(
                "❌ Model file not found after download: {}",
                source_path.display()
            );
            return Err(anyhow::anyhow!(
                "Model file not found after git clone: {}",
                source_path.display()
            ));
        }

        println!("📦 Moving model file to: {}", model_path.display());
        std::fs::copy(&source_path, &model_path).context("Failed to copy model file")?;

        // Clean up temporary directory
        std::fs::remove_dir_all(&temp_dir).context("Failed to remove temp directory")?;

        println!("✅ Model downloaded successfully using git clone!");
        Ok(model_path.to_string_lossy().to_string())
    }

    pub async fn ensure_model_downloaded(&self) -> Result<String> {
        let model_path = self.config.get_setting("model_path");
        if let Some(path) = model_path {
            if !path.is_empty() && std::path::Path::new(path).exists() {
                println!("✅ Model already downloaded at: {path}");
                return Ok(path.clone());
            }
        }

        // Get model name from configuration
        let model_name = self
            .config
            .get_setting_or_default("model", "Qwen2.5-Coder-1.5B");
        println!("📥 Checking {model_name} model...");

        // Create model directory
        let home_dir = dirs::home_dir().context("Failed to find home directory")?;
        let model_dir = home_dir.join(".terminalai").join("models");
        std::fs::create_dir_all(&model_dir).context("Failed to create model directory")?;

        // Determine model filename based on model name
        let model_filename = match model_name.as_str() {
            "Qwen2.5-Coder-1.5B" => "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-3B" => "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-7B" => "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-14B" => "qwen2.5-coder-14b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-32B" => "qwen2.5-coder-32b-instruct-q4_k_m.gguf",
            "Qwen2.5-Coder-72B" => "qwen2.5-coder-72b-instruct-q4_k_m.gguf",
            "Phi-3.5-Mini" => "phi-3.5-mini-4k-instruct.Q4_K_M.gguf",
            "Phi-3.5-Mini-128K" => "phi-3.5-mini-128k-instruct.Q4_K_M.gguf",
            "CodeLlama-3.8B" => "codellama-3.8b-instruct.Q4_K_M.gguf",
            "CodeLlama-7B" => "codellama-7b-instruct.Q4_K_M.gguf",
            _ => "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf", // Default fallback to Qwen2.5-Coder-1.5B
        };

        let model_path = model_dir.join(model_filename);

        if model_path.exists() {
            println!("✅ Model already exists at: {}", model_path.display());
            return Ok(model_path.to_string_lossy().to_string());
        }

        // Since Hugging Face requires authentication, try git clone as fallback
        println!("⚠️  Model download requires Hugging Face authentication.");
        println!("🚀 Attempting to download model using git clone...");

        // Get the model repository URL
        let model_repo = match model_name.as_str() {
            "Qwen2.5-Coder-1.5B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF",
            "Qwen2.5-Coder-3B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
            "Qwen2.5-Coder-7B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
            "Qwen2.5-Coder-14B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-14B-Instruct-GGUF",
            "Qwen2.5-Coder-32B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-32B-Instruct-GGUF",
            "Qwen2.5-Coder-72B" => "https://huggingface.co/Qwen/Qwen2.5-Coder-72B-Instruct-GGUF",
            "Phi-3.5-Mini" => "https://huggingface.co/TheBloke/Phi-3.5-Mini-4K-Instruct-GGUF",
            "Phi-3.5-Mini-128K" => {
                "https://huggingface.co/TheBloke/Phi-3.5-Mini-128K-Instruct-GGUF"
            }
            "CodeLlama-3.8B" => "https://huggingface.co/TheBloke/CodeLlama-3.8B-Instruct-GGUF",
            "CodeLlama-7B" => "https://huggingface.co/TheBloke/CodeLlama-7B-Instruct-GGUF",
            _ => "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF", // Default fallback
        };

        // Check if git-lfs is installed
        let lfs_check = std::process::Command::new("git")
            .arg("lfs")
            .arg("version")
            .output();

        if lfs_check.is_err() {
            println!("❌ Git LFS is not installed. Please install it first:");
            println!();
            println!("macOS (using Homebrew):");
            println!("   brew install git-lfs");
            println!();
            println!("Ubuntu/Debian:");
            println!("   sudo apt-get install git-lfs");
            println!();
            println!("Or visit: https://git-lfs.com");
            println!();
            println!("After installing git-lfs, run:");
            println!("   git lfs install");
            println!();
            return Err(anyhow::anyhow!(
                "Git LFS is not installed. Please install it first."
            ));
        }

        // Create a temporary directory for cloning
        let temp_dir = model_dir.join(format!("temp_{}", model_name.replace('/', "_")));
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)
                .context("Failed to remove existing temp directory")?;
        }

        // Clone the repository
        println!("📥 Cloning repository: {model_repo}");
        let clone_output = std::process::Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--filter=blob:none")
            .arg("--sparse")
            .arg(model_repo)
            .arg(&temp_dir)
            .output()
            .context("Failed to run git clone")?;

        if !clone_output.status.success() {
            let stderr = String::from_utf8_lossy(&clone_output.stderr);
            println!("❌ Git clone failed: {stderr}");
            println!("📋 Please download the model manually:");
            println!();
            println!("1. Visit: {model_repo}");
            println!("2. Download: {model_filename}");
            println!("3. Copy the downloaded file to: {}", model_dir.display());
            println!();
            println!("Or use Ollama to download the model:");
            println!("   ollama pull qwen2.5-coder:1.5b");
            println!();
            return Err(anyhow::anyhow!(
                "Git clone failed. Please download the model manually or use Ollama."
            ));
        }

        // Initialize Git LFS
        println!("🔧 Initializing Git LFS...");
        let lfs_init_output = std::process::Command::new("git")
            .arg("lfs")
            .arg("install")
            .current_dir(&temp_dir)
            .output()
            .context("Failed to run git lfs install")?;

        if !lfs_init_output.status.success() {
            let stderr = String::from_utf8_lossy(&lfs_init_output.stderr);
            println!("⚠️  Git LFS install warning: {stderr}");
            // Continue anyway as LFS might already be installed
        }

        // Sparse checkout the specific model file
        println!("📥 Downloading model file: {model_filename}");
        let sparse_output = std::process::Command::new("git")
            .arg("sparse-checkout")
            .arg("set")
            .arg(model_filename)
            .current_dir(&temp_dir)
            .output()
            .context("Failed to run git sparse-checkout")?;

        if !sparse_output.status.success() {
            let stderr = String::from_utf8_lossy(&sparse_output.stderr);
            println!("❌ Sparse checkout failed: {stderr}");
            return Err(anyhow::anyhow!("Git sparse-checkout failed: {}", stderr));
        }

        let checkout_output = std::process::Command::new("git")
            .arg("checkout")
            .current_dir(&temp_dir)
            .output()
            .context("Failed to run git checkout")?;

        if !checkout_output.status.success() {
            let stderr = String::from_utf8_lossy(&checkout_output.stderr);
            println!("❌ Git checkout failed: {stderr}");
            return Err(anyhow::anyhow!("Git checkout failed: {}", stderr));
        }

        // Pull LFS files
        println!("📥 Pulling LFS files...");
        let lfs_pull_output = std::process::Command::new("git")
            .arg("lfs")
            .arg("pull")
            .current_dir(&temp_dir)
            .output()
            .context("Failed to run git lfs pull")?;

        if !lfs_pull_output.status.success() {
            let stderr = String::from_utf8_lossy(&lfs_pull_output.stderr);
            println!("⚠️  Git LFS pull warning: {stderr}");
            // Continue anyway as the file might already be downloaded
        }

        // Move the model file to the models directory
        let source_path = temp_dir.join(model_filename);
        if !source_path.exists() {
            println!(
                "❌ Model file not found after download: {}",
                source_path.display()
            );
            return Err(anyhow::anyhow!(
                "Model file not found after git clone: {}",
                source_path.display()
            ));
        }

        println!("📦 Moving model file to: {}", model_path.display());
        std::fs::copy(&source_path, &model_path).context("Failed to copy model file")?;

        // Clean up temporary directory
        std::fs::remove_dir_all(&temp_dir).context("Failed to remove temp directory")?;

        println!("✅ Model downloaded successfully using git clone!");
        Ok(model_path.to_string_lossy().to_string())
    }
}

#[async_trait]
impl AIProvider for LocalProvider {
    async fn send_query(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        println!("🔧 Setting up local AI provider...");

        // Ensure llama.cpp is installed
        let llama_cpp_path = self.ensure_llama_cpp_installed()?;

        // Check for existing model first, only download if absolutely necessary
        let model_path = match self.get_existing_model_path() {
            Ok(path) => {
                println!("✅ Using existing model at: {path}");
                path
            }
            Err(_) => {
                println!("⚠️  No existing model found. This will require downloading a large model file.");
                println!("💡 Consider using Ollama instead for easier model management:");
                println!("   tai init");
                println!("   # Select Ollama provider");
                println!("   ollama pull qwen2.5-coder:1.5b");
                return Err(anyhow::anyhow!(
                    "No model found. Please set up a model or use a different provider."
                ));
            }
        };

        println!("🤖 Running local AI model...");

        // Prepare the prompt
        let combined_prompt = format!("{system_prompt}\n\nUser Request: {user_prompt}");

        // Run llama.cpp with optimized parameters
        let output = std::process::Command::new(&llama_cpp_path)
            .arg("-m")
            .arg(&model_path)
            .arg("-p")
            .arg(&combined_prompt)
            .arg("-n")
            .arg("512") // Max tokens
            .arg("-c")
            .arg("2048") // Context size
            .arg("-t")
            .arg("4") // Threads
            .arg("--temp")
            .arg("0.1") // Temperature
            .arg("--repeat-penalty")
            .arg("1.1") // Repeat penalty
            .output()
            .context("Failed to run llama.cpp")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("llama.cpp failed: {}", stderr));
        }

        let response = String::from_utf8_lossy(&output.stdout);
        Ok(response.trim().to_string())
    }

    fn provider_name(&self) -> &'static str {
        "Local (llama.cpp)"
    }

    fn validate_config(&self) -> Result<()> {
        // Local provider doesn't require specific settings for validation
        Ok(())
    }
}
