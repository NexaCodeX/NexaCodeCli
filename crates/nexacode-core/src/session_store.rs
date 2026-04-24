//! Session Persistence
//!
//! This module handles saving and loading conversation sessions from disk.

use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use crate::data_dir::NexaCodeDir;
use crate::Session;

/// Session metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    /// Session ID
    pub id: String,
    /// Session name (if any)
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Number of messages
    pub message_count: usize,
    /// Preview of the first user message
    pub preview: Option<String>,
}

/// Session storage manager
pub struct SessionStore {
    data_dir: NexaCodeDir,
}

impl SessionStore {
    /// Create a new session store
    pub fn new() -> Self {
        Self {
            data_dir: NexaCodeDir::new(),
        }
    }

    /// Save a session to disk
    pub fn save_session(&self, session: &Session) -> Result<()> {
        self.data_dir.ensure_dirs()?;
        
        let path = self.data_dir.session_file(&session.id);
        let content = serde_json::to_string_pretty(session)
            .with_context(|| "Failed to serialize session")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write session file: {:?}", path))?;
        
        Ok(())
    }

    /// Save session as default
    pub fn save_default_session(&self, session: &Session) -> Result<()> {
        self.data_dir.ensure_dirs()?;
        
        let path = self.data_dir.default_session_file();
        let content = serde_json::to_string_pretty(session)
            .with_context(|| "Failed to serialize session")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write default session file"))?;
        
        Ok(())
    }

    /// Load a session by ID
    pub fn load_session(&self, id: &str) -> Result<Option<Session>> {
        let path = self.data_dir.session_file(id);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read session file: {:?}", path))?;
        
        let session: Session = serde_json::from_str(&content)
            .with_context(|| "Failed to parse session file")?;
        
        Ok(Some(session))
    }

    /// Load the default session
    pub fn load_default_session(&self) -> Result<Option<Session>> {
        let path = self.data_dir.default_session_file();
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&path)
            .with_context(|| "Failed to read default session file")?;
        
        let session: Session = serde_json::from_str(&content)
            .with_context(|| "Failed to parse default session file")?;
        
        Ok(Some(session))
    }

    /// Delete a session by ID
    pub fn delete_session(&self, id: &str) -> Result<bool> {
        let path = self.data_dir.session_file(id);
        
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete session file: {:?}", path))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List all sessions with metadata
    pub fn list_sessions(&self) -> Result<Vec<SessionMeta>> {
        let files = self.data_dir.list_session_files()?;
        let mut sessions = Vec::new();
        
        for path in files {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(session) = serde_json::from_str::<Session>(&content) {
                    let preview = session.messages
                        .iter()
                        .find(|m| m.role == crate::MessageRole::User)
                        .map(|m| {
                            let content = &m.content;
                            if content.len() > 50 {
                                format!("{}...", &content[..50])
                            } else {
                                content.clone()
                            }
                        });
                    
                    sessions.push(SessionMeta {
                        id: session.id.clone(),
                        name: session.name.clone(),
                        created_at: session.created_at,
                        modified_at: session.modified_at,
                        message_count: session.messages.len(),
                        preview,
                    });
                }
            }
        }
        
        Ok(sessions)
    }

    /// Get the sessions directory
    pub fn sessions_dir(&self) -> PathBuf {
        self.data_dir.sessions_dir()
    }

    /// Format sessions list for display
    pub fn format_sessions_list(&self) -> Result<String> {
        let sessions = self.list_sessions()?;
        
        if sessions.is_empty() {
            return Ok("No saved sessions.\n\nUse /save to save the current session.".to_string());
        }
        
        let mut output = String::new();
        output.push_str("Saved Sessions:\n\n");
        
        for session in sessions {
            let name = session.name.as_deref().unwrap_or("Untitled");
            let preview = session.preview.as_deref().unwrap_or("");
            
            // Format timestamp
            let modified = format_timestamp(session.modified_at);
            
            output.push_str(&format!(
                "  {}  {}  ({} messages) - {}\n    └─ {}\n\n",
                session.id,
                name,
                session.message_count,
                modified,
                preview
            ));
        }
        
        output.push_str("\nUse /load <id> to restore a session");
        
        Ok(output)
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Format timestamp to human-readable string
fn format_timestamp(timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let diff_secs = (now - timestamp) / 1000;
    
    if diff_secs < 60 {
        "Just now".to_string()
    } else if diff_secs < 3600 {
        format!("{} min ago", diff_secs / 60)
    } else if diff_secs < 86400 {
        format!("{} hours ago", diff_secs / 3600)
    } else if diff_secs < 604800 {
        format!("{} days ago", diff_secs / 86400)
    } else {
        format!("{} weeks ago", diff_secs / 604800)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_store_creation() {
        let store = SessionStore::new();
        assert!(store.sessions_dir().ends_with("sessions"));
    }
}
