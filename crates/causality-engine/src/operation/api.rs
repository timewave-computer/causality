// Operation API for external integrations
// Original file: src/operation/api.rs

// Operation API Module
//
// This module provides a simplified API for working with the unified operation
// model, making it easier for developers to create and manage operations.

use std::collections::HashMap;
use std::sync::Arc;
// async_trait import was unused, removing it.
// use async_trait::async_trait; 

use causality_error::{EngineResult as Result, EngineError as Error};
use causality_core::effect::{Effect, EffectOutcome};
use causality_types::ContentId;
use causality_types::DomainId;

// Import only necessary types from super
use super::{
    execution::{AbstractExecutor, RegisterExecutor, ZkExecutor, execute_operation, OperationResult}, // Use non-generic OperationResult
    // transformation::transform_operation, // Remove unused import
    zk::OperationProofGenerator,
    verification::VerificationService
};
// Import core operation types
use causality_core::resource::{Operation as CoreOperation, OperationType as CoreOperationType, ResourceId}; // Add ResourceId import
use causality_core::resource::agent::operation::IdentityId;

// Define local traits for this module
#[async_trait::async_trait] // Re-add async_trait here as it's used by the traits
pub trait ResourceRegisterTrait: Send + Sync {
    async fn create_register(&self, register_id: &str, data: &HashMap<String, String>) -> Result<()>;
    async fn update_register(&self, register_id: &str, data: &HashMap<String, String>) -> Result<()>;
    async fn transfer_register(&self, register_id: &str, new_owner: &str) -> Result<()>;
    async fn lock_register(&self, register_id: &str) -> Result<()>;
    async fn unlock_register(&self, register_id: &str) -> Result<()>;
    async fn freeze_register(&self, register_id: &str) -> Result<()>;
    async fn archive_register(&self, register_id: &str) -> Result<()>;
}

// Define Interpreter trait locally
#[async_trait::async_trait] // Re-add async_trait here
pub trait Interpreter: Send + Sync {
    async fn execute_effect(&self, effect: &dyn Effect) -> Result<EffectOutcome>;
}

/// Adapter from API Interpreter to execution Interpreter
struct InterpreterAdapter {
    inner: Arc<dyn Interpreter>,
}

#[async_trait::async_trait] // Re-add async_trait here
impl crate::operation::execution::Interpreter for InterpreterAdapter {
    async fn execute_effect(&self, effect: &dyn Effect) -> Result<EffectOutcome> {
        self.inner.execute_effect(effect).await
    }
}

/// Adapter from API ResourceRegisterTrait to execution ResourceRegisterTrait
struct ResourceRegisterAdapter {
    inner: Arc<dyn ResourceRegisterTrait>,
}

#[async_trait::async_trait] // Re-add async_trait here
impl crate::operation::execution::ResourceRegisterTrait for ResourceRegisterAdapter {
    async fn create_register(&self, register_id: &str, data: &HashMap<String, String>) -> Result<()> {
        self.inner.create_register(register_id, data).await
    }
    
    async fn update_register(&self, register_id: &str, data: &HashMap<String, String>) -> Result<()> {
        self.inner.update_register(register_id, data).await
    }
    
    async fn transfer_register(&self, register_id: &str, new_owner: &str) -> Result<()> {
        self.inner.transfer_register(register_id, new_owner).await
    }
    
    async fn lock_register(&self, register_id: &str) -> Result<()> {
        self.inner.lock_register(register_id).await
    }
    
    async fn unlock_register(&self, register_id: &str) -> Result<()> {
        self.inner.unlock_register(register_id).await
    }
    
    async fn freeze_register(&self, register_id: &str) -> Result<()> {
        self.inner.freeze_register(register_id).await
    }
    
    async fn archive_register(&self, register_id: &str) -> Result<()> {
        self.inner.archive_register(register_id).await
    }
}

/// High-level operation manager for creating and executing operations
pub struct OperationManager {
    interpreter: Arc<dyn Interpreter>,
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
        interpreter: Arc<dyn Interpreter>,
        resource_register: Arc<dyn ResourceRegisterTrait>,
        verification_service: Arc<VerificationService>,
    ) -> Self {
        // Create adapters for the internal traits
        let interpreter_adapter = Arc::new(InterpreterAdapter {
            inner: interpreter.clone(),
        });
        
        let register_adapter = Arc::new(ResourceRegisterAdapter {
            inner: resource_register.clone(),
        });
        
        // Create executors
        let abstract_executor = Arc::new(AbstractExecutor::new(
            interpreter_adapter,
            // verification_service.clone(), // Remove second argument
        ));
        
        let register_executor = Arc::new(RegisterExecutor::new(
            register_adapter,
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
        operation_type: CoreOperationType,
        effect: Box<dyn Effect>,
        identity: IdentityId,
        target: ResourceId,
    ) -> CoreOperation {
        
        CoreOperation::new(
            identity,
            operation_type,
            target,
            vec![effect], 
        )
    }
    
    /// Execute an operation in the abstract context
    pub async fn execute_abstract(
        &self,
        operation: &CoreOperation,
    ) -> Result<OperationResult> {
        execute_operation(operation, &*self.abstract_executor).await.map_err(|e| e.into())
    }
    
    /// Execute an operation in the register context
    pub async fn execute_register(
        &self,
        operation: &CoreOperation,
    ) -> Result<OperationResult> {
        execute_operation(operation, &*self.register_executor).await.map_err(|e| e.into())
    }
    
    /// Execute an operation in the ZK context
    pub async fn execute_zk(
        &self,
        operation: &CoreOperation,
    ) -> Result<OperationResult> {
        if let Some(executor) = &self.zk_executor {
            execute_operation(operation, &**executor).await.map_err(|e| e.into())
        } else {
            Err(Error::InvalidArgument("ZK executor not available".to_string()))
        }
    }
    
    /// Transform and execute an operation from abstract to register context
    pub async fn execute_as_register(
        &self,
        operation: &CoreOperation,
    ) -> Result<OperationResult> {
        self.execute_register(operation).await
    }
    
    /// Transform and execute an operation from abstract to ZK context
    pub async fn execute_as_zk(
        &self,
        operation: &CoreOperation,
    ) -> Result<OperationResult> {
        self.execute_zk(operation).await
    }
    
    /// Create and execute a transfer operation
    pub async fn transfer_resource(
        &self,
        resource_id: ContentId,
        from: &str,
        to: &str,
        _domain_id: Option<DomainId>,
        identity: IdentityId,
    ) -> Result<OperationResult> {
        let effect = crate::effect::factory::create_transfer_effect(
            resource_id.to_string(),
            from.to_string(),
            to.to_string(),
        ).map_err(|e| Error::ExecutionFailed(format!("Effect creation failed: {}", e.to_string())))?;
        
        let target_resource_id = ResourceId::from_legacy_content_id(&resource_id);
        let operation = self.create_operation(
            CoreOperationType::Custom("Transfer".to_string()),
            effect,
            identity,
            target_resource_id, 
        );
        
        self.execute_as_register(&operation).await
    }
    
    /// Create and execute a deposit operation
    pub async fn deposit_resource(
        &self,
        resource_id: ContentId,
        owner: &str,
        initial_state: Option<&str>,
        _domain_id: Option<DomainId>,
        identity: IdentityId,
    ) -> Result<OperationResult> {
        let effect = crate::effect::factory::create_deposit_effect(
            resource_id.to_string(),
            owner.to_string(),
            initial_state.unwrap_or("").to_string(),
        ).map_err(|e| Error::ExecutionFailed(e.to_string()))?;
        
        let target_resource_id = ResourceId::from_legacy_content_id(&resource_id);
        let operation = self.create_operation(
            CoreOperationType::Create,
            effect,
            identity,
            target_resource_id,
        );
        
        self.execute_as_register(&operation).await
    }
    
    /// Create and execute a withdrawal operation
    pub async fn withdraw_resource(
        &self,
        resource_id: ContentId,
        owner: &str,
        _domain_id: Option<DomainId>,
        identity: IdentityId,
    ) -> Result<OperationResult> {
        let effect = crate::effect::factory::create_withdrawal_effect(
            resource_id.to_string(),
            owner.to_string(),
        ).map_err(|e| Error::ExecutionFailed(e.to_string()))?;
        
        let target_resource_id = ResourceId::from_legacy_content_id(&resource_id);
        let operation = self.create_operation(
            CoreOperationType::Delete,
            effect,
            identity,
            target_resource_id,
        );
        
        self.execute_as_register(&operation).await
    }
}

// Convenience API for creating operations (needs update for CoreOperation)
pub mod builder {
    use super::*;
    // ResourceId is already imported via causality_core::resource::*
    // use causality_core::resource::ResourceId; 
    
    /// Builder for creating CoreOperations
    pub struct OperationBuilder {
        operation_type: CoreOperationType,
        effect: Option<Box<dyn Effect>>,
        // inputs: Vec<ResourceRef>, // CoreOperation doesn't have inputs/outputs
        // outputs: Vec<ResourceRef>,
        metadata: HashMap<String, String>,
        parameters: HashMap<String, String>,
        identity: Option<IdentityId>,
        target: Option<ResourceId>,
        // authorization: Option<Authorization>, // Authorization handling in CoreOperation TBD
    }
    
    impl OperationBuilder {
        /// Create a new operation builder
        pub fn new(operation_type: CoreOperationType) -> Self {
            Self {
                operation_type,
                effect: None,
                // inputs: Vec::new(),
                // outputs: Vec::new(),
                metadata: HashMap::new(),
                parameters: HashMap::new(),
                identity: None,
                target: None,
                // authorization: None,
            }
        }
        
        /// Set the effect for this operation
        pub fn with_effect(mut self, effect: Box<dyn Effect>) -> Self {
            self.effect = Some(effect);
            self
        }

        /// Set the identity for this operation
        pub fn with_identity(mut self, identity: IdentityId) -> Self {
            self.identity = Some(identity);
            self
        }

        /// Set the target resource ID for this operation
        pub fn with_target(mut self, target: ResourceId) -> Self {
            self.target = Some(target);
            self
        }
        
        /* Input/Output methods removed as they don't map directly to CoreOperation
        /// Add an input resource
        pub fn with_input(mut self, resource_id: ContentId, domain_id: Option<DomainId>, state: Option<&str>) -> Self { ... }
        
        /// Add an output resource
        pub fn with_output(mut self, resource_id: ContentId, domain_id: Option<DomainId>, state: Option<&str>) -> Self { ... }
        */
        
        /// Add metadata
        pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
            self.metadata.insert(key.to_string(), value.to_string());
            self
        }

        /// Add parameters
        pub fn with_parameters(mut self, params: HashMap<String, String>) -> Self {
            self.parameters = params;
            self
        }
        
        /* Authorization method removed - TBD for CoreOperation
        /// Set authorization
        pub fn with_authorization(mut self, auth_type: AuthorizationType, data: Vec<u8>, authorizer: &str) -> Self { ... }
        */
        
        /// Build the CoreOperation
        pub fn build(self) -> Result<CoreOperation> { // Return Result<CoreOperation>
            
            // Use the provided effect or create an empty one
            let effect = self.effect.unwrap_or_else(|| {
                Box::new(crate::effect::factory::EmptyEffect::new("empty_effect"))
            });

            let identity = self.identity.ok_or_else(|| Error::InvalidArgument("Identity is required".to_string()))?;
            let target = self.target.ok_or_else(|| Error::InvalidArgument("Target is required".to_string()))?;
            
            // Create the operation using the 4-argument constructor
            let mut operation = CoreOperation::new(
                identity,
                self.operation_type,
                target,
                // Pass the Vec<Box<dyn Effect>> directly
                vec![effect], 
                // Remove parameters/metadata arguments
            );

            // Assign parameters and metadata after creation
            operation.parameters = self.parameters;
            operation.metadata = self.metadata;
            
            Ok(operation)
        }
    }
    
    /// Create a transfer operation builder
    pub fn transfer(
        resource_id: ContentId,
        from: &str,
        to: &str,
        // domain_id: Option<DomainId> // Domain info might go in metadata/params
    ) -> OperationBuilder {
        let mut builder = OperationBuilder::new(CoreOperationType::Custom("Transfer".to_string()));
        builder = builder.with_target(ResourceId::from_legacy_content_id(&resource_id));
        
        // Create a transfer effect
        if let Ok(effect) = crate::effect::factory::create_transfer_effect(
            resource_id.to_string(),
            from.to_string(),
            to.to_string(),
        ) {
            builder = builder.with_effect(effect);
        }
        
        // Store from/to in parameters or metadata
        let mut params = HashMap::new();
        params.insert("from".to_string(), from.to_string());
        params.insert("to".to_string(), to.to_string());
        builder = builder.with_parameters(params);

        /* Input/Output resource state handling removed - use metadata/params
        builder = builder.with_input( ... );
        builder = builder.with_output( ... );
        */
        
        builder
    }
    
    /// Create a deposit operation builder
    pub fn deposit(
        resource_id: ContentId,
        owner: &str,
        initial_state: Option<&str>,
        // domain_id: Option<DomainId>
    ) -> OperationBuilder {
        let mut builder = OperationBuilder::new(CoreOperationType::Create);
        builder = builder.with_target(ResourceId::from_legacy_content_id(&resource_id));
        
        // Create a deposit effect
        if let Ok(effect) = crate::effect::factory::create_deposit_effect(
            resource_id.to_string(),
            owner.to_string(),
            initial_state.unwrap_or("").to_string(),
        ) {
            builder = builder.with_effect(effect);
        }
        
        // Store owner/state in parameters or metadata
        let mut params = HashMap::new();
        params.insert("owner".to_string(), owner.to_string());
        if let Some(state) = initial_state {
            params.insert("initial_state".to_string(), state.to_string());
        }
        builder = builder.with_parameters(params);

        /* Output resource state handling removed
        builder = builder.with_output( ... );
        */
        
        builder
    }
    
    /// Create a withdrawal operation builder
    pub fn withdraw(
        resource_id: ContentId,
        owner: &str,
        // domain_id: Option<DomainId>
    ) -> OperationBuilder {
        let mut builder = OperationBuilder::new(CoreOperationType::Delete);
        builder = builder.with_target(ResourceId::from_legacy_content_id(&resource_id));
        
        // Create a withdrawal effect
        if let Ok(effect) = crate::effect::factory::create_withdrawal_effect(
            resource_id.to_string(),
            owner.to_string(),
        ) {
            builder = builder.with_effect(effect);
        }
        
        // Store owner in parameters or metadata
        let mut params = HashMap::new();
        params.insert("owner".to_string(), owner.to_string());
        builder = builder.with_parameters(params);

        /* Input resource state handling removed
        builder = builder.with_input( ... );
        */
        
        builder
    }
} 
