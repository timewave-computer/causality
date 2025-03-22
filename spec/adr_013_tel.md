# ADR-013: Temporal Effect Language

## Status

Accepted

## Context

Causality programs need to express complex **effects across domains**. Currently, effects are expressed using several incompatible approaches:

1. Ad-hoc JSON structures
2. Serialized protocol buffers
3. Domain-specific formats

This has led to:

- Complex integration code in adapters
- Inconsistent error handling
- Difficulties in testing and verification
- Limited expressiveness for cross-domain operations

## Decision

We will implement a unified **Temporal Effect Language (TEL)** in Rust that can express effects across domains. TEL will be:

1. **Declarative**: Focus on what effects should happen, not how
2. **Domain-agnostic**: Same language for all domains
3. **Strongly typed**: Catch errors at compile time
4. **Composable**: Build complex effects from simpler ones
5. **Resource-aware**: Express resource operations and ZK proofs

### Core Language Features

```rust
//! Temporal Effect Language core types and operations
use std::collections::HashMap;

/// Domain identifier
pub type DomainId = String;
/// Asset identifier
pub type AssetId = String;
/// Amount with precision
pub type Amount = u128;
/// Address on a domain
pub type Address = Vec<u8>;
/// Resource identifier
pub type ResourceId = [u8; 32];
/// Verification key for ZK proofs
pub type VerificationKey = Vec<u8>;
/// Zero-knowledge proof
pub type Proof = Vec<u8>;
/// Timestamp in milliseconds
pub type Timestamp = u64;

/// Core effect types in the Temporal Effect Language
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    /// Deposit assets from external domain
    Deposit {
        domain: DomainId,
        asset: AssetId,
        amount: Amount,
    },
    /// Withdraw assets to external domain
    Withdraw {
        domain: DomainId,
        asset: AssetId,
        amount: Amount,
        address: Address,
    },
    /// Transfer assets between addresses
    Transfer {
        from: Address,
        to: Address,
        asset: AssetId,
        amount: Amount,
    },
    /// Invoke another program with effects
    Invoke {
        program: Address,
        effects: Vec<Effect>,
    },
    /// Observe an external fact
    Observe {
        fact_type: FactType,
        parameters: HashMap<String, Value>,
    },
    /// Execute a sequence of effects
    Sequence(Vec<Effect>),
    /// Create a resource with contents
    ResourceCreate {
        contents: ResourceContents,
    },
    /// Update a resource with new contents
    ResourceUpdate {
        resource_id: ResourceId,
        contents: ResourceContents,
    },
    /// Transfer a resource to another domain
    ResourceTransfer {
        resource_id: ResourceId,
        target_domain: DomainId,
    },
    /// Merge multiple resources into one
    ResourceMerge {
        source_ids: Vec<ResourceId>,
        target_id: ResourceId,
    },
    /// Split a resource into multiple resources
    ResourceSplit {
        source_id: ResourceId,
        target_ids: Vec<ResourceId>,
        distribution: Vec<Amount>,
    },
    /// Verify a zero-knowledge proof
    VerifyProof {
        verification_key: VerificationKey,
        proof: Proof,
    },
    /// Execute a ZK circuit
    ExecuteCircuit {
        circuit_type: CircuitType,
        inputs: HashMap<String, Value>,
    },
}

/// Authorization methods for effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Authorization {
    /// Digital signature from an address
    Signature {
        address: Address,
        signature: Vec<u8>,
    },
    /// Zero-knowledge proof authorization
    ZKProof {
        verification_key: VerificationKey,
        proof: Proof,
    },
    /// Authorization via token ownership
    TokenOwnership {
        asset: AssetId,
        amount: Amount,
    },
    /// Multi-signature authorization
    MultiSig {
        addresses: Vec<Address>,
        threshold: usize,
        signatures: Vec<Vec<u8>>,
    },
    /// Time-locked authorization
    Timelock {
        unlock_time: Timestamp,
        inner_auth: Box<Authorization>,
    },
}

/// Resource contents types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceContents {
    /// Token balance
    TokenBalance {
        asset: AssetId,
        amount: Amount,
    },
    /// Non-fungible token
    NFT {
        collection: String,
        token_id: String,
        metadata: HashMap<String, Value>,
    },
    /// Data resource
    Data {
        data_type: String,
        value: Value,
    },
    /// Composite resource
    Composite {
        components: HashMap<String, ResourceContents>,
    },
}

/// Circuit types for ZK operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitType {
    /// Token swap circuit
    TokenSwap,
    /// Private transfer circuit
    PrivateTransfer,
    /// Zero-knowledge proof of balance
    BalanceProof,
    /// Anonymous voting circuit
    AnonymousVote,
    /// Custom circuit with identifier
    Custom(String),
}

/// Fact types that can be observed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FactType {
    /// Block header fact
    BlockHeader,
    /// Transaction inclusion proof
    TransactionInclusion,
    /// Token balance fact
    TokenBalance,
    /// Price oracle fact
    PriceOracle,
    /// Custom fact with identifier
    Custom(String),
}

/// Builder methods for constructing effects
impl Effect {
    /// Create a new sequence of effects
    pub fn sequence(effects: Vec<Effect>) -> Self {
        Effect::Sequence(effects)
    }
    
    /// Add authorization to an effect
    pub fn with_auth(self, auth: Authorization) -> AuthorizedEffect {
        AuthorizedEffect {
            effect: self,
            authorization: auth,
        }
    }
    
    /// Add a condition to an effect
    pub fn with_condition(self, condition: Condition) -> ConditionalEffect {
        ConditionalEffect {
            effect: self,
            condition,
        }
    }
    
    /// Add a timeout to an effect
    pub fn with_timeout(self, timeout: Timestamp) -> TimedEffect {
        TimedEffect {
            effect: self,
            timeout,
        }
    }
    
    /// Create a new deposit effect
    pub fn deposit(domain: &str, asset: &str, amount: Amount) -> Self {
        Effect::Deposit {
            domain: domain.to_string(),
            asset: asset.to_string(),
            amount,
        }
    }
    
    /// Create a new withdraw effect
    pub fn withdraw(domain: &str, asset: &str, amount: Amount, address: Address) -> Self {
        Effect::Withdraw {
            domain: domain.to_string(),
            asset: asset.to_string(),
            amount,
            address,
        }
    }
    
    /// Create a resource
    pub fn create_resource(contents: ResourceContents) -> Self {
        Effect::ResourceCreate { contents }
    }
    
    /// Update a resource
    pub fn update_resource(resource_id: ResourceId, contents: ResourceContents) -> Self {
        Effect::ResourceUpdate { resource_id, contents }
    }
    
    /// Transfer a resource
    pub fn transfer_resource(resource_id: ResourceId, target_domain: &str) -> Self {
        Effect::ResourceTransfer {
            resource_id,
            target_domain: target_domain.to_string(),
        }
    }
    
    /// Merge resources
    pub fn merge_resources(source_ids: Vec<ResourceId>, target_id: ResourceId) -> Self {
        Effect::ResourceMerge { source_ids, target_id }
    }
    
    /// Split a resource
    pub fn split_resource(
        source_id: ResourceId,
        target_ids: Vec<ResourceId>,
        distribution: Vec<Amount>,
    ) -> Self {
        Effect::ResourceSplit {
            source_id,
            target_ids,
            distribution,
        }
    }
    
    /// Verify a ZK proof
    pub fn verify_proof(verification_key: VerificationKey, proof: Proof) -> Self {
        Effect::VerifyProof { verification_key, proof }
    }
    
    /// Execute a circuit
    pub fn execute_circuit(
        circuit_type: CircuitType,
        inputs: HashMap<String, Value>,
    ) -> Self {
        Effect::ExecuteCircuit { circuit_type, inputs }
    }
}

/// Effect with authorization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorizedEffect {
    pub effect: Effect,
    pub authorization: Authorization,
}

/// Effect with a condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalEffect {
    pub effect: Effect,
    pub condition: Condition,
}

/// Effect with a timeout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimedEffect {
    pub effect: Effect,
    pub timeout: Timestamp,
}

/// Condition for conditional effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    /// Fact observed
    FactObserved(FactType),
    /// Resource state matches expression
    ResourceState {
        resource_id: ResourceId,
        predicate: Predicate,
    },
    /// Timestamp condition
    Time(TimeCondition),
    /// Combine conditions with AND
    And(Vec<Condition>),
    /// Combine conditions with OR
    Or(Vec<Condition>),
    /// Negate a condition
    Not(Box<Condition>),
}

/// Time-based condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeCondition {
    /// Before a timestamp
    Before(Timestamp),
    /// After a timestamp
    After(Timestamp),
    /// Between two timestamps
    Between(Timestamp, Timestamp),
}

/// Predicate for resource state conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Predicate {
    /// Equal to value
    Eq(Value),
    /// Greater than value
    Gt(Value),
    /// Less than value
    Lt(Value),
    /// Greater than or equal to value
    Gte(Value),
    /// Less than or equal to value
    Lte(Value),
    /// Value in range
    InRange(Value, Value),
    /// Contains key (for map types)
    Contains(String),
}
```

### Example Usage

```rust
use std::collections::HashMap;
use tel::{Effect, Authorization, ResourceContents, CircuitType, ResourceId};

// Helper function to create a resource ID
fn resource_id(id: &str) -> ResourceId {
    let mut res_id = [0u8; 32];
    let bytes = id.as_bytes();
    res_id[..bytes.len().min(32)].copy_from_slice(&bytes[..bytes.len().min(32)]);
    res_id
}

// Simple token transfer
fn token_transfer() -> Effect {
    Effect::sequence(vec![
        Effect::withdraw("Ethereum", "USDC", 1000, vec![0x12, 0x34]),
        Effect::update_resource(
            resource_id("reg123"),
            ResourceContents::TokenBalance {
                asset: "USDC".to_string(),
                amount: 1000,
            },
        ).with_auth(Authorization::Signature {
            address: vec![0x12, 0x34],
            signature: vec![/* signature bytes */],
        }),
    ])
}

// Cross-domain bridge with resource transfer
fn cross_domain_bridge() -> Effect {
    let proof_bytes = vec![/* proof bytes */];
    
    Effect::sequence(vec![
        Effect::withdraw("Ethereum", "ETH", 1, vec![0x12, 0x34]),
        Effect::transfer_resource(
            resource_id("reg123"),
            "Solana",
        ).with_auth(Authorization::ZKProof {
            verification_key: vec![/* verification key bytes */],
            proof: proof_bytes,
        }),
    ])
}

// Complex ZK-based operation
fn zk_operation() -> Effect {
    let proof_bytes = vec![/* proof bytes */];
    let mut swap_inputs = HashMap::new();
    swap_inputs.insert("tokenA".to_string(), Value::String("USDC".to_string()));
    swap_inputs.insert("tokenB".to_string(), Value::String("ETH".to_string()));
    swap_inputs.insert("amountA".to_string(), Value::Number(1000.into()));
    
    Effect::sequence(vec![
        Effect::create_resource(
            ResourceContents::TokenBalance {
                asset: "USDC".to_string(),
                amount: 1000,
            },
        ),
        Effect::execute_circuit(CircuitType::TokenSwap, swap_inputs),
        Effect::update_resource(
            resource_id("reg456"),
            ResourceContents::TokenBalance {
                asset: "ETH".to_string(),
                amount: 500_000_000, // 0.5 ETH with precision
            },
        ).with_auth(Authorization::ZKProof {
            verification_key: vec![/* verification key bytes */],
            proof: proof_bytes,
        }),
    ])
}

// Execute a sequence of resource operations
fn resource_sequence() -> Effect {
    let proof_bytes = vec![/* proof bytes */];
    
    Effect::sequence(vec![
        Effect::create_resource(
            ResourceContents::TokenBalance {
                asset: "USDC".to_string(),
                amount: 1000,
            },
        ),
        Effect::create_resource(
            ResourceContents::TokenBalance {
                asset: "ETH".to_string(),
                amount: 1_000_000_000, // 1 ETH with precision
            },
        ),
        Effect::merge_resources(
            vec![resource_id("reg1"), resource_id("reg2")],
            resource_id("reg3"),
        ).with_auth(Authorization::ZKProof {
            verification_key: vec![/* verification key bytes */],
            proof: proof_bytes,
        }),
    ])
}
```

### Compilation and Execution

TEL effects will be compiled to domain-specific formats by adapters:

```rust
/// TEL Compiler trait
pub trait EffectCompiler {
    type Error;
    type Output;
    
    /// Compile an effect into domain-specific representation
    fn compile(&self, effect: &Effect) -> Result<Self::Output, Self::Error>;
    
    /// Check if an effect is valid for this compiler
    fn validate(&self, effect: &Effect) -> Result<(), Self::Error>;
}

/// Domain adapter for Ethereum
pub struct EthereumAdapter {
    provider_url: String,
    chain_id: u64,
    contracts: HashMap<String, Address>,
}

impl EffectCompiler for EthereumAdapter {
    type Error = EthereumAdapterError;
    type Output = EthereumTransaction;
    
    fn compile(&self, effect: &Effect) -> Result<Self::Output, Self::Error> {
        // Convert effect to Ethereum transaction
        // Implementation details...
    }
    
    fn validate(&self, effect: &Effect) -> Result<(), Self::Error> {
        // Validate effect for Ethereum compatibility
        // Implementation details...
    }
}

/// Effect execution engine
pub struct EffectExecutor {
    adapters: HashMap<DomainId, Box<dyn EffectCompiler>>,
    resource_manager: ResourceManager,
}

impl EffectExecutor {
    /// Execute an effect
    pub async fn execute(&self, effect: Effect) -> Result<EffectResult, ExecutionError> {
        // 1. Validate effect
        self.validate_effect(&effect)?;
        
        // 2. Compile effect for specific domains
        let domain_txs = self.compile_effect(&effect)?;
        
        // 3. Execute transactions on respective domains
        let results = self.submit_transactions(domain_txs).await?;
        
        // 4. Update resource state
        self.update_resources(&effect, &results)?;
        
        // 5. Return results
        Ok(EffectResult { /* ... */ })
    }
    
    // Other methods...
}
```

Resource operations will have special handling:

1. **ZK Circuit Generation**: Using our Rust ZK library to generate circuits
2. **Proof Generation**: Integration with the prover system
3. **Resource State Tracking**: Tracking resource state in the Causality runtime
4. **Cross-domain Coordination**: Using domain adapters for cross-domain resource transfers

## Consequences

### Positive

- Unified Rust API for expressing effects across domains
- Type safety through Rust's strong type system
- Integration with Rust-based testing and simulation framework
- Enhanced expressiveness for cross-domain operations
- Direct support for resource operations and ZK proofs
- Declarative syntax with builder pattern for complex operations

### Negative

- Learning curve for developers new to Rust
- Complexity in implementing adapters for all domains
- Performance considerations in effect validation and compilation
- Increased complexity for ZK circuit integration
- FFI overhead when integrating with non-Rust languages

### Neutral

- Requires standardization of effect types across the codebase
- Will evolve as new domains and resource types emerge
- Needs companion libraries for languages other than Rust

## Related ADRs

- [ADR 004: Concurrency](adr_concurrency.md)
- [ADR 007: Fact Management](adr_007_fact_management.md)
- [ADR 022: ZK Registers](adr_022_zk_registers.md)

## Implementation Plan

1. Implement core TEL types and traits in Rust
2. Build domain adapters for Ethereum, Solana, and other major chains
3. Integrate with the resource management system
4. Develop test suite and simulation environment
5. Create compiler integration with the VM system
6. Add formal verification capabilities via Rust's type system
7. Build developer tools and documentation
8. Implement IDE integrations with Rust-Analyzer