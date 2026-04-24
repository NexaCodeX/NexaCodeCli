//! NexaCode Core Library
//!
//! This crate contains the core functionality of NexaCode:
//! - Core: Agent controller, Task engine, Planning engine, Command system
//! - State: State management with actions and reducers
//! - Skills: Skills system with manager, registry, and executor
//! - MCP: Model Context Protocol implementation
//! - Infra: Infrastructure layer (LLM, FS, Git, Shell)

pub mod core;
pub mod state;
pub mod skills;
pub mod mcp;
pub mod infra;

// Re-export commonly used types
pub use self::core::{agent, task_engine, planning, command};
pub use self::state::{actions, reducers, history};
