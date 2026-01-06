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

/// Google Gemini API adapter
pub struct GeminiAdapter {
    client: Client,
    api_key: String,
    instance: LLMInstance,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

impl GeminiAdapter {
    pub fn new(api_key: String, model: String) -> Self {
        let instance = LLMInstance {
            id: format!("gemini-{}", model),
            provider: LLMProviderType::Gemini,
            capabilities: vec![
                Capability::Code,
                Capability::General,
                Capability::Analysis,
                Capability::Creative,
            ],
            model_name: model,
            max_context: 1_000_000, // Gemini 1.5 Pro context
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
impl LLMProvider for GeminiAdapter {
    fn capabilities(&self) -> Vec<Capability> {
        self.instance.capabilities.clone()
    }

    fn instance(&self) -> &LLMInstance {
        &self.instance
    }

    async fn complete(
        &self,
        prompt: &str,
        _context: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        debug!("🤖 Calling Gemini API...");

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.instance.model_name, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| HybridLLMError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Gemini API error: {}", error_text);
            return Err(HybridLLMError::LLMError(format!(
                "Gemini API error: {}",
                error_text
            )));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| HybridLLMError::LLMError(e.to_string()))?;

        let text = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
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
