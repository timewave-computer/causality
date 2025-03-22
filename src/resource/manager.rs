// Resource Manager for Causality Resource System
//
// This module provides a high-level interface for resource management,
// with support for register fact emission to track register operations.

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use crate::types::{ResourceId, DomainId, TraceId};
use crate::error::{Error, Result};
use crate::resource::{
    ResourceAllocator, 
    ResourceRequest, 
    ResourceGrant, 
    ResourceUsage,
    GrantId
};
use crate::log::FactLogger;
use crate::domain::fact::register_observer::RegisterFactObserver;
use crate::resource::register::RegisterId;

/// Resource Guard that automatically releases resources when dropped
pub struct ResourceGuard {
    /// The resource grant
    grant: Option<ResourceGrant>,
    /// The resource manager
    manager: Arc<ResourceManager>,
    /// The resource ID (if associated with a register)
    register_id: Option<RegisterId>,
}

impl ResourceGuard {
    /// Create a new resource guard
    fn new(grant: ResourceGrant, manager: Arc<ResourceManager>) -> Self {
        ResourceGuard {
            grant: Some(grant),
            manager,
            register_id: None,
        }
    }
    
    /// Create a new resource guard with a register ID
    fn with_register(grant: ResourceGrant, manager: Arc<ResourceManager>, register_id: RegisterId) -> Self {
        ResourceGuard {
            grant: Some(grant),
            manager,
            register_id: Some(register_id),
        }
    }
    
    /// Get a reference to the grant
    pub fn grant(&self) -> Option<&ResourceGrant> {
        self.grant.as_ref()
    }
    
    /// Get the grant ID
    pub fn grant_id(&self) -> Option<&GrantId> {
        self.grant.as_ref().map(|g| g.id())
    }
    
    /// Get the register ID
    pub fn register_id(&self) -> Option<&RegisterId> {
        self.register_id.as_ref()
    }
    
    /// Release the resources manually
    pub fn release(&mut self) {
        if let Some(grant) = self.grant.take() {
            self.manager.release_resources(grant);
        }
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        self.release();
    }
}

/// Manager for resource allocation and tracking
pub struct ResourceManager {
    /// The underlying resource allocator
    allocator: Arc<dyn ResourceAllocator>,
    /// Active resource grants
    active_grants: RwLock<HashMap<GrantId, ResourceGrant>>,
    /// Register fact observer for emitting register facts
    register_observer: Option<Arc<RegisterFactObserver>>,
    /// Domain ID for this manager
    domain_id: DomainId,
    /// Current trace ID
    current_trace: Mutex<Option<TraceId>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(allocator: Arc<dyn ResourceAllocator>, domain_id: DomainId) -> Self {
        ResourceManager {
            allocator,
            active_grants: RwLock::new(HashMap::new()),
            register_observer: None,
            domain_id,
            current_trace: Mutex::new(None),
        }
    }
    
    /// Create a new resource manager with register fact observation
    pub fn with_register_observation(
        allocator: Arc<dyn ResourceAllocator>,
        domain_id: DomainId,
        fact_logger: Arc<FactLogger>,
    ) -> Self {
        let observer = Arc::new(RegisterFactObserver::new(
            fact_logger,
            domain_id.clone(),
            "resource-manager".to_string(),
        ));
        
        ResourceManager {
            allocator,
            active_grants: RwLock::new(HashMap::new()),
            register_observer: Some(observer),
            domain_id,
            current_trace: Mutex::new(None),
        }
    }
    
    /// Set the current trace ID
    pub fn set_trace(&self, trace_id: TraceId) {
        let mut current = self.current_trace.lock().unwrap();
        *current = Some(trace_id);
    }
    
    /// Get the current trace ID
    fn get_trace(&self) -> Option<TraceId> {
        let current = self.current_trace.lock().unwrap();
        current.clone()
    }
    
    /// Allocate resources
    pub fn allocate_resources(&self, request: ResourceRequest) -> Result<ResourceGuard> {
        let grant = self.allocator.allocate(request)
            .map_err(|e| Error::ResourceError(format!("Failed to allocate resources: {}", e)))?;
        
        // Add to active grants
        let mut active_grants = self.active_grants.write().unwrap();
        active_grants.insert(grant.grant_id.clone(), grant.clone());
        
        Ok(ResourceGuard::new(grant, Arc::new(self.clone())))
    }
    
    /// Release resources
    pub fn release_resources(&self, grant: ResourceGrant) {
        // Remove from active grants
        let mut active_grants = self.active_grants.write().unwrap();
        active_grants.remove(&grant.grant_id);
        
        // Release through allocator
        self.allocator.release(grant);
    }
    
    /// Create a register with initial data
    pub fn create_register(
        &self,
        register_id: RegisterId,
        initial_data: &[u8],
        request: ResourceRequest,
    ) -> Result<ResourceGuard> {
        // Allocate resources for the register
        let grant = self.allocate_resources(request)?.grant().ok_or_else(|| {
            Error::ResourceError("No grant returned from allocator".to_string())
        })?.clone();
        
        // Emit a register creation fact if an observer is configured
        if let Some(observer) = &self.register_observer {
            observer.observe_register_creation(
                &register_id,
                initial_data,
                self.get_trace(),
            )?;
        }
        
        Ok(ResourceGuard::with_register(grant, Arc::new(self.clone()), register_id))
    }
    
    /// Update a register with new data
    pub fn update_register(
        &self,
        register_id: RegisterId,
        new_data: &[u8],
        previous_version: &str,
    ) -> Result<()> {
        // Emit a register update fact if an observer is configured
        if let Some(observer) = &self.register_observer {
            observer.observe_register_update(
                &register_id,
                new_data,
                previous_version,
                self.get_trace(),
            )?;
        }
        
        Ok(())
    }
    
    /// Transfer a register between domains
    pub fn transfer_register(
        &self,
        register_id: RegisterId,
        source_domain: &str,
        target_domain: &str,
    ) -> Result<()> {
        // Emit a register transfer fact if an observer is configured
        if let Some(observer) = &self.register_observer {
            observer.observe_register_transfer(
                &register_id,
                source_domain,
                target_domain,
                self.get_trace(),
            )?;
        }
        
        Ok(())
    }
    
    /// Merge multiple registers into a single one
    pub fn merge_registers(
        &self,
        source_registers: &[RegisterId],
        result_register: RegisterId,
        request: ResourceRequest,
    ) -> Result<ResourceGuard> {
        // Allocate resources for the merged register
        let grant = self.allocate_resources(request)?.grant().ok_or_else(|| {
            Error::ResourceError("No grant returned from allocator".to_string())
        })?.clone();
        
        // Emit a register merge fact if an observer is configured
        if let Some(observer) = &self.register_observer {
            observer.observe_register_merge(
                source_registers,
                &result_register,
                self.get_trace(),
            )?;
        }
        
        Ok(ResourceGuard::with_register(grant, Arc::new(self.clone()), result_register))
    }
    
    /// Split a register into multiple ones
    pub fn split_register(
        &self,
        source_register: RegisterId,
        result_registers: &[RegisterId],
        requests: Vec<ResourceRequest>,
    ) -> Result<Vec<ResourceGuard>> {
        // Ensure we have the same number of requests as result registers
        if requests.len() != result_registers.len() {
            return Err(Error::ResourceError(
                "Number of requests must match number of result registers".to_string()
            ));
        }
        
        // Allocate resources for each result register
        let mut grants = Vec::with_capacity(requests.len());
        for request in requests {
            let grant = self.allocate_resources(request)?.grant().ok_or_else(|| {
                Error::ResourceError("No grant returned from allocator".to_string())
            })?.clone();
            grants.push(grant);
        }
        
        // Emit a register split fact if an observer is configured
        if let Some(observer) = &self.register_observer {
            observer.observe_register_split(
                &source_register,
                result_registers,
                self.get_trace(),
            )?;
        }
        
        // Create resource guards for each result register
        let mut result_guards = Vec::with_capacity(result_registers.len());
        for i in 0..result_registers.len() {
            result_guards.push(ResourceGuard::with_register(
                grants[i].clone(), 
                Arc::new(self.clone()), 
                result_registers[i].clone()
            ));
        }
        
        Ok(result_guards)
    }
    
    /// Get the resource usage for a grant
    pub fn check_usage(&self, guard: &ResourceGuard) -> Result<ResourceUsage> {
        if let Some(grant) = guard.grant() {
            Ok(self.allocator.check_usage(grant))
        } else {
            Err(Error::ResourceError("Guard has no associated grant".to_string()))
        }
    }
    
    /// Validate a resource guard
    pub fn validate_guard(&self, guard: &ResourceGuard) -> Result<()> {
        if let Some(grant) = guard.grant() {
            self.allocator.validate_grant(grant)
                .map_err(|e| Error::ResourceError(format!("Invalid resource grant: {}", e)))
        } else {
            Err(Error::ResourceError("Guard has no associated grant".to_string()))
        }
    }
    
    /// Get the domain ID
    pub fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
}

impl Clone for ResourceManager {
    fn clone(&self) -> Self {
        ResourceManager {
            allocator: self.allocator.clone(),
            active_grants: RwLock::new(HashMap::new()),
            register_observer: self.register_observer.clone(),
            domain_id: self.domain_id.clone(),
            current_trace: Mutex::new(self.get_trace()),
        }
    }
}

/// Shared reference to a ResourceManager
pub type SharedResourceManager = Arc<ResourceManager>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::allocator::StaticAllocator;
    use crate::resource::register::RegisterId;
    use crate::log::MemoryLogStorage;
    use std::sync::Mutex as StdMutex;
    
    #[test]
    fn test_resource_allocation() {
        let allocator = Arc::new(StaticAllocator::new(
            1024 * 1024, // 1MB memory
            1000,        // 1 second CPU
            100,         // 100 I/O ops
            10,          // 10 effects
        ));
        
        let domain_id = DomainId::new("test-domain");
        let manager = ResourceManager::new(allocator, domain_id);
        
        let request = ResourceRequest::new(1024, 100, 10, 1);
        let guard = manager.allocate_resources(request).unwrap();
        
        assert!(guard.grant().is_some());
    }
    
    #[test]
    fn test_create_register() {
        // Create a mock allocator
        let allocator = Arc::new(StaticAllocator::new(
            1024 * 1024, // 1MB memory
            1000,        // 1 second CPU
            100,         // 100 I/O ops
            10,          // 10 effects
        ));
        
        let domain_id = DomainId::new("test-domain");
        let manager = ResourceManager::new(allocator, domain_id);
        
        let register_id = RegisterId::new("test-register");
        let initial_data = vec![1, 2, 3, 4];
        let request = ResourceRequest::new(1024, 100, 10, 1);
        
        let guard = manager.create_register(register_id.clone(), &initial_data, request).unwrap();
        
        assert!(guard.grant().is_some());
        assert_eq!(guard.register_id(), Some(&register_id));
    }
    
    #[test]
    fn test_register_operations_with_facts() {
        // Create a mock allocator
        let allocator = Arc::new(StaticAllocator::new(
            1024 * 1024, // 1MB memory
            1000,        // 1 second CPU
            100,         // 100 I/O ops
            10,          // 10 effects
        ));
        
        // Create a memory log storage
        let storage = Arc::new(StdMutex::new(MemoryLogStorage::new()));
        
        let domain_id = DomainId::new("test-domain");
        let fact_logger = Arc::new(FactLogger::new(storage.clone(), domain_id.clone()));
        
        // Create a resource manager with register fact observation
        let manager = ResourceManager::with_register_observation(
            allocator,
            domain_id,
            fact_logger,
        );
        
        // Set trace ID
        let trace_id = TraceId::new();
        manager.set_trace(trace_id);
        
        // Create a register
        let register_id = RegisterId::new("test-register");
        let initial_data = vec![1, 2, 3, 4];
        let request = ResourceRequest::new(1024, 100, 10, 1);
        
        let guard = manager.create_register(register_id.clone(), &initial_data, request).unwrap();
        assert!(guard.grant().is_some());
        
        // Test update register
        manager.update_register(
            register_id.clone(),
            &vec![5, 6, 7, 8],
            "v1",
        ).unwrap();
        
        // Test register transfer
        manager.transfer_register(
            register_id.clone(),
            "domain-1",
            "domain-2",
        ).unwrap();
    }
    
    #[test]
    fn test_register_merge() {
        // Create a mock allocator
        let allocator = Arc::new(StaticAllocator::new(
            1024 * 1024, // 1MB memory
            1000,        // 1 second CPU
            100,         // 100 I/O ops
            10,          // 10 effects
        ));
        
        // Create a memory log storage
        let storage = Arc::new(StdMutex::new(MemoryLogStorage::new()));
        
        let domain_id = DomainId::new("test-domain");
        let fact_logger = Arc::new(FactLogger::new(storage.clone(), domain_id.clone()));
        
        // Create a resource manager with register fact observation
        let manager = ResourceManager::with_register_observation(
            allocator,
            domain_id,
            fact_logger,
        );
        
        // Set trace ID
        let trace_id = TraceId::new();
        manager.set_trace(trace_id);
        
        // Create source registers
        let register_id1 = RegisterId::new("test-register-1");
        let register_id2 = RegisterId::new("test-register-2");
        let merged_register = RegisterId::new("merged-register");
        
        // Create the first register
        let request1 = ResourceRequest::new(1024, 100, 10, 1);
        let _guard1 = manager.create_register(register_id1.clone(), &vec![1, 2, 3, 4], request1).unwrap();
        
        // Create the second register
        let request2 = ResourceRequest::new(1024, 100, 10, 1);
        let _guard2 = manager.create_register(register_id2.clone(), &vec![5, 6, 7, 8], request2).unwrap();
        
        // Merge the registers
        let merge_request = ResourceRequest::new(2048, 200, 20, 2);
        let merged_guard = manager.merge_registers(
            &[register_id1.clone(), register_id2.clone()],
            merged_register.clone(),
            merge_request,
        ).unwrap();
        
        // Verify the merged guard
        assert!(merged_guard.grant().is_some());
        assert_eq!(merged_guard.register_id(), Some(&merged_register));
        
        // Check the log storage for facts
        let storage_lock = storage.lock().unwrap();
        assert!(!storage_lock.facts.is_empty());
    }
    
    #[test]
    fn test_register_split() {
        // Create a mock allocator
        let allocator = Arc::new(StaticAllocator::new(
            1024 * 1024, // 1MB memory
            1000,        // 1 second CPU
            100,         // 100 I/O ops
            10,          // 10 effects
        ));
        
        // Create a memory log storage
        let storage = Arc::new(StdMutex::new(MemoryLogStorage::new()));
        
        let domain_id = DomainId::new("test-domain");
        let fact_logger = Arc::new(FactLogger::new(storage.clone(), domain_id.clone()));
        
        // Create a resource manager with register fact observation
        let manager = ResourceManager::with_register_observation(
            allocator,
            domain_id,
            fact_logger,
        );
        
        // Set trace ID
        let trace_id = TraceId::new();
        manager.set_trace(trace_id);
        
        // Create source register
        let source_register = RegisterId::new("source-register");
        
        // Create the source register
        let request = ResourceRequest::new(2048, 200, 20, 2);
        let _guard = manager.create_register(source_register.clone(), &vec![1, 2, 3, 4, 5, 6, 7, 8], request).unwrap();
        
        // Define result registers
        let result_register1 = RegisterId::new("result-register-1");
        let result_register2 = RegisterId::new("result-register-2");
        
        // Split the register
        let split_requests = vec![
            ResourceRequest::new(1024, 100, 10, 1),
            ResourceRequest::new(1024, 100, 10, 1),
        ];
        
        let result_guards = manager.split_register(
            source_register.clone(),
            &[result_register1.clone(), result_register2.clone()],
            split_requests,
        ).unwrap();
        
        // Verify the result guards
        assert_eq!(result_guards.len(), 2);
        assert_eq!(result_guards[0].register_id(), Some(&result_register1));
        assert_eq!(result_guards[1].register_id(), Some(&result_register2));
        
        // Check the log storage for facts
        let storage_lock = storage.lock().unwrap();
        assert!(!storage_lock.facts.is_empty());
    }
} 