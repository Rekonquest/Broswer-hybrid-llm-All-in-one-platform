use common::{
    errors::{Result, HybridLLMError},
    traits::LLMProvider,
    types::{Capability, LLMInstance, LLMProvider as LLMProviderType},
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error};

/// Claude API adapter
pub struct ClaudeAdapter {
    client: Client,
    api_key: String,
    instance: LLMInstance,
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

impl ClaudeAdapter {
    pub fn new(api_key: String, model: String) -> Self {
        let instance = LLMInstance {
            id: format!("claude-{}", model),
            provider: LLMProviderType::Claude,
            capabilities: vec![
                Capability::Code,
                Capability::General,
                Capability::Analysis,
                Capability::Creative,
            ],
            model_name: model,
            max_context: 200_000, // Claude 3.5 Sonnet context window
            is_loaded: true, // Cloud models are always "loaded"
        };

        Self {
            client: Client::new(),
            api_key,
            instance,
        }
    }
}

#[async_trait]
impl LLMProvider for ClaudeAdapter {
    fn capabilities(&self) -> Vec<Capability> {
        self.instance.capabilities.clone()
    }

    fn instance(&self) -> &LLMInstance {
        &self.instance
    }

    async fn complete(
        &self,
        prompt: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        debug!("🤖 Calling Claude API...");

        let system_prompt = context
            .get("system")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let request = ClaudeRequest {
            model: self.instance.model_name.clone(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 4096,
            system: system_prompt,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| HybridLLMError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Claude API error: {}", error_text);
            return Err(HybridLLMError::LLMError(format!(
                "Claude API error: {}",
                error_text
            )));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| HybridLLMError::LLMError(e.to_string()))?;

        let text = claude_response
            .content
            .first()
            .map(|block| block.text.clone())
            .unwrap_or_default();

        Ok(text)
    }

    async fn complete_stream(
        &self,
        prompt: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String>>> {
        // TODO: Implement streaming
        // For now, return non-streaming response
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let result = self.complete(prompt, context).await;

        tokio::spawn(async move {
            let _ = tx.send(result).await;
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool> {
        // Simple health check - could ping the API
        Ok(true)
    }

    async fn load(&mut self) -> Result<()> {
        // Cloud models don't need loading
        Ok(())
    }

    async fn unload(&mut self) -> Result<()> {
        // Cloud models don't need unloading
        Ok(())
    }
}
