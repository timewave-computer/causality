// Tests for the effect context implementations
#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use crate::effect::context::{Capability, EffectContext};
    use crate::effect::types::{EffectId, Right};
    use crate::resource::types::ResourceId;

    // Simple context implementation for testing
    #[derive(Debug, Clone)]
    struct SimpleEffectContext {
        capabilities: Vec<Capability>,
        resources: HashSet<ResourceId>,
        metadata: HashMap<String, String>,
        effect_id: EffectId,
        parent: Option<Arc<dyn EffectContext>>,
    }

    impl SimpleEffectContext {
        fn new() -> Self {
            Self {
                capabilities: Vec::new(),
                resources: HashSet::new(),
                metadata: HashMap::new(),
                effect_id: EffectId::from("default-id".to_string()),
                parent: None,
            }
        }
        
        fn with_capability(mut self, capability: Capability) -> Self {
            self.capabilities.push(capability);
            self
        }
        
        fn with_effect_id(mut self, effect_id: EffectId) -> Self {
            self.effect_id = effect_id;
            self
        }
        
        fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.metadata.insert(key.into(), value.into());
            self
        }
    }

    impl EffectContext for SimpleEffectContext {
        fn effect_id(&self) -> &EffectId {
            &self.effect_id
        }
        
        fn capabilities(&self) -> &[Capability] {
            &self.capabilities
        }
        
        fn resources(&self) -> &HashSet<ResourceId> {
            &self.resources
        }
        
        fn parent_context(&self) -> Option<&Arc<dyn EffectContext>> {
            self.parent.as_ref()
        }
        
        fn has_capability(&self, capability: &Capability) -> bool {
            self.capabilities.iter().any(|cap| cap.resource_id == capability.resource_id && cap.right == capability.right)
        }
        
        fn metadata(&self) -> &HashMap<String, String> {
            &self.metadata
        }
        
        fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
            Box::new(Self {
                effect_id,
                capabilities: self.capabilities.clone(),
                resources: self.resources.clone(),
                metadata: self.metadata.clone(),
                parent: Some(Arc::new(self.clone())),
            })
        }
        
        fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext> {
            let mut new_caps = self.capabilities.clone();
            new_caps.extend(capabilities);
            Box::new(Self {
                effect_id: self.effect_id.clone(),
                capabilities: new_caps,
                resources: self.resources.clone(),
                metadata: self.metadata.clone(),
                parent: self.parent.clone(),
            })
        }
        
        fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
            let mut new_resources = self.resources.clone();
            new_resources.extend(resources);
            Box::new(Self {
                effect_id: self.effect_id.clone(),
                capabilities: self.capabilities.clone(),
                resources: new_resources,
                metadata: self.metadata.clone(),
                parent: self.parent.clone(),
            })
        }
        
        fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
            let mut new_metadata = self.metadata.clone();
            new_metadata.extend(metadata);
            Box::new(Self {
                effect_id: self.effect_id.clone(),
                capabilities: self.capabilities.clone(),
                resources: self.resources.clone(),
                metadata: new_metadata,
                parent: self.parent.clone(),
            })
        }
        
        fn clone_context(&self) -> Box<dyn EffectContext> {
            Box::new(self.clone())
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_simple_effect_context() {
        // Create a basic context
        let context = SimpleEffectContext::new();
        
        // Verify the default values
        assert_eq!(context.effect_id().to_string(), "default-id");
        assert!(context.capabilities().is_empty());
        assert!(context.resources().is_empty());
        assert!(context.metadata().is_empty());
        assert!(context.parent_context().is_none());
        
        // Create a capability
        let resource_id = ResourceId::from_string("test-resource").unwrap_or_else(|_| ResourceId::new_random());
        let capability = Capability {
            resource_id: resource_id.clone(),
            right: Right::Read,
        };
        
        // Create a context with the capability
        let context_with_cap = context.clone().with_capability(capability.clone());
        
        // Verify the context has the capability
        assert!(!context_with_cap.capabilities().is_empty());
        assert!(context_with_cap.has_capability(&capability));
        
        // Test deriving a context
        let derived_context = context_with_cap.derive_context(EffectId::from("derived-id".to_string()));
        
        // Verify the derived context
        assert_eq!(derived_context.effect_id().to_string(), "derived-id");
        assert!(derived_context.has_capability(&capability));
        assert!(derived_context.parent_context().is_some());
        
        // Test adding additional capabilities
        let new_capability = Capability {
            resource_id: ResourceId::from_string("second-resource").unwrap_or_else(|_| ResourceId::new_random()),
            right: Right::Write,
        };
        
        let context_with_additional_caps = context_with_cap.with_additional_capabilities(vec![new_capability.clone()]);
        
        // Verify the context has both capabilities
        assert_eq!(context_with_additional_caps.capabilities().len(), 2);
        assert!(context_with_additional_caps.has_capability(&capability));
        assert!(context_with_additional_caps.has_capability(&new_capability));
        
        // Test adding metadata
        let mut test_metadata = HashMap::new();
        test_metadata.insert("test-key".to_string(), "test-value".to_string());
        
        let context_with_metadata = context.with_additional_metadata(test_metadata);
        
        // Verify the metadata
        assert_eq!(context_with_metadata.metadata().get("test-key").unwrap(), "test-value");
    }
} 