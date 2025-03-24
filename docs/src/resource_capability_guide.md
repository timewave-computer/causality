# Resource Capability System Guide

The Resource Capability System in Causality provides a secure, capability-based approach to resource access control. This guide explains how to use the system to manage access to resources in a secure and composable way.

## Core Concepts

### Resources

Resources are discrete units of data or functionality that can be accessed and manipulated through the ResourceAPI. Each resource:
- Has a unique ID
- Belongs to a specific resource type
- Has an owner
- Contains data and metadata
- Has a state (Active, Locked, Archived, etc.)

### Capabilities

Capabilities are unforgeable access tokens that grant specific rights to holders for accessing resources. Key properties:
- Capabilities can only be created by resource owners or through delegation
- They grant specific rights (Read, Write, Delete, Transfer, Delegate)
- They can be revoked by issuers
- They can be delegated to other users with the same or reduced rights
- They can be composed to create new capabilities

## Using the Resource API

### Creating the API

```rust
// Create addresses for users
let admin_address = Address::from("admin:0x1234");
let user_address = Address::from("user:alice");

// Create a memory-backed implementation of ResourceAPI
let api = MemoryResourceAPI::new(admin_address.clone());

// Get the root capability (admin access)
let root_cap = api.root_capability();
```

### Creating Resources

```rust
// Create a resource with data and metadata
let (resource_id, owner_capability) = api.create_resource(
    &root_cap,                 // Capability for creating resources
    "document",                // Resource type
    &user_address,             // Owner address
    data,                      // Resource data (Vec<u8>)
    Some(metadata),            // Optional metadata (HashMap<String, String>)
).await?;
```

### Reading Resources

```rust
// Get a resource using a capability
let resource = api.get_resource(&capability, &resource_id).await?;

// Read the resource data and metadata
let data = resource.data();
let metadata = resource.metadata();
let state = resource.state();
```

### Updating Resources

```rust
// Get a mutable resource
let resource = api.get_resource_mut(&capability, &resource_id).await?;

// Update the resource data
api.update_resource(
    &capability,
    &resource_id,
    Some(new_data),
    Some(options),  // Optional update options
).await?;
```

### Delegating Access

```rust
// Create a capability for another user with limited rights
let new_capability = api.create_capability(
    &owner_capability,     // Issuer's capability
    &resource_id,
    vec![Right::Read],     // Only grant read rights
    &other_user_address,
).await?;
```

### Revoking Access

```rust
// Revoke a previously issued capability
api.revoke_capability(
    &owner_capability,
    &capability_to_revoke.id(),
).await?;
```

### Querying Resources

```rust
// Create a query to find resources by type, owner, or metadata
let query = ResourceQuery {
    resource_type: Some("document".to_string()),
    owner: Some(user_address.clone()),
    domain: None,
    metadata: HashMap::new(),
    sort_by: Some("created_at".to_string()),
    ascending: false,
    offset: Some(0),
    limit: Some(10),
};

// Find resources matching the query
let resources = api.find_resources(&capability, query).await?;
```

## Advanced Capability Features

### Delegating Capabilities

```rust
// Delegate a capability with reduced rights
let delegated_capability = api.delegate_capability(
    &capability,
    &resource_id,
    vec![Right::Read],  // Only delegate read rights
    &new_holder_address,
).await?;
```

### Composing Capabilities

```rust
// Compose multiple capabilities to create a new one
let composed_capability = api.compose_capabilities(
    &[capability1, capability2],
    &new_holder_address,
).await?;
```

## Best Practices

1. **Principle of Least Privilege**: Only grant the minimal rights needed for a particular operation.

2. **Capability Revocation**: Revoke capabilities as soon as they're no longer needed to minimize the attack surface.

3. **Capability Delegation**: Use delegation to create fine-grained capabilities for specific operations.

4. **Capability Composition**: Use composition to combine multiple capabilities for complex operations.

5. **Resource State Management**: Use resource state transitions (Active -> Locked -> Archived) to manage the lifecycle of resources.

## Example Workflows

### Document Sharing Workflow

1. Owner creates a document resource
2. Owner delegates read-only capability to a specific user
3. User reads the document using their capability
4. Owner revokes the capability when sharing is no longer needed

### Collaborative Editing Workflow

1. Owner creates a document resource
2. Owner delegates read-write capability to collaborators
3. Collaborators update the document using their capabilities
4. Owner can revoke specific collaborators' capabilities if needed

### Resource Transfer Workflow

1. Original owner has a resource with a capability that includes Transfer right
2. Original owner creates a transfer capability for new owner
3. New owner accepts the transfer, changing resource ownership
4. System revokes original owner's capabilities and creates new capabilities for new owner 