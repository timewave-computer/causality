// Resource request handling
// Original file: src/resource/request.rs

// Resource request module for Causality Content-Addressed Code System
//
// This module provides types for requesting and granting resources,
// enabling controlled resource allocation and tracking.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

// Global counter for generating unique IDs
static GRANT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// A unique identifier for a resource grant
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GrantId(String);

impl GrantId {
    /// Create a new random grant ID
    pub fn new() -> Self {
        // Get current timestamp as milliseconds
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
            
        // Get and increment the counter
        let counter = GRANT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // Combine into a unique string ID
        GrantId(format!("grant-{}-{}", timestamp, counter))
    }
    
    /// Create a grant ID from a string
    pub fn from_string(id: String) -> Self {
        GrantId(id)
    }
    
    /// Get the string representation of this grant ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GrantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A request for execution resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    /// Memory in bytes
    pub memory_bytes: usize,
    /// CPU time in milliseconds
    pub cpu_millis: usize,
    /// Number of I/O operations
    pub io_operations: usize,
    /// Number of effects
    pub effect_count: usize,
    /// Optional description for this request
    pub description: Option<String>,
}

impl ResourceRequest {
    /// Create a new resource request
    pub fn new(
        memory_bytes: usize,
        cpu_millis: usize,
        io_operations: usize,
        effect_count: usize,
    ) -> Self {
        ResourceRequest {
            memory_bytes,
            cpu_millis,
            io_operations,
            effect_count,
            description: None,
        }
    }
    
    /// Create a new resource request with a description
    pub fn with_description(
        memory_bytes: usize,
        cpu_millis: usize,
        io_operations: usize,
        effect_count: usize,
        description: String,
    ) -> Self {
        ResourceRequest {
            memory_bytes,
            cpu_millis,
            io_operations,
            effect_count,
            description: Some(description),
        }
    }
    
    /// Set the description for this request
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }
    
    /// Get a builder for resource requests
    pub fn builder() -> ResourceRequestBuilder {
        ResourceRequestBuilder::new()
    }
}

/// A builder for resource requests
#[derive(Debug, Default)]
pub struct ResourceRequestBuilder {
    memory_bytes: usize,
    cpu_millis: usize,
    io_operations: usize,
    effect_count: usize,
    description: Option<String>,
}

impl ResourceRequestBuilder {
    /// Create a new resource request builder
    pub fn new() -> Self {
        ResourceRequestBuilder {
            memory_bytes: 0,
            cpu_millis: 0,
            io_operations: 0,
            effect_count: 0,
            description: None,
        }
    }
    
    /// Set the memory bytes
    pub fn memory_bytes(mut self, bytes: usize) -> Self {
        self.memory_bytes = bytes;
        self
    }
    
    /// Set the CPU milliseconds
    pub fn cpu_millis(mut self, millis: usize) -> Self {
        self.cpu_millis = millis;
        self
    }
    
    /// Set the I/O operations
    pub fn io_operations(mut self, ops: usize) -> Self {
        self.io_operations = ops;
        self
    }
    
    /// Set the effect count
    pub fn effect_count(mut self, count: usize) -> Self {
        self.effect_count = count;
        self
    }
    
    /// Set the description
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// Build the resource request
    pub fn build(self) -> ResourceRequest {
        ResourceRequest {
            memory_bytes: self.memory_bytes,
            cpu_millis: self.cpu_millis,
            io_operations: self.io_operations,
            effect_count: self.effect_count,
            description: self.description,
        }
    }
}

/// A grant of resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGrant {
    /// Unique ID for this grant
    pub grant_id: GrantId,
    /// Memory in bytes
    pub memory_bytes: usize,
    /// CPU time in milliseconds
    pub cpu_millis: usize,
    /// Number of I/O operations
    pub io_operations: usize,
    /// Number of effects
    pub effect_count: usize,
}

impl ResourceGrant {
    /// Create a new resource grant
    pub fn new(
        grant_id: GrantId,
        memory_bytes: usize,
        cpu_millis: usize,
        io_operations: usize,
        effect_count: usize,
    ) -> Self {
        ResourceGrant {
            grant_id,
            memory_bytes,
            cpu_millis,
            io_operations,
            effect_count,
        }
    }
    
    /// Create a resource grant from a request
    pub fn from_request(request: &ResourceRequest) -> Self {
        ResourceGrant {
            grant_id: GrantId::new(),
            memory_bytes: request.memory_bytes,
            cpu_millis: request.cpu_millis,
            io_operations: request.io_operations,
            effect_count: request.effect_count,
        }
    }
    
    /// Get the grant ID
    pub fn id(&self) -> &GrantId {
        &self.grant_id
    }
    
    /// Check if this grant has sufficient resources for a request
    pub fn has_sufficient_resources(&self, request: &ResourceRequest) -> bool {
        self.memory_bytes >= request.memory_bytes
            && self.cpu_millis >= request.cpu_millis
            && self.io_operations >= request.io_operations
            && self.effect_count >= request.effect_count
    }
    
    /// Subtract resources from this grant
    pub fn subtract_resources(&mut self, resources: &ResourceGrant) {
        self.memory_bytes = self.memory_bytes.saturating_sub(resources.memory_bytes);
        self.cpu_millis = self.cpu_millis.saturating_sub(resources.cpu_millis);
        self.io_operations = self.io_operations.saturating_sub(resources.io_operations);
        self.effect_count = self.effect_count.saturating_sub(resources.effect_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grant_id_creation() {
        let id1 = GrantId::new();
        let id2 = GrantId::new();
        
        assert_ne!(id1, id2);
        
        let string_id = "test-id".to_string();
        let id3 = GrantId::from_string(string_id.clone());
        
        assert_eq!(id3.as_str(), string_id);
    }
    
    #[test]
    fn test_resource_request() {
        let request = ResourceRequest::new(1024, 1000, 100, 50);
        
        assert_eq!(request.memory_bytes, 1024);
        assert_eq!(request.cpu_millis, 1000);
        assert_eq!(request.io_operations, 100);
        assert_eq!(request.effect_count, 50);
        assert_eq!(request.description, None);
        
        let request_with_desc = ResourceRequest::with_description(
            1024, 1000, 100, 50, "Test request".to_string(),
        );
        
        assert_eq!(request_with_desc.description, Some("Test request".to_string()));
    }
    
    #[test]
    fn test_resource_request_builder() {
        let request = ResourceRequest::builder()
            .memory_bytes(1024)
            .cpu_millis(1000)
            .io_operations(100)
            .effect_count(50)
            .description("Test builder".to_string())
            .build();
        
        assert_eq!(request.memory_bytes, 1024);
        assert_eq!(request.cpu_millis, 1000);
        assert_eq!(request.io_operations, 100);
        assert_eq!(request.effect_count, 50);
        assert_eq!(request.description, Some("Test builder".to_string()));
    }
    
    #[test]
    fn test_resource_grant() {
        let request = ResourceRequest::new(1024, 1000, 100, 50);
        let grant = ResourceGrant::from_request(&request);
        
        assert_eq!(grant.memory_bytes, request.memory_bytes);
        assert_eq!(grant.cpu_millis, request.cpu_millis);
        assert_eq!(grant.io_operations, request.io_operations);
        assert_eq!(grant.effect_count, request.effect_count);
        
        let smaller_request = ResourceRequest::new(512, 500, 50, 25);
        assert!(grant.has_sufficient_resources(&smaller_request));
        
        let larger_request = ResourceRequest::new(2048, 2000, 200, 100);
        assert!(!grant.has_sufficient_resources(&larger_request));
    }
    
    #[test]
    fn test_subtract_resources() {
        let mut grant = ResourceGrant::new(
            GrantId::new(),
            1024,
            1000,
            100,
            50,
        );
        
        let subtraction = ResourceGrant::new(
            GrantId::new(),
            512,
            500,
            50,
            25,
        );
        
        grant.subtract_resources(&subtraction);
        
        assert_eq!(grant.memory_bytes, 512);
        assert_eq!(grant.cpu_millis, 500);
        assert_eq!(grant.io_operations, 50);
        assert_eq!(grant.effect_count, 25);
    }
} 