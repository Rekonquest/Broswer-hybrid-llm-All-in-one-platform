use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::types::{Capability, TaskType};

/// Messages passed through the orchestrator's message bus
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrchestratorMessage {
    /// User request to be routed to appropriate LLM
    UserRequest {
        id: Uuid,
        content: String,
        context: HashMap<String, serde_json::Value>,
    },

    /// LLM delegation to another LLM
    LLMDelegation {
        id: Uuid,
        from: String,
        to: Option<String>, // None = orchestrator decides
        task: TaskDescription,
        callback: bool,
    },

    /// LLM response
    LLMResponse {
        id: Uuid,
        request_id: Uuid,
        llm_id: String,
        content: String,
        metadata: HashMap<String, serde_json::Value>,
    },

    /// Permission request
    PermissionRequest {
        id: Uuid,
        llm_id: String,
        permission_type: PermissionType,
        explanation: String,
    },

    /// Permission response
    PermissionResponse {
        id: Uuid,
        request_id: Uuid,
        granted: bool,
        reason: Option<String>,
    },

    /// Security alert
    SecurityAlert {
        id: Uuid,
        severity: AlertSeverity,
        reason: String,
        llm_id: Option<String>,
        suggested_action: SuggestedAction,
    },

    /// Sandbox request
    SandboxRequest {
        id: Uuid,
        llm_id: String,
        purpose: String,
    },

    /// Sandbox artifact approval request
    ArtifactApproval {
        id: Uuid,
        sandbox_id: Uuid,
        file_path: String,
        destination: String,
        explanation: String,
    },

    /// System state change
    StateChange {
        id: Uuid,
        change_type: StateChangeType,
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDescription {
    pub description: String,
    pub task_type: TaskType,
    pub required_capabilities: Vec<Capability>,
    pub context: HashMap<String, serde_json::Value>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionType {
    FileRead { path: String },
    FileWrite { path: String },
    FileExecute { path: String },
    Command { command: String },
    NetworkAccess { url: String },
    ResourceIncrease { resource: String, amount: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestedAction {
    Allow,
    Deny,
    Lockdown,
    RequestHumanReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateChangeType {
    LockdownTriggered,
    LockdownReleased,
    LLMLoaded,
    LLMUnloaded,
    PermissionGranted,
    PermissionDenied,
}
