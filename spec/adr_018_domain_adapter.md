# ADR-018: Domain Adapter Architecture

## Status

Accepted

## Context

Causality needs to interact with multiple chains, each with different:

- APIs and RPC endpoints
- State models and data structures
- Transaction formats and signatures
- Smart contract interfaces
- Register implementations

The system needs a standardized approach to:

1. Deposit to and withdraw from chains
2. Observe facts from chains
3. Interact with smart contracts
4. Manage register operations and ZK proofs
5. Handle cross-domain register transfers

## Decision

We will implement a standardized **chain Adapter** architecture with the following components:

1. **Core Adapter Interface**: Common interface for all chain integrations
2. **Per-chain Adapter**: Implementation for each supported chain
3. **Register System Integration**: Functions for register operations and ZK proofs
4. **Cross-domain Coordination**: Methods for register transfers across Domains

### Core Adapter Interface

```rust
use std::sync::Arc;
use async_trait::async_trait;

type DomainID = String;
type TxHash = String;
type SerializedTx = Vec<u8>;
type RegisterID = String;
type ControllerLabel = String;
type VerificationKey = String;
type Proof = Vec<u8>;
type Inputs = Vec<u8>;

#[async_trait]
trait ChainAdapter {
    /// Unique identifier
    fn adapter_id(&self) -> String;
    
    /// Domain supported
    fn supported_domain(&self) -> DomainID;
    
    /// Connect to chain
    async fn connect(&self, config: ConnectionConfig) -> Result<Arc<dyn Connection>, AdapterError>;
    
    /// Observe external fact
    async fn observe_fact(&self, fact_type: FactType) -> Result<Fact, AdapterError>;
    
    /// Deposit to chain
    async fn deposit(&self, account: Account, asset: Asset, amount: Amount) -> Result<Effect, AdapterError>;
    
    /// Withdraw from chain
    async fn withdraw(&self, account: Account, asset: Asset, amount: Amount, address: Address) -> Result<Effect, AdapterError>;
    
    /// Submit raw transaction
    async fn submit_transaction(&self, tx: SerializedTx) -> Result<TxHash, AdapterError>;
    
    // Register system extensions
    
    /// Create register
    async fn create_register(&self, contents: RegisterContents) -> Result<RegisterID, AdapterError>;
    
    /// Update register
    async fn update_register(&self, register_id: RegisterID, contents: RegisterContents, authorization: Authorization) -> Result<Effect, AdapterError>;
    
    /// Observe register state
    async fn observe_register(&self, register_id: RegisterID) -> Result<RegisterFact, AdapterError>;
    
    /// Verify ZK proof
    async fn verify_proof(&self, verification_key: VerificationKey, proof: Proof) -> Result<ZKProofFact, AdapterError>;
    
    /// Cross-domain transfer
    async fn transfer_register(&self, register_id: RegisterID, target_domain: DomainID, controller_label: ControllerLabel) -> Result<Effect, AdapterError>;
    
    /// Check if register exists
    async fn register_exists(&self, register_id: RegisterID) -> Result<bool, AdapterError>;
    
    /// Generate ZK proof
    async fn generate_proof(&self, circuit_type: CircuitType, inputs: Inputs) -> Result<Proof, AdapterError>;
}
```

### Register System Integration

chain adapters must handle register operations:

1. **Register Creation**: Creating on-Domain representations of registers
2. **Register Updates**: Updating register contents with appropriate authorization
3. **Register Observation**: Observing register state from the chain
4. **Proof Verification**: Verifying ZK proofs for register operations
5. **Cross-domain Transfers**: Transferring registers between chains

### Register Operation Flow

For register operations, the flow is:

1. **Input Validation**: Validate register operation request
2. **Authorization Check**: Verify operation is authorized
3. **ZK Proof Generation**: Generate proof if required
4. **Transaction Creation**: Create chain-specific transaction
5. **Transaction Submission**: Submit transaction to chain
6. **Observation**: Observe register state after operation
7. **Fact Generation**: Generate fact for register operation
8. **Propagation**: Propagate fact to other components

### Cross-domain Register Transfer

Cross-domain register transfers require special handling:

1. **Source Domain**: Lock or burn register on source Domain
2. **Controller Label**: Generate controller label for ancestral validation
3. **Proof Generation**: Generate proof of source Domain operation
4. **Target Domain**: Create register on target Domain with proof
5. **Time Map**: Update time map for temporal validation
6. **Observation**: Observe register transfer as a fact

## Consequences

### Positive

- Standardized interface for all chain interactions
- Simplified integration of new chains
- Consistent handling of register operations
- Improved cross-domain coordination
- Enhanced security through ZK proof verification
- Clear separation of concerns between business logic and chain interaction

### Negative

- Complexity in implementing adapters for diverse chains
- Performance overhead from abstraction
- Challenges in handling chain-specific features
- Development effort required for ZK circuit implementation

### Neutral

- Requires ongoing maintenance as chains evolve
- May need extensions for chain-specific features
- Adapters may vary in feature support based on chain capabilities