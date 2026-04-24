//! LLM types for request and response

use serde::{Deserialize, Serialize};

/// LLM message for API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

impl LlmMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

/// LLM request
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub provider_name: String,
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
    pub tools: Vec<crate::core::agent::ToolDefinition>,
}

/// LLM response
#[derive(Debug, Clone)]
pub enum LlmResponse {
    /// Text response
    Text(String),
    /// Tool call request
    ToolCall {
        name: String,
        arguments: serde_json::Value,
    },
    /// Error
    Error(String),
}

/// OpenAI-compatible API request format
#[derive(Debug, Serialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<OpenAITool>,
}

/// OpenAI message format
#[derive(Debug, Serialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI tool definition
#[derive(Debug, Serialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIFunction,
}

/// OpenAI function definition
#[derive(Debug, Serialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// OpenAI API response
#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<OpenAIChoice>,
}

/// OpenAI choice
#[derive(Debug, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIResponseMessage,
}

/// OpenAI response message
#[derive(Debug, Deserialize)]
pub struct OpenAIResponseMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<OpenAIToolCall>,
}

/// OpenAI tool call
#[derive(Debug, Deserialize)]
pub struct OpenAIToolCall {
    pub function: OpenAIToolCallFunction,
}

/// OpenAI tool call function
#[derive(Debug, Deserialize)]
pub struct OpenAIToolCallFunction {
    pub name: String,
    pub arguments: String,
}

/// Anthropic API request format
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

/// Anthropic message format
#[derive(Debug, Serialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

/// Anthropic API response
#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<AnthropicContent>,
}

/// Anthropic content block
#[derive(Debug, Deserialize)]
pub struct AnthropicContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}
