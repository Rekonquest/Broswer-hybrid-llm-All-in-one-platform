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

/// OpenAI API adapter
pub struct OpenAIAdapter {
    client: Client,
    api_key: String,
    instance: LLMInstance,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: OpenAIMessage,
}

impl OpenAIAdapter {
    pub fn new(api_key: String, model: String) -> Self {
        let instance = LLMInstance {
            id: format!("openai-{}", model),
            provider: LLMProviderType::OpenAI,
            capabilities: vec![
                Capability::Code,
                Capability::General,
                Capability::Analysis,
                Capability::Creative,
            ],
            model_name: model,
            max_context: 128_000, // GPT-4 Turbo context
            is_loaded: true,
        };

        Self {
            client: Client::new(),
            api_key,
            instance,
        }
    }
}

#[async_trait]
impl LLMProvider for OpenAIAdapter {
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
        debug!("🤖 Calling OpenAI API...");

        let mut messages = Vec::new();

        if let Some(system) = context.get("system").and_then(|v| v.as_str()) {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: system.to_string(),
            });
        }

        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        let request = OpenAIRequest {
            model: self.instance.model_name.clone(),
            messages,
            max_tokens: Some(4096),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| HybridLLMError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("OpenAI API error: {}", error_text);
            return Err(HybridLLMError::LLMError(format!(
                "OpenAI API error: {}",
                error_text
            )));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| HybridLLMError::LLMError(e.to_string()))?;

        let text = openai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        Ok(text)
    }

    async fn complete_stream(
        &self,
        prompt: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String>>> {
        // TODO: Implement streaming
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let result = self.complete(prompt, context).await;

        tokio::spawn(async move {
            let _ = tx.send(result).await;
        });

        Ok(rx)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn load(&mut self) -> Result<()> {
        Ok(())
    }

    async fn unload(&mut self) -> Result<()> {
        Ok(())
    }
}
