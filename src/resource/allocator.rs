// Resource allocator module for Causality Content-Addressed Code System
//
// This module defines the ResourceAllocator trait, which provides an interface
// for requesting, tracking, and managing resource usage.

use std::fmt;
use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::resource::{ResourceRequest, ResourceGrant};
use crate::resource::ResourceUsage;
use crate::ast::AstContext;

/// An error that can occur during resource allocation
#[derive(Debug, Clone)]
pub enum AllocationError {
    /// The requested resources exceed the available resources
    InsufficientResources(String),
    /// The grant is invalid or has been released
    InvalidGrant(String),
    /// The allocation failed for another reason
    AllocationFailed(String),
    /// Allocation timeout
    Timeout(String),
    /// Other allocation error
    Other(String),
}

impl fmt::Display for AllocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocationError::InsufficientResources(msg) => write!(f, "Insufficient resources: {}", msg),
            AllocationError::InvalidGrant(msg) => write!(f, "Invalid grant: {}", msg),
            AllocationError::AllocationFailed(msg) => write!(f, "Allocation failed: {}", msg),
            AllocationError::Timeout(msg) => write!(f, "Allocation timeout: {}", msg),
            AllocationError::Other(msg) => write!(f, "Allocation error: {}", msg),
        }
    }
}

impl std::error::Error for AllocationError {}

impl From<AllocationError> for Error {
    fn from(err: AllocationError) -> Self {
        match err {
            AllocationError::InsufficientResources(msg) => Error::OperationFailed(format!("Insufficient resources: {}", msg)),
            AllocationError::InvalidGrant(msg) => Error::OperationFailed(format!("Invalid grant: {}", msg)),
            AllocationError::AllocationFailed(msg) => Error::OperationFailed(format!("Allocation failed: {}", msg)),
            AllocationError::Timeout(msg) => Error::Timeout(format!("Allocation timeout: {}", msg)),
            AllocationError::Other(msg) => Error::OperationFailed(format!("Allocation error: {}", msg)),
        }
    }
}

/// Trait for resource allocation
#[async_trait]
pub trait ResourceAllocator: Send + Sync {
    /// Allocate resources based on a request
    async fn allocate(&self, request: &ResourceRequest) -> Result<ResourceGrant>;
    
    /// Allocate resources with AST context for resource attribution
    async fn allocate_with_context(&self, request: &ResourceRequest, context: &AstContext) -> Result<ResourceGrant> {
        // Default implementation falls back to regular allocation
        // Implementors should override this to support resource attribution
        self.allocate(request).await
    }
    
    /// Release resources
    fn release(&self, grant: &ResourceGrant) -> Result<()>;
    
    /// Check current resource usage
    fn check_usage(&self, grant: &ResourceGrant) -> ResourceUsage;
    
    /// Subdivide resources for child contexts
    async fn subdivide(
        &self,
        grant: ResourceGrant,
        requests: Vec<ResourceRequest>,
    ) -> Result<Vec<ResourceGrant>>;
    
    /// Subdivide a resource grant with AST context
    async fn subdivide_with_context(
        &self,
        grant: ResourceGrant,
        requests: Vec<(ResourceRequest, AstContext)>
    ) -> Result<Vec<ResourceGrant>> {
        // Default implementation falls back to regular subdivision
        let just_requests = requests.into_iter().map(|(req, _)| req).collect();
        self.subdivide(grant, just_requests).await
    }
    
    /// Validate that a grant is valid and has sufficient resources
    fn validate_grant(&self, grant: &ResourceGrant) -> std::result::Result<(), AllocationError>;
    
    /// Get the name of this allocator
    fn name(&self) -> &str;
}

/// A trait for managing resource limits
pub trait ResourceLimiter: Send + Sync {
    /// Check if a request exceeds the available resources
    fn check_request(&self, request: &ResourceRequest) -> std::result::Result<(), AllocationError>;
    
    /// Update the available resources after an allocation
    fn update_allocation(&self, grant: &ResourceGrant);
    
    /// Update the available resources after a release
    fn update_release(&self, grant: &ResourceGrant);
    
    /// Get the current resource limits
    fn get_limits(&self) -> ResourceUsage;
    
    /// Get the current resource usage
    fn get_usage(&self) -> ResourceUsage;
}

/// A trait for tracking resource usage
pub trait ResourceTracker: Send + Sync {
    /// Record resource usage
    fn record_usage(&self, grant: &ResourceGrant, usage: ResourceUsage);
    
    /// Get the current resource usage for a grant
    fn get_usage(&self, grant: &ResourceGrant) -> ResourceUsage;
    
    /// Reset usage tracking for a grant
    fn reset_usage(&self, grant: &ResourceGrant);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::GrantId;
    
    // Mock implementations for testing
    struct MockAllocator;
    
    impl ResourceAllocator for MockAllocator {
        async fn allocate(&self, request: &ResourceRequest) -> Result<ResourceGrant> {
            Ok(ResourceGrant {
                grant_id: GrantId::new(),
                memory_bytes: request.memory_bytes,
                cpu_millis: request.cpu_millis,
                io_operations: request.io_operations,
                effect_count: request.effect_count,
            })
        }
        
        fn release(&self, _grant: &ResourceGrant) -> Result<()> {
            Ok(())
        }
        
        fn check_usage(&self, _grant: &ResourceGrant) -> ResourceUsage {
            ResourceUsage {
                memory_bytes: 0,
                cpu_millis: 0,
                io_operations: 0,
                effect_count: 0,
            }
        }
        
        async fn subdivide(
            &self,
            grant: ResourceGrant,
            requests: Vec<ResourceRequest>,
        ) -> Result<Vec<ResourceGrant>> {
            // Simple mock implementation - just allocate each request
            let mut grants = Vec::new();
            
            // Calculate total requested resources
            let total_memory = requests.iter().map(|r| r.memory_bytes).sum::<usize>();
            let total_cpu = requests.iter().map(|r| r.cpu_millis).sum::<usize>();
            let total_io = requests.iter().map(|r| r.io_operations).sum::<usize>();
            let total_effects = requests.iter().map(|r| r.effect_count).sum::<usize>();
            
            // Check if we have enough resources
            if total_memory > grant.memory_bytes
                || total_cpu > grant.cpu_millis
                || total_io > grant.io_operations
                || total_effects > grant.effect_count
            {
                return Err(AllocationError::InsufficientResources(
                    "Not enough resources for subdivision".to_string(),
                ));
            }
            
            // Allocate resources
            for request in requests {
                grants.push(ResourceGrant {
                    grant_id: GrantId::new(),
                    memory_bytes: request.memory_bytes,
                    cpu_millis: request.cpu_millis,
                    io_operations: request.io_operations,
                    effect_count: request.effect_count,
                });
            }
            
            Ok(grants)
        }
        
        fn validate_grant(&self, _grant: &ResourceGrant) -> std::result::Result<(), AllocationError> {
            Ok(())
        }
        
        fn name(&self) -> &str {
            "MockAllocator"
        }
    }
    
    #[test]
    fn test_mock_allocator() {
        let allocator = MockAllocator;
        
        let request = ResourceRequest {
            memory_bytes: 1024,
            cpu_millis: 1000,
            io_operations: 100,
            effect_count: 50,
        };
        
        let grant = allocator.allocate(&request).unwrap();
        
        assert_eq!(grant.memory_bytes, request.memory_bytes);
        assert_eq!(grant.cpu_millis, request.cpu_millis);
        assert_eq!(grant.io_operations, request.io_operations);
        assert_eq!(grant.effect_count, request.effect_count);
        
        let usage = allocator.check_usage(&grant);
        assert_eq!(usage.memory_bytes, 0);
        assert_eq!(usage.cpu_millis, 0);
        assert_eq!(usage.io_operations, 0);
        assert_eq!(usage.effect_count, 0);
    }
    
    #[test]
    fn test_mock_subdivision() {
        let allocator = MockAllocator;
        
        let parent_request = ResourceRequest {
            memory_bytes: 2048,
            cpu_millis: 2000,
            io_operations: 200,
            effect_count: 100,
        };
        
        let parent_grant = allocator.allocate(&parent_request).unwrap();
        
        let child_requests = vec![
            ResourceRequest {
                memory_bytes: 1024,
                cpu_millis: 1000,
                io_operations: 100,
                effect_count: 50,
            },
            ResourceRequest {
                memory_bytes: 1024,
                cpu_millis: 1000,
                io_operations: 100,
                effect_count: 50,
            },
        ];
        
        let child_grants = allocator.subdivide(parent_grant, child_requests).unwrap();
        
        assert_eq!(child_grants.len(), 2);
        assert_eq!(child_grants[0].memory_bytes, 1024);
        assert_eq!(child_grants[1].memory_bytes, 1024);
    }
} 