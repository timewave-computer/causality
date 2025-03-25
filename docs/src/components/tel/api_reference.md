<!-- API reference for TEL -->
<!-- Original file: docs/src/tel_api_reference.md -->

# TEL API Reference

This document provides a comprehensive reference for the Temporal Effect Language (TEL) API.

## Core Components

TEL is composed of several key components that work together to provide a robust system for managing resources with temporal effects.

### Resource Management

#### `ResourceId`

A unique identifier for a resource in the system.

```rust
let resource_id = ResourceId::new();
```

#### `ResourceManager`

The central component responsible for managing resources. It provides methods for creating, updating, and deleting resources.

```rust
let resource_manager = ResourceManager::new();
let resource_id = resource_manager.create_resource(&owner, &domain, initial_data)?;
resource_manager.update_resource(&resource_id, new_data)?;
resource_manager.delete_resource(&resource_id)?;
```

#### `ResourceOperation`

Represents operations that can be performed on resources, such as create, update, delete, transfer, lock, and unlock.

```rust
let operation = ResourceOperation::new(
    ResourceOperationType::Create {
        owner: Address::random(),
        domain: Domain::new("test"),
        initial_data: RegisterContents::Text("Hello, World!".to_string()),
    }
);
```

### Version Control

#### `VersionManager`

Tracks versions of resources over time, allowing for history browsing and rollbacks.

```rust
let version_manager = VersionManager::new();
let version_id = version_manager.create_version(resource_id, data)?;
let history = version_manager.get_history(resource_id)?;
version_manager.rollback(resource_id, version_id)?;
```

### Snapshot System

#### `SnapshotManager`

Generates point-in-time snapshots of the system, useful for backup and restoration.

```rust
let snapshot_manager = SnapshotManager::new(
    Arc::clone(&resource_manager),
    snapshot_storage,
    config
);
let snapshot_id = snapshot_manager.create_snapshot()?;
snapshot_manager.restore_snapshot(snapshot_id)?;
```

### Effect System

#### `ResourceEffect`

Represents an effect that can be applied to resources, with optional proof verification.

```rust
let effect = ResourceEffect::new(operation)
    .requires_verification(true);
```

#### `ResourceEffectAdapter`

Applies effects to resources by translating them into resource operations.

```rust
let adapter = ResourceEffectAdapter::new(Arc::clone(&resource_manager));
let result = adapter.apply(effect)?;
```

#### `EffectComposer`

Composes multiple effects into a single composite effect for batch processing.

```rust
let mut composer = EffectComposer::new();
composer.add_effect(effect1);
composer.add_effect(effect2);
composer.with_condition(should_include, |c| {
    c.add_effect(conditional_effect);
});
```

#### `RepeatingEffect`

Creates effects that repeat according to a specified schedule.

```rust
// Repeat an effect 5 times
let repeating = RepeatingEffect::repeat_count(effect, 5);

// Repeat an effect every 10 seconds
let repeating = RepeatingEffect::repeat_interval(effect, Duration::from_secs(10));

// Repeat an effect until a specific time
let repeating = RepeatingEffect::repeat_until(effect, end_time);

// Repeat an effect indefinitely (with max iterations safeguard)
let repeating = RepeatingEffect::repeat_indefinitely(effect);
```

### Proof System

#### `EffectProofGenerator`

Generates cryptographic proofs for effects, which can later be verified.

```rust
let generator = EffectProofGenerator::new(
    EffectProofFormat::Groth16,
    Address::random()
);
let proof = generator.generate_proof(&effect, None)?;
```

#### `EffectProofVerifier`

Verifies proofs attached to effects to ensure their validity.

```rust
let verifier = EffectProofVerifier::default();
let is_valid = verifier.verify_proof(&effect, &proof)?;
```

### Builder System

#### `TelBuilder`

A builder pattern implementation for constructing and configuring a TEL system.

```rust
let tel = TelBuilder::new()
    .with_snapshot_storage(storage)
    .with_verifier(verifier)
    .with_instance_id("my-tel-instance")
    .build();
```

## Common Types

### `Address`

Represents an entity address in the system, such as a resource owner.

```rust
let addr = Address::random();
let addr_from_string = Address::from_string("0x1234...")?;
```

### `Domain`

Represents a domain within which resources exist, providing isolation between different applications or contexts.

```rust
let domain = Domain::new("finance");
```

### `RegisterContents`

The data stored within a resource register, supporting various data types.

```rust
// Store text
let contents = RegisterContents::Text("Hello, World!".to_string());

// Store a number
let contents = RegisterContents::Number(42);

// Store binary data
let contents = RegisterContents::Binary(vec![1, 2, 3, 4]);

// Store a resource ID (reference to another resource)
let contents = RegisterContents::ResourceId(other_resource_id);
```

## Error Handling

The TEL system uses a custom error type `TelError` and result type `TelResult` for operations that may fail.

```rust
fn my_function() -> TelResult<ResourceId> {
    // Implementation
    Ok(resource_id)
}

// Handle errors
match my_function() {
    Ok(resource_id) => println!("Success: {:?}", resource_id),
    Err(err) => match err {
        TelError::ResourceNotFound(id) => println!("Resource not found: {:?}", id),
        TelError::PermissionDenied(msg) => println!("Permission denied: {}", msg),
        // Handle other error types
        _ => println!("An error occurred: {:?}", err),
    }
}
``` 