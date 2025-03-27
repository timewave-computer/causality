# Cross-Domain Operation Guide

This guide explains how to perform operations across domains in the Causality system, focusing on the Agent Resource System and Effect System integration.

## Overview

Domains in Causality represent isolated contexts with their own resources, policies, and operations. Cross-domain operations involve coordinating actions across these boundaries while maintaining security, consistency, and causal ordering.

## Key Concepts

### Domains

A domain is a logical boundary in the Causality system with:

- A unique identifier (`DomainId`)
- Associated resources (agents, facts, etc.)
- Security policies
- Local causal time ordering

### Cross-Domain Operation Patterns

There are several patterns for cross-domain operations:

1. **Direct Resource Reference**: Referencing resources from other domains
2. **Capability Delegation**: Using capabilities from one domain in another
3. **Cross-Domain Effects**: Creating and executing effects that span domains
4. **Message Passing**: Exchanging messages between domains
5. **Shared Facts**: Creating facts visible across domain boundaries

## Setting Up Cross-Domain Communication

### Domain Registration

Before domains can communicate, they must be registered and discoverable:

```rust
use causality_core::{
    types::DomainId,
    domain::registry::DomainRegistry,
};

// Create domain IDs
let domain_a_id = DomainId::new("domain_a");
let domain_b_id = DomainId::new("domain_b");

// Register domains
let domain_registry = DomainRegistry::new();
domain_registry.register(domain_a_id.clone(), domain_a_info).await?;
domain_registry.register(domain_b_id.clone(), domain_b_info).await?;
```

### Establishing Trust Between Domains

Domains need trust relationships for secure cross-domain operations:

```rust
use causality_core::{
    crypto::KeyPair,
    domain::trust::TrustRelationship,
    resource::agent::Authorization,
};

// Generate key pairs for domains
let domain_a_key = KeyPair::generate()?;
let domain_b_key = KeyPair::generate()?;

// Create authorization proofs
let domain_a_auth = Authorization::new(domain_a_id.clone(), domain_a_key.public());
let domain_b_auth = Authorization::new(domain_b_id.clone(), domain_b_key.public());

// Establish bidirectional trust
let trust_a_to_b = TrustRelationship::new(
    domain_a_id.clone(),
    domain_b_id.clone(),
    vec![domain_b_auth.clone()],
);

let trust_b_to_a = TrustRelationship::new(
    domain_b_id.clone(),
    domain_a_id.clone(),
    vec![domain_a_auth.clone()],
);

// Register trust relationships
domain_registry.register_trust(trust_a_to_b).await?;
domain_registry.register_trust(trust_b_to_a).await?;
```

## Direct Resource Reference

The simplest form of cross-domain operation is referencing a resource from another domain:

```rust
use causality_core::{
    types::ResourceId,
    resource::ResourceReference,
};

// Create a reference to a resource in another domain
let resource_id = ResourceId::new("resource_123");
let resource_ref = ResourceReference::new(
    domain_b_id.clone(),
    resource_id,
);

// Use the reference to access the resource
let resource = domain_registry.resolve_reference(&resource_ref).await?;
```

## Capability Delegation

Capability delegation allows domains to grant access to their resources:

```rust
use causality_core::{
    resource::agent::{
        capability::{CapabilityBundle, CapabilityScope, DelegationRules},
        Authorization,
    },
};

// Create a capability bundle in domain A
let mut bundle = CapabilityBundle::new(
    "access_resources",
    domain_a_id.clone(),
);

// Add capabilities to the bundle
bundle.add_capability("read", CapabilityScope::Resource("resource_123"));
bundle.add_capability("write", CapabilityScope::Resource("resource_123"));

// Set delegation rules
let delegation_rules = DelegationRules::new()
    .allow_domain(domain_b_id.clone())
    .set_max_delegation_depth(2)
    .set_expiration_time(std::time::SystemTime::now() + std::time::Duration::from_secs(3600));

bundle.set_delegation_rules(delegation_rules);

// Sign the bundle with domain A's key
let authorization = Authorization::new(domain_a_id.clone(), domain_a_key.public());
let signed_bundle = bundle.sign(&domain_a_key, &authorization)?;

// Delegate the bundle to domain B
let delegation = signed_bundle.delegate_to(
    domain_b_id.clone(),
    &domain_a_key,
    &authorization,
)?;

// In domain B, use the delegated capability
let capability_manager = domain_b.capability_manager();
capability_manager.register_delegation(delegation).await?;
capability_manager.can_access("read", "resource_123").await?; // Should return true
```

## Cross-Domain Effects

Effects can span multiple domains by composing domain-specific effects:

```rust
use causality_core::{
    effect::{Effect, EffectBuilder, EffectRegistry},
    resource::agent::capability::CapabilityVerification,
};

// Create a domain A effect
let effect_a = create_domain_a_effect();

// Create a domain B effect
let effect_b = create_domain_b_effect();

// Create a cross-domain effect
let cross_domain_effect = EffectBuilder::new("cross_domain_operation")
    .add_effect(effect_a)
    .add_effect(effect_b)
    .build();

// Create verifiers for both domains
let verifier_a = CapabilityVerification::new(domain_a_id.clone());
let verifier_b = CapabilityVerification::new(domain_b_id.clone());

// Register effects with their verifiers
let mut registry = EffectRegistry::new();
registry.register("domain_a_effect", domain_a_handler, verifier_a);
registry.register("domain_b_effect", domain_b_handler, verifier_b);

// Execute the cross-domain effect
let context = EffectContext::new()
    .with_authorization(signed_authorization)
    .with_capability_delegations(vec![delegation]);

let outcome = registry.execute(&cross_domain_effect, &context).await?;
```

## Message Passing

Domains can communicate by sending messages:

```rust
use causality_core::{
    domain::messaging::{Message, MessageBus},
    types::FactId,
};

// Create a message bus
let message_bus = MessageBus::new();

// Send a message from domain A to domain B
let message = Message::new()
    .from_domain(domain_a_id.clone())
    .to_domain(domain_b_id.clone())
    .with_subject("resource_update")
    .with_content(content_bytes)
    .with_related_fact(FactId::new("fact_123"))
    .sign(&domain_a_key, &authorization)?;

message_bus.send(message).await?;

// In domain B, receive messages
let receiver = message_bus.subscribe(domain_b_id.clone());
while let Some(message) = receiver.receive().await {
    // Verify the message signature
    message.verify(domain_registry.get_public_key(&message.from()))?;
    
    // Process the message
    match message.subject() {
        "resource_update" => process_resource_update(message.content()),
        _ => log::warn!("Unknown message subject"),
    }
}
```

## Shared Facts

Facts can be shared across domains to establish common ground:

```rust
use causality_core::{
    fact::{Fact, FactStore},
    types::FactId,
};

// Create a fact in domain A
let fact_id = FactId::new("shared_fact_123");
let fact = Fact::new(fact_id.clone(), fact_content);

// Store the fact in domain A's fact store
let domain_a_store = FactStore::for_domain(domain_a_id.clone());
domain_a_store.store(fact.clone()).await?;

// Make the fact available to domain B (by reference)
let shared_reference = domain_a.share_fact(fact_id.clone(), domain_b_id.clone()).await?;

// In domain B, retrieve the shared fact
let domain_b_store = FactStore::for_domain(domain_b_id.clone());
domain_b_store.import_from_reference(shared_reference).await?;

// Both domains can now reference the same fact
let fact_in_a = domain_a_store.get(fact_id.clone()).await?;
let fact_in_b = domain_b_store.get(fact_id.clone()).await?;
```

## Cross-Domain Causal Time

Maintaining causal relationships across domains is crucial:

```rust
use causality_core::time::{
    service::TimeService,
    integration::TimeEffectIntegration,
};

// Create time services for both domains
let time_service_a = TimeService::for_domain(domain_a_id.clone());
let time_service_b = TimeService::for_domain(domain_b_id.clone());

// In domain A, create a fact with causal time
let fact_a = create_fact_with_content("fact_a_content");
let fact_a_id = fact_a.id();

// Record causal time for the fact
time_service_a.record_logical_time(fact_a_id.clone(), 42).await?;

// In domain B, create a fact that depends on fact_a
let fact_b = create_fact_with_content("fact_b_content");
let fact_b_id = fact_b.id();

// Record the dependency
time_service_b.record_dependencies(fact_b_id.clone(), vec![fact_a_id.clone()]).await?;

// Check causal relationships
let happened_before = time_service_b.happened_before(fact_a_id.clone(), fact_b_id.clone()).await?;
assert!(happened_before);
```

## Agent Capabilities Across Domains

Agents can operate across domains with proper authorization:

```rust
use causality_core::resource::agent::{Agent, Operation};

// Create an agent in domain A
let agent_a = Agent::new("agent_a", domain_a_id.clone());

// Create an operation that crosses domains
let operation = Operation::new()
    .in_domain(domain_a_id.clone())
    .target_resource(resource_id.clone())
    .with_action("read")
    .with_cross_domain_target(domain_b_id.clone(), target_resource_id.clone());

// Verify the operation with capabilities
let can_perform = agent_a
    .verify_operation(&operation, &capability_manager)
    .await?;

if can_perform {
    // Execute the cross-domain operation
    let effect = operation.to_effect();
    registry.execute(&effect, &context).await?;
}
```

## Security Considerations

When implementing cross-domain operations, consider these security best practices:

1. **Validate All Cross-Domain References**: Always verify that cross-domain resource references are valid and accessible.

2. **Enforce Capability Chains**: Ensure capability delegations form a valid chain back to the originating domain.

3. **Verify Signatures**: Always verify signatures on cross-domain messages, capabilities, and effects.

4. **Respect Delegation Rules**: Honor max delegation depth, expiration times, and domain restrictions.

5. **Log Cross-Domain Operations**: Maintain audit logs of all cross-domain operations for security analysis.

6. **Implement Rate Limiting**: Protect against denial-of-service attacks by limiting cross-domain operation rates.

7. **Use Timeouts**: Implement timeouts for cross-domain operations to prevent hanging operations.

8. **Validate Causal Ordering**: Ensure that causal relationships are maintained across domain boundaries.

## Advanced Patterns

### Federated Domains

Multiple domains can form a federation with shared governance:

```rust
use causality_core::domain::federation::{Federation, FederationPolicy};

// Create a federation
let federation = Federation::new("production_federation")
    .add_domain(domain_a_id.clone())
    .add_domain(domain_b_id.clone())
    .add_domain(domain_c_id.clone());

// Define federation policies
let policy = FederationPolicy::new()
    .require_consensus(2) // Require at least 2 domains to agree on operations
    .allow_resource_sharing(true)
    .allow_capability_delegation(true);

federation.set_policy(policy);

// Register the federation
domain_registry.register_federation(federation).await?;
```

### Cross-Domain Transactions

Atomic operations across domains can be implemented using two-phase commit:

```rust
use causality_core::domain::transaction::{Transaction, TransactionCoordinator};

// Create a transaction coordinator
let coordinator = TransactionCoordinator::new();

// Create a cross-domain transaction
let transaction = Transaction::new("update_resources")
    .add_operation(domain_a_operation)
    .add_operation(domain_b_operation);

// Execute the transaction (implements two-phase commit internally)
let result = coordinator.execute(transaction).await?;

if result.is_success() {
    println!("Transaction committed successfully");
} else {
    println!("Transaction rolled back: {}", result.error().unwrap());
}
```

### Domain Bridges

For specialized cross-domain interactions, create dedicated bridges:

```rust
use causality_core::domain::bridge::{DomainBridge, BridgeConfig};

// Create a bridge between domains
let bridge = DomainBridge::new(
    domain_a_id.clone(),
    domain_b_id.clone(),
    BridgeConfig::new()
        .with_resource_mapping(resource_mapping)
        .with_capability_translation(capability_translation)
        .with_message_handlers(message_handlers)
);

// Start the bridge
bridge.start().await?;

// Use the bridge to perform cross-domain operations
bridge.forward_operation(operation).await?;
```

## Example: Resource Synchronization

This example shows how to synchronize a resource between two domains:

```rust
async fn synchronize_resource(
    resource_id: ResourceId,
    source_domain: DomainId,
    target_domain: DomainId,
    registry: &EffectRegistry,
) -> Result<(), Error> {
    // Create a resource reference
    let resource_ref = ResourceReference::new(source_domain.clone(), resource_id.clone());
    
    // Resolve the reference to get the resource from the source domain
    let resource = domain_registry.resolve_reference(&resource_ref).await?;
    
    // Create an effect to update the resource in the target domain
    let update_effect = EffectBuilder::new("resource_sync")
        .add_attribute("domain", target_domain.clone())
        .add_attribute("resource_id", resource_id.clone())
        .add_attribute("operation", "update")
        .add_data("resource_data", resource.data().clone())
        .build();
    
    // Create a time effect to maintain causal ordering
    let time_integration = TimeEffectIntegration::for_domain(target_domain.clone());
    let causal_effect = time_integration
        .create_causal_time_effect(
            target_domain.clone(),
            vec![resource.fact_id().clone()],
        )
        .await?;
    
    // Combine the effects
    let composite_effect = EffectBuilder::new("sync_with_causality")
        .add_effect(update_effect)
        .add_causal_time_effect(causal_effect)
        .build();
    
    // Execute the composite effect
    let context = EffectContext::new();
    registry.execute(&composite_effect, &context).await?;
    
    Ok(())
}
```

## Conclusion

Cross-domain operations in Causality enable powerful coordination between isolated domains while maintaining security and consistency. By using capabilities, shared facts, and cross-domain effects, applications can build sophisticated systems that span multiple security boundaries.

Always remember to:
- Establish proper trust relationships between domains
- Verify capabilities and authorizations for cross-domain operations
- Maintain causal ordering across domain boundaries
- Use appropriate patterns based on your security and coordination needs

With these principles in mind, cross-domain operations in Causality provide a robust foundation for building distributed systems with clear security boundaries. 