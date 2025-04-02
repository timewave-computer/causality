use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use causality_core::effect::{Effect, EffectContext, EffectOutcome, EffectId};
use causality_core::resource::types::ResourceId;
use causality_engine::effect::factory::EmptyEffect;

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
            effect_id: "minimal-effect-id".to_string(),
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

#[tokio::test]
async fn test_minimal_empty_effect() {
    let effect = EmptyEffect::new("test_effect");
    
    assert_eq!(
        effect.effect_type().to_string(),
        "Custom(test_effect)"
    );
    
    // Create a minimal context for testing
    let context = MinimalEffectContext::new();
    
    // Test execution
    let outcome = effect.execute(&context).await.unwrap();
    assert!(matches!(outcome, EffectOutcome::Success));
} 