use common::{
    errors::Result,
    types::Capability,
};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Load balancer for distributing requests across LLMs
pub struct LoadBalancer {
    /// Round-robin counter
    counter: AtomicUsize,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }

    /// Select the next LLM from a list using round-robin
    pub fn select_round_robin<'a>(&self, llm_ids: &'a [String]) -> Option<&'a String> {
        if llm_ids.is_empty() {
            return None;
        }

        let count = self.counter.fetch_add(1, Ordering::Relaxed);
        let index = count % llm_ids.len();
        Some(&llm_ids[index])
    }

    /// Select the best LLM based on current load (placeholder)
    /// In a real implementation, this would consider actual resource usage
    pub fn select_least_loaded<'a>(&self, llm_ids: &'a [String]) -> Option<&'a String> {
        // For now, just use round-robin
        // TODO: Implement actual load tracking
        self.select_round_robin(llm_ids)
    }

    /// Select LLM with preference for local models
    pub fn select_prefer_local<'a>(
        &self,
        llm_ids: &'a [String],
        is_local: impl Fn(&str) -> bool,
    ) -> Option<&'a String> {
        // Separate local and cloud LLMs
        let local: Vec<&String> = llm_ids.iter().filter(|id| is_local(id)).collect();
        let cloud: Vec<&String> = llm_ids.iter().filter(|id| !is_local(id)).collect();

        // Prefer local if available
        if !local.is_empty() {
            let count = self.counter.fetch_add(1, Ordering::Relaxed);
            let index = count % local.len();
            Some(local[index])
        } else if !cloud.is_empty() {
            let count = self.counter.fetch_add(1, Ordering::Relaxed);
            let index = count % cloud.len();
            Some(cloud[index])
        } else {
            None
        }
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_robin() {
        let balancer = LoadBalancer::new();
        let llms = vec!["llm1".to_string(), "llm2".to_string(), "llm3".to_string()];

        assert_eq!(balancer.select_round_robin(&llms), Some(&"llm1".to_string()));
        assert_eq!(balancer.select_round_robin(&llms), Some(&"llm2".to_string()));
        assert_eq!(balancer.select_round_robin(&llms), Some(&"llm3".to_string()));
        assert_eq!(balancer.select_round_robin(&llms), Some(&"llm1".to_string()));
    }

    #[test]
    fn test_prefer_local() {
        let balancer = LoadBalancer::new();
        let llms = vec![
            "cloud1".to_string(),
            "local1".to_string(),
            "local2".to_string(),
        ];

        let is_local = |id: &str| id.starts_with("local");

        // Should always select local
        let selected = balancer.select_prefer_local(&llms, is_local).unwrap();
        assert!(selected.starts_with("local"));
    }
}
