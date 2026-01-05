use async_trait::async_trait;
use std::collections::HashMap;

use crate::{
    errors::Result,
    types::{Capability, LLMInstance, Message},
};

/// Trait that all LLM providers must implement
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Get provider capabilities
    fn capabilities(&self) -> Vec<Capability>;

    /// Get provider instance info
    fn instance(&self) -> &LLMInstance;

    /// Complete a prompt with context
    async fn complete(
        &self,
        prompt: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<String>;

    /// Stream a completion (returns chunks)
    async fn complete_stream(
        &self,
        prompt: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String>>>;

    /// Check if the provider is healthy
    async fn health_check(&self) -> Result<bool>;

    /// Load the model (for local models)
    async fn load(&mut self) -> Result<()>;

    /// Unload the model (to free memory)
    async fn unload(&mut self) -> Result<()>;
}

/// Trait for the security engine
#[async_trait]
pub trait SecurityEngine: Send + Sync {
    /// Check if a permission request should be granted
    async fn check_permission(
        &self,
        llm_id: &str,
        permission: &crate::messages::PermissionType,
        explanation: &str,
    ) -> Result<bool>;

    /// Analyze if a command is safe
    async fn analyze_command(&self, command: &str) -> Result<SecurityAnalysis>;

    /// Trigger lockdown
    async fn trigger_lockdown(&self, reason: crate::types::LockdownReason) -> Result<()>;

    /// Release lockdown (requires authentication)
    async fn release_lockdown(&self, auth_token: &str) -> Result<()>;

    /// Get current lockdown state
    async fn lockdown_state(&self) -> Result<crate::types::LockdownState>;
}

#[derive(Debug, Clone)]
pub struct SecurityAnalysis {
    pub safe: bool,
    pub risk_level: RiskLevel,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Trait for context management
#[async_trait]
pub trait ContextManager: Send + Sync {
    /// Get global context
    async fn get_global_context(&self) -> Result<HashMap<String, serde_json::Value>>;

    /// Update global context
    async fn update_global_context(
        &self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<()>;

    /// Get per-LLM context
    async fn get_llm_context(&self, llm_id: &str) -> Result<HashMap<String, serde_json::Value>>;

    /// Update per-LLM context
    async fn update_llm_context(
        &self,
        llm_id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<()>;

    /// Get conversation history
    async fn get_conversation(&self, conversation_id: &uuid::Uuid) -> Result<Vec<Message>>;

    /// Add message to conversation
    async fn add_message(&self, conversation_id: &uuid::Uuid, message: Message) -> Result<()>;

    /// Search RAG context
    async fn search_rag(&self, query: &str, llm_id: Option<&str>, limit: usize) -> Result<Vec<RAGResult>>;
}

#[derive(Debug, Clone)]
pub struct RAGResult {
    pub id: uuid::Uuid,
    pub content: String,
    pub similarity: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}
