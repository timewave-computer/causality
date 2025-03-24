// Component module for Causality
//
// This module provides content-addressed component registry implementations for Causality,
// enabling components to be registered, discovered, and managed using content-based identifiers.

pub mod content_addressed_component;

// Re-export component types
pub use content_addressed_component::{
    Component,
    ComponentType,
    ComponentState,
    ComponentMetadata,
    ContentAddressedComponent,
    ContentAddressedComponentRegistry,
    ComponentError,
};

// Factory for component registry
use std::sync::Arc;

/// Component registry factory
pub struct ComponentRegistryFactory {
    /// Default component registry
    registry: Arc<ContentAddressedComponentRegistry>,
}

impl ComponentRegistryFactory {
    /// Create a new component registry factory
    pub fn new() -> Self {
        Self {
            registry: Arc::new(ContentAddressedComponentRegistry::new()),
        }
    }
    
    /// Get the default component registry
    pub fn default_registry(&self) -> Arc<ContentAddressedComponentRegistry> {
        self.registry.clone()
    }
    
    /// Create a new component registry
    pub fn create_registry(&self) -> Arc<ContentAddressedComponentRegistry> {
        Arc::new(ContentAddressedComponentRegistry::new())
    }
}

/// Default implementation for the component registry factory
impl Default for ComponentRegistryFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content_addressed_component::{
        ContentAddressedComponent,
        ComponentType,
    };
    
    #[test]
    fn test_component_registry_factory() {
        // Create factory
        let factory = ComponentRegistryFactory::new();
        
        // Get default registry
        let registry = factory.default_registry();
        
        // Create component
        let component = ContentAddressedComponent::new(
            "test_component".to_string(),
            "Test Component".to_string(),
            ComponentType::Core,
            "1.0.0".to_string(),
            "Test component for registry factory".to_string(),
            vec![1, 2, 3, 4],
            vec![],
            vec!["test".to_string()],
        );
        
        // Register component
        let result = registry.register_component(component);
        assert!(result.is_ok());
        
        // Check component count
        assert_eq!(registry.count_components().unwrap(), 1);
        
        // Create another registry
        let another_registry = factory.create_registry();
        
        // This one should be empty
        assert_eq!(another_registry.count_components().unwrap(), 0);
    }
} 