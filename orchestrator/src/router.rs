use common::{
    messages::{OrchestratorMessage, TaskDescription},
    types::{Capability, TaskType, LLMInstance},
    errors::{Result, HybridLLMError},
};
use std::collections::HashMap;
use tracing::{debug, info};

/// Routes requests to appropriate LLMs based on capabilities
pub struct Router {
    /// Registry of available LLMs and their capabilities
    llm_registry: HashMap<String, LLMInstance>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            llm_registry: HashMap::new(),
        }
    }

    /// Register an LLM instance
    pub fn register_llm(&mut self, instance: LLMInstance) {
        info!("📝 Registering LLM: {} with capabilities: {:?}",
              instance.id, instance.capabilities);
        self.llm_registry.insert(instance.id.clone(), instance);
    }

    /// Unregister an LLM instance
    pub fn unregister_llm(&mut self, llm_id: &str) {
        info!("🗑️  Unregistering LLM: {}", llm_id);
        self.llm_registry.remove(llm_id);
    }

    /// Route a task to the best available LLM
    pub fn route_task(&self, task: &TaskDescription) -> Result<String> {
        debug!("🎯 Routing task: {:?}", task.task_type);

        // Find LLMs that have the required capabilities
        let mut candidates: Vec<&LLMInstance> = self.llm_registry
            .values()
            .filter(|instance| {
                instance.is_loaded &&
                task.required_capabilities
                    .iter()
                    .all(|cap| instance.capabilities.contains(cap))
            })
            .collect();

        if candidates.is_empty() {
            return Err(HybridLLMError::LLMNotFound(
                format!("No LLM available for capabilities: {:?}", task.required_capabilities)
            ));
        }

        // Sort by preference (for now, prefer local models)
        candidates.sort_by(|a, b| {
            // Prefer loaded models
            match (a.is_loaded, b.is_loaded) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    // Then prefer models with more specific capabilities
                    b.capabilities.len().cmp(&a.capabilities.len())
                }
            }
        });

        Ok(candidates[0].id.clone())
    }

    /// Get all registered LLMs
    pub fn get_all_llms(&self) -> Vec<&LLMInstance> {
        self.llm_registry.values().collect()
    }

    /// Get a specific LLM by ID
    pub fn get_llm(&self, llm_id: &str) -> Option<&LLMInstance> {
        self.llm_registry.get(llm_id)
    }

    /// Find LLMs by capability
    pub fn find_by_capability(&self, capability: &Capability) -> Vec<&LLMInstance> {
        self.llm_registry
            .values()
            .filter(|instance| instance.capabilities.contains(capability))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::types::LLMProvider;

    #[test]
    fn test_router() {
        let mut router = Router::new();

        let llm = LLMInstance {
            id: "test-llm".to_string(),
            provider: LLMProvider::Local("test".to_string()),
            capabilities: vec![Capability::Code],
            model_name: "test-model".to_string(),
            max_context: 4096,
            is_loaded: true,
        };

        router.register_llm(llm);

        let task = TaskDescription {
            description: "Test task".to_string(),
            task_type: TaskType::Code,
            required_capabilities: vec![Capability::Code],
            context: HashMap::new(),
            constraints: vec![],
        };

        let result = router.route_task(&task).unwrap();
        assert_eq!(result, "test-llm");
    }
}
