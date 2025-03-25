# Resource Management System Migration Guide

This document provides guidance for migrating code from the legacy resource management implementations to the new unified resource system.

## Overview of Changes

The Causality resource management system has been refactored to provide a cleaner, more unified approach that works seamlessly across domain and effect boundaries. The key changes include:

1. **Unified Interface Traits**: Core resource management functionality is now defined as traits in `causality-resource::interface`
2. **Implementation Separation**: Concrete implementations are provided in both the `causality-effects` and `causality-domain` crates
3. **Cross-Boundary Adapters**: Adapters enable seamless resource management across domain and effect boundaries
4. **Deprecation of Duplicates**: Redundant implementations are marked as deprecated and will be removed in future versions

## Migration Steps

### Step 1: Update Imports

Replace imports from deprecated modules with imports from the new unified system:

**Before:**
```rust
use causality_resource::manager::ResourceManager;
use causality_resource::registry::ResourceRegistry;
```

**After:**
```rust
// If working in the effect system:
use causality_effects::resource::implementation::EffectResourceImplementation;
// OR if working in the domain system:
use causality_domain::resource_impl::DomainResourceImplementation;

// For interfaces (regardless of context):
use causality_resource::interface::{
    ResourceAccess, ResourceLifecycle, ResourceLocking, ResourceDependency
};
```

### Step 2: Use Trait-Based Access

Update your code to use trait-based access patterns instead of concrete types:

**Before:**
```rust
let manager = ResourceManager::new();
manager.register_resource(resource_id, initial_state)?;
```

**After:**
```rust
// Accept any type that implements ResourceLifecycle
fn register_my_resource(
    lifecycle_manager: &dyn ResourceLifecycle,
    resource_id: ContentId,
    context: &dyn ResourceContext
) -> Result<()> {
    lifecycle_manager.register_resource(
        resource_id,
        ResourceState::Created,
        context
    ).await
}
```

### Step 3: Create Appropriate Context

The new system requires a `ResourceContext` for most operations:

```rust
use causality_resource::interface::BasicResourceContext;

// For effect-based operations
let context = causality_effects::resource::implementation::create_effect_context(
    effect_id,
    Some(context_id)
);

// For domain-based operations
let context = causality_domain::resource_impl::create_domain_context(
    domain_id,
    Some(context_id)
);

// Generic context creation
let context = BasicResourceContext::new(context_id)
    .with_domain(domain_id)  // Optional
    .with_effect(effect_id); // Optional
```

### Step 4: Access Implementation Through Interfaces

Always access implementations through their trait interfaces:

**Before:**
```rust
let lock_result = resource_manager.lock_resource(
    resource_id, 
    holder_id, 
    lock_type
)?;
```

**After:**
```rust
let lock_status = resource_locking.acquire_lock(
    &resource_id,
    lock_type,
    &holder_id,
    Some(Duration::from_secs(10)),
    &context
).await?;
```

### Step 5: Cross-Domain Operations

For operations that span multiple domains:

```rust
use causality_effects::resource::effects::{
    transfer_resource,
    lock_resource_across_domains,
    add_cross_domain_dependency
};

// Transfer a resource between domains
let result = transfer_resource(
    resource_id,
    source_domain_id,
    target_domain_id,
    resource_managers,
    context
).await?;

// Lock a resource across multiple domains
let lock_result = lock_resource_across_domains(
    resource_id,
    lock_type,
    holder_id,
    vec![domain_id1, domain_id2],
    Some(timeout),
    resource_managers,
    context
).await?;
```

## Common Migration Patterns

### Resource Access Control Migration

**Before:**
```rust
let access_tracker = ResourceAccessTracker::new();
access_tracker.record_access(ResourceAccess {
    resource_id: resource_id.clone(),
    access_type: ResourceAccessType::Read,
    // other fields...
})?;
```

**After:**
```rust
let access_manager: Arc<dyn ResourceAccess> = get_resource_access_implementation();
access_manager.record_access(
    &resource_id,
    ResourceAccessType::Read,
    &context
).await?;
```

### Resource Lifecycle Migration

**Before:**
```rust
let lifecycle_manager = ResourceLifecycleManager::new();
lifecycle_manager.create_resource(resource_id.clone())?;
lifecycle_manager.activate_resource(resource_id.clone())?;
```

**After:**
```rust
let lifecycle_manager: Arc<dyn ResourceLifecycle> = get_resource_lifecycle_implementation();
lifecycle_manager.register_resource(
    resource_id.clone(),
    ResourceState::Created,
    &context
).await?;
lifecycle_manager.update_resource_state(
    &resource_id,
    ResourceState::Active,
    &context
).await?;
```

### Resource Locking Migration

**Before:**
```rust
let lock_manager = LockManager::new();
let lock = lock_manager.acquire_lock(
    resource_id.clone(),
    holder_id.clone(),
    LockType::Exclusive
)?;
```

**After:**
```rust
let lock_manager: Arc<dyn ResourceLocking> = get_resource_locking_implementation();
let lock_status = lock_manager.acquire_lock(
    &resource_id,
    LockType::Exclusive,
    &holder_id,
    None, // No timeout
    &context
).await?;
```

### Resource Dependency Migration

**Before:**
```rust
let dependency_manager = DependencyManager::new();
dependency_manager.add_dependency(
    source_id.clone(),
    target_id.clone(),
    DependencyType::Strong
)?;
```

**After:**
```rust
let dependency_manager: Arc<dyn ResourceDependency> = get_resource_dependency_implementation();
dependency_manager.add_dependency(
    &source_id,
    &target_id,
    DependencyType::Strong,
    &context
).await?;
```

## Choosing the Right Implementation

Choose the appropriate implementation based on your context:

1. **For Effect System Code**:
   - Use `EffectResourceImplementation` from the `causality-effects` crate
   - Access through `ResourceAccess`, `ResourceLifecycle`, etc. interfaces

2. **For Domain System Code**:
   - Use `DomainResourceImplementation` from the `causality-domain` crate
   - Access through the same interface traits

3. **For Cross-Boundary Code**:
   - Use the adapter patterns: `DomainToEffectResourceAdapter` or `EffectToDomainResourceAdapter`
   - These adapters implement all the resource traits and handle context conversion

## Handling Deprecation Warnings

If you see deprecation warnings:

1. Identify which deprecated component you're using
2. Find the corresponding interface in `causality_resource::interface`
3. Find an implementation in either `causality_effects` or `causality_domain`
4. Follow the migration steps above

## Timeline for Deprecation

- **Version 0.2.0**: Deprecated modules are marked with warnings
- **Version 0.3.0**: Deprecated modules will emit compiler errors
- **Version 0.4.0**: Deprecated modules will be removed entirely

## Testing Your Migration

Run the included deprecation detection script to identify any remaining uses of deprecated code:

```bash
./scripts/find_deprecated_resource_usage.sh
```

## Example: Complete Migration

Here's a complete example of migrating a function that manages resources:

**Before:**
```rust
use causality_resource::manager::ResourceManager;
use causality_resource::registry::ResourceRegistry;

fn manage_resource(
    resource_id: ContentId,
    registry: &ResourceRegistry,
) -> Result<()> {
    let manager = ResourceManager::new(registry.clone());
    
    // Register and activate
    manager.register_resource(resource_id.clone(), ResourceState::Created)?;
    manager.update_resource_state(resource_id.clone(), ResourceState::Active)?;
    
    // Lock the resource
    let lock = manager.lock_resource(
        resource_id.clone(),
        holder_id.clone(),
        LockType::Exclusive
    )?;
    
    // Do something with the locked resource
    
    // Release the lock
    manager.unlock_resource(resource_id.clone(), holder_id.clone())?;
    
    Ok(())
}
```

**After:**
```rust
use causality_resource::interface::{
    ResourceLifecycle, ResourceLocking, ResourceState, 
    LockType, ResourceContext, BasicResourceContext
};
use causality_effects::resource::implementation::EffectResourceImplementation;

async fn manage_resource(
    resource_id: ContentId,
    effect_id: EffectId,
    resource_impl: &EffectResourceImplementation,
) -> Result<()> {
    // Create context
    let context = BasicResourceContext::new(ContentId::from_string("operation-context")?)
        .with_effect(effect_id);
    
    // Register and activate as separate steps
    resource_impl.register_resource(
        resource_id.clone(),
        ResourceState::Created,
        &context
    ).await?;
    
    resource_impl.update_resource_state(
        &resource_id,
        ResourceState::Active,
        &context
    ).await?;
    
    // Lock the resource
    let holder_id = ContentId::from_string("holder")?;
    let lock_status = resource_impl.acquire_lock(
        &resource_id,
        LockType::Exclusive,
        &holder_id,
        None, // No timeout
        &context
    ).await?;
    
    // Do something with the locked resource
    
    // Release the lock
    resource_impl.release_lock(
        &resource_id,
        &holder_id,
        &context
    ).await?;
    
    Ok(())
}
```

## Getting Help

If you encounter difficulties with the migration:

1. Check the examples in `causality-effects/src/resource/examples/`
2. Review the cross-domain examples in `causality-effects/src/resource/effects.rs`
3. See the implementation adapter pattern in `causality-resource/src/adapter.rs` 