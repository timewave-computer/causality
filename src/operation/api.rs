// Operation API Module
//
// This module provides a simplified API for working with the unified operation
// model, making it easier for developers to create and manage operations.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::effect::{Effect, EffectOutcome};
use crate::types::{ResourceId, DomainId};
use crate::verification::UnifiedProof;
use crate::interpreter::Interpreter;
use crate::resource::{ResourceRegisterTrait};
use crate::verification::VerificationService;

use super::{
    Operation, OperationType, ExecutionContext, ExecutionPhase, ExecutionEnvironment,
    AbstractContext, RegisterContext, PhysicalContext, ZkContext,
    ResourceRef, RegisterOperation, PhysicalOperation, ResourceRefType,
    RegisterOperationType, Authorization, AuthorizationType, ResourceConservation, ConservationDomain,
    execution::{OperationExecutor, OperationResult, ExecutionError, AbstractExecutor, RegisterExecutor, 
                ZkExecutor, execute_operation},
    transformation::transform_operation,
    zk::{OperationProofGenerator, CircuitSelector, DefaultCircuitSelector}
};

/// High-level operation manager for creating and executing operations
pub struct OperationManager {
    interpreter: Arc<Interpreter>,
    resource_register: Arc<dyn ResourceRegisterTrait>,
    verification_service: Arc<VerificationService>,
    proof_generator: Option<Arc<OperationProofGenerator>>,
    
    // Executors for different contexts
    abstract_executor: Arc<AbstractExecutor>,
    register_executor: Arc<RegisterExecutor>,
    zk_executor: Option<Arc<ZkExecutor>>,
}

impl OperationManager {
    /// Create a new operation manager
    pub fn new(
        interpreter: Arc<Interpreter>,
        resource_register: Arc<dyn ResourceRegisterTrait>,
        verification_service: Arc<VerificationService>,
    ) -> Self {
        // Create executors
        let abstract_executor = Arc::new(AbstractExecutor::new(
            interpreter.clone(),
            verification_service.clone(),
        ));
        
        let register_executor = Arc::new(RegisterExecutor::new(
            resource_register.clone(),
            verification_service.clone(),
        ));
        
        let zk_executor = Some(Arc::new(ZkExecutor::new(
            verification_service.clone(),
        )));
        
        Self {
            interpreter,
            resource_register,
            verification_service,
            proof_generator: None,
            abstract_executor,
            register_executor,
            zk_executor,
        }
    }
    
    /// Set the proof generator for ZK operations
    pub fn with_proof_generator(mut self, proof_generator: Arc<OperationProofGenerator>) -> Self {
        self.proof_generator = Some(proof_generator);
        self
    }
    
    /// Create a new abstract operation
    pub fn create_operation(
        &self,
        operation_type: OperationType,
        effect: Box<dyn Effect>,
    ) -> Operation<AbstractContext> {
        let context = AbstractContext::new(ExecutionPhase::Planning);
        
        Operation::new(
            operation_type,
            effect,
            context
        )
    }
    
    /// Execute an operation in the abstract context
    pub async fn execute_abstract(
        &self,
        operation: &Operation<AbstractContext>,
    ) -> Result<OperationResult<AbstractContext>, ExecutionError> {
        execute_operation(operation, &*self.abstract_executor).await
    }
    
    /// Execute an operation in the register context
    pub async fn execute_register(
        &self,
        operation: &Operation<RegisterContext>,
    ) -> Result<OperationResult<RegisterContext>, ExecutionError> {
        execute_operation(operation, &*self.register_executor).await
    }
    
    /// Execute an operation in the ZK context
    pub async fn execute_zk(
        &self,
        operation: &Operation<ZkContext>,
    ) -> Result<OperationResult<ZkContext>, ExecutionError> {
        if let Some(executor) = &self.zk_executor {
            execute_operation(operation, &**executor).await
        } else {
            Err(ExecutionError::InvalidContext(ExecutionEnvironment::ZkVm))
        }
    }
    
    /// Transform and execute an operation from abstract to register context
    pub async fn execute_as_register(
        &self,
        operation: &Operation<AbstractContext>,
    ) -> Result<OperationResult<RegisterContext>, ExecutionError> {
        // Transform the operation to register context
        let register_operation = transform_operation::<AbstractContext, RegisterContext>(operation)
            .map_err(|e| ExecutionError::TransformationError(e.to_string()))?;
        
        // Execute in register context
        self.execute_register(&register_operation).await
    }
    
    /// Transform and execute an operation from abstract to ZK context
    pub async fn execute_as_zk(
        &self,
        operation: &Operation<AbstractContext>,
    ) -> Result<OperationResult<ZkContext>, ExecutionError> {
        // Generate proof if we have a proof generator
        let zk_operation = if let Some(generator) = &self.proof_generator {
            generator.transform_to_zk_operation(operation)
                .await
                .map_err(|e| ExecutionError::TransformationError(e.to_string()))?
        } else {
            // Use regular transformation without proof
            transform_operation::<AbstractContext, ZkContext>(operation)
                .map_err(|e| ExecutionError::TransformationError(e.to_string()))?
        };
        
        // Execute in ZK context
        self.execute_zk(&zk_operation).await
    }
    
    /// Create and execute a transfer operation
    pub async fn transfer_resource(
        &self,
        resource_id: ResourceId,
        from: &str,
        to: &str,
        domain_id: Option<DomainId>,
    ) -> Result<OperationResult<RegisterContext>, ExecutionError> {
        // Create a transfer effect
        let effect = crate::effect::factory::create_transfer_effect(
            resource_id.to_string(),
            from.to_string(),
            to.to_string(),
        );
        
        // Create the operation
        let mut operation = self.create_operation(OperationType::Transfer, Box::new(effect));
        
        // Add input and output resources
        let input_ref = ResourceRef {
            resource_id: resource_id.clone(),
            domain_id: domain_id.clone(),
            ref_type: ResourceRefType::Input,
            before_state: Some("owned_by_".to_string() + from),
            after_state: None,
        };
        
        let output_ref = ResourceRef {
            resource_id,
            domain_id,
            ref_type: ResourceRefType::Output,
            before_state: None,
            after_state: Some("owned_by_".to_string() + to),
        };
        
        operation = operation.with_input(input_ref).with_output(output_ref);
        
        // Execute as register operation
        self.execute_as_register(&operation).await
    }
    
    /// Create and execute a deposit operation
    pub async fn deposit_resource(
        &self,
        resource_id: ResourceId,
        owner: &str,
        initial_state: Option<&str>,
        domain_id: Option<DomainId>,
    ) -> Result<OperationResult<RegisterContext>, ExecutionError> {
        // Create a deposit effect
        let effect = crate::effect::factory::create_deposit_effect(
            resource_id.to_string(),
            owner.to_string(),
            initial_state.unwrap_or("").to_string(),
        );
        
        // Create the operation
        let mut operation = self.create_operation(OperationType::Deposit, Box::new(effect));
        
        // Add output resource
        let output_ref = ResourceRef {
            resource_id,
            domain_id,
            ref_type: ResourceRefType::Output,
            before_state: None,
            after_state: initial_state.map(|s| s.to_string()),
        };
        
        operation = operation.with_output(output_ref);
        
        // Execute as register operation
        self.execute_as_register(&operation).await
    }
    
    /// Create and execute a withdrawal operation
    pub async fn withdraw_resource(
        &self,
        resource_id: ResourceId,
        owner: &str,
        domain_id: Option<DomainId>,
    ) -> Result<OperationResult<RegisterContext>, ExecutionError> {
        // Create a withdrawal effect
        let effect = crate::effect::factory::create_withdrawal_effect(
            resource_id.to_string(),
            owner.to_string(),
        );
        
        // Create the operation
        let mut operation = self.create_operation(OperationType::Withdrawal, Box::new(effect));
        
        // Add input resource
        let input_ref = ResourceRef {
            resource_id,
            domain_id,
            ref_type: ResourceRefType::Input,
            before_state: Some("owned_by_".to_string() + owner),
            after_state: None,
        };
        
        operation = operation.with_input(input_ref);
        
        // Execute as register operation
        self.execute_as_register(&operation).await
    }
}

// Convenience API for creating operations
pub mod builder {
    use super::*;
    
    /// Builder for creating operations
    pub struct OperationBuilder {
        operation_type: OperationType,
        effect: Option<Box<dyn Effect>>,
        inputs: Vec<ResourceRef>,
        outputs: Vec<ResourceRef>,
        metadata: HashMap<String, String>,
        authorization: Option<Authorization>,
    }
    
    impl OperationBuilder {
        /// Create a new operation builder
        pub fn new(operation_type: OperationType) -> Self {
            Self {
                operation_type,
                effect: None,
                inputs: Vec::new(),
                outputs: Vec::new(),
                metadata: HashMap::new(),
                authorization: None,
            }
        }
        
        /// Set the effect for this operation
        pub fn with_effect(mut self, effect: Box<dyn Effect>) -> Self {
            self.effect = Some(effect);
            self
        }
        
        /// Add an input resource
        pub fn with_input(mut self, resource_id: ResourceId, domain_id: Option<DomainId>, state: Option<&str>) -> Self {
            let resource_ref = ResourceRef {
                resource_id,
                domain_id,
                ref_type: ResourceRefType::Input,
                before_state: state.map(|s| s.to_string()),
                after_state: None,
            };
            
            self.inputs.push(resource_ref);
            self
        }
        
        /// Add an output resource
        pub fn with_output(mut self, resource_id: ResourceId, domain_id: Option<DomainId>, state: Option<&str>) -> Self {
            let resource_ref = ResourceRef {
                resource_id,
                domain_id,
                ref_type: ResourceRefType::Output,
                before_state: None,
                after_state: state.map(|s| s.to_string()),
            };
            
            self.outputs.push(resource_ref);
            self
        }
        
        /// Add metadata
        pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
            self.metadata.insert(key.to_string(), value.to_string());
            self
        }
        
        /// Set authorization
        pub fn with_authorization(mut self, auth_type: AuthorizationType, data: Vec<u8>, authorizer: &str) -> Self {
            self.authorization = Some(Authorization {
                auth_type,
                data,
                authorizer: authorizer.to_string(),
            });
            
            self
        }
        
        /// Build the operation
        pub fn build(self) -> Operation<AbstractContext> {
            let context = AbstractContext::new(ExecutionPhase::Planning);
            
            // Use the provided effect or create an empty one
            let effect = self.effect.unwrap_or_else(|| {
                Box::new(crate::effect::EmptyEffect::new("empty_effect"))
            });
            
            let mut operation = Operation::new(
                self.operation_type,
                effect,
                context
            );
            
            // Add inputs and outputs
            for input in self.inputs {
                operation = operation.with_input(input);
            }
            
            for output in self.outputs {
                operation = operation.with_output(output);
            }
            
            // Add metadata
            for (key, value) in self.metadata {
                operation = operation.with_metadata(&key, &value);
            }
            
            // Add authorization if provided
            if let Some(auth) = self.authorization {
                operation = operation.with_authorization(auth);
            }
            
            operation
        }
    }
    
    /// Create a transfer operation
    pub fn transfer(
        resource_id: ResourceId,
        from: &str,
        to: &str,
        domain_id: Option<DomainId>
    ) -> OperationBuilder {
        let mut builder = OperationBuilder::new(OperationType::Transfer);
        
        // Create a transfer effect
        let effect = crate::effect::factory::create_transfer_effect(
            resource_id.to_string(),
            from.to_string(),
            to.to_string(),
        );
        
        // Add the effect
        builder = builder.with_effect(Box::new(effect));
        
        // Add input and output resources
        builder = builder.with_input(
            resource_id.clone(),
            domain_id.clone(),
            Some(&format!("owned_by_{}", from))
        );
        
        builder = builder.with_output(
            resource_id,
            domain_id,
            Some(&format!("owned_by_{}", to))
        );
        
        builder
    }
    
    /// Create a deposit operation
    pub fn deposit(
        resource_id: ResourceId,
        owner: &str,
        initial_state: Option<&str>,
        domain_id: Option<DomainId>
    ) -> OperationBuilder {
        let mut builder = OperationBuilder::new(OperationType::Deposit);
        
        // Create a deposit effect
        let effect = crate::effect::factory::create_deposit_effect(
            resource_id.to_string(),
            owner.to_string(),
            initial_state.unwrap_or("").to_string(),
        );
        
        // Add the effect
        builder = builder.with_effect(Box::new(effect));
        
        // Add output resource
        builder = builder.with_output(
            resource_id,
            domain_id,
            initial_state
        );
        
        builder
    }
    
    /// Create a withdrawal operation
    pub fn withdraw(
        resource_id: ResourceId,
        owner: &str,
        domain_id: Option<DomainId>
    ) -> OperationBuilder {
        let mut builder = OperationBuilder::new(OperationType::Withdrawal);
        
        // Create a withdrawal effect
        let effect = crate::effect::factory::create_withdrawal_effect(
            resource_id.to_string(),
            owner.to_string(),
        );
        
        // Add the effect
        builder = builder.with_effect(Box::new(effect));
        
        // Add input resource
        builder = builder.with_input(
            resource_id,
            domain_id,
            Some(&format!("owned_by_{}", owner))
        );
        
        builder
    }
} 