// Operation Execution Module
//
// This module provides utilities for executing operations in different contexts
// and handling the execution pipeline for the unified operation model.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, warn};

use crate::error::{Error, Result};
use crate::effect::{Effect, EffectOutcome};
use crate::resource::ResourceRegisterTrait;
use crate::crypto::hash::ContentId;
use crate::verification::{VerificationService, VerificationContext, VerificationOptions};

use super::{
    Operation, OperationType, ExecutionContext, ExecutionPhase, ExecutionEnvironment,
    AbstractContext, RegisterContext, PhysicalContext, ZkContext,
    ResourceRef, RegisterOperation, PhysicalOperation, ResourceRefType
};

use super::transformation::transform_operation;

/// Error when executing operations
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Cannot execute operation in context {0:?}")]
    InvalidContext(ExecutionEnvironment),

    #[error("Missing concrete implementation")]
    MissingImplementation,

    #[error("Missing required proof")]
    MissingProof,

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Effect execution failed: {0}")]
    EffectExecutionFailed(String),

    #[error("Register operation failed: {0}")]
    RegisterOperationFailed(String),

    #[error("Physical operation failed: {0}")]
    PhysicalOperationFailed(String),

    #[error("Transformation error: {0}")]
    TransformationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result of an operation execution
#[derive(Debug, Clone)]
pub struct OperationResult<C: ExecutionContext> {
    /// The operation that was executed
    pub operation: Operation<C>,
    
    /// The outcome of the effect execution
    pub effect_outcome: Option<EffectOutcome>,
    
    /// Whether the execution was successful
    pub success: bool,
    
    /// Error message (if execution failed)
    pub error: Option<String>,
    
    /// Additional result data
    pub result_data: HashMap<String, String>,
}

/// Trait for executing operations in a specific context
#[async_trait]
pub trait OperationExecutor<C: ExecutionContext>: Send + Sync {
    /// Execute an operation in the given context
    async fn execute(&self, operation: &Operation<C>) -> std::result::Result<OperationResult<C>, ExecutionError>;
    
    /// Transform and execute an operation in a different context
    async fn transform_and_execute<D: ExecutionContext>(
        &self,
        operation: &Operation<C>,
        target_executor: &dyn OperationExecutor<D>
    ) -> std::result::Result<OperationResult<D>, ExecutionError> {
        let transformed = transform_operation::<C, D>(operation)
            .map_err(|e| ExecutionError::TransformationError(e.to_string()))?;
        
        target_executor.execute(&transformed).await
    }
}

/// Executor for abstract operations
pub struct AbstractExecutor {
    interpreter: Arc<Interpreter>,
    verification_service: Arc<VerificationService>,
}

impl AbstractExecutor {
    /// Create a new abstract executor
    pub fn new(interpreter: Arc<Interpreter>, verification_service: Arc<VerificationService>) -> Self {
        Self {
            interpreter,
            verification_service,
        }
    }
}

#[async_trait]
impl OperationExecutor<AbstractContext> for AbstractExecutor {
    async fn execute(&self, operation: &Operation<AbstractContext>) -> std::result::Result<OperationResult<AbstractContext>, ExecutionError> {
        // For abstract operations, we just execute the effect
        let effect = &operation.abstract_representation;
        
        // Execute the effect
        let effect_outcome = self.interpreter.execute_effect(&**effect)
            .await
            .map_err(|e| ExecutionError::EffectExecutionFailed(e.to_string()))?;
        
        // Create the result
        let result = OperationResult {
            operation: operation.clone(),
            effect_outcome: Some(effect_outcome.clone()),
            success: effect_outcome.success,
            error: effect_outcome.error.clone(),
            result_data: effect_outcome.data.iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
        };
        
        Ok(result)
    }
}

/// Executor for register operations
pub struct RegisterExecutor {
    resource_register: Arc<dyn ResourceRegisterTrait>,
    verification_service: Arc<VerificationService>,
}

impl RegisterExecutor {
    /// Create a new register executor
    pub fn new(
        resource_register: Arc<dyn ResourceRegisterTrait>,
        verification_service: Arc<VerificationService>
    ) -> Self {
        Self {
            resource_register,
            verification_service,
        }
    }
    
    /// Execute a register operation
    async fn execute_register_operation(
        &self,
        register_op: &RegisterOperation
    ) -> Result<HashMap<String, String>, Error> {
        // Execute the register operation
        let result = match &register_op.operation {
            super::RegisterOperationType::Create => {
                self.resource_register.create_register(&register_op.register_id, &register_op.data)
                    .await
                    .map(|_| HashMap::new())
            },
            super::RegisterOperationType::Update => {
                self.resource_register.update_register(&register_op.register_id, &register_op.data)
                    .await
                    .map(|_| HashMap::new())
            },
            super::RegisterOperationType::Transfer => {
                // For transfers, we need to extract the new owner
                let new_owner = register_op.data.get("new_owner")
                    .ok_or_else(|| Error::InvalidArgument("Missing 'new_owner' for transfer operation".to_string()))?;
                
                self.resource_register.transfer_register(&register_op.register_id, new_owner)
                    .await
                    .map(|_| HashMap::new())
            },
            super::RegisterOperationType::Lock => {
                self.resource_register.lock_register(&register_op.register_id)
                    .await
                    .map(|_| HashMap::new())
            },
            super::RegisterOperationType::Unlock => {
                self.resource_register.unlock_register(&register_op.register_id)
                    .await
                    .map(|_| HashMap::new())
            },
            super::RegisterOperationType::Freeze => {
                self.resource_register.freeze_register(&register_op.register_id)
                    .await
                    .map(|_| HashMap::new())
            },
            super::RegisterOperationType::Archive => {
                self.resource_register.archive_register(&register_op.register_id)
                    .await
                    .map(|_| HashMap::new())
            },
            super::RegisterOperationType::Custom(ref op_name) => {
                // For custom operations, we pass all data to the register
                self.resource_register.custom_operation(&register_op.register_id, op_name, &register_op.data)
                    .await
            },
        };
        
        result
    }

    /// Execute abstract effect if available
    async fn execute_abstract_effect(&self, effect: &Option<Box<dyn Effect>>) -> Result<EffectOutcome, Error> {
        if let Some(effect) = effect {
            // Execute the effect (implementation would depend on your effect system)
            // This is a placeholder for the actual implementation
            Ok(EffectOutcome {
                success: true,
                error: None,
                data: HashMap::new(),
            })
        } else {
            Err(Error::InvalidArgument("No abstract effect to execute".to_string()))
        }
    }
}

#[async_trait]
impl OperationExecutor<RegisterContext> for RegisterExecutor {
    async fn execute(&self, operation: &Operation<RegisterContext>) -> std::result::Result<OperationResult<RegisterContext>, ExecutionError> {
        // For register operations, we need to verify the operation first
        let verification_context = VerificationContext::new()
            .with_operation_id(operation.id.to_string())
            .with_operation_type(format!("{:?}", operation.op_type))
            .with_resources(operation.inputs.iter().map(|r| r.resource_id.clone()).collect());
        
        let verification_options = VerificationOptions::default();
        
        // Verify the operation
        let verification_result = self.verification_service.verify(verification_context, verification_options)
            .await
            .map_err(|e| ExecutionError::VerificationFailed(e.to_string()))?;
        
        if !verification_result.is_valid() {
            return Err(ExecutionError::VerificationFailed(
                format!("Operation verification failed: {:?}", verification_result.reasons())
            ));
        }
        
        // Get the concrete implementation or error
        let register_op = operation.concrete_implementation.as_ref()
            .ok_or(ExecutionError::MissingImplementation)?;
        
        // Execute the register operation
        let result_data = self.execute_register_operation(register_op)
            .await
            .map_err(|e| ExecutionError::RegisterOperationFailed(e.to_string()))?;
        
        // Execute the abstract effect if available
        let effect_outcome = self.execute_abstract_effect(&operation.abstract_representation).await?;
        
        // Create the result
        let result = OperationResult {
            operation: operation.clone(),
            effect_outcome: Some(effect_outcome),
            success: true,
            error: None,
            result_data,
        };
        
        Ok(result)
    }
}

/// Executor for ZK operations
pub struct ZkExecutor {
    verification_service: Arc<VerificationService>,
}

impl ZkExecutor {
    /// Create a new ZK executor
    pub fn new(verification_service: Arc<VerificationService>) -> Self {
        Self {
            verification_service,
        }
    }
}

#[async_trait]
impl OperationExecutor<ZkContext> for ZkExecutor {
    async fn execute(&self, operation: &Operation<ZkContext>) -> std::result::Result<OperationResult<ZkContext>, ExecutionError> {
        // For ZK operations, we need to generate or verify a proof
        if operation.context.requires_proof() && operation.proof.is_none() && operation.zk_proof.is_none() {
            return Err(ExecutionError::MissingProof);
        }
        
        // Verify the proof if present
        if let Some(proof) = &operation.proof {
            let verification_context = VerificationContext::new()
                .with_operation_id(operation.id.to_string())
                .with_operation_type(format!("{:?}", operation.op_type))
                .with_proof(proof.clone());
            
            let verification_options = VerificationOptions::default()
                .with_strict_verification(true);
            
            let verification_result = self.verification_service.verify(verification_context, verification_options)
                .await
                .map_err(|e| ExecutionError::VerificationFailed(e.to_string()))?;
            
            if !verification_result.is_valid() {
                return Err(ExecutionError::VerificationFailed(
                    format!("Proof verification failed: {:?}", verification_result.reasons())
                ));
            }
        }
        
        // Create the result
        let result = OperationResult {
            operation: operation.clone(),
            effect_outcome: None, // ZK operations don't directly execute effects
            success: true,
            error: None,
            result_data: HashMap::new(),
        };
        
        Ok(result)
    }
}

/// Execute an operation with the given executor
pub async fn execute_operation<C: ExecutionContext>(
    operation: &Operation<C>,
    executor: &dyn OperationExecutor<C>
) -> std::result::Result<OperationResult<C>, ExecutionError> {
    // Log operation execution
    debug!("Executing operation: {:?}", operation);
    
    // Execute the operation
    let result = executor.execute(operation).await?;
    
    // Record execution for auditing
    if let Err(err) = record_operation_execution(operation).await {
        warn!("Failed to record operation execution: {:?}", err);
    }
    
    Ok(result)
}

/// Record operation execution for auditing purposes
async fn record_operation_execution<C: ExecutionContext>(
    operation: &Operation<C>
) -> std::result::Result<(), ExecutionError> {
    // TODO: Implement recording of operation execution
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::EmptyEffect;
    use crate::crypto::hash::ContentId;
    
    #[tokio::test]
    async fn test_abstract_executor() {
        // Create an abstract operation
        let abstract_context = AbstractContext::new(ExecutionPhase::Planning);
        let effect = Box::new(EmptyEffect::new("test_effect"));
        
        let operation = Operation::new(
            OperationType::Create,
            effect,
            abstract_context
        );
        
        // Create a mock interpreter and verification service
        let interpreter = Arc::new(MockInterpreter {});
        let verification_service = Arc::new(MockVerificationService {});
        
        // Create the executor
        let executor = AbstractExecutor::new(interpreter, verification_service);
        
        // Execute the operation
        let result = executor.execute(&operation).await.unwrap();
        
        // Verify the result
        assert!(result.success);
        assert!(result.effect_outcome.is_some());
        assert_eq!(result.operation.id, operation.id);
    }
    
    // Mock implementations for testing
    struct MockInterpreter {}
    
    #[async_trait]
    impl Interpreter for MockInterpreter {
        async fn execute_effect(&self, effect: &dyn Effect) -> Result<EffectOutcome, Error> {
            Ok(EffectOutcome {
                id: effect.id().to_string(),
                success: true,
                data: HashMap::new(),
                error: None,
                execution_id: None,
                resource_changes: Vec::new(),
                metadata: HashMap::new(),
            })
        }
    }
    
    struct MockVerificationService {}
    
    impl VerificationService {
        pub async fn verify(
            &self,
            _context: VerificationContext,
            _options: VerificationOptions
        ) -> Result<MockVerificationResult, Error> {
            Ok(MockVerificationResult {})
        }
    }
    
    struct MockVerificationResult {}
    
    impl MockVerificationResult {
        pub fn is_valid(&self) -> bool {
            true
        }
        
        pub fn reasons(&self) -> Vec<String> {
            Vec::new()
        }
    }
} 
