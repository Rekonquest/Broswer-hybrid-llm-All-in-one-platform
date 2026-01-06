use common::{
    errors::{Result, HybridLLMError},
    traits::{ContextManager, RAGResult},
    types::Message,
};
use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgPoolOptions, Row};
use std::collections::HashMap;
use tracing::{info, debug, error};

/// PostgreSQL-backed context manager with RAG support
pub struct DatabaseContextManager {
    pool: PgPool,
}

impl DatabaseContextManager {
    /// Create a new database context manager
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("🔌 Connecting to PostgreSQL database...");

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| HybridLLMError::DatabaseError(format!("Failed to connect: {}", e)))?;

        info!("✅ Connected to PostgreSQL");

        Ok(Self { pool })
    }

    /// Get the database pool for direct access
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl ContextManager for DatabaseContextManager {
    async fn get_global_context(&self) -> Result<HashMap<String, serde_json::Value>> {
        debug!("📖 Reading global context from database");

        let rows = sqlx::query("SELECT context_key, context_value FROM global_context")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;

        let mut context = HashMap::new();
        for row in rows {
            let key: String = row.try_get("context_key")
                .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;
            let value: serde_json::Value = row.try_get("context_value")
                .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;
            context.insert(key, value);
        }

        Ok(context)
    }

    async fn update_global_context(&self, key: &str, value: serde_json::Value) -> Result<()> {
        debug!("💾 Updating global context: {}", key);

        sqlx::query(
            "INSERT INTO global_context (context_key, context_value) \
             VALUES ($1, $2) \
             ON CONFLICT (context_key) DO UPDATE \
             SET context_value = $2, updated_at = NOW()"
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_llm_context(&self, llm_id: &str) -> Result<HashMap<String, serde_json::Value>> {
        debug!("📖 Reading LLM context for: {}", llm_id);

        let rows = sqlx::query("SELECT context_key, context_value FROM llm_contexts WHERE llm_id = $1")
            .bind(llm_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;

        let mut context = HashMap::new();
        for row in rows {
            let key: String = row.try_get("context_key")
                .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;
            let value: serde_json::Value = row.try_get("context_value")
                .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;
            context.insert(key, value);
        }

        Ok(context)
    }

    async fn update_llm_context(
        &self,
        llm_id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<()> {
        debug!("💾 Updating LLM context for {}: {}", llm_id, key);

        sqlx::query(
            "INSERT INTO llm_contexts (llm_id, context_key, context_value) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (llm_id, context_key) DO UPDATE \
             SET context_value = $3, updated_at = NOW()"
        )
        .bind(llm_id)
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_conversation(&self, conversation_id: &uuid::Uuid) -> Result<Vec<Message>> {
        debug!("📖 Reading conversation: {}", conversation_id);

        let rows = sqlx::query(
            "SELECT id, role, content, timestamp, metadata \
             FROM messages \
             WHERE conversation_id = $1 \
             ORDER BY timestamp ASC"
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;

        let mut messages = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.try_get("id")
                .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;
            let role_str: String = row.try_get("role")
                .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;
            let content: String = row.try_get("content")
                .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;
            let timestamp: chrono::NaiveDateTime = row.try_get("timestamp")
                .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;
            let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();

            let role = match role_str.as_str() {
                "user" => common::types::MessageRole::User,
                "assistant" => common::types::MessageRole::Assistant,
                "system" => common::types::MessageRole::System,
                _ => common::types::MessageRole::User,
            };

            let metadata_map = metadata
                .and_then(|v| v.as_object().map(|o| o.clone()))
                .unwrap_or_default()
                .into_iter()
                .collect();

            messages.push(Message {
                id,
                role,
                content,
                timestamp: chrono::DateTime::from_naive_utc_and_offset(timestamp, chrono::Utc),
                metadata: metadata_map,
            });
        }

        Ok(messages)
    }

    async fn add_message(&self, conversation_id: &uuid::Uuid, message: Message) -> Result<()> {
        debug!("💾 Adding message to conversation: {}", conversation_id);

        // Ensure conversation exists
        sqlx::query("INSERT INTO conversations (id) VALUES ($1) ON CONFLICT (id) DO NOTHING")
            .bind(conversation_id)
            .execute(&self.pool)
            .await
            .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;

        // Add message
        let role_str = match message.role {
            common::types::MessageRole::User => "user",
            common::types::MessageRole::Assistant => "assistant",
            common::types::MessageRole::System => "system",
        };

        let metadata_json = serde_json::to_value(&message.metadata)
            .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content, timestamp, metadata) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(message.id)
        .bind(conversation_id)
        .bind(role_str)
        .bind(message.content)
        .bind(message.timestamp.naive_utc())
        .bind(metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| HybridLLMError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn search_rag(&self, query: &str, llm_id: Option<&str>, limit: usize) -> Result<Vec<RAGResult>> {
        debug!("🔍 RAG search: {} (LLM: {:?}, limit: {})", query, llm_id, limit);

        // TODO: Implement actual vector search
        // For now, this is a placeholder
        // Full implementation would:
        // 1. Generate embedding for query
        // 2. Perform vector similarity search
        // 3. Filter by llm_visibility if llm_id is provided
        // 4. Return top-k results

        error!("⚠️  RAG vector search not yet implemented (requires embeddings)");

        Ok(Vec::new())
    }
}
