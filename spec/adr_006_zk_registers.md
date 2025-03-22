# ADR-006: Register System for ZK-Domain Integration

## Status

Proposed

## Author

Sam Hart

## Last updated

2023-21-03

## Context

Provable execution across domains enables building secure cross-domain applications. Our time-oriented model ensures no causality violation by ordering events in a coherent manner using the distributed time map. However, as the system grows to include more domains and participants, we will need a more structured way to handle the state interaction between domains. This structure should:

1. Make the safety and correctness of operations provable on-domain
2. Maintain chain-level security guarantees across domains
3. Support atomic and conditional transfers across domains

The current approach relies on each adapter being independently designed, creating inconsistencies and increasing the complexity of verifying correct operation across domains.

## Decision

We will implement a register-based architecture for Domain adapters that uses Zero-Knowledge (ZK) proofs to verify operations. This approach standardizes how resources move across domains and enables on-domain verification of the correctness of these operations.

### Core Components

1. Registers: Discrete units of storage that can hold resources, commitments, or other data
2. Register Operations: Well-defined operations that can be performed on registers (create, update, delete)
3. Register Authorization: Mechanisms to authorize register operations, including ZK proofs
4. Resource Conservation Laws: Mathematical invariants that ensure resources are neither created nor destroyed in an operation
5. Proof Generation: Generation of ZK proofs to verify the correctness of register operations
6. On-Domain Verification: Mechanisms to verify ZK proofs on-domain to ensure operation correctness

### Register Data Structure

```rust
use std::collections::HashMap;

type RegisterID = String;
type Address = String;
type BlockHeight = u64;
type TokenType = String;
type Amount = u64;
type CollectionAddress = String;
type TokenId = String;
type CommitmentType = String;
type DataFormat = String;
type EffectID = String;
type NullifierKey = String;
type CommitmentKey = String;

struct Register {
    register_id: RegisterID,
    owner: Address,
    contents: RegisterContents,
    last_updated: BlockHeight,
    metadata: HashMap<String, serde_json::Value>,
}

enum RegisterContents {
    Resource(Resource),
    TokenBalance {
        token_type: TokenType,
        address: Address,
        amount: Amount,
    },
    NFTContent {
        collection_address: CollectionAddress,
        token_id: TokenId,
    },
    StateCommitment {
        commitment_type: CommitmentType,
        data: Vec<u8>,
    },
    TimeMapCommitment {
        block_height: BlockHeight,
        data: Vec<u8>,
    },
    DataObject {
        data_format: DataFormat,
        data: Vec<u8>,
    },
    EffectDAG {
        effect_id: EffectID,
        data: Vec<u8>,
    },
    ResourceNullifier {
        nullifier_key: NullifierKey,
        data: Vec<u8>,
    },
    ResourceCommitment {
        commitment_key: CommitmentKey,
        data: Vec<u8>,
    },
    CompositeContents(Vec<RegisterContents>),
}
```

### Register Operations

```rust
type Authorization = String;
type Proof = String;
type Delta = String;

struct RegisterOperation {
    op_type: OperationType,
    registers: Vec<RegisterID>,
    new_contents: Option<RegisterContents>,
    authorization: Authorization,
    proof: Proof,
    resource_delta: Delta,
}

enum OperationType {
    CreateRegister,
    UpdateRegister,
    DeleteRegister,
    TransferOwnership(Address),
    CompositeOperation(Vec<OperationType>),
}
```

### Operation Authorization Methods

```rust
type VerificationKey = String;
type TokenAddress = String;
type DAOAddress = String;
type ProposalId = String;
type Signature = String;
type Timestamp = u64;

enum AuthorizationMethod {
    ZKProofAuthorization {
        verification_key: VerificationKey,
        proof: Proof,
    },
    TokenOwnershipAuthorization {
        token_address: TokenAddress,
        amount: Amount,
    },
    NFTOwnershipAuthorization {
        collection_address: CollectionAddress,
        token_id: TokenId,
    },
    MultiSigAuthorization {
        addresses: Vec<Address>,
        threshold: i32,
        signatures: Vec<Signature>,
    },
    DAOAuthorization {
        dao_address: DAOAddress,
        proposal_id: ProposalId,
    },
    TimelockAuthorization {
        address: Address,
        timestamp: Timestamp,
    },
    CompositeAuthorization {
        methods: Vec<AuthorizationMethod>,
        auth_combinator: AuthCombinator,
    },
}

enum AuthCombinator {
    All,
    Any,
    Threshold(usize),
}
```

### Resource Conservation

For any operation that manipulates resources, the system enforces:

```
Σ resources_before_operation = Σ resources_after_operation
```

This is verified through ZK proofs, ensuring that resources are neither created nor destroyed inappropriately, maintaining mathematical invariants even across multiple domains.

### One-Time Use Registers

Registers in the system are designed to be one-time use to prevent replay attacks and ensure the integrity of operations across domains. This means:

1. Once a register is used in an operation (update, transfer, deletion), it cannot be reused for another operation.
2. For each operation, a new register must be created to store the resulting state.
3. Previous registers are marked as "consumed" using a nullifier mechanism similar to ZK systems.
4. The one-time use property is enforced through ZK proofs that verify a register has not been previously consumed.

This design provides several benefits:
- Prevention of Double-Spending: Resources cannot be spent multiple times
- Immutable History: Creates an append-only log of operations
- Simplified Verification: Each register has a clear, single purpose in its lifecycle
- Enhanced Security: Makes many classes of replay attacks impossible

For operations that logically update the same resource (e.g., token balance changes), each operation creates a new register that references the previous one, forming a region of register operations that can be tracked and verified.

### ZK Proof System Integration

Domain adapters will generate ZK proofs to verify:

1. Valid register ownership and authorization
2. Correct execution of register operations
3. Maintenance of resource conservation laws
4. Temporal validation (ensuring operation follows causal rules)
5. Correct construction of register contents

These proofs are verified on-domain to provide chain-level security guarantees for cross-domain operations.

### Cross-domain Operations

For operations involving multiple domains:

1. The source domain records the operation and generates a proof
2. The destination domain verifies the proof
3. Both domains maintain register records that are provably consistent
4. Resource conservation is verified across the entire operation

### Temporal Validation Integration

The register system integrates with temporal validation by:

1. Storing time map commitments in registers for on-domain verification
2. Generating ZK proofs that operations respect temporal ordering
3. Using the distributed map of time to order register operations causally

### Register ID Generation

To ensure uniqueness, each chain will have a dedicated function for register creation that sequentially increments an integer register identifier.

### Data Availability Requirements

To ensure all necessary data is available for on-domain verification, all transactions that reference data posted to a domain's data availability layer must include the data itself, not just a reference.

## Advantages Over Traditional Adapter Approach

1. Unified Model: All adapters follow the same register-based architecture
2. Provable Correctness: Operations can be verified on-domain through ZK proofs
3. Enhanced Security: Resource conservation laws provide mathematical guarantees
4. Cross-domain Atomicity: Operations can be composed across domains with atomic guarantees
5. Standardized Development: Adapter development follows consistent patterns
6. Reduced Attack Surface: The attack surface is minimized through formal verification

## Conclusion

The ZK-based register system provides a robust, secure, and standardized approach to handling resources and state across multiple domains. By using ZK proofs and enforcing resource conservation laws, we can build cross-domain applications with chain-level security guarantees. This architecture is a significant step toward making the system production-ready for critical financial applications across multiple chains.

---

# Addendum: Register Lifecycle and Garbage Collection

## One-Time Use Registers and State Management

After further consideration of the register model, we've determined that one-time use registers provide the cleanest conceptual model for our ZK-based architecture. However, to address the practical concern of state explosion, we are introducing a structured approach to register lifecycle management and garbage collection.

## Register Lifecycle Model

```rust
use std::collections::{HashMap, HashSet};

type EpochID = u64;
type TransactionID = String;
type SummaryGroup = String;
type ResourceID = String;
type ArchiveID = String;
type Hash = String;
type Query = String;

// Enhanced register with lifecycle metadata
struct Register {
    register_id: RegisterID,
    contents: RegisterContents,
    epoch: EpochID,
    status: RegisterStatus,
    consumed_by: Option<TransactionID>,
    successors: Vec<RegisterID>,  // Registers created when this one was consumed
    created_at: BlockHeight,
    last_updated: BlockHeight,
    metadata: HashMap<String, serde_json::Value>,
}

enum RegisterStatus {
    Active,
    Consumed,
    Archived,
    Summarized,
}

// Epoch management
struct EpochManager {
    current_epoch: EpochID,
    epoch_boundaries: HashMap<EpochID, BlockHeight>,
    registers_per_epoch: HashMap<EpochID, HashSet<RegisterID>>,
    archival_policy: ArchivalPolicy,
}

struct ArchivalPolicy {
    keep_epochs: i32,  // How many epochs to keep fully accessible
    prune_after: i32,  // When to start garbage collection
    summary_strategy: SummaryStrategy,
    archive_location: ArchiveLocation, // Where to store compressed register data
}

enum SummaryStrategy {
    SummarizeByResource,  // Create one summary per resource per epoch
    SummarizeByAccount,   // Create one summary per account per epoch
    SummarizeByType,      // Create one summary per register type per epoch
    CustomSummary(fn(&Register) -> SummaryGroup), // Custom grouping function
}

enum ArchiveLocation {
    LocalStorage(String),
    RemoteStorage { url: String, api_key: String },
    DistributedStorage { peers: Vec<String> },
}
```

## One-Time Use Register Pattern

With this model, registers are used once and then marked as consumed:

```rust
use std::error::Error;
use std::result::Result;

enum EffectError {
    RegisterAlreadyConsumed,
    AuthorizationFailed,
    // Other effect errors...
}

struct Effect {
    // Effect properties...
}

struct Resource {
    // Resource properties...
}

struct RegisterSummary {
    // Summary properties...
}

// Apply effect, consuming input registers and creating new ones
async fn apply_effect(
    input_register_ids: Vec<RegisterID>, 
    effect: Effect, 
    auth_method: AuthorizationMethod
) -> Result<Vec<RegisterID>, EffectError> {
    // Verify all input registers are Active
    let mut statuses = Vec::new();
    for id in &input_register_ids {
        statuses.push(get_register_status(id).await?);
    }
    
    if !statuses.iter().all(|s| matches!(s, RegisterStatus::Active)) {
        return Err(EffectError::RegisterAlreadyConsumed);
    }
    
    // Verify authorization
    let authorized = verify_authorization(&input_register_ids, &auth_method).await?;
    if !authorized {
        return Err(EffectError::AuthorizationFailed);
    }
    
    // Read input registers
    let mut input_registers = Vec::new();
    for id in &input_register_ids {
        input_registers.push(read_register(id).await?);
    }
    
    // Apply effect to create new register contents
    let output_contents = apply_effect_to_contents(&effect, &input_registers).await?;
    
    // Create new registers in current epoch
    let current_epoch = get_current_epoch().await?;
    let mut output_register_ids = Vec::new();
    
    for content in output_contents {
        let reg_id = create_register(content, current_epoch).await?;
        output_register_ids.push(reg_id);
    }
    
    // Mark input registers as consumed, linking to successors
    let tx_id = get_current_transaction_id().await?;
    for reg_id in input_register_ids {
        mark_register_consumed(&reg_id, &tx_id, &output_register_ids).await?;
    }
    
    // Record the transition
    record_transition(&input_register_ids, &output_register_ids, &effect, &auth_method).await?;
    
    // Return new register IDs
    Ok(output_register_ids)
}
```

This approach provides a clean audit trail and simplifies reasoning about register state. Each operation consumes input registers and produces new output registers, creating an explicit causal structure.

## Garbage Collection at Epoch Boundaries

To prevent state explosion, we implement garbage collection at epoch boundaries:

```rust
struct GCStats {
    // Statistics for garbage collection...
}

struct BatchResult {
    processed_count: usize,
    summary_count: usize,
    archived_count: usize,
}

// Garbage collection process
async fn garbage_collect_epoch(epoch_id: EpochID) -> Result<GCStats, Box<dyn Error>> {
    // 1. Check if epoch is eligible for garbage collection
    let policy = get_archival_policy().await?;
    let current_epoch = get_current_epoch().await?;
    
    let age = current_epoch - epoch_id;
    
    if age < policy.prune_after as u64 {
        return Err("Epoch too recent for garbage collection".into());
    }
    
    // 2. Get all consumed registers from the epoch
    let consumed_registers = get_registers_in_epoch(epoch_id, RegisterStatus::Consumed).await?;
    
    // 3. Process registers in batches
    let batches = create_batches(&consumed_registers, 1000);
    let mut results = Vec::new();
    
    for batch in batches {
        // 3a. Create register summaries based on policy
        let summaries = create_register_summaries(&policy.summary_strategy, &batch).await?;
        
        // 3b. Store summaries as new registers
        let summary_registers = store_summaries(&summaries).await?;
        
        // 3c. Compress and archive original registers
        let archive_ids = archive_registers(&batch).await?;
        
        // 3d. Replace full registers with stubs
        let mut stubs = Vec::new();
        for (register, archive_id) in batch.iter().zip(archive_ids.iter()) {
            stubs.push(create_register_stub(register, archive_id).await?);
        }
        
        for (i, stub) in stubs.iter().enumerate() {
            replace_register_with_stub(&batch[i].register_id, stub).await?;
        }
        
        // 3e. Update register status
        for r in &batch {
            update_register_status(&r.register_id, RegisterStatus::Archived).await?;
        }
        
        results.push(BatchResult {
            processed_count: batch.len(),
            summary_count: summary_registers.len(),
            archived_count: archive_ids.len(),
        });
    }
    
    // 4. Update global indices
    update_global_indices(epoch_id).await?;
    
    // 5. Return statistics
    Ok(aggregate_results(&results))
}
```

## Register Summarization

A key component of our approach is register summarization, which creates compact representations of register state at epoch boundaries:

```rust
use std::collections::HashMap;

// Create register summaries based on strategy
async fn create_register_summaries(
    strategy: &SummaryStrategy, 
    registers: &[Register]
) -> Result<Vec<RegisterSummary>, Box<dyn Error>> {
    match strategy {
        SummaryStrategy::SummarizeByResource => {
            // Group registers by resource
            let groups = group_registers_by_resource(registers).await?;
            
            // For each resource, create a summary
            let mut summaries = Vec::new();
            for (resource_id, regs) in groups {
                summaries.push(create_resource_summary(&resource_id, &regs).await?);
            }
            
            Ok(summaries)
        },
        
        SummaryStrategy::SummarizeByAccount => {
            // Group registers by owner account
            let groups = group_registers_by_account(registers).await?;
            
            // For each account, create a summary
            let mut summaries = Vec::new();
            for (account_id, regs) in groups {
                summaries.push(create_account_summary(&account_id, &regs).await?);
            }
            
            Ok(summaries)
        },
        
        SummaryStrategy::SummarizeByType => {
            // Group registers by type
            let groups = group_registers_by_type(registers).await?;
            
            // For each type, create a summary
            let mut summaries = Vec::new();
            for (reg_type, regs) in groups {
                summaries.push(create_type_summary(&reg_type, &regs).await?);
            }
            
            Ok(summaries)
        },
        
        SummaryStrategy::CustomSummary(grouping_fn) => {
            // Apply custom grouping
            let groups = group_registers_by(grouping_fn, registers).await?;
            
            // For each group, create a summary
            let mut summaries = Vec::new();
            for (group_key, regs) in groups {
                summaries.push(create_custom_summary(&group_key, &regs).await?);
            }
            
            Ok(summaries)
        }
    }
}
```

Summaries provide an efficient way to preserve essential state information while reclaiming storage. They include:

1. Initial State: The state at the beginning of the epoch
2. Final State: The state at the end of the epoch
3. Key Transitions: Important state changes within the epoch
4. Proofs: Cryptographic proofs linking to archived details

## Resource Mapping Update

As registers are summarized and archived, resource mappings must be updated:

```rust
// Update resource mapping after summarization
async fn update_resource_mappings(
    current_mapping: HashMap<ResourceID, RegisterID>, 
    summaries: &[RegisterSummary]
) -> Result<(), Box<dyn Error>> {
    // Create new mapping based on summaries
    let mut new_mapping = HashMap::new();
    for summary in summaries {
        new_mapping.insert(summary.resource_id.clone(), summary.summary_register_id.clone());
    }
    
    // Update resource map with new register IDs
    // New mappings take precedence
    let updated_map = current_mapping.clone();
    for (k, v) in new_mapping {
        updated_map.insert(k, v);
    }
    
    update_resource_map(&updated_map).await
}
```

This ensures that despite garbage collection, the system maintains accurate references to the current state of all resources.

## Archive Management

Archived registers are stored in a dedicated archive system:

```rust
enum ArchiveError {
    StorageError,
    NotFound,
    CorruptData,
    // Other archive errors...
}

// Archive component
struct ArchiveSystem {
    store_registers: fn(&[Register]) -> Result<Vec<ArchiveID>, ArchiveError>,
    retrieve_register: fn(&ArchiveID) -> Result<Register, ArchiveError>,
    verify_archive: fn(&ArchiveID, &Hash) -> Result<bool, ArchiveError>,
    list_archives: fn(&Query) -> Result<Vec<ArchiveID>, ArchiveError>,
}

// Register stub (on-domain representation of archived register)
struct RegisterStub {
    register_id: RegisterID,
    archive_id: ArchiveID,
    summary_data: RegisterSummary,  // Compact summary of key information
    verification_hash: Hash,        // Hash to verify retrieved data
    status: RegisterStatus,         // Always Archived
    metadata: HashMap<String, serde_json::Value>,
}
```

The archive system provides:

1. Efficient Storage: Compressed, potentially off-domain storage of register data
2. Data Integrity: Hashes ensure archived data hasn't been tampered with
3. On-Demand Retrieval: Registers can be retrieved when needed for deep auditing
4. Searchability: Archived registers can be located by various criteria

## Implementation Considerations

Implementing this register lifecycle approach requires:

1. Epoch Definition: Clear boundaries for when garbage collection occurs (block height, time period, event-based)
2. Tunable Policies: Different applications may need different retention policies
3. Resource Mapping: Efficient tracking of which register currently represents each resource
4. Summary Strategies: Application-specific approaches to register summarization
5. Archive Storage: Efficient, secure storage for archived register data
6. Recovery Procedures: Clear process for retrieving archived data when needed

## Impact on ZK Circuits

This lifecycle model is particularly compatible with ZK circuits because:

1. Immutable Inputs: One-time use registers provide clean, immutable circuit inputs
2. Explicit Causality: The register consumption model creates clear causal domains
3. Compact Representation: Summaries can be designed to minimize circuit complexity
4. Proof Tracing: Each epoch's summaries can include proofs linking to previous epochs

## Conclusion

The one-time use register model with epoch-based garbage collection provides a good balance between conceptual clarity and practical efficiency. By treating registers as immutable and creating explicit state transitions, we gain strong auditability and causal reasoning. By implementing structured garbage collection at epoch boundaries, we prevent state explosion without sacrificing these properties.