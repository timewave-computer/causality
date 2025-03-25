// Tests for resource management integration
//
// This file contains tests for the resource management integration,
// including access control, lifecycle management, locking, capabilities,
// and dependency tracking.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::resource::access::{ResourceAccessType, ResourceAccessManager};
    use crate::resource::lifecycle::{ResourceLifecycleEvent, EffectResourceLifecycle};
    use crate::resource::locking::{CrossDomainLockManager, CrossDomainLockType, LockStatus};
    use crate::resource::dependency::{ResourceDependencyManager, DependencyType};
    use crate::resource::capability::{ResourceCapabilityManager, ResourceCapability, ResourceLifecycleCapability};
    use crate::capability::{UnifiedCapabilityManager, UnifiedCapabilityContext};
    use causality_types::ContentId;
    use crate::effect_id::EffectId;
    use causality_domain::domain::DomainId;
    
    // Mock capability manager for testing
    struct MockCapabilityManager;
    
    impl MockCapabilityManager {
        fn new() -> Arc<UnifiedCapabilityManager> {
            // In a real test, this would return a proper mock
            // For now, we'll just use a type assertion to make the compiler happy
            unimplemented!("MockCapabilityManager not implemented for tests")
        }
    }
    
    // Test resource access patterns
    #[test]
    fn test_resource_access() {
        let resource_id = ContentId::from_string("resource-1");
        let effect_id = EffectId::from_string("effect-1");
        let domain_id = DomainId::from_string("domain-1");
        
        let access_manager = ResourceAccessManager::new();
        
        // Test recording access
        access_manager.record_access(
            &resource_id,
            ResourceAccessType::Read,
            Some(&effect_id),
            Some(&domain_id)
        );
        
        // Test checking access
        let accesses = access_manager.get_resource_accesses(&resource_id);
        assert_eq!(accesses.len(), 1);
        assert_eq!(accesses[0].access_type, ResourceAccessType::Read);
        assert_eq!(accesses[0].effect_id, Some(effect_id.clone()));
        assert_eq!(accesses[0].domain_id, Some(domain_id.clone()));
        
        // Test resource is not locked
        assert!(!access_manager.is_resource_locked(&resource_id));
        
        // Lock resource
        access_manager.record_access(
            &resource_id,
            ResourceAccessType::Lock,
            Some(&effect_id),
            Some(&domain_id)
        );
        
        // Test resource is locked
        assert!(access_manager.is_resource_locked(&resource_id));
    }
    
    // Test resource lifecycle management
    #[test]
    fn test_resource_lifecycle() {
        let resource_id = ContentId::from_string("resource-2");
        let effect_id = EffectId::from_string("effect-2");
        let domain_id = DomainId::from_string("domain-2");
        
        let lifecycle_manager = EffectResourceLifecycle::new();
        
        // Register resource
        lifecycle_manager.register_resource(&resource_id, Some(&effect_id), Some(&domain_id));
        
        // Test resource activation
        lifecycle_manager.activate_resource(&resource_id, Some(&effect_id), Some(&domain_id));
        
        let state = lifecycle_manager.get_resource_state(&resource_id);
        assert!(state.is_some());
        assert!(state.unwrap().is_active);
        
        // Test consuming resource
        lifecycle_manager.consume_resource(&resource_id, Some(&effect_id), Some(&domain_id));
        
        let state = lifecycle_manager.get_resource_state(&resource_id);
        assert!(state.is_some());
        assert!(state.unwrap().is_consumed);
    }
    
    // Test resource locking
    #[test]
    fn test_resource_locking() {
        let resource_id = ContentId::from_string("resource-3");
        let effect_id = EffectId::from_string("effect-3");
        let domain_id = DomainId::from_string("domain-3");
        
        let lock_manager = CrossDomainLockManager::new();
        
        // Test acquiring lock
        let lock_result = lock_manager.acquire_lock(
            &resource_id,
            CrossDomainLockType::Exclusive,
            &domain_id,
            &effect_id,
            None,
            None
        );
        
        assert_eq!(lock_result, LockStatus::Acquired);
        
        // Test lock is held
        assert!(lock_manager.is_resource_locked(&resource_id));
        
        // Test acquiring same lock again
        let lock_result = lock_manager.acquire_lock(
            &resource_id,
            CrossDomainLockType::Exclusive,
            &domain_id,
            &effect_id,
            None,
            None
        );
        
        assert_eq!(lock_result, LockStatus::AlreadyHeld);
        
        // Test releasing lock
        lock_manager.release_lock(&resource_id, &effect_id);
        
        // Test lock is released
        assert!(!lock_manager.is_resource_locked(&resource_id));
    }
    
    // Test resource dependencies
    #[test]
    fn test_resource_dependencies() {
        let source_id = ContentId::from_string("resource-source");
        let target_id = ContentId::from_string("resource-target");
        let effect_id = EffectId::from_string("effect-4");
        let domain_id = DomainId::from_string("domain-4");
        
        let dependency_manager = ResourceDependencyManager::new();
        
        // Add dependency
        dependency_manager.add_dependency(
            &source_id,
            &target_id,
            DependencyType::Strong,
            Some(&domain_id),
            Some(&effect_id),
            None
        );
        
        // Test getting dependencies
        let deps = dependency_manager.get_dependencies_for_source(&source_id);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].source_id, source_id);
        assert_eq!(deps[0].target_id, target_id);
        assert_eq!(deps[0].dependency_type, DependencyType::Strong);
        
        // Test has dependency
        assert!(dependency_manager.has_dependency(&source_id, &target_id));
        
        // Test removing dependency
        dependency_manager.remove_dependency(&source_id, &target_id);
        
        // Test dependency is removed
        assert!(!dependency_manager.has_dependency(&source_id, &target_id));
    }
} 