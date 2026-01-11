use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::types::{LLMInstance, PermissionScope, LockdownState};
use llm_pool::LLMPool;
use security_engine::SecurityEngineImpl;
use context_manager::DatabaseContextManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub lockdown: LockdownState,
    pub active_llms: Vec<String>,
    pub pending_approvals: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub filename: String,
    pub size: usize,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    pub indexed: bool,
    pub chunk_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub llm_id: Option<String>,
    pub action: String,
    pub approved: bool,
    pub reason: Option<String>,
}

/// Application state shared across Tauri commands
/// Note: All fields are skipped during serialization as they contain
/// thread-safe locks (Arc<RwLock>) that cannot be serialized
#[derive(Serialize, Deserialize)]
pub struct AppState {
    #[serde(skip)]
    pub llm_pool: Arc<RwLock<LLMPool>>,
    #[serde(skip)]
    pub security_engine: Arc<SecurityEngineImpl>,
    #[serde(skip)]
    pub permissions: Arc<RwLock<PermissionScope>>,
    #[serde(skip)]
    pub documents: Arc<RwLock<Vec<Document>>>,
    #[serde(skip)]
    pub audit_log: Arc<RwLock<Vec<AuditLogEntry>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            llm_pool: Arc::new(RwLock::new(LLMPool::new())),
            security_engine: Arc::new(SecurityEngineImpl::new()),
            permissions: Arc::new(RwLock::new(PermissionScope::default())),
            documents: Arc::new(RwLock::new(Vec::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_system_state(&self) -> SystemState {
        let pool = self.llm_pool.read().await;
        let lockdown = self.security_engine
            .lockdown_state()
            .await
            .unwrap_or(LockdownState::Normal);

        SystemState {
            lockdown,
            active_llms: pool.get_all_loaded()
                .iter()
                .map(|llm| llm.instance().id.clone())
                .collect(),
            pending_approvals: 0, // TODO: Track pending approvals
        }
    }
}
