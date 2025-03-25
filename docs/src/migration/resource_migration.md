# Resource Management System Migration Guide

This guide helps you migrate from the legacy resource management system to the new unified resource management system.

## Overview of Changes

The resource management system has been refactored to provide:

1. A unified trait-based interface across all components
2. Better cross-domain support
3. Improved performance and error handling
4. Cleaner integration with the effects system

The legacy implementations in the `causality-resource` crate are now deprecated and will be removed in a future version. You should migrate to the new implementations in `causality-effects`.

## Migration Steps

### Step 1: Update Dependencies

Ensure your `Cargo.toml` includes the necessary dependencies:

```toml
[dependencies]
causality-effects = { path = "../causality-effects", features = ["resource"] }
```

### Step 2: Replace Legacy Imports

Replace your legacy imports with the new trait-based imports:

#### Legacy (Deprecated):
```rust
use causality_resource::{
    ResourceState,
    ResourceLifecycle,
    ResourceAccessType,
    ResourceAccessManager,
    LockType,
    ResourceLockManager,
    DependencyType,
    ResourceDependencyManager,
};
```

#### New Approach:
```rust
use causality_effects::resource::{
    lifecycle::{ResourceLifecycle, ResourceState},
    access::{ResourceAccess, ResourceAccessType},
    locking::{ResourceLocking, LockType},
    dependency::{ResourceDependency, DependencyType},
    implementation::EffectResourceImplementation,
    context::BasicResourceContext,
};
```

### Step 3: Adapt Resource Lifecycle Management

#### Legacy (Deprecated):
```rust
let lifecycle = ResourceLifecycle::new();

// Register a resource
lifecycle.register_resource(resource_id, ResourceState::Created)?;

// Update state
lifecycle.update_resource_state(&resource_id, ResourceState::Active)?;

// Check state
let state = lifecycle.get_resource_state(&resource_id)?;
```

#### New Approach:
```rust
let resource_impl = create_resource_implementation();
let context = create_resource_context(effect_id, None);

// Register a resource
resource_impl.register_resource(resource_id, ResourceState::Created, &context).await?;

// Update state
resource_impl.update_resource_state(&resource_id, ResourceState::Active, &context).await?;

// Check state
let state = resource_impl.get_resource_state(&resource_id).await?;
```

### Step 4: Adapt Resource Access Management

#### Legacy (Deprecated):
```rust
let access_manager = ResourceAccessManager::new();

// Record access
let access = ResourceAccess {
    resource_id: resource_id.clone(),
    access_type: ResourceAccessType::Read,
    accessor_id: "effect-123".to_string(),
    timestamp: std::time::SystemTime::now(),
};
access_manager.record_access(access)?;

// Check access
let is_locked = access_manager.is_resource_locked(&resource_id);
```

#### New Approach:
```rust
let resource_impl = create_resource_implementation();
let context = create_resource_context(effect_id, None);

// Record access
resource_impl.record_access(
    &resource_id,
    ResourceAccessType::Read,
    &context
).await?;

// Check access
let is_allowed = resource_impl.is_access_allowed(
    &resource_id,
    ResourceAccessType::Read,
    &context
).await?;
```

### Step 5: Adapt Resource Locking

#### Legacy (Deprecated):
```rust
let lock_manager = ResourceLockManager::new();

// Acquire lock
lock_manager.acquire_lock(
    &resource_id,
    LockType::Exclusive,
    &holder_id,
    None,
    None
)?;

// Release lock
lock_manager.release_lock(&resource_id, &holder_id)?;
```

#### New Approach:
```rust
let resource_impl = create_resource_implementation();
let context = create_resource_context(effect_id, None);

// Acquire lock
resource_impl.acquire_lock(
    &resource_id,
    LockType::Exclusive,
    &holder_id,
    None,
    &context
).await?;

// Release lock
resource_impl.release_lock(
    &resource_id,
    &holder_id,
    &context
).await?;
```

### Step 6: Adapt Resource Dependencies

#### Legacy (Deprecated):
```rust
let dependency_manager = ResourceDependencyManager::new();

// Add dependency
let dependency = ResourceDependency {
    source_id: source_id.clone(),
    target_id: target_id.clone(),
    dependency_type: DependencyType::Strong,
    domain_ids: None,
    creator_effect_id: None,
    metadata: HashMap::new(),
};
dependency_manager.add_dependency(dependency)?;

// Remove dependency
dependency_manager.remove_dependency(
    &source_id,
    &target_id,
    DependencyType::Strong
)?;
```

#### New Approach:
```rust
let resource_impl = create_resource_implementation();
let context = create_resource_context(effect_id, None);

// Add dependency
resource_impl.add_dependency(
    &source_id,
    &target_id,
    DependencyType::Strong,
    &context
).await?;

// Remove dependency
resource_impl.remove_dependency(
    &source_id,
    &target_id,
    DependencyType::Strong,
    &context
).await?;
```

### Step 7: Create Resource Implementation

The new approach uses a unified implementation that combines all resource management functionality:

```rust
use std::sync::Arc;
use causality_effects::{
    effect::EffectRegistry,
    resource::{
        access::ResourceAccessManager,
        lifecycle::EffectResourceLifecycle,
        locking::CrossDomainLockManager,
        dependency::ResourceDependencyManager,
        implementation::EffectResourceImplementation,
    },
};

fn create_resource_implementation() -> EffectResourceImplementation {
    // Create the components
    let effect_registry = Arc::new(EffectRegistry::new());
    let access_manager = Arc::new(ResourceAccessManager::new());
    let lifecycle_manager = Arc::new(EffectResourceLifecycle::new());
    let lock_manager = Arc::new(CrossDomainLockManager::new());
    let dependency_manager = Arc::new(ResourceDependencyManager::new());
    
    // Create the unified implementation
    EffectResourceImplementation::new(
        effect_registry,
        access_manager,
        lifecycle_manager,
        lock_manager,
        dependency_manager
    )
}
```

### Step 8: Create Resource Context

The new approach uses a context object to track authorization and effect information:

```rust
use causality_effects::{
    effect::EffectId,
    resource::implementation::create_effect_context,
};

fn create_resource_context(
    effect_id: EffectId,
    domain_id: Option<ContentId>
) -> BasicResourceContext {
    create_effect_context(effect_id, domain_id)
}
```

## Using Feature Flags for Gradual Migration

You can use feature flags to control the deprecation level during migration:

```toml
[dependencies]
causality-resource = { path = "../causality-resource", features = ["allow-deprecated"] }
```

Available flags:
- `allow-deprecated`: Suppresses all deprecation warnings
- `deprecation-error`: Turns deprecation warnings into errors (for CI)
- `suppress-deprecation-warnings`: Suppresses runtime deprecation warnings

## Common Migration Challenges

### Async vs. Sync API

The new API is fully asynchronous, requiring `async`/`await` syntax. Make sure to update your code accordingly.

### Context Object

The new API requires a context object for most operations. This provides better tracking and authorization.

### Capability Checking

The new API integrates with the capability system. Make sure your effects have the appropriate capabilities.

### Error Handling

Error types have changed. Update your error handling to work with the new error types.

## Testing Your Migration

Run tests with the `deprecation-error` feature to ensure you've migrated all code:

```bash
cargo test --features deprecation-error
```

## Getting Help

If you encounter issues during migration, refer to:
- Example code in `causality-effects/examples/resource_examples.rs`
- Documentation in `docs/src/concepts/resources/`
- The API documentation for the `causality-effects` crate

## Timeline

- **0.2.0**: Deprecation warnings for legacy code
- **0.3.0**: Legacy code marked with error-level deprecation
- **0.4.0**: Legacy code removed

We recommend migrating as soon as possible to benefit from the improved implementation. 