use common::types::AuditLogEntry;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};
use uuid::Uuid;
use chrono::Utc;

/// Audit logger for tracking all system actions
pub struct AuditLogger {
    /// In-memory log (in production, this would be a database)
    logs: Arc<RwLock<Vec<AuditLogEntry>>>,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log an action
    pub async fn log(
        &self,
        llm_id: Option<String>,
        action: String,
        details: serde_json::Value,
        approved: bool,
        reason: Option<String>,
    ) {
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            llm_id,
            action: action.clone(),
            details,
            approved,
            reason,
        };

        debug!("📋 Audit log: {} - {}", action, if approved { "✅" } else { "❌" });

        let mut logs = self.logs.write().await;
        logs.push(entry);
    }

    /// Get all logs
    pub async fn get_all(&self) -> Vec<AuditLogEntry> {
        let logs = self.logs.read().await;
        logs.clone()
    }

    /// Get logs for a specific LLM
    pub async fn get_by_llm(&self, llm_id: &str) -> Vec<AuditLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .filter(|log| {
                log.llm_id.as_ref().map(|id| id == llm_id).unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Get denied actions
    pub async fn get_denied(&self) -> Vec<AuditLogEntry> {
        let logs = self.logs.read().await;
        logs.iter()
            .filter(|log| !log.approved)
            .cloned()
            .collect()
    }

    /// Clear all logs
    pub async fn clear(&self) {
        let mut logs = self.logs.write().await;
        logs.clear();
        info!("🗑️  Audit logs cleared");
    }

    /// Get log count
    pub async fn count(&self) -> usize {
        let logs = self.logs.read().await;
        logs.len()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
