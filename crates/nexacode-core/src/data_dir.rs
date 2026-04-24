//! Data Directory Management
//!
//! This module manages the NexaCode data directory structure:
//! ```text
//! ~/.nexacode/
//! ├── config.toml         # Application configuration
//! ├── sessions/
//! │   ├── default.json    # Default session
//! │   └── sess_*.json     # Saved sessions
//! ├── cache/
//! │   └── ...             # Cache files
//! └── logs/
//!     └── nexacode.log    # Application logs
//! ```

use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use tracing::{info, debug};
use crate::config::Config;
use crate::infra::llm::config::{CustomProviderConfig, ProviderType};

/// NexaCode data directory manager
pub struct NexaCodeDir {
    /// Root directory path
    root: PathBuf,
}

impl NexaCodeDir {
    /// Create a new NexaCodeDir instance
    pub fn new() -> Self {
        let root = Self::get_root_dir();
        Self { root }
    }

    /// Get the root directory path (~/.nexacode)
    pub fn get_root_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".nexacode")
    }

    /// Check if the data directory exists
    pub fn exists(&self) -> bool {
        self.root.exists()
    }

    /// Check if this is a first-time run (directory doesn't exist)
    pub fn is_first_run(&self) -> bool {
        !self.root.exists()
    }

    /// Initialize the data directory for first-time setup
    /// This creates all necessary directories and a default config file
    pub fn initialize(&self) -> Result<()> {
        info!("Initializing NexaCode data directory at {:?}", self.root);

        // Create all directories
        self.ensure_dirs()?;

        // Create default config if it doesn't exist
        let config_file = self.config_file();
        if !config_file.exists() {
            info!("Creating default configuration file");
            self.create_default_config()?;
        }

        // Create a default session file placeholder if it doesn't exist
        let default_session = self.default_session_file();
        if !default_session.exists() {
            debug!("Creating default session placeholder");
            self.create_default_session()?;
        }

        info!("NexaCode data directory initialized successfully");
        Ok(())
    }

    /// Create the default configuration file
    fn create_default_config(&self) -> Result<()> {
        // Use programmatic config generation to avoid string formatting issues
        let mut config = Config::default();
        
        // Set default provider
        config.llm.default_provider = "CLIProxyAPI".to_string();
        
        // Add custom providers
        config.llm.add_provider("CLIProxyAPI", CustomProviderConfig {
            r#type: ProviderType::OpenAI,
            api_key: "sk-cliproxyapi-local".to_string(),
            model: "glm-5".to_string(),
            base_url: "http://127.0.0.1:8317/v1".to_string(),
            context_window: Some(16000),
        });
        
        config.llm.add_provider("HuaWei", CustomProviderConfig {
            r#type: ProviderType::OpenAI,
            api_key: "xdyoRB5FO29f2OCRGZhsyznjhmTSDeIsB36wjZoLYM94rY8gmx2T4qmaesTOTsrYVkcsfbZ0W1nzI81AzKZY3A".to_string(),
            model: "glm-5".to_string(),
            base_url: "https://api.modelarts-maas.com/v2".to_string(),
            context_window: Some(16000),
        });
        
        config.llm.add_provider("OpenRouter", CustomProviderConfig {
            r#type: ProviderType::OpenAI,
            api_key: "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".to_string(),
            model: "z-ai/glm-5".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            context_window: Some(16000),
        });
        
        config.llm.add_provider("AtomGit-MiniMax-M2.7", CustomProviderConfig {
            r#type: ProviderType::OpenAI,
            api_key: String::new(),
            model: "MiniMax-M2.7".to_string(),
            base_url: "https://api-ai.gitcode.com/v1".to_string(),
            context_window: Some(64000),
        });
        
        config.llm.add_provider("AtomGit-Qwen3.5", CustomProviderConfig {
            r#type: ProviderType::OpenAI,
            api_key: String::new(),
            model: "Qwen/Qwen3.5-122B-A10B".to_string(),
            base_url: "https://api-ai.gitcode.com/v1".to_string(),
            context_window: Some(64000),
        });
        
        config.llm.add_provider("AtomGit-Qwen3.6", CustomProviderConfig {
            r#type: ProviderType::OpenAI,
            api_key: String::new(),
            model: "Qwen/Qwen3.6-35B-A3B".to_string(),
            base_url: "https://api-ai.gitcode.com/v1".to_string(),
            context_window: Some(64000),
        });

        // Generate TOML with header comment
        let toml_content = toml::to_string_pretty(&config)
            .with_context(|| "Failed to serialize config")?;
        
        let content = format!(
            "# NexaCode Configuration\n# \n# This file contains your NexaCode settings.\n# Edit this file to customize your experience.\n\n{}",
            toml_content
        );

        fs::write(self.config_file(), content)
            .with_context(|| "Failed to create default config file")?;

        Ok(())
    }

    /// Create the default session file
    fn create_default_session(&self) -> Result<()> {
        let default_session = r#"{
  "id": "default",
  "messages": [],
  "created_at": null,
  "updated_at": null
}"#;

        fs::write(self.default_session_file(), default_session)
            .with_context(|| "Failed to create default session file")?;

        Ok(())
    }

    /// Ensure all directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.root)
            .with_context(|| format!("Failed to create root directory: {:?}", self.root))?;
        
        fs::create_dir_all(self.sessions_dir())
            .with_context(|| format!("Failed to create sessions directory"))?;
        
        fs::create_dir_all(self.cache_dir())
            .with_context(|| format!("Failed to create cache directory"))?;
        
        fs::create_dir_all(self.logs_dir())
            .with_context(|| format!("Failed to create logs directory"))?;
        
        Ok(())
    }

    // ========================================
    // Directory Paths
    // ========================================

    /// Get the sessions directory
    pub fn sessions_dir(&self) -> PathBuf {
        self.root.join("sessions")
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    /// Get the logs directory
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    // ========================================
    // Config Paths
    // ========================================

    /// Get the config file path (directly in root)
    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    // ========================================
    // Session Paths
    // ========================================

    /// Get the default session file path
    pub fn default_session_file(&self) -> PathBuf {
        self.sessions_dir().join("default.json")
    }

    /// Get a session file path by ID
    pub fn session_file(&self, session_id: &str) -> PathBuf {
        self.sessions_dir().join(format!("{}.json", session_id))
    }

    /// List all session files
    pub fn list_session_files(&self) -> Result<Vec<PathBuf>> {
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&sessions_dir)
            .with_context(|| format!("Failed to read sessions directory"))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                files.push(path);
            }
        }
        
        // Sort by modification time (newest first)
        files.sort_by(|a, b| {
            let time_a = a.metadata().and_then(|m| m.modified()).ok();
            let time_b = b.metadata().and_then(|m| m.modified()).ok();
            time_b.cmp(&time_a)
        });
        
        Ok(files)
    }

    // ========================================
    // Cache Paths
    // ========================================

    /// Get a cache file path
    pub fn cache_file(&self, name: &str) -> PathBuf {
        self.cache_dir().join(name)
    }

    /// Clear all cache files
    pub fn clear_cache(&self) -> Result<()> {
        let cache_dir = self.cache_dir();
        if cache_dir.exists() {
            for entry in fs::read_dir(&cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        Ok(())
    }

    // ========================================
    // Log Paths
    // ========================================

    /// Get the main log file path
    pub fn log_file(&self) -> PathBuf {
        self.logs_dir().join("nexacode.log")
    }

    // ========================================
    // Utility
    // ========================================

    /// Get the root directory path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get the total size of the data directory in bytes
    pub fn total_size(&self) -> Result<u64> {
        let mut total = 0u64;
        self.calculate_dir_size(&self.root, &mut total)?;
        Ok(total)
    }

    fn calculate_dir_size(&self, dir: &PathBuf, total: &mut u64) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.calculate_dir_size(&path, total)?;
            } else {
                *total += entry.metadata()?.len();
            }
        }
        Ok(())
    }

    /// Display directory info
    pub fn to_display_string(&self) -> String {
        let size = self.total_size().unwrap_or(0);
        let size_mb = size as f64 / (1024.0 * 1024.0);
        
        format!(
            r#"NexaCode Data Directory

Location: {}

Structure:
  config.toml  - Configuration file
  sessions/    - Saved conversation sessions
  cache/       - Temporary cache files
  logs/        - Application logs

Total Size: {:.2} MB

Sessions: {} saved
"#,
            self.root.display(),
            size_mb,
            self.list_session_files().unwrap_or_default().len(),
        )
    }
}

impl Default for NexaCodeDir {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_dir_paths() {
        let dir = NexaCodeDir::new();
        assert!(dir.root().ends_with(".nexacode"));
        assert!(dir.config_file().ends_with("config.toml"));
        assert!(dir.sessions_dir().ends_with("sessions"));
    }

    #[test]
    fn test_is_first_run() {
        // Use a temp directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join(".nexacode-test");
        
        let mut dir = NexaCodeDir::new();
        dir.root = test_path.clone();
        
        // Should be first run if directory doesn't exist
        assert!(dir.is_first_run());
        assert!(!dir.exists());
        
        // After initialization, should not be first run
        dir.initialize().unwrap();
        assert!(!dir.is_first_run());
        assert!(dir.exists());
        
        // Check that config file was created
        assert!(dir.config_file().exists());
        
        // Check that directories were created
        assert!(dir.sessions_dir().exists());
        assert!(dir.cache_dir().exists());
        assert!(dir.logs_dir().exists());
    }

    #[test]
    fn test_initialize_creates_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join(".nexacode-init-test");
        
        let mut dir = NexaCodeDir::new();
        dir.root = test_path;
        
        dir.initialize().unwrap();
        
        // Verify config file exists and has content
        let config_path = dir.config_file();
        assert!(config_path.exists());
        
        let config_content = fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("[llm]"));
        assert!(config_content.contains("default_provider"));
        assert!(config_content.contains("[llm.anthropic]"));
        assert!(config_content.contains("[llm.openai]"));
        assert!(config_content.contains("[llm.providers.CLIProxyAPI]"));
        assert!(config_content.contains("[ui]"));
        
        // Verify we can parse the config
        let config: Config = toml::from_str(&config_content).unwrap();
        assert!(config.llm.providers.contains_key("CLIProxyAPI"));
        assert!(config.llm.providers.contains_key("HuaWei"));
        assert_eq!(config.llm.providers.len(), 6);
    }
}
