<!-- Transactions in the registry -->
<!-- Original file: docs/src/registry_transactions.md -->

# Registry Transactions in Causality

## Overview

This document describes the transaction system for registry operations in the Causality architecture. Registry transactions enable atomic, consistent, isolated, and durable (ACID) operations across multiple registry entities. Transactions ensure system integrity even during complex operations that touch multiple registries or entities.

## Core Concepts

### Transaction Model

The fundamental transaction model is built on these core abstractions:

```rust
pub struct RegistryTransaction {
    /// Transaction ID
    id: TransactionId,
    
    /// Operations to perform
    operations: Vec<RegistryOperation>,
    
    /// Transaction status
    status: TransactionStatus,
    
    /// Timestamp of creation
    created_at: Timestamp,
    
    /// Transaction metadata
    metadata: TransactionMetadata,
}

pub enum TransactionStatus {
    /// Transaction has been created but not executed
    Created,
    
    /// Transaction is currently executing
    Executing,
    
    /// Transaction completed successfully
    Committed,
    
    /// Transaction was aborted
    Aborted,
    
    /// Transaction failed
    Failed(String),
}

pub enum RegistryOperation {
    /// Register a new resource
    RegisterResource {
        /// Resource to register
        resource: Resource,
    },
    
    /// Update an existing resource
    UpdateResource {
        /// Resource ID
        resource_id: ResourceId,
        /// Updates to apply
        updates: ResourceUpdates,
    },
    
    /// Deregister a resource
    DeregisterResource {
        /// Resource ID
        resource_id: ResourceId,
    },
    
    /// Register a capability
    RegisterCapability {
        /// Capability to register
        capability: Capability,
    },
    
    /// Register a relationship
    RegisterRelationship {
        /// Relationship to register
        relationship: Relationship,
    },
    
    /// Register a fact
    RegisterFact {
        /// Fact to register
        fact: TemporalFact,
    },
    
    /// Execute a custom operation
    Custom {
        /// Operation type
        operation_type: String,
        /// Operation data
        data: Vec<u8>,
    },
}
```

## Transaction Execution

### Transaction Manager

The Transaction Manager orchestrates transaction execution:

```rust
pub struct TransactionManager {
    /// Registry hub for accessing registries
    registry_hub: Arc<RegistryHub>,
    
    /// Transaction storage
    storage: Arc<dyn TransactionStorage>,
    
    /// Transaction validators
    validators: Vec<Box<dyn TransactionValidator>>,
    
    /// Transaction processors
    processors: HashMap<RegistryOperationType, Box<dyn OperationProcessor>>,
    
    /// Transaction observers
    observers: Vec<Box<dyn TransactionObserver>>,
}

impl TransactionManager {
    /// Create a new transaction
    pub fn create_transaction(&self) -> RegistryTransaction {
        RegistryTransaction {
            id: TransactionId::generate(),
            operations: Vec::new(),
            status: TransactionStatus::Created,
            created_at: system.current_time(),
            metadata: TransactionMetadata::new(),
        }
    }
    
    /// Execute a transaction
    pub fn execute_transaction(
        &self,
        transaction: RegistryTransaction,
        context: &TransactionContext,
    ) -> Result<TransactionResult, TransactionError> {
        // Record transaction start
        let mut tx = transaction.clone();
        tx.status = TransactionStatus::Executing;
        self.storage.store_transaction(&tx)?;
        
        // Validate transaction
        self.validate_transaction(&tx, context)?;
        
        // Begin database transaction if storage supports it
        let db_transaction = self.storage.begin_transaction()?;
        
        // Execute each operation
        let mut results = Vec::new();
        let mut failed_operation: Option<(usize, RegistryOperation, String)> = None;
        
        for (index, operation) in tx.operations.iter().enumerate() {
            // Get processor for this operation type
            let processor = match self.get_processor_for_operation(operation) {
                Ok(p) => p,
                Err(e) => {
                    failed_operation = Some((index, operation.clone(), e.to_string()));
                    break;
                }
            };
            
            // Process the operation
            match processor.process_operation(operation, context, &db_transaction) {
                Ok(result) => {
                    results.push(OperationResult {
                        operation: operation.clone(),
                        status: OperationStatus::Success,
                        result_data: result,
                    });
                }
                Err(e) => {
                    failed_operation = Some((index, operation.clone(), e.to_string()));
                    break;
                }
            }
        }
        
        // If any operation failed, rollback
        if let Some((index, operation, error)) = failed_operation {
            // Rollback database transaction
            self.storage.rollback_transaction(db_transaction)?;
            
            // Update transaction status
            tx.status = TransactionStatus::Failed(error.clone());
            self.storage.store_transaction(&tx)?;
            
            // Notify observers of failure
            for observer in &self.observers {
                observer.on_transaction_failed(&tx, index, &operation, &error)?;
            }
            
            return Err(TransactionError::OperationFailed {
                operation_index: index,
                operation: operation,
                error,
            });
        }
        
        // All operations succeeded, commit the transaction
        self.storage.commit_transaction(db_transaction)?;
        
        // Update transaction status
        tx.status = TransactionStatus::Committed;
        self.storage.store_transaction(&tx)?;
        
        // Notify observers of success
        for observer in &self.observers {
            observer.on_transaction_committed(&tx, &results)?;
        }
        
        // Return results
        Ok(TransactionResult {
            transaction_id: tx.id,
            status: TransactionStatus::Committed,
            operation_results: results,
            timestamp: system.current_time(),
        })
    }
    
    /// Validate a transaction before execution
    fn validate_transaction(
        &self,
        transaction: &RegistryTransaction,
        context: &TransactionContext,
    ) -> Result<(), TransactionError> {
        // Run all validators
        for validator in &self.validators {
            if let Err(e) = validator.validate_transaction(transaction, context) {
                return Err(TransactionError::ValidationFailed(e.to_string()));
            }
        }
        
        Ok(())
    }
}
```

## Transaction Components

### Operation Processors

Process individual operations within a transaction:

```rust
pub trait OperationProcessor: Send + Sync {
    /// Process an operation within a transaction
    fn process_operation(
        &self,
        operation: &RegistryOperation,
        context: &TransactionContext,
        db_transaction: &Transaction,
    ) -> Result<Value, OperationError>;
    
    /// Get the operation types this processor handles
    fn handles_operation_type(&self) -> RegistryOperationType;
}

/// Processor for resource operations
pub struct ResourceOperationProcessor {
    resource_registry: Arc<ResourceRegistry>,
}

impl OperationProcessor for ResourceOperationProcessor {
    fn process_operation(
        &self,
        operation: &RegistryOperation,
        context: &TransactionContext,
        db_transaction: &Transaction,
    ) -> Result<Value, OperationError> {
        match operation {
            RegistryOperation::RegisterResource { resource } => {
                // Register the resource within the transaction
                let resource_id = self.resource_registry.register_resource_in_transaction(
                    resource.clone(),
                    db_transaction,
                )?;
                
                Ok(Value::String(resource_id.to_string()))
            }
            RegistryOperation::UpdateResource { resource_id, updates } => {
                // Update the resource within the transaction
                let updated = self.resource_registry.update_resource_in_transaction(
                    *resource_id,
                    updates.clone(),
                    db_transaction,
                )?;
                
                Ok(Value::Object(serde_json::to_value(updated)?))
            }
            RegistryOperation::DeregisterResource { resource_id } => {
                // Deregister the resource within the transaction
                self.resource_registry.deregister_resource_in_transaction(
                    *resource_id,
                    db_transaction,
                )?;
                
                Ok(Value::Null)
            }
            _ => Err(OperationError::UnsupportedOperation(
                "ResourceOperationProcessor does not support this operation".to_string(),
            )),
        }
    }
    
    fn handles_operation_type(&self) -> RegistryOperationType {
        RegistryOperationType::Resource
    }
}
```

### Transaction Validation

Validates transactions before execution:

```rust
pub trait TransactionValidator: Send + Sync {
    /// Validate a transaction before execution
    fn validate_transaction(
        &self,
        transaction: &RegistryTransaction,
        context: &TransactionContext,
    ) -> Result<(), ValidationError>;
    
    /// Get validator name
    fn name(&self) -> &str;
}

/// Validates transaction permissions
pub struct PermissionValidator {
    capability_registry: Arc<CapabilityRegistry>,
}

impl TransactionValidator for PermissionValidator {
    fn validate_transaction(
        &self,
        transaction: &RegistryTransaction,
        context: &TransactionContext,
    ) -> Result<(), ValidationError> {
        // Check if user has permission for all operations in the transaction
        for operation in &transaction.operations {
            match operation {
                RegistryOperation::RegisterResource { resource } => {
                    // Check if user can register this resource type
                    let can_register = self.capability_registry.check_capability_for_resource_type(
                        context.identity(),
                        resource.resource_type(),
                        PermissionType::Register,
                    )?;
                    
                    if !can_register {
                        return Err(ValidationError::PermissionDenied(format!(
                            "User does not have permission to register resources of type {}",
                            resource.resource_type()
                        )));
                    }
                }
                // Similar checks for other operation types...
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "PermissionValidator"
    }
}
```

### Transaction Storage

Persists transaction history:

```rust
pub trait TransactionStorage: Send + Sync {
    /// Store a transaction
    fn store_transaction(&self, transaction: &RegistryTransaction) -> Result<(), StorageError>;
    
    /// Get a transaction by ID
    fn get_transaction(&self, id: &TransactionId) -> Result<RegistryTransaction, StorageError>;
    
    /// List transactions matching criteria
    fn list_transactions(
        &self,
        criteria: &TransactionCriteria,
        pagination: Option<Pagination>,
    ) -> Result<Vec<RegistryTransaction>, StorageError>;
    
    /// Begin a database transaction
    fn begin_transaction(&self) -> Result<Transaction, StorageError>;
    
    /// Commit a database transaction
    fn commit_transaction(&self, transaction: Transaction) -> Result<(), StorageError>;
    
    /// Rollback a database transaction
    fn rollback_transaction(&self, transaction: Transaction) -> Result<(), StorageError>;
}
```

## Advanced Transaction Patterns

### Cross-Domain Transactions

Transactions that span multiple domains:

```rust
pub struct CrossDomainTransactionCoordinator {
    /// Local transaction manager
    local_manager: Arc<TransactionManager>,
    
    /// Cross-domain messenger
    messenger: Arc<CrossDomainMessenger>,
}

impl CrossDomainTransactionCoordinator {
    /// Execute a transaction across multiple domains
    pub fn execute_cross_domain_transaction(
        &self,
        transaction: CrossDomainTransaction,
        context: &TransactionContext,
    ) -> Result<CrossDomainTransactionResult, TransactionError> {
        // Phase 1: Prepare
        let prepare_results = self.prepare_transaction(&transaction, context)?;
        
        // Check if all domains are ready
        if !prepare_results.all_prepared() {
            // Abort on all domains
            self.abort_transaction(&transaction, &prepare_results)?;
            
            return Err(TransactionError::PreparationFailed(
                prepare_results.failed_domains()
            ));
        }
        
        // Phase 2: Commit
        let commit_results = self.commit_transaction(&transaction, &prepare_results)?;
        
        // Check if all domains committed
        if !commit_results.all_committed() {
            // Some domains failed to commit, this is a partial failure
            // In a real system, this would require manual intervention
            return Err(TransactionError::PartialCommitFailure(
                commit_results.failed_domains()
            ));
        }
        
        // All domains committed successfully
        Ok(CrossDomainTransactionResult {
            transaction_id: transaction.id,
            status: TransactionStatus::Committed,
            domain_results: commit_results.domain_results,
            timestamp: system.current_time(),
        })
    }
    
    /// Phase 1: Prepare transaction on all domains
    fn prepare_transaction(
        &self,
        transaction: &CrossDomainTransaction,
        context: &TransactionContext,
    ) -> Result<PrepareResults, TransactionError> {
        let mut domain_results = HashMap::new();
        
        // Prepare on local domain first
        let local_domain = system.domain_id();
        let local_prepare = self.local_manager.prepare_transaction(
            transaction.local_transaction.clone(),
            context,
        )?;
        
        domain_results.insert(local_domain, local_prepare);
        
        // Prepare on each remote domain
        for (domain_id, remote_tx) in &transaction.remote_transactions {
            // Send prepare message
            let prepare_message = CrossDomainMessage::TransactionPrepare {
                transaction_id: transaction.id,
                transaction_data: remote_tx.clone(),
                coordinator_domain: local_domain,
                timestamp: system.current_time(),
            };
            
            // Wait for response
            let response = self.messenger.send_and_wait_response(
                *domain_id,
                prepare_message,
                Duration::from_secs(30),
            )?;
            
            // Process response
            match response {
                CrossDomainMessage::TransactionPrepareResponse { 
                    transaction_id, 
                    status, 
                    ..
                } => {
                    if transaction_id != transaction.id {
                        return Err(TransactionError::InvalidResponse(
                            "Transaction ID mismatch".to_string()
                        ));
                    }
                    
                    domain_results.insert(*domain_id, status);
                }
                _ => {
                    return Err(TransactionError::InvalidResponse(
                        "Unexpected response message".to_string()
                    ));
                }
            }
        }
        
        Ok(PrepareResults {
            domain_results,
        })
    }
    
    // Additional methods for commit and abort phases...
}
```

### Compensating Transactions

Transactions that can be automatically rolled back:

```rust
pub struct CompensatingTransaction {
    /// Original transaction
    original_transaction: RegistryTransaction,
    
    /// Compensation operations to perform if original fails
    compensation_operations: Vec<CompensationOperation>,
}

impl TransactionManager {
    /// Execute a transaction with compensation
    pub fn execute_with_compensation(
        &self,
        transaction: CompensatingTransaction,
        context: &TransactionContext,
    ) -> Result<TransactionResult, TransactionError> {
        // Try to execute the original transaction
        match self.execute_transaction(transaction.original_transaction.clone(), context) {
            Ok(result) => Ok(result),
            Err(error) => {
                // Transaction failed, execute compensation
                self.execute_compensation(&transaction, &error, context)?;
                Err(error)
            }
        }
    }
    
    /// Execute compensation operations
    fn execute_compensation(
        &self,
        transaction: &CompensatingTransaction,
        original_error: &TransactionError,
        context: &TransactionContext,
    ) -> Result<(), CompensationError> {
        // Create compensation transaction
        let mut comp_tx = self.create_transaction();
        
        // Add all compensation operations
        for op in &transaction.compensation_operations {
            comp_tx.operations.push(op.to_registry_operation()?);
        }
        
        // Execute compensation transaction
        match self.execute_transaction(comp_tx, context) {
            Ok(_) => Ok(()),
            Err(e) => Err(CompensationError::CompensationFailed {
                original_error: original_error.to_string(),
                compensation_error: e.to_string(),
            }),
        }
    }
}
```

## Usage Examples

### Simple Transaction

```rust
// Create a transaction
let mut transaction = transaction_manager.create_transaction();

// Add operations to the transaction
transaction.operations.push(RegistryOperation::RegisterResource {
    resource: document_resource.clone(),
});

transaction.operations.push(RegistryOperation::RegisterResource {
    resource: metadata_resource.clone(),
});

transaction.operations.push(RegistryOperation::RegisterRelationship {
    relationship: Relationship::new(
        RelationshipType::new("describes"),
        ResourceEndpoint::new(metadata_resource.id(), "metadata"),
        ResourceEndpoint::new(document_resource.id(), "document"),
        RelationshipAttributes::default(),
    ),
});

// Set transaction metadata
transaction.metadata.insert("purpose", "Document creation with metadata");
transaction.metadata.insert("source", "document_service");

// Create transaction context
let context = TransactionContext::new()
    .with_identity(user_identity)
    .with_auth_context(auth_context);

// Execute the transaction
let result = transaction_manager.execute_transaction(transaction, &context)?;

println!("Transaction completed successfully with {} operations", result.operation_results.len());
```

### Cross-Domain Transaction

```rust
// Create local transaction
let local_transaction = transaction_manager.create_transaction();
local_transaction.operations.push(RegistryOperation::RegisterResource {
    resource: local_document.clone(),
});

// Create remote transaction
let remote_transaction = RegistryTransaction {
    id: TransactionId::generate(),
    operations: vec![
        RegistryOperation::RegisterResource {
            resource: remote_reference.clone(),
        },
    ],
    status: TransactionStatus::Created,
    created_at: system.current_time(),
    metadata: TransactionMetadata::new(),
};

// Create cross-domain transaction
let cross_domain_tx = CrossDomainTransaction {
    id: TransactionId::generate(),
    local_transaction,
    remote_transactions: HashMap::from([
        (remote_domain_id, remote_transaction),
    ]),
    timestamp: system.current_time(),
};

// Execute the cross-domain transaction
let result = cross_domain_coordinator.execute_cross_domain_transaction(
    cross_domain_tx,
    &context,
)?;

println!("Cross-domain transaction completed successfully with results from {} domains", 
    result.domain_results.len());
```

### Compensating Transaction

```rust
// Create original transaction
let original_tx = transaction_manager.create_transaction();
original_tx.operations.push(RegistryOperation::RegisterResource {
    resource: document_resource.clone(),
});

// Create compensation operations
let compensation_ops = vec![
    CompensationOperation::DeregisterResource {
        resource_id: document_resource.id(),
    },
];

// Create compensating transaction
let compensating_tx = CompensatingTransaction {
    original_transaction: original_tx,
    compensation_operations: compensation_ops,
};

// Execute with automatic compensation if needed
match transaction_manager.execute_with_compensation(compensating_tx, &context) {
    Ok(result) => {
        println!("Transaction completed successfully");
    }
    Err(error) => {
        println!("Transaction failed and was compensated: {}", error);
    }
}
```

## Implementation Status

The current implementation status of Registry Transactions:

- ✅ Core transaction model
- ✅ Basic transaction execution
- ✅ Resource transaction operations
- ⚠️ Transaction validation (partially implemented)
- ⚠️ Transaction storage (partially implemented)
- ⚠️ Cross-domain transactions (early implementation)
- ❌ Compensating transactions (not yet implemented)
- ❌ Transaction monitoring and analytics (not yet implemented)

## Future Enhancements

Planned future enhancements for Registry Transactions:

1. **Long-Running Transactions**: Support for transactions that execute over extended periods
2. **Transaction Scheduling**: Ability to schedule transactions for future execution
3. **Transaction Templates**: Reusable templates for common transaction patterns
4. **Transaction Monitoring**: Advanced monitoring and diagnostic tools
5. **Transaction Analytics**: Insights into transaction patterns and performance
6. **Enhanced Recovery**: More sophisticated recovery mechanisms for failed transactions
7. **Distributed Transaction Log**: Replicated transaction logs for fault tolerance 