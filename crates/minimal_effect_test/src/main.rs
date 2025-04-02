use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use causality_core::effect::{Effect, EffectContext, EffectOutcome, EffectId, EffectType, EffectError};
use causality_core::resource::types::ResourceId;
use async_trait::async_trait;

// Define a simple effect for testing
#[derive(Debug, Clone)]
struct SimpleEmptyEffect {
    name: String,
}

impl SimpleEmptyEffect {
    pub fn new(name: &str) -> Self {
        SimpleEmptyEffect {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl Effect for SimpleEmptyEffect {
    fn effect_type(&self) -> EffectType {
        EffectType::Custom(self.name.clone())
    }
    
    fn description(&self) -> String {
        format!("Simple empty effect: {}", self.name)
    }
    
    async fn execute(&self, _context: &dyn EffectContext) -> Result<EffectOutcome, EffectError> {
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Define a minimal context for testing
#[derive(Debug)]
struct MinimalEffectContext {
    effect_id: EffectId,
    capabilities: Vec<causality_core::effect::Capability>,
    metadata: HashMap<String, String>,
    resources: HashSet<ResourceId>,
}

impl MinimalEffectContext {
    fn new() -> Self {
        Self {
            effect_id: EffectId::from_string("minimal-effect-id"),
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            resources: HashSet::new(),
        }
    }
}

impl EffectContext for MinimalEffectContext {
    fn effect_id(&self) -> &EffectId {
        &self.effect_id
    }
    
    fn capabilities(&self) -> &[causality_core::effect::Capability] {
        &self.capabilities
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    fn resources(&self) -> &HashSet<ResourceId> {
        &self.resources
    }
    
    fn parent_context(&self) -> Option<&Arc<dyn EffectContext>> {
        None
    }
    
    fn has_capability(&self, _capability: &causality_core::effect::Capability) -> bool {
        false
    }
    
    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
        Box::new(MinimalEffectContext {
            effect_id,
            capabilities: self.capabilities.clone(),
            metadata: self.metadata.clone(),
            resources: self.resources.clone(),
        })
    }
    
    fn with_additional_capabilities(&self, capabilities: Vec<causality_core::effect::Capability>) -> Box<dyn EffectContext> {
        let mut new_capabilities = self.capabilities.clone();
        new_capabilities.extend(capabilities);
        Box::new(MinimalEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: new_capabilities,
            metadata: self.metadata.clone(),
            resources: self.resources.clone(),
        })
    }
    
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
        let mut new_resources = self.resources.clone();
        new_resources.extend(resources);
        Box::new(MinimalEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            metadata: self.metadata.clone(),
            resources: new_resources,
        })
    }
    
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
        let mut new_metadata = self.metadata.clone();
        new_metadata.extend(metadata);
        Box::new(MinimalEffectContext {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            metadata: new_metadata,
            resources: self.resources.clone(),
        })
    }
    
    fn clone_context(&self) -> Box<dyn EffectContext> {
        Box::new(Self {
            effect_id: self.effect_id.clone(),
            capabilities: self.capabilities.clone(),
            metadata: self.metadata.clone(),
            resources: self.resources.clone(),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::main]
async fn main() {
    // Create a simple effect
    let effect = SimpleEmptyEffect::new("test_effect");
    
    println!("Effect type: {}", effect.effect_type());
    println!("Effect description: {}", effect.description());
    
    // Create a minimal context for testing
    let context = MinimalEffectContext::new();
    
    // Execute the effect
    match effect.execute(&context).await {
        Ok(outcome) => {
            println!("Effect execution successful: {:?}", outcome);
            println!("Is success: {}", outcome.is_success());
            println!("Test passed!");
        },
        Err(e) => {
            println!("Effect execution failed: {}", e);
            std::process::exit(1);
        }
    }
} 