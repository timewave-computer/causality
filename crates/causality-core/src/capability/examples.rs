// Examples of using the capability system
//
// This file provides examples of how to use the capability-based resource
// safety system with content addressing.

use super::{
    Capability, CapabilityGrants, ResourceRegistry,
    ContentRef, CapabilityError
};
use std::marker::PhantomData;
use serde;

// Example 1: Basic capability usage
pub fn basic_capability_example() -> Result<(), CapabilityError> {
    // Create a registry and an identity
    let mut registry = ResourceRegistry::new();
    let alice = "alice".to_string();
    
    // Register a resource (a simple String in this case)
    let data = "Hello, world!".to_string();
    let capability = registry.register(data, alice.clone())?;
    
    // Access the resource with the capability
    let guard = registry.access(&capability)?;
    
    // Read the resource
    let data = guard.read()?;
    assert_eq!(*data, "Hello, world!".to_string());
    
    // Create a restricted capability (read-only)
    let read_only: Capability<String> = Capability {
        id: capability.id.clone(),
        grants: CapabilityGrants::read_only(),
        origin: Some(alice.clone()),
        _phantom: PhantomData,
    };
    
    // Access with the restricted capability
    let read_guard = registry.access(&read_only)?;
    
    // Reading works
    let data = read_guard.read()?;
    assert_eq!(*data, "Hello, world!".to_string());
    
    // Writing should fail
    // We need to create a mutable guard to try writing
    let mut read_guard = registry.access(&read_only)?;
    assert!(read_guard.write().is_err());
    
    Ok(())
}

// Example 2: Content-addressed capabilities
pub fn content_addressed_example() -> Result<(), CapabilityError> {
    // Create a registry with content addressing
    let mut registry = ResourceRegistry::new();
    let alice = "alice".to_string();
    
    // Register a resource
    let data = vec![1, 2, 3, 4, 5];
    let capability = registry.register(data, alice.clone())?;
    
    // Access the resource
    let guard = registry.access(&capability)?;
    
    // Convert capability to content-addressed capability
    let content_capability: Capability<Vec<i32>> = guard.to_content_addressed()?;
    
    // Create a content reference from the content hash
    let content_ref: ContentRef<Vec<i32>> = ContentRef {
        hash: content_capability.id.hash.clone(),
        _phantom: PhantomData,
    };
    
    // Access by content reference
    let content_guard = registry.access_by_content(&content_ref)?;
    
    // Verify the content
    let data = content_guard.read()?;
    assert_eq!(*data, vec![1, 2, 3, 4, 5]);
    
    Ok(())
}

// Example 3: Capability delegation
pub fn capability_delegation_example() -> Result<(), CapabilityError> {
    // Create a registry
    let mut registry = ResourceRegistry::new();
    let alice = "alice".to_string();
    let bob = "bob".to_string();
    
    // Register a resource
    let data = "Shared data".to_string();
    let capability = registry.register(data, alice.clone())?;
    
    // Create a delegatable read-only capability
    let delegatable: Capability<String> = Capability {
        id: capability.id.clone(),
        grants: CapabilityGrants::new(true, false, true), // read + delegate
        origin: Some(alice.clone()),
        _phantom: PhantomData,
    };
    
    // Transfer the capability to Bob
    registry.transfer_capability(&delegatable, &alice, &bob)?;
    
    // Bob can now access the resource
    assert!(registry.has_capability(&bob, &delegatable.id)?);
    
    // Bob can read the resource
    let bob_guard = registry.access(&delegatable)?;
    let data = bob_guard.read()?;
    assert_eq!(*data, "Shared data".to_string());
    
    Ok(())
}

// Example 4: Working with complex resources
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComplexResource {
    name: String,
    data: Vec<u8>,
    metadata: std::collections::HashMap<String, String>,
}

pub fn complex_resource_example() -> Result<(), CapabilityError> {
    // Create a registry
    let mut registry = ResourceRegistry::new();
    let alice = "alice".to_string();
    
    // Create a complex resource
    let resource = ComplexResource {
        name: "Complex".to_string(),
        data: vec![1, 2, 3],
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("created".to_string(), "today".to_string());
            map.insert("owner".to_string(), "alice".to_string());
            map
        },
    };
    
    // Register the resource
    let capability = registry.register(resource, alice.clone())?;
    
    // Access the resource
    let mut guard = registry.access(&capability)?;
    
    // Modify the resource
    {
        let resource = guard.write()?;
        resource.data.push(4);
        resource.metadata.insert("modified".to_string(), "now".to_string());
    }
    
    // Read the modified resource
    let resource = guard.read()?;
    assert_eq!(resource.data, vec![1, 2, 3, 4]);
    assert_eq!(resource.metadata.get("modified"), Some(&"now".to_string()));
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_capability() {
        basic_capability_example().unwrap();
    }
    
    #[test]
    fn test_content_addressed_capability() {
        content_addressed_example().unwrap();
    }
    
    #[test]
    fn test_capability_delegation() {
        capability_delegation_example().unwrap();
    }
    
    #[test]
    fn test_complex_resource() {
        complex_resource_example().unwrap();
    }
} 