use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug)]
pub enum HybridLLMError {
    #[error("LLM error: {0}")]
    LLMError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),

    #[error("Lockdown active: {0}")]
    LockdownActive(String),

    #[error("Sandbox error: {0}")]
    SandboxError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("File system error: {0}")]
    FileSystemError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("LLM not found: {0}")]
    LLMNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Resource limit exceeded: {resource} (limit: {limit}, actual: {actual})")]
    ResourceLimitExceeded {
        resource: String,
        limit: f32,
        actual: f32,
    },

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Custom Serialize implementation for Tauri IPC bridge compatibility
// Converts the error to a string representation for serialization
impl Serialize for HybridLLMError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type Result<T> = std::result::Result<T, HybridLLMError>;
