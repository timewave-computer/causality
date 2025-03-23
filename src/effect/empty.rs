use std::fmt;
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::effect::{Effect, EffectOutcome, EffectId, ExecutionBoundary, EffectContext};

/// An empty effect that does nothing
#[derive(Debug, Clone)]
pub struct EmptyEffect {
    id: EffectId,
    name: String,
}

impl EmptyEffect {
    /// Create a new empty effect with a default name
    pub fn new() -> Self {
        Self {
            id: EffectId::new_unique(),
            name: "empty".to_string(),
        }
    }

    /// Create a new empty effect with a specific name
    pub fn with_name(name: &str) -> Self {
        Self {
            id: EffectId::new_unique(),
            name: name.to_string(),
        }
    }
}

impl fmt::Display for EmptyEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EmptyEffect({})", self.name)
    }
}

impl Effect for EmptyEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
    
    fn display_name(&self) -> String {
        format!("Empty Effect: {}", self.name)
    }
    
    fn description(&self) -> String {
        "An empty effect that performs no operation".to_string()
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn can_execute_in(&self, _boundary: ExecutionBoundary) -> bool {
        true  // Can execute in any boundary
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Local  // Prefer local execution
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("name".to_string(), self.name.clone());
        params.insert("type".to_string(), "empty".to_string());
        params
    }
    
    async fn execute(&self, _context: &crate::effect::EffectContext) -> Result<EffectOutcome> {
        Ok(EffectOutcome::success(self.id.clone())
            .with_data("status", "completed")
            .with_data("effect_type", "empty"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_empty_effect() {
        let effect = EmptyEffect::new();
        let context = EffectContext::new();
        let outcome = effect.execute(&context).await.unwrap();
        
        assert!(outcome.success);
        assert_eq!(outcome.get_data("status"), Some(&"completed".to_string()));
        assert_eq!(outcome.get_data("effect_type"), Some(&"empty".to_string()));
    }
} 