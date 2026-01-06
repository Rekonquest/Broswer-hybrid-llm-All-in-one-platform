use common::{
    errors::{Result, HybridLLMError},
    traits::{SecurityAnalysis, RiskLevel},
};
use regex::Regex;
use tracing::{debug, warn};

/// Guardrail system for analyzing commands and actions
pub struct Guardrails {
    rules: Vec<GuardrailRule>,
}

pub struct GuardrailRule {
    pub name: String,
    pub pattern: Regex,
    pub risk_level: RiskLevel,
    pub description: String,
}

impl Guardrails {
    pub fn new() -> Self {
        let rules = Self::default_rules();
        Self { rules }
    }

    /// Analyze a command for security risks
    pub fn analyze_command(&self, command: &str) -> Result<SecurityAnalysis> {
        debug!("🔍 Analyzing command: {}", command);

        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut max_risk = RiskLevel::Low;

        for rule in &self.rules {
            if rule.pattern.is_match(command) {
                warn!("⚠️  Matched guardrail rule: {}", rule.name);
                issues.push(format!("{}: {}", rule.name, rule.description));

                // Update max risk level
                if (rule.risk_level as u8) > (max_risk as u8) {
                    max_risk = rule.risk_level;
                }

                // Add suggestions based on the rule
                match rule.name.as_str() {
                    "dangerous_rm" => {
                        suggestions.push("Use specific paths instead of wildcards".to_string());
                        suggestions.push("Consider using 'trash' or 'safe-rm' instead".to_string());
                    }
                    "sudo_usage" => {
                        suggestions.push("Explain why elevated privileges are needed".to_string());
                    }
                    "disk_operations" => {
                        suggestions.push("Use file-level operations instead".to_string());
                    }
                    _ => {}
                }
            }
        }

        let safe = max_risk as u8 <= RiskLevel::Medium as u8;

        Ok(SecurityAnalysis {
            safe,
            risk_level: max_risk,
            issues,
            suggestions,
        })
    }

    /// Add a custom guardrail rule
    pub fn add_rule(&mut self, rule: GuardrailRule) {
        self.rules.push(rule);
    }

    /// Default security rules
    fn default_rules() -> Vec<GuardrailRule> {
        vec![
            GuardrailRule {
                name: "dangerous_rm".to_string(),
                pattern: Regex::new(r"rm\s+(-rf?|--recursive|--force).*(/|\*|\$HOME)").unwrap(),
                risk_level: RiskLevel::Critical,
                description: "Dangerous recursive file deletion detected".to_string(),
            },
            GuardrailRule {
                name: "sudo_usage".to_string(),
                pattern: Regex::new(r"\bsudo\b").unwrap(),
                risk_level: RiskLevel::High,
                description: "Elevated privileges requested".to_string(),
            },
            GuardrailRule {
                name: "disk_operations".to_string(),
                pattern: Regex::new(r"\b(dd|mkfs|fdisk)\b").unwrap(),
                risk_level: RiskLevel::Critical,
                description: "Low-level disk operations detected".to_string(),
            },
            GuardrailRule {
                name: "network_exposure".to_string(),
                pattern: Regex::new(r"\b(nc|netcat|ncat)\b.*-l").unwrap(),
                risk_level: RiskLevel::Medium,
                description: "Network port listening detected".to_string(),
            },
            GuardrailRule {
                name: "system_modification".to_string(),
                pattern: Regex::new(r"\b(chmod\s+777|chown\s+root)").unwrap(),
                risk_level: RiskLevel::High,
                description: "Dangerous permission changes detected".to_string(),
            },
            GuardrailRule {
                name: "data_exfiltration".to_string(),
                pattern: Regex::new(r"\b(curl|wget|scp|rsync)\b.*\|.*\b(nc|netcat|bash)\b").unwrap(),
                risk_level: RiskLevel::Critical,
                description: "Potential data exfiltration pattern detected".to_string(),
            },
            GuardrailRule {
                name: "shell_injection".to_string(),
                pattern: Regex::new(r"[;&|`$]\s*\(").unwrap(),
                risk_level: RiskLevel::High,
                description: "Potential shell injection detected".to_string(),
            },
            GuardrailRule {
                name: "password_exposure".to_string(),
                pattern: Regex::new(r#"(password|passwd|secret|api[_-]?key)\s*=\s*['"]?\w+"#).unwrap(),
                risk_level: RiskLevel::High,
                description: "Hardcoded credentials detected".to_string(),
            },
        ]
    }
}

impl Default for Guardrails {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_rm() {
        let guardrails = Guardrails::new();
        let result = guardrails.analyze_command("rm -rf /").unwrap();

        assert!(!result.safe);
        assert_eq!(result.risk_level, RiskLevel::Critical);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_safe_command() {
        let guardrails = Guardrails::new();
        let result = guardrails.analyze_command("ls -la").unwrap();

        assert!(result.safe);
        assert_eq!(result.risk_level, RiskLevel::Low);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_sudo() {
        let guardrails = Guardrails::new();
        let result = guardrails.analyze_command("sudo apt update").unwrap();

        assert!(!result.safe);
        assert_eq!(result.risk_level, RiskLevel::High);
    }
}
