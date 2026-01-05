use common::{
    errors::Result,
    traits::{ContextManager, RAGResult},
    types::Message,
};
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, debug};

/// Context manager implementation
pub struct ContextManagerImpl {
    /// Global context shared across all LLMs
    global_context: Arc<DashMap<String, serde_json::Value>>,
    /// Per-LLM private context
    llm_contexts: Arc<DashMap<String, HashMap<String, serde_json::Value>>>,
    /// Conversation storage (in-memory for now, will use PostgreSQL)
    conversations: Arc<DashMap<uuid::Uuid, Vec<Message>>>,
}

impl ContextManagerImpl {
    pub fn new() -> Self {
        Self {
            global_context: Arc::new(DashMap::new()),
            llm_contexts: Arc::new(DashMap::new()),
            conversations: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl ContextManager for ContextManagerImpl {
    async fn get_global_context(&self) -> Result<HashMap<String, serde_json::Value>> {
        let context: HashMap<_, _> = self
            .global_context
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        Ok(context)
    }

    async fn update_global_context(&self, key: &str, value: serde_json::Value) -> Result<()> {
        debug!("🌍 Updating global context: {}", key);
        self.global_context.insert(key.to_string(), value);
        Ok(())
    }

    async fn get_llm_context(&self, llm_id: &str) -> Result<HashMap<String, serde_json::Value>> {
        Ok(self
            .llm_contexts
            .get(llm_id)
            .map(|ctx| ctx.clone())
            .unwrap_or_default())
    }

    async fn update_llm_context(
        &self,
        llm_id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<()> {
        debug!("🔧 Updating LLM context for {}: {}", llm_id, key);

        self.llm_contexts
            .entry(llm_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value);

        Ok(())
    }

    async fn get_conversation(&self, conversation_id: &uuid::Uuid) -> Result<Vec<Message>> {
        Ok(self
            .conversations
            .get(conversation_id)
            .map(|conv| conv.clone())
            .unwrap_or_default())
    }

    async fn add_message(&self, conversation_id: &uuid::Uuid, message: Message) -> Result<()> {
        debug!("💬 Adding message to conversation {}", conversation_id);

        self.conversations
            .entry(*conversation_id)
            .or_insert_with(Vec::new)
            .push(message);

        Ok(())
    }

    async fn search_rag(&self, query: &str, llm_id: Option<&str>, limit: usize) -> Result<Vec<RAGResult>> {
        // TODO: Implement actual RAG search with pgvector
        // For now, return empty results
        debug!("🔍 RAG search: {} (LLM: {:?}, limit: {})", query, llm_id, limit);
        Ok(Vec::new())
    }
}

impl Default for ContextManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}
