//! Tool Execution Sandbox
//!
//! This module implements a secure sandbox for tool execution including:
//! - Security checks (path validation, command restrictions)
//! - Timeout control
//! - Output capture
//! - Change tracking

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use super::tools::{ToolResult, ToolExecutor, Tool};
use async_trait::async_trait;

// ============================================================================
// Security Configuration
// ============================================================================

/// Security configuration for sandbox
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum command execution time
    pub command_timeout: Duration,
    /// Maximum file size to read (bytes)
    pub max_file_size: u64,
    /// Allowed file extensions (empty = all allowed)
    pub allowed_extensions: Vec<String>,
    /// Blocked commands/patterns
    pub blocked_patterns: Vec<String>,
    /// Allow network access
    pub allow_network: bool,
    /// Allow environment variable access
    pub allow_env: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            command_timeout: Duration::from_secs(30),
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_extensions: vec![],
            blocked_patterns: vec![
                "rm -rf".to_string(),
                "sudo".to_string(),
                "chmod".to_string(),
                "chown".to_string(),
                "> /dev/".to_string(),
                "mkfs".to_string(),
                "dd if=".to_string(),
                ":(){ :|:& };:".to_string(), // Fork bomb
            ],
            allow_network: false,
            allow_env: false,
        }
    }
}

// ============================================================================
// Security Checker
// ============================================================================

/// Security checker for validating operations
pub struct SecurityChecker {
    config: SecurityConfig,
    workspace_root: PathBuf,
}

impl SecurityChecker {
    /// Create a new security checker
    pub fn new(workspace_root: PathBuf, config: SecurityConfig) -> Self {
        Self { config, workspace_root }
    }

    /// Check if a path is within the workspace
    pub fn is_path_allowed(&self, path: &Path) -> Result<(), String> {
        let canonical_path = path.canonicalize()
            .map_err(|e| format!("Cannot resolve path: {}", e))?;
        
        let canonical_root = self.workspace_root.canonicalize()
            .map_err(|e| format!("Cannot resolve workspace root: {}", e))?;
        
        if !canonical_path.starts_with(&canonical_root) {
            return Err(format!("Path '{}' is outside workspace", path.display()));
        }
        
        Ok(())
    }

    /// Check if a file extension is allowed
    pub fn is_extension_allowed(&self, path: &Path) -> Result<(), String> {
        if self.config.allowed_extensions.is_empty() {
            return Ok(());
        }
        
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        if !self.config.allowed_extensions.contains(&ext.to_string()) {
            return Err(format!("File extension '.{}' is not allowed", ext));
        }
        
        Ok(())
    }

    /// Check if a command is safe
    pub fn is_command_safe(&self, command: &str) -> Result<(), String> {
        for pattern in &self.config.blocked_patterns {
            if command.contains(pattern) {
                return Err(format!("Blocked command pattern detected: {}", pattern));
            }
        }
        
        Ok(())
    }

    /// Check file size
    pub fn check_file_size(&self, path: &Path) -> Result<(), String> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Cannot read file metadata: {}", e))?;
        
        if metadata.len() > self.config.max_file_size {
            return Err(format!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                self.config.max_file_size
            ));
        }
        
        Ok(())
    }
}

// ============================================================================
// Change Tracking
// ============================================================================

/// Represents a file change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path (relative to workspace)
    pub path: String,
    /// Change type
    pub change_type: ChangeType,
    /// Timestamp
    pub timestamp: u64,
    /// Old content hash (if applicable)
    pub old_hash: Option<String>,
    /// New content hash (if applicable)
    pub new_hash: Option<String>,
}

/// Type of file change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    /// File created
    Created,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
}

/// Change tracker for recording modifications
#[derive(Debug, Default)]
pub struct ChangeTracker {
    changes: Vec<FileChange>,
}

impl ChangeTracker {
    /// Create a new change tracker
    pub fn new() -> Self {
        Self { changes: Vec::new() }
    }

    /// Record a file change
    pub fn record(&mut self, change: FileChange) {
        self.changes.push(change);
    }

    /// Record file creation
    pub fn record_created(&mut self, path: &str, content: &str) {
        self.record(FileChange {
            path: path.to_string(),
            change_type: ChangeType::Created,
            timestamp: current_timestamp(),
            old_hash: None,
            new_hash: Some(hash_content(content)),
        });
    }

    /// Record file modification
    pub fn record_modified(&mut self, path: &str, old_content: &str, new_content: &str) {
        self.record(FileChange {
            path: path.to_string(),
            change_type: ChangeType::Modified,
            timestamp: current_timestamp(),
            old_hash: Some(hash_content(old_content)),
            new_hash: Some(hash_content(new_content)),
        });
    }

    /// Record file deletion
    pub fn record_deleted(&mut self, path: &str, content: &str) {
        self.record(FileChange {
            path: path.to_string(),
            change_type: ChangeType::Deleted,
            timestamp: current_timestamp(),
            old_hash: Some(hash_content(content)),
            new_hash: None,
        });
    }

    /// Get all changes
    pub fn changes(&self) -> &[FileChange] {
        &self.changes
    }

    /// Clear all changes
    pub fn clear(&mut self) {
        self.changes.clear();
    }

    /// Get changes for a specific file
    pub fn get_changes(&self, path: &str) -> Vec<&FileChange> {
        self.changes.iter().filter(|c| c.path == path).collect()
    }
}

/// Simple hash function for content
fn hash_content(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// ============================================================================
// Execution Context
// ============================================================================

/// Context for tool execution
#[derive(Debug)]
pub struct ExecutionContext {
    /// Workspace root
    pub workspace_root: PathBuf,
    /// Security configuration
    pub security: SecurityConfig,
    /// Change tracker
    pub changes: ChangeTracker,
    /// Execution start time
    pub start_time: Instant,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            security: SecurityConfig::default(),
            changes: ChangeTracker::new(),
            start_time: Instant::now(),
        }
    }

    /// Create with custom security config
    pub fn with_security(mut self, config: SecurityConfig) -> Self {
        self.security = config;
        self
    }

    /// Get security checker
    pub fn security_checker(&self) -> SecurityChecker {
        SecurityChecker::new(self.workspace_root.clone(), self.security.clone())
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

// ============================================================================
// Sandboxed Tool Executor
// ============================================================================

/// Wrapper that adds sandboxing to any tool executor
pub struct SandboxedExecutor<T: ToolExecutor> {
    inner: T,
    context: ExecutionContext,
}

impl<T: ToolExecutor> SandboxedExecutor<T> {
    /// Create a new sandboxed executor
    pub fn new(inner: T, context: ExecutionContext) -> Self {
        Self { inner, context }
    }

    /// Get the inner executor
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get mutable reference to context
    pub fn context_mut(&mut self) -> &mut ExecutionContext {
        &mut self.context
    }
}

#[async_trait]
impl<T: ToolExecutor + 'static> ToolExecutor for SandboxedExecutor<T> {
    fn definition(&self) -> Tool {
        self.inner.definition()
    }

    async fn execute(&self, args: serde_json::Value) -> ToolResult {
        let start = Instant::now();
        debug!("Executing tool: {}", self.definition().name);
        
        // Execute with timeout
        let result = tokio::time::timeout(
            self.context.security.command_timeout,
            self.inner.execute(args)
        ).await;
        
        let elapsed = start.elapsed();
        debug!("Tool execution completed in {:?}", elapsed);
        
        match result {
            Ok(res) => {
                if elapsed > self.context.security.command_timeout {
                    warn!("Tool execution took longer than timeout but completed");
                }
                res
            }
            Err(_) => {
                error!("Tool execution timed out after {:?}", self.context.security.command_timeout);
                ToolResult::error(format!("Tool execution timed out after {:?}", self.context.security.command_timeout))
            }
        }
    }
}

// ============================================================================
// Execution Result
// ============================================================================

/// Result of tool execution with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Tool name
    pub tool: String,
    /// Success status
    pub success: bool,
    /// Output
    pub output: String,
    /// Execution duration (milliseconds)
    pub duration_ms: u64,
    /// Security warnings
    pub warnings: Vec<String>,
    /// File changes made
    pub changes: Vec<FileChange>,
}

impl From<ToolResult> for ExecutionResult {
    fn from(result: ToolResult) -> Self {
        Self {
            tool: String::new(),
            success: result.success,
            output: result.output,
            duration_ms: 0,
            warnings: Vec::new(),
            changes: Vec::new(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_security_checker_path() {
        let temp_dir = TempDir::new().unwrap();
        let checker = SecurityChecker::new(
            temp_dir.path().to_path_buf(),
            SecurityConfig::default(),
        );
        
        // Allowed path
        let allowed = temp_dir.path().join("test.txt");
        std::fs::write(&allowed, "test").unwrap();
        assert!(checker.is_path_allowed(&allowed).is_ok());
        
        // Disallowed path
        let disallowed = PathBuf::from("/etc/passwd");
        assert!(checker.is_path_allowed(&disallowed).is_err());
    }

    #[test]
    fn test_security_checker_command() {
        let checker = SecurityChecker::new(
            PathBuf::from("/tmp"),
            SecurityConfig::default(),
        );
        
        // Safe command
        assert!(checker.is_command_safe("ls -la").is_ok());
        
        // Unsafe command
        assert!(checker.is_command_safe("rm -rf /").is_err());
        assert!(checker.is_command_safe("sudo apt-get install").is_err());
    }

    #[test]
    fn test_change_tracker() {
        let mut tracker = ChangeTracker::new();
        
        tracker.record_created("test.txt", "hello");
        tracker.record_modified("test.txt", "hello", "hello world");
        tracker.record_deleted("old.txt", "old content");
        
        assert_eq!(tracker.changes().len(), 3);
        assert_eq!(tracker.get_changes("test.txt").len(), 2);
    }

    #[test]
    fn test_hash_content() {
        let hash1 = hash_content("hello");
        let hash2 = hash_content("hello");
        let hash3 = hash_content("world");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_execution_context() {
        let temp_dir = TempDir::new().unwrap();
        let ctx = ExecutionContext::new(temp_dir.path().to_path_buf());
        
        assert!(ctx.elapsed() < Duration::from_secs(1));
        assert!(ctx.security_checker().is_command_safe("ls").is_ok());
    }
}
