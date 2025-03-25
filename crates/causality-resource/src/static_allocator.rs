// Static resource allocation
// Original file: src/resource/static_alloc.rs

// Static resource allocator for Causality Content-Addressed Code System
//
// This module provides a simple static resource allocator implementation that
// manages fixed resource limits.

use std::sync::{Mutex};
use std::collections::HashMap;
use std::sync::{Arc};
use async_trait::async_trait;

use super::{ResourceAllocator, AllocationError};
use super::{ResourceRequest, ResourceGrant, GrantId};
use super::ResourceUsage;
use causality_types::Result;
use causality_patterns::{AstContext, CorrelationTracker};

/// A static resource allocator with fixed limits
#[derive(Debug, Clone)]
pub struct StaticAllocator {
    /// Name of this allocator
    name: String,
    /// Maximum resources available for allocation
    max_resources: ResourceUsage,
    /// Currently allocated resources
    allocated: Arc<Mutex<ResourceUsage>>,
    /// Map of currently active grants
    active_grants: Arc<Mutex<HashMap<String, ResourceUsage>>>,
    /// Correlation tracker for AST-resource attribution, if enabled
    correlation_tracker: Option<Arc<CorrelationTracker>>,
}

impl StaticAllocator {
    /// Create a new static allocator with given limits
    pub fn new(name: &str, max_resources: ResourceUsage) -> Self {
        StaticAllocator {
            name: name.to_string(),
            max_resources,
            allocated: Arc::new(Mutex::new(ResourceUsage::new())),
            active_grants: Arc::new(Mutex::new(HashMap::new())),
            correlation_tracker: None,
        }
    }
    
    /// Get the maximum resources available
    pub fn max_resources(&self) -> &ResourceUsage {
        &self.max_resources
    }
    
    /// Get a snapshot of currently allocated resources
    pub fn current_allocated(&self) -> ResourceUsage {
        self.allocated.lock().unwrap().clone()
    }
    
    /// Calculate remaining available resources
    pub fn available_resources(&self) -> ResourceUsage {
        let allocated = self.allocated.lock().unwrap();
        ResourceUsage {
            memory_bytes: self.max_resources.memory_bytes.saturating_sub(allocated.memory_bytes),
            cpu_millis: self.max_resources.cpu_millis.saturating_sub(allocated.cpu_millis),
            io_operations: self.max_resources.io_operations.saturating_sub(allocated.io_operations),
            effect_count: self.max_resources.effect_count.saturating_sub(allocated.effect_count),
        }
    }
    
    /// Check if the requested resources are available
    fn can_allocate(&self, request: &ResourceRequest) -> bool {
        let available = self.available_resources();
        
        request.memory_bytes <= available.memory_bytes
            && request.cpu_millis <= available.cpu_millis
            && request.io_operations <= available.io_operations
            && request.effect_count <= available.effect_count
    }
    
    /// Add allocated resources to the allocator's tracking
    fn add_to_allocated(&self, usage: &ResourceUsage) {
        let mut allocated = self.allocated.lock().unwrap();
        allocated.add(usage);
    }
    
    /// Remove allocated resources from the allocator's tracking
    fn remove_from_allocated(&self, usage: &ResourceUsage) {
        let mut allocated = self.allocated.lock().unwrap();
        allocated.memory_bytes = allocated.memory_bytes.saturating_sub(usage.memory_bytes);
        allocated.cpu_millis = allocated.cpu_millis.saturating_sub(usage.cpu_millis);
        allocated.io_operations = allocated.io_operations.saturating_sub(usage.io_operations);
        allocated.effect_count = allocated.effect_count.saturating_sub(usage.effect_count);
    }
    
    /// Internal method to get usage for a grant ID
    fn get_grant_usage(&self, grant_id: &str) -> Option<ResourceUsage> {
        self.active_grants.lock().unwrap().get(grant_id).cloned()
    }
    
    /// Add a correlation tracker for AST-resource attribution
    pub fn with_correlation_tracker(mut self, tracker: Arc<CorrelationTracker>) -> Self {
        self.correlation_tracker = Some(tracker);
        self
    }
}

#[async_trait]
impl ResourceAllocator for StaticAllocator {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn allocate(&self, request: &ResourceRequest) -> Result<ResourceGrant> {
        // Check if we have enough resources for the request
        let mut allocated = self.allocated.lock().unwrap();
        
        // Calculate what would be the new allocated total if we approve this
        let new_allocated = ResourceUsage {
            memory_bytes: allocated.memory_bytes + request.memory_bytes,
            cpu_millis: allocated.cpu_millis + request.cpu_millis,
            io_operations: allocated.io_operations + request.io_operations,
            effect_count: allocated.effect_count + request.effect_count,
        };
        
        // Check against maximum resources
        if new_allocated.memory_bytes > self.max_resources.memory_bytes {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient memory: available {}, requested {}", 
                    self.max_resources.memory_bytes - allocated.memory_bytes,
                    request.memory_bytes)
            ).into());
        }
        
        if new_allocated.cpu_millis > self.max_resources.cpu_millis {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient CPU: available {}, requested {}", 
                    self.max_resources.cpu_millis - allocated.cpu_millis,
                    request.cpu_millis)
            ).into());
        }
        
        if new_allocated.io_operations > self.max_resources.io_operations {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient IO: available {}, requested {}", 
                    self.max_resources.io_operations - allocated.io_operations,
                    request.io_operations)
            ).into());
        }
        
        if new_allocated.effect_count > self.max_resources.effect_count {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient effects: available {}, requested {}", 
                    self.max_resources.effect_count - allocated.effect_count,
                    request.effect_count)
            ).into());
        }
        
        // All checks passed, we can allocate these resources
        *allocated = new_allocated;
        
        // Create a resource grant
        let grant = ResourceGrant::from_request(request);
        
        // Store the grant in our active grants map
        let usage = ResourceUsage {
            memory_bytes: request.memory_bytes,
            cpu_millis: request.cpu_millis,
            io_operations: request.io_operations,
            effect_count: request.effect_count,
        };
        self.active_grants.lock().unwrap().insert(grant.grant_id.to_string(), usage);
        
        Ok(grant)
    }
    
    async fn allocate_with_context(&self, request: &ResourceRequest, context: &AstContext) -> Result<ResourceGrant> {
        // First, perform the regular allocation
        let grant = self.allocate(request).await?;
        
        // If we have a correlation tracker, record the allocation
        if let Some(tracker) = &self.correlation_tracker {
            tracker.record_allocation(
                context.ast_node_id.clone(),
                grant.grant_id.clone(),
                &grant
            )?;
        }
        
        Ok(grant)
    }
    
    fn release(&self, grant: &ResourceGrant) -> Result<()> {
        let mut allocated = self.allocated.lock().unwrap();
        let mut active_grants = self.active_grants.lock().unwrap();
        
        // Find the grant in our active grants map
        let usage = match active_grants.remove(&grant.grant_id.to_string()) {
            Some(usage) => usage,
            None => return Err(AllocationError::InvalidGrant(
                format!("Grant {} not found in active grants", grant.grant_id)
            ).into()),
        };
        
        // Update allocated resources
        allocated.memory_bytes -= usage.memory_bytes;
        allocated.cpu_millis -= usage.cpu_millis;
        allocated.io_operations -= usage.io_operations;
        allocated.effect_count -= usage.effect_count;
        
        Ok(())
    }
    
    fn check_usage(&self, grant: &ResourceGrant) -> ResourceUsage {
        match self.get_grant_usage(&grant.grant_id.to_string()) {
            Some(usage) => usage,
            None => ResourceUsage::new(),
        }
    }
    
    async fn subdivide(
        &self, 
        grant: ResourceGrant, 
        requests: Vec<ResourceRequest>
    ) -> Result<Vec<ResourceGrant>> {
        // Calculate total requested resources
        let mut total_memory = 0;
        let mut total_cpu = 0;
        let mut total_io = 0;
        let mut total_effects = 0;
        
        for req in &requests {
            total_memory += req.memory_bytes;
            total_cpu += req.cpu_millis;
            total_io += req.io_operations;
            total_effects += req.effect_count;
        }
        
        // Check if the grant has enough resources for all requests
        if total_memory > grant.memory_bytes {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient memory: available {}, requested {}", 
                    grant.memory_bytes, total_memory)
            ).into());
        }
        
        if total_cpu > grant.cpu_millis {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient CPU: available {}, requested {}", 
                    grant.cpu_millis, total_cpu)
            ).into());
        }
        
        if total_io > grant.io_operations {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient IO: available {}, requested {}", 
                    grant.io_operations, total_io)
            ).into());
        }
        
        if total_effects > grant.effect_count {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient effects: available {}, requested {}", 
                    grant.effect_count, total_effects)
            ).into());
        }
        
        // Release the parent grant
        self.release(&grant)?;
        
        // Allocate resources for each request
        let mut grants = Vec::new();
        
        for req in requests {
            let grant = self.allocate(&req).await?;
            grants.push(grant);
        }
        
        Ok(grants)
    }
    
    async fn subdivide_with_context(
        &self,
        grant: ResourceGrant,
        requests: Vec<(ResourceRequest, AstContext)>
    ) -> Result<Vec<ResourceGrant>> {
        // Calculate total requested resources
        let mut total_memory = 0;
        let mut total_cpu = 0;
        let mut total_io = 0;
        let mut total_effects = 0;
        
        for (req, _) in &requests {
            total_memory += req.memory_bytes;
            total_cpu += req.cpu_millis;
            total_io += req.io_operations;
            total_effects += req.effect_count;
        }
        
        // Check if the grant has enough resources for all requests
        if total_memory > grant.memory_bytes {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient memory: available {}, requested {}", 
                    grant.memory_bytes, total_memory)
            ).into());
        }
        
        if total_cpu > grant.cpu_millis {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient CPU: available {}, requested {}", 
                    grant.cpu_millis, total_cpu)
            ).into());
        }
        
        if total_io > grant.io_operations {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient IO: available {}, requested {}", 
                    grant.io_operations, total_io)
            ).into());
        }
        
        if total_effects > grant.effect_count {
            return Err(AllocationError::InsufficientResources(
                format!("Insufficient effects: available {}, requested {}", 
                    grant.effect_count, total_effects)
            ).into());
        }
        
        // Release the parent grant
        self.release(&grant)?;
        
        // Allocate new grants for each request with context
        let mut grants = Vec::new();
        for (req, ctx) in requests {
            let grant = self.allocate_with_context(&req, &ctx).await?;
            grants.push(grant);
        }
        
        Ok(grants)
    }
    
    fn validate_grant(&self, grant: &ResourceGrant) -> std::result::Result<(), AllocationError> {
        // Check if the grant is in our active grants map
        if self.active_grants.lock().unwrap().contains_key(&grant.grant_id.to_string()) {
            Ok(())
        } else {
            Err(AllocationError::InvalidGrant(format!("Grant {} not found in active grants", grant.grant_id)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_static_allocator_creation() {
        let max = ResourceUsage::with_values(1024 * 1024, 5000, 1000, 500);
        let allocator = StaticAllocator::new("test_allocator", max.clone());
        
        assert_eq!(allocator.name(), "test_allocator");
        assert_eq!(allocator.max_resources().memory_bytes, max.memory_bytes);
        assert_eq!(allocator.max_resources().cpu_millis, max.cpu_millis);
        assert_eq!(allocator.max_resources().io_operations, max.io_operations);
        assert_eq!(allocator.max_resources().effect_count, max.effect_count);
        
        // Initially all resources should be available
        let available = allocator.available_resources();
        assert_eq!(available.memory_bytes, max.memory_bytes);
        assert_eq!(available.cpu_millis, max.cpu_millis);
        assert_eq!(available.io_operations, max.io_operations);
        assert_eq!(available.effect_count, max.effect_count);
    }
    
    #[test]
    fn test_static_allocator_allocation() {
        let max = ResourceUsage::with_values(1024 * 1024, 5000, 1000, 500);
        let allocator = StaticAllocator::new("test_allocator", max);
        
        // Create a resource request
        let request = ResourceRequest {
            memory_bytes: 512 * 1024,
            cpu_millis: 2500,
            io_operations: 500,
            effect_count: 250,
            description: Some("Test allocation".to_string()),
        };
        
        // Allocate resources
        let grant = allocator.allocate(&request).unwrap();
        
        // Check the grant
        assert_eq!(grant.memory_bytes, 512 * 1024);
        assert_eq!(grant.cpu_millis, 2500);
        assert_eq!(grant.io_operations, 500);
        assert_eq!(grant.effect_count, 250);
        
        // Check the allocated resources
        let current = allocator.current_allocated();
        assert_eq!(current.memory_bytes, 512 * 1024);
        assert_eq!(current.cpu_millis, 2500);
        assert_eq!(current.io_operations, 500);
        assert_eq!(current.effect_count, 250);
        
        // Check the available resources
        let available = allocator.available_resources();
        assert_eq!(available.memory_bytes, 512 * 1024);
        assert_eq!(available.cpu_millis, 2500);
        assert_eq!(available.io_operations, 500);
        assert_eq!(available.effect_count, 250);
    }
    
    #[test]
    fn test_static_allocator_insufficient_resources() {
        let max = ResourceUsage::with_values(1024 * 1024, 5000, 1000, 500);
        let allocator = StaticAllocator::new("test_allocator", max);
        
        // Create a request that exceeds available resources
        let request = ResourceRequest {
            memory_bytes: 2 * 1024 * 1024, // 2 MB, which is more than the 1 MB max
            cpu_millis: 2500,
            io_operations: 500,
            effect_count: 250,
            description: Some("Test allocation".to_string()),
        };
        
        // Attempt to allocate resources
        let result = allocator.allocate(&request);
        
        // Should fail with InsufficientResources
        assert!(matches!(result, Err(AllocationError::InsufficientResources(_))));
    }
    
    #[test]
    fn test_static_allocator_release() {
        let max = ResourceUsage::with_values(1024 * 1024, 5000, 1000, 500);
        let allocator = StaticAllocator::new("test_allocator", max);
        
        // Create a resource request
        let request = ResourceRequest {
            memory_bytes: 512 * 1024,
            cpu_millis: 2500,
            io_operations: 500,
            effect_count: 250,
            description: Some("Test allocation".to_string()),
        };
        
        // Allocate resources
        let grant = allocator.allocate(&request).unwrap();
        
        // Release the grant
        allocator.release(&grant).unwrap();
        
        // Check the allocated resources (should be back to zero)
        let current = allocator.current_allocated();
        assert_eq!(current.memory_bytes, 0);
        assert_eq!(current.cpu_millis, 0);
        assert_eq!(current.io_operations, 0);
        assert_eq!(current.effect_count, 0);
        
        // Check the available resources (should be back to max)
        let available = allocator.available_resources();
        assert_eq!(available.memory_bytes, 1024 * 1024);
        assert_eq!(available.cpu_millis, 5000);
        assert_eq!(available.io_operations, 1000);
        assert_eq!(available.effect_count, 500);
    }
    
    #[test]
    fn test_static_allocator_validate_grant() {
        let max = ResourceUsage::with_values(1024 * 1024, 5000, 1000, 500);
        let allocator = StaticAllocator::new("test_allocator", max);
        
        // Create a resource request
        let request = ResourceRequest {
            memory_bytes: 512 * 1024,
            cpu_millis: 2500,
            io_operations: 500,
            effect_count: 250,
            description: Some("Test allocation".to_string()),
        };
        
        // Allocate resources
        let grant = allocator.allocate(&request).unwrap();
        
        // Validate the grant
        assert!(allocator.validate_grant(&grant).is_ok());
        
        // Create an invalid grant
        let invalid_grant = ResourceGrant {
            grant_id: GrantId::new(),
            memory_bytes: 1024,
            cpu_millis: 1000,
            io_operations: 100,
            effect_count: 50,
        };
        
        // Validate the invalid grant (should fail)
        assert!(matches!(allocator.validate_grant(&invalid_grant), Err(AllocationError::InvalidGrant(_))));
    }
    
    #[test]
    fn test_static_allocator_subdivision() {
        let max = ResourceUsage::with_values(1024 * 1024, 5000, 1000, 500);
        let allocator = StaticAllocator::new("test_allocator", max);
        
        // Create a parent grant
        let parent_request = ResourceRequest {
            memory_bytes: 512 * 1024,
            cpu_millis: 2500,
            io_operations: 500,
            effect_count: 250,
            description: Some("Parent allocation".to_string()),
        };
        
        // Allocate resources for the parent
        let parent_grant = allocator.allocate(&parent_request).unwrap();
        
        // Create two child requests
        let child_requests = vec![
            ResourceRequest {
                memory_bytes: 256 * 1024,
                cpu_millis: 1000,
                io_operations: 200,
                effect_count: 100,
                description: Some("Child 1".to_string()),
            },
            ResourceRequest {
                memory_bytes: 256 * 1024,
                cpu_millis: 1500,
                io_operations: 300,
                effect_count: 150,
                description: Some("Child 2".to_string()),
            },
        ];
        
        // Subdivide the parent grant
        let child_grants = allocator.subdivide(parent_grant, child_requests).unwrap();
        
        // Check that we got two child grants
        assert_eq!(child_grants.len(), 2);
        
        // Check the first child grant
        assert_eq!(child_grants[0].memory_bytes, 256 * 1024);
        assert_eq!(child_grants[0].cpu_millis, 1000);
        assert_eq!(child_grants[0].io_operations, 200);
        assert_eq!(child_grants[0].effect_count, 100);
        
        // Check the second child grant
        assert_eq!(child_grants[1].memory_bytes, 256 * 1024);
        assert_eq!(child_grants[1].cpu_millis, 1500);
        assert_eq!(child_grants[1].io_operations, 300);
        assert_eq!(child_grants[1].effect_count, 150);
    }
    
    #[test]
    fn test_static_allocator_check_usage() {
        let max = ResourceUsage::with_values(1024 * 1024, 5000, 1000, 500);
        let allocator = StaticAllocator::new("test_allocator", max);
        
        // Create a resource request
        let request = ResourceRequest {
            memory_bytes: 512 * 1024,
            cpu_millis: 2500,
            io_operations: 500,
            effect_count: 250,
            description: Some("Test allocation".to_string()),
        };
        
        // Allocate resources
        let grant = allocator.allocate(&request).unwrap();
        
        // Check usage
        let usage = allocator.check_usage(&grant);
        
        // Verify the usage matches the grant
        assert_eq!(usage.memory_bytes, grant.memory_bytes);
        assert_eq!(usage.cpu_millis, grant.cpu_millis);
        assert_eq!(usage.io_operations, grant.io_operations);
        assert_eq!(usage.effect_count, grant.effect_count);
    }
} 