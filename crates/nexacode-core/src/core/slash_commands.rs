//! CLI Slash Commands System
//!
//! This module implements the slash command system for CLI control:
//! - Command parsing and execution
//! - Model management (/model, /models, /config, /provider)
//! - Session management (/new, /sessions, /load, /save)
//! - Conversation control (/undo, /redo, /rollback, /clear)
//! - System commands (/help, /version, /quit, /theme)

// ============================================================================
// Slash Command Definition
// ============================================================================

/// All available slash commands
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    // Model management
    /// /model [name] - Switch or view current model
    Model { name: Option<String> },
    /// /models - List available models
    Models,
    /// /provider [name] - Switch or view current provider
    Provider { name: Option<String> },
    /// /config - Show current configuration
    Config,
    
    // Session management
    /// /new - Create new conversation
    New,
    /// /sessions - List all sessions
    Sessions,
    /// /load <id> - Load a session by ID
    Load { id: String },
    /// /save - Save current session
    Save,
    /// /export [format] - Export session to file
    Export { format: Option<String> },
    
    // Conversation control
    /// /undo - Undo last message pair
    Undo,
    /// /redo - Redo undone message
    Redo,
    /// /rollback [n] - Rollback to message n
    Rollback { count: Option<usize> },
    /// /clear - Clear current conversation
    Clear,
    
    // System commands
    /// /help [command] - Show help
    Help { command: Option<String> },
    /// /version - Show version
    Version,
    /// /quit or /exit - Exit program
    Quit,
    /// /theme [name] - Switch theme
    Theme { name: Option<String> },
}

// ============================================================================
// Command Parsing
// ============================================================================

/// Parse result
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Successfully parsed command
    Ok(SlashCommand),
    /// Input is not a slash command (regular message)
    NotACommand(String),
    /// Parse error
    Error(String),
}

/// Parse a slash command from input string
pub fn parse_slash_command(input: &str) -> ParseResult {
    let trimmed = input.trim();
    
    // Check if it starts with /
    if !trimmed.starts_with('/') {
        return ParseResult::NotACommand(input.to_string());
    }
    
    // Split into command and arguments
    let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
    
    if parts.is_empty() {
        return ParseResult::Error("Empty command".to_string());
    }
    
    let command = parts[0].to_lowercase();
    let args: Vec<&str> = parts[1..].to_vec();
    
    // Parse command
    let result = match command.as_str() {
        // Model management
        "model" => SlashCommand::Model {
            name: args.first().map(|s| s.to_string()),
        },
        "models" => SlashCommand::Models,
        "provider" => SlashCommand::Provider {
            name: args.first().map(|s| s.to_string()),
        },
        "config" => SlashCommand::Config,
        
        // Session management
        "new" => SlashCommand::New,
        "sessions" => SlashCommand::Sessions,
        "load" => {
            if args.is_empty() {
                return ParseResult::Error("/load requires a session ID".to_string());
            }
            SlashCommand::Load { id: args[0].to_string() }
        },
        "save" => SlashCommand::Save,
        "export" => SlashCommand::Export {
            format: args.first().map(|s| s.to_string()),
        },
        
        // Conversation control
        "undo" => SlashCommand::Undo,
        "redo" => SlashCommand::Redo,
        "rollback" => SlashCommand::Rollback {
            count: args.first().and_then(|s| s.parse().ok()),
        },
        "clear" => SlashCommand::Clear,
        
        // System commands
        "help" | "h" | "?" => SlashCommand::Help {
            command: args.first().map(|s| s.to_string()),
        },
        "version" | "v" => SlashCommand::Version,
        "quit" | "q" | "exit" => SlashCommand::Quit,
        "theme" => SlashCommand::Theme {
            name: args.first().map(|s| s.to_string()),
        },
        
        // Unknown command
        unknown => {
            return ParseResult::Error(format!("Unknown command: /{}", unknown));
        }
    };
    
    ParseResult::Ok(result)
}

// ============================================================================
// Command Execution Result
// ============================================================================

/// Result of command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command was successful
    pub success: bool,
    /// Output message to display
    pub output: String,
    /// Additional action to take (if any)
    pub action: Option<CommandAction>,
}

/// Actions that commands can trigger
#[derive(Debug, Clone)]
pub enum CommandAction {
    /// Switch model
    SwitchModel(String),
    /// Switch provider
    SwitchProvider(String),
    /// Create new session
    NewSession,
    /// Load session
    LoadSession(String),
    /// Save session
    SaveSession,
    /// Export session
    ExportSession { format: String },
    /// Undo
    Undo,
    /// Redo
    Redo,
    /// Rollback
    Rollback(usize),
    /// Clear conversation
    ClearConversation,
    /// Switch theme
    SwitchTheme(String),
    /// Exit program
    Quit,
}

impl CommandResult {
    /// Create a successful result
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            action: None,
        }
    }
    
    /// Create a successful result with action
    pub fn success_with_action(output: impl Into<String>, action: CommandAction) -> Self {
        Self {
            success: true,
            output: output.into(),
            action: Some(action),
        }
    }
    
    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: message.into(),
            action: None,
        }
    }
}

// ============================================================================
// Command Help
// ============================================================================

/// Get help text for a command
pub fn get_help(command: Option<&str>) -> String {
    match command {
        None => get_general_help(),
        Some(cmd) => get_command_help(cmd),
    }
}

fn get_general_help() -> String {
    r#"NexaCode CLI Commands

Model Management:
  /model [name]    Switch model or show current model
  /models          List all available models
  /provider [name] Switch provider (anthropic/openai)
  /config          Show current configuration

Session Management:
  /new             Start a new conversation
  /sessions        List all saved sessions
  /load <id>       Load a previous session
  /save            Save current session
  /export [fmt]    Export session (json/markdown)

Conversation Control:
  /undo            Undo last message pair
  /redo            Redo undone message
  /rollback [n]    Rollback to message n
  /clear           Clear current conversation

System:
  /help [cmd]      Show help (this message)
  /version         Show version info
  /quit            Exit NexaCode
  /theme [name]    Switch theme (dark/light)

Shortcuts:
  /h, /?           Same as /help
  /v               Same as /version
  /q               Same as /quit
"#.to_string()
}

fn get_command_help(cmd: &str) -> String {
    match cmd.to_lowercase().as_str() {
        "model" => r#"/model [name]

Switch the current model or show current model.

Examples:
  /model                    Show current model
  /model claude-3-5-sonnet  Switch to Claude 3.5 Sonnet
  /model gpt-4o             Switch to GPT-4o (auto-switches provider)

Use /models to see all available models.
"#.to_string(),
        
        "models" => r#"/models

List all available models.

Shows models from both Anthropic and OpenAI.
Use /provider to switch between providers.
"#.to_string(),
        
        "provider" => r#"/provider [name]

Switch or show the current LLM provider.

Providers:
  anthropic  Anthropic Claude (default)
  openai     OpenAI GPT

Examples:
  /provider           Show current provider
  /provider openai    Switch to OpenAI
"#.to_string(),
        
        "config" => r#"/config

Display current configuration including:
- Current model
- API settings
- Token limits
- Theme
"#.to_string(),
        
        "new" => r#"/new

Start a new conversation session.

The current session will be saved automatically
and a new empty session will be created.
"#.to_string(),
        
        "sessions" => r#"/sessions

List all saved conversation sessions.

Shows session ID, date, and message count.
Use /load <id> to restore a session.
"#.to_string(),
        
        "load" => r#"/load <id>

Load a previous conversation session.

Arguments:
  id    The session ID to load

Example:
  /load session-2024-01-15-abc123
"#.to_string(),
        
        "save" => r#"/save

Save the current conversation session.

Sessions are also auto-saved periodically.
"#.to_string(),
        
        "export" => r#"/export [format]

Export current session to a file.

Formats:
  json      JSON format (default)
  markdown  Markdown format

Example:
  /export markdown
"#.to_string(),
        
        "undo" => r#"/undo

Undo the last message pair (user + assistant).

Can be called multiple times to undo more.
Use /redo to restore undone messages.
"#.to_string(),
        
        "redo" => r#"/redo

Redo a previously undone message pair.

Only works if there are messages to redo.
"#.to_string(),
        
        "rollback" => r#"/rollback [n]

Rollback conversation to message n.

Arguments:
  n    Number of messages to keep (default: all but last)

Example:
  /rollback 5     Keep first 5 messages
  /rollback       Remove last message pair
"#.to_string(),
        
        "clear" => r#"/clear

Clear all messages in current conversation.

The session will be reset but not deleted.
"#.to_string(),
        
        "theme" => r#"/theme [name]

Switch or show the current theme.

Themes:
  dark    Dark theme (default)
  light   Light theme

Example:
  /theme light
"#.to_string(),
        
        "quit" | "exit" => r#"/quit

Exit NexaCode.

Aliases: /q, /exit
"#.to_string(),
        
        "help" => r#"/help [command]

Show help for commands.

Arguments:
  command    Optional command name for detailed help

Examples:
  /help           Show general help
  /help model     Show help for /model command
"#.to_string(),
        
        "version" => r#"/version

Show NexaCode version information.

Alias: /v
"#.to_string(),
        
        _ => format!("No help available for: {}", cmd),
    }
}

// ============================================================================
// Auto-complete
// ============================================================================

/// Get command suggestions for auto-complete
pub fn get_suggestions(input: &str) -> Vec<String> {
    if !input.starts_with('/') {
        return Vec::new();
    }
    
    let partial = &input[1..].to_lowercase();
    
    let commands = [
        "model", "models", "provider", "config",
        "new", "sessions", "load", "save", "export",
        "undo", "redo", "rollback", "clear",
        "help", "version", "quit", "exit", "theme",
    ];
    
    commands
        .iter()
        .filter(|cmd| cmd.starts_with(partial))
        .map(|cmd| format!("/{}", cmd))
        .collect()
}

/// Get command suggestions with arguments (loads from config at runtime)
pub fn get_suggestions_with_config(input: &str, providers: &[String], models: &[crate::infra::llm::config::ModelInfo]) -> Vec<String> {
    if !input.starts_with('/') {
        return Vec::new();
    }
    
    // Check if we're typing a command argument
    let parts: Vec<&str> = input[1..].split_whitespace().collect();
    
    if parts.is_empty() {
        return get_suggestions(input);
    }
    
    let command = parts[0].to_lowercase();
    
    // If we have a complete command and are typing arguments
    if parts.len() == 1 && !input.ends_with(' ') {
        // Still typing the command
        let commands = [
            "model", "models", "provider", "config",
            "new", "sessions", "load", "save", "export",
            "undo", "redo", "rollback", "clear",
            "help", "version", "quit", "exit", "theme",
        ];
        
        return commands
            .iter()
            .filter(|cmd| cmd.starts_with(&command))
            .map(|cmd| format!("/{}", cmd))
            .collect();
    }
    
    // We're typing arguments for a command
    let partial = parts.get(1).unwrap_or(&"");
    let _full_partial = input.split_whitespace().last().unwrap_or("");
    
    match command.as_str() {
        "provider" => {
            providers
                .iter()
                .filter(|p| p.to_lowercase().starts_with(&partial.to_lowercase()))
                .map(|p| format!("/provider {}", p))
                .collect()
        }
        "model" => {
            models
                .iter()
                .filter(|m| m.id.to_lowercase().starts_with(&partial.to_lowercase()) ||
                            m.display_name.to_lowercase().starts_with(&partial.to_lowercase()))
                .map(|m| format!("/model {}", m.id))
                .collect()
        }
        "theme" => {
            let themes = ["dark", "light"];
            themes
                .iter()
                .filter(|t| t.starts_with(&partial.to_lowercase()))
                .map(|t| format!("/theme {}", t))
                .collect()
        }
        "export" => {
            let formats = ["json", "markdown", "md"];
            formats
                .iter()
                .filter(|f| f.starts_with(&partial.to_lowercase()))
                .map(|f| format!("/export {}", f))
                .collect()
        }
        _ => Vec::new()
    }
}

/// Get argument suggestions for a command
pub fn get_argument_suggestions(command: &str, partial: &str) -> Vec<String> {
    match command.to_lowercase().as_str() {
        "model" => {
            // Suggest configured models from config
            // Note: This returns empty list as we need to load config at runtime
            // The TUI will handle this separately
            Vec::new()
        }
        "provider" => {
            // Return empty - TUI will load from config
            Vec::new()
        }
        "theme" => {
            let themes = ["dark", "light"];
            themes
                .iter()
                .filter(|t| t.starts_with(partial))
                .map(|s| s.to_string())
                .collect()
        }
        "export" => {
            let formats = ["json", "markdown", "md"];
            formats
                .iter()
                .filter(|f| f.starts_with(partial))
                .map(|s| s.to_string())
                .collect()
        }
        _ => Vec::new(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_command() {
        match parse_slash_command("/model claude-3-opus") {
            ParseResult::Ok(SlashCommand::Model { name }) => {
                assert_eq!(name, Some("claude-3-opus".to_string()));
            }
            _ => panic!("Failed to parse /model command"),
        }
    }

    #[test]
    fn test_parse_help_command() {
        match parse_slash_command("/help model") {
            ParseResult::Ok(SlashCommand::Help { command }) => {
                assert_eq!(command, Some("model".to_string()));
            }
            _ => panic!("Failed to parse /help command"),
        }
    }

    #[test]
    fn test_parse_not_a_command() {
        match parse_slash_command("Hello world") {
            ParseResult::NotACommand(msg) => {
                assert_eq!(msg, "Hello world");
            }
            _ => panic!("Should be NotACommand"),
        }
    }

    #[test]
    fn test_parse_unknown_command() {
        match parse_slash_command("/unknown") {
            ParseResult::Error(msg) => {
                assert!(msg.contains("Unknown command"));
            }
            _ => panic!("Should be Error"),
        }
    }

    #[test]
    fn test_suggestions() {
        let suggestions = get_suggestions("/mod");
        assert!(suggestions.contains(&"/model".to_string()));
        assert!(suggestions.contains(&"/models".to_string()));
    }

    #[test]
    fn test_argument_suggestions() {
        // Note: get_argument_suggestions now returns empty because
        // suggestions are loaded from config at runtime via get_suggestions_with_config
        let suggestions = get_argument_suggestions("model", "claude-3-");
        // Should be empty since we don't have config here
        assert!(suggestions.is_empty());
        
        // Test theme suggestions still work (not config-dependent)
        let suggestions = get_argument_suggestions("theme", "da");
        assert!(suggestions.contains(&"dark".to_string()));
    }

    #[test]
    fn test_help_output() {
        let help = get_help(None);
        assert!(help.contains("Model Management"));
        assert!(help.contains("/model"));
    }

    #[test]
    fn test_command_aliases() {
        // Test /h alias
        match parse_slash_command("/h") {
            ParseResult::Ok(SlashCommand::Help { .. }) => {}
            _ => panic!("Failed to parse /h alias"),
        }
        
        // Test /q alias
        match parse_slash_command("/q") {
            ParseResult::Ok(SlashCommand::Quit) => {}
            _ => panic!("Failed to parse /q alias"),
        }
        
        // Test /v alias
        match parse_slash_command("/v") {
            ParseResult::Ok(SlashCommand::Version) => {}
            _ => panic!("Failed to parse /v alias"),
        }
    }
}
