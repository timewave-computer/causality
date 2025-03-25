<!-- Lifecycle of transactions -->
<!-- Original file: docs/src/transaction_lifecycle.md -->

# Transaction Lifecycle in Causality

## Overview

This document describes the complete lifecycle of transactions within the Causality architecture. Transactions represent groups of operations that need to be executed atomically, ensuring that either all operations succeed or none of them take effect. The transaction lifecycle encompasses all stages from creation through validation, execution, commitment, and observation.

## Core Concepts

### Transaction Model

Transactions are represented by the following core model:

```rust
pub struct Transaction {
    /// Unique identifier for this transaction
    id: TransactionId,
    
    /// Operations included in this transaction
    operations: Vec<Operation>,
    
    /// Transaction status
    status: TransactionStatus,
    
    /// Authentication information
    auth_info: AuthInfo,
    
    /// Metadata associated with this transaction
    metadata: TransactionMetadata,
    
    /// Creation timestamp
    created_at: Timestamp,
    
    /// Expiration timestamp
    expires_at: Option<Timestamp>,
}

pub enum TransactionStatus {
    /// Transaction has been created but not submitted
    Created,
    
    /// Transaction has been submitted for processing
    Submitted,
    
    /// Transaction is being processed
    Processing,
    
    /// Transaction has been validated
    Validated,
    
    /// Transaction is being executed
    Executing,
    
    /// Transaction has been committed
    Committed,
    
    /// Transaction has been aborted
    Aborted,
    
    /// Transaction has failed
    Failed(String),
    
    /// Transaction has expired
    Expired,
}
```

## Lifecycle Stages

### 1. Transaction Creation

Transactions begin their lifecycle through creation:

```rust
/// Creates a new transaction
pub fn create_transaction(
    auth_info: AuthInfo,
    metadata: Option<TransactionMetadata>,
    expires_after: Option<Duration>,
) -> Transaction {
    let current_time = system.current_time();
    
    Transaction {
        id: TransactionId::generate(),
        operations: Vec::new(),
        status: TransactionStatus::Created,
        auth_info,
        metadata: metadata.unwrap_or_default(),
        created_at: current_time,
        expires_at: expires_after.map(|duration| current_time + duration),
    }
}
```

### 2. Operation Addition

Operations are added to the transaction:

```rust
impl Transaction {
    /// Add an operation to this transaction
    pub fn add_operation(&mut self, operation: Operation) -> Result<&mut Self, TransactionError> {
        // Check if transaction is still modifiable
        if !self.is_modifiable() {
            return Err(TransactionError::InvalidState(
                "Transaction is no longer modifiable".to_string(),
            ));
        }
        
        // Add the operation
        self.operations.push(operation);
        
        Ok(self)
    }
    
    /// Is this transaction still modifiable?
    fn is_modifiable(&self) -> bool {
        matches!(self.status, TransactionStatus::Created)
    }
}
```

### 3. Submission

After creation and adding operations, transactions are submitted:

```rust
/// Submits a transaction for processing
pub fn submit_transaction(
    transaction: Transaction,
) -> Result<TransactionReceipt, TransactionError> {
    // Check if expired
    if let Some(expires_at) = transaction.expires_at {
        if expires_at <= system.current_time() {
            return Err(TransactionError::Expired);
        }
    }
    
    // Update status
    let mut tx = transaction;
    tx.status = TransactionStatus::Submitted;
    
    // Log transaction submission
    system.transaction_logger().log_submission(&tx)?;
    
    // Place in transaction queue
    system.transaction_queue().enqueue(tx.clone())?;
    
    // Return receipt
    Ok(TransactionReceipt {
        transaction_id: tx.id,
        submission_time: system.current_time(),
        status: tx.status,
    })
}
```

### 4. Validation

Transactions undergo validation to ensure correctness:

```rust
/// Validates a transaction
pub fn validate_transaction(
    transaction: &Transaction,
    validation_context: &ValidationContext,
) -> Result<ValidationResult, ValidationError> {
    // Validate each operation first
    let mut operation_results = Vec::new();
    for operation in &transaction.operations {
        let operation_result = validate_operation(operation, validation_context)?;
        operation_results.push(operation_result);
        
        // If operation is invalid, fail early
        if !operation_result.is_valid() {
            return Ok(ValidationResult::invalid(
                format!("Operation {} is invalid: {}", 
                    operation.id, 
                    operation_result.error_messages().join(", ")
                )
            ));
        }
    }
    
    // Validate transaction-level constraints
    let transaction_validators = system.transaction_registry().get_validators();
    let mut validation_results = Vec::new();
    
    for validator in transaction_validators {
        let result = validator.validate_transaction(transaction, validation_context)?;
        validation_results.push(result);
        
        // If validator reports invalid, fail early
        if !result.is_valid() {
            return Ok(result);
        }
    }
    
    // All validations passed
    Ok(ValidationResult::valid())
}
```

### 5. Operation Ordering

Before execution, operations within the transaction are ordered:

```rust
/// Orders operations within a transaction for proper execution
pub fn order_operations(
    transaction: &Transaction,
) -> Result<Vec<Operation>, TransactionError> {
    // Create a dependency graph
    let mut graph = DependencyGraph::new();
    
    // Add all operations as nodes
    for operation in &transaction.operations {
        graph.add_node(operation.id);
    }
    
    // Determine dependencies
    for (i, op1) in transaction.operations.iter().enumerate() {
        for (j, op2) in transaction.operations.iter().enumerate() {
            if i == j {
                continue; // Skip self
            }
            
            // Check if op1 depends on op2
            if operation_depends_on(op1, op2) {
                graph.add_edge(op1.id, op2.id);
            }
        }
    }
    
    // Check for cycles (would indicate impossible ordering)
    if graph.has_cycle() {
        return Err(TransactionError::CyclicDependency);
    }
    
    // Perform topological sort to get execution order
    let ordered_ids = graph.topological_sort()?;
    
    // Map back to operations
    let id_to_op: HashMap<_, _> = transaction.operations.iter()
        .map(|op| (op.id, op))
        .collect();
    
    let ordered_ops = ordered_ids.into_iter()
        .map(|id| id_to_op[&id].clone())
        .collect();
    
    Ok(ordered_ops)
}
```

### 6. Execution

Validated transactions are executed:

```rust
/// Executes a transaction
pub fn execute_transaction(
    transaction: &Transaction,
    execution_context: &ExecutionContext,
) -> Result<TransactionExecutionResult, ExecutionError> {
    // Get ordered operations
    let ordered_operations = order_operations(transaction)?;
    
    // Begin execution
    let mut results = Vec::new();
    let mut effects = Vec::new();
    let mut failed_operation: Option<(Operation, ExecutionError)> = None;
    
    // Execute each operation in order
    for operation in ordered_operations {
        // Execute the operation
        match execute_operation(&operation, execution_context) {
            Ok(result) => {
                // Generate effects
                let operation_effects = generate_effects(
                    &operation,
                    &result,
                    &execution_context.into(),
                )?;
                
                // Add to results and effects
                results.push((operation.id, result));
                effects.extend(operation_effects);
            }
            Err(error) => {
                failed_operation = Some((operation, error));
                break;
            }
        }
    }
    
    // If any operation failed, return error
    if let Some((operation, error)) = failed_operation {
        return Err(ExecutionError::OperationFailed {
            operation_id: operation.id,
            error: error.to_string(),
        });
    }
    
    // All operations executed successfully
    Ok(TransactionExecutionResult {
        transaction_id: transaction.id,
        operation_results: results,
        effects,
        timestamp: system.current_time(),
    })
}
```

### 7. Commitment

Transaction execution results are committed:

```rust
/// Commits a transaction
pub fn commit_transaction(
    transaction: &Transaction,
    execution_result: TransactionExecutionResult,
    context: &CommitContext,
) -> Result<CommitResult, CommitError> {
    // Begin database transaction
    let db_transaction = system.storage().begin_transaction()?;
    
    // Store transaction result
    system.transaction_storage().store_result(
        transaction.id,
        &execution_result,
        &db_transaction,
    )?;
    
    // Apply and store effects
    for effect in &execution_result.effects {
        system.effect_applier().apply_effect(effect, &db_transaction)?;
        system.effect_storage().store_effect(effect, &db_transaction)?;
    }
    
    // Store operation results
    for (operation_id, result) in &execution_result.operation_results {
        system.operation_storage().store_result(
            *operation_id,
            result,
            &db_transaction,
        )?;
    }
    
    // Commit database transaction
    system.storage().commit_transaction(db_transaction)?;
    
    // Update transaction status
    system.transaction_registry().update_status(
        transaction.id,
        TransactionStatus::Committed,
    )?;
    
    Ok(CommitResult {
        transaction_id: transaction.id,
        status: CommitStatus::Committed,
        effect_count: execution_result.effects.len(),
        timestamp: system.current_time(),
    })
}
```

### 8. Observation

Transactions and their effects can be observed after completion:

```rust
/// Observes a transaction's effects
pub fn observe_transaction_effects(
    transaction_id: TransactionId,
) -> Result<Vec<Effect>, ObservationError> {
    // Get effects for transaction
    system.effect_storage().get_effects_for_transaction(transaction_id)
}

/// Gets the result of a transaction
pub fn get_transaction_result(
    transaction_id: TransactionId,
) -> Result<TransactionExecutionResult, StorageError> {
    system.transaction_storage().get_result(transaction_id)
}
```

## Transaction Processor

The TransactionProcessor orchestrates the complete lifecycle:

```rust
pub struct TransactionProcessor {
    validation_service: Arc<ValidationService>,
    execution_service: Arc<ExecutionService>,
    commit_service: Arc<CommitService>,
}

impl TransactionProcessor {
    /// Process a transaction through its complete lifecycle
    pub fn process_transaction(
        &self,
        transaction: Transaction,
        context: &ProcessContext,
    ) -> Result<ProcessResult, ProcessError> {
        // Create transaction context
        let transaction_context = TransactionContext::from(context);
        
        // Update status to processing
        let mut tx = transaction;
        tx.status = TransactionStatus::Processing;
        system.transaction_registry().update_status(tx.id, tx.status.clone())?;
        
        // 1. Validate the transaction
        let validation_result = self.validation_service.validate_transaction(
            &tx,
            &transaction_context.into(),
        )?;
        
        if !validation_result.is_valid() {
            // Update status to failed
            tx.status = TransactionStatus::Failed(validation_result.error_messages().join(", "));
            system.transaction_registry().update_status(tx.id, tx.status.clone())?;
            
            return Err(ProcessError::ValidationFailed(validation_result));
        }
        
        // Update status to validated
        tx.status = TransactionStatus::Validated;
        system.transaction_registry().update_status(tx.id, tx.status.clone())?;
        
        // 2. Execute the transaction
        tx.status = TransactionStatus::Executing;
        system.transaction_registry().update_status(tx.id, tx.status.clone())?;
        
        let execution_result = match self.execution_service.execute_transaction(
            &tx,
            &transaction_context.into(),
        ) {
            Ok(result) => result,
            Err(e) => {
                // Update status to failed
                tx.status = TransactionStatus::Failed(e.to_string());
                system.transaction_registry().update_status(tx.id, tx.status.clone())?;
                
                return Err(ProcessError::ExecutionFailed(e));
            }
        };
        
        // 3. Commit the transaction
        let commit_result = match self.commit_service.commit_transaction(
            &tx,
            execution_result.clone(),
            &transaction_context.into(),
        ) {
            Ok(result) => result,
            Err(e) => {
                // Update status to failed
                tx.status = TransactionStatus::Failed(e.to_string());
                system.transaction_registry().update_status(tx.id, tx.status.clone())?;
                
                return Err(ProcessError::CommitFailed(e));
            }
        };
        
        // Update status to committed
        tx.status = TransactionStatus::Committed;
        system.transaction_registry().update_status(tx.id, tx.status.clone())?;
        
        // Return process result
        Ok(ProcessResult {
            transaction_id: tx.id,
            status: ProcessStatus::Completed,
            execution_result,
            commit_result,
        })
    }
}
```

## Asynchronous Transaction Processing

For long-running transactions:

```rust
pub struct AsyncTransactionProcessor {
    processor: Arc<TransactionProcessor>,
    status_tracker: Arc<TransactionStatusTracker>,
    worker_pool: Arc<WorkerPool>,
}

impl AsyncTransactionProcessor {
    /// Process a transaction asynchronously
    pub fn process_transaction_async(
        &self,
        transaction: Transaction,
        context: &ProcessContext,
    ) -> Result<TransactionReceipt, ProcessError> {
        // Create initial receipt
        let receipt = TransactionReceipt {
            transaction_id: transaction.id,
            submission_time: system.current_time(),
            status: TransactionStatus::Submitted,
        };
        
        // Update status
        system.transaction_registry().update_status(
            transaction.id,
            TransactionStatus::Submitted,
        )?;
        
        // Submit to worker pool
        self.worker_pool.submit(move || {
            // Process transaction
            let result = self.processor.process_transaction(transaction, &context);
            
            // Update status based on result
            match &result {
                Ok(process_result) => {
                    system.transaction_registry().update_status(
                        process_result.transaction_id,
                        TransactionStatus::Committed,
                    ).unwrap_or_else(|e| {
                        // Log error
                        system.logger().error(
                            &format!("Failed to update transaction status: {}", e),
                        );
                    });
                }
                Err(e) => {
                    system.transaction_registry().update_status(
                        transaction.id,
                        TransactionStatus::Failed(e.to_string()),
                    ).unwrap_or_else(|e| {
                        // Log error
                        system.logger().error(
                            &format!("Failed to update transaction status: {}", e),
                        );
                    });
                }
            }
        })?;
        
        Ok(receipt)
    }
}
```

## Cross-Domain Transactions

Transactions spanning multiple domains:

```rust
pub struct CrossDomainTransactionProcessor {
    local_processor: Arc<TransactionProcessor>,
    messenger: Arc<CrossDomainMessenger>,
}

impl CrossDomainTransactionProcessor {
    /// Process a transaction across multiple domains
    pub fn process_cross_domain_transaction(
        &self,
        transaction: CrossDomainTransaction,
        context: &ProcessContext,
    ) -> Result<CrossDomainProcessResult, ProcessError> {
        // Phase 1: Prepare
        let prepare_results = self.prepare_transaction(&transaction, context)?;
        
        // Check if all domains prepared successfully
        if !prepare_results.all_prepared() {
            // Abort on all domains
            self.abort_transaction(&transaction, &prepare_results)?;
            
            return Err(ProcessError::PreparationFailed(
                prepare_results.failed_domains()
                    .into_iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }
        
        // Phase 2: Commit
        let commit_results = self.commit_transaction(&transaction, &prepare_results)?;
        
        // Check if all domains committed
        if !commit_results.all_committed() {
            // Some domains failed to commit - this is a partial failure
            // Record the inconsistency for manual reconciliation
            system.inconsistency_tracker().record_inconsistency(
                InconsistencyType::PartialCommit,
                transaction.id,
                commit_results.domain_results.clone(),
            )?;
            
            return Err(ProcessError::PartialCommitFailure(
                commit_results.failed_domains()
                    .into_iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }
        
        // All domains committed successfully
        Ok(CrossDomainProcessResult {
            transaction_id: transaction.id,
            status: CrossDomainProcessStatus::Completed,
            domain_results: commit_results.domain_results,
            timestamp: system.current_time(),
        })
    }
}
```

## Transaction Recovery

Handling transaction failures and recovery:

```rust
pub struct TransactionRecoveryService {
    transaction_registry: Arc<TransactionRegistry>,
    transaction_processor: Arc<TransactionProcessor>,
}

impl TransactionRecoveryService {
    /// Recover incomplete transactions
    pub fn recover_incomplete_transactions(&self) -> Result<RecoveryResult, RecoveryError> {
        // Find transactions that need recovery
        let incomplete = self.transaction_registry.find_transactions(
            TransactionQuery::new()
                .with_status_in(vec![
                    TransactionStatus::Submitted,
                    TransactionStatus::Processing,
                    TransactionStatus::Validated,
                    TransactionStatus::Executing,
                ])
                .with_created_before(system.current_time() - Duration::from_hours(1)),
        )?;
        
        let mut recovered = Vec::new();
        let mut failed = Vec::new();
        
        // Process each incomplete transaction
        for tx in incomplete {
            match self.recover_transaction(&tx) {
                Ok(result) => recovered.push((tx.id, result)),
                Err(e) => failed.push((tx.id, e)),
            }
        }
        
        Ok(RecoveryResult {
            recovered,
            failed,
            timestamp: system.current_time(),
        })
    }
    
    /// Recover a specific transaction
    pub fn recover_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<TransactionStatus, RecoveryError> {
        // Check current status
        match &transaction.status {
            TransactionStatus::Created | TransactionStatus::Submitted => {
                // Transaction hasn't started processing yet, resubmit
                self.resubmit_transaction(transaction)
            }
            TransactionStatus::Processing | TransactionStatus::Validated | TransactionStatus::Executing => {
                // Transaction started but didn't complete, check for partial execution
                self.check_and_resolve_partial_execution(transaction)
            }
            TransactionStatus::Committed => {
                // Transaction already committed, nothing to do
                Ok(TransactionStatus::Committed)
            }
            TransactionStatus::Aborted => {
                // Transaction was aborted, nothing to do
                Ok(TransactionStatus::Aborted)
            }
            TransactionStatus::Failed(_) => {
                // Transaction failed, nothing to do
                Ok(transaction.status.clone())
            }
            TransactionStatus::Expired => {
                // Transaction expired, nothing to do
                Ok(TransactionStatus::Expired)
            }
        }
    }
}
```

## Transaction Monitoring

Monitoring transaction activities:

```rust
pub trait TransactionObserver: Send + Sync {
    /// Called when a transaction is created
    fn on_transaction_created(&self, transaction: &Transaction) -> Result<(), ObserverError>;
    
    /// Called when a transaction is submitted
    fn on_transaction_submitted(&self, transaction: &Transaction) -> Result<(), ObserverError>;
    
    /// Called when a transaction completes validation
    fn on_transaction_validated(&self, transaction: &Transaction, result: &ValidationResult) -> Result<(), ObserverError>;
    
    /// Called when a transaction is executed
    fn on_transaction_executed(&self, transaction: &Transaction, result: &TransactionExecutionResult) -> Result<(), ObserverError>;
    
    /// Called when a transaction is committed
    fn on_transaction_committed(&self, transaction: &Transaction, result: &CommitResult) -> Result<(), ObserverError>;
    
    /// Called when a transaction fails
    fn on_transaction_failed(&self, transaction: &Transaction, error: &ProcessError) -> Result<(), ObserverError>;
}
```

## Usage Examples

### Basic Transaction Execution

```rust
// Create a transaction
let mut transaction = create_transaction(
    AuthInfo::new()
        .with_identity(user_identity)
        .with_authentication_token(auth_token),
    Some(TransactionMetadata::new()
        .with_field("source", "document_service")
        .with_field("purpose", "document_update")),
    Some(Duration::from_minutes(30)),
);

// Add operations to the transaction
transaction.add_operation(Operation::new(
    OperationType::Update,
    document_id,
    OperationParameters::new()
        .with_parameter("title", "Updated Document Title")
        .with_parameter("status", "published"),
    AuthInfo::new().with_identity(user_identity),
))?;

transaction.add_operation(Operation::new(
    OperationType::Create,
    metadata_registry_id,
    OperationParameters::new()
        .with_parameter("type", "document_metadata")
        .with_parameter("document_id", document_id.to_string())
        .with_parameter("last_updated", system.current_time().to_string()),
    AuthInfo::new().with_identity(user_identity),
))?;

// Create processing context
let context = ProcessContext::new()
    .with_correlation_id(correlation_id)
    .with_source("document_service");

// Process the transaction
let result = transaction_processor.process_transaction(transaction, &context)?;

println!("Transaction completed with status: {:?}", result.status);
```

### Asynchronous Transaction

```rust
// Create a transaction with multiple operations
let mut transaction = create_transaction(
    AuthInfo::new().with_identity(admin_identity),
    None,
    Some(Duration::from_minutes(60)),
);

// Add multiple operations
for document_id in document_ids {
    transaction.add_operation(Operation::new(
        OperationType::Update,
        document_id,
        OperationParameters::new()
            .with_parameter("status", "archived"),
        AuthInfo::new().with_identity(admin_identity),
    ))?;
}

// Submit for async processing
let receipt = async_transaction_processor.process_transaction_async(
    transaction,
    &context,
)?;

println!("Transaction submitted with ID: {}. Current status: {:?}", 
    receipt.transaction_id, receipt.status);

// Later, check status
let status = system.transaction_registry().get_transaction_status(receipt.transaction_id)?;
println!("Current transaction status: {:?}", status);
```

### Cross-Domain Transaction

```rust
// Create a cross-domain transaction
let local_transaction = create_transaction(
    AuthInfo::new().with_identity(user_identity),
    None,
    None,
);

let remote_transaction = create_transaction(
    AuthInfo::new().with_identity(user_identity),
    None,
    None,
);

// Add operations to local transaction
local_transaction.add_operation(Operation::new(
    OperationType::Update,
    local_document_id,
    OperationParameters::new()
        .with_parameter("status", "published"),
    AuthInfo::new().with_identity(user_identity),
))?;

// Add operations to remote transaction
remote_transaction.add_operation(Operation::new(
    OperationType::Create,
    remote_registry_id,
    OperationParameters::new()
        .with_parameter("type", "reference")
        .with_parameter("source_document", local_document_id.to_string())
        .with_parameter("domain", local_domain_id.to_string()),
    AuthInfo::new().with_identity(user_identity),
))?;

// Create cross-domain transaction
let cross_domain_tx = CrossDomainTransaction {
    id: TransactionId::generate(),
    local_transaction,
    remote_transactions: HashMap::from([
        (remote_domain_id, remote_transaction),
    ]),
    timestamp: system.current_time(),
};

// Process the cross-domain transaction
let result = cross_domain_processor.process_cross_domain_transaction(
    cross_domain_tx,
    &context,
)?;

println!("Cross-domain transaction completed with results from {} domains", 
    result.domain_results.len());
```

## Implementation Status

The current implementation status of the Transaction Lifecycle:

- ✅ Core transaction model
- ✅ Basic lifecycle stages
- ✅ Transaction validation
- ✅ Transaction execution
- ⚠️ Cross-domain transactions (partially implemented)
- ⚠️ Transaction recovery (partially implemented)
- ❌ Advanced monitoring (not yet implemented)
- ❌ Distributed transaction coordination (not yet implemented)

## Future Enhancements

Planned future enhancements for Transaction Lifecycle:

1. **Distributed Transaction Coordination**: Enhanced protocols for cross-domain transactions
2. **Transaction Scheduling**: Support for scheduling transactions for future execution
3. **Transaction Templates**: Reusable templates for common transaction patterns
4. **Advanced Monitoring Dashboard**: Real-time visibility into transaction processing
5. **Transaction Analytics**: Insights into transaction patterns and performance
6. **Transaction Replay**: Support for replaying transactions for recovery or testing
7. **Enhanced Recovery Mechanisms**: More sophisticated recovery for failed transactions
``` 