# Cross-Domain Operations

This document describes the architecture and implementation of cross-domain operations within the Causality system, focusing on how operations can be executed across multiple domains while maintaining consistency and security.

## Core Concepts

### Cross-Domain Operations

A **Cross-Domain Operation** is an operation that spans multiple domains, affecting resources or state in each domain. These operations must maintain several key properties:

1. **Atomicity**: The operation either succeeds in all domains or fails in all domains
2. **Consistency**: The operation maintains system invariants across domain boundaries
3. **Security**: The operation is properly authorized in all domains
4. **Verifiability**: The execution of the operation can be verified across domains
5. **Temporal Coherence**: The operation maintains happened-before relationships across domains

### Domain Boundaries

Domain boundaries define the separation between different execution environments:

```
┌───────────────────┐     ┌───────────────────┐     ┌───────────────────┐
│                   │     │                   │     │                   │
│     Domain A      │     │     Domain B      │     │     Domain C      │
│  (Ethereum)       │     │  (Cosmos)         │     │  (TEL Environment)│
│                   │     │                   │     │                   │
└─────────┬─────────┘     └────────┬──────────┘     └────────┬──────────┘
          │                        │                         │
          │                        │                         │
          ▼                        ▼                         ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                      Causality Cross-Domain Layer                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Operation Structure

Cross-domain operations have a specific structure to facilitate execution across domains:

```rust
/// A cross-domain operation
pub struct CrossDomainOperation {
    /// Unique identifier
    id: OperationId,
    
    /// The unified operation being executed
    operation: Operation,
    
    /// Domains involved in this operation
    domains: Vec<DomainId>,
    
    /// Execution strategy
    strategy: CrossDomainStrategy,
    
    /// Coordination metadata
    coordination: CoordinationMetadata,
    
    /// Current execution status
    status: CrossDomainOperationStatus,
    
    /// Domain-specific operation parts
    domain_operations: HashMap<DomainId, DomainSpecificOperation>,
    
    /// Temporal verification data
    temporal_data: TemporalContext,
    
    /// Cross-domain authorization
    authorization: CrossDomainAuthorization,
}

/// Strategy for executing cross-domain operations
pub enum CrossDomainStrategy {
    /// All parts must succeed or all fail
    AtomicCommit {
        /// Timeout for the entire operation
        timeout: Duration,
        /// Verification level required
        verification_level: VerificationLevel,
    },
    
    /// Operations executed in sequence, later ones can be skipped if earlier ones fail
    Sequential {
        /// Continue on errors in some domains
        continue_on_error: bool,
        /// Domains in execution order
        domain_order: Vec<DomainId>,
    },
    
    /// Operations coordinated by a specific domain
    Coordinated {
        /// Domain that coordinates the operation
        coordinator: DomainId,
        /// Fallback behavior if coordinator fails
        fallback: Option<Box<CrossDomainStrategy>>,
    },
}
```

## Execution Model

### Execution Phases

Cross-domain operations go through several execution phases:

1. **Planning**: Define the operation and its domain-specific parts
2. **Authorization**: Verify authorization across all domains
3. **Preparation**: Prepare the state in each domain for execution
4. **Validation**: Validate the operation across all domains
5. **Commitment**: Commit the operation in all domains
6. **Finalization**: Update all domains with the final results
7. **Verification**: Verify that the operation completed correctly

### Phase Transitions

```
                  ┌───────────┐
                  │           │
                  │ Planning  │
                  │           │
                  └─────┬─────┘
                        │
                        ▼
                  ┌───────────┐
                  │           │
                  │Authorization
                  │           │
                  └─────┬─────┘
                        │
                        ▼
                  ┌───────────┐
                  │           │
                  │Preparation│
                  │           │
                  └─────┬─────┘
                        │
                        ▼
┌──────────┐      ┌───────────┐     ┌───────────┐
│          │      │           │     │           │
│ Rollback │◄─────┤Validation │────►│Commitment │
│          │      │           │     │           │
└──────────┘      └───────────┘     └─────┬─────┘
                                          │
                                          ▼
                                    ┌───────────┐
                                    │           │
                                    │Finalization
                                    │           │
                                    └─────┬─────┘
                                          │
                                          ▼
                                    ┌───────────┐
                                    │           │
                                    │Verification
                                    │           │
                                    └───────────┘
```

## Coordination Mechanisms

### Two-Phase Commit Protocol

For atomic operations, a two-phase commit protocol is used:

```rust
/// Execute an atomic cross-domain operation
pub async fn execute_atomic_operation(
    &self,
    operation: CrossDomainOperation,
) -> Result<CrossDomainOperationResult> {
    // Phase 1: Prepare
    let prepare_results = self.prepare_all_domains(&operation).await?;
    
    if prepare_results.all_prepared() {
        // Phase 2: Commit
        let commit_results = self.commit_all_domains(&operation).await?;
        
        if commit_results.all_committed() {
            // Finalize
            let finalize_results = self.finalize_all_domains(&operation).await?;
            Ok(CrossDomainOperationResult::success(operation.id(), finalize_results))
        } else {
            // Some domains failed to commit, try to rollback prepared domains
            let rollback_results = self.rollback_prepared_domains(&operation).await?;
            Ok(CrossDomainOperationResult::failure(
                operation.id(),
                "Some domains failed to commit",
                rollback_results
            ))
        }
    } else {
        // Some domains failed to prepare, rollback any that did prepare
        let rollback_results = self.rollback_prepared_domains(&operation).await?;
        Ok(CrossDomainOperationResult::failure(
            operation.id(),
            "Some domains failed to prepare",
            rollback_results
        ))
    }
}
```

### Sequential Coordination

For sequential operations, domains are processed in order:

```rust
/// Execute a sequential cross-domain operation
pub async fn execute_sequential_operation(
    &self,
    operation: CrossDomainOperation,
) -> Result<CrossDomainOperationResult> {
    let strategy = match &operation.strategy {
        CrossDomainStrategy::Sequential { continue_on_error, domain_order } => {
            (continue_on_error, domain_order)
        },
        _ => return Err(Error::invalid_strategy("Expected Sequential strategy")),
    };
    
    let (continue_on_error, domain_order) = strategy;
    let mut results = HashMap::new();
    let mut overall_success = true;
    
    for domain_id in domain_order {
        let domain_operation = operation.domain_operations.get(domain_id)
            .ok_or_else(|| Error::missing_domain_operation(domain_id.clone()))?;
        
        let domain_result = self.execute_in_domain(
            domain_id,
            domain_operation,
            &operation
        ).await;
        
        match domain_result {
            Ok(result) => {
                results.insert(domain_id.clone(), result);
                if !result.success && !continue_on_error {
                    overall_success = false;
                    break;
                }
            },
            Err(e) => {
                results.insert(
                    domain_id.clone(),
                    DomainOperationResult::error(domain_id.clone(), e.to_string())
                );
                if !continue_on_error {
                    overall_success = false;
                    break;
                }
            }
        }
    }
    
    if overall_success {
        Ok(CrossDomainOperationResult::success(operation.id(), results))
    } else {
        Ok(CrossDomainOperationResult::partial(
            operation.id(),
            "Sequential operation partially completed",
            results
        ))
    }
}
```

### Coordinator-Based

For operations with a coordinator domain:

```rust
/// Execute a coordinated cross-domain operation
pub async fn execute_coordinated_operation(
    &self,
    operation: CrossDomainOperation,
) -> Result<CrossDomainOperationResult> {
    let (coordinator, fallback) = match &operation.strategy {
        CrossDomainStrategy::Coordinated { coordinator, fallback } => {
            (coordinator, fallback)
        },
        _ => return Err(Error::invalid_strategy("Expected Coordinated strategy")),
    };
    
    // Delegate coordination to the coordinator domain
    let coordination_result = self.coordinate_operation(
        coordinator,
        &operation
    ).await;
    
    match coordination_result {
        Ok(result) => Ok(result),
        Err(e) => {
            if let Some(fallback_strategy) = fallback {
                // Use fallback strategy
                let mut fallback_operation = operation.clone();
                fallback_operation.strategy = *fallback_strategy.clone();
                self.execute_cross_domain_operation(fallback_operation).await
            } else {
                Err(e)
            }
        }
    }
}
```

## Temporal Consistency

### Cross-Domain Causal Ordering

Cross-domain operations maintain causality through temporal facts:

```rust
/// Ensure temporal consistency across domains
pub fn ensure_temporal_consistency(
    &self,
    operation: &CrossDomainOperation,
) -> Result<()> {
    // Get all temporal facts related to resources in this operation
    let mut facts = Vec::new();
    
    for (domain_id, domain_op) in &operation.domain_operations {
        let domain_facts = self.fact_service.get_facts_for_resources(
            domain_id,
            &domain_op.resources()
        )?;
        
        facts.extend(domain_facts);
    }
    
    // Create a temporal context from the facts
    let temporal_context = TemporalContext::new(facts);
    
    // Validate happened-before relationships
    for dependency in operation.temporal_data.dependencies() {
        temporal_context.validate_dependency(dependency)?;
    }
    
    // Validate no temporal cycles are created
    temporal_context.validate_acyclic()?;
    
    Ok(())
}
```

### Cross-Domain Fact Propagation

Facts are propagated between domains to maintain consistency:

```rust
/// Propagate facts across domains
pub async fn propagate_facts(
    &self,
    operation: &CrossDomainOperation,
    facts: &[TemporalFact],
) -> Result<()> {
    for domain_id in &operation.domains {
        let domain = self.domain_registry.get_domain(domain_id)?;
        
        if domain.supports_fact_observation() {
            let domain_adapter = self.domain_adapter_factory.create_adapter(domain_id)?;
            
            for fact in facts {
                domain_adapter.observe_fact(fact).await?;
            }
        }
    }
    
    Ok(())
}
```

## Authorization Model

### Cross-Domain Authorization

Operations must be authorized in all affected domains:

```rust
/// Verify authorization across all domains
pub async fn verify_cross_domain_authorization(
    &self,
    operation: &CrossDomainOperation,
) -> Result<bool> {
    let mut all_authorized = true;
    
    for (domain_id, domain_op) in &operation.domain_operations {
        let domain_adapter = self.domain_adapter_factory.create_adapter(domain_id)?;
        
        let auth_result = domain_adapter.verify_authorization(
            &operation.authorization.for_domain(domain_id),
            domain_op
        ).await?;
        
        if !auth_result {
            all_authorized = false;
            break;
        }
    }
    
    Ok(all_authorized)
}
```

### Capability Projection

Capabilities are projected across domains for authorization:

```rust
/// Project capabilities across domains
pub fn project_capabilities(
    &self,
    capabilities: &[Capability],
    source_domain: &DomainId,
    target_domain: &DomainId,
) -> Result<Vec<Capability>> {
    let mut projected_capabilities = Vec::new();
    
    for capability in capabilities {
        let projected = self.capability_projection_service.project_capability(
            capability,
            source_domain,
            target_domain,
            ProjectionType::Bridged
        )?;
        
        projected_capabilities.push(projected);
    }
    
    Ok(projected_capabilities)
}
```

## Resource Integration

### Cross-Domain Resource Operations

Operations that affect resources across domains:

```rust
/// Create a cross-domain transfer operation
pub fn create_cross_domain_transfer(
    &self,
    source_resource: &ResourceId,
    source_domain: &DomainId,
    target_domain: &DomainId,
    recipient: &str,
    amount: u64,
) -> Result<CrossDomainOperation> {
    // Create the withdrawal effect for source domain
    let withdrawal_effect = WithdrawalEffect::new(
        source_resource.clone(),
        amount,
        Some(target_domain.clone()),
        HashMap::new()
    );
    
    // Create the deposit effect for target domain
    let deposit_effect = DepositEffect::new(
        ResourceId::from_string(recipient)?,
        amount,
        Some(source_domain.clone()),
        HashMap::new()
    );
    
    // Create the domain-specific operations
    let mut domain_operations = HashMap::new();
    
    // Source domain operation
    domain_operations.insert(
        source_domain.clone(),
        DomainSpecificOperation::new(
            OperationType::Withdrawal,
            Box::new(withdrawal_effect.clone())
        )
    );
    
    // Target domain operation
    domain_operations.insert(
        target_domain.clone(),
        DomainSpecificOperation::new(
            OperationType::Deposit,
            Box::new(deposit_effect.clone())
        )
    );
    
    // Create the cross-domain operation
    let operation = Operation::new(OperationType::CrossDomainTransfer)
        .with_abstract_representation(Box::new(CompositeEffect::new(
            vec![
                Box::new(withdrawal_effect),
                Box::new(deposit_effect),
            ],
            CompositionMode::Sequential
        )));
    
    let cross_domain_operation = CrossDomainOperation::new(
        operation,
        vec![source_domain.clone(), target_domain.clone()],
        CrossDomainStrategy::AtomicCommit {
            timeout: Duration::from_secs(300),
            verification_level: VerificationLevel::Full,
        },
        domain_operations
    );
    
    Ok(cross_domain_operation)
}
```

### Resource Synchronization

Keeping resource state synchronized across domains:

```rust
/// Synchronize resource state across domains
pub async fn synchronize_resource(
    &self,
    resource_id: &ResourceId,
    source_domain: &DomainId,
    target_domains: &[DomainId],
) -> Result<Vec<ResourceSyncResult>> {
    let source_adapter = self.domain_adapter_factory.create_adapter(source_domain)?;
    
    // Get the resource state from the source domain
    let resource_state = source_adapter.get_resource_state(resource_id).await?;
    
    let mut results = Vec::new();
    
    // Synchronize to each target domain
    for target_domain in target_domains {
        let target_adapter = self.domain_adapter_factory.create_adapter(target_domain)?;
        
        // Create a sync operation for this domain
        let sync_operation = ResourceSyncOperation::new(
            resource_id.clone(),
            resource_state.clone(),
            source_domain.clone(),
            target_domain.clone()
        );
        
        // Execute the sync operation
        let result = target_adapter.sync_resource(&sync_operation).await;
        
        results.push(ResourceSyncResult::new(
            resource_id.clone(),
            target_domain.clone(),
            result.is_ok(),
            result.err().map(|e| e.to_string())
        ));
    }
    
    Ok(results)
}
```

## Domain Adapter Integration

### Domain-Specific Transformations

Operations are transformed for domain-specific execution:

```rust
/// Transform an operation for a specific domain
pub fn transform_for_domain(
    &self,
    operation: &Operation,
    domain_id: &DomainId,
) -> Result<DomainSpecificOperation> {
    let domain = self.domain_registry.get_domain(domain_id)?;
    let transformation_service = self.transformation_service_factory.create_service(domain_id)?;
    
    // Get the abstract effect from the operation
    let effect = operation.abstract_representation()?;
    
    // Transform the effect for this domain
    let domain_specific_effect = transformation_service.transform_effect(
        effect,
        domain_id
    )?;
    
    // Create a domain-specific operation
    let domain_operation = DomainSpecificOperation::new(
        operation.operation_type().clone(),
        domain_specific_effect
    );
    
    Ok(domain_operation)
}
```

### Domain Adapter Interface

```rust
/// Interface for domain adapters that support cross-domain operations
pub trait CrossDomainAdapter: DomainAdapter {
    /// Prepare for a cross-domain operation
    async fn prepare_operation(
        &self,
        operation: &DomainSpecificOperation,
        context: &CrossDomainContext,
    ) -> Result<PrepareResult>;
    
    /// Commit a prepared operation
    async fn commit_operation(
        &self,
        operation_id: &OperationId,
        context: &CrossDomainContext,
    ) -> Result<CommitResult>;
    
    /// Rollback a prepared operation
    async fn rollback_operation(
        &self,
        operation_id: &OperationId,
        context: &CrossDomainContext,
    ) -> Result<RollbackResult>;
    
    /// Observe facts from another domain
    async fn observe_facts(
        &self,
        facts: &[TemporalFact],
        source_domain: &DomainId,
    ) -> Result<Vec<FactObservationResult>>;
    
    /// Get the supported cross-domain capabilities
    fn supported_cross_domain_capabilities(&self) -> Vec<CrossDomainCapability>;
}
```

## Error Handling and Recovery

### Operation Rollback

Handling failures in cross-domain operations:

```rust
/// Rollback a failed cross-domain operation
pub async fn rollback_operation(
    &self,
    operation: &CrossDomainOperation,
    results: &HashMap<DomainId, DomainOperationResult>,
) -> Result<HashMap<DomainId, RollbackResult>> {
    let mut rollback_results = HashMap::new();
    
    for (domain_id, result) in results {
        if result.status == DomainOperationStatus::Prepared ||
           result.status == DomainOperationStatus::Committed {
            let domain_adapter = self.domain_adapter_factory.create_adapter(domain_id)?;
            
            let rollback_result = domain_adapter.rollback_operation(
                &operation.id,
                &CrossDomainContext::new(operation, domain_id)
            ).await;
            
            match rollback_result {
                Ok(result) => {
                    rollback_results.insert(domain_id.clone(), result);
                },
                Err(e) => {
                    rollback_results.insert(
                        domain_id.clone(),
                        RollbackResult::error(domain_id.clone(), e.to_string())
                    );
                }
            }
        }
    }
    
    Ok(rollback_results)
}
```

### Partial Operation Handling

Handling partially completed operations:

```rust
/// Handle a partially completed operation
pub async fn handle_partial_operation(
    &self,
    operation: &CrossDomainOperation,
    results: &HashMap<DomainId, DomainOperationResult>,
) -> Result<CrossDomainOperationResult> {
    // Record the partial operation for later recovery
    self.partial_operation_store.store_partial_operation(
        operation.clone(),
        results.clone()
    )?;
    
    // Create a recovery operation if possible
    if let Some(recovery_operation) = self.create_recovery_operation(operation, results)? {
        // Execute the recovery operation
        self.execute_cross_domain_operation(recovery_operation).await
    } else {
        // No automatic recovery possible, return partial result
        Ok(CrossDomainOperationResult::partial(
            operation.id(),
            "Operation partially completed without recovery",
            results.clone()
        ))
    }
}
```

## Usage Examples

### Basic Cross-Domain Transfer

```rust
// Create the cross-domain service
let cross_domain_service = CrossDomainOperationService::new(
    domain_registry.clone(),
    domain_adapter_factory.clone(),
    transformation_service_factory.clone()
);

// Create a cross-domain transfer
let transfer_operation = cross_domain_service.create_cross_domain_transfer(
    &eth_token_resource_id,            // Source resource (in Ethereum)
    &ethereum_domain_id,               // Source domain
    &cosmos_domain_id,                 // Target domain
    "cosmos1a2b3c4d5e6f7g8h9i0j",      // Recipient address
    1000                               // Amount to transfer
)?;

// Add authorization
let authorized_operation = transfer_operation.with_authorization(
    CrossDomainAuthorization::new(
        user_id.clone(),
        vec![transfer_capability_id]
    )
);

// Execute the operation
let result = cross_domain_service.execute_cross_domain_operation(
    authorized_operation
).await?;

// Process the result
if result.success {
    println!("Cross-domain transfer completed successfully");
    
    // Access domain-specific results
    let eth_result = result.domain_results.get(&ethereum_domain_id).unwrap();
    let cosmos_result = result.domain_results.get(&cosmos_domain_id).unwrap();
    
    println!("Ethereum transaction hash: {}", 
        eth_result.metadata.get("transaction_hash").unwrap());
    println!("Cosmos transaction hash: {}", 
        cosmos_result.metadata.get("transaction_hash").unwrap());
} else {
    println!("Cross-domain transfer failed: {}", result.error.unwrap_or_default());
    
    // Check which domains failed
    for (domain_id, domain_result) in &result.domain_results {
        if !domain_result.success {
            println!("Failed in domain {}: {}", 
                domain_id, domain_result.error.unwrap_or_default());
        }
    }
}
```

### Multi-Domain Resource Update

```rust
// Create a cross-domain update operation that affects resources in multiple domains
let update_operation = cross_domain_service.create_multi_domain_update(
    &[eth_resource_id, cosmos_resource_id, tel_resource_id],
    &[ethereum_domain_id, cosmos_domain_id, tel_domain_id],
    update_properties,
    CrossDomainStrategy::Sequential {
        continue_on_error: true,
        domain_order: vec![
            ethereum_domain_id.clone(),
            cosmos_domain_id.clone(),
            tel_domain_id.clone()
        ]
    }
)?;

// Execute the sequential update
let result = cross_domain_service.execute_cross_domain_operation(
    update_operation
).await?;

// Check each domain result
for (domain_id, domain_result) in &result.domain_results {
    if domain_result.success {
        println!("Update succeeded in domain {}", domain_id);
    } else {
        println!("Update failed in domain {}: {}", 
            domain_id, domain_result.error.unwrap_or_default());
    }
}
```

### Coordinated Multi-Domain Operation

```rust
// Create a coordinated cross-domain operation with the TEL domain as coordinator
let coordinated_operation = CrossDomainOperation::new(
    complex_operation,
    vec![ethereum_domain_id.clone(), cosmos_domain_id.clone(), tel_domain_id.clone()],
    CrossDomainStrategy::Coordinated {
        coordinator: tel_domain_id.clone(),
        fallback: Some(Box::new(CrossDomainStrategy::AtomicCommit {
            timeout: Duration::from_secs(300),
            verification_level: VerificationLevel::Full,
        }))
    },
    domain_operations
);

// Execute the coordinated operation
let result = cross_domain_service.execute_cross_domain_operation(
    coordinated_operation
).await?;

// Process results
println!("Operation completed with status: {}", result.status);
```

## Implementation Status

The cross-domain operations system is partially implemented:

- ✅ Core cross-domain operation model
- ✅ Two-phase commit protocol for atomic operations
- ✅ Sequential operation execution
- ✅ Cross-domain resource projection
- ✅ Fact propagation between domains
- ✅ Basic cross-domain authorization
- ⚠️ Coordinated operations (in progress)
- ⚠️ Advanced error recovery (in progress)
- ❌ Cross-domain transaction rollback
- ❌ Optimistic cross-domain execution

## Future Enhancements

1. **Cross-Domain Proof Generation**: ZK proofs that span multiple domains
2. **Speculative Execution**: Execute operations optimistically and roll back if necessary
3. **Cross-Domain Sharding**: Partition operations across domain shards
4. **Resilient Coordination**: Coordinator election and fallback mechanisms
5. **Performance Optimization**: Parallelized cross-domain operations
6. **Enhanced Recovery**: Advanced recovery strategies for partial operations 