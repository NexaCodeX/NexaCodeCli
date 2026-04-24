//! LLM client trait and implementation

use anyhow::{Context, Result};
use reqwest::Client;
use tracing::{debug, info};

use super::types::*;
use super::config::{LlmConfig, ProviderType};

/// LLM Client trait
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    /// Call the LLM with a request
    async fn call(&self, request: LlmRequest) -> Result<LlmResponse>;
}

/// HTTP-based LLM client
pub struct HttpLlmClient {
    config: LlmConfig,
    http_client: Client,
}

impl HttpLlmClient {
    /// Create a new HTTP LLM client
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Get the provider type
    fn provider_type(&self) -> ProviderType {
        self.config.provider_type()
    }

    /// Call OpenAI-compatible API
    async fn call_openai_compatible(
        &self,
        request: LlmRequest,
        base_url: &str,
        api_key: &str,
    ) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        
        debug!("Calling OpenAI-compatible API: {}", url);
        
        // Build OpenAI request
        let openai_request = OpenAIRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|m| OpenAIMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            }).collect(),
            max_tokens: Some(request.max_tokens),
            temperature: request.temperature,
            tools: request.tools.iter().map(|t| OpenAITool {
                tool_type: "function".to_string(),
                function: OpenAIFunction {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.parameters.clone(),
                },
            }).collect(),
        };

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API error: {} - {}", status, body);
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .context("Failed to parse LLM API response")?;

        // Extract response
        if let Some(choice) = openai_response.choices.first() {
            // Check for tool calls
            if !choice.message.tool_calls.is_empty() {
                let tool_call = &choice.message.tool_calls[0];
                let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                    .unwrap_or(serde_json::Value::Object(Default::default()));
                
                return Ok(LlmResponse::ToolCall {
                    name: tool_call.function.name.clone(),
                    arguments,
                });
            }

            // Return text response
            let content = choice.message.content.clone().unwrap_or_default();
            Ok(LlmResponse::Text(content))
        } else {
            anyhow::bail!("No response choices from LLM API");
        }
    }

    /// Call Anthropic API
    async fn call_anthropic(
        &self,
        request: LlmRequest,
        base_url: &str,
        api_key: &str,
    ) -> Result<LlmResponse> {
        let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));
        
        debug!("Calling Anthropic API: {}", url);
        
        // Separate system message from other messages
        let mut system_prompt = None;
        let messages: Vec<AnthropicMessage> = request.messages.iter()
            .filter_map(|m| {
                if m.role == "system" {
                    system_prompt = Some(m.content.clone());
                    None
                } else {
                    Some(AnthropicMessage {
                        role: m.role.clone(),
                        content: m.content.clone(),
                    })
                }
            })
            .collect();

        let anthropic_request = AnthropicRequest {
            model: request.model.clone(),
            messages,
            max_tokens: Some(request.max_tokens),
            system: system_prompt,
        };

        let response = self.http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error: {} - {}", status, body);
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic API response")?;

        // Extract text from content blocks
        let text = anthropic_response.content.iter()
            .filter_map(|c| {
                if c.content_type == "text" {
                    c.text.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        if text.is_empty() {
            anyhow::bail!("No text content in Anthropic response");
        }

        Ok(LlmResponse::Text(text))
    }
}

#[async_trait::async_trait]
impl LlmClient for HttpLlmClient {
    async fn call(&self, request: LlmRequest) -> Result<LlmResponse> {
        let provider_type = self.provider_type();
        let base_url = self.config.current_base_url();
        let api_key = self.config.current_api_key();

        if api_key.is_empty() {
            anyhow::bail!("No API key configured for provider: {}", self.config.current_provider_name());
        }

        info!("Calling LLM: provider={}, model={}", self.config.current_provider_name(), request.model);

        match provider_type {
            ProviderType::Anthropic => {
                self.call_anthropic(request, &base_url, &api_key).await
            }
            ProviderType::OpenAI => {
                self.call_openai_compatible(request, &base_url, &api_key).await
            }
        }
    }
}
