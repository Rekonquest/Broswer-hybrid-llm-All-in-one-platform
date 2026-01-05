use common::{
    errors::{Result, HybridLLMError},
    messages::PermissionType,
    traits::{SecurityEngine, SecurityAnalysis},
    types::{LockdownState, LockdownReason},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::{Guardrails, PermissionManager, AuditLogger};

/// Implementation of the SecurityEngine trait
pub struct SecurityEngineImpl {
    guardrails: Arc<Guardrails>,
    permissions: Arc<PermissionManager>,
    audit: Arc<AuditLogger>,
    lockdown_state: Arc<RwLock<LockdownState>>,
}

impl SecurityEngineImpl {
    pub fn new() -> Self {
        Self {
            guardrails: Arc::new(Guardrails::new()),
            permissions: Arc::new(PermissionManager::new()),
            audit: Arc::new(AuditLogger::new()),
            lockdown_state: Arc::new(RwLock::new(LockdownState::Normal)),
        }
    }

    /// Get the permission manager
    pub fn permissions(&self) -> Arc<PermissionManager> {
        Arc::clone(&self.permissions)
    }

    /// Get the audit logger
    pub fn audit(&self) -> Arc<AuditLogger> {
        Arc::clone(&self.audit)
    }
}

#[async_trait::async_trait]
impl SecurityEngine for SecurityEngineImpl {
    async fn check_permission(
        &self,
        llm_id: &str,
        permission: &PermissionType,
        explanation: &str,
    ) -> Result<bool> {
        // Check current lockdown state
        let state = self.lockdown_state.read().await;
        if *state == LockdownState::Locked {
            error!("🔒 System locked, denying permission request");
            return Ok(false);
        }
        drop(state);

        // Check permission
        let granted = self.permissions
            .check_permission(llm_id, permission, explanation)
            .await?;

        // Log the decision
        self.audit
            .log(
                Some(llm_id.to_string()),
                format!("Permission request: {:?}", permission),
                serde_json::json!({
                    "permission": permission,
                    "explanation": explanation,
                }),
                granted,
                if !granted {
                    Some("Permission denied by policy".to_string())
                } else {
                    None
                },
            )
            .await;

        // Check if too many failed requests
        if !granted {
            let failed_count = self.permissions.get_failed_count(llm_id).await;
            if failed_count >= 5 {
                warn!("⚠️  LLM {} has {} failed requests, triggering lockdown", llm_id, failed_count);
                self.trigger_lockdown(LockdownReason::MultipleFailedRequests {
                    count: failed_count,
                })
                .await?;
            }
        }

        Ok(granted)
    }

    async fn analyze_command(&self, command: &str) -> Result<SecurityAnalysis> {
        let analysis = self.guardrails.analyze_command(command)?;

        // Log the analysis
        self.audit
            .log(
                None,
                "Command analysis".to_string(),
                serde_json::json!({
                    "command": command,
                    "safe": analysis.safe,
                    "risk_level": format!("{:?}", analysis.risk_level),
                    "issues": analysis.issues,
                }),
                analysis.safe,
                if !analysis.safe {
                    Some(format!("Risk level: {:?}", analysis.risk_level))
                } else {
                    None
                },
            )
            .await;

        Ok(analysis)
    }

    async fn trigger_lockdown(&self, reason: LockdownReason) -> Result<()> {
        error!("🚨 LOCKDOWN TRIGGERED: {:?}", reason);

        let mut state = self.lockdown_state.write().await;
        *state = LockdownState::Locked;

        // Log the lockdown
        self.audit
            .log(
                None,
                "Lockdown triggered".to_string(),
                serde_json::json!({
                    "reason": format!("{:?}", reason),
                }),
                false,
                Some(format!("Lockdown: {:?}", reason)),
            )
            .await;

        info!("🔒 System is now in LOCKDOWN mode - read-only access");

        Ok(())
    }

    async fn release_lockdown(&self, auth_token: &str) -> Result<()> {
        // TODO: Implement proper authentication
        // For now, accept any non-empty token
        if auth_token.is_empty() {
            return Err(HybridLLMError::PermissionDenied(
                "Invalid authentication token".to_string(),
            ));
        }

        info!("🔓 Releasing lockdown with auth token");

        let mut state = self.lockdown_state.write().await;
        *state = LockdownState::Normal;

        // Log the release
        self.audit
            .log(
                None,
                "Lockdown released".to_string(),
                serde_json::json!({
                    "authenticated": true,
                }),
                true,
                Some("User authenticated".to_string()),
            )
            .await;

        info!("✅ Lockdown released - normal operations resumed");

        Ok(())
    }

    async fn lockdown_state(&self) -> Result<LockdownState> {
        let state = self.lockdown_state.read().await;
        Ok(*state)
    }
}

impl Default for SecurityEngineImpl {
    fn default() -> Self {
        Self::new()
    }
}
