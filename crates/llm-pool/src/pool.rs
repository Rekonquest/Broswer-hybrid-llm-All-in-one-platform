use common::{
    errors::{Result, HybridLLMError},
    traits::LLMProvider,
    types::{Capability, LLMInstance},
};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, debug, warn};

/// Manages a pool of LLM instances
pub struct LLMPool {
    /// Map of LLM ID to provider instance
    providers: DashMap<String, Arc<Box<dyn LLMProvider>>>,
    /// Capability index for fast lookups
    capability_index: DashMap<Capability, Vec<String>>,
}

impl LLMPool {
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
            capability_index: DashMap::new(),
        }
    }

    /// Register a new LLM provider
    pub fn register(&self, provider: Box<dyn LLMProvider>) -> Result<()> {
        let instance = provider.instance();
        let id = instance.id.clone();
        let capabilities = instance.capabilities.clone();

        info!("📝 Registering LLM: {} ({:?})", id, capabilities);

        // Add to providers map
        self.providers.insert(id.clone(), Arc::new(provider));

        // Update capability index
        for cap in capabilities {
            self.capability_index
                .entry(cap)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        Ok(())
    }

    /// Unregister an LLM provider
    pub fn unregister(&self, llm_id: &str) -> Result<()> {
        info!("🗑️  Unregistering LLM: {}", llm_id);

        if let Some((_, provider)) = self.providers.remove(llm_id) {
            let capabilities = provider.instance().capabilities.clone();

            // Remove from capability index
            for cap in capabilities {
                if let Some(mut ids) = self.capability_index.get_mut(&cap) {
                    ids.retain(|id| id != llm_id);
                }
            }

            Ok(())
        } else {
            Err(HybridLLMError::LLMNotFound(llm_id.to_string()))
        }
    }

    /// Get a provider by ID
    pub fn get(&self, llm_id: &str) -> Option<Arc<Box<dyn LLMProvider>>> {
        self.providers.get(llm_id).map(|r| Arc::clone(&r))
    }

    /// Find providers by capability
    pub fn find_by_capability(&self, capability: &Capability) -> Vec<Arc<Box<dyn LLMProvider>>> {
        if let Some(ids) = self.capability_index.get(capability) {
            ids.iter()
                .filter_map(|id| self.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all loaded providers
    pub fn get_all_loaded(&self) -> Vec<Arc<Box<dyn LLMProvider>>> {
        self.providers
            .iter()
            .filter(|entry| entry.value().instance().is_loaded)
            .map(|entry| Arc::clone(entry.value()))
            .collect()
    }

    /// Get all provider IDs
    pub fn get_all_ids(&self) -> Vec<String> {
        self.providers
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Load a provider
    pub async fn load(&self, llm_id: &str) -> Result<()> {
        info!("⬆️  Loading LLM: {}", llm_id);

        if let Some(provider) = self.providers.get_mut(llm_id) {
            // Note: We can't mutate through Arc, so this is a placeholder
            // In practice, we'd need interior mutability (RwLock) or different design
            debug!("LLM {} load requested", llm_id);
            Ok(())
        } else {
            Err(HybridLLMError::LLMNotFound(llm_id.to_string()))
        }
    }

    /// Unload a provider
    pub async fn unload(&self, llm_id: &str) -> Result<()> {
        info!("⬇️  Unloading LLM: {}", llm_id);

        if let Some(provider) = self.providers.get_mut(llm_id) {
            debug!("LLM {} unload requested", llm_id);
            Ok(())
        } else {
            Err(HybridLLMError::LLMNotFound(llm_id.to_string()))
        }
    }

    /// Health check all providers
    pub async fn health_check_all(&self) -> Vec<(String, bool)> {
        let mut results = Vec::new();

        for entry in self.providers.iter() {
            let id = entry.key().clone();
            let provider = entry.value();

            match provider.health_check().await {
                Ok(healthy) => {
                    results.push((id, healthy));
                }
                Err(e) => {
                    warn!("Health check failed for {}: {}", id, e);
                    results.push((id, false));
                }
            }
        }

        results
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let total = self.providers.len();
        let loaded = self.get_all_loaded().len();

        PoolStats {
            total_providers: total,
            loaded_providers: loaded,
            unloaded_providers: total - loaded,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_providers: usize,
    pub loaded_providers: usize,
    pub unloaded_providers: usize,
}

impl Default for LLMPool {
    fn default() -> Self {
        Self::new()
    }
}
