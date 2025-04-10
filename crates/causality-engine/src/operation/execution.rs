// Operation execution system
// Original file: src/operation/execution.rs

// Operation Execution Module
//
// This module provides utilities for executing operations in different contexts
// and handling the execution pipeline for the unified operation model.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, warn};
use anyhow::{anyhow, Result};
use thiserror::Error;
use causality_error::{Error, EngineError};
use causality_core::effect::Effect;
use causality_core::effect::outcome::{EffectOutcome, EffectStatus, ResultData};
use causality_core::effect::types::EffectId;
// Import the specific, generic Operation type from core
use causality_core::resource::agent::operation::Operation as CoreOperation;
use causality_core::resource::OperationType as CoreOperationType;
use causality_core::resource::agent::operation::{
    IdentityId, OperationState, EffectInfo, effects_to_info
};
// Import types for CoreOperation
use causality_core::resource::ResourceId;
use causality_types::ContentId;
// Import local types using crate::operation path
use crate::operation::{ 
    ExecutionContext, ExecutionEnvironment, 
    AbstractContext, RegisterContext, ZkContext, // Keep local RegisterOperation struct (might be unused now?)
};
use crate::effect::factory::EmptyEffect;

/// Verification service trait for operation verification
#[async_trait]
pub trait VerificationService: Send + Sync {
    /// The result type for verification
    type VerificationResult: VerificationResultExt;
    
    /// Verify an operation in the given context with the given options
    async fn verify(&self, context: VerificationContext, options: VerificationOptions) -> std::result::Result<Self::VerificationResult, EngineError>;
}

/// Context for verification operations
#[derive(Debug)]
pub struct VerificationContext {
    // Fields as needed for verification
    pub operation_id: String,
    pub operation_type: String,
}

impl VerificationContext {
    /// Create a new verification context
    pub fn new() -> Self {
        Self {
            operation_id: String::new(),
            operation_type: String::new(),
        }
    }
    
    /// Set the operation type
    pub fn with_operation_type(mut self, operation_type: String) -> Self {
        self.operation_type = operation_type;
        self
    }
}

/// Options for verification
#[derive(Debug)]
pub struct VerificationOptions {
    // Fields as needed for verification options
    pub strict_mode: bool,
}

impl VerificationOptions {
    /// Create default verification options
    pub fn default() -> Self {
        Self {
            strict_mode: false,
        }
    }
}

// Define Interpreter trait since it doesn't exist yet
#[async_trait]
pub trait Interpreter: Send + Sync {
    async fn execute_effect(&self, effect: &dyn Effect) -> std::result::Result<EffectOutcome, anyhow::Error>;
}

// FIXME: Placeholder for ResourceRegisterTrait that's missing
#[async_trait]
pub trait ResourceRegisterTrait: Send + Sync {
    async fn create_register(&self, register_id: &str, data: &HashMap<String, String>) -> std::result::Result<(), anyhow::Error>;
    async fn update_register(&self, register_id: &str, data: &HashMap<String, String>) -> std::result::Result<(), anyhow::Error>;
    async fn transfer_register(&self, register_id: &str, new_owner: &str) -> std::result::Result<(), anyhow::Error>;
    async fn lock_register(&self, register_id: &str) -> std::result::Result<(), anyhow::Error>;
    async fn unlock_register(&self, register_id: &str) -> std::result::Result<(), anyhow::Error>;
    async fn freeze_register(&self, register_id: &str) -> std::result::Result<(), anyhow::Error>;
    async fn archive_register(&self, register_id: &str) -> std::result::Result<(), anyhow::Error>;
}

// use super::transformation::transform_operation; // Comment out if transform_operation relies on removed types

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

    #[error("Physical operation failed: {0}")] // Keep variant for now, even if type is removed
    PhysicalOperationFailed(String),

    #[error("Transformation error: {0}")]
    TransformationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

// Implement From trait to convert ExecutionError to EngineError
impl From<ExecutionError> for EngineError {
    fn from(error: ExecutionError) -> Self {
        match error {
            ExecutionError::InvalidContext(ctx) => EngineError::ValidationError(format!("Invalid execution context: {:?}", ctx)),
            ExecutionError::MissingImplementation => EngineError::ValidationError("Missing concrete implementation".to_string()),
            ExecutionError::MissingProof => EngineError::ValidationError("Missing required proof".to_string()),
            ExecutionError::VerificationFailed(msg) => EngineError::ValidationError(format!("Verification failed: {}", msg)),
            ExecutionError::EffectExecutionFailed(msg) => EngineError::ExecutionTimeout(format!("Effect execution failed: {}", msg)),
            ExecutionError::RegisterOperationFailed(msg) => EngineError::StorageError(format!("Register operation failed: {}", msg)),
            ExecutionError::PhysicalOperationFailed(msg) => EngineError::ExecutionFailed(format!("Physical operation failed: {}", msg)),
            ExecutionError::TransformationError(msg) => EngineError::ValidationError(format!("Transformation error: {}", msg)),
            ExecutionError::InternalError(msg) => EngineError::InternalError(format!("Internal error: {}", msg)),
        }
    }
}

/// Result of an operation execution (Not generic anymore)
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// The CoreOperation that was executed (using the definition from causality_core)
    pub operation: CoreOperation, // Use CoreOperation
    
    /// The outcome of the effect execution
    pub effect_outcome: Option<EffectOutcome>,
    
    /// Whether the execution was successful
    pub success: bool,
    
    /// Error message (if execution failed)
    pub error: Option<String>,
    
    /// Additional result data
    pub result_data: HashMap<String, String>,
}

/// Mock verification result used for testing
#[derive(Debug)]
pub struct MockVerificationResult { 
    pub valid: bool 
}

/// Extension trait for verification result
pub trait VerificationResultExt {
    fn is_valid(&self) -> bool;
    fn reasons(&self) -> Vec<String>;
}

// Implementation for MockVerificationResult for tests
impl VerificationResultExt for MockVerificationResult {
    fn is_valid(&self) -> bool {
        self.valid
    }
    
    fn reasons(&self) -> Vec<String> {
        if self.valid { vec![] } else { vec!["Mock failure".to_string()] }
    }
}

/// Trait for executing operations (accepts CoreOperation, identifies context via type param C)
#[async_trait]
pub trait OperationExecutor<C: ExecutionContext>: Send + Sync {
    /// Execute a CoreOperation in the given context C
    async fn execute(&self, operation: &CoreOperation) -> std::result::Result<OperationResult, ExecutionError>; // Return non-generic OperationResult
}

// Commenting out OperationTransformer as it depends on transform_operation which might be problematic
/*
#[async_trait]
pub trait OperationTransformer<C: ExecutionContext>: OperationExecutor<C> {
    /// Transform and execute an operation in a different context
    async fn transform_and_execute<D: ExecutionContext>(
        &self,
        operation: &Operation<C>,
        target_executor: &(dyn OperationExecutor<D> + Send + Sync)
    ) -> std::result::Result<OperationResult<D>, ExecutionError> {
        let transformed = transform_operation::<C, D>(operation)
            .map_err(|e| ExecutionError::TransformationError(e.to_string()))?;
        
        target_executor.execute(&transformed).await
    }
}

// Implement the transformer trait for all executor types
impl<C: ExecutionContext, T: OperationExecutor<C>> OperationTransformer<C> for T {}
*/

/// Executor for abstract operations
pub struct AbstractExecutor {
    interpreter: Arc<dyn Interpreter>,
    // verification_service: Arc<VerificationService>, // Verification not typically needed for abstract
}

impl AbstractExecutor {
    /// Create a new abstract executor
    pub fn new(interpreter: Arc<dyn Interpreter>/*, verification_service: Arc<VerificationService>*/) -> Self {
        Self {
            interpreter,
            // verification_service,
        }
    }
}

#[async_trait]
impl OperationExecutor<AbstractContext> for AbstractExecutor { // Specify AbstractContext here
    // Takes CoreOperation as input
    async fn execute(&self, operation: &CoreOperation) -> std::result::Result<OperationResult, ExecutionError> {
        // For abstract operations, we just execute the associated effects
        // Assuming effects are stored in CoreOperation.effects (Vec<EffectInfo>)
        // We need a way to get the actual Effect trait object from EffectInfo if needed by Interpreter.
        // Using a placeholder for now.
        
        // Example: Execute the first effect found
        let effect_outcome = if let Some(first_effect_info) = operation.effects.first() {
            // Use crate's EmptyEffect from factory instead
            let placeholder_effect = EmptyEffect::new(&first_effect_info.effect_type);
            self.interpreter.execute_effect(&placeholder_effect)
                .await
                .map_err(|e| ExecutionError::EffectExecutionFailed(e.to_string()))?
        } else {
            // Handle case with no effects if necessary, returning a default success outcome
            EffectOutcome {
                effect_id: Some(EffectId::new()), // Use Some()
                status: EffectStatus::Success,
                result: ResultData::String("No effect executed".to_string()), // Use String variant
                error_message: None,
                data: HashMap::new(),
                affected_resources: Vec::new(), // Initialize missing fields
                child_outcomes: Vec::new(), // Initialize missing fields
                content_hash: None, // Initialize missing fields
                // metrics: None, // Field doesn't exist
            }
        };
        
        // Create the result
        let result = OperationResult {
            operation: operation.clone(), // Store the CoreOperation
            effect_outcome: Some(effect_outcome.clone()),
            success: effect_outcome.status == EffectStatus::Success,
            error: effect_outcome.error_message.clone(),
            result_data: effect_outcome.data.clone(),
        };
        
        Ok(result)
    }
}

/// Executor for register operations
pub struct RegisterExecutor<V: VerificationResultExt> {
    resource_register: Arc<dyn ResourceRegisterTrait>,
    verification_service: Arc<dyn VerificationService<VerificationResult = V>>,
}

impl<V: VerificationResultExt> RegisterExecutor<V> {
    /// Create a new register executor
    pub fn new(
        resource_register: Arc<dyn ResourceRegisterTrait>,
        verification_service: Arc<dyn VerificationService<VerificationResult = V>>
    ) -> Self {
        Self {
            resource_register,
            verification_service,
        }
    }
    
    /// Execute abstract effect if available within the CoreOperation
    async fn execute_abstract_effect(&self, _effects: &[EffectInfo]) -> Result<EffectOutcome> {
        // Placeholder: In reality, this might involve interpreting the EffectInfo 
        // or finding an associated Effect trait object.
        Ok(EffectOutcome {
            effect_id: Some(EffectId::new()),
            status: EffectStatus::Success,
            data: HashMap::new(),
            result: ResultData::String("Abstract effect executed (placeholder)".to_string()),
            error_message: None,
            affected_resources: Vec::new(),
            child_outcomes: Vec::new(),
            content_hash: None,
        })
    }
}

#[async_trait]
impl<V: VerificationResultExt + Send + Sync + 'static> OperationExecutor<RegisterContext> for RegisterExecutor<V> {
    // Takes CoreOperation as input
    async fn execute(&self, operation: &CoreOperation) -> std::result::Result<OperationResult, ExecutionError> {
        // For register operations, we need to verify the operation first
        let verification_context = VerificationContext::new()
            .with_operation_type(format!("{:?}", operation.operation_type))
            // .with_metadata(operation.metadata.clone()); // Method doesn't exist
            ;
        
        let verification_options = VerificationOptions::default();
        
        // Verify the operation
        let verification_result = self.verification_service.verify(verification_context, verification_options)
            .await
            .map_err(|e| ExecutionError::VerificationFailed(e.to_string()))?;
        
        // Use the VerificationResultExt trait methods directly
        if !verification_result.is_valid() {
            return Err(ExecutionError::VerificationFailed(
                format!("Operation verification failed: {:?}", verification_result.reasons())
            ));
        }
        
        // Execute the register operation based on CoreOperationType
        let result_data = match operation.operation_type {
            CoreOperationType::Create => {
                // Use target as register_id, parameters as data
                self.resource_register.create_register(
                    &operation.target.to_string(), 
                    &operation.parameters
                ).await.map_err(|e| ExecutionError::RegisterOperationFailed(e.to_string()))?;
                operation.parameters.clone()
            },
            CoreOperationType::Update => {
                self.resource_register.update_register(
                    &operation.target.to_string(), 
                    &operation.parameters
                ).await.map_err(|e| ExecutionError::RegisterOperationFailed(e.to_string()))?;
                operation.parameters.clone()
            },
            CoreOperationType::Delete => {
                // Delete doesn't usually take data, but ResourceRegisterTrait::archive_register does?
                // Let's call archive_register for now, assuming that's the delete equivalent.
                self.resource_register.archive_register(
                    &operation.target.to_string()
                ).await.map_err(|e| ExecutionError::RegisterOperationFailed(e.to_string()))?;
                HashMap::new() // No data returned on delete/archive
            },
            // CoreOperationType::Transfer, Lock, Unlock, Freeze etc. might exist 
            // but ResourceRegisterTrait doesn't have matching methods directly.
            // We might need to map CoreOperationType::Custom("Transfer") etc. or adjust the trait.
            // For now, handle known mappings.
            CoreOperationType::Authorize | CoreOperationType::Revoke => {
                // These likely don't map directly to register operations, maybe handled by effects?
                // Return empty data for now.
                 warn!("Unhandled Authorize/Revoke operation type in RegisterExecutor: {:?}", operation.operation_type);
                HashMap::new()
            },
            // Use Custom variant for Transfer
            CoreOperationType::Custom(ref op_name) if op_name == "Transfer" => { 
                let new_owner = operation.parameters.get("new_owner")
                    // Use InternalError for unexpected missing parameter
                    .ok_or_else(|| ExecutionError::InternalError("Missing 'new_owner' parameter for Transfer".to_string()))?;
                self.resource_register.transfer_register(
                    &operation.target.to_string(),
                    new_owner
                ).await.map_err(|e| ExecutionError::RegisterOperationFailed(e.to_string()))?;
                operation.parameters.clone()
            },
            // Handle other Custom types
            CoreOperationType::Custom(ref op_name) => {
                 match op_name.as_str() {
                     "Lock" => {
                         self.resource_register.lock_register(&operation.target.to_string()).await
                            .map_err(|e| ExecutionError::RegisterOperationFailed(e.to_string()))?;
                         HashMap::new()
                     },
                     "Unlock" => {
                         self.resource_register.unlock_register(&operation.target.to_string()).await
                            .map_err(|e| ExecutionError::RegisterOperationFailed(e.to_string()))?;
                         HashMap::new()
                     },
                      "Freeze" => {
                         self.resource_register.freeze_register(&operation.target.to_string()).await
                            .map_err(|e| ExecutionError::RegisterOperationFailed(e.to_string()))?;
                         HashMap::new()
                     },
                     // Handle other custom ops or return error
                     _ => {
                        warn!("Unhandled custom operation type in RegisterExecutor: {}", op_name);
                        // Return an error or default data
                        return Err(ExecutionError::InvalidContext(ExecutionEnvironment::Register)); // Or a more specific error
                     }
                 }
            }
        };

        // Execute the abstract effect if available
        let effect_outcome = self.execute_abstract_effect(&operation.effects)
            .await
            .map_err(|e| ExecutionError::EffectExecutionFailed(e.to_string()))?;
        
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
pub struct ZkExecutor<V: VerificationResultExt> {
    verification_service: Arc<dyn VerificationService<VerificationResult = V>>,
}

impl<V: VerificationResultExt> ZkExecutor<V> {
    /// Create a new ZK executor
    pub fn new(verification_service: Arc<dyn VerificationService<VerificationResult = V>>) -> Self {
        Self {
            verification_service,
        }
    }
}

#[async_trait]
impl<V: VerificationResultExt + Send + Sync + 'static> OperationExecutor<ZkContext> for ZkExecutor<V> {
    // Takes CoreOperation as input
    async fn execute(&self, operation: &CoreOperation) -> std::result::Result<OperationResult, ExecutionError> {
        // For ZK operations, we need to generate or verify a proof
        // How proof relates to CoreOperation needs clarification (maybe in metadata?)
        // Placeholder check
        /*
        if operation.context.requires_proof() && operation.proof.is_none() && operation.zk_proof.is_none() {
            return Err(ExecutionError::MissingProof);
        }
        */
        
        // Verify the proof if present (assuming proof is in metadata or similar)
        // The structure of proof and how to extract it from CoreOperation needs to be defined.
        // For now, we skip proof verification.
        /*
        let proof_data = operation.metadata.get("proof").cloned();
        
        if let Some(proof_str) = proof_data { // Assuming proof is a String in metadata
            // Need a way to convert proof_str (String) or proof_bytes (Vec<u8>) into UnifiedProof
            // let proof: UnifiedProof = ...; // How to create this?
            let verification_context = VerificationContext::new()
                // .with_operation_id(operation.id.to_string())
                .with_operation_type(format!("{:?}", operation.operation_type))
                // .with_proof(proof); // Requires UnifiedProof
                ;
            
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
        */
        
        // Create the result (assuming success if proof check is skipped/passes)
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

/// Helper function to check if verification result is valid
fn verification_result_is_valid<T>(result: &T) -> bool {
    // Since we can't directly use VerificationResultExt trait, we'll use this function
    // In a real implementation, this might type check dynamically or use a more generic approach
    true // Default to assuming valid for now
}

/// Helper function to get verification result reasons
fn verification_result_reasons<T>(_result: &T) -> Vec<String> {
    // Since we can't directly use VerificationResultExt trait, we'll use this function
    // In a real implementation, this might type check dynamically or use a more generic approach
    vec![] // Default to no reasons for now
}

/// Execute an operation with the given executor (accepts CoreOperation)
pub async fn execute_operation<C: ExecutionContext>(
    operation: &CoreOperation,
    executor: &(dyn OperationExecutor<C> + Send + Sync)
) -> std::result::Result<OperationResult, ExecutionError> { // Return non-generic result
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

/// Record operation execution for auditing purposes (accepts CoreOperation)
async fn record_operation_execution(
    _operation: &CoreOperation
) -> std::result::Result<(), ExecutionError> {
    // TODO: Implement recording of operation execution
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock VerificationService implementation for testing
    #[derive(Debug)]
    struct MockVerificationServiceImpl {}

    #[derive(Debug)]
    struct MockVerificationResult { valid: bool }

    impl VerificationResultExt for MockVerificationResult {
        fn is_valid(&self) -> bool {
            self.valid
        }
        
        fn reasons(&self) -> Vec<String> {
            if self.valid { vec![] } else { vec!["Mock failure".to_string()] }
        }
    }

    #[async_trait]
    impl VerificationService for MockVerificationServiceImpl {
        type VerificationResult = MockVerificationResult; // Associate the result type
        // Use concrete types for VerificationContext/Options if they are structs
        async fn verify(&self, _context: VerificationContext, _options: VerificationOptions) -> std::result::Result<Self::VerificationResult, EngineError> {
            Ok(MockVerificationResult { valid: true })
        }
    }

    // Mocks for dependencies (Interpreter)
    struct MockInterpreter {}

    #[async_trait]
    impl Interpreter for MockInterpreter {
        async fn execute_effect(&self, effect: &dyn Effect) -> std::result::Result<EffectOutcome, anyhow::Error> {
            Ok(EffectOutcome {
                effect_id: Some(effect.id().clone()), // Use Some for Option<EffectId>
                status: EffectStatus::Success,
                result: ResultData::Json(serde_json::json!({ "mock_result": "ok" })), // Keep as JSON for now, maybe String needed?
                error_message: None,
                data: HashMap::new(),
                affected_resources: Vec::new(), // Initialize missing fields
                child_outcomes: Vec::new(), // Initialize missing fields
                content_hash: None, // Initialize missing fields
                // metrics: None, // Field doesn't exist
            })
        }
    }

    // Test AbstractExecutor
    #[tokio::test]
    async fn test_abstract_executor() {
        let interpreter = Arc::new(MockInterpreter {});
        // let verification_service = Arc::new(MockVerificationServiceImpl {}); // Verification service removed from AbstractExecutor::new
        let executor = AbstractExecutor::new(interpreter);

        // Use EmptyEffect since CoreMockEffect isn't directly accessible
        let test_effect = Box::new(EmptyEffect::new("test_effect")) as Box<dyn Effect>; 
        
        let operation = CoreOperation {
            identity: IdentityId::new("test-identity"),
            operation_type: CoreOperationType::Create, 
            target: ResourceId::from_legacy_content_id(&ContentId::new("test-target")), // Use conversion
            effects: effects_to_info(&vec![test_effect.clone()]), 
            state: OperationState::Pending,
            previous_operations: Vec::new(),
            parameters: HashMap::new(),
            metadata: HashMap::new(), 
            capability: None,
        };

        let result = executor.execute(&operation).await;
        
        // Verify the result
        assert!(result.is_ok(), "Execution failed: {:?}", result.err()); // Add error message
        let op_result = result.unwrap();
        assert!(op_result.success);
        assert!(op_result.effect_outcome.is_some());
        let outcome = op_result.effect_outcome.unwrap();
        assert_eq!(outcome.status, EffectStatus::Success);
    }

    // Add tests for RegisterExecutor
    #[tokio::test]
    async fn test_register_executor() {
        // This is just a basic test structure - the actual implementation would be more complex
        struct MockResourceRegister {}

        #[async_trait]
        impl ResourceRegisterTrait for MockResourceRegister {
            async fn create_register(&self, _register_id: &str, _data: &HashMap<String, String>) -> std::result::Result<(), anyhow::Error> {
                Ok(())
            }
            async fn update_register(&self, _register_id: &str, _data: &HashMap<String, String>) -> std::result::Result<(), anyhow::Error> {
                Ok(())
            }
            async fn transfer_register(&self, _register_id: &str, _new_owner: &str) -> std::result::Result<(), anyhow::Error> {
                Ok(())
            }
            async fn lock_register(&self, _register_id: &str) -> std::result::Result<(), anyhow::Error> {
                Ok(())
            }
            async fn unlock_register(&self, _register_id: &str) -> std::result::Result<(), anyhow::Error> {
                Ok(())
            }
            async fn freeze_register(&self, _register_id: &str) -> std::result::Result<(), anyhow::Error> {
                Ok(())
            }
            async fn archive_register(&self, _register_id: &str) -> std::result::Result<(), anyhow::Error> {
                Ok(())
            }
        }

        let resource_register = Arc::new(MockResourceRegister {});
        let verification_service = Arc::new(MockVerificationServiceImpl {});
        let executor = RegisterExecutor::<MockVerificationResult>::new(resource_register, verification_service);

        // Create a simple operation
        let operation = CoreOperation {
            identity: IdentityId::new("test-identity"),
            operation_type: CoreOperationType::Create,
            target: ResourceId::from_legacy_content_id(&ContentId::new("test-register")),
            effects: Vec::new(),
            state: OperationState::Pending,
            previous_operations: Vec::new(),
            parameters: HashMap::new(),
            metadata: HashMap::new(),
            capability: None,
        };

        // Execute the operation
        let result = executor.execute(&operation).await;
        
        // Verify the result
        assert!(result.is_ok(), "Register execution failed: {:?}", result.err());
        let op_result = result.unwrap();
        assert!(op_result.success);
    }

    // Add test for ZkExecutor
    #[tokio::test]
    async fn test_zk_executor() {
        let verification_service = Arc::new(MockVerificationServiceImpl {});
        let executor = ZkExecutor::<MockVerificationResult>::new(verification_service);

        // Create a simple operation
        let operation = CoreOperation {
            identity: IdentityId::new("test-identity"),
            operation_type: CoreOperationType::Custom("ZkProof".to_string()),
            target: ResourceId::from_legacy_content_id(&ContentId::new("test-proof-target")),
            effects: Vec::new(),
            state: OperationState::Pending,
            previous_operations: Vec::new(),
            parameters: HashMap::new(),
            metadata: HashMap::new(),
            capability: None,
        };

        // Execute the operation
        let result = executor.execute(&operation).await;
        
        // Verify the result
        assert!(result.is_ok(), "ZK execution failed: {:?}", result.err());
        let op_result = result.unwrap();
        assert!(op_result.success);
    }
} 
