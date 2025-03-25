<!-- Lifecycle of operations -->
<!-- Original file: docs/src/operation_lifecycle.md -->

# Operation Lifecycle in Causality

## Overview

This document describes the complete lifecycle of operations within the Causality architecture. Operations represent the primary mechanism for performing changes within the system, including modifying resources, managing capabilities, and interacting with the registry system. The operation lifecycle encompasses all stages from creation through validation, authorization, execution, effect generation, and finalization.

## Core Concepts

### Operation Model

Operations are represented by the following core model:

```rust
pub struct Operation {
    /// Unique identifier for this operation
    id: OperationId,
    
    /// Type of operation
    operation_type: OperationType,
    
    /// Target resource for this operation
    resource_id: ResourceId,
    
    /// Parameters for the operation
    parameters: OperationParameters,
    
    /// Authentication information
    auth_info: AuthInfo,
    
    /// Metadata associated with this operation
    metadata: OperationMetadata,
    
    /// Creation timestamp
    created_at: Timestamp,
}

pub enum OperationType {
    /// Create a new resource
    Create,
    
    /// Read a resource
    Read,
    
    /// Update a resource
    Update,
    
    /// Delete a resource
    Delete,
    
    /// Execute a method on a resource
    Execute {
        /// Method name to execute
        method: String,
    },
    
    /// Custom operation type
    Custom {
        /// Name of the custom operation
        name: String,
    },
}
```

## Lifecycle Stages

### 1. Operation Creation

Operations begin their lifecycle through creation:

```rust
/// Creates a new operation
pub fn create_operation(
    operation_type: OperationType,
    resource_id: ResourceId,
    parameters: OperationParameters,
    auth_info: AuthInfo,
) -> Operation {
    Operation {
        id: OperationId::generate(),
        operation_type,
        resource_id,
        parameters,
        auth_info,
        metadata: OperationMetadata::new(),
        created_at: system.current_time(),
    }
}
```

### 2. Submission

After creation, operations are submitted to the system:

```rust
/// Submits an operation for processing
pub fn submit_operation(
    operation: Operation,
) -> Result<OperationReceipt, OperationError> {
    // Log operation submission
    system.operation_logger().log_submission(&operation)?;
    
    // Place in operation queue
    system.operation_queue().enqueue(operation.clone())?;
    
    // Return receipt
    Ok(OperationReceipt {
        operation_id: operation.id,
        submission_time: system.current_time(),
        status: OperationStatus::Submitted,
    })
}
```

### 3. Validation

Operations undergo validation to ensure correctness:

```rust
/// Validates an operation
pub fn validate_operation(
    operation: &Operation,
    validation_context: &ValidationContext,
) -> Result<ValidationResult, ValidationError> {
    // Create validation pipeline
    let validation_pipeline = ValidationPipeline::new()
        .add_stage(Box::new(InputValidationStage::new()))
        .add_stage(Box::new(SemanticValidationStage::new()))
        .add_stage(Box::new(CapabilityValidationStage::new()))
        .add_stage(Box::new(TemporalValidationStage::new()))
        .add_stage(Box::new(ResourceValidationStage::new()))
        .add_stage(Box::new(RelationshipValidationStage::new()))
        .add_stage(Box::new(DomainValidationStage::new()));
    
    // Execute validation
    validation_pipeline.validate(operation, validation_context)
}
```

### 4. Authorization

Operations are authorized to ensure the requester has appropriate permissions:

```rust
/// Authorizes an operation
pub fn authorize_operation(
    operation: &Operation,
    auth_context: &AuthContext,
) -> Result<AuthorizationResult, AuthorizationError> {
    // Create authorization service
    let auth_service = system.authorization_service();
    
    // Check authorization
    auth_service.authorize_operation(operation, auth_context)
}
```

### 5. Execution

Authorized operations are executed by appropriate handlers:

```rust
/// Executes an operation
pub fn execute_operation(
    operation: &Operation,
    execution_context: &ExecutionContext,
) -> Result<ExecutionResult, ExecutionError> {
    // Get operation handler
    let handler = system.operation_registry()
        .get_handler(operation.operation_type, operation.resource_id.resource_type())?;
    
    // Execute operation
    handler.execute(operation, execution_context)
}
```

### 6. Effect Generation

Operations generate effects representing the changes they make:

```rust
/// Generates effects from an operation execution
pub fn generate_effects(
    operation: &Operation,
    execution_result: &ExecutionResult,
    context: &EffectContext,
) -> Result<Vec<Effect>, EffectError> {
    // Get effect generator
    let generator = system.effect_registry()
        .get_generator(operation.operation_type, operation.resource_id.resource_type())?;
    
    // Generate effects
    generator.generate_effects(operation, execution_result, context)
}
```

### 7. Commitment

Operation results and effects are committed to finalize the operation:

```rust
/// Commits an operation
pub fn commit_operation(
    operation: &Operation,
    execution_result: ExecutionResult,
    effects: Vec<Effect>,
    context: &CommitContext,
) -> Result<CommitResult, CommitError> {
    // Begin transaction
    let tx = system.storage().begin_transaction()?;
    
    // Store operation result
    system.operation_storage().store_result(operation.id, &execution_result, &tx)?;
    
    // Apply and store effects
    for effect in &effects {
        system.effect_applier().apply_effect(effect, &tx)?;
        system.effect_storage().store_effect(effect, &tx)?;
    }
    
    // Commit transaction
    system.storage().commit_transaction(tx)?;
    
    // Update operation status
    system.operation_registry().update_status(operation.id, OperationStatus::Completed)?;
    
    Ok(CommitResult {
        operation_id: operation.id,
        status: CommitStatus::Committed,
        effect_count: effects.len(),
        timestamp: system.current_time(),
    })
}
```

### 8. Observation

Operations and their effects can be observed after completion:

```rust
/// Observes an operation's effects
pub fn observe_operation_effects(
    operation_id: OperationId,
) -> Result<Vec<Effect>, ObservationError> {
    // Get effects for operation
    system.effect_storage().get_effects_for_operation(operation_id)
}
```

## Operation Processor

The OperationProcessor orchestrates the complete lifecycle:

```rust
pub struct OperationProcessor {
    validation_service: Arc<ValidationService>,
    authorization_service: Arc<AuthorizationService>,
    execution_service: Arc<ExecutionService>,
    effect_service: Arc<EffectService>,
    commit_service: Arc<CommitService>,
}

impl OperationProcessor {
    /// Process an operation through its complete lifecycle
    pub fn process_operation(
        &self,
        operation: Operation,
        context: &ProcessContext,
    ) -> Result<ProcessResult, ProcessError> {
        // Create operation context
        let operation_context = OperationContext::from(context);
        
        // 1. Validate the operation
        let validation_result = self.validation_service.validate_operation(
            &operation,
            &operation_context.into(),
        )?;
        
        if !validation_result.is_valid() {
            return Err(ProcessError::ValidationFailed(validation_result));
        }
        
        // 2. Authorize the operation
        let auth_result = self.authorization_service.authorize_operation(
            &operation,
            &operation_context.into(),
        )?;
        
        if !auth_result.is_authorized() {
            return Err(ProcessError::AuthorizationFailed(auth_result));
        }
        
        // 3. Execute the operation
        let execution_result = self.execution_service.execute_operation(
            &operation,
            &operation_context.into(),
        )?;
        
        // 4. Generate effects
        let effects = self.effect_service.generate_effects(
            &operation,
            &execution_result,
            &operation_context.into(),
        )?;
        
        // 5. Commit the operation and effects
        let commit_result = self.commit_service.commit_operation(
            &operation,
            execution_result.clone(),
            effects.clone(),
            &operation_context.into(),
        )?;
        
        // Return process result
        Ok(ProcessResult {
            operation_id: operation.id,
            status: ProcessStatus::Completed,
            execution_result,
            effects,
            commit_result,
        })
    }
}
```

## Asynchronous Operation Processing

For long-running operations:

```rust
pub struct AsyncOperationProcessor {
    processor: Arc<OperationProcessor>,
    status_tracker: Arc<OperationStatusTracker>,
    worker_pool: Arc<WorkerPool>,
}

impl AsyncOperationProcessor {
    /// Process an operation asynchronously
    pub fn process_operation_async(
        &self,
        operation: Operation,
        context: &ProcessContext,
    ) -> Result<OperationReceipt, ProcessError> {
        // Create initial receipt
        let receipt = OperationReceipt {
            operation_id: operation.id,
            submission_time: system.current_time(),
            status: OperationStatus::Processing,
        };
        
        // Update status
        self.status_tracker.update_status(
            operation.id,
            OperationStatus::Processing,
        )?;
        
        // Submit to worker pool
        self.worker_pool.submit(move || {
            // Process operation
            let result = self.processor.process_operation(operation, &context);
            
            // Update status based on result
            match &result {
                Ok(process_result) => {
                    self.status_tracker.update_status(
                        process_result.operation_id,
                        OperationStatus::Completed,
                    ).unwrap_or_else(|e| {
                        // Log error
                        system.logger().error(
                            &format!("Failed to update operation status: {}", e),
                        );
                    });
                }
                Err(e) => {
                    self.status_tracker.update_status(
                        operation.id,
                        OperationStatus::Failed(e.to_string()),
                    ).unwrap_or_else(|e| {
                        // Log error
                        system.logger().error(
                            &format!("Failed to update operation status: {}", e),
                        );
                    });
                }
            }
        })?;
        
        Ok(receipt)
    }
}
```

## Error Handling

Operations can encounter various errors during their lifecycle:

```rust
pub enum OperationError {
    /// Validation error
    ValidationFailed(ValidationResult),
    
    /// Authorization error
    AuthorizationFailed(AuthorizationResult),
    
    /// Execution error
    ExecutionFailed(String),
    
    /// Effect error
    EffectGenerationFailed(String),
    
    /// Commit error
    CommitFailed(String),
    
    /// System error
    SystemError(String),
}

impl OperationProcessor {
    /// Handle operation error
    pub fn handle_operation_error(
        &self,
        operation: &Operation,
        error: &OperationError,
        context: &ProcessContext,
    ) -> Result<(), ErrorHandlingError> {
        // Log the error
        system.logger().error(&format!(
            "Operation error: {} - {}",
            operation.id,
            error,
        ));
        
        // Update operation status
        system.operation_registry().update_status(
            operation.id,
            OperationStatus::Failed(error.to_string()),
        )?;
        
        // Notify error observers
        for observer in system.error_observers() {
            observer.on_operation_error(operation, error, context)?;
        }
        
        Ok(())
    }
}
```

## Monitoring and Observation

Operation lifecycle events can be monitored:

```rust
pub trait OperationObserver: Send + Sync {
    /// Called when an operation is submitted
    fn on_operation_submitted(&self, operation: &Operation) -> Result<(), ObserverError>;
    
    /// Called when an operation completes validation
    fn on_operation_validated(&self, operation: &Operation, result: &ValidationResult) -> Result<(), ObserverError>;
    
    /// Called when an operation is authorized
    fn on_operation_authorized(&self, operation: &Operation, result: &AuthorizationResult) -> Result<(), ObserverError>;
    
    /// Called when an operation is executed
    fn on_operation_executed(&self, operation: &Operation, result: &ExecutionResult) -> Result<(), ObserverError>;
    
    /// Called when effects are generated
    fn on_effects_generated(&self, operation: &Operation, effects: &[Effect]) -> Result<(), ObserverError>;
    
    /// Called when an operation is committed
    fn on_operation_committed(&self, operation: &Operation, result: &CommitResult) -> Result<(), ObserverError>;
    
    /// Called when an operation fails
    fn on_operation_failed(&self, operation: &Operation, error: &OperationError) -> Result<(), ObserverError>;
}
```

## Usage Examples

### Basic Operation Execution

```rust
// Create an operation to update a document resource
let operation = create_operation(
    OperationType::Update,
    document_id,
    OperationParameters::new()
        .with_parameter("title", "Updated Document Title")
        .with_parameter("status", "published"),
    AuthInfo::new()
        .with_identity(user_identity)
        .with_authentication_token(auth_token),
);

// Create processing context
let context = ProcessContext::new()
    .with_correlation_id(correlation_id)
    .with_source("document_service");

// Process the operation
let result = operation_processor.process_operation(operation, &context)?;

println!("Operation completed with status: {:?}", result.status);
```

### Asynchronous Operation

```rust
// Create a long-running operation
let operation = create_operation(
    OperationType::Execute { method: "generate_report".to_string() },
    report_generator_id,
    OperationParameters::new()
        .with_parameter("report_type", "monthly_summary")
        .with_parameter("time_range", TimeRange::from_last_month()),
    AuthInfo::new()
        .with_identity(admin_identity)
        .with_authentication_token(admin_token),
);

// Submit for async processing
let receipt = async_operation_processor.process_operation_async(
    operation,
    &context,
)?;

println!("Operation submitted with ID: {}. Current status: {:?}", 
    receipt.operation_id, receipt.status);

// Later, check status
let status = operation_status_tracker.get_status(receipt.operation_id)?;
println!("Current operation status: {:?}", status);
```

## Implementation Status

The current implementation status of the Operation Lifecycle:

- ✅ Core operation model
- ✅ Basic lifecycle stages
- ✅ Validation pipeline
- ✅ Authorization framework
- ✅ Execution framework
- ⚠️ Effect generation (partially implemented)
- ⚠️ Asynchronous processing (partially implemented)
- ❌ Advanced monitoring (not yet implemented)
- ❌ Distributed operation processing (not yet implemented)

## Future Enhancements

Planned future enhancements for Operation Lifecycle:

1. **Distributed Operation Processing**: Support for processing operations across multiple nodes
2. **Operation Prioritization**: Advanced queueing and prioritization mechanisms
3. **Operation Replay**: Ability to replay operations for recovery or testing
4. **Operation Composition**: Compose multiple operations into higher-level operations
5. **Operation Templates**: Reusable templates for common operation patterns
6. **Enhanced Monitoring**: Richer monitoring and analytics for operations
7. **Cross-Domain Operation Coordination**: Improved coordination of operations across domains 