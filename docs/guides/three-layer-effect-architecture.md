# Implementing with the Three-Layer Effect Architecture

*This guide provides practical instructions for working with the Three-Layer Effect Architecture in Causality.*

*Last updated: 2023-09-05*

## Overview

This guide shows how to implement, extend, and work with the Three-Layer Effect Architecture in Causality. We'll cover practical patterns and best practices for each layer of the architecture.

## Prerequisites

- Familiarity with the [Three-Layer Effect Architecture](../../architecture/core/three-layer-effect-architecture.md) concept
- Basic understanding of Rust traits and generics
- Knowledge of the [Effect System](../../architecture/core/effect-system.md)

## Creating Custom Effects

### 1. Define the Effect

Start by defining your effect struct:

```rust
/// Asset transfer effect
pub struct TransferAssetEffect<R> {
    /// Source account
    pub source: AccountId,
    /// Destination account
    pub destination: AccountId,
    /// Asset to transfer
    pub asset: AssetId,
    /// Amount to transfer
    pub amount: u64,
    /// Continuation for the effect
    pub continuation: Box<dyn Continuation<TransferResult, R>>,
}
```

### 2. Implement the Effect Trait

Implement the `Effect` trait for your effect:

```rust
impl<R> Effect<R> for TransferAssetEffect<R> {
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R> {
        // Try to downcast the handler to the appropriate type
        if let Some(asset_handler) = handler.downcast_ref::<AssetEffectHandler>() {
            // Execute the transfer
            match asset_handler.transfer_asset(
                self.source,
                self.destination,
                self.asset,
                self.amount
            ) {
                Ok(result) => {
                    // Apply continuation to the result
                    let value = self.continuation.apply(result);
                    EffectOutcome::Success(value)
                }
                Err(error) => EffectOutcome::Error(error.into()),
            }
        } else {
            // Return error if handler not found
            EffectOutcome::Error(EffectError::UnsupportedEffect)
        }
    }
    
    fn effect_id(&self) -> EffectId {
        // Generate a unique ID based on the effect type and parameters
        let mut hasher = ContentHasher::new();
        hasher.update("TransferAssetEffect".as_bytes());
        hasher.update(&self.source.to_bytes());
        hasher.update(&self.destination.to_bytes());
        hasher.update(&self.asset.to_bytes());
        hasher.update(&self.amount.to_le_bytes());
        
        EffectId::from_content_hash(hasher.finalize())
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        // Report the resources this effect needs access to
        vec![
            ResourceId::from_asset(self.asset, self.source),
            ResourceId::from_asset(self.asset, self.destination)
        ]
    }
    
    fn required_capabilities(&self) -> Vec<CapabilityType> {
        // Report the capabilities required for this effect
        vec![
            CapabilityType::AssetTransfer
        ]
    }
}
```

### 3. Implement Cross-Domain Support (Optional)

If your effect needs to work across domains, add cross-domain support:

```rust
impl<R> CrossDomainEffect for TransferAssetEffect<R> {
    fn crosses_domains(&self) -> bool {
        // Check if source and destination are in different domains
        self.source.domain_id() != self.destination.domain_id()
    }
    
    fn involved_domains(&self) -> Vec<DomainId> {
        // Return the domains involved in this effect
        vec![
            self.source.domain_id(),
            self.destination.domain_id()
        ]
    }
}
```

### 4. Create Effect Factory

Add a factory function to simplify effect creation:

```rust
/// Create a transfer asset effect
pub fn transfer_asset<R, F>(
    source: AccountId,
    destination: AccountId,
    asset: AssetId,
    amount: u64,
    f: F
) -> TransferAssetEffect<R>
where
    F: FnOnce(TransferResult) -> R + Send + 'static
{
    TransferAssetEffect {
        source,
        destination,
        asset,
        amount,
        continuation: Box::new(FnContinuation::new(f)),
    }
}
```

## Implementing Effect Constraints

### 1. Create Resource Requirements Validator

```rust
/// Validate resource requirements for an effect
pub struct ResourceValidator {
    resource_manager: Arc<ResourceManager>,
}

impl ResourceValidator {
    /// Create a new resource validator
    pub fn new(resource_manager: Arc<ResourceManager>) -> Self {
        Self { resource_manager }
    }
    
    /// Validate that resources exist and are accessible
    pub async fn validate<E: Effect<R>, R>(
        &self,
        effect: &E,
        agent_id: &AgentId
    ) -> Result<(), ResourceError> {
        for resource_id in effect.resources() {
            // Check if resource exists
            if !self.resource_manager.exists(&resource_id).await? {
                return Err(ResourceError::ResourceNotFound(resource_id));
            }
            
            // Check if agent has access to resource
            if !self.resource_manager.has_access(&resource_id, agent_id).await? {
                return Err(ResourceError::AccessDenied(resource_id, agent_id.clone()));
            }
        }
        
        Ok(())
    }
}
```

### 2. Create Capability Validator

```rust
/// Validate capability requirements for an effect
pub struct CapabilityValidator {
    capability_registry: Arc<CapabilityRegistry>,
}

impl CapabilityValidator {
    /// Create a new capability validator
    pub fn new(capability_registry: Arc<CapabilityRegistry>) -> Self {
        Self { capability_registry }
    }
    
    /// Validate that the agent has the required capabilities
    pub async fn validate<E: Effect<R>, R>(
        &self,
        effect: &E,
        agent_id: &AgentId
    ) -> Result<(), CapabilityError> {
        // Get resources and required capabilities
        let resources = effect.resources();
        let capabilities = effect.required_capabilities();
        
        // Check each capability for each resource
        for resource_id in &resources {
            for capability_type in &capabilities {
                let verified = self.capability_registry.verify_capability(
                    agent_id,
                    resource_id,
                    capability_type
                ).await?;
                
                if !verified {
                    return Err(CapabilityError::MissingCapability(
                        agent_id.clone(),
                        resource_id.clone(),
                        *capability_type
                    ));
                }
            }
        }
        
        Ok(())
    }
}
```

### 3. Integrate Validators into Effect Execution

```rust
/// Composite effect validator
pub struct EffectValidator {
    resource_validator: ResourceValidator,
    capability_validator: CapabilityValidator,
    domain_validator: DomainValidator,
}

impl EffectValidator {
    /// Validate an effect
    pub async fn validate<E: Effect<R>, R>(
        &self,
        effect: &E,
        context: &EffectContext
    ) -> Result<(), EffectError> {
        // Get agent ID from context
        let agent_id = context.agent_id()
            .ok_or(EffectError::MissingAgentId)?;
        
        // Validate resources
        self.resource_validator.validate(effect, agent_id).await
            .map_err(EffectError::ResourceError)?;
        
        // Validate capabilities
        self.capability_validator.validate(effect, agent_id).await
            .map_err(EffectError::CapabilityError)?;
        
        // Validate domains (if cross-domain)
        if let Some(cross_domain) = effect.as_any().downcast_ref::<dyn CrossDomainEffect>() {
            if cross_domain.crosses_domains() {
                self.domain_validator.validate(cross_domain).await
                    .map_err(EffectError::DomainError)?;
            }
        }
        
        Ok(())
    }
}
```

## Implementing Domain Handlers

### 1. Create Effect Handler

```rust
/// Asset effect handler
pub struct AssetEffectHandler {
    asset_registry: Arc<AssetRegistry>,
}

impl AssetEffectHandler {
    /// Create a new asset effect handler
    pub fn new(asset_registry: Arc<AssetRegistry>) -> Self {
        Self { asset_registry }
    }
    
    /// Transfer asset between accounts
    pub fn transfer_asset(
        &self,
        source: AccountId,
        destination: AccountId,
        asset: AssetId,
        amount: u64
    ) -> Result<TransferResult, AssetError> {
        // Check if source has sufficient balance
        let source_balance = self.asset_registry.get_balance(&source, &asset)?;
        if source_balance < amount {
            return Err(AssetError::InsufficientBalance);
        }
        
        // Perform the transfer
        self.asset_registry.decrease_balance(&source, &asset, amount)?;
        self.asset_registry.increase_balance(&destination, &asset, amount)?;
        
        // Return result
        Ok(TransferResult {
            source,
            destination,
            asset,
            amount,
            timestamp: current_timestamp(),
        })
    }
}

impl EffectHandler for AssetEffectHandler {
    fn handle<R>(&self, effect: Box<dyn Effect<R>>) -> Result<EffectOutcome<R>, EffectError> {
        // Handle different effect types
        if let Some(transfer) = effect.downcast_ref::<TransferAssetEffect<R>>() {
            // Execute the transfer effect
            let result = self.transfer_asset(
                transfer.source.clone(),
                transfer.destination.clone(),
                transfer.asset.clone(),
                transfer.amount
            ).map_err(|e| EffectError::DomainError(Box::new(e)))?;
            
            // Apply continuation
            let value = transfer.continuation.apply(result);
            return Ok(EffectOutcome::Success(value));
        }
        
        // Return error for unsupported effects
        Err(EffectError::UnsupportedEffect)
    }
    
    fn supported_effects(&self) -> Vec<EffectType> {
        vec![
            EffectType::of::<TransferAssetEffect<()>>(),
        ]
    }
}
```

### 2. Create Domain Adapter

```rust
/// Ethereum domain adapter
pub struct EthereumDomainAdapter {
    domain_id: DomainId,
    provider: Arc<Provider>,
    asset_registry: Arc<AssetRegistry>,
}

impl EthereumDomainAdapter {
    /// Create a new Ethereum domain adapter
    pub fn new(
        domain_id: DomainId,
        provider: Arc<Provider>,
        asset_registry: Arc<AssetRegistry>
    ) -> Self {
        Self {
            domain_id,
            provider,
            asset_registry,
        }
    }
}

impl DomainAdapter for EthereumDomainAdapter {
    fn domain_id(&self) -> DomainId {
        self.domain_id.clone()
    }
    
    async fn handle_effect<R>(
        &self,
        effect: Box<dyn Effect<R>>,
        context: &EffectContext
    ) -> Result<EffectOutcome<R>, EffectError> {
        // Create appropriate handler based on effect type
        if effect.effect_type() == EffectType::of::<TransferAssetEffect<R>>() {
            let handler = AssetEffectHandler::new(self.asset_registry.clone());
            return handler.handle(effect);
        }
        
        // Handle other effect types...
        
        Err(EffectError::UnsupportedEffect)
    }
    
    fn can_handle(&self, effect_type: &EffectType) -> bool {
        // Check if this domain can handle the effect type
        let supported = [
            EffectType::of::<TransferAssetEffect<()>>(),
            // Add other supported effect types...
        ];
        
        supported.contains(effect_type)
    }
    
    fn resource_accessor(&self) -> Box<dyn ResourceAccessor> {
        Box::new(EthereumResourceAccessor::new(
            self.provider.clone(),
            self.asset_registry.clone()
        ))
    }
}
```

### 3. Register Domain Adapter

```rust
/// Register domain adapters in the registry
pub fn register_domain_adapters(
    registry: &mut DomainAdapterRegistry,
    provider: Arc<Provider>,
    asset_registry: Arc<AssetRegistry>
) {
    // Create Ethereum domain adapter
    let ethereum_adapter = EthereumDomainAdapter::new(
        DomainId::new("ethereum"),
        provider,
        asset_registry
    );
    
    // Register the adapter
    registry.register(ethereum_adapter);
    
    // Register other domain adapters...
}
```

## Using Effects in Your Code

### 1. Basic Effect Execution

```rust
/// Execute a transfer effect
pub async fn transfer_asset(
    source: AccountId,
    destination: AccountId,
    asset: AssetId,
    amount: u64,
    effect_engine: &EffectEngine,
    context: &EffectContext
) -> Result<TransferResult, EffectError> {
    // Create the effect
    let effect = transfer_asset(
        source,
        destination,
        asset,
        amount,
        identity
    );
    
    // Execute the effect
    effect_engine.execute(effect, context).await
}
```

### 2. Composing Effects

```rust
/// Transfer assets and notify recipient
pub async fn transfer_and_notify(
    source: AccountId,
    destination: AccountId,
    asset: AssetId,
    amount: u64,
    message: String,
    effect_engine: &EffectEngine,
    context: &EffectContext
) -> Result<NotificationResult, EffectError> {
    // Create composed effect
    let effect = transfer_asset(
        source,
        destination,
        asset,
        amount,
        move |transfer_result| {
            // After transfer succeeds, send notification
            send_notification(
                destination,
                format!("Received {} {}: {}", amount, asset, message),
                |notification_result| notification_result
            )
        }
    );
    
    // Execute the composed effect
    effect_engine.execute(effect, context).await
}
```

### 3. Parallel Effects

```rust
/// Transfer assets to multiple recipients
pub async fn batch_transfer(
    source: AccountId,
    destinations: Vec<(AccountId, u64)>,
    asset: AssetId,
    effect_engine: &EffectEngine,
    context: &EffectContext
) -> Result<Vec<TransferResult>, EffectError> {
    // Create effects for each transfer
    let mut effects = Vec::new();
    for (destination, amount) in destinations {
        effects.push(transfer_asset(
            source.clone(),
            destination,
            asset.clone(),
            amount,
            identity
        ));
    }
    
    // Combine effects to run in parallel
    let parallel_effect = parallel_all(effects, |results| results);
    
    // Execute the parallel effect
    effect_engine.execute(parallel_effect, context).await
}
```

## Cross-Domain Effects

### 1. Implementing a Cross-Domain Transfer

```rust
/// Cross-domain asset transfer
pub async fn cross_domain_transfer(
    source: AccountId,
    destination: AccountId,
    asset: AssetId,
    amount: u64,
    effect_engine: &EffectEngine,
    context: &EffectContext
) -> Result<CrossDomainTransferResult, EffectError> {
    // Create cross-domain transfer effect
    let effect = cross_domain_transfer(
        source,
        destination,
        asset,
        amount,
        identity
    );
    
    // Execute the effect
    effect_engine.execute(effect, context).await
}
```

### 2. Cross-Domain Effect Handler

```rust
/// Handle cross-domain effects
pub struct CrossDomainHandler {
    domain_registry: Arc<DomainRegistry>,
    bridge: Arc<DomainBridge>,
}

impl EffectHandler for CrossDomainHandler {
    fn handle<R>(&self, effect: Box<dyn Effect<R>>) -> Result<EffectOutcome<R>, EffectError> {
        if let Some(transfer) = effect.downcast_ref::<CrossDomainTransferEffect<R>>() {
            // 1. Lock assets in source domain
            let source_domain = self.domain_registry.get_domain(&transfer.source.domain_id())?;
            let lock_result = source_domain.lock_asset(
                transfer.source.clone(),
                transfer.asset.clone(),
                transfer.amount
            )?;
            
            // 2. Generate proof of lock
            let proof = self.bridge.generate_proof(
                lock_result.lock_id,
                transfer.source.domain_id(),
                transfer.destination.domain_id()
            )?;
            
            // 3. Create assets in destination domain
            let dest_domain = self.domain_registry.get_domain(&transfer.destination.domain_id())?;
            let mint_result = dest_domain.mint_with_proof(
                transfer.destination.clone(),
                transfer.asset.clone(),
                transfer.amount,
                proof
            )?;
            
            // 4. Create result and apply continuation
            let result = CrossDomainTransferResult {
                source: transfer.source.clone(),
                destination: transfer.destination.clone(),
                asset: transfer.asset.clone(),
                amount: transfer.amount,
                source_lock_id: lock_result.lock_id,
                destination_mint_id: mint_result.mint_id,
            };
            
            let value = transfer.continuation.apply(result);
            return Ok(EffectOutcome::Success(value));
        }
        
        Err(EffectError::UnsupportedEffect)
    }
}
```

## ZK Integration

### 1. Creating ZK-Compatible Effects

```rust
/// ZK-enabled transfer effect
pub struct ZkTransferEffect<R> {
    source: AccountId,
    destination: AccountId,
    asset: AssetId,
    amount: u64,
    zk_proof: Option<Proof>,
    continuation: Box<dyn Continuation<TransferResult, R>>,
}

impl<R> Effect<R> for ZkTransferEffect<R> {
    // Standard effect implementation
}

impl<R> ZkEffect<R> for ZkTransferEffect<R> {
    fn generate_circuit(&self) -> Result<Circuit, CircuitError> {
        // Generate a ZK circuit for the transfer
        let mut circuit = Circuit::new("transfer");
        
        // Add circuit constraints
        circuit.add_input("source", self.source.to_bytes());
        circuit.add_input("destination", self.destination.to_bytes());
        circuit.add_input("asset", self.asset.to_bytes());
        circuit.add_input("amount", self.amount.to_le_bytes());
        
        // Add transfer constraints
        circuit.add_constraint("valid_source", /* constraint definition */);
        circuit.add_constraint("sufficient_balance", /* constraint definition */);
        circuit.add_constraint("valid_destination", /* constraint definition */);
        
        Ok(circuit)
    }
    
    fn generate_witness(&self, inputs: &[Value]) -> Result<Witness, WitnessError> {
        // Generate a witness for the circuit
        let mut witness = Witness::new();
        
        // Add required values
        witness.add("source_pk", /* source public key */);
        witness.add("source_balance", /* source balance */);
        witness.add("destination_pk", /* destination public key */);
        
        Ok(witness)
    }
}
```

### 2. ZK Effect Handler

```rust
/// Handler for ZK effects
pub struct ZkEffectHandler {
    prover: Arc<Prover>,
    verifier: Arc<Verifier>,
    underlying_handler: Box<dyn EffectHandler>,
}

impl EffectHandler for ZkEffectHandler {
    fn handle<R>(&self, effect: Box<dyn Effect<R>>) -> Result<EffectOutcome<R>, EffectError> {
        // Check if effect implements ZkEffect
        if let Some(zk_effect) = effect.as_any().downcast_ref::<dyn ZkEffect<R>>() {
            // Generate circuit
            let circuit = zk_effect.generate_circuit()?;
            
            // Generate witness
            let witness = zk_effect.generate_witness(&[])?;
            
            // Generate proof
            let proof = self.prover.generate_proof(&circuit, &witness)?;
            
            // Verify proof
            if !self.verifier.verify_proof(&circuit, &proof, &[])? {
                return Err(EffectError::InvalidProof);
            }
            
            // Delegate to underlying handler
            return self.underlying_handler.handle(effect);
        } else {
            // Not a ZK effect, delegate directly
            self.underlying_handler.handle(effect)
        }
    }
}
```

## Best Practices

### Organizing Effect Code

Structure your effect-related code following the three-layer architecture:

```
src/
  effects/
    # Algebraic Layer
    mod.rs              # Public exports
    traits.rs           # Effect trait definitions
    continuations.rs    # Continuation implementations
    composition.rs      # Effect composition operators
    
    # Constraints Layer
    constraints/
      mod.rs            # Public exports
      resource.rs       # Resource requirement validation
      capability.rs     # Capability validation
      cross_domain.rs   # Cross-domain validation
      
    # Domain Layer
    handlers/
      mod.rs            # Public exports
      asset.rs          # Asset effect handlers
      ethereum.rs       # Ethereum domain adapter
      cosmwasm.rs       # CosmWasm domain adapter
```

### Effect Naming Conventions

Follow these naming conventions for effects:

- **Effect Structs**: Use `VerbNounEffect` naming pattern (e.g., `TransferAssetEffect`, `QueryBalanceEffect`)
- **Effect Factories**: Use `verb_noun` naming pattern (e.g., `transfer_asset`, `query_balance`)
- **Handlers**: Use `NounEffectHandler` naming pattern (e.g., `AssetEffectHandler`, `BalanceEffectHandler`)
- **Adapters**: Use `NounDomainAdapter` naming pattern (e.g., `EthereumDomainAdapter`, `CosmWasmDomainAdapter`)

### Error Handling

Follow these best practices for error handling:

1. **Specific Error Types**: Define domain-specific error types that implement `std::error::Error`
2. **Error Wrapping**: Wrap domain errors in `EffectError` when crossing layer boundaries
3. **Error Propagation**: Use the `?` operator to propagate errors up the call stack
4. **Error Contexts**: Include context information in errors to aid debugging

```rust
/// Effect-specific error
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Asset {0} not found")]
    AssetNotFound(AssetId),
    
    #[error("Account {0} not found")]
    AccountNotFound(AccountId),
}

/// System-wide effect error
#[derive(Debug, thiserror::Error)]
pub enum EffectError {
    #[error("Missing required capability")]
    CapabilityError(#[from] CapabilityError),
    
    #[error("Resource error: {0}")]
    ResourceError(#[from] ResourceError),
    
    #[error("Domain error: {0}")]
    DomainError(#[source] Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Effect not supported")]
    UnsupportedEffect,
}
```

### Testing Effects

Test each layer of the effect architecture separately:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Test the algebraic layer
    #[test]
    fn test_transfer_effect_creation() {
        let effect = transfer_asset(
            AccountId::from_str("alice").unwrap(),
            AccountId::from_str("bob").unwrap(),
            AssetId::from_str("token").unwrap(),
            100,
            |result| result
        );
        
        assert_eq!(effect.amount, 100);
        assert_eq!(effect.resources().len(), 2);
    }
    
    // Test the constraints layer
    #[tokio::test]
    async fn test_capability_validation() {
        let registry = setup_test_capability_registry();
        let validator = CapabilityValidator::new(Arc::new(registry));
        
        let effect = transfer_asset(
            AccountId::from_str("alice").unwrap(),
            AccountId::from_str("bob").unwrap(),
            AssetId::from_str("token").unwrap(),
            100,
            identity
        );
        
        let result = validator.validate(
            &effect,
            &AgentId::from_str("alice").unwrap()
        ).await;
        
        assert!(result.is_ok());
    }
    
    // Test the domain layer
    #[tokio::test]
    async fn test_asset_effect_handler() {
        let registry = setup_test_asset_registry();
        let handler = AssetEffectHandler::new(Arc::new(registry));
        
        let effect = Box::new(transfer_asset(
            AccountId::from_str("alice").unwrap(),
            AccountId::from_str("bob").unwrap(),
            AssetId::from_str("token").unwrap(),
            100,
            identity
        ));
        
        let outcome = handler.handle(effect).unwrap();
        
        match outcome {
            EffectOutcome::Success(result) => {
                assert_eq!(result.amount, 100);
            },
            _ => panic!("Expected success outcome"),
        }
    }
}
```

## Troubleshooting

### Common Issues

1. **Unsupported Effect Error**: Check if the correct handler is registered for the effect type
2. **Missing Capability Error**: Ensure the agent has the required capabilities for the operation
3. **Resource Not Found Error**: Verify that all resources exist and are accessible
4. **Domain Not Found Error**: Check that the domain adapter is registered correctly
5. **Effect Execution Timeout**: Review effect handlers for potential deadlocks or long-running operations

### Debugging Tips

1. **Effect Tracing**: Enable effect tracing to see the execution flow
   ```rust
   let effect_engine = EffectEngine::builder()
       .with_tracing(true)
       .build();
   ```

2. **Effect Composition Debugging**: Break down complex compositions into smaller parts
   ```rust
   // Break this:
   let complex = effect1.and_then(|r1| 
       effect2.and_then(|r2| 
           effect3.map(|r3| (r1, r2, r3))
       )
   );
   
   // Into this:
   let step1 = effect1;
   let step2 = effect2;
   let step3 = effect3;
   
   // Execute step by step
   let result1 = effect_engine.execute(step1, context).await?;
   let result2 = effect_engine.execute(step2, context).await?;
   let result3 = effect_engine.execute(step3, context).await?;
   ```

3. **Resource Lock Debugging**: Check for potential deadlocks in resource acquisition
   ```rust
   // Enable deadlock detection
   let resource_manager = ResourceManager::builder()
       .with_deadlock_detection(true)
       .build();
   ```

## References

- [Three-Layer Effect Architecture](../../architecture/core/three-layer-effect-architecture.md)
- [Effect System](../../architecture/core/effect-system.md)
- [Domain System](../../architecture/core/domain-system.md)
- [API Reference](../../reference/libraries/causality-core.md) 