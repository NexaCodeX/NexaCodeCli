//! Tool System
//!
//! This module implements the tool system including:
//! - Tool data structures (name, description, input schema)
//! - Built-in tools (file operations, commands, git)
//! - Tool registry and execution

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use async_trait::async_trait;

// ============================================================================
// Tool Data Structures
// ============================================================================

/// Tool definition with name, description, and input schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name (unique identifier)
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema (JSON Schema)
    pub input_schema: ToolInputSchema,
}

/// Tool input schema (JSON Schema format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    /// Schema type (always "object")
    #[serde(rename = "type")]
    pub type_name: String,
    /// Required properties
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    /// Property definitions
    pub properties: HashMap<String, ToolProperty>,
}

impl ToolInputSchema {
    /// Create a new input schema
    pub fn new() -> Self {
        Self {
            type_name: "object".to_string(),
            required: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Add a property
    pub fn with_property(mut self, name: &str, property: ToolProperty, required: bool) -> Self {
        self.properties.insert(name.to_string(), property);
        if required {
            self.required.push(name.to_string());
        }
        self
    }
}

impl Default for ToolInputSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProperty {
    /// Property type
    #[serde(rename = "type")]
    pub type_name: String,
    /// Property description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Enum values (if applicable)
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

impl ToolProperty {
    /// Create a string property
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            type_name: "string".to_string(),
            description: Some(description.into()),
            enum_values: None,
            default: None,
        }
    }

    /// Create a number property
    pub fn number(description: impl Into<String>) -> Self {
        Self {
            type_name: "number".to_string(),
            description: Some(description.into()),
            enum_values: None,
            default: None,
        }
    }

    /// Create a boolean property
    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            type_name: "boolean".to_string(),
            description: Some(description.into()),
            enum_values: None,
            default: None,
        }
    }

    /// Create an array property
    pub fn array(description: impl Into<String>, _items_type: &str) -> Self {
        Self {
            type_name: "array".to_string(),
            description: Some(description.into()),
            enum_values: None,
            default: None,
        }
    }

    /// Add enum values
    pub fn with_enum(mut self, values: Vec<&str>) -> Self {
        self.enum_values = Some(values.into_iter().map(String::from).collect());
        self
    }

    /// Add default value
    pub fn with_default(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool executed successfully
    pub success: bool,
    /// Output or error message
    pub output: String,
    /// Additional data (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            data: None,
        }
    }

    /// Create a successful result with data
    pub fn success_with_data(output: impl Into<String>, data: Value) -> Self {
        Self {
            success: true,
            output: output.into(),
            data: Some(data),
        }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: message.into(),
            data: None,
        }
    }
}

// ============================================================================
// Tool Executor Trait
// ============================================================================

/// Trait for tool execution
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Get tool definition
    fn definition(&self) -> Tool;
    
    /// Execute the tool
    async fn execute(&self, args: Value) -> ToolResult;
}

// ============================================================================
// Built-in Tools
// ============================================================================

/// Read file tool
pub struct ReadFileTool {
    workspace_root: PathBuf,
}

impl ReadFileTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl ToolExecutor for ReadFileTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "read_file".to_string(),
            description: "Read the contents of a file. Returns the file content as a string.".to_string(),
            input_schema: ToolInputSchema::new()
                .with_property(
                    "path",
                    ToolProperty::string("The path to the file to read (relative to workspace root)"),
                    true,
                ),
        }
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let full_path = self.workspace_root.join(path);
        
        // Security check: ensure path is within workspace
        if !full_path.starts_with(&self.workspace_root) {
            return ToolResult::error("Access denied: path is outside workspace");
        }

        match std::fs::read_to_string(&full_path) {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(format!("Failed to read file: {}", e)),
        }
    }
}

/// Write file tool
pub struct WriteFileTool {
    workspace_root: PathBuf,
}

impl WriteFileTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl ToolExecutor for WriteFileTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "write_file".to_string(),
            description: "Write content to a file. Creates the file if it doesn't exist, overwrites if it does.".to_string(),
            input_schema: ToolInputSchema::new()
                .with_property(
                    "path",
                    ToolProperty::string("The path to the file to write (relative to workspace root)"),
                    true,
                )
                .with_property(
                    "content",
                    ToolProperty::string("The content to write to the file"),
                    true,
                ),
        }
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let full_path = self.workspace_root.join(path);
        
        // Security check
        if !full_path.starts_with(&self.workspace_root) {
            return ToolResult::error("Access denied: path is outside workspace");
        }

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResult::error(format!("Failed to create directories: {}", e));
            }
        }

        match std::fs::write(&full_path, content) {
            Ok(_) => ToolResult::success(format!("Successfully wrote to {}", path)),
            Err(e) => ToolResult::error(format!("Failed to write file: {}", e)),
        }
    }
}

/// Edit file tool (search and replace)
pub struct EditFileTool {
    workspace_root: PathBuf,
}

impl EditFileTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl ToolExecutor for EditFileTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "edit_file".to_string(),
            description: "Edit a file by replacing specific text. Uses search and replace.".to_string(),
            input_schema: ToolInputSchema::new()
                .with_property(
                    "path",
                    ToolProperty::string("The path to the file to edit"),
                    true,
                )
                .with_property(
                    "old_text",
                    ToolProperty::string("The text to search for"),
                    true,
                )
                .with_property(
                    "new_text",
                    ToolProperty::string("The text to replace with"),
                    true,
                ),
        }
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let old_text = args.get("old_text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let new_text = args.get("new_text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let full_path = self.workspace_root.join(path);
        
        // Security check
        if !full_path.starts_with(&self.workspace_root) {
            return ToolResult::error("Access denied: path is outside workspace");
        }

        // Read file
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(e) => return ToolResult::error(format!("Failed to read file: {}", e)),
        };

        // Replace
        let new_content = content.replace(old_text, new_text);
        
        if new_content == content {
            return ToolResult::error("Text not found in file");
        }

        // Write back
        match std::fs::write(&full_path, new_content) {
            Ok(_) => ToolResult::success(format!("Successfully edited {}", path)),
            Err(e) => ToolResult::error(format!("Failed to write file: {}", e)),
        }
    }
}

/// List directory tool
pub struct ListDirTool {
    workspace_root: PathBuf,
}

impl ListDirTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl ToolExecutor for ListDirTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "list_dir".to_string(),
            description: "List the contents of a directory. Returns a list of files and directories.".to_string(),
            input_schema: ToolInputSchema::new()
                .with_property(
                    "path",
                    ToolProperty::string("The path to the directory to list (relative to workspace root, defaults to root)"),
                    false,
                ),
        }
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let full_path = if path.is_empty() {
            self.workspace_root.clone()
        } else {
            self.workspace_root.join(path)
        };
        
        // Security check
        if !full_path.starts_with(&self.workspace_root) {
            return ToolResult::error("Access denied: path is outside workspace");
        }

        match std::fs::read_dir(&full_path) {
            Ok(entries) => {
                let mut result = Vec::new();
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    result.push(if is_dir {
                        format!("{}/", name)
                    } else {
                        name
                    });
                }
                result.sort();
                ToolResult::success(result.join("\n"))
            }
            Err(e) => ToolResult::error(format!("Failed to list directory: {}", e)),
        }
    }
}

/// Run command tool
pub struct RunCommandTool {
    workspace_root: PathBuf,
    #[allow(dead_code)]
    timeout: Duration,
}

impl RunCommandTool {
    pub fn new(workspace_root: PathBuf, timeout: Duration) -> Self {
        Self { workspace_root, timeout }
    }
}

#[async_trait]
impl ToolExecutor for RunCommandTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "run_command".to_string(),
            description: "Run a shell command in the workspace directory. Use with caution.".to_string(),
            input_schema: ToolInputSchema::new()
                .with_property(
                    "command",
                    ToolProperty::string("The command to run"),
                    true,
                )
                .with_property(
                    "args",
                    ToolProperty::array("Command arguments", "string"),
                    false,
                ),
        }
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let command = args.get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let cmd_args: Vec<&str> = args.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        // Security: block dangerous commands
        let dangerous = ["rm -rf", "sudo", "chmod", "chown", ">", ">>", "|", ";", "&&", "||"];
        for danger in dangerous {
            if command.contains(danger) {
                return ToolResult::error(format!("Dangerous command pattern detected: {}", danger));
            }
        }

        let output = Command::new(command)
            .args(&cmd_args)
            .current_dir(&self.workspace_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if output.status.success() {
                    ToolResult::success(stdout.to_string())
                } else {
                    ToolResult::error(format!("Command failed with status: {}\n{}", 
                        output.status, stderr))
                }
            }
            Err(e) => ToolResult::error(format!("Failed to execute command: {}", e)),
        }
    }
}

/// Git status tool
pub struct GitStatusTool {
    workspace_root: PathBuf,
}

impl GitStatusTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl ToolExecutor for GitStatusTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "git_status".to_string(),
            description: "Get the git status of the repository. Shows changed files.".to_string(),
            input_schema: ToolInputSchema::new(),
        }
    }

    async fn execute(&self, _args: Value) -> ToolResult {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.workspace_root)
            .output();

        match output {
            Ok(output) => {
                let status = String::from_utf8_lossy(&output.stdout);
                if status.is_empty() {
                    ToolResult::success("Working tree clean")
                } else {
                    ToolResult::success(status.to_string())
                }
            }
            Err(e) => ToolResult::error(format!("Failed to run git status: {}", e)),
        }
    }
}

/// Git diff tool
pub struct GitDiffTool {
    workspace_root: PathBuf,
}

impl GitDiffTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl ToolExecutor for GitDiffTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "git_diff".to_string(),
            description: "Get the git diff of changes. Shows unstaged changes by default.".to_string(),
            input_schema: ToolInputSchema::new()
                .with_property(
                    "staged",
                    ToolProperty::boolean("Show staged changes instead of unstaged"),
                    false,
                ),
        }
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let staged = args.get("staged")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut cmd = Command::new("git");
        cmd.arg("diff");
        if staged {
            cmd.arg("--staged");
        }
        cmd.current_dir(&self.workspace_root);

        let output = cmd.output();

        match output {
            Ok(output) => {
                let diff = String::from_utf8_lossy(&output.stdout);
                if diff.is_empty() {
                    ToolResult::success("No changes")
                } else {
                    ToolResult::success(diff.to_string())
                }
            }
            Err(e) => ToolResult::error(format!("Failed to run git diff: {}", e)),
        }
    }
}

// ============================================================================
// Tool Registry
// ============================================================================

/// Tool registry - manages available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolExecutor>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn ToolExecutor>) {
        let name = tool.definition().name;
        self.tools.insert(name, tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn ToolExecutor> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Get all tool definitions
    pub fn definitions(&self) -> Vec<Tool> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Check if a tool exists
    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// List all tool names
    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default tool registry with built-in tools
pub fn create_default_registry(workspace_root: PathBuf) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    registry.register(Box::new(ReadFileTool::new(workspace_root.clone())));
    registry.register(Box::new(WriteFileTool::new(workspace_root.clone())));
    registry.register(Box::new(EditFileTool::new(workspace_root.clone())));
    registry.register(Box::new(ListDirTool::new(workspace_root.clone())));
    registry.register(Box::new(RunCommandTool::new(
        workspace_root.clone(),
        Duration::from_secs(30),
    )));
    registry.register(Box::new(GitStatusTool::new(workspace_root.clone())));
    registry.register(Box::new(GitDiffTool::new(workspace_root)));
    
    registry
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tool_schema() {
        let schema = ToolInputSchema::new()
            .with_property("path", ToolProperty::string("File path"), true)
            .with_property("content", ToolProperty::string("File content"), true);
        
        assert!(schema.properties.contains_key("path"));
        assert!(schema.properties.contains_key("content"));
        assert_eq!(schema.required.len(), 2);
    }

    #[tokio::test]
    async fn test_read_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let tool = ReadFileTool::new(temp_dir.path().to_path_buf());
        let def = tool.definition();
        assert_eq!(def.name, "read_file");

        let result = tool.execute(serde_json::json!({"path": "test.txt"})).await;
        assert!(result.success);
        assert_eq!(result.output, "Hello, World!");
    }

    #[tokio::test]
    async fn test_write_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        
        let tool = WriteFileTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(serde_json::json!({
            "path": "new_file.txt",
            "content": "Test content"
        })).await;
        
        assert!(result.success);
        
        let content = std::fs::read_to_string(temp_dir.path().join("new_file.txt")).unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_list_dir_tool() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        
        let tool = ListDirTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(serde_json::json!({})).await;
        
        assert!(result.success);
        assert!(result.output.contains("file1.txt"));
        assert!(result.output.contains("subdir/"));
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(ReadFileTool::new(PathBuf::from("/tmp"))));
        
        assert!(registry.has("read_file"));
        assert!(!registry.has("write_file"));
        assert_eq!(registry.names().len(), 1);
    }
}
