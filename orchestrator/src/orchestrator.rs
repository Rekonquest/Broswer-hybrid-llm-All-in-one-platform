use common::{
    messages::OrchestratorMessage,
    errors::Result,
    types::LockdownState,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, error};

use crate::{message_bus::MessageBus, router::Router};

/// Main orchestrator that coordinates all system components
pub struct Orchestrator {
    /// Message bus for inter-component communication
    message_bus: Arc<MessageBus>,
    /// Router for task delegation
    router: Arc<RwLock<Router>>,
    /// Current system lockdown state
    lockdown_state: Arc<RwLock<LockdownState>>,
}

impl Orchestrator {
    /// Create a new orchestrator instance
    pub async fn new() -> Result<Self> {
        info!("🏗️  Initializing orchestrator...");

        let message_bus = Arc::new(MessageBus::new(1000));
        let router = Arc::new(RwLock::new(Router::new()));
        let lockdown_state = Arc::new(RwLock::new(LockdownState::Normal));

        Ok(Self {
            message_bus,
            router,
            lockdown_state,
        })
    }

    /// Run the orchestrator
    pub async fn run(self) -> Result<()> {
        info!("▶️  Starting orchestrator event loop...");

        // Subscribe to message bus
        let mut receiver = self.message_bus.subscribe();

        // Main event loop
        loop {
            tokio::select! {
                Ok(message) = receiver.recv() => {
                    self.handle_message(message).await?;
                }
            }
        }
    }

    /// Handle incoming messages
    async fn handle_message(&self, message: OrchestratorMessage) -> Result<()> {
        debug!("📨 Handling message: {:?}", message);

        // Check lockdown state before processing
        let lockdown = self.lockdown_state.read().await;
        if *lockdown == LockdownState::Locked {
            error!("🔒 System is locked down, rejecting message");
            return Ok(());
        }
        drop(lockdown); // Release the lock

        match message {
            OrchestratorMessage::UserRequest { id, content, context } => {
                self.handle_user_request(id, content, context).await?;
            }
            OrchestratorMessage::LLMDelegation { id, from, to, task, callback } => {
                self.handle_llm_delegation(id, from, to, task, callback).await?;
            }
            OrchestratorMessage::SecurityAlert { id, severity, reason, llm_id, suggested_action } => {
                self.handle_security_alert(id, severity, reason, llm_id, suggested_action).await?;
            }
            OrchestratorMessage::StateChange { id, change_type, data } => {
                self.handle_state_change(id, change_type, data).await?;
            }
            _ => {
                debug!("Unhandled message type, passing through");
            }
        }

        Ok(())
    }

    async fn handle_user_request(
        &self,
        id: uuid::Uuid,
        content: String,
        context: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        info!("👤 Handling user request: {}", id);
        // TODO: Classify task and route to appropriate LLM
        Ok(())
    }

    async fn handle_llm_delegation(
        &self,
        id: uuid::Uuid,
        from: String,
        to: Option<String>,
        task: common::messages::TaskDescription,
        callback: bool,
    ) -> Result<()> {
        info!("🔄 LLM {} delegating task to {:?}", from, to);

        let target_llm = if let Some(to_id) = to {
            to_id
        } else {
            // Route based on task requirements
            let router = self.router.read().await;
            router.route_task(&task)?
        };

        info!("✅ Routed to LLM: {}", target_llm);
        // TODO: Forward to LLM pool manager
        Ok(())
    }

    async fn handle_security_alert(
        &self,
        id: uuid::Uuid,
        severity: common::messages::AlertSeverity,
        reason: String,
        llm_id: Option<String>,
        suggested_action: common::messages::SuggestedAction,
    ) -> Result<()> {
        error!("🚨 Security alert [{:?}]: {} (LLM: {:?})", severity, reason, llm_id);

        match suggested_action {
            common::messages::SuggestedAction::Lockdown => {
                info!("🔒 Triggering lockdown");
                let mut lockdown = self.lockdown_state.write().await;
                *lockdown = LockdownState::Locked;
            }
            common::messages::SuggestedAction::RequestHumanReview => {
                info!("👨‍💼 Requesting human review");
                // TODO: Notify UI
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_state_change(
        &self,
        id: uuid::Uuid,
        change_type: common::messages::StateChangeType,
        data: serde_json::Value,
    ) -> Result<()> {
        info!("📊 State change: {:?}", change_type);

        match change_type {
            common::messages::StateChangeType::LockdownTriggered => {
                let mut lockdown = self.lockdown_state.write().await;
                *lockdown = LockdownState::Locked;
            }
            common::messages::StateChangeType::LockdownReleased => {
                let mut lockdown = self.lockdown_state.write().await;
                *lockdown = LockdownState::Normal;
            }
            common::messages::StateChangeType::LLMLoaded => {
                // TODO: Update router
            }
            common::messages::StateChangeType::LLMUnloaded => {
                // TODO: Update router
            }
            _ => {}
        }

        Ok(())
    }

    /// Get the message bus (for components to publish messages)
    pub fn message_bus(&self) -> Arc<MessageBus> {
        Arc::clone(&self.message_bus)
    }
}
