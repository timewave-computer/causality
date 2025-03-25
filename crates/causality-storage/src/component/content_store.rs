// Content-addressed component implementation
// Original file: src/component/content_addressed_component.rs

// Content-addressed component registry
//
// This module implements a content-addressed component registry system, allowing
// components to be registered, discovered, and managed using content-based identifiers.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::crypto::{ContentAddressed, ContentId, HashOutput, HashError, HashFactory};
// Define ComponentId here rather than importing from types
pub type ComponentId = String;

/// Error type for content-addressed component operations
#[derive(Error, Debug)]
pub enum ComponentError {
    /// Component not found
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
    
    /// Component validation failed
    #[error("Component validation failed: {0}")]
    ValidationFailed(String),
    
    /// Component already exists
    #[error("Component already exists: {0}")]
    ComponentExists(String),
    
    /// Component is incompatible
    #[error("Component is incompatible: {0}")]
    Incompatible(String),
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
    
    /// Internal error
    #[error("Internal component error: {0}")]
    InternalError(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Component type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ComponentType {
    /// Core system component
    Core,
    /// UI component
    UI,
    /// Network component
    Network,
    /// Storage component
    Storage,
    /// Processing component
    Processing,
    /// Integration component
    Integration,
    /// Custom component type
    Custom(String),
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::Core => write!(f, "Core"),
            ComponentType::UI => write!(f, "UI"),
            ComponentType::Network => write!(f, "Network"),
            ComponentType::Storage => write!(f, "Storage"),
            ComponentType::Processing => write!(f, "Processing"),
            ComponentType::Integration => write!(f, "Integration"),
            ComponentType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Component lifecycle state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ComponentState {
    /// Component is uninitialized
    Uninitialized,
    /// Component is initialized
    Initialized,
    /// Component is running
    Running,
    /// Component is paused
    Paused,
    /// Component is stopped
    Stopped,
    /// Component has failed
    Failed(String),
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ComponentMetadata {
    /// Component ID
    pub id: ComponentId,
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: ComponentType,
    /// Component version
    pub version: String,
    /// Component description
    pub description: String,
    /// Component state
    pub state: ComponentState,
    /// Component dependencies
    pub dependencies: Vec<ComponentId>,
    /// Component capabilities
    pub capabilities: Vec<String>,
    /// Component configuration
    pub config: HashMap<String, String>,
}

/// Component interface for content-addressed system
#[async_trait]
pub trait Component: ContentAddressed + Send + Sync {
    /// Get the component's ID
    fn id(&self) -> &ComponentId;
    
    /// Get the component's type
    fn component_type(&self) -> ComponentType;
    
    /// Get the component's metadata
    fn metadata(&self) -> &ComponentMetadata;
    
    /// Initialize the component
    async fn initialize(&self) -> std::result::Result<(), ComponentError>;
    
    /// Start the component
    async fn start(&self) -> std::result::Result<(), ComponentError>;
    
    /// Stop the component
    async fn stop(&self) -> std::result::Result<(), ComponentError>;
    
    /// Check if the component is compatible with another component
    fn is_compatible_with(&self, other: &dyn Component) -> bool;
    
    /// Get capabilities provided by this component
    fn get_capabilities(&self) -> Vec<String>;
    
    /// Check if this component provides a specific capability
    fn has_capability(&self, capability: &str) -> bool {
        self.get_capabilities().contains(&capability.to_string())
    }
}

/// A content-addressed component with implementation
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentAddressedComponent {
    /// Component metadata
    pub metadata: ComponentMetadata,
    /// Component implementation data
    pub implementation: Vec<u8>,
    /// Component hash
    pub content_hash: Option<HashOutput>,
}

impl ContentAddressed for ContentAddressedComponent {
    fn content_hash(&self) -> HashOutput {
        // If we already have a computed hash, return it
        if let Some(hash) = &self.content_hash {
            return hash.clone();
        }
        
        // Otherwise compute the hash
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        // If we have a stored hash, verify against it
        if let Some(stored_hash) = &self.content_hash {
            // Create a version without the stored hash for verification
            let mut verify_component = self.clone();
            verify_component.content_hash = None;
            
            // Compute the hash of this version
            let hash_factory = HashFactory::default();
            let hasher = hash_factory.create_hasher().unwrap();
            let data = verify_component.try_to_vec().unwrap();
            let computed_hash = hasher.hash(&data);
            
            return computed_hash == *stored_hash;
        }
        
        // Otherwise just verify the serialization
        let serialized = self.to_bytes();
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let _ = hasher.hash(&serialized);
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl ContentAddressedComponent {
    /// Create a new content-addressed component
    pub fn new(
        id: ComponentId,
        name: String,
        component_type: ComponentType,
        version: String,
        description: String,
        implementation: Vec<u8>,
        dependencies: Vec<ComponentId>,
        capabilities: Vec<String>,
    ) -> Self {
        let metadata = ComponentMetadata {
            id,
            name,
            component_type,
            version,
            description,
            state: ComponentState::Uninitialized,
            dependencies,
            capabilities,
            config: HashMap::new(),
        };
        
        let mut component = Self {
            metadata,
            implementation,
            content_hash: None,
        };
        
        // Compute and store the hash
        let hash = component.content_hash();
        component.content_hash = Some(hash);
        
        component
    }
    
    /// Set the component state
    pub fn set_state(&mut self, state: ComponentState) {
        self.metadata.state = state;
        
        // Invalidate the stored hash since we modified the component
        self.content_hash = None;
    }
    
    /// Add a configuration value
    pub fn add_config(&mut self, key: &str, value: &str) {
        self.metadata.config.insert(key.to_string(), value.to_string());
        
        // Invalidate the stored hash since we modified the component
        self.content_hash = None;
    }
    
    /// Set component dependencies
    pub fn set_dependencies(&mut self, dependencies: Vec<ComponentId>) {
        self.metadata.dependencies = dependencies;
        
        // Invalidate the stored hash since we modified the component
        self.content_hash = None;
    }
    
    /// Add a capability
    pub fn add_capability(&mut self, capability: &str) {
        if !self.metadata.capabilities.contains(&capability.to_string()) {
            self.metadata.capabilities.push(capability.to_string());
            
            // Invalidate the stored hash since we modified the component
            self.content_hash = None;
        }
    }
    
    /// Get the component implementation data
    pub fn implementation(&self) -> &[u8] {
        &self.implementation
    }
}

/// A registry for content-addressed components
pub struct ContentAddressedComponentRegistry {
    /// Components by ID
    components_by_id: RwLock<HashMap<ComponentId, Arc<ContentAddressedComponent>>>,
    
    /// Components by content ID
    components_by_content_id: RwLock<HashMap<ContentId, Arc<ContentAddressedComponent>>>,
    
    /// Components by type
    components_by_type: RwLock<HashMap<ComponentType, Vec<ComponentId>>>,
    
    /// Components by capability
    components_by_capability: RwLock<HashMap<String, Vec<ComponentId>>>,
}

impl ContentAddressedComponentRegistry {
    /// Create a new empty component registry
    pub fn new() -> Self {
        Self {
            components_by_id: RwLock::new(HashMap::new()),
            components_by_content_id: RwLock::new(HashMap::new()),
            components_by_type: RwLock::new(HashMap::new()),
            components_by_capability: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a component
    pub fn register_component(&self, component: ContentAddressedComponent) -> Result<ContentId, ComponentError> {
        let component_id = component.metadata.id.clone();
        let content_id = ContentId::from(component.content_hash());
        let component = Arc::new(component);
        
        // Check if component with this ID already exists
        {
            let components_by_id = self.components_by_id.read().map_err(|_| {
                ComponentError::InternalError("Failed to acquire read lock on components_by_id".to_string())
            })?;
            
            if components_by_id.contains_key(&component_id) {
                return Err(ComponentError::ComponentExists(format!(
                    "Component with ID '{}' already exists", component_id
                )));
            }
        }
        
        // Register by ID
        {
            let mut components_by_id = self.components_by_id.write().map_err(|_| {
                ComponentError::InternalError("Failed to acquire write lock on components_by_id".to_string())
            })?;
            
            components_by_id.insert(component_id.clone(), component.clone());
        }
        
        // Register by content ID
        {
            let mut components_by_content_id = self.components_by_content_id.write().map_err(|_| {
                ComponentError::InternalError("Failed to acquire write lock on components_by_content_id".to_string())
            })?;
            
            components_by_content_id.insert(content_id.clone(), component.clone());
        }
        
        // Register by type
        {
            let mut components_by_type = self.components_by_type.write().map_err(|_| {
                ComponentError::InternalError("Failed to acquire write lock on components_by_type".to_string())
            })?;
            
            let component_type = component.metadata.component_type.clone();
            let components = components_by_type.entry(component_type).or_insert_with(Vec::new);
            components.push(component_id.clone());
        }
        
        // Register by capability
        {
            let mut components_by_capability = self.components_by_capability.write().map_err(|_| {
                ComponentError::InternalError("Failed to acquire write lock on components_by_capability".to_string())
            })?;
            
            for capability in &component.metadata.capabilities {
                let components = components_by_capability.entry(capability.clone()).or_insert_with(Vec::new);
                components.push(component_id.clone());
            }
        }
        
        Ok(content_id)
    }
    
    /// Get a component by ID
    pub fn get_component_by_id(&self, id: &ComponentId) -> Result<Arc<ContentAddressedComponent>, ComponentError> {
        let components_by_id = self.components_by_id.read().map_err(|_| {
            ComponentError::InternalError("Failed to acquire read lock on components_by_id".to_string())
        })?;
        
        components_by_id.get(id).cloned().ok_or_else(|| {
            ComponentError::ComponentNotFound(format!("Component with ID '{}' not found", id))
        })
    }
    
    /// Get a component by content ID
    pub fn get_component_by_content_id(&self, content_id: &ContentId) -> Result<Arc<ContentAddressedComponent>, ComponentError> {
        let components_by_content_id = self.components_by_content_id.read().map_err(|_| {
            ComponentError::InternalError("Failed to acquire read lock on components_by_content_id".to_string())
        })?;
        
        components_by_content_id.get(content_id).cloned().ok_or_else(|| {
            ComponentError::ComponentNotFound(format!("Component with content ID '{:?}' not found", content_id))
        })
    }
    
    /// Get components by type
    pub fn get_components_by_type(&self, component_type: &ComponentType) -> Result<Vec<Arc<ContentAddressedComponent>>, ComponentError> {
        let components_by_type = self.components_by_type.read().map_err(|_| {
            ComponentError::InternalError("Failed to acquire read lock on components_by_type".to_string())
        })?;
        
        let components_by_id = self.components_by_id.read().map_err(|_| {
            ComponentError::InternalError("Failed to acquire read lock on components_by_id".to_string())
        })?;
        
        let component_ids = components_by_type.get(component_type).ok_or_else(|| {
            ComponentError::ComponentNotFound(format!("No components of type '{:?}' found", component_type))
        })?;
        
        let mut components = Vec::new();
        for id in component_ids {
            if let Some(component) = components_by_id.get(id) {
                components.push(component.clone());
            }
        }
        
        Ok(components)
    }
    
    /// Get components by capability
    pub fn get_components_by_capability(&self, capability: &str) -> Result<Vec<Arc<ContentAddressedComponent>>, ComponentError> {
        let components_by_capability = self.components_by_capability.read().map_err(|_| {
            ComponentError::InternalError("Failed to acquire read lock on components_by_capability".to_string())
        })?;
        
        let components_by_id = self.components_by_id.read().map_err(|_| {
            ComponentError::InternalError("Failed to acquire read lock on components_by_id".to_string())
        })?;
        
        let component_ids = components_by_capability.get(capability).ok_or_else(|| {
            ComponentError::ComponentNotFound(format!("No components with capability '{}' found", capability))
        })?;
        
        let mut components = Vec::new();
        for id in component_ids {
            if let Some(component) = components_by_id.get(id) {
                components.push(component.clone());
            }
        }
        
        Ok(components)
    }
    
    /// Get all components
    pub fn get_all_components(&self) -> Result<Vec<Arc<ContentAddressedComponent>>, ComponentError> {
        let components_by_id = self.components_by_id.read().map_err(|_| {
            ComponentError::InternalError("Failed to acquire read lock on components_by_id".to_string())
        })?;
        
        Ok(components_by_id.values().cloned().collect())
    }
    
    /// Remove a component by ID
    pub fn remove_component(&self, id: &ComponentId) -> Result<(), ComponentError> {
        // Get the component first to get its content ID and other metadata
        let component = self.get_component_by_id(id)?;
        let content_id = ContentId::from(component.content_hash());
        let component_type = component.metadata.component_type.clone();
        
        // Remove from components_by_id
        {
            let mut components_by_id = self.components_by_id.write().map_err(|_| {
                ComponentError::InternalError("Failed to acquire write lock on components_by_id".to_string())
            })?;
            
            components_by_id.remove(id);
        }
        
        // Remove from components_by_content_id
        {
            let mut components_by_content_id = self.components_by_content_id.write().map_err(|_| {
                ComponentError::InternalError("Failed to acquire write lock on components_by_content_id".to_string())
            })?;
            
            components_by_content_id.remove(&content_id);
        }
        
        // Remove from components_by_type
        {
            let mut components_by_type = self.components_by_type.write().map_err(|_| {
                ComponentError::InternalError("Failed to acquire write lock on components_by_type".to_string())
            })?;
            
            if let Some(components) = components_by_type.get_mut(&component_type) {
                components.retain(|component_id| component_id != id);
            }
        }
        
        // Remove from components_by_capability
        {
            let mut components_by_capability = self.components_by_capability.write().map_err(|_| {
                ComponentError::InternalError("Failed to acquire write lock on components_by_capability".to_string())
            })?;
            
            for capability in &component.metadata.capabilities {
                if let Some(components) = components_by_capability.get_mut(capability) {
                    components.retain(|component_id| component_id != id);
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if a component with the given ID exists
    pub fn contains_component(&self, id: &ComponentId) -> Result<bool, ComponentError> {
        let components_by_id = self.components_by_id.read().map_err(|_| {
            ComponentError::InternalError("Failed to acquire read lock on components_by_id".to_string())
        })?;
        
        Ok(components_by_id.contains_key(id))
    }
    
    /// Count the number of registered components
    pub fn count_components(&self) -> Result<usize, ComponentError> {
        let components_by_id = self.components_by_id.read().map_err(|_| {
            ComponentError::InternalError("Failed to acquire read lock on components_by_id".to_string())
        })?;
        
        Ok(components_by_id.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_registry() {
        // Create registry
        let registry = ContentAddressedComponentRegistry::new();
        
        // Create component
        let component1 = ContentAddressedComponent::new(
            "component1".to_string(),
            "Test Component 1".to_string(),
            ComponentType::Core,
            "1.0.0".to_string(),
            "Test component for registry".to_string(),
            vec![1, 2, 3, 4],
            vec![],
            vec!["test".to_string(), "example".to_string()],
        );
        
        // Create another component
        let component2 = ContentAddressedComponent::new(
            "component2".to_string(),
            "Test Component 2".to_string(),
            ComponentType::Storage,
            "1.0.0".to_string(),
            "Another test component".to_string(),
            vec![5, 6, 7, 8],
            vec![],
            vec!["storage".to_string()],
        );
        
        // Register components
        let content_id1 = registry.register_component(component1).unwrap();
        let content_id2 = registry.register_component(component2).unwrap();
        
        // Check counts
        assert_eq!(registry.count_components().unwrap(), 2);
        
        // Get by ID
        let retrieved1 = registry.get_component_by_id(&"component1".to_string()).unwrap();
        assert_eq!(retrieved1.metadata.name, "Test Component 1");
        
        // Get by content ID
        let retrieved_by_content = registry.get_component_by_content_id(&content_id1).unwrap();
        assert_eq!(retrieved_by_content.metadata.id, "component1");
        
        // Get by type
        let core_components = registry.get_components_by_type(&ComponentType::Core).unwrap();
        assert_eq!(core_components.len(), 1);
        assert_eq!(core_components[0].metadata.id, "component1");
        
        let storage_components = registry.get_components_by_type(&ComponentType::Storage).unwrap();
        assert_eq!(storage_components.len(), 1);
        assert_eq!(storage_components[0].metadata.id, "component2");
        
        // Get by capability
        let test_capability_components = registry.get_components_by_capability("test").unwrap();
        assert_eq!(test_capability_components.len(), 1);
        assert_eq!(test_capability_components[0].metadata.id, "component1");
        
        let storage_capability_components = registry.get_components_by_capability("storage").unwrap();
        assert_eq!(storage_capability_components.len(), 1);
        assert_eq!(storage_capability_components[0].metadata.id, "component2");
        
        // Get all components
        let all_components = registry.get_all_components().unwrap();
        assert_eq!(all_components.len(), 2);
        
        // Check contains
        assert!(registry.contains_component(&"component1".to_string()).unwrap());
        assert!(!registry.contains_component(&"nonexistent".to_string()).unwrap());
        
        // Remove component
        registry.remove_component(&"component1".to_string()).unwrap();
        
        // Check counts after removal
        assert_eq!(registry.count_components().unwrap(), 1);
        
        // Verify component is removed
        assert!(!registry.contains_component(&"component1".to_string()).unwrap());
        assert!(registry.contains_component(&"component2".to_string()).unwrap());
    }
    
    #[test]
    fn test_content_addressing() {
        // Create component
        let mut component = ContentAddressedComponent::new(
            "test_component".to_string(),
            "Test Component".to_string(),
            ComponentType::Core,
            "1.0.0".to_string(),
            "Component for testing content addressing".to_string(),
            vec![1, 2, 3, 4],
            vec![],
            vec!["test".to_string()],
        );
        
        // Get the content hash
        let hash1 = component.content_hash();
        
        // Verify the component
        assert!(component.verify());
        
        // Modify the component
        component.add_capability("modified");
        
        // Get the hash again
        let hash2 = component.content_hash();
        
        // Hashes should be different
        assert_ne!(hash1, hash2);
        
        // Verify the component again
        assert!(component.verify());
        
        // Test serialization and deserialization
        let serialized = component.to_bytes();
        let deserialized = ContentAddressedComponent::from_bytes(&serialized).unwrap();
        
        // Content hashes should match
        assert_eq!(deserialized.content_hash(), component.content_hash());
    }
} 