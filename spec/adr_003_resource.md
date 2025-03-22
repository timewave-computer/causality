# ADR-003: Resource System

## Status

Accepted

## Context

Causality programs need to interact with resources on external chains and across domains, including:

- Fungible tokens (e.g., ERC-20)
- Non-fungible tokens (e.g., ERC-721)
- Native Domain assets (e.g., ETH, BTC)
- Data items (e.g., price feeds, oracle responses)
- State commitments (e.g., Merkle roots)
- Access rights

In previous iterations, resources were directly associated with programs, leading to:

1. Inconsistent conservation rules
2. Unclear ownership boundaries
3. Limited composability
4. Insufficient verification mechanisms
5. Complex access control patterns
6. Security vulnerabilities from direct resource manipulation

We need a formal model that enables:
- Resource conservation across domains
- Verifiable resource ownership
- Resource composition and transformation
- Zero-knowledge verification of resource operations

## Decision

We will transition to a **register-based resource ownership model** where resources are formalized as structured types with well-defined rules and characteristics.

### Resource Model

In Rust, resources will be represented as:

```rust
/// Represents a Causality resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Rules governing usage
    pub resource_logic: ResourceLogic,
    /// Defines equivalence class
    pub fungibility_domain: Label,
    /// Numerical representation
    pub quantity: Quantity,
    /// Associated data
    pub metadata: Value,
    /// Whether existence must be verified
    pub ephemeral: bool,
    /// Uniqueness identifier
    pub nonce: Nonce,
    /// For verifying consumption
    pub nullifier_pub_key: NullifierPK,
    /// For deriving randomness
    pub randomness_seed: Seed,
}

impl Resource {
    /// Commitment to resource existence
    pub fn commitment(&self) -> Commitment {
        // Derive commitment from fields
    }
    
    /// Proof of resource consumption
    pub fn nullifier(&self) -> Nullifier {
        // Derive nullifier from fields
    }
    
    /// Classification of resource type
    pub fn kind(&self) -> ResourceKind {
        // Derive kind from fields
    }
    
    /// Resource balance change
    pub fn delta(&self) -> Delta {
        // Calculate balance change
    }
}
```

### Register-Based Resource Management

Resources will be managed through registers:

```rust
/// A register containing resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Register {
    /// Unique identifier for this register
    pub register_id: RegisterID,
    /// Address of the owner
    pub owner: Address,
    /// Contents of the register
    pub contents: RegisterContents,
    /// Last block when register was updated
    pub last_updated: BlockHeight,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
    /// Optional controller label for cross-domain resources
    pub controller_label: Option<ControllerLabel>,
}

/// Contents that can be stored in a register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegisterContents {
    /// A resource
    Resource(Resource),
    /// Token balance
    TokenBalance {
        token_type: TokenType,
        address: Address,
        amount: Amount,
    },
    /// NFT content
    NFTContent {
        collection_address: CollectionAddress,
        token_id: TokenId,
    },
    /// State commitment
    StateCommitment {
        commitment_type: CommitmentType,
        data: Vec<u8>,
    },
    /// Generic data object
    DataObject {
        format: DataFormat,
        data: Vec<u8>,
    },
    /// Composite contents (multiple contained resources)
    CompositeContents(Vec<RegisterContents>),
}
```

### Register Lifecycle States

Registers exist in one of several lifecycle states, which determine valid operations:

```rust
/// Lifecycle state of a resource register
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterState {
    /// Register is active and can be operated on
    Active,
    /// Register is locked for a specific operation
    Locked {
        /// ID of the operation locking the register
        operation_id: OperationId,
        /// When the lock expires
        expiry: Timestamp,
    },
    /// Register is frozen (operations suspended)
    Frozen {
        /// Reason for freezing
        reason: String,
        /// Authority that froze the register
        authority: Address,
    },
    /// Register is marked for deletion
    PendingDeletion {
        /// When the register will be deleted
        scheduled_time: Timestamp,
    },
    /// Register contains a tombstone (was deleted)
    Tombstone {
        /// When the register was deleted
        deletion_time: Timestamp,
        /// Hash of the register before deletion
        content_hash: ContentHash,
    },
}
```

### Resource Access Patterns

Access to resources follows these patterns:

1. **Creation**: Resources are created as registers by account programs.
2. **Authorization**: Register operations require explicit authorization.
3. **Operations**: Resources can be manipulated through authorized register operations.
4. **Transfer**: Resources can be transferred between registers with appropriate authorization.
5. **Cross-domain Movement**: Resources can move across domains through register transfers with controller labels.

### Authorization Methods

Register operations can be authorized through various methods:

```rust
/// Methods for authorizing register operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationMethod {
    /// ZK proof-based authorization
    ZKProofAuthorization {
        verification_key: VerificationKey,
        proof: Proof,
    },
    /// Authorization based on token ownership
    TokenOwnershipAuthorization {
        token_address: TokenAddress,
        amount: Amount,
    },
    /// Authorization based on NFT ownership
    NFTOwnershipAuthorization {
        collection_address: CollectionAddress,
        token_id: TokenId,
    },
    /// Multi-signature authorization
    MultiSigAuthorization {
        addresses: Vec<Address>,
        threshold: usize,
        signatures: Vec<Signature>,
    },
    /// DAO-based authorization
    DAOAuthorization {
        dao_address: DAOAddress,
        proposal_id: ProposalId,
    },
    /// Time-locked authorization
    TimelockAuthorization {
        address: Address,
        timestamp: Timestamp,
    },
    /// Composite authorization (multiple methods)
    CompositeAuthorization {
        methods: Vec<AuthorizationMethod>,
        combinator: AuthCombinator,
    },
}
```

### Resource Operations and ZK Verification

Resource operations will be verified using zero-knowledge proofs:

```rust
/// A resource operation with ZK verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceOperation {
    /// Type of operation
    pub op_type: OperationType,
    /// Input resources
    pub inputs: Vec<Resource>,
    /// Output resources
    pub outputs: Vec<Resource>,
    /// Net balance change
    pub delta: Delta,
    /// Operation authorization
    pub authorization: Authorization,
    /// Zero-knowledge proof
    pub proof: Proof,
}

/// Zero-knowledge proof for a resource operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// Verification key for the proof
    pub verification_key: VerificationKey,
    /// Proof data
    pub proof_data: Vec<u8>,
    /// Public inputs to the proof
    pub public_inputs: Vec<Vec<u8>>,
}

/// Supported operation types for resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// Create a new resource
    Create,
    /// Transfer resource between registers
    Transfer,
    /// Split a resource into multiple parts
    Split,
    /// Merge multiple resources
    Merge,
    /// Transform a resource to a different type
    Transform,
    /// Burn/destroy a resource
    Burn,
    /// Lock a resource temporarily
    Lock,
    /// Unlock a previously locked resource
    Unlock,
    /// Verify a resource's properties
    Verify,
}
```

ZK circuits will verify:
1. Resource conservation (sum of inputs = sum of outputs)
2. Valid transformations according to resource logic
3. Proper authorization of operations
4. Correct nullifier generation
5. Valid commitment opening

### Register State Transitions

Registers follow well-defined state transition rules to ensure consistency:

```rust
/// Defines a valid state transition for a resource register
#[derive(Debug)]
pub struct RegisterStateTransition {
    /// The register ID
    pub register_id: RegisterID,
    /// Previous state
    pub from_state: RegisterState,
    /// New state
    pub to_state: RegisterState,
    /// Operation causing the transition
    pub operation: OperationType,
    /// Timestamp of the transition
    pub timestamp: Timestamp,
    /// Authorization for the transition
    pub authorization: Authorization,
}
```

Valid transitions include:
- Active → Locked (for pending operations)
- Locked → Active (after operation completion)
- Active → Frozen (by authorized entity)
- Frozen → Active (when unfrozen)
- Active → PendingDeletion (marked for garbage collection)
- PendingDeletion → Tombstone (after garbage collection)

### Resource Conservation Laws

All register operations must satisfy resource conservation laws:

1. **Creation Conservation**: Resources can only be created with valid external proofs.
2. **Transfer Conservation**: Transfers must preserve total resource amounts.
3. **Destruction Conservation**: Resources can only be destroyed with valid external proofs.
4. **Cross-domain Conservation**: Cross-domain transfers must preserve controller labels for ancestral validation.

### Register Garbage Collection

Resources must be properly garbage collected to prevent state bloat:

```rust
/// Configuration for register garbage collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectionConfig {
    /// Time period before a PendingDeletion register is collected
    pub deletion_delay: Duration,
    /// Time period to keep tombstones before final removal
    pub tombstone_retention: Duration,
    /// Maximum number of registers to collect in one cycle
    pub max_collection_batch_size: usize,
    /// Whether to compress tombstones
    pub compress_tombstones: bool,
}

/// Collection statistics from a garbage collection run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    /// Number of registers marked for collection
    pub pending_count: usize,
    /// Number of registers collected
    pub collected_count: usize,
    /// Number of tombstones removed
    pub tombstones_removed: usize,
    /// Total bytes freed
    pub bytes_freed: usize,
    /// Duration of collection
    pub duration: Duration,
}
```

Garbage collection follows a two-phase process:
1. Registers are marked as `PendingDeletion` with a scheduled deletion time
2. After the deletion delay, registers are converted to `Tombstone` entries
3. After the tombstone retention period, tombstones are permanently removed

### Cross-domain Resource Flow

Resources that cross domains will maintain controller labels:

```rust
/// Tracks the lineage of a resource across domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerLabel {
    /// Controller that created the resource
    pub creating_controller: ControllerID,
    /// Current terminal controller
    pub terminal_controller: ControllerID,
    /// Controllers that have affected this resource
    pub affecting_controllers: Vec<ControllerID>,
    /// Backup controllers for recovery
    pub backup_controllers: Vec<ControllerID>,
}
```

Cross-domain transfers will undergo dual validation:
1. **Temporal Validation** - Using time maps to verify causal consistency
2. **Ancestral Validation** - Using controller labels to verify provenance

### Integration with Time System

Resources interact deeply with the Causality time system:

```rust
/// Time-related metadata for a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTimeData {
    /// Creation time point
    pub creation_time: TimePoint,
    /// Last modification time point
    pub last_modified: TimePoint,
    /// Expected expiration (if applicable)
    pub expiry: Option<TimePoint>,
    /// Time range during which the resource is valid
    pub validity_range: Option<TimeRange>,
    /// Time map entry for cross-domain resources
    pub time_map_entry: Option<TimeMapEntry>,
}
```

Resource operations are ordered through the time system:
1. Each operation receives a timestamp from the domain's time authority
2. Operations on the same register must respect causal ordering
3. Cross-domain operations use the time map for consistency
4. Time-locked resources can restrict operations to specific time ranges

## Implementation

The Rust implementation will consist of the following components:

1. **Resource Manager** - Core system for tracking and validating resources:

```rust
/// Manages resources within the Causality system
pub struct ResourceManager {
    /// Registers tracked by this manager
    registers: HashMap<RegisterID, Register>,
    /// Resource operations log
    operations_log: Vec<ResourceOperation>,
    /// Verification keys for proofs
    verification_keys: HashMap<OperationType, VerificationKey>,
    /// Garbage collection configuration
    gc_config: GarbageCollectionConfig,
    /// Integration with VM system
    vm_integration: ResourceVmIntegration,
}

impl ResourceManager {
    /// Create a new resource
    pub fn create_resource(&mut self, content: RegisterContents, owner: Address) -> Result<RegisterID> {
        // Implementation
    }
    
    /// Transfer a resource between registers
    pub fn transfer_resource(&mut self, from: RegisterID, to: RegisterID, amount: Amount) -> Result<()> {
        // Implementation
    }
    
    /// Update a resource
    pub fn update_resource(&mut self, register_id: RegisterID, new_content: RegisterContents) -> Result<()> {
        // Implementation
    }
    
    /// Destroy a resource
    pub fn destroy_resource(&mut self, register_id: RegisterID) -> Result<()> {
        // Implementation
    }
    
    /// Verify a resource operation
    pub fn verify_operation(&self, operation: &ResourceOperation) -> Result<bool> {
        // Implementation
    }
    
    /// Run garbage collection
    pub fn collect_garbage(&mut self) -> Result<CollectionStats> {
        // Implementation
    }
}
```

2. **Virtual Machine Integration** - For executing resource operations in the VM:

```rust
/// Integrates resources with the Causality VM
pub struct ResourceVmIntegration {
    /// Reference to VM runtime
    vm_runtime: Arc<VirtualMachine>,
    /// Register access control
    access_control: ResourceAccessControl,
    /// Operation circuit generators
    operation_circuits: HashMap<OperationType, Arc<dyn CircuitGenerator>>,
}

impl ResourceVmIntegration {
    /// Execute a resource operation in the VM
    pub fn execute_operation(&self, operation: &ResourceOperation) -> Result<VmWitness> {
        // Implementation
    }
    
    /// Generate a circuit for an operation type
    pub fn generate_circuit(&self, op_type: OperationType, params: CircuitParams) -> Result<Circuit> {
        // Implementation
    }
    
    /// Verify an operation witness
    pub fn verify_witness(&self, operation: &ResourceOperation, witness: &VmWitness) -> Result<bool> {
        // Implementation
    }
}
```

3. **Zero-Knowledge Verifier** - For verifying proofs of resource operations:

```rust
/// Verifies zero-knowledge proofs for resource operations
pub struct ZkVerifier {
    /// Backend for proof verification
    backend: VerifierBackend,
}

impl ZkVerifier {
    /// Verify a resource operation proof
    pub fn verify_proof(&self, proof: &Proof, public_inputs: &[Vec<u8>]) -> Result<bool> {
        // Implementation
    }
    
    /// Batch verify multiple proofs
    pub fn batch_verify(&self, proofs: &[Proof], public_inputs: &[Vec<Vec<u8>>]) -> Result<bool> {
        // Implementation
    }
}
```

4. **Resource Guards** - Safe access handles for resources:

```rust
/// A guard providing safe access to a resource
pub struct ResourceGuard<'a> {
    /// Reference to the manager
    manager: &'a mut ResourceManager,
    /// ID of the register
    register_id: RegisterID,
}

impl<'a> ResourceGuard<'a> {
    /// Get the contents of the resource
    pub fn contents(&self) -> &RegisterContents {
        // Implementation
    }
    
    /// Update the resource contents
    pub fn update(&mut self, new_contents: RegisterContents) -> Result<()> {
        // Implementation
    }
    
    /// Release the guard (called on drop)
    fn release(self) {
        // Implementation
    }
}
```

5. **Resource Snapshot System** - For capturing resource state for recovery:

```rust
/// A snapshot of the resource system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    /// Snapshot ID
    pub snapshot_id: SnapshotID,
    /// Time of snapshot creation
    pub creation_time: Timestamp,
    /// Registers at snapshot time
    pub registers: HashMap<RegisterID, Register>,
    /// Operation log up to snapshot
    pub operations_log: Vec<OperationId>,
    /// Content hash of the snapshot
    pub content_hash: ContentHash,
}

impl ResourceSnapshot {
    /// Create a snapshot from the current state
    pub fn create(manager: &ResourceManager) -> Result<Self> {
        // Implementation
    }
    
    /// Restore a manager from snapshot
    pub fn restore(&self) -> Result<ResourceManager> {
        // Implementation
    }
    
    /// Verify snapshot integrity
    pub fn verify(&self) -> Result<bool> {
        // Implementation
    }
}
```

6. **Performance Optimization Components**:

```rust
/// Optimization settings for resource operations
pub struct ResourceOptimizationConfig {
    /// Whether to batch proofs when possible
    pub batch_proofs: bool,
    /// Maximum batch size for proof verification
    pub max_batch_size: usize,
    /// Whether to cache verification keys
    pub cache_verification_keys: bool,
    /// Whether to precompute common circuits
    pub precompute_circuits: bool,
    /// Memory limit for proof generation
    pub proof_memory_limit: usize,
}

/// Batching optimization for resource operations
pub struct ResourceBatcher {
    /// Current batch of operations
    operations: Vec<ResourceOperation>,
    /// Batch configuration
    config: BatchConfig,
    /// Aggregated proof (if applicable)
    aggregated_proof: Option<Proof>,
}

impl ResourceBatcher {
    /// Add an operation to the batch
    pub fn add(&mut self, operation: ResourceOperation) -> Result<()> {
        // Implementation
    }
    
    /// Process the current batch
    pub fn process(&mut self) -> Result<BatchResult> {
        // Implementation
    }
    
    /// Clear the current batch
    pub fn clear(&mut self) {
        // Implementation
    }
}
```

## Security Considerations

### Register Access Control

Resource security depends on proper access control:

```rust
/// Access control for resource registers
pub struct ResourceAccessControl {
    /// Access control lists for registers
    acls: HashMap<RegisterID, AccessControlList>,
    /// Default access policy
    default_policy: AccessPolicy,
}

/// Access control list for a register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlList {
    /// Owner of the register
    pub owner: Address,
    /// Read permissions
    pub read_access: HashSet<Address>,
    /// Write permissions
    pub write_access: HashSet<Address>,
    /// Admin permissions
    pub admin_access: HashSet<Address>,
    /// Public access flag
    pub is_public_readable: bool,
}
```

### Attack Vectors and Mitigations

1. **Double-Spending**: Prevented through nullifier tracking for spent resources
2. **Replay Attacks**: Mitigated via unique operation IDs and nonces
3. **Front-Running**: Addressed with temporal ordering through the time system
4. **Register Squatting**: Prevented with authorization requirements for register creation
5. **Eclipse Attacks**: Mitigated through controller label verification
6. **Resource Lock Abuse**: Resolved with automatic expiry for locks

### Recovery Mechanisms

Resources support recovery mechanisms:

```rust
/// Recovery options for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryMethod {
    /// Recovery via backup controllers
    BackupController {
        /// Authorized backup controllers
        controllers: Vec<ControllerID>,
        /// Quorum needed for recovery
        quorum: usize,
    },
    /// Recovery via social mechanisms
    SocialRecovery {
        /// Guardian addresses
        guardians: Vec<Address>,
        /// Required threshold
        threshold: usize,
        /// Delay before recovery completes
        delay: Duration,
    },
    /// Recovery via cryptographic backdoor
    CryptographicRecovery {
        /// Recovery public key
        recovery_key: PublicKey,
        /// Encrypted private data
        encrypted_data: Vec<u8>,
    },
}
```

## Performance Considerations

### Proof Generation Optimization

The resource system employs techniques to optimize ZK proof generation:

1. **Circuit Reuse**: Common operation types use precomputed circuits
2. **Proof Batching**: Multiple operations are batched where possible
3. **Incremental Proving**: Updates to resources use incremental proving
4. **Witness Reuse**: Shared witnesses are reused across related proofs
5. **Parallel Proof Generation**: Operations are proven in parallel

### Resource Access Optimization

Efficient resource access is achieved through:

1. **Register Caching**: Frequently accessed registers are cached
2. **Lazy Loading**: Register contents are loaded on demand
3. **Hierarchical Storage**: Registers use tiered storage based on access patterns
4. **Content-Addressing**: Resources are stored and retrieved by content hash
5. **Register Prefetching**: Predictive loading of related registers

## Migration Strategy

Existing resources will be migrated to the register system through:

1. **Inventory Phase**: All existing resources are inventoried
2. **Register Creation**: Each resource gets allocated to a register
3. **Ownership Verification**: Ownership claims are verified
4. **Controller Labeling**: Cross-domain resources receive controller labels
5. **Validation**: All registers undergo validation
6. **Gradual Rollout**: Migration occurs domain by domain

## Consequences

### Positive

- Formal guarantee of resource conservation across operations
- Clear resource ownership through register system
- ZK proofs provide strong verification with privacy
- Improved security through formalized access control
- Composable resource operations
- Cross-domain resource tracking
- Better tracking of resource provenance
- Enhanced auditability through register operations log
- Simplified resource management for program developers
- Recoverable resource state
- Efficient garbage collection

### Negative

- Additional complexity in implementing resource operations
- Computational overhead for ZK proof generation and verification
- Learning curve for developers accustomed to direct resource access
- ZK circuit complexity for complex resource transformations
- Storage requirements for operation logs
- Performance impact of constant state verification

### Neutral

- Requires standardization of resource representations across domains
- May need extensions for specialized resource types
- Evolution of ZK technology may require updates
- May need periodic updates to authorization methods as security practices evolve
- Interactions with time system add complexity but also security 