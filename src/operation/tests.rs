// Operation Tests Module
//
// This module provides test fixtures and unit tests for the unified
// operation model implementation.

use std::collections::HashMap;
use std::sync::Arc;

use crate::effect::{Effect, EffectOutcome, EmptyEffect};
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::capabilities::Capability;
use crate::verification::UnifiedProof;

use super::{
    Operation, OperationType, ExecutionContext, ExecutionPhase, ExecutionEnvironment,
    AbstractContext, RegisterContext, PhysicalContext, ZkContext,
    ResourceRef, RegisterOperation, PhysicalOperation, ResourceRefType,
    RegisterOperationType, Authorization, AuthorizationType,
    ResourceConservation, ConservationDomain,
    transformation::transform_operation
};

/// Create a test operation with an abstract context
pub fn create_test_abstract_operation() -> Operation<AbstractContext> {
    let context = AbstractContext::new(ExecutionPhase::Planning);
    let effect = Box::new(EmptyEffect::new("test_effect"));
    
    Operation::new(
        OperationType::Create,
        effect,
        context
    )
    .with_output(ResourceRef {
        resource_id: ContentId::from_str("test:resource:123").unwrap(),
        domain_id: None,
        ref_type: ResourceRefType::Output,
        before_state: None,
        after_state: Some("created".to_string()),
    })
    .with_metadata("test_key", "test_value")
}

/// Create a test operation with a register context
pub fn create_test_register_operation() -> Operation<RegisterContext> {
    let context = RegisterContext::new(ExecutionPhase::Execution, "test_namespace");
    let effect = Box::new(EmptyEffect::new("test_effect"));
    
    let abstract_op = Operation::new(
        OperationType::Create,
        effect,
        context
    )
    .with_output(ResourceRef {
        resource_id: ContentId::from_str("test:resource:123").unwrap(),
        domain_id: None,
        ref_type: ResourceRefType::Output,
        before_state: None,
        after_state: Some("created".to_string()),
    })
    .with_metadata("test_key", "test_value");
    
    // Add a concrete implementation
    let register_op = RegisterOperation {
        register_id: "test:resource:123".to_string(),
        operation: RegisterOperationType::Create,
        data: HashMap::new(),
    };
    
    abstract_op.with_concrete_implementation(register_op)
}

/// Create a test operation with a physical context
pub fn create_test_physical_operation() -> Operation<PhysicalContext> {
    let domain_id = DomainId::from("test_domain");
    let context = PhysicalContext::new(ExecutionPhase::Execution, domain_id.clone());
    let effect = Box::new(EmptyEffect::new("test_effect"));
    
    let abstract_op = Operation::new(
        OperationType::Create,
        effect,
        context
    )
    .with_output(ResourceRef {
        resource_id: ContentId::from_str("test:resource:123").unwrap(),
        domain_id: Some(domain_id.clone()),
        ref_type: ResourceRefType::Output,
        before_state: None,
        after_state: Some("created".to_string()),
    })
    .with_metadata("test_key", "test_value");
    
    // Add a concrete implementation
    let register_op = RegisterOperation {
        register_id: "test:resource:123".to_string(),
        operation: RegisterOperationType::Create,
        data: HashMap::new(),
    };
    
    // Add a physical execution
    let physical_op = PhysicalOperation {
        domain_id,
        tx_hash: Some("0x123456789abcdef".to_string()),
        block_height: Some(12345),
        data: Vec::new(),
    };
    
    abstract_op
        .with_concrete_implementation(register_op)
        .with_physical_execution(physical_op)
}

/// Create a test operation with a ZK context
pub fn create_test_zk_operation() -> Operation<ZkContext> {
    let context = ZkContext::new(ExecutionPhase::Validation, "test_circuit");
    let effect = Box::new(EmptyEffect::new("test_effect"));
    
    let abstract_op = Operation::new(
        OperationType::Create,
        effect,
        context
    )
    .with_output(ResourceRef {
        resource_id: ContentId::from_str("test:resource:123").unwrap(),
        domain_id: None,
        ref_type: ResourceRefType::Output,
        before_state: None,
        after_state: Some("created".to_string()),
    })
    .with_metadata("test_key", "test_value");
    
    // Add a proof
    let proof = UnifiedProof::new("test_proof", HashMap::new());
    
    abstract_op.with_proof(proof)
}

#[cfg(test)]
mod transformation_tests {
    use super::*;
    use crate::operation::transformation::transform_operation;
    
    #[test]
    fn test_abstract_to_register_transformation() {
        let abstract_op = create_test_abstract_operation();
        
        // Define the target context
        let register_context = RegisterContext::new(
            ExecutionPhase::Execution,
            "test_namespace"
        );
        
        // Use the transformation module to transform the operation
        let register_op = transform_abstract_to_register(
            &abstract_op,
            register_context
        ).unwrap();
        
        // Verify the transformation result
        assert_eq!(register_op.id, abstract_op.id);
        assert_eq!(register_op.op_type, abstract_op.op_type);
        assert!(register_op.concrete_implementation.is_some());
        
        let concrete = register_op.concrete_implementation.unwrap();
        assert_eq!(concrete.operation, RegisterOperationType::Create);
        assert_eq!(concrete.register_id, "test:resource:123");
    }
}

#[cfg(test)]
mod execution_tests {
    use super::*;
    use crate::operation::execution::{
        execute_operation, OperationExecutor, OperationResult, ExecutionError
    };
    use async_trait::async_trait;
    
    #[tokio::test]
    async fn test_execute_abstract_operation() {
        let operation = create_test_abstract_operation();
        let executor = TestAbstractExecutor {};
        
        let result = execute_operation(&operation, &executor).await.unwrap();
        
        assert!(result.success);
        assert!(result.effect_outcome.is_some());
        assert_eq!(result.operation.id, operation.id);
    }
    
    struct TestAbstractExecutor {}
    
    #[async_trait]
    impl OperationExecutor<AbstractContext> for TestAbstractExecutor {
        async fn execute(&self, operation: &Operation<AbstractContext>) 
            -> Result<OperationResult<AbstractContext>, ExecutionError> {
            
            // Simulated effect execution
            let effect_outcome = EffectOutcome {
                id: operation.abstract_representation.id().to_string(),
                success: true,
                data: HashMap::new(),
                error: None,
                execution_id: None,
                resource_changes: Vec::new(),
                metadata: HashMap::new(),
            };
            
            Ok(OperationResult {
                operation: operation.clone(),
                effect_outcome: Some(effect_outcome),
                success: true,
                error: None,
                result_data: HashMap::new(),
            })
        }
    }
} 
