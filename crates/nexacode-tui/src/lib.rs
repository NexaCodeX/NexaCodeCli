//! NexaCode TUI Library
//!
//! This crate contains the Terminal User Interface of NexaCode:
//! - App state management
//! - TUI components and views
//! - Event handling
//! - Theme system

pub mod app;
pub mod tui;

// Re-export commonly used types
pub use self::app::{App, AgentState, Message, MessageRole};
pub use self::tui::Theme;
pub use self::tui::event::handle_event;
pub use self::tui::views::render;
