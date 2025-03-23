use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use async_trait::async_trait;

use crate::effect::{Effect, EffectContext, EffectOutcome, EffectResult};

/// A simple effect that does nothing, used as a placeholder
pub struct EmptyEffect {
    id: String,
    description: String,
}

impl EmptyEffect {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description: "Empty effect that performs no action".to_string(),
        }
    }
    
    pub fn with_description(description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description,
        }
    }
}

#[async_trait]
impl Effect for EmptyEffect {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn display_name(&self) -> String {
        "Empty Effect".to_string()
    }
    
    fn description(&self) -> String {
        self.description.clone()
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        Ok(EffectOutcome {
            id: self.id.clone(),
            success: true,
            data: HashMap::new(),
            error: None,
            execution_id: context.execution_id.clone(),
            resource_changes: Vec::new(),
            metadata: HashMap::new(),
        })
    }
} 