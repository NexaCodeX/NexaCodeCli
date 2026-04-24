//! LLM configuration
//!
//! Supports multiple LLM providers with custom configurations.
//! Built-in providers: Anthropic (Claude) and OpenAI.
//! Custom providers: Any OpenAI-compatible API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Provider Types
// ============================================================================

/// LLM Provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// Anthropic Claude API
    Anthropic,
    /// OpenAI GPT API (or compatible)
    #[default]
    OpenAI,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic => write!(f, "anthropic"),
            Self::OpenAI => write!(f, "openai"),
        }
    }
}

// ============================================================================
// Custom Provider Configuration
// ============================================================================

/// Custom provider configuration (OpenAI-compatible APIs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProviderConfig {
    /// Provider type (currently only "openai" for compatibility)
    #[serde(default = "default_provider_type")]
    pub r#type: ProviderType,
    /// API key (can be empty if using local proxy)
    #[serde(default)]
    pub api_key: String,
    /// Model name to use
    pub model: String,
    /// Base URL for the API
    pub base_url: String,
    /// Context window size (optional)
    #[serde(default)]
    pub context_window: Option<u32>,
}

fn default_provider_type() -> ProviderType {
    ProviderType::OpenAI
}

// ============================================================================
// Built-in Anthropic Models
// ============================================================================

/// Available Anthropic Claude models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AnthropicModel {
    /// Claude 3.7 Sonnet (latest)
    #[default]
    Claude37Sonnet,
    /// Claude 3.5 Sonnet
    Claude35Sonnet,
    /// Claude 3.5 Haiku
    Claude35Haiku,
    /// Claude 3 Opus
    Claude3Opus,
    /// Claude 3 Sonnet
    Claude3Sonnet,
    /// Claude 3 Haiku
    Claude3Haiku,
}

impl AnthropicModel {
    /// Get the API model name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude37Sonnet => "claude-3-7-sonnet-20250219",
            Self::Claude35Sonnet => "claude-3-5-sonnet-20241022",
            Self::Claude35Haiku => "claude-3-5-haiku-20241022",
            Self::Claude3Opus => "claude-3-opus-20240229",
            Self::Claude3Sonnet => "claude-3-sonnet-20240229",
            Self::Claude3Haiku => "claude-3-haiku-20240307",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Claude37Sonnet => "Claude 3.7 Sonnet",
            Self::Claude35Sonnet => "Claude 3.5 Sonnet",
            Self::Claude35Haiku => "Claude 3.5 Haiku",
            Self::Claude3Opus => "Claude 3 Opus",
            Self::Claude3Sonnet => "Claude 3 Sonnet",
            Self::Claude3Haiku => "Claude 3 Haiku",
        }
    }

    /// Get all available models
    pub fn all() -> &'static [AnthropicModel] {
        &[
            Self::Claude37Sonnet,
            Self::Claude35Sonnet,
            Self::Claude35Haiku,
            Self::Claude3Opus,
            Self::Claude3Sonnet,
            Self::Claude3Haiku,
        ]
    }
}

// ============================================================================
// Built-in OpenAI Models
// ============================================================================

/// Available OpenAI models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum OpenAIModel {
    /// GPT-4o (latest)
    #[default]
    Gpt4o,
    /// GPT-4o Mini
    Gpt4oMini,
    /// GPT-4 Turbo
    Gpt4Turbo,
    /// GPT-4
    Gpt4,
    /// GPT-3.5 Turbo
    Gpt35Turbo,
    /// o1 (reasoning model)
    O1,
    /// o1-mini
    O1Mini,
    /// o3-mini
    O3Mini,
}

impl OpenAIModel {
    /// Get the API model name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gpt4o => "gpt-4o",
            Self::Gpt4oMini => "gpt-4o-mini",
            Self::Gpt4Turbo => "gpt-4-turbo",
            Self::Gpt4 => "gpt-4",
            Self::Gpt35Turbo => "gpt-3.5-turbo",
            Self::O1 => "o1",
            Self::O1Mini => "o1-mini",
            Self::O3Mini => "o3-mini",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Gpt4o => "GPT-4o",
            Self::Gpt4oMini => "GPT-4o Mini",
            Self::Gpt4Turbo => "GPT-4 Turbo",
            Self::Gpt4 => "GPT-4",
            Self::Gpt35Turbo => "GPT-3.5 Turbo",
            Self::O1 => "o1",
            Self::O1Mini => "o1-mini",
            Self::O3Mini => "o3-mini",
        }
    }

    /// Get all available models
    pub fn all() -> &'static [OpenAIModel] {
        &[
            Self::Gpt4o,
            Self::Gpt4oMini,
            Self::Gpt4Turbo,
            Self::Gpt4,
            Self::Gpt35Turbo,
            Self::O1,
            Self::O1Mini,
            Self::O3Mini,
        ]
    }
}

// ============================================================================
// Built-in Provider Configuration
// ============================================================================

/// Anthropic-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    /// API key
    #[serde(default)]
    pub api_key: String,
    /// Model to use
    #[serde(default)]
    pub model: AnthropicModel,
    /// Base URL (for custom endpoints)
    #[serde(default = "default_anthropic_base_url")]
    pub base_url: String,
}

fn default_anthropic_base_url() -> String {
    "https://api.anthropic.com".to_string()
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: AnthropicModel::default(),
            base_url: default_anthropic_base_url(),
        }
    }
}

/// OpenAI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// API key
    #[serde(default)]
    pub api_key: String,
    /// Model to use (string for flexibility)
    #[serde(default = "default_openai_model")]
    pub model: String,
    /// Base URL (for custom endpoints or Azure)
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,
    /// Organization ID (optional)
    #[serde(default)]
    pub organization: Option<String>,
}

fn default_openai_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_openai_model() -> String {
    "gpt-4o".to_string()
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: default_openai_model(),
            base_url: default_openai_base_url(),
            organization: None,
        }
    }
}

// ============================================================================
// Unified LLM Configuration
// ============================================================================

/// Unified LLM configuration supporting multiple providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Default provider name (references built-in or custom provider)
    #[serde(default = "default_provider")]
    pub default_provider: String,
    
    /// Built-in Anthropic configuration
    #[serde(default)]
    pub anthropic: AnthropicConfig,
    
    /// Built-in OpenAI configuration
    #[serde(default)]
    pub openai: OpenAIConfig,
    
    /// Custom providers (name -> config)
    #[serde(default)]
    pub providers: HashMap<String, CustomProviderConfig>,
    
    /// Maximum tokens in response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    
    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_provider() -> String {
    "openai".to_string()
}
fn default_max_tokens() -> u32 { 4096 }
fn default_temperature() -> f32 { 1.0 }
fn default_timeout() -> u64 { 60 }

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
            anthropic: AnthropicConfig::default(),
            openai: OpenAIConfig::default(),
            providers: HashMap::new(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            timeout_secs: default_timeout(),
        }
    }
}

impl LlmConfig {
    /// Create a new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Determine provider from env
        if let Ok(provider) = std::env::var("NEXACODE_PROVIDER") {
            config.default_provider = provider;
        }

        // Anthropic config
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            config.anthropic.api_key = key;
        }
        if let Ok(base_url) = std::env::var("ANTHROPIC_BASE_URL") {
            config.anthropic.base_url = base_url;
        }

        // OpenAI config
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            config.openai.api_key = key;
        }
        if let Ok(base_url) = std::env::var("OPENAI_BASE_URL") {
            config.openai.base_url = base_url;
        }
        if let Ok(org) = std::env::var("OPENAI_ORG_ID") {
            config.openai.organization = Some(org);
        }

        // General config
        if let Ok(max_tokens) = std::env::var("NEXACODE_MAX_TOKENS") {
            if let Ok(val) = max_tokens.parse() {
                config.max_tokens = val;
            }
        }
        if let Ok(temperature) = std::env::var("NEXACODE_TEMPERATURE") {
            if let Ok(val) = temperature.parse() {
                config.temperature = val;
            }
        }

        config
    }

    /// Get the current provider name
    pub fn current_provider_name(&self) -> &str {
        &self.default_provider
    }

    /// Get the current model name (API format)
    pub fn current_model(&self) -> String {
        let provider = self.default_provider.to_lowercase();
        
        // Check built-in providers first
        match provider.as_str() {
            "anthropic" => self.anthropic.model.as_str().to_string(),
            "openai" => self.openai.model.clone(),
            _ => {
                // Look for custom provider
                if let Some(custom) = self.providers.get(&self.default_provider) {
                    custom.model.clone()
                } else {
                    self.openai.model.clone()
                }
            }
        }
    }

    /// Get the current model display name
    pub fn current_model_display(&self) -> String {
        self.current_model()
    }

    /// Get the current API key
    pub fn current_api_key(&self) -> String {
        let provider = self.default_provider.to_lowercase();
        
        match provider.as_str() {
            "anthropic" => self.anthropic.api_key.clone(),
            "openai" => self.openai.api_key.clone(),
            _ => {
                if let Some(custom) = self.providers.get(&self.default_provider) {
                    custom.api_key.clone()
                } else {
                    String::new()
                }
            }
        }
    }

    /// Get the current base URL
    pub fn current_base_url(&self) -> String {
        let provider = self.default_provider.to_lowercase();
        
        match provider.as_str() {
            "anthropic" => self.anthropic.base_url.clone(),
            "openai" => self.openai.base_url.clone(),
            _ => {
                if let Some(custom) = self.providers.get(&self.default_provider) {
                    custom.base_url.clone()
                } else {
                    default_openai_base_url()
                }
            }
        }
    }

    /// Check if current provider has API key configured
    pub fn has_api_key(&self) -> bool {
        !self.current_api_key().is_empty()
    }

    /// Set the default provider
    pub fn set_provider(&mut self, provider: impl Into<String>) {
        self.default_provider = provider.into();
    }

    /// Set the model by name
    pub fn set_model(&mut self, model: &str) {
        // Update the current provider's model
        let provider = self.default_provider.to_lowercase();
        
        match provider.as_str() {
            "anthropic" => {
                if let Ok(m) = parse_anthropic_model(model) {
                    self.anthropic.model = m;
                }
            }
            "openai" => {
                self.openai.model = model.to_string();
            }
            _ => {
                if let Some(custom) = self.providers.get_mut(&self.default_provider) {
                    custom.model = model.to_string();
                }
            }
        }
    }

    /// Add a custom provider
    pub fn add_provider(&mut self, name: impl Into<String>, config: CustomProviderConfig) {
        self.providers.insert(name.into(), config);
    }

    /// Get list of all available provider names
    pub fn available_providers(&self) -> Vec<String> {
        let mut providers = Vec::new();
        
        // Only add anthropic if it has an API key configured
        if !self.anthropic.api_key.is_empty() {
            providers.push("anthropic".to_string());
        }
        
        // Only add openai if it has an API key configured
        if !self.openai.api_key.is_empty() {
            providers.push("openai".to_string());
        }
        
        // Add all custom providers
        for name in self.providers.keys() {
            providers.push(name.clone());
        }
        
        providers
    }

    /// Get list of configured models (from all providers)
    /// Returns a list of (provider_name, model_name) tuples
    pub fn configured_models(&self) -> Vec<ModelInfo> {
        let mut models = Vec::new();
        
        // Add anthropic model if configured
        if !self.anthropic.api_key.is_empty() {
            models.push(ModelInfo {
                id: self.anthropic.model.as_str().to_string(),
                display_name: self.anthropic.model.display_name().to_string(),
                provider: "anthropic".to_string(),
            });
        }
        
        // Add openai model if configured
        if !self.openai.api_key.is_empty() {
            models.push(ModelInfo {
                id: self.openai.model.clone(),
                display_name: self.openai.model.clone(),
                provider: "openai".to_string(),
            });
        }
        
        // Add all custom provider models
        for (name, config) in &self.providers {
            models.push(ModelInfo {
                id: config.model.clone(),
                display_name: config.model.clone(),
                provider: name.clone(),
            });
        }
        
        models
    }

    /// Get all available models (from all providers)
    pub fn all_models() -> Vec<ModelInfo> {
        let mut models = Vec::new();

        for m in AnthropicModel::all() {
            models.push(ModelInfo {
                id: m.as_str().to_string(),
                display_name: m.display_name().to_string(),
                provider: "anthropic".to_string(),
            });
        }

        for m in OpenAIModel::all() {
            models.push(ModelInfo {
                id: m.as_str().to_string(),
                display_name: m.display_name().to_string(),
                provider: "openai".to_string(),
            });
        }

        models
    }

    /// Get provider type
    pub fn provider_type(&self) -> ProviderType {
        let provider = self.default_provider.to_lowercase();
        
        match provider.as_str() {
            "anthropic" => ProviderType::Anthropic,
            "openai" => ProviderType::OpenAI,
            _ => {
                if let Some(custom) = self.providers.get(&self.default_provider) {
                    custom.r#type
                } else {
                    ProviderType::OpenAI
                }
            }
        }
    }
}

/// Model information for display
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub provider: String,
}

impl std::fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.display_name, self.provider)
    }
}

impl PartialEq for ModelInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialEq<str> for ModelInfo {
    fn eq(&self, other: &str) -> bool {
        self.id == other
    }
}

// ============================================================================
// Parsing Helpers
// ============================================================================

/// Parse a string as an Anthropic model
pub fn parse_anthropic_model(s: &str) -> Result<AnthropicModel, String> {
    let s_lower = s.to_lowercase();
    for model in AnthropicModel::all() {
        if model.as_str() == s_lower || model.display_name().to_lowercase() == s_lower {
            return Ok(*model);
        }
    }
    Err(format!("Unknown Anthropic model: {}", s))
}

/// Parse a string as an OpenAI model
pub fn parse_openai_model(s: &str) -> Result<OpenAIModel, String> {
    let s_lower = s.to_lowercase();
    for model in OpenAIModel::all() {
        if model.as_str() == s_lower || model.display_name().to_lowercase() == s_lower {
            return Ok(*model);
        }
    }
    Err(format!("Unknown OpenAI model: {}", s))
}

// ============================================================================
// Legacy compatibility
// ============================================================================

/// Legacy LlmProvider enum for backwards compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    #[default]
    Anthropic,
    OpenAI,
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic => write!(f, "Anthropic"),
            Self::OpenAI => write!(f, "OpenAI"),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_model_str() {
        assert_eq!(AnthropicModel::Claude37Sonnet.as_str(), "claude-3-7-sonnet-20250219");
        assert_eq!(AnthropicModel::Claude35Sonnet.as_str(), "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_openai_model_str() {
        assert_eq!(OpenAIModel::Gpt4o.as_str(), "gpt-4o");
        assert_eq!(OpenAIModel::Gpt4Turbo.as_str(), "gpt-4-turbo");
    }

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert_eq!(config.default_provider, "openai");
    }

    #[test]
    fn test_custom_provider() {
        let mut config = LlmConfig::default();
        
        config.add_provider("CLIProxyAPI", CustomProviderConfig {
            r#type: ProviderType::OpenAI,
            api_key: "sk-cliproxyapi-local".to_string(),
            model: "glm-5".to_string(),
            base_url: "http://127.0.0.1:8317/v1".to_string(),
            context_window: Some(16000),
        });
        
        assert!(config.providers.contains_key("CLIProxyAPI"));
        // Only CLIProxyAPI has an API key, so only 1 provider is available
        assert_eq!(config.available_providers().len(), 1);
        
        // Add API key to openai - now should have 2 providers
        config.openai.api_key = "test-key".to_string();
        assert_eq!(config.available_providers().len(), 2);
    }

    #[test]
    fn test_set_provider() {
        let mut config = LlmConfig::default();
        
        config.set_provider("anthropic");
        assert_eq!(config.current_provider_name(), "anthropic");
        
        config.set_provider("CLIProxyAPI");
        assert_eq!(config.current_provider_name(), "CLIProxyAPI");
    }

    #[test]
    fn test_serialization() {
        let config = LlmConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: LlmConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.default_provider, parsed.default_provider);
    }

    #[test]
    fn print_config_format() {
        let config = LlmConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        println!("\n--- Default LLM Config Format ---\n{}", toml_str);
    }

    #[test]
    fn test_parse_nested_providers() {
        // Test parsing TOML with nested provider syntax
        let toml_str = r#"
default_provider = "CLIProxyAPI"

[providers.CLIProxyAPI]
type = "openai"
api_key = "test-key"
model = "glm-5"
base_url = "http://127.0.0.1:8317/v1"

[providers.HuaWei]
type = "openai"
api_key = "hw-key"
model = "glm-5"
base_url = "https://api.modelarts-maas.com/v2"
"#;

        let config: LlmConfig = toml::from_str(toml_str).unwrap();
        println!("Parsed providers: {:?}", config.providers);
        println!("Providers count: {}", config.providers.len());
        
        assert!(config.providers.contains_key("CLIProxyAPI"));
        assert!(config.providers.contains_key("HuaWei"));
        assert_eq!(config.providers.len(), 2);
        
        // Test configured_models returns custom provider models
        let models = config.configured_models();
        println!("Configured models: {:?}", models);
        assert_eq!(models.len(), 2);
    }
}
