// Cross-domain resource management effects
//
// This file implements effects for managing resources across domain boundaries,
// demonstrating the integration between the resource management system and
// the effect system.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;

use causality_domain::domain::DomainId;
use causality_types::{Error, Result, ContentId};
use crate::boundary::ExecutionBoundary;
use crate::effect::{Effect, EffectContext, EffectOutcome, EffectResult, EffectError};
use crate::effect_id::EffectId;
use crate::capability::{
    UnifiedCapability, EffectCapability, CrossDomainCapability,
    UnifiedCapabilityContext, EffectContextCapabilityExt
};
use crate::capability::verification::{CapabilityVerifier, DefaultCapabilityVerifier};

use super::access::{ResourceAccessType, ResourceAccessManager};
use super::lifecycle::{ResourceLifecycleEvent, EffectResourceLifecycle};
use super::locking::{CrossDomainLockManager, CrossDomainLockType, LockStatus};
use super::dependency::{ResourceDependencyManager, DependencyType, ResourceDependency};
use super::capability::{ResourceCapabilityManager, ResourceCapability, ResourceLifecycleCapability};

/// Effect for transferring a resource across domains
pub struct CrossDomainResourceTransferEffect {
    /// Effect ID
    id: EffectId,
    
    /// Resource ID to transfer
    resource_id: ContentId,
    
    /// Source domain ID
    source_domain_id: DomainId,
    
    /// Target domain ID
    target_domain_id: DomainId,
    
    /// Optional timeout for the transfer operation
    timeout: Option<Duration>,
    
    /// Optional metadata for the transfer
    metadata: HashMap<String, String>,
    
    /// Resource managers
    resource_managers: CrossDomainResourceManagers,
}

/// Structure holding references to resource managers
pub struct CrossDomainResourceManagers {
    /// Access manager
    pub access_manager: Arc<ResourceAccessManager>,
    
    /// Lifecycle manager
    pub lifecycle_manager: Arc<EffectResourceLifecycle>,
    
    /// Lock manager
    pub lock_manager: Arc<CrossDomainLockManager>,
    
    /// Dependency manager
    pub dependency_manager: Arc<ResourceDependencyManager>,
    
    /// Capability manager
    pub capability_manager: Arc<ResourceCapabilityManager>,
    
    /// Capability verifier
    pub capability_verifier: Arc<DefaultCapabilityVerifier>,
}

impl CrossDomainResourceTransferEffect {
    /// Create a new cross-domain resource transfer effect
    pub fn new(
        resource_id: ContentId,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        resource_managers: CrossDomainResourceManagers,
    ) -> Self {
        Self {
            id: EffectId::new(),
            resource_id,
            source_domain_id,
            target_domain_id,
            timeout: None,
            metadata: HashMap::new(),
            resource_managers,
        }
    }
    
    /// Set a timeout for the transfer operation
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Add metadata to the transfer
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Verify capabilities for the transfer
    async fn verify_capabilities(&self, context: &EffectContext) -> EffectResult<()> {
        // Check if the context has the necessary cross-domain capability
        if !context.has_cross_domain_capability(&CrossDomainCapability::TransferAssets) {
            return Err(EffectError::CapabilityError(
                "Missing TransferAssets capability".to_string()
            ));
        }
        
        // Check if the context has the necessary resource capabilities
        let capability_context = context.to_unified_capability_context();
        
        // Check access capability for source domain
        let has_source_access = self.resource_managers.capability_manager
            .check_access_capability(
                &self.resource_id,
                ResourceAccessType::Transfer,
                Some(&self.id),
                Some(&self.source_domain_id),
                &capability_context,
            )
            .await
            .map_err(|e| EffectError::CapabilityError(e.to_string()))?;
            
        if !has_source_access {
            return Err(EffectError::CapabilityError(
                format!("Missing transfer access capability for resource {} in source domain", self.resource_id)
            ));
        }
        
        // Check lifecycle capability for target domain
        let has_lifecycle_capability = self.resource_managers.capability_manager
            .check_lifecycle_capability(
                &self.resource_id,
                ResourceLifecycleCapability::Create,
                Some(&self.id),
                Some(&self.target_domain_id),
                &capability_context,
            )
            .await
            .map_err(|e| EffectError::CapabilityError(e.to_string()))?;
            
        if !has_lifecycle_capability {
            return Err(EffectError::CapabilityError(
                format!("Missing lifecycle capability for resource {} in target domain", self.resource_id)
            ));
        }
        
        Ok(())
    }
}

#[async_trait]
impl Effect for CrossDomainResourceTransferEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::CrossDomain
    }
    
    fn description(&self) -> String {
        format!(
            "Transfer resource {} from domain {} to domain {}",
            self.resource_id,
            self.source_domain_id,
            self.target_domain_id
        )
    }
    
    async fn validate(&self, context: &EffectContext) -> EffectResult<()> {
        // Verify capabilities
        self.verify_capabilities(context).await?;
        
        // Check if the resource is locked by another effect
        if self.resource_managers.access_manager.is_resource_locked(&self.resource_id) {
            return Err(EffectError::ResourceError(
                format!("Resource {} is locked and cannot be transferred", self.resource_id)
            ));
        }
        
        // Check if the resource exists in the source domain
        // This would typically involve a more complex check against a resource registry
        
        Ok(())
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Validate first
        self.validate(context).await?;
        
        // 1. Lock the resource in the source domain
        let lock_result = self.resource_managers.lock_manager.acquire_lock(
            &self.resource_id,
            CrossDomainLockType::Exclusive,
            &self.source_domain_id,
            &self.id,
            self.timeout,
            None
        );
        
        if lock_result != LockStatus::Acquired {
            return Err(EffectError::ResourceError(
                format!("Failed to lock resource {} in source domain: {:?}", self.resource_id, lock_result)
            ));
        }
        
        // 2. Record access in the source domain
        self.resource_managers.access_manager.record_access(
            &self.resource_id,
            ResourceAccessType::Transfer,
            Some(&self.id),
            Some(&self.source_domain_id)
        );
        
        // 3. Update resource lifecycle in source domain (mark as transferred)
        self.resource_managers.lifecycle_manager.prepare_resource_transfer(
            &self.resource_id,
            Some(&self.id),
            Some(&self.source_domain_id),
            Some(&self.target_domain_id)
        );
        
        // 4. Create resource in target domain
        self.resource_managers.lifecycle_manager.register_resource(
            &self.resource_id,
            Some(&self.id),
            Some(&self.target_domain_id)
        );
        
        // 5. Add domain dependency
        let dependency = ResourceDependency::new(
            self.resource_id.clone(),
            ContentId::from_string(format!("domain-{}", self.target_domain_id)),
            DependencyType::Strong,
            Some(self.source_domain_id.clone()),
            Some(self.target_domain_id.clone()),
            Some(self.id.clone()),
            self.metadata.clone()
        );
        
        self.resource_managers.dependency_manager.add_dependency(dependency)
            .map_err(|e| EffectError::ResourceError(format!("Failed to add dependency: {}", e)))?;
        
        // 6. Activate resource in target domain
        self.resource_managers.lifecycle_manager.activate_resource(
            &self.resource_id,
            Some(&self.id),
            Some(&self.target_domain_id)
        );
        
        // 7. Release lock in source domain
        self.resource_managers.lock_manager.release_lock(
            &self.resource_id,
            &self.id
        );
        
        // Build outcome
        let mut outcome = EffectOutcome::success(self.id.clone());
        
        // Add resource change
        outcome = outcome.with_resource_change(
            ResourceChange {
                resource_id: self.resource_id.clone(),
                change_type: ResourceChangeType::Transferred,
                domain_id: Some(self.target_domain_id.clone()),
                metadata: self.metadata.clone(),
            }
        );
        
        // Add metadata
        outcome = outcome.with_metadata("source_domain", self.source_domain_id.to_string());
        outcome = outcome.with_metadata("target_domain", self.target_domain_id.to_string());
        
        Ok(outcome)
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper function to create a cross-domain resource transfer effect
pub fn transfer_resource(
    resource_id: ContentId,
    source_domain_id: DomainId,
    target_domain_id: DomainId,
    resource_managers: CrossDomainResourceManagers,
) -> CrossDomainResourceTransferEffect {
    CrossDomainResourceTransferEffect::new(
        resource_id,
        source_domain_id,
        target_domain_id,
        resource_managers
    )
}

/// Effect for locking a resource across domains
pub struct CrossDomainResourceLockEffect {
    /// Effect ID
    id: EffectId,
    
    /// Resource ID to lock
    resource_id: ContentId,
    
    /// Lock type
    lock_type: CrossDomainLockType,
    
    /// Domain IDs involved
    domain_ids: Vec<DomainId>,
    
    /// Optional timeout for the lock operation
    timeout: Option<Duration>,
    
    /// Optional transaction ID
    transaction_id: Option<String>,
    
    /// Resource managers
    resource_managers: CrossDomainResourceManagers,
}

impl CrossDomainResourceLockEffect {
    /// Create a new cross-domain resource lock effect
    pub fn new(
        resource_id: ContentId,
        lock_type: CrossDomainLockType,
        domain_ids: Vec<DomainId>,
        resource_managers: CrossDomainResourceManagers,
    ) -> Self {
        Self {
            id: EffectId::new(),
            resource_id,
            lock_type,
            domain_ids,
            timeout: None,
            transaction_id: None,
            resource_managers,
        }
    }
    
    /// Set a timeout for the lock operation
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Set a transaction ID for the lock operation
    pub fn with_transaction(mut self, transaction_id: String) -> Self {
        self.transaction_id = Some(transaction_id);
        self
    }
    
    /// Verify capabilities for the lock operation
    async fn verify_capabilities(&self, context: &EffectContext) -> EffectResult<()> {
        // Check if the context has the necessary cross-domain capability
        let lock_type_str = format!("{:?}", self.lock_type);
        let resource_locking_cap = CrossDomainCapability::ResourceLocking {
            lock_type: lock_type_str.clone()
        };
        
        if !context.has_cross_domain_capability(&resource_locking_cap) {
            return Err(EffectError::CapabilityError(
                format!("Missing ResourceLocking capability for lock type: {}", lock_type_str)
            ));
        }
        
        // Check if the context has the necessary resource capabilities for each domain
        let capability_context = context.to_unified_capability_context();
        
        for domain_id in &self.domain_ids {
            // Check lock capability for this domain
            let has_lock_capability = self.resource_managers.capability_manager
                .check_lock_capability(
                    &self.resource_id,
                    self.lock_type.clone(),
                    Some(&self.id),
                    Some(domain_id),
                    &capability_context,
                )
                .await
                .map_err(|e| EffectError::CapabilityError(e.to_string()))?;
                
            if !has_lock_capability {
                return Err(EffectError::CapabilityError(
                    format!("Missing lock capability for resource {} in domain {}", self.resource_id, domain_id)
                ));
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl Effect for CrossDomainResourceLockEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::CrossDomain
    }
    
    fn description(&self) -> String {
        format!(
            "Lock resource {} with {:?} lock across {} domains",
            self.resource_id,
            self.lock_type,
            self.domain_ids.len()
        )
    }
    
    async fn validate(&self, context: &EffectContext) -> EffectResult<()> {
        // Verify capabilities
        self.verify_capabilities(context).await?;
        
        // Check if the resource can be locked
        for domain_id in &self.domain_ids {
            if !self.resource_managers.lock_manager.can_acquire_lock(
                &self.resource_id,
                self.lock_type.clone(),
                &self.id
            ) {
                return Err(EffectError::ResourceError(
                    format!("Resource {} cannot be locked in domain {}", self.resource_id, domain_id)
                ));
            }
        }
        
        Ok(())
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Validate first
        self.validate(context).await?;
        
        // Acquire locks in all domains
        let mut acquired_locks = Vec::new();
        
        for domain_id in &self.domain_ids {
            // Acquire lock
            let lock_result = self.resource_managers.lock_manager.acquire_lock(
                &self.resource_id,
                self.lock_type.clone(),
                domain_id,
                &self.id,
                self.timeout,
                self.transaction_id.clone()
            );
            
            if lock_result != LockStatus::Acquired {
                // Release any acquired locks
                for acquired_domain in &acquired_locks {
                    self.resource_managers.lock_manager.release_lock(
                        &self.resource_id,
                        &self.id
                    );
                }
                
                return Err(EffectError::ResourceError(
                    format!("Failed to lock resource {} in domain {}: {:?}", self.resource_id, domain_id, lock_result)
                ));
            }
            
            // Record lock in access manager
            self.resource_managers.access_manager.record_access(
                &self.resource_id,
                ResourceAccessType::Lock,
                Some(&self.id),
                Some(domain_id)
            );
            
            // Update resource lifecycle
            self.resource_managers.lifecycle_manager.lock_resource(
                &self.resource_id,
                Some(&self.id),
                Some(domain_id)
            );
            
            acquired_locks.push(domain_id.clone());
        }
        
        // Build outcome
        let mut outcome = EffectOutcome::success(self.id.clone());
        
        // Add resource change
        outcome = outcome.with_resource_change(
            ResourceChange {
                resource_id: self.resource_id.clone(),
                change_type: ResourceChangeType::Locked,
                domain_id: Some(self.domain_ids[0].clone()),
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert("lock_type".to_string(), format!("{:?}", self.lock_type));
                    if let Some(txn_id) = &self.transaction_id {
                        metadata.insert("transaction_id".to_string(), txn_id.clone());
                    }
                    metadata
                },
            }
        );
        
        // Add metadata
        outcome = outcome.with_metadata("lock_type", format!("{:?}", self.lock_type));
        outcome = outcome.with_metadata("domains", self.domain_ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(","));
        
        if let Some(txn_id) = &self.transaction_id {
            outcome = outcome.with_metadata("transaction_id", txn_id.clone());
        }
        
        Ok(outcome)
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper function to create a cross-domain resource lock effect
pub fn lock_resource_across_domains(
    resource_id: ContentId,
    lock_type: CrossDomainLockType,
    domain_ids: Vec<DomainId>,
    resource_managers: CrossDomainResourceManagers,
) -> CrossDomainResourceLockEffect {
    CrossDomainResourceLockEffect::new(
        resource_id,
        lock_type,
        domain_ids,
        resource_managers
    )
}

/// Effect for establishing a dependency between resources across domains
pub struct CrossDomainResourceDependencyEffect {
    /// Effect ID
    id: EffectId,
    
    /// Source resource ID
    source_id: ContentId,
    
    /// Source domain ID
    source_domain_id: DomainId,
    
    /// Target resource ID
    target_id: ContentId,
    
    /// Target domain ID
    target_domain_id: DomainId,
    
    /// Dependency type
    dependency_type: DependencyType,
    
    /// Optional metadata
    metadata: HashMap<String, String>,
    
    /// Resource managers
    resource_managers: CrossDomainResourceManagers,
}

impl CrossDomainResourceDependencyEffect {
    /// Create a new cross-domain resource dependency effect
    pub fn new(
        source_id: ContentId,
        source_domain_id: DomainId,
        target_id: ContentId,
        target_domain_id: DomainId,
        dependency_type: DependencyType,
        resource_managers: CrossDomainResourceManagers,
    ) -> Self {
        Self {
            id: EffectId::new(),
            source_id,
            source_domain_id,
            target_id,
            target_domain_id,
            dependency_type,
            metadata: HashMap::new(),
            resource_managers,
        }
    }
    
    /// Add metadata to the dependency
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Verify capabilities for the dependency operation
    async fn verify_capabilities(&self, context: &EffectContext) -> EffectResult<()> {
        // Check if the context has the necessary cross-domain capability
        let dep_type_str = format!("{:?}", self.dependency_type);
        let resource_dependency_cap = CrossDomainCapability::ResourceDependency {
            dependency_type: dep_type_str.clone()
        };
        
        if !context.has_cross_domain_capability(&resource_dependency_cap) {
            return Err(EffectError::CapabilityError(
                format!("Missing ResourceDependency capability for dependency type: {}", dep_type_str)
            ));
        }
        
        // Check if the context has the necessary resource capabilities
        let capability_context = context.to_unified_capability_context();
        
        // Check dependency capability for source domain
        let has_source_capability = self.resource_managers.capability_manager
            .check_dependency_capability(
                &self.source_id,
                self.dependency_type.clone(),
                Some(&self.id),
                Some(&self.source_domain_id),
                &capability_context,
            )
            .await
            .map_err(|e| EffectError::CapabilityError(e.to_string()))?;
            
        if !has_source_capability {
            return Err(EffectError::CapabilityError(
                format!("Missing dependency capability for source resource {} in domain {}", 
                    self.source_id, self.source_domain_id)
            ));
        }
        
        // Check dependency capability for target domain
        let has_target_capability = self.resource_managers.capability_manager
            .check_dependency_capability(
                &self.target_id,
                self.dependency_type.clone(),
                Some(&self.id),
                Some(&self.target_domain_id),
                &capability_context,
            )
            .await
            .map_err(|e| EffectError::CapabilityError(e.to_string()))?;
            
        if !has_target_capability {
            return Err(EffectError::CapabilityError(
                format!("Missing dependency capability for target resource {} in domain {}", 
                    self.target_id, self.target_domain_id)
            ));
        }
        
        Ok(())
    }
}

#[async_trait]
impl Effect for CrossDomainResourceDependencyEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::CrossDomain
    }
    
    fn description(&self) -> String {
        format!(
            "Establish {:?} dependency from resource {} in domain {} to resource {} in domain {}",
            self.dependency_type,
            self.source_id,
            self.source_domain_id,
            self.target_id,
            self.target_domain_id
        )
    }
    
    async fn validate(&self, context: &EffectContext) -> EffectResult<()> {
        // Verify capabilities
        self.verify_capabilities(context).await?;
        
        // Check if the resources exist
        // This would typically involve a more complex check against a resource registry
        
        // Check if a dependency already exists
        if self.resource_managers.dependency_manager.has_dependency(
            &self.source_id,
            &self.target_id
        ) {
            return Err(EffectError::ResourceError(
                format!("Dependency already exists from {} to {}", self.source_id, self.target_id)
            ));
        }
        
        Ok(())
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Validate first
        self.validate(context).await?;
        
        // Create the dependency
        let dependency = ResourceDependency::new(
            self.source_id.clone(),
            self.target_id.clone(),
            self.dependency_type.clone(),
            Some(self.source_domain_id.clone()),
            Some(self.target_domain_id.clone()),
            Some(self.id.clone()),
            self.metadata.clone()
        );
        
        self.resource_managers.dependency_manager.add_dependency(dependency)
            .map_err(|e| EffectError::ResourceError(format!("Failed to add dependency: {}", e)))?;
        
        // Record access for both resources
        self.resource_managers.access_manager.record_access(
            &self.source_id,
            ResourceAccessType::Read,
            Some(&self.id),
            Some(&self.source_domain_id)
        );
        
        self.resource_managers.access_manager.record_access(
            &self.target_id,
            ResourceAccessType::Read,
            Some(&self.id),
            Some(&self.target_domain_id)
        );
        
        // Build outcome
        let mut outcome = EffectOutcome::success(self.id.clone());
        
        // Add resource change
        outcome = outcome.with_resource_change(
            ResourceChange {
                resource_id: self.source_id.clone(),
                change_type: ResourceChangeType::DependencyAdded,
                domain_id: Some(self.source_domain_id.clone()),
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert("target_resource".to_string(), self.target_id.to_string());
                    metadata.insert("dependency_type".to_string(), format!("{:?}", self.dependency_type));
                    metadata.insert("target_domain".to_string(), self.target_domain_id.to_string());
                    metadata
                },
            }
        );
        
        // Add metadata
        outcome = outcome.with_metadata("source_domain", self.source_domain_id.to_string());
        outcome = outcome.with_metadata("target_domain", self.target_domain_id.to_string());
        outcome = outcome.with_metadata("dependency_type", format!("{:?}", self.dependency_type));
        
        Ok(outcome)
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper function to create a cross-domain resource dependency effect
pub fn add_cross_domain_dependency(
    source_id: ContentId,
    source_domain_id: DomainId,
    target_id: ContentId,
    target_domain_id: DomainId,
    dependency_type: DependencyType,
    resource_managers: CrossDomainResourceManagers,
) -> CrossDomainResourceDependencyEffect {
    CrossDomainResourceDependencyEffect::new(
        source_id,
        source_domain_id,
        target_id,
        target_domain_id,
        dependency_type,
        resource_managers
    )
}

/// Example demonstrating cross-domain resource management
pub async fn cross_domain_resource_example() -> Result<()> {
    println!("Cross-Domain Resource Management Example");
    
    // Create resource IDs
    let resource_id = ContentId::from_string("example-resource");
    let source_domain_id = DomainId::from_string("source-domain");
    let target_domain_id = DomainId::from_string("target-domain");
    
    // Create resource managers
    let access_manager = Arc::new(ResourceAccessManager::new());
    let lifecycle_manager = Arc::new(EffectResourceLifecycle::new());
    let lock_manager = Arc::new(CrossDomainLockManager::new());
    let dependency_manager = Arc::new(ResourceDependencyManager::new());
    
    // We'd need a proper capability manager in a real example
    // This is a placeholder to show the structure
    let capability_manager = Arc::new(
        ResourceCapabilityManager::new(
            Arc::new(UnifiedCapabilityManager::placeholder()),
            access_manager.clone(),
            lifecycle_manager.clone(),
            lock_manager.clone(),
            dependency_manager.clone()
        )
    );
    
    let capability_verifier = Arc::new(DefaultCapabilityVerifier::new());
    
    let resource_managers = CrossDomainResourceManagers {
        access_manager: access_manager.clone(),
        lifecycle_manager: lifecycle_manager.clone(),
        lock_manager: lock_manager.clone(),
        dependency_manager: dependency_manager.clone(),
        capability_manager: capability_manager.clone(),
        capability_verifier: capability_verifier.clone(),
    };
    
    // Create a mock effect context with necessary capabilities
    let mut context = EffectContext::new();
    context.add_cross_domain_capability(CrossDomainCapability::TransferAssets);
    context.add_cross_domain_capability(
        CrossDomainCapability::ResourceLocking { lock_type: "Exclusive".to_string() }
    );
    context.add_cross_domain_capability(
        CrossDomainCapability::ResourceDependency { dependency_type: "Strong".to_string() }
    );
    context.add_effect_capability(EffectCapability::CreateResource);
    context.add_effect_capability(EffectCapability::ReadResource);
    context.add_effect_capability(EffectCapability::UpdateResource);
    
    // 1. Register resource in source domain
    println!("1. Registering resource in source domain");
    lifecycle_manager.register_resource(&resource_id, None, Some(&source_domain_id));
    
    // 2. Activate resource in source domain
    println!("2. Activating resource in source domain");
    lifecycle_manager.activate_resource(&resource_id, None, Some(&source_domain_id));
    
    // 3. Create transfer effect
    println!("3. Creating transfer effect");
    let transfer_effect = transfer_resource(
        resource_id.clone(),
        source_domain_id.clone(),
        target_domain_id.clone(),
        resource_managers.clone()
    );
    
    // 4. Execute transfer effect
    println!("4. Executing transfer effect");
    let transfer_result = transfer_effect.execute(&context).await;
    match transfer_result {
        Ok(outcome) => {
            println!("Transfer successful: {}", outcome.success);
            println!("Resource changes: {:?}", outcome.resource_changes);
        },
        Err(e) => {
            println!("Transfer failed: {}", e);
            return Err(Error::from_string(format!("Transfer failed: {}", e)));
        }
    }
    
    // 5. Create dependency effect
    println!("5. Creating dependency effect");
    let dependency_effect = add_cross_domain_dependency(
        resource_id.clone(),
        target_domain_id.clone(),
        ContentId::from_string("metadata-resource"),
        source_domain_id.clone(),
        DependencyType::Data,
        resource_managers.clone()
    );
    
    // 6. Execute dependency effect
    println!("6. Executing dependency effect");
    let dependency_result = dependency_effect.execute(&context).await;
    match dependency_result {
        Ok(outcome) => {
            println!("Dependency creation successful: {}", outcome.success);
            println!("Resource changes: {:?}", outcome.resource_changes);
        },
        Err(e) => {
            println!("Dependency creation failed: {}", e);
            return Err(Error::from_string(format!("Dependency creation failed: {}", e)));
        }
    }
    
    // 7. Create lock effect
    println!("7. Creating lock effect");
    let lock_effect = lock_resource_across_domains(
        resource_id.clone(),
        CrossDomainLockType::Exclusive,
        vec![source_domain_id.clone(), target_domain_id.clone()],
        resource_managers.clone()
    );
    
    // 8. Execute lock effect
    println!("8. Executing lock effect");
    let lock_result = lock_effect.execute(&context).await;
    match lock_result {
        Ok(outcome) => {
            println!("Lock successful: {}", outcome.success);
            println!("Resource changes: {:?}", outcome.resource_changes);
        },
        Err(e) => {
            println!("Lock failed: {}", e);
            return Err(Error::from_string(format!("Lock failed: {}", e)));
        }
    }
    
    // 9. Check final resource state
    println!("9. Checking final resource state");
    let is_locked = lock_manager.is_resource_locked(&resource_id);
    println!("Resource is locked: {}", is_locked);
    
    let dependencies = dependency_manager.get_dependencies_for_source(&resource_id);
    println!("Resource has {} dependencies", dependencies.len());
    
    println!("Cross-domain resource management example completed successfully");
    Ok(())
} 