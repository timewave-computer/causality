# Capability API Reference

## Overview

The Capability API provides a secure and flexible way to control access to resources in the Causality system. This capability-based security model allows for fine-grained access control, delegation, and composition of rights, while preventing the "confused deputy" problem common in traditional access control systems.

## Core Concepts

### Rights

Rights define the specific permissions granted by a capability.

```rust
enum Right {
    /// Right to read the resource
    Read,
    
    /// Right to write to the resource
    Write,
    
    /// Right to delete the resource
    Delete,
    
    /// Right to transfer ownership of the resource
    Transfer,
    
    /// Right to delegate capabilities on the resource
    Delegate,
    
    /// Right to execute a specific operation on the resource
    Execute(String),
    
    /// Custom right with a named identifier
    Custom(String),
}
```

Rights can be combined to create complex permission sets. For example, a capability might grant [Read, Write] rights but not [Delete].

### Restrictions

Restrictions place constraints on how a capability can be used. They are represented as a set of key-value pairs.

```rust
struct Restrictions {
    /// Map of restriction names to their values
    restrictions: HashMap<String, String>,
}
```

Common restrictions include:
- `max_amount`: Limits the maximum amount that can be transferred
- `expiration`: Sets a time limit on the capability
- `domain`: Restricts usage to specific domains
- `operation`: Restricts to specific operations

### ResourceCapability

The central type in the Capability API, representing a capability to access and operate on a resource.

```rust
struct ResourceCapability {
    /// Unique identifier for this capability
    id: CapabilityId,
    
    /// ID of the resource this capability grants access to
    resource_id: String,
    
    /// Type of the resource
    resource_type: String,
    
    /// The address of the issuer of this capability
    issuer: Address,
    
    /// The address of the holder of this capability
    holder: Address,
    
    /// Rights granted by this capability
    rights: Vec<Right>,
    
    /// Restrictions on the use of this capability
    restrictions: Restrictions,
    
    /// Optional expiration time for this capability
    expires_at: Option<SystemTime>,
    
    /// Optional signature by the issuer
    signature: Option<Signature>,
    
    /// Optional parent capability ID if this was delegated
    parent_id: Option<CapabilityId>,
    
    /// Revocation status
    revoked: bool,
}
```

## Key Operations

### Creating Capabilities

```rust
// Create a new capability
let capability = ResourceCapability::new(
    "register_123",      // resource_id
    "Token",             // resource_type
    issuer_address,      // issuer
    holder_address,      // holder
    vec![Right::Read, Right::Transfer],  // rights
);

// Add restrictions
let mut capability = capability
    .add_restriction("max_amount", "1000")
    .add_restriction("domain", "ethereum")
    .with_expiration(SystemTime::now() + Duration::from_secs(3600));
    
// Sign the capability (cryptographically binds the issuer)
capability.sign(private_key);
```

### Validating Capabilities

Before using a capability, it should be validated to ensure it's still valid:

```rust
// Validate a capability
match capability.validate() {
    Ok(_) => {
        // Capability is valid, proceed with operation
        perform_operation_with_capability(&capability);
    },
    Err(e) => {
        match e {
            CapabilityError::Expired => { /* Handle expired capability */ },
            CapabilityError::Revoked => { /* Handle revoked capability */ },
            CapabilityError::InvalidSignature => { /* Handle invalid signature */ },
            _ => { /* Handle other errors */ },
        }
    }
}
```

### Delegation

Delegation allows a capability holder to create a new capability derived from one they hold:

```rust
// Delegate a capability to another holder
let delegated_capability = original_capability.delegate(
    new_holder_address,
    vec![Right::Read],  // Subset of rights from original
    Some(additional_restrictions)
)?;

// The delegated capability is linked to its parent
assert_eq!(delegated_capability.parent_id(), Some(original_capability.id()));

// Verify the delegation chain
validate_delegation_chain(&delegated_capability);
```

### Attenuation

Attenuation is a form of delegation where the rights or restrictions are tightened:

```rust
// Attenuate a capability (restrict further)
let attenuated = capability.attenuate(
    vec![Right::Read],  // Only grant read (even if original had more)
    Some(Restrictions::new().add("max_amount", "100"))  // Add stricter restrictions
)?;

// Attenuation always results in equal or lesser rights
assert!(attenuated.rights().len() <= capability.rights().len());
```

## Capability Repository

The `CapabilityRepository` provides centralized storage and validation for capabilities:

```rust
struct CapabilityRepository {
    /// Map of capability IDs to capabilities
    capabilities: HashMap<CapabilityId, CapabilityRef>,
    
    /// Map of resource IDs to capabilities that target them
    resource_capabilities: HashMap<String, Vec<CapabilityId>>,
    
    /// Map of holder addresses to capabilities they hold
    holder_capabilities: HashMap<Address, Vec<CapabilityId>>,
    
    /// Map of revoked capability IDs
    revoked_capabilities: Vec<CapabilityId>,
}
```

### Repository Operations

```rust
// Create a repository
let mut repo = CapabilityRepository::new();

// Register a capability
let cap_ref = repo.register(capability);

// Look up capabilities
let capabilities_for_resource = repo.get_for_resource("register_123");
let capabilities_for_holder = repo.get_for_holder(&holder_address);

// Revoke a capability
repo.revoke(&capability_id)?;

// Validate through the repository
let validated_cap = repo.validate(&capability_id)?;
```

## Capability Chains

The system supports capability chains for complex operations that require multiple capabilities:

```rust
// Create a capability chain
let mut chain = CapabilityChain::new();

// Add capabilities to the chain
chain.add(capability1)?;
chain.add(capability2)?;

// Validate the entire chain
chain.validate()?;

// Execute an operation using the chain
let result = execute_operation_with_chain(&chain, operation_params);
```

## Resource API with Capabilities

The Resource API uses capabilities to control access to resources:

```rust
trait ResourceAPI {
    // Read operations require a capability with Read right
    fn read_resource(&self, id: &str, capability: &ResourceCapability) -> Result<Resource>;
    
    // Write operations require a capability with Write right
    fn update_resource(&self, id: &str, data: &[u8], capability: &ResourceCapability) -> Result<()>;
    
    // Transfer operations require a capability with Transfer right
    fn transfer_resource(&self, id: &str, to: &Address, capability: &ResourceCapability) -> Result<()>;
    
    // Deletion operations require a capability with Delete right
    fn delete_resource(&self, id: &str, capability: &ResourceCapability) -> Result<()>;
}
```

## Integration with Register System

Capabilities integrate with the register system to control access to registers:

```rust
// Create a capability for a register
let register_capability = ResourceCapability::new(
    register.id().to_string(),
    "Register",
    owner_address,
    user_address,
    vec![Right::Read, Right::Write]
);

// Use the capability to perform register operations
match register_service.update_register(
    register_id,
    new_data,
    &register_capability
) {
    Ok(_) => println!("Register updated successfully"),
    Err(e) => println!("Failed to update register: {}", e),
}
```

## Error Handling

The Capability API uses a dedicated error type:

```rust
enum CapabilityError {
    Expired,
    Revoked,
    InvalidSignature,
    OperationNotPermitted,
    ResourceTypeMismatch { expected: String, actual: String },
    RestrictionViolated(String),
    InvalidDelegation(String),
    MissingParameter(String),
    ValidationFailed(String),
}
```

Example of handling capability errors:

```rust
fn perform_secure_operation(capability: &ResourceCapability) -> CapabilityResult<()> {
    // Validate the capability
    capability.validate()?;
    
    // Check for specific right
    if !capability.has_right(&Right::Execute("secure_operation".to_string())) {
        return Err(CapabilityError::OperationNotPermitted);
    }
    
    // Check restrictions
    let restrictions = capability.restrictions();
    if let Some(max_amount) = restrictions.get("max_amount") {
        let amount = max_amount.parse::<u64>().map_err(|_| {
            CapabilityError::RestrictionViolated("Invalid max_amount format".to_string())
        })?;
        
        if amount < 100 {
            return Err(CapabilityError::RestrictionViolated(
                "Operation requires max_amount of at least 100".to_string()
            ));
        }
    }
    
    // Perform the operation
    // ...
    
    Ok(())
}
```

## Best Practices

1. **Principle of Least Privilege**: Grant only the minimum rights necessary for an operation
2. **Always Validate**: Validate capabilities before use, especially when received from external sources
3. **Use Restrictions**: Add appropriate restrictions to limit capability scope
4. **Sign Capabilities**: Always sign capabilities to prevent tampering
5. **Capability Lifetime**: Use expirations to limit the time window of access
6. **Revocation**: Design systems with capability revocation in mind for emergency situations
7. **Audit Trail**: Keep records of capability usage for auditing purposes

## Example Workflow: Secure Asset Transfer

```rust
// 1. User requests to transfer assets
let transfer_request = TransferRequest {
    from: user_address,
    to: recipient_address,
    asset_id: "token_123",
    amount: 500,
};

// 2. Authorization system creates a capability for this specific operation
let transfer_capability = ResourceCapability::new(
    transfer_request.asset_id,
    "Token",
    system_address,
    user_address,
    vec![Right::Transfer]
)
.add_restriction("recipient", recipient_address.to_string())
.add_restriction("max_amount", transfer_request.amount.to_string())
.add_restriction("operation", "transfer")
.with_expiration(SystemTime::now() + Duration::from_secs(300))
.sign(system_private_key);

// 3. User signs the capability to authorize the operation
let user_signed_capability = transfer_capability.with_user_signature(
    user_private_key
);

// 4. System performs the transfer using the capability
let result = token_service.transfer(
    transfer_request.asset_id,
    transfer_request.to,
    transfer_request.amount,
    &user_signed_capability
);

// 5. Capability is automatically invalidated after use
capability_repository.revoke(&user_signed_capability.id())?;
```

This pattern ensures that:
- The operation is limited to exactly what the user authorized
- The capability cannot be reused for other purposes
- The system can verify that both it and the user authorized the operation
- The operation has a limited time window in which it's valid 