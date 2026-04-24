//! Context Management
//!
//! This module implements context management including:
//! - Token counting and budget management
//! - Context pruning strategies (sliding window, important message retention)
//! - Message history management with priority

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::Message;

// ============================================================================
// Token Counter
// ============================================================================

/// Token counter for estimating token usage
///
/// Uses a simple heuristic-based approach similar to tiktoken:
/// - ~4 characters per token for English text
/// - Adjustments for whitespace and special characters
pub struct TokenCounter {
    /// Characters per token ratio
    chars_per_token: f32,
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self {
            chars_per_token: 4.0, // Standard approximation
        }
    }
}

impl TokenCounter {
    /// Create a new token counter
    pub fn new() -> Self {
        Self::default()
    }

    /// Count tokens in a string
    pub fn count(&self, text: &str) -> u64 {
        if text.is_empty() {
            return 0;
        }

        // Count words (tokens roughly correlate with words)
        let word_count = text.split_whitespace().count() as f32;
        
        // Count characters
        let char_count = text.len() as f32;
        
        // Estimate based on both metrics
        let estimate_by_chars = char_count / self.chars_per_token;
        let estimate_by_words = word_count * 1.3; // Words are slightly more than tokens
        
        // Use weighted average
        ((estimate_by_chars + estimate_by_words) / 2.0).ceil() as u64
    }

    /// Count tokens for a message (including metadata overhead)
    pub fn count_message(&self, message: &Message) -> u64 {
        let content_tokens = self.count(&message.content);
        
        // Add overhead for message structure (role, metadata)
        // Typical overhead: ~4 tokens per message
        content_tokens + 4
    }

    /// Count tokens for a list of messages
    pub fn count_messages(&self, messages: &[Message]) -> u64 {
        messages.iter().map(|m| self.count_message(m)).sum()
    }
}

// ============================================================================
// Message Priority
// ============================================================================

/// Priority level for messages (used in pruning decisions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Can be pruned first
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority, prefer to keep
    High = 2,
    /// Critical, should never be pruned
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Message with priority metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedMessage {
    /// The message
    pub message: Message,
    /// Priority level
    pub priority: MessagePriority,
    /// Whether this message is pinned (never prune)
    pub pinned: bool,
    /// Token count (cached)
    #[serde(skip)]
    pub token_count: u64,
}

impl PrioritizedMessage {
    /// Create a new prioritized message
    pub fn new(message: Message, priority: MessagePriority) -> Self {
        Self {
            message,
            priority,
            pinned: false,
            token_count: 0,
        }
    }

    /// Create a pinned message
    pub fn pinned(message: Message) -> Self {
        Self {
            message,
            priority: MessagePriority::Critical,
            pinned: true,
            token_count: 0,
        }
    }

    /// Set the token count
    pub fn with_token_count(mut self, count: u64) -> Self {
        self.token_count = count;
        self
    }
}

// ============================================================================
// Pruning Strategy
// ============================================================================

/// Strategy for pruning context
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum PruningStrategy {
    /// Remove oldest messages first
    #[default]
    OldestFirst,
    /// Remove by priority (lowest first)
    PriorityBased,
    /// Keep a sliding window of recent messages
    SlidingWindow {
        /// Maximum messages to keep
        max_messages: usize,
    },
    /// Hybrid: consider both priority and recency
    Hybrid {
        /// Maximum messages to keep
        max_messages: usize,
        /// Ratio of important messages to preserve (0.0-1.0)
        important_ratio: f32,
    },
}

// ============================================================================
// Context Manager
// ============================================================================

/// Configuration for context management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum token budget
    pub max_tokens: u64,
    /// Reserve tokens for response generation
    pub reserve_tokens: u64,
    /// Pruning strategy to use
    pub pruning_strategy: PruningStrategy,
    /// Minimum messages to always keep
    pub min_messages: usize,
    /// System prompt (always included)
    pub system_prompt: Option<String>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 200_000,      // Claude 3 context window
            reserve_tokens: 8_192,    // Reserve for response
            pruning_strategy: PruningStrategy::default(),
            min_messages: 2,          // Keep at least user + assistant
            system_prompt: None,
        }
    }
}

/// Context manager for handling conversation context
pub struct ContextManager {
    /// Configuration
    config: ContextConfig,
    /// Token counter
    token_counter: TokenCounter,
    /// Message history with priorities
    messages: VecDeque<PrioritizedMessage>,
    /// Current token count
    current_tokens: u64,
    /// System message (if any)
    system_message: Option<PrioritizedMessage>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(config: ContextConfig) -> Self {
        Self {
            config,
            token_counter: TokenCounter::new(),
            messages: VecDeque::new(),
            current_tokens: 0,
            system_message: None,
        }
    }

    /// Get available tokens for messages (after reserve)
    pub fn available_tokens(&self) -> u64 {
        self.config.max_tokens.saturating_sub(self.config.reserve_tokens)
    }

    /// Get remaining tokens
    pub fn remaining_tokens(&self) -> u64 {
        self.available_tokens().saturating_sub(self.current_tokens)
    }

    /// Check if we're within budget
    pub fn is_within_budget(&self) -> bool {
        self.current_tokens < self.available_tokens()
    }

    /// Set system prompt
    pub fn set_system_prompt(&mut self, prompt: String) {
        let token_count = self.token_counter.count(&prompt) + 4;
        let message = Message::new(crate::MessageRole::System, prompt);
        
        // Subtract old system message if exists
        if let Some(ref old) = self.system_message {
            self.current_tokens = self.current_tokens.saturating_sub(old.token_count);
        }
        
        self.system_message = Some(
            PrioritizedMessage::pinned(message)
                .with_token_count(token_count)
        );
        self.current_tokens += token_count;
    }

    /// Add a message with default priority
    pub fn add_message(&mut self, message: Message) {
        self.add_prioritized_message(PrioritizedMessage::new(message, MessagePriority::Normal));
    }

    /// Add a message with specific priority
    pub fn add_message_with_priority(&mut self, message: Message, priority: MessagePriority) {
        self.add_prioritized_message(PrioritizedMessage::new(message, priority));
    }

    /// Add a pinned message (never pruned)
    pub fn add_pinned_message(&mut self, message: Message) {
        self.add_prioritized_message(PrioritizedMessage::pinned(message));
    }

    /// Add a prioritized message
    fn add_prioritized_message(&mut self, prioritized: PrioritizedMessage) {
        let token_count = self.token_counter.count_message(&prioritized.message);
        let prioritized = prioritized.with_token_count(token_count);
        
        self.current_tokens += token_count;
        self.messages.push_back(prioritized);
        
        // Check if pruning is needed
        if !self.is_within_budget() {
            self.prune();
        }
    }

    /// Get all messages formatted for LLM
    pub fn get_messages_for_llm(&self) -> Vec<serde_json::Value> {
        let mut result = Vec::new();
        
        // Add system message if present
        if let Some(ref sys_msg) = self.system_message {
            result.push(self.message_to_json(&sys_msg.message));
        }
        
        // Add conversation messages
        for prioritized in &self.messages {
            result.push(self.message_to_json(&prioritized.message));
        }
        
        result
    }

    /// Convert message to JSON format for LLM API
    fn message_to_json(&self, message: &Message) -> serde_json::Value {
        serde_json::json!({
            "role": match message.role {
                crate::MessageRole::User => "user",
                crate::MessageRole::Assistant => "assistant",
                crate::MessageRole::System => "system",
                crate::MessageRole::Tool => "tool",
            },
            "content": message.content,
        })
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get current token count
    pub fn token_count(&self) -> u64 {
        self.current_tokens
    }

    /// Prune messages based on strategy
    pub fn prune(&mut self) {
        if self.messages.len() <= self.config.min_messages {
            debug!("Skipping prune: minimum messages reached");
            return;
        }

        let target_tokens = self.available_tokens() * 90 / 100; // Target 90% of capacity
        let mut pruned_count = 0;

        match self.config.pruning_strategy {
            PruningStrategy::OldestFirst => {
                while self.current_tokens > target_tokens 
                    && self.messages.len() > self.config.min_messages 
                {
                    if let Some(msg) = self.messages.pop_front() {
                        if !msg.pinned {
                            self.current_tokens = self.current_tokens.saturating_sub(msg.token_count);
                            pruned_count += 1;
                        } else {
                            // Put pinned message back
                            self.messages.push_front(msg);
                            break;
                        }
                    }
                }
            }
            
            PruningStrategy::PriorityBased => {
                // Sort by priority (ascending), then by position (oldest first for same priority)
                let mut messages: Vec<_> = self.messages.drain(..).collect();
                messages.sort_by(|a, b| {
                    a.priority.cmp(&b.priority)
                        .then_with(|| a.message.timestamp.cmp(&b.message.timestamp))
                });
                
                // Remove lowest priority, non-pinned messages
                messages.retain(|msg| {
                    if self.current_tokens <= target_tokens {
                        return true;
                    }
                    if msg.pinned {
                        return true;
                    }
                    self.current_tokens = self.current_tokens.saturating_sub(msg.token_count);
                    pruned_count += 1;
                    false
                });
                
                self.messages = messages.into_iter().collect();
            }
            
            PruningStrategy::SlidingWindow { max_messages } => {
                while self.messages.len() > max_messages {
                    if let Some(msg) = self.messages.pop_front() {
                        if !msg.pinned {
                            self.current_tokens = self.current_tokens.saturating_sub(msg.token_count);
                            pruned_count += 1;
                        } else {
                            self.messages.push_front(msg);
                            break;
                        }
                    }
                }
            }
            
            PruningStrategy::Hybrid { max_messages, important_ratio } => {
                let mut messages: Vec<_> = self.messages.drain(..).collect();
                
                // Separate pinned/important from normal
                let (mut important, mut normal): (Vec<_>, Vec<_>) = 
                    messages.into_iter().partition(|m| m.pinned || m.priority >= MessagePriority::High);
                
                // Trim normal messages from the front (oldest)
                while (normal.len() + important.len() > max_messages || self.current_tokens > target_tokens)
                    && !normal.is_empty()
                {
                    if let Some(msg) = normal.first() {
                        self.current_tokens = self.current_tokens.saturating_sub(msg.token_count);
                        pruned_count += 1;
                    }
                    normal.remove(0);
                }
                
                // Ensure we keep important ratio
                let min_important = ((important.len() + normal.len()) as f32 * important_ratio) as usize;
                while important.len() > min_important && important.len() > 1 {
                    if let Some(msg) = important.first() {
                        if !msg.pinned {
                            self.current_tokens = self.current_tokens.saturating_sub(msg.token_count);
                            pruned_count += 1;
                            important.remove(0);
                        } else {
                            break;
                        }
                    }
                }
                
                // Recombine and sort by timestamp
                messages = [important, normal].concat();
                messages.sort_by_key(|m| m.message.timestamp);
                
                self.messages = messages.into_iter().collect();
            }
        }

        if pruned_count > 0 {
            debug!("Pruned {} messages, current tokens: {}", pruned_count, self.current_tokens);
        }
    }

    /// Clear all messages (except system)
    pub fn clear(&mut self) {
        let system = self.system_message.take();
        self.messages.clear();
        self.current_tokens = system.as_ref().map(|s| s.token_count).unwrap_or(0);
        self.system_message = system;
    }

    /// Get last N messages
    pub fn get_recent_messages(&self, n: usize) -> Vec<&Message> {
        self.messages
            .iter()
            .rev()
            .take(n)
            .rev()
            .map(|p| &p.message)
            .collect()
    }

    /// Get statistics about the context
    pub fn stats(&self) -> ContextStats {
        ContextStats {
            message_count: self.messages.len(),
            token_count: self.current_tokens,
            max_tokens: self.config.max_tokens,
            available_tokens: self.available_tokens(),
            remaining_tokens: self.remaining_tokens(),
            has_system_prompt: self.system_message.is_some(),
        }
    }
}

// ============================================================================
// Context Statistics
// ============================================================================

/// Statistics about current context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub message_count: usize,
    pub token_count: u64,
    pub max_tokens: u64,
    pub available_tokens: u64,
    pub remaining_tokens: u64,
    pub has_system_prompt: bool,
}

impl ContextStats {
    /// Get utilization percentage
    pub fn utilization_percent(&self) -> f32 {
        if self.available_tokens == 0 {
            return 0.0;
        }
        (self.token_count as f32 / self.available_tokens as f32) * 100.0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MessageRole;

    #[test]
    fn test_token_counter() {
        let counter = TokenCounter::new();
        
        // Test basic counting
        let tokens = counter.count("Hello, world!");
        assert!(tokens > 0);
        assert!(tokens < 20); // Should be reasonable
        
        // Test empty string
        assert_eq!(counter.count(""), 0);
        
        // Test longer text
        let long_text = "This is a longer piece of text that should have more tokens.";
        let long_tokens = counter.count(long_text);
        assert!(long_tokens > tokens);
    }

    #[test]
    fn test_context_manager_basic() {
        let manager = ContextManager::new(ContextConfig::default());
        
        assert_eq!(manager.message_count(), 0);
        assert!(manager.is_within_budget());
        assert!(manager.remaining_tokens() > 0);
    }

    #[test]
    fn test_add_message() {
        let mut manager = ContextManager::new(ContextConfig::default());
        
        let msg = Message::new(MessageRole::User, "Hello".to_string());
        manager.add_message(msg);
        
        assert_eq!(manager.message_count(), 1);
        assert!(manager.token_count() > 0);
    }

    #[test]
    fn test_priority_pruning() {
        let mut config = ContextConfig::default();
        config.max_tokens = 100; // Very small for testing
        config.pruning_strategy = PruningStrategy::PriorityBased;
        
        let mut manager = ContextManager::new(config);
        
        // Add low priority messages
        for i in 0..10 {
            let msg = Message::new(MessageRole::User, format!("Message {}", i));
            manager.add_message_with_priority(msg, MessagePriority::Low);
        }
        
        // Add high priority message
        let important_msg = Message::new(MessageRole::User, "Important!".to_string());
        manager.add_message_with_priority(important_msg, MessagePriority::High);
        
        // Should have pruned some messages
        assert!(manager.message_count() < 11);
    }

    #[test]
    fn test_pinned_message() {
        let mut config = ContextConfig::default();
        config.max_tokens = 50;
        
        let mut manager = ContextManager::new(config);
        
        // Add pinned message
        let pinned = Message::new(MessageRole::System, "System prompt".to_string());
        manager.add_pinned_message(pinned);
        
        // Add many normal messages
        for i in 0..20 {
            let msg = Message::new(MessageRole::User, format!("Message {}", i));
            manager.add_message(msg);
        }
        
        // Pinned message should still be there
        assert!(manager.messages.iter().any(|m| m.pinned));
    }

    #[test]
    fn test_context_stats() {
        let mut manager = ContextManager::new(ContextConfig::default());
        
        let msg = Message::new(MessageRole::User, "Test".to_string());
        manager.add_message(msg);
        
        let stats = manager.stats();
        assert_eq!(stats.message_count, 1);
        assert!(stats.token_count > 0);
        assert!(stats.utilization_percent() < 100.0);
    }
}
