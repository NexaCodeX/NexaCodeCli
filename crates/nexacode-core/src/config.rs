//! Configuration management
//!
//! This module handles loading and saving configuration from:
//! - Config file: ~/.nexacode/config.toml
//! - Environment variables
//! - Default values

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use crate::data_dir::NexaCodeDir;
use crate::infra::llm::config::{LlmConfig, ModelInfo};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLM configuration
    pub llm: LlmConfig,
    /// UI configuration
    pub ui: UiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme: "dark" or "light"
    pub theme: String,
    /// Show line numbers in code blocks
    pub show_line_numbers: bool,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            show_line_numbers: true,
            auto_save_interval: 30,
        }
    }
}

impl Config {
    /// Create a new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the NexaCode data directory manager
    fn data_dir() -> NexaCodeDir {
        NexaCodeDir::new()
    }

    /// Get the config file path
    pub fn config_path() -> PathBuf {
        Self::data_dir().config_file()
    }

    /// Get the data directory root path
    pub fn data_directory() -> PathBuf {
        Self::data_dir().root().clone()
    }

    /// Load configuration from file, with fallback to defaults
    pub fn load() -> Self {
        let data_dir = Self::data_dir();
        
        // Ensure directories exist
        if let Err(e) = data_dir.ensure_dirs() {
            eprintln!("Warning: Failed to create data directories: {}", e);
        }
        
        let path = Self::config_path();
        
        if path.exists() {
            match Self::load_from_file(&path) {
                Ok(config) => {
                    // Merge with environment variables (env takes precedence)
                    let mut config = config;
                    config.merge_env();
                    return config;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load config file: {}", e);
                }
            }
        }

        // Config file doesn't exist - use the data_dir initialization
        // which creates a config with custom providers
        if let Err(e) = data_dir.initialize() {
            eprintln!("Warning: Failed to initialize data directory: {}", e);
        }
        
        // Now load the newly created config
        if path.exists() {
            match Self::load_from_file(&path) {
                Ok(config) => {
                    let mut config = config;
                    config.merge_env();
                    return config;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load newly created config: {}", e);
                }
            }
        }

        // Fallback to default config
        let mut config = Self::default();
        config.merge_env();
        
        config
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
        
        Ok(config)
    }

    /// Save configuration to the default file
    pub fn save(&self) -> Result<()> {
        // Ensure directories exist
        Self::data_dir().ensure_dirs()?;
        
        let path = Self::config_path();
        self.save_to_file(&path)
    }

    /// Save configuration to a specific file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        
        Ok(())
    }

    /// Merge environment variables into config
    pub fn merge_env(&mut self) {
        // Provider selection
        if let Ok(provider) = std::env::var("NEXACODE_PROVIDER") {
            self.llm.default_provider = provider;
        }

        // Anthropic config from env
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            self.llm.anthropic.api_key = api_key;
        }
        if let Ok(base_url) = std::env::var("ANTHROPIC_BASE_URL") {
            self.llm.anthropic.base_url = base_url;
        }

        // OpenAI config from env
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            self.llm.openai.api_key = api_key;
        }
        if let Ok(base_url) = std::env::var("OPENAI_BASE_URL") {
            self.llm.openai.base_url = base_url;
        }

        // General LLM config from env
        if let Ok(max_tokens) = std::env::var("NEXACODE_MAX_TOKENS") {
            if let Ok(val) = max_tokens.parse() {
                self.llm.max_tokens = val;
            }
        }
        if let Ok(temperature) = std::env::var("NEXACODE_TEMPERATURE") {
            if let Ok(val) = temperature.parse() {
                self.llm.temperature = val;
            }
        }
        
        // UI config from env
        if let Ok(theme) = std::env::var("NEXACODE_THEME") {
            self.ui.theme = theme;
        }
    }

    /// Set the provider
    pub fn set_provider(&mut self, provider: impl Into<String>) {
        self.llm.set_provider(provider);
    }

    /// Set the model
    pub fn set_model(&mut self, model: &str) {
        self.llm.set_model(model);
    }

    /// Set the API key for current provider
    pub fn set_api_key(&mut self, api_key: impl Into<String>) {
        let provider = self.llm.default_provider.to_lowercase();
        let api_key = api_key.into();
        
        match provider.as_str() {
            "anthropic" => self.llm.anthropic.api_key = api_key,
            "openai" => self.llm.openai.api_key = api_key,
            _ => {
                if let Some(custom) = self.llm.providers.get_mut(&self.llm.default_provider) {
                    custom.api_key = api_key;
                }
            }
        }
    }

    /// Set the theme
    pub fn set_theme(&mut self, theme: impl Into<String>) {
        self.ui.theme = theme.into();
    }

    /// Check if current provider has API key configured
    pub fn has_api_key(&self) -> bool {
        self.llm.has_api_key()
    }

    /// Get current provider name
    pub fn current_provider(&self) -> &str {
        self.llm.current_provider_name()
    }

    /// Get current model name
    pub fn current_model(&self) -> String {
        self.llm.current_model()
    }

    /// Get current model display name
    pub fn current_model_display(&self) -> String {
        self.llm.current_model_display()
    }

    /// Get a list of all available providers
    pub fn available_providers() -> Vec<String> {
        LlmConfig::default().available_providers()
    }

    /// Get available providers from config
    pub fn providers(&self) -> Vec<String> {
        self.llm.available_providers()
    }

    /// Get a list of configured models (from actual config)
    pub fn configured_models(&self) -> Vec<ModelInfo> {
        self.llm.configured_models()
    }

    /// Get a list of available models (deprecated - use configured_models)
    #[deprecated(note = "Use configured_models() instead")]
    pub fn available_models() -> Vec<ModelInfo> {
        LlmConfig::all_models()
    }

    /// Format config for display
    pub fn to_display_string(&self) -> String {
        let api_key_display = if self.llm.current_api_key().is_empty() {
            "Not set".to_string()
        } else {
            // Show only first 10 chars
            format!("{}...", &self.llm.current_api_key().chars().take(10).collect::<String>())
        };

        format!(
            r#"Configuration

LLM Settings:
  Provider:    {}
  Model:       {}
  API Key:     {}
  Base URL:    {}
  Max Tokens:  {}
  Temperature: {}
  Timeout:     {}s

UI Settings:
  Theme:            {}
  Line Numbers:     {}
  Auto-save:        {}s

Config File: {}
"#,
            self.llm.current_provider_name(),
            self.llm.current_model_display(),
            api_key_display,
            self.llm.current_base_url(),
            self.llm.max_tokens,
            self.llm.temperature,
            self.llm.timeout_secs,
            self.ui.theme,
            self.ui.show_line_numbers,
            self.ui.auto_save_interval,
            Self::config_path().display(),
        )
    }
}

/// Generate a default config file content
pub fn generate_default_config() -> String {
    let config = Config::default();
    toml::to_string_pretty(&config).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::llm::config::{CustomProviderConfig, ProviderType};

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.llm.default_provider, "openai");
        assert_eq!(config.ui.theme, "dark");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.llm.default_provider, parsed.llm.default_provider);
    }

    #[test]
    fn test_custom_provider() {
        let mut config = Config::default();
        
        config.llm.add_provider("CLIProxyAPI", CustomProviderConfig {
            r#type: ProviderType::OpenAI,
            api_key: "test-key".to_string(),
            model: "glm-5".to_string(),
            base_url: "http://127.0.0.1:8317/v1".to_string(),
            context_window: Some(16000),
        });
        
        assert!(config.llm.providers.contains_key("CLIProxyAPI"));
    }

    #[test]
    fn test_load_config_with_custom_providers() {
        use tempfile::TempDir;
        
        // Create a temp config file with custom providers
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let config_content = r#"
[llm]
default_provider = "CLIProxyAPI"
max_tokens = 4096
temperature = 1.0
timeout_secs = 60

[llm.anthropic]
api_key = ""
model = "claude37-sonnet"
base_url = "https://api.anthropic.com"

[llm.openai]
api_key = ""
model = "gpt-4o"
base_url = "https://api.openai.com/v1"

[llm.providers.CLIProxyAPI]
type = "openai"
api_key = "test-key"
model = "glm-5"
base_url = "http://127.0.0.1:8317/v1"
context_window = 16000

[llm.providers.HuaWei]
type = "openai"
api_key = "hw-key"
model = "glm-5"
base_url = "https://api.modelarts-maas.com/v2"
context_window = 16000

[ui]
theme = "dark"
show_line_numbers = true
auto_save_interval = 30
"#;
        
        std::fs::write(&config_path, config_content).unwrap();
        
        // Load the config
        let config = Config::load_from_file(&config_path).unwrap();
        
        // Verify providers loaded
        println!("Loaded providers: {:?}", config.llm.providers.keys().collect::<Vec<_>>());
        assert!(config.llm.providers.contains_key("CLIProxyAPI"));
        assert!(config.llm.providers.contains_key("HuaWei"));
        
        // Verify configured models
        let models = config.configured_models();
        println!("Configured models: {:?}", models);
        assert_eq!(models.len(), 2);
        
        // Verify current provider
        assert_eq!(config.current_provider(), "CLIProxyAPI");
        assert_eq!(config.current_model(), "glm-5");
    }
}
