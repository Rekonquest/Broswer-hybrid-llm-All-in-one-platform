use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents the different types of LLM providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LLMProvider {
    /// Local model via llama.cpp
    Local(String), // model name
    /// Claude API
    Claude,
    /// OpenAI API
    OpenAI,
    /// Google Gemini API
    Gemini,
}

/// Capabilities that an LLM can have
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Code,
    Security,
    General,
    Analysis,
    Creative,
}

/// LLM instance identifier and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMInstance {
    pub id: String,
    pub provider: LLMProvider,
    pub capabilities: Vec<Capability>,
    pub model_name: String,
    pub max_context: usize,
    pub is_loaded: bool,
}

/// Context types for LLM operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextType {
    Global,
    PerLLM { llm_id: String },
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Permission scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionScope {
    pub file_system: FileSystemPermissions,
    pub network: NetworkPermissions,
    pub commands: CommandPermissions,
    pub resources: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemPermissions {
    pub read_paths: Vec<String>,
    pub write_paths: Vec<String>,
    pub execute_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermissions {
    pub inbound: bool,
    pub outbound: bool,
    pub require_approval: Vec<String>, // glob patterns
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPermissions {
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
    pub require_explanation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_percent: f32,
    pub max_memory_gb: f32,
    pub max_disk_gb: f32,
}

/// Security lockdown state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LockdownState {
    Normal,
    ReadOnly,
    Locked,
}

/// Lockdown trigger reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockdownReason {
    PolicyViolation { details: String },
    SuspiciousPattern { pattern: String },
    ResourceExceeded { resource: String, limit: f32, actual: f32 },
    UserPanicButton,
    MultipleFailedRequests { count: usize },
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub llm_id: Option<String>,
    pub action: String,
    pub details: serde_json::Value,
    pub approved: bool,
    pub reason: Option<String>,
}

/// Task classification for routing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Code,
    Security,
    General,
    Analysis,
    MultiStep,
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub id: Uuid,
    pub network_enabled: bool,
    pub cpu_limit: f32,
    pub memory_limit_gb: f32,
    pub disk_limit_gb: f32,
    pub allowed_commands: Vec<String>,
}

/// Artifact transfer request from sandbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactTransfer {
    pub sandbox_id: Uuid,
    pub file_path: String,
    pub destination: String,
    pub explanation: String,
    pub approved: Option<bool>,
}

impl Default for PermissionScope {
    fn default() -> Self {
        Self {
            file_system: FileSystemPermissions {
                read_paths: vec![
                    "/home/*/downloads/*".to_string(),
                    "/rag/*".to_string(),
                ],
                write_paths: vec!["/home/*/downloads/*".to_string()],
                execute_paths: vec![],
            },
            network: NetworkPermissions {
                inbound: true,
                outbound: true,
                require_approval: vec!["*".to_string()],
            },
            commands: CommandPermissions {
                whitelist: vec![
                    "git".to_string(),
                    "npm".to_string(),
                    "python".to_string(),
                    "cargo".to_string(),
                    "ls".to_string(),
                    "cat".to_string(),
                ],
                blacklist: vec![
                    "rm -rf /".to_string(),
                    "sudo".to_string(),
                    "dd".to_string(),
                    "mkfs".to_string(),
                ],
                require_explanation: true,
            },
            resources: ResourceLimits {
                max_cpu_percent: 80.0,
                max_memory_gb: 8.0,
                max_disk_gb: 50.0,
            },
        }
    }
}
