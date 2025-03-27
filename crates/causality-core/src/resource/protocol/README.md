# Cross-Domain Resource Protocol

The Cross-Domain Resource Protocol provides a secure and standardized way to reference, transfer, and synchronize resources across different domains in the Causality system.

## Core Components

### Resource References

Resource references provide a way to reference resources that exist in other domains:

- **Shadow References**: Read-only projections of resources from another domain
- **Bridged References**: Mutable projections with synchronized state
- **Locked References**: Resources locked in one domain and represented in another
- **Transferred References**: Resources that have been fully moved to a new domain

### Verification Levels

Different levels of verification are supported for cross-domain resource references:

- **Hash-Based**: Simple content hash verification
- **Merkle Proof**: Verification via Merkle proofs
- **Zero-Knowledge Proofs**: Cryptographic verification without revealing data
- **Consensus-Based**: Verification via domain consensus
- **Multi-Signature**: Verification via multiple authorizations

### Resource Transfer Operations

The protocol supports secure resource transfer between domains with:

- **Transfer Planning**: Preparing resources for transfer
- **Progress Tracking**: Monitoring the status of transfers
- **Verification**: Ensuring the integrity of transferred resources
- **Authorization**: Capability-based control of transfers

## Implementation

The protocol is implemented with the following components:

- **CrossDomainResourceProtocol**: Core interface for cross-domain operations
- **DomainResourceAdapter**: Domain-specific implementation of resource operations
- **BasicCrossDomainResourceProtocol**: Reference implementation

## Usage Examples

### Creating a Reference

```rust
// Create a reference to a resource in another domain
let reference = protocol.create_reference(
    resource_id,
    target_domain,
    ResourceProjectionType::Shadow,
    VerificationLevel::Hash,
    context
).await?;
```

### Transferring a Resource

```rust
// Create a transfer operation
let operation = ResourceTransferOperation::new(
    resource_id,
    source_domain,
    target_domain,
    ResourceProjectionType::Transferred,
    VerificationLevel::Hash,
    capability
);

// Execute the transfer
let transferred_reference = protocol.transfer_resource(
    operation,
    context
).await?;
```

### Synchronizing References

```rust
// Synchronize a bridged reference with its source
let updated_reference = protocol.synchronize_reference(
    bridged_reference,
    context
).await?;
```

## Integration with Domains

The cross-domain resource protocol integrates with the domain system through:

- Domain-specific resource adapters
- Domain capability mapping
- Cross-domain authorization

## Security Considerations

- All cross-domain operations require appropriate capabilities
- Resources maintain content integrity through verification
- References can be time-limited through expiration 