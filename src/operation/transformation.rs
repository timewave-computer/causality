// Operation Transformation Module
//
// This module provides utilities for transforming operations between different
// execution contexts, allowing operations to move through the various stages
// of their lifecycle.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{Error};
use crate::effect::{Effect, EffectOutcome};
use crate::types::DomainId;
use crate::verification::UnifiedProof;

use super::{
    Operation, OperationType, ExecutionContext, ExecutionPhase, ExecutionEnvironment,
    AbstractContext, RegisterContext, PhysicalContext, ZkContext,
    ResourceRef, RegisterOperation, PhysicalOperation, ResourceRefType,
    RegisterOperationType, Authorization
};

/// Error when transforming operations between contexts
#[derive(Debug, thiserror::Error)]
pub enum TransformationError {
    #[error("Cannot transform operation from {from} to {to}")]
    InvalidTransformation { from: String, to: String },

    #[error("Missing concrete implementation")]
    MissingImplementation,

    #[error("Missing domain ID for physical execution")]
    MissingDomainId,

    #[error("Missing proof for ZK context")]
    MissingProof,

    #[error("Invalid operation type for transformation: {0}")]
    InvalidOperationType(String),

    #[error("Resource transformation error: {0}")]
    ResourceError(String),

    #[error("Authorization transformation error: {0}")]
    AuthorizationError(String),

    #[error("Transformation not implemented: {0}")]
    NotImplemented(String),
}

/// Transform an operation to a different execution context
pub fn transform_operation<C: ExecutionContext, D: ExecutionContext>(
    operation: &Operation<C>,
) -> std::result::Result<Operation<D>, TransformationError> {
    let from_env = operation.context.environment();
    let to_env = D::default_environment();

    // Validate the transformation is possible
    validate_transformation_path(&from_env, &to_env)?;

    // Create the appropriate target context
    let target_context = create_target_context::<D>(&operation.context)?;

    // Transform the operation
    let transformed_operation = match (from_env, to_env) {
        (ExecutionEnvironment::Abstract, ExecutionEnvironment::Register) => {
            transform_abstract_to_register(operation, target_context)?
        },
        (ExecutionEnvironment::Register, ExecutionEnvironment::OnChain(_)) => {
            transform_register_to_physical(operation, target_context)?
        },
        (ExecutionEnvironment::Abstract, ExecutionEnvironment::ZkVm) => {
            transform_abstract_to_zk(operation, target_context)?
        },
        (ExecutionEnvironment::OnChain(_), ExecutionEnvironment::Abstract) => {
            transform_physical_to_abstract(operation, target_context)?
        },
        _ => {
            return Err(TransformationError::NotImplemented(
                format!("Transformation from {:?} to {:?} not implemented", from_env, to_env)
            ));
        }
    };

    Ok(transformed_operation)
}

/// Validate that a transformation path is valid
fn validate_transformation_path(
    from: &ExecutionEnvironment,
    to: &ExecutionEnvironment,
) -> std::result::Result<(), TransformationError> {
    match (from, to) {
        // Valid transformations
        (ExecutionEnvironment::Abstract, ExecutionEnvironment::Register) => Ok(()),
        (ExecutionEnvironment::Abstract, ExecutionEnvironment::ZkVm) => Ok(()),
        (ExecutionEnvironment::Register, ExecutionEnvironment::OnChain(_)) => Ok(()),
        (ExecutionEnvironment::OnChain(_), ExecutionEnvironment::Abstract) => Ok(()),
        (ExecutionEnvironment::ZkVm, ExecutionEnvironment::Abstract) => Ok(()),
        (ExecutionEnvironment::ZkVm, ExecutionEnvironment::OnChain(_)) => Ok(()),
        
        // Invalid transformations
        _ => Err(TransformationError::InvalidTransformation {
            from: format!("{:?}", from),
            to: format!("{:?}", to),
        }),
    }
}

/// Create a target context of the appropriate type
fn create_target_context<D: ExecutionContext>(
    source_context: &dyn ExecutionContext,
) -> std::result::Result<D, TransformationError> {
    D::from_previous_context(source_context)
        .map_err(|_| TransformationError::InvalidTransformation {
            from: format!("{:?}", source_context.environment()),
            to: format!("{:?}", D::default_environment()),
        })
}

/// Transform an abstract operation to a register operation
pub fn transform_abstract_to_register<C: ExecutionContext>(
    operation: &Operation<C>,
    target_context: RegisterContext,
) -> std::result::Result<Operation<RegisterContext>, TransformationError> {
    // Create a new register operation based on the abstract operation
    let register_operation = create_register_operation_from_abstract(operation)?;
    
    // Create a new operation with the register context
    Ok(Operation {
        id: operation.id.clone(),
        op_type: operation.op_type.clone(),
        abstract_representation: operation.abstract_representation.clone(),
        concrete_implementation: Some(register_operation),
        physical_execution: None,
        context: target_context,
        inputs: operation.inputs.clone(),
        outputs: operation.outputs.clone(),
        authorization: operation.authorization.clone(),
        proof: operation.proof.clone(),
        zk_proof: operation.zk_proof.clone(),
        conservation: operation.conservation.clone(),
        metadata: operation.metadata.clone(),
    })
}

/// Create a register operation from an abstract operation
fn create_register_operation_from_abstract<C: ExecutionContext>(
    operation: &Operation<C>,
) -> std::result::Result<RegisterOperation, TransformationError> {
    // Extract register ID from the first output (if available)
    let register_id = operation.outputs.first()
        .map(|output| output.resource_id.to_string())
        .ok_or_else(|| TransformationError::ResourceError(
            "No output resource specified for register operation".to_string()
        ))?;
    
    // Map the operation type to register operation type
    let register_op_type = match operation.op_type {
        OperationType::Create => RegisterOperationType::Create,
        OperationType::Update => RegisterOperationType::Update,
        OperationType::Delete => RegisterOperationType::Archive, // Delete maps to archive in register model
        OperationType::Transfer => RegisterOperationType::Transfer,
        OperationType::Merge => RegisterOperationType::Custom("Merge".to_string()),
        OperationType::Split => RegisterOperationType::Custom("Split".to_string()),
        OperationType::Deposit => RegisterOperationType::Create, // Deposit is a special case of create
        OperationType::Withdrawal => RegisterOperationType::Custom("Withdrawal".to_string()),
        OperationType::Custom(ref name) => RegisterOperationType::Custom(name.clone()),
    };
    
    // Gather operation data from metadata and inputs/outputs
    let mut data = HashMap::new();
    
    // Add basic operation information
    data.insert("operation_id".to_string(), operation.id.to_string());
    data.insert("operation_type".to_string(), format!("{:?}", operation.op_type));
    
    // Add metadata from the original operation
    for (key, value) in &operation.metadata {
        data.insert(format!("meta_{}", key), value.clone());
    }
    
    // Create the register operation
    Ok(RegisterOperation {
        register_id,
        operation: register_op_type,
        data,
    })
}

/// Transform a register operation to a physical operation
fn transform_register_to_physical<C: ExecutionContext>(
    operation: &Operation<C>,
    target_context: PhysicalContext,
) -> std::result::Result<Operation<PhysicalContext>, TransformationError> {
    // Get the concrete implementation
    let register_op = operation.concrete_implementation.as_ref()
        .ok_or(TransformationError::MissingImplementation)?;
    
    // Create physical operation for the specified domain
    let domain_id = target_context.domain_id.clone();
    
    let physical_op = PhysicalOperation {
        domain_id: domain_id.clone(),
        tx_hash: None, // Will be filled in after execution
        block_height: None, // Will be filled in after execution
        data: serde_json::to_vec(&register_op)
            .map_err(|e| TransformationError::ResourceError(e.to_string()))?,
    };
    
    // Create a new operation with the physical context
    Ok(Operation {
        id: operation.id.clone(),
        op_type: operation.op_type.clone(),
        abstract_representation: operation.abstract_representation.clone(),
        concrete_implementation: operation.concrete_implementation.clone(),
        physical_execution: Some(physical_op),
        context: target_context,
        inputs: operation.inputs.clone(),
        outputs: operation.outputs.clone(),
        authorization: operation.authorization.clone(),
        proof: operation.proof.clone(),
        zk_proof: operation.zk_proof.clone(),
        conservation: operation.conservation.clone(),
        metadata: operation.metadata.clone(),
    })
}

/// Transform an abstract operation to a ZK operation
fn transform_abstract_to_zk<C: ExecutionContext>(
    operation: &Operation<C>,
    target_context: ZkContext,
) -> std::result::Result<Operation<ZkContext>, TransformationError> {
    // For ZK transformations, we need to ensure there's a proof
    if target_context.requires_proof() && operation.proof.is_none() {
        return Err(TransformationError::MissingProof);
    }
    
    // Create a new operation with the ZK context
    Ok(Operation {
        id: operation.id.clone(),
        op_type: operation.op_type.clone(),
        abstract_representation: operation.abstract_representation.clone(),
        concrete_implementation: operation.concrete_implementation.clone(),
        physical_execution: operation.physical_execution.clone(),
        context: target_context,
        inputs: operation.inputs.clone(),
        outputs: operation.outputs.clone(),
        authorization: operation.authorization.clone(),
        proof: operation.proof.clone(),
        zk_proof: operation.zk_proof.clone(),
        conservation: operation.conservation.clone(),
        metadata: operation.metadata.clone(),
    })
}

/// Transform a physical operation back to an abstract operation
fn transform_physical_to_abstract<C: ExecutionContext>(
    operation: &Operation<C>,
    target_context: AbstractContext,
) -> std::result::Result<Operation<AbstractContext>, TransformationError> {
    // Create a new operation with the abstract context
    Ok(Operation {
        id: operation.id.clone(),
        op_type: operation.op_type.clone(),
        abstract_representation: operation.abstract_representation.clone(),
        concrete_implementation: operation.concrete_implementation.clone(),
        physical_execution: operation.physical_execution.clone(),
        context: target_context,
        inputs: operation.inputs.clone(),
        outputs: operation.outputs.clone(),
        authorization: operation.authorization.clone(),
        proof: operation.proof.clone(),
        zk_proof: operation.zk_proof.clone(),
        conservation: operation.conservation.clone(),
        metadata: operation.metadata.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::Capability;
    use crate::effect::EmptyEffect;
    use crate::types::ResourceId;
    
    #[test]
    fn test_transform_abstract_to_register() {
        // Create an abstract operation
        let abstract_context = AbstractContext::new(ExecutionPhase::Planning);
        let effect = Box::new(EmptyEffect::new("test_effect"));
        
        let operation = Operation::new(
            OperationType::Create,
            effect,
            abstract_context.clone()
        )
        .with_output(ResourceRef {
            resource_id: ResourceId::from_str("test_resource").unwrap(),
            domain_id: None,
            ref_type: ResourceRefType::Output,
            before_state: None,
            after_state: Some("created".to_string()),
        });
        
        // Transform to register context
        let register_context = RegisterContext::new(
            ExecutionPhase::Execution,
            "test_namespace"
        );
        
        let transformed = transform_abstract_to_register(&operation, register_context)
            .expect("Transformation should succeed");
        
        // Verify the transformation was successful
        assert_eq!(transformed.id, operation.id);
        assert_eq!(transformed.op_type, operation.op_type);
        assert!(transformed.concrete_implementation.is_some());
        assert_eq!(transformed.concrete_implementation.unwrap().operation, RegisterOperationType::Create);
        assert_eq!(transformed.context.phase(), ExecutionPhase::Execution);
        assert_eq!(transformed.context.environment(), ExecutionEnvironment::Register);
    }
} 