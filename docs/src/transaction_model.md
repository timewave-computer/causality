# Unified Transaction Model

This document describes the unified transaction model within the Causality architecture, detailing how transactions are structured, validated, and processed across domains.

## Core Concepts

### Transaction Overview

A **Transaction** is an atomic unit of work that encapsulates one or more operations. Transactions in Causality provide:

1. **Atomicity**: All operations in a transaction either succeed or fail together
2. **Consistency**: Transactions maintain system invariants
3. **Isolation**: Concurrent transactions don't interfere with each other
4. **Durability**: Once committed, transaction effects are permanent
5. **Verifiability**: Transactions can be cryptographically verified

### Transaction Structure

```rust
/// A unified transaction
pub struct Transaction {
    /// Unique identifier
    id: TransactionId,
    
    /// Operations contained in this transaction
    operations: Vec<Operation>,
    
    /// Transaction metadata
    metadata: TransactionMetadata,
    
    /// Authentication data
    authentication: Authentication,
    
    /// Execution context
    context: ExecutionContext,
    
    /// Transaction status
    status: TransactionStatus,
    
    /// Results of operation execution
    results: Option<Vec<OperationResult>>,
}

/// Metadata for a transaction
pub struct TransactionMetadata {
    /// Transaction nonce for uniqueness
    nonce: u64,
    
    /// Submitter of the transaction
    submitter: ActorIdBox,
    
    /// Timestamp when the transaction was created
    timestamp: Timestamp,
    
    /// Expiration time
    expiration: Option<Timestamp>,
    
    /// Maximum gas limit
    gas_limit: Option<u64>,
    
    /// Fee information
    fee: Option<Fee>,
    
    /// Chain or domain ID
    domain_id: Option<DomainId>,
    
    /// Transaction type
    transaction_type: TransactionType,
    
    /// Additional metadata
    additional_metadata: HashMap<String, Value>,
}

/// Authentication data for a transaction
pub struct Authentication {
    /// Authentication type
    auth_type: AuthenticationType,
    
    /// Signatures
    signatures: Vec<Signature>,
    
    /// Authorization proofs
    auth_proofs: Vec<AuthorizationProof>,
    
    /// Additional authentication data
    additional_data: Option<Value>,
}
```

### Transaction Lifecycle

A transaction goes through several phases during its lifecycle:

```
┌─────────────┐       ┌─────────────┐       ┌─────────────┐       ┌─────────────┐
│             │       │             │       │             │       │             │
│  Creation   │──────►│ Validation  │──────►│  Execution  │──────►│ Finalization│
│             │       │             │       │             │       │             │
└─────────────┘       └──────┬──────┘       └──────┬──────┘       └─────────────┘
                             │                     │
                             ▼                     ▼
                      ┌─────────────┐       ┌─────────────┐
                      │             │       │             │
                      │  Rejection  │       │  Rollback   │
                      │             │       │             │
                      └─────────────┘       └─────────────┘
```

## Transaction Components

### Transaction Types

Different transaction types serve various purposes:

```rust
/// Types of transactions
pub enum TransactionType {
    /// Standard transaction with operations
    Standard,
    
    /// Cross-domain transaction affecting multiple domains
    CrossDomain {
        /// Source domain
        source_domain: DomainId,
        /// Target domains
        target_domains: Vec<DomainId>,
        /// Coordination strategy
        strategy: CrossDomainStrategy,
    },
    
    /// System transaction for internal operations
    System {
        /// System transaction type
        system_type: SystemTransactionType,
    },
    
    /// Batch transaction containing multiple sub-transactions
    Batch {
        /// Execution mode
        mode: BatchExecutionMode,
    },
    
    /// Deferred transaction for future execution
    Deferred {
        /// Trigger condition
        trigger: DeferredTrigger,
    },
    
    /// Custom transaction type
    Custom {
        /// Custom type name
        name: String,
        /// Type-specific data
        data: Value,
    },
}
```

### Fee Model

Transactions can include fee information:

```rust
/// Fee information for a transaction
pub struct Fee {
    /// Fee amount
    amount: u64,
    
    /// Fee token
    token: Option<ResourceId>,
    
    /// Gas price (if gas-based)
    gas_price: Option<u64>,
    
    /// Fee payer (if different from submitter)
    payer: Option<ActorIdBox>,
    
    /// Fee model
    model: FeeModel,
}

/// Fee models for transactions
pub enum FeeModel {
    /// Fixed fee regardless of execution cost
    Fixed,
    
    /// Fee based on gas consumption
    GasBased,
    
    /// Fee based on resource consumption
    ResourceBased,
    
    /// Priority-based fee model
    PriorityBased {
        /// Priority level
        priority: u8,
    },
}
```

### Authentication

Transactions can use different authentication methods:

```rust
/// Authentication types for transactions
pub enum AuthenticationType {
    /// Single signature authentication
    SingleSignature,
    
    /// Multi-signature authentication
    MultiSignature {
        /// Threshold of required signatures
        threshold: u32,
    },
    
    /// Zero-knowledge authentication
    ZeroKnowledge {
        /// Proof system used
        proof_system: String,
    },
    
    /// Account abstraction (programmable validation)
    AccountAbstraction {
        /// Validation logic reference
        validation_reference: String,
    },
}

/// A cryptographic signature
pub struct Signature {
    /// Signer ID
    signer: ActorIdBox,
    
    /// Signature data
    data: Vec<u8>,
    
    /// Signature scheme
    scheme: SignatureScheme,
    
    /// Public key (if needed)
    public_key: Option<Vec<u8>>,
}
```

## Transaction Processing

### Transaction Manager

The transaction manager oversees the transaction lifecycle:

```rust
/// Manages transaction processing
pub struct TransactionManager {
    /// Transaction store
    store: Arc<TransactionStore>,
    
    /// Transaction scheduler
    scheduler: Arc<TransactionScheduler>,
    
    /// Transaction validator
    validator: Arc<TransactionValidator>,
    
    /// Transaction executor
    executor: Arc<TransactionExecutor>,
    
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
}

impl TransactionManager {
    /// Submit a transaction for processing
    pub async fn submit_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionSubmissionResult>;
    
    /// Get the status of a transaction
    pub async fn get_transaction_status(
        &self,
        transaction_id: &TransactionId,
    ) -> Result<TransactionStatus>;
    
    /// Get transaction by ID
    pub async fn get_transaction(
        &self,
        transaction_id: &TransactionId,
    ) -> Result<Option<Transaction>>;
    
    /// Cancel a pending transaction
    pub async fn cancel_transaction(
        &self,
        transaction_id: &TransactionId,
        canceler: &ActorIdBox,
    ) -> Result<CancellationResult>;
}
```

### Transaction Validation

Transactions undergo multiple validation steps:

```rust
/// Validates transactions
pub struct TransactionValidator {
    /// Structure validator
    structure_validator: Box<dyn StructureValidator>,
    
    /// Authentication validator
    auth_validator: Box<dyn AuthenticationValidator>,
    
    /// Operation validator
    operation_validator: Box<dyn OperationValidator>,
    
    /// Domain validators
    domain_validators: HashMap<DomainId, Box<dyn DomainValidator>>,
    
    /// Policy validators
    policy_validators: Vec<Box<dyn PolicyValidator>>,
}

impl TransactionValidator {
    /// Validate a transaction
    pub async fn validate_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<ValidationResult>;
    
    /// Check if a transaction conforms to domain rules
    pub async fn validate_domain_rules(
        &self,
        transaction: &Transaction,
        domain_id: &DomainId,
    ) -> Result<DomainValidationResult>;
}
```

### Transaction Execution

Transaction execution is managed through an execution pipeline:

```rust
/// Executes transactions
pub struct TransactionExecutor {
    /// Operation executor
    operation_executor: Arc<OperationExecutor>,
    
    /// Transaction context provider
    context_provider: Arc<TransactionContextProvider>,
    
    /// Cross-domain executor (for cross-domain transactions)
    cross_domain_executor: Option<Arc<CrossDomainExecutor>>,
    
    /// Execution pipeline
    pipeline: ExecutionPipeline,
}

impl TransactionExecutor {
    /// Execute a transaction
    pub async fn execute_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<TransactionExecutionResult>;
    
    /// Execute a batch of transactions
    pub async fn execute_batch(
        &self,
        batch: &[Transaction],
        mode: BatchExecutionMode,
    ) -> Result<BatchExecutionResult>;
    
    /// Rollback a transaction
    pub async fn rollback_transaction(
        &self,
        transaction_id: &TransactionId,
    ) -> Result<RollbackResult>;
}
```

## Transaction Integration

### Operation Integration

Transactions incorporate operations from the unified operation model:

```rust
/// Build a transaction from operations
pub fn build_transaction(
    operations: Vec<Operation>,
    submitter: ActorIdBox,
    domain_id: Option<DomainId>,
) -> Result<Transaction> {
    // Generate a unique ID
    let id = TransactionId::new();
    
    // Create metadata
    let metadata = TransactionMetadata::new(
        0, // Nonce will be set by the transaction manager
        submitter.clone(),
        Timestamp::now(),
        None, // No expiration
        None, // Default gas limit
        None, // No fee specified
        domain_id,
        TransactionType::Standard,
        HashMap::new(),
    );
    
    // Create execution context
    let context = ExecutionContext::new(ExecutionPhase::Planning)
        .with_environment(ExecutionEnvironment::Production)
        .with_invoker(submitter.clone());
    
    // Create the transaction
    let transaction = Transaction::new(
        id,
        operations,
        metadata,
        Authentication::empty(), // Authentication will be added later
        context,
    );
    
    Ok(transaction)
}
```

### Resource Integration

Transactions often include resource operations:

```rust
/// Create a transaction for resource transfer
pub fn create_resource_transfer_transaction(
    source_resource: ResourceId,
    destination_resource: ResourceId,
    amount: u64,
    submitter: ActorIdBox,
    capabilities: Vec<CapabilityId>,
) -> Result<Transaction> {
    // Create the transfer effect
    let transfer_effect = TransferEffect::new(
        source_resource.clone(),
        destination_resource.clone(),
        amount,
        HashMap::new(),
    );
    
    // Create the operation
    let operation = Operation::new(OperationType::TransferResource)
        .with_input(source_resource)
        .with_output(destination_resource)
        .with_abstract_representation(Box::new(transfer_effect))
        .with_authorization(Authorization::with_capabilities(
            submitter.clone(),
            capabilities,
        ));
    
    // Build the transaction
    build_transaction(vec![operation], submitter, None)
}
```

### Cross-Domain Integration

Transactions can span multiple domains:

```rust
/// Create a cross-domain transaction
pub fn create_cross_domain_transaction(
    operations: Vec<Operation>,
    submitter: ActorIdBox,
    source_domain: DomainId,
    target_domains: Vec<DomainId>,
    strategy: CrossDomainStrategy,
) -> Result<Transaction> {
    // Create metadata with cross-domain information
    let metadata = TransactionMetadata::new(
        0, // Nonce will be set by the transaction manager
        submitter.clone(),
        Timestamp::now(),
        None, // No expiration
        None, // Default gas limit
        None, // No fee specified
        Some(source_domain.clone()),
        TransactionType::CrossDomain {
            source_domain: source_domain.clone(),
            target_domains: target_domains.clone(),
            strategy: strategy.clone(),
        },
        HashMap::new(),
    );
    
    // Create execution context
    let context = ExecutionContext::new(ExecutionPhase::Planning)
        .with_environment(ExecutionEnvironment::Production)
        .with_invoker(submitter.clone())
        .with_domain(source_domain);
    
    // Create the transaction
    let transaction = Transaction::new(
        TransactionId::new(),
        operations,
        metadata,
        Authentication::empty(), // Authentication will be added later
        context,
    );
    
    Ok(transaction)
}
```

## Advanced Transaction Features

### Batch Transactions

Batch transactions combine multiple transactions into one:

```rust
/// Create a batch transaction
pub fn create_batch_transaction(
    transactions: Vec<Transaction>,
    mode: BatchExecutionMode,
    submitter: ActorIdBox,
) -> Result<Transaction> {
    // Extract operations from all transactions
    let mut all_operations = Vec::new();
    for transaction in &transactions {
        all_operations.extend(transaction.operations().clone());
    }
    
    // Create metadata
    let metadata = TransactionMetadata::new(
        0, // Nonce will be set by the transaction manager
        submitter.clone(),
        Timestamp::now(),
        None, // No expiration
        None, // Default gas limit
        None, // No fee specified
        None, // No specific domain
        TransactionType::Batch { mode },
        HashMap::new(),
    );
    
    // Create the batch transaction
    let batch_transaction = Transaction::new(
        TransactionId::new(),
        all_operations,
        metadata,
        Authentication::empty(), // Authentication will be added later
        ExecutionContext::new(ExecutionPhase::Planning),
    );
    
    Ok(batch_transaction)
}
```

### Deferred Transactions

Transactions can be scheduled for future execution:

```rust
/// Create a deferred transaction
pub fn create_deferred_transaction(
    operations: Vec<Operation>,
    submitter: ActorIdBox,
    trigger: DeferredTrigger,
) -> Result<Transaction> {
    // Create metadata with deferred information
    let metadata = TransactionMetadata::new(
        0, // Nonce will be set by the transaction manager
        submitter.clone(),
        Timestamp::now(),
        None, // No expiration
        None, // Default gas limit
        None, // No fee specified
        None, // No specific domain
        TransactionType::Deferred { trigger: trigger.clone() },
        HashMap::new(),
    );
    
    // Create the transaction
    let transaction = Transaction::new(
        TransactionId::new(),
        operations,
        metadata,
        Authentication::empty(), // Authentication will be added later
        ExecutionContext::new(ExecutionPhase::Planning),
    );
    
    Ok(transaction)
}

/// Trigger for deferred transactions
pub enum DeferredTrigger {
    /// Execute at a specific time
    Time(Timestamp),
    
    /// Execute when a block height is reached
    BlockHeight {
        /// Domain ID
        domain_id: DomainId,
        /// Target block height
        height: u64,
    },
    
    /// Execute when a condition is met
    Condition(Box<dyn Condition>),
    
    /// Execute when a fact is observed
    FactObservation {
        /// Fact pattern to observe
        pattern: FactPattern,
    },
}
```

### Multi-Signature Transactions

Transactions can require multiple signatures:

```rust
/// Add multi-signature authentication to a transaction
pub fn add_multi_signature_authentication(
    transaction: &mut Transaction,
    signers: Vec<ActorIdBox>,
    threshold: u32,
) -> Result<()> {
    // Create multi-signature authentication
    let authentication = Authentication::new(
        AuthenticationType::MultiSignature { threshold },
        Vec::new(), // No signatures yet
        Vec::new(), // No authorization proofs yet
        None,
    );
    
    // Set the authentication
    transaction.set_authentication(authentication);
    
    // Store signers in metadata
    transaction.metadata_mut().insert("signers", signers_to_json(&signers));
    
    Ok(())
}

/// Add a signature to a multi-signature transaction
pub fn add_signature(
    transaction: &mut Transaction,
    signer: &ActorIdBox,
    signature_data: Vec<u8>,
    scheme: SignatureScheme,
) -> Result<()> {
    // Create the signature
    let signature = Signature::new(
        signer.clone(),
        signature_data,
        scheme,
        None, // No public key provided
    );
    
    // Add the signature
    transaction.authentication_mut().add_signature(signature);
    
    Ok(())
}
```

### Transaction Batching

Transactions can be batched for efficiency:

```rust
/// Batch execution modes
pub enum BatchExecutionMode {
    /// All transactions must succeed
    AllOrNothing,
    
    /// Continue executing even if some transactions fail
    ContinueOnError,
    
    /// Execute until the first failure
    StopOnError,
    
    /// Parallel execution (when possible)
    Parallel,
}
```

## Authorization and Security

### Transaction Authorization

Transactions require proper authorization:

```rust
/// Check if a transaction is properly authorized
pub async fn verify_transaction_authorization(
    transaction: &Transaction,
    authorization_service: &AuthorizationService,
) -> Result<bool> {
    // Verify authentication
    let auth_valid = verify_authentication(transaction).await?;
    if !auth_valid {
        return Ok(false);
    }
    
    // Verify operation authorizations
    for operation in transaction.operations() {
        let auth_result = authorization_service.is_authorized(
            transaction.submitter(),
            operation,
            &transaction.context(),
        ).await?;
        
        if !auth_result.authorized {
            return Ok(false);
        }
    }
    
    Ok(true)
}
```

### Transaction Security

Transactions include security features:

```rust
/// Security features for transactions
pub struct TransactionSecurity {
    /// Apply replay protection
    pub fn apply_replay_protection(
        transaction: &mut Transaction,
        nonce_provider: &NonceProvider,
    ) -> Result<()> {
        // Get the next nonce for the submitter
        let nonce = nonce_provider.get_next_nonce(transaction.submitter())?;
        
        // Set the nonce in the transaction
        transaction.metadata_mut().set_nonce(nonce);
        
        Ok(())
    }
    
    /// Apply expiration
    pub fn apply_expiration(
        transaction: &mut Transaction,
        ttl: Duration,
    ) -> Result<()> {
        // Calculate expiration time
        let expiration = transaction.timestamp() + ttl;
        
        // Set the expiration
        transaction.metadata_mut().set_expiration(Some(expiration));
        
        Ok(())
    }
}
```

## Usage Examples

### Basic Transaction

Creating and submitting a basic transaction:

```rust
// Create a resource transfer effect
let transfer_effect = TransferEffect::new(
    source_resource.clone(),
    destination_resource.clone(),
    100,
    HashMap::new(),
);

// Create the operation
let operation = Operation::new(OperationType::TransferResource)
    .with_abstract_representation(Box::new(transfer_effect))
    .with_authorization(Authorization::with_capabilities(
        user_id.clone(),
        vec![transfer_capability]
    ));

// Build the transaction
let mut transaction = build_transaction(
    vec![operation],
    user_id.clone(),
    Some(domain_id.clone())
)?;

// Apply security features
TransactionSecurity::apply_replay_protection(&mut transaction, &nonce_provider)?;
TransactionSecurity::apply_expiration(&mut transaction, Duration::from_secs(3600))?;

// Sign the transaction
let signature = crypto_service.sign(
    &private_key,
    &transaction.to_bytes()?
)?;

let auth = Authentication::new(
    AuthenticationType::SingleSignature,
    vec![Signature::new(
        user_id.clone(),
        signature,
        SignatureScheme::Ed25519,
        None
    )],
    Vec::new(),
    None
);

transaction.set_authentication(auth);

// Submit the transaction
let result = transaction_manager.submit_transaction(transaction).await?;

println!("Transaction submitted with ID: {}", result.transaction_id);
```

### Cross-Domain Transaction

Creating and submitting a cross-domain transaction:

```rust
// Create a cross-domain transfer operation
let cross_domain_operation = cross_domain_service.create_cross_domain_transfer(
    &source_resource,
    &source_domain,
    &target_domain,
    "recipient_account",
    1000
)?;

// Create the cross-domain transaction
let mut transaction = create_cross_domain_transaction(
    vec![cross_domain_operation],
    user_id.clone(),
    source_domain.clone(),
    vec![target_domain.clone()],
    CrossDomainStrategy::AtomicCommit {
        timeout: Duration::from_secs(300),
        verification_level: VerificationLevel::Full,
    }
)?;

// Apply security features
TransactionSecurity::apply_replay_protection(&mut transaction, &nonce_provider)?;
TransactionSecurity::apply_expiration(&mut transaction, Duration::from_secs(3600))?;

// Sign the transaction
let signature = crypto_service.sign(
    &private_key,
    &transaction.to_bytes()?
)?;

let auth = Authentication::new(
    AuthenticationType::SingleSignature,
    vec![Signature::new(
        user_id.clone(),
        signature,
        SignatureScheme::Ed25519,
        None
    )],
    Vec::new(),
    None
);

transaction.set_authentication(auth);

// Submit the transaction
let result = transaction_manager.submit_transaction(transaction).await?;

// Monitor the transaction status
let status = transaction_manager.get_transaction_status(&result.transaction_id).await?;
println!("Transaction status: {:?}", status);
```

### Multi-Signature Transaction

Creating and processing a multi-signature transaction:

```rust
// Create resource update operations
let update_effect = UpdateEffect::new(
    resource_id.clone(),
    update_properties,
    HashMap::new(),
);

let operation = Operation::new(OperationType::UpdateResource)
    .with_abstract_representation(Box::new(update_effect))
    .with_authorization(Authorization::with_capabilities(
        org_id.clone(),
        vec![admin_capability]
    ));

// Build the transaction
let mut transaction = build_transaction(
    vec![operation],
    org_id.clone(),
    Some(domain_id.clone())
)?;

// Set up multi-signature authentication
add_multi_signature_authentication(
    &mut transaction,
    vec![user1_id.clone(), user2_id.clone(), user3_id.clone()],
    2 // Threshold: 2 of 3 signatures required
)?;

// Apply security features
TransactionSecurity::apply_replay_protection(&mut transaction, &nonce_provider)?;
TransactionSecurity::apply_expiration(&mut transaction, Duration::from_secs(86400))?;

// Submit the incomplete transaction
let partial_result = transaction_manager.submit_transaction(transaction.clone()).await?;

// First user signs
let signature1 = crypto_service.sign(
    &user1_private_key,
    &transaction.to_bytes()?
)?;

let signature_result1 = transaction_manager.add_signature(
    &partial_result.transaction_id,
    &user1_id,
    signature1,
    SignatureScheme::Ed25519
).await?;

// Second user signs
let signature2 = crypto_service.sign(
    &user2_private_key,
    &transaction.to_bytes()?
)?;

let signature_result2 = transaction_manager.add_signature(
    &partial_result.transaction_id,
    &user2_id,
    signature2,
    SignatureScheme::Ed25519
).await?;

// Transaction automatically executes when signature threshold is reached
let final_status = transaction_manager.get_transaction_status(&partial_result.transaction_id).await?;
println!("Final transaction status: {:?}", final_status);
```

### Batch Transaction

Creating and submitting a batch transaction:

```rust
// Create multiple transfer operations
let transfer1 = create_resource_transfer_transaction(
    source1.clone(),
    destination1.clone(),
    100,
    user_id.clone(),
    vec![transfer_capability1]
)?;

let transfer2 = create_resource_transfer_transaction(
    source2.clone(),
    destination2.clone(),
    200,
    user_id.clone(),
    vec![transfer_capability2]
)?;

let transfer3 = create_resource_transfer_transaction(
    source3.clone(),
    destination3.clone(),
    300,
    user_id.clone(),
    vec![transfer_capability3]
)?;

// Create a batch transaction
let mut batch_transaction = create_batch_transaction(
    vec![transfer1, transfer2, transfer3],
    BatchExecutionMode::AllOrNothing,
    user_id.clone()
)?;

// Apply security features
TransactionSecurity::apply_replay_protection(&mut batch_transaction, &nonce_provider)?;
TransactionSecurity::apply_expiration(&mut batch_transaction, Duration::from_secs(3600))?;

// Sign the batch transaction
let signature = crypto_service.sign(
    &private_key,
    &batch_transaction.to_bytes()?
)?;

let auth = Authentication::new(
    AuthenticationType::SingleSignature,
    vec![Signature::new(
        user_id.clone(),
        signature,
        SignatureScheme::Ed25519,
        None
    )],
    Vec::new(),
    None
);

batch_transaction.set_authentication(auth);

// Submit the batch transaction
let result = transaction_manager.submit_transaction(batch_transaction).await?;

// Check the result
let transaction = transaction_manager.get_transaction(&result.transaction_id).await?
    .ok_or(Error::transaction_not_found(result.transaction_id))?;

let batch_success = transaction.status() == TransactionStatus::Committed;
println!("Batch transaction success: {}", batch_success);
```

## Implementation Status

The unified transaction model is mostly implemented:

- ✅ Core transaction structure
- ✅ Transaction validation
- ✅ Transaction execution
- ✅ Multi-signature support
- ✅ Basic cross-domain transactions
- ✅ Transaction security features
- ⚠️ Deferred transactions (in progress)
- ⚠️ Batch transaction optimizations (in progress)
- ⚠️ Fee models (in progress)
- ❌ ZK-based transaction authentication
- ❌ Account abstraction

## Future Enhancements

1. **Transaction Streaming**: Support for streaming large transactions
2. **Smart Transaction Routing**: Automatic routing of transactions to appropriate domains
3. **Transaction Privacy**: Enhanced privacy features for transactions
4. **Adaptive Batching**: Dynamic batching based on transaction characteristics
5. **Gas Optimizations**: Advanced gas estimation and optimization
6. **Account Abstraction**: Programmable validation for transactions
7. **Parallel Transaction Execution**: Concurrent execution of independent operations 