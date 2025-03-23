# ADR-023: Three-Layer Effect Architecture with TEL Integration

## Status

Proposed

## Context

Currently, the Causality system implements two separate but related abstractions for handling effects:

1. **Effect Handlers**: Internal mechanisms that process effects within the Causality system
2. **Domain Adapters**: External interfaces that translate effects into domain-specific operations

This separation creates several challenges:

- **Duplicated Logic**: Similar code appears in both handlers and adapters
- **Complex Extension Path**: Adding a new effect requires changes in multiple places:
  - Effect definition in the core library
  - Handler implementation for internal processing
  - Adapter implementations for each supported domain
  - Schema definitions for code generation
  - Helper functions for common patterns

- **Fragmented Responsibility**: Unclear where certain functionality should live
- **Limited Composability**: Difficult to compose operations from simpler primitives
- **High Maintenance Burden**: Changes to effect behavior often require updates in multiple places

The TEL (Temporal Effect Language) was intended to simplify this process, but developers still face significant overhead when adding new effects, particularly for cross-domain operations. The current architecture does not sufficiently leverage Rust's type system in combination with TEL's domain-specific capabilities.

## Decision

We will redesign our effect system as a three-layer architecture that cleanly separates abstractions while unifying the previously disparate mechanisms:

1. **Algebraic Effect Layer** (Rust): Core effect abstractions and interfaces
2. **Effect Constraints Layer** (Rust): Type constraints and validation rules
3. **Domain Implementation Layer** (TEL): Domain-specific implementations

This approach leverages Rust's trait system for the first two layers and TEL for the third layer, providing a natural fit for each aspect of the system.

### Data Structure Unification

The following existing data structures will be unified:

| Current Structure | New Structure | Notes |
|------------------|---------------|-------|
| `EffectHandler` | Replaced by `EffectConstraints` traits | Handlers become constraint traits |
| `DomainAdapter` | Replaced by TEL implementations | Domain logic moves to TEL |
| `EffectDefinition` | Unified into `Effect` trait | Core abstraction |
| `EffectSchema` | Eliminated | Schema replaced by type constraints |
| `AdapterSchema` | Eliminated | Implementation details move to TEL |
| `ValidationRule` | Moved to constraint traits | Validation becomes methods |
| `EffectRegistry` | Simplified to `EffectRuntime` | Runtime executor for effects |

This unification reduces the number of concepts developers need to understand while providing stronger compile-time guarantees.

### Core Components

#### 1. Algebraic Effect Layer (First Layer)

```rust
/// Base trait for all effects
pub trait Effect: Send + Sync {
    /// The output type of this effect
    type Output;
    
    /// Get the type of this effect
    fn get_type(&self) -> EffectType;
    
    /// Get the domains this effect interacts with
    fn domains(&self) -> Vec<DomainId>;
    
    /// Get the resources this effect uses
    fn resources(&self) -> Vec<ResourceId>;
    
    /// Get fact dependencies for this effect
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        Vec::new()
    }
    
    /// Get the fact snapshot for this effect
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        None
    }
}

/// Core effect types in the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectType {
    Deposit,
    Withdraw,
    Transfer,
    Observe,
    Invoke,
    // Other effect types...
}
```

#### 2. Effect Constraints Layer (Second Layer)

```rust
/// Constraints for transfer effects
pub trait TransferEffect: Effect {
    /// The source of the transfer
    fn from(&self) -> ResourceId;
    
    /// The destination of the transfer
    fn to(&self) -> ResourceId;
    
    /// The amount being transferred
    fn quantity(&self) -> u64;
    
    /// The domain in which the transfer occurs
    fn domain(&self) -> DomainId;
    
    /// Validate that the source exists
    fn validate_source_exists(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        if !context.resource_exists(self.from()) {
            return Err(ValidationError::ResourceNotFound(self.from()));
        }
        Ok(())
    }
    
    /// Validate that the destination exists
    fn validate_destination_exists(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        if !context.resource_exists(self.to()) {
            return Err(ValidationError::ResourceNotFound(self.to()));
        }
        Ok(())
    }
    
    /// Validate that the source has sufficient balance
    fn validate_sufficient_balance(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        let balance = context.resource_balance(self.from());
        if balance < self.quantity() {
            return Err(ValidationError::InsufficientBalance {
                resource: self.from(),
                required: self.quantity(),
                available: balance,
            });
        }
        Ok(())
    }
    
    /// Verify conservation rules
    fn verify_conservation(&self, before: &ResourceState, after: &ResourceState) -> Result<(), ValidationError> {
        // Source balance should decrease by the transfer amount
        if before.get_balance(self.from()) - self.quantity() != after.get_balance(self.from()) {
            return Err(ValidationError::ConservationViolation {
                resource: self.from(),
                expected: before.get_balance(self.from()) - self.quantity(),
                actual: after.get_balance(self.from()),
            });
        }
        
        // Destination balance should increase by the transfer amount
        if before.get_balance(self.to()) + self.quantity() != after.get_balance(self.to()) {
            return Err(ValidationError::ConservationViolation {
                resource: self.to(),
                expected: before.get_balance(self.to()) + self.quantity(),
                actual: after.get_balance(self.to()),
            });
        }
        
        Ok(())
    }
    
    /// Run all validations
    fn validate(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        self.validate_source_exists(context)?;
        self.validate_destination_exists(context)?;
        self.validate_sufficient_balance(context)?;
        
        // Domain-specific validations will be defined in implementations
        
        Ok(())
    }
}

/// Constraints for deposit effects
pub trait DepositEffect: Effect {
    // Similar constraint methods for deposits
    fn asset(&self) -> ResourceId;
    fn quantity(&self) -> u64;
    fn destination(&self) -> Address;
    fn domain(&self) -> DomainId;
    
    // Validation methods...
}

/// Constraints for withdrawal effects
pub trait WithdrawEffect: Effect {
    // Similar constraint methods for withdrawals
    fn asset(&self) -> ResourceId;
    fn quantity(&self) -> u64;
    fn source(&self) -> Address;
    fn domain(&self) -> DomainId;
    
    // Validation methods...
}
```

#### 3. Domain Implementation Layer (Third Layer - TEL)

TEL code for implementing a transfer on Ethereum:

```rust
effect EthereumTransfer implements TransferEffect {
    // State fields
    from: ResourceId
    to: ResourceId
    quantity: u64
    domain: DomainId
    
    // Implementation of required accessors
    fn from() -> ResourceId { return this.from; }
    fn to() -> ResourceId { return this.to; }
    fn quantity() -> u64 { return this.quantity; }
    fn domain() -> DomainId { return this.domain; }
    
    // Domain-specific validation
    fn validate_ethereum_gas(context) -> Result<(), ValidationError> {
        let current_gas = context.observe("ethereum.gas_price");
        require(current_gas < 100, "Gas price too high");
        return Ok(());
    }
    
    // Execution logic
    fn execute(context) -> Result<TransactionHash, EffectError> {
        // Get Ethereum client from context
        let client = ethereum_client(context.domain);
        
        // Build transaction
        let tx = {
            from: address_from_resource(this.from()),
            to: address_from_resource(this.to()),
            value: this.quantity(),
            gas: estimate_gas(this)
        };
        
        // Submit transaction
        let receipt = client.send_transaction(tx);
        if !receipt.success {
            return Err(EffectError::TransactionFailed(receipt.error));
        }
        
        // Return transaction hash
        return Ok(TransactionHash(receipt.hash));
    }
}
```

### Effect Runtime

The Effect Runtime is responsible for executing effects by finding the appropriate TEL implementation and running it:

```rust
/// Runtime for executing effects
pub struct EffectRuntime {
    tel_compiler: TelCompiler,
    tel_runtime: TelRuntime,
    effect_implementations: HashMap<(TypeId, DomainType), ImplementationInfo>,
}

impl EffectRuntime {
    /// Create a new effect runtime
    pub fn new() -> Self {
        // Initialization code
    }
    
    /// Register a TEL implementation for an effect type
    pub fn register_implementation<E: Effect + 'static>(
        &mut self,
        domain_type: DomainType,
        tel_code: &str,
    ) -> Result<(), RegistrationError> {
        // Parse and compile TEL code
        let compiled = self.tel_compiler.compile(tel_code)?;
        
        // Verify it implements the correct trait for E
        self.verify_implementation::<E>(&compiled)?;
        
        // Store the implementation
        let type_id = TypeId::of::<E>();
        self.effect_implementations.insert(
            (type_id, domain_type),
            ImplementationInfo {
                compiled_code: compiled,
                domain_type,
                effect_type: std::any::type_name::<E>().to_string(),
            },
        );
        
        Ok(())
    }
    
    /// Execute an effect
    pub async fn execute<E: Effect>(
        &self,
        effect: &E,
        context: &ExecutionContext,
    ) -> Result<E::Output, EffectError> {
        // Get the domain type
        let domain = effect.domains().first().ok_or(EffectError::NoDomain)?;
        let domain_type = domain.domain_type();
        
        // Find the implementation
        let type_id = TypeId::of::<E>();
        let implementation = self.effect_implementations
            .get(&(type_id, domain_type))
            .ok_or(EffectError::NoImplementation {
                effect_type: std::any::type_name::<E>().to_string(),
                domain_type: domain_type.to_string(),
            })?;
        
        // Create execution environment
        let mut env = self.tel_runtime.create_environment();
        
        // Prepare effect data for TEL
        env.set_input("effect", effect);
        env.set_input("context", context);
        
        // Execute the TEL code
        let result = self.tel_runtime.execute(&implementation.compiled_code, &env).await?;
        
        // Convert result to expected type
        let output = result.try_into()?;
        
        Ok(output)
    }
    
    /// Validate an effect
    pub fn validate<E: Effect + TransferEffect>(
        &self,
        effect: &E,
        context: &ValidationContext,
    ) -> Result<(), ValidationError> {
        // Core validations from trait
        effect.validate(context)?;
        
        // Domain-specific validations from TEL
        let domain = effect.domain();
        let domain_type = domain.domain_type();
        
        // Find the implementation
        let type_id = TypeId::of::<E>();
        let implementation = self.effect_implementations
            .get(&(type_id, domain_type))
            .ok_or(ValidationError::NoImplementation {
                effect_type: std::any::type_name::<E>().to_string(),
                domain_type: domain_type.to_string(),
            })?;
        
        // Create validation environment
        let mut env = self.tel_runtime.create_environment();
        
        // Prepare effect data for TEL
        env.set_input("effect", effect);
        env.set_input("context", context);
        
        // Execute validation in TEL
        let result = self.tel_runtime.call_function(
            &implementation.compiled_code,
            "validate_ethereum_gas",
            &env,
        )?;
        
        Ok(())
    }
}
```

### Effect Execution Flow

1. **Create a concrete effect instance**:
   ```rust
   // Create a transfer effect
   let transfer = EthereumTransfer {
       from: alice_resource,
       to: bob_resource,
       quantity: 100,
       domain: ethereum_domain,
   };
   ```

2. **Validate the effect using static constraints and domain rules**:
   ```rust
   // Validate the transfer
   effect_runtime.validate(&transfer, &validation_context)?;
   ```

3. **Execute the effect using the TEL implementation**:
   ```rust
   // Execute the transfer
   let result = effect_runtime.execute(&transfer, &execution_context).await?;
   ```

4. **Process the result**:
   ```rust
   // Handle the result
   match result {
       Ok(tx_hash) => println!("Transfer successful: {}", tx_hash),
       Err(e) => println!("Transfer failed: {}", e),
   }
   ```

## Complete Example

Let's walk through a complete example for implementing a token transfer across three domains (Ethereum, Solana, and CosmWasm) using our three-layer architecture.

### 1. Algebraic Effect Layer (Rust)

```rust
// Core effect trait
pub trait Effect: Send + Sync {
    type Output;
    fn get_type(&self) -> EffectType;
    fn domains(&self) -> Vec<DomainId>;
    fn resources(&self) -> Vec<ResourceId>;
    fn fact_dependencies(&self) -> Vec<FactDependency> { Vec::new() }
    fn fact_snapshot(&self) -> Option<FactSnapshot> { None }
}

// The "Transfer" effect type is part of our effect enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectType {
    Transfer,
    // Other types...
}
```

### 2. Effect Constraints Layer (Rust)

```rust
// Transfer specific constraints
pub trait TransferEffect: Effect {
    // Required accessor methods
    fn from(&self) -> ResourceId;
    fn to(&self) -> ResourceId;
    fn quantity(&self) -> u64;
    fn domain(&self) -> DomainId;
    
    // Default validation methods that can be overridden
    fn validate_source_exists(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        // Default implementation...
    }
    
    fn validate_destination_exists(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        // Default implementation...
    }
    
    fn validate_sufficient_balance(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        // Default implementation...
    }
    
    fn verify_conservation(&self, before: &ResourceState, after: &ResourceState) -> Result<(), ValidationError> {
        // Default implementation...
    }
    
    // Generic validation runner
    fn validate(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        // Run standard validations
        self.validate_source_exists(context)?;
        self.validate_destination_exists(context)?;
        self.validate_sufficient_balance(context)?;
        
        Ok(())
    }
}

// Concrete implementation for Ethereum
pub struct EthereumTransfer {
    pub from: ResourceId,
    pub to: ResourceId,
    pub quantity: u64,
    pub domain: DomainId,
}

// Implement the core Effect trait
impl Effect for EthereumTransfer {
    type Output = EthTransactionHash;
    
    fn get_type(&self) -> EffectType {
        EffectType::Transfer
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.domain.clone()]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.from.clone(), self.to.clone()]
    }
}

// Implement the constraint trait
impl TransferEffect for EthereumTransfer {
    fn from(&self) -> ResourceId { self.from.clone() }
    fn to(&self) -> ResourceId { self.to.clone() }
    fn quantity(&self) -> u64 { self.quantity }
    fn domain(&self) -> DomainId { self.domain.clone() }
}

// Concrete implementation for Solana
pub struct SolanaTransfer {
    pub from: ResourceId,
    pub to: ResourceId,
    pub quantity: u64,
    pub domain: DomainId,
}

// Implement Effect trait
impl Effect for SolanaTransfer {
    type Output = SolTransactionSignature;
    
    fn get_type(&self) -> EffectType {
        EffectType::Transfer
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.domain.clone()]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.from.clone(), self.to.clone()]
    }
}

// Implement the constraint trait
impl TransferEffect for SolanaTransfer {
    fn from(&self) -> ResourceId { self.from.clone() }
    fn to(&self) -> ResourceId { self.to.clone() }
    fn quantity(&self) -> u64 { self.quantity }
    fn domain(&self) -> DomainId { self.domain.clone() }
    
    // Override validation for Solana-specific rules
    fn validate_sufficient_balance(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        // Solana requires a minimum balance for rent exemption
        let balance = context.resource_balance(self.from());
        let min_balance = context.rent_exempt_minimum(ResourceType::Token);
        
        if balance < self.quantity() + min_balance {
            return Err(ValidationError::InsufficientBalance {
                resource: self.from(),
                required: self.quantity() + min_balance,
                available: balance,
            });
        }
        
        Ok(())
    }
}
```

### 3. Domain Implementation Layer (TEL)

Ethereum implementation in TEL:

```rust
effect EthereumTransferImpl for EthereumTransfer {
    // Execute method
    fn execute(context) -> Result<EthTransactionHash, EffectError> {
        // Get Ethereum client
        let client = ethereum_client(this.domain());
        
        // Convert resource IDs to Ethereum addresses
        let from_address = resource_to_eth_address(this.from());
        let to_address = resource_to_eth_address(this.to());
        
        // Build transaction
        let tx = {
            from: from_address,
            to: to_address,
            value: this.quantity(),
            gas: 21000,
            gasPrice: context.gas_price()
        };
        
        // Send transaction
        let receipt = client.send_transaction(tx);
        
        // Return result
        if receipt.success {
            return Ok(EthTransactionHash(receipt.hash));
        } else {
            return Err(EffectError::TransactionFailed(receipt.error));
        }
    }
    
    // Custom Ethereum validation
    fn validate_gas_price(context) -> Result<(), ValidationError> {
        let gas_price = context.get("ethereum.gas_price");
        let max_gas = context.get("ethereum.max_gas_price");
        
        if gas_price > max_gas {
            return Err(ValidationError::GasToHigh {
                current: gas_price,
                maximum: max_gas
            });
        }
        
        return Ok(());
    }
}
```

Solana implementation in TEL:

```rust
effect SolanaTransferImpl for SolanaTransfer {
    // Execute method
    fn execute(context) -> Result<SolTransactionSignature, EffectError> {
        // Get Solana client
        let client = solana_client(this.domain());
        
        // Convert resource IDs to Solana public keys
        let from_key = resource_to_sol_pubkey(this.from());
        let to_key = resource_to_sol_pubkey(this.to());
        
        // Get token program
        let token_program = context.get("solana.token_program");
        
        // Build instruction
        let ix = {
            program_id: token_program,
            accounts: [
                {pubkey: from_key, is_signer: true, is_writable: true},
                {pubkey: to_key, is_signer: false, is_writable: true},
                {pubkey: context.rent.pubkey(), is_signer: false, is_writable: false}
            ],
            data: encode_transfer_instruction(this.quantity())
        };
        
        // Build and send transaction
        let tx = {instructions: [ix]};
        let signature = client.send_transaction(tx);
        
        // Return result
        if signature.success {
            return Ok(SolTransactionSignature(signature.value));
        } else {
            return Err(EffectError::TransactionFailed(signature.error));
        }
    }
    
    // Custom Solana validation
    fn validate_rent_exemption(context) -> Result<(), ValidationError> {
        let min_balance = context.get("solana.rent_exempt_minimum");
        let account_balance = context.get_balance(this.from());
        
        if account_balance - this.quantity() < min_balance {
            return Err(ValidationError::InsufficientRentExemption {
                required: min_balance,
                remaining: account_balance - this.quantity()
            });
        }
        
        return Ok(());
    }
}
```

CosmWasm implementation in TEL:

```rust
effect CosmWasmTransferImpl for CosmWasmTransfer {
    // Execute method
    fn execute(context) -> Result<CosmosTransactionHash, EffectError> {
        // Get Cosmos client
        let client = cosmos_client(this.domain());
        
        // Convert resource IDs to Cosmos addresses
        let from_address = resource_to_cosmos_address(this.from());
        let to_address = resource_to_cosmos_address(this.to());
        
        // Build message
        let msg = {
            type: "cosmos-sdk/MsgSend",
            value: {
                from_address: from_address,
                to_address: to_address,
                amount: [
                    {
                        denom: resource_to_denom(this.from()),
                        amount: this.quantity().to_string()
                    }
                ]
            }
        };
        
        // Send transaction
        let result = client.broadcast_tx(msg);
        
        // Return result
        if result.code == 0 {
            return Ok(CosmosTransactionHash(result.txhash));
        } else {
            return Err(EffectError::TransactionFailed(result.raw_log));
        }
    }
    
    // Custom CosmWasm validation
    fn validate_chain_id(context) -> Result<(), ValidationError> {
        let chain_id = context.get("cosmos.chain_id");
        let expected_id = context.get("cosmos.expected_chain_id");
        
        if chain_id != expected_id {
            return Err(ValidationError::ChainIdMismatch {
                expected: expected_id,
                actual: chain_id
            });
        }
        
        return Ok(());
    }
}
```

### Using the Effect System

Here's how you would use this system in practice:

```rust
// Initialize the runtime
let mut effect_runtime = EffectRuntime::new();

// Register TEL implementations
effect_runtime.register_implementation::<EthereumTransfer>(
    DomainType::Ethereum,
    ETHEREUM_TRANSFER_TEL_CODE
)?;

effect_runtime.register_implementation::<SolanaTransfer>(
    DomainType::Solana,
    SOLANA_TRANSFER_TEL_CODE
)?;

effect_runtime.register_implementation::<CosmWasmTransfer>(
    DomainType::CosmWasm,
    COSMWASM_TRANSFER_TEL_CODE
)?;

// Create transfer effects for different domains
let eth_transfer = EthereumTransfer {
    from: eth_alice_resource,
    to: eth_bob_resource,
    quantity: 100,
    domain: ethereum_domain,
};

let sol_transfer = SolanaTransfer {
    from: sol_alice_resource,
    to: sol_bob_resource,
    quantity: 50,
    domain: solana_domain,
};

// Execute transfers
let eth_result = effect_runtime.execute(&eth_transfer, &execution_context).await?;
let sol_result = effect_runtime.execute(&sol_transfer, &execution_context).await?;

println!("Ethereum transfer completed with hash: {}", eth_result);
println!("Solana transfer completed with signature: {}", sol_result);
```

## Consequences

### Positive

1. **Natural Fit for Rust**: The trait-based approach leverages Rust's type system for strong compile-time guarantees.
2. **Domain-Specific Power**: TEL provides specialized syntax for domain-specific implementations while maintaining type safety.
3. **Clear Separation of Concerns**: The three-layer architecture separates abstractions, constraints, and implementations.
4. **Explicit Contracts**: Constraint traits provide clear interfaces that must be implemented.
5. **Enhanced Debuggability**: No macro magic means better debugging experience and IDE support.
6. **Reduced Duplication**: Unifying handlers and adapters eliminates duplicated code.
7. **Type-Safe Composition**: Effects can be composed through Rust's type system.
8. **TEL Integration**: Leverages the expressiveness of TEL where it makes most sense.

### Negative

1. **Initial Refactoring Effort**: Substantial code changes required to implement the three-layer model.
2. **TEL Compilation Overhead**: Runtime compilation of TEL adds complexity.
3. **More Boilerplate**: More explicit trait implementations compared to macro-based approach.
4. **Learning Curve**: Developers must understand both Rust traits and TEL.

### Neutral

1. **API Changes**: Public APIs will need to be updated to use the three-layer model.
2. **Documentation Updates**: Comprehensive updates required to reflect new patterns.
3. **Migration Period**: During transition, both old and new systems may need to coexist.

## Implementation Plan

1. **Phase 1**: Define the core traits and interfaces
   - Create base `Effect` trait
   - Implement constraint traits for common effect types
   - Set up the TEL integration layer

2. **Phase 2**: Build the EffectRuntime
   - Implement TEL compilation and execution
   - Create registration and discovery mechanisms
   - Implement validation pipeline

3. **Phase 3**: Convert existing effects to the new model
   - Start with simple effects
   - Gradually convert more complex effects
   - Update tests to use the new model

4. **Phase 4**: Build cross-domain capabilities
   - Implement cross-domain effect composition
   - Add support for multi-domain effects
   - Build tools for visualizing effect flows

5. **Phase 5**: Optimize performance
   - Implement TEL caching
   - Add JIT compilation for frequently used effects
   - Optimize validation pipelines

## References

1. [ADR-001: Effects Library](./adr_001_effects.md)
2. [ADR-002: Effect Adapters](./adr_002_effect_adapters.md)
3. [ADR-016: Temporal Effect Language](./adr_016_tel.md)