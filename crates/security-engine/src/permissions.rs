use common::{
    errors::{Result, HybridLLMError},
    messages::PermissionType,
    types::PermissionScope,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

/// Manages permissions for LLMs
pub struct PermissionManager {
    /// Global default permissions
    global_scope: Arc<RwLock<PermissionScope>>,
    /// Per-LLM permission overrides
    llm_scopes: Arc<RwLock<HashMap<String, PermissionScope>>>,
    /// Failed permission request tracking
    failed_requests: Arc<RwLock<HashMap<String, usize>>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            global_scope: Arc::new(RwLock::new(PermissionScope::default())),
            llm_scopes: Arc::new(RwLock::new(HashMap::new())),
            failed_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a permission request should be granted
    pub async fn check_permission(
        &self,
        llm_id: &str,
        permission: &PermissionType,
        explanation: &str,
    ) -> Result<bool> {
        debug!("🔐 Checking permission for {}: {:?}", llm_id, permission);
        debug!("📝 Explanation: {}", explanation);

        // Get the applicable scope (LLM-specific or global)
        let scope = self.get_scope(llm_id).await;

        let granted = match permission {
            PermissionType::FileRead { path } => {
                self.check_file_access(&scope.file_system.read_paths, path)
            }
            PermissionType::FileWrite { path } => {
                self.check_file_access(&scope.file_system.write_paths, path)
            }
            PermissionType::FileExecute { path } => {
                self.check_file_access(&scope.file_system.execute_paths, path)
            }
            PermissionType::Command { command } => {
                self.check_command(&scope.commands, command)
            }
            PermissionType::NetworkAccess { url } => {
                // Network requires approval by default
                scope.network.outbound || scope.network.inbound
            }
            PermissionType::ResourceIncrease { resource, amount } => {
                self.check_resource_increase(&scope.resources, resource, *amount)
            }
        };

        if granted {
            info!("✅ Permission granted for {}: {:?}", llm_id, permission);
        } else {
            warn!("❌ Permission denied for {}: {:?}", llm_id, permission);
            self.track_failed_request(llm_id).await;
        }

        Ok(granted)
    }

    /// Get the applicable permission scope for an LLM
    async fn get_scope(&self, llm_id: &str) -> PermissionScope {
        let llm_scopes = self.llm_scopes.read().await;

        if let Some(scope) = llm_scopes.get(llm_id) {
            scope.clone()
        } else {
            let global = self.global_scope.read().await;
            global.clone()
        }
    }

    /// Check file access against allowed paths
    fn check_file_access(&self, allowed_paths: &[String], path: &str) -> bool {
        for allowed in allowed_paths {
            if Self::path_matches(allowed, path) {
                return true;
            }
        }
        false
    }

    /// Check if a path matches a pattern (simple glob-like matching)
    fn path_matches(pattern: &str, path: &str) -> bool {
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            path.starts_with(prefix)
        } else if pattern.contains('*') {
            // More complex glob patterns would need a proper glob library
            true // For now, allow
        } else {
            pattern == path
        }
    }

    /// Check if a command is allowed
    fn check_command(&self, cmd_perms: &common::types::CommandPermissions, command: &str) -> bool {
        // Extract the binary name from the command
        let binary = command.split_whitespace().next().unwrap_or("");

        // Check blacklist first
        for blocked in &cmd_perms.blacklist {
            if command.contains(blocked) {
                return false;
            }
        }

        // Check whitelist
        cmd_perms.whitelist.iter().any(|allowed| binary == allowed)
    }

    /// Check if resource increase is within limits
    fn check_resource_increase(
        &self,
        limits: &common::types::ResourceLimits,
        resource: &str,
        requested: f32,
    ) -> bool {
        match resource {
            "cpu" => requested <= limits.max_cpu_percent,
            "memory" => requested <= limits.max_memory_gb,
            "disk" => requested <= limits.max_disk_gb,
            _ => false,
        }
    }

    /// Track failed permission requests
    async fn track_failed_request(&self, llm_id: &str) {
        let mut failed = self.failed_requests.write().await;
        let count = failed.entry(llm_id.to_string()).or_insert(0);
        *count += 1;

        if *count >= 5 {
            warn!("⚠️  LLM {} has {} failed permission requests", llm_id, count);
        }
    }

    /// Get failed request count for an LLM
    pub async fn get_failed_count(&self, llm_id: &str) -> usize {
        let failed = self.failed_requests.read().await;
        failed.get(llm_id).copied().unwrap_or(0)
    }

    /// Reset failed request count
    pub async fn reset_failed_count(&self, llm_id: &str) {
        let mut failed = self.failed_requests.write().await;
        failed.remove(llm_id);
    }

    /// Set global permission scope
    pub async fn set_global_scope(&self, scope: PermissionScope) {
        let mut global = self.global_scope.write().await;
        *global = scope;
    }

    /// Set per-LLM permission scope
    pub async fn set_llm_scope(&self, llm_id: &str, scope: PermissionScope) {
        let mut scopes = self.llm_scopes.write().await;
        scopes.insert(llm_id.to_string(), scope);
    }

    /// Get global permission scope
    pub async fn get_global_scope(&self) -> PermissionScope {
        let global = self.global_scope.read().await;
        global.clone()
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_permission() {
        let manager = PermissionManager::new();

        let perm = PermissionType::FileRead {
            path: "/home/user/downloads/file.txt".to_string(),
        };

        let granted = manager
            .check_permission("test-llm", &perm, "Need to read file")
            .await
            .unwrap();

        assert!(granted);
    }

    #[tokio::test]
    async fn test_command_whitelist() {
        let manager = PermissionManager::new();

        let perm = PermissionType::Command {
            command: "git status".to_string(),
        };

        let granted = manager
            .check_permission("test-llm", &perm, "Check git status")
            .await
            .unwrap();

        assert!(granted);
    }

    #[tokio::test]
    async fn test_command_blacklist() {
        let manager = PermissionManager::new();

        let perm = PermissionType::Command {
            command: "rm -rf /".to_string(),
        };

        let granted = manager
            .check_permission("test-llm", &perm, "Delete files")
            .await
            .unwrap();

        assert!(!granted);
    }
}
