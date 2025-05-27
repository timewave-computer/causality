// Operation transformation
// Original file: src/operation/transformation.rs

// Operation Transformation Module
//
// This module provides utilities for transforming operations from one
// representation to another, e.g., from abstract to register operations.

use std::collections::HashMap;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

use causality_types::DomainId;
use causality_error::EngineError;

// Import context-related items directly from context module
use super::context::{
    ExecutionContext, ExecutionEnvironment,
    AbstractContext, PhysicalContext, ZkContext
};

// Import ResourceRegisterTrait
use crate::operation::api::ResourceRegisterTrait;

// Import items from causality_core
use causality_core::{
    resource::{Operation, OperationType, ResourceId}, // Import EffectInfo and function
    serialization::SerializationError // Import SerializationError
};

// Engine-specific types
use crate::log::fact_types::RegisterOperationType; // Import engine-specific type

// Import the Context enum wrapper from local types module


// Define local structs needed for transformation output (if not imported)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterOperation {
    pub register_id: ResourceId,
    pub operation: RegisterOperationType,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalOperation {
    pub domain_id: DomainId,
    pub tx_hash: Option<String>,
    pub block_height: Option<u64>,
    pub data: Vec<u8>, // Assuming serialized data
}

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

    #[error("Operation ID error: {0}")]
    IdError(String),

    #[error("Serialization error during transformation: {0}")]
    Serialization(String),
}

// Implement From<SerializationError> for TransformationError
impl From<SerializationError> for TransformationError {
    fn from(err: SerializationError) -> Self {
        TransformationError::Serialization(err.to_string())
    }
}

/// Transform an operation for a different execution context
/// This now focuses on generating necessary data/side-effects for the target context
/// rather than creating a distinct Operation<D> struct.
pub fn transform_operation_for_context(
    _operation: &Operation, 
    target_environment: &ExecutionEnvironment
) -> std::result::Result<(), TransformationError> { // Return Ok or Error, side effects handled internally or via return data (TBD)
    
    // Placeholder: Validation and dispatch based on source/target env
    // let source_environment = determine_source_environment(operation)?; // Need logic to get current env
    // validate_transformation_path(&source_environment, target_environment)?;
    // match (source_environment, target_environment) { ... }

    println!("Placeholder: transform_operation_for_context called for {:?}", target_environment);
    // TODO: Implement actual transformation logic based on environments
    Ok(())
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

/// Generate data needed for a register operation from a core Operation
/// Returns RegisterOperation data (e.g., as a struct or HashMap)
pub fn generate_register_data_from_operation(
    operation: &Operation,
) -> std::result::Result<RegisterOperation, TransformationError> { 
    // Placeholder logic - reconstruct RegisterOperation from core Operation fields
    create_register_operation_from_core(operation)
}

/// Helper to create RegisterOperation data from a core Operation
fn create_register_operation_from_core(
    operation: &Operation,
) -> std::result::Result<RegisterOperation, TransformationError> {
    let register_id = operation.target.clone();
    let register_op_type = match operation.operation_type {
        OperationType::Create => RegisterOperationType::Create,
        OperationType::Update => RegisterOperationType::Update,
        OperationType::Delete => RegisterOperationType::Delete, // Correct mapping
        OperationType::Custom(ref name) => {
            // TODO: Define a proper mapping from custom core OperationTypes 
            //       to RegisterOperationType variants or handle differently.
            match name.as_str() {
                "Lock" => RegisterOperationType::Update, // Placeholder: No Lock variant
                _ => RegisterOperationType::Update, // Placeholder: No general Custom variant
            }
        },
        // Handle other OperationType variants if necessary, e.g., Read?
        // If OperationType::Read exists and should map:
        // OperationType::Read => RegisterOperationType::Read, 
        _ => return Err(TransformationError::InvalidOperationType(format!("Unhandled core operation type for register fact: {:?}", operation.operation_type)))
    };
    let mut data = HashMap::new();
    data.insert("operation_id".to_string(), operation.id()?.to_string());
    data.insert("operation_type".to_string(), format!("{:?}", operation.operation_type));
    data.extend(operation.parameters.clone());
    data.extend(operation.metadata.clone());
    
    Ok(RegisterOperation {
        register_id,
        operation: register_op_type,
        data,
    })
}

/// Generate data needed for a physical operation from a core Operation
/// Returns PhysicalOperation data (e.g., as a struct or HashMap)
fn generate_physical_data_from_operation(
    operation: &Operation, // Core Operation
    target_context: &PhysicalContext, // Need context info
) -> std::result::Result<PhysicalOperation, TransformationError> { 
    let register_id_str = operation.parameters.get("register_id")
        .ok_or(TransformationError::MissingImplementation)?;
    // Use ResourceId::from_string instead of from_str
    let register_id = ResourceId::from_string(register_id_str) 
        .map_err(|e| TransformationError::ResourceError(format!("Invalid register_id parameter: {}", e)))?;
    
    // Placeholder: Reconstruct RegisterOperation - needs proper parsing from params
    // Needs to align with available RegisterOperationType variants
    let register_op = RegisterOperation {
         register_id: register_id.clone(),
         operation: RegisterOperationType::Update, // Placeholder: Use an existing variant like Update
         data: operation.parameters.clone(), 
    };

    // Create physical operation data struct
    Ok(PhysicalOperation {
        domain_id: target_context.domain_id.clone(),
        tx_hash: None, 
        block_height: None,
        data: serde_json::to_vec(&register_op)
            .map_err(|e| TransformationError::ResourceError(e.to_string()))?,
    })
}

/// Generate data needed for a ZK operation from a core Operation
/// Returns ZkOperation data (e.g., proof request info or proof CID)
fn generate_zk_data_from_operation(
    operation: &Operation, // Core operation
    target_context: &ZkContext,
) -> std::result::Result<(), TransformationError> { 
    // Call requires_proof using explicit trait path
    if ExecutionContext::proof_required(target_context) { 
        if operation.metadata.get("proof_cid").is_none() { 
             return Err(TransformationError::MissingProof);
        }
    }
    Ok(())
}

/// Generate abstract representation data from a core Operation
// This might involve extracting key info from parameters/metadata if transformed previously
fn generate_abstract_data_from_operation(
    operation: &Operation,
    _target_context: &AbstractContext, 
) -> std::result::Result<(), TransformationError> {
    // Placeholder: Logic to potentially clean up metadata/parameters from previous transformations.
    println!("Generating abstract data for operation: {}", operation.id()?); // Added ? to propagate potential ID error
    Ok(())
}

/// Define TransformedOutput type for the transformation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedOutput {
    pub id: String,
    pub data: HashMap<String, String>,
}

// Define RegisterData type for operation outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterData {
    pub register_id: ResourceId,
    pub data: HashMap<String, String>,
}

/// Generate the output transformation for a given operation
/// This transforms the output from the operation
fn generate_output_transformation(
    _resources: &dyn ResourceRegisterTrait,
    _operation: &Operation, 
    output: &RegisterData,
) -> Result<TransformedOutput, EngineError> {
    // For now, simply construct a TransformedOutput instance with minimal data
    let mut output_data = output.data.clone();
    output_data.insert("transformed".to_string(), "true".to_string());
    
    Ok(TransformedOutput {
        id: output.register_id.to_string(),
        data: output_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // use causality_patterns::Capability; // Import might be needed if capability parsing is added
    use crate::effect::EmptyEffect;
    use std::str::FromStr;
    use crate::operation::AbstractContext;
    use crate::operation::ExecutionPhase;
    use crate::effect::capability::IdentityId;
    // ResourceId is imported correctly now via causality_core::resource
    
    #[test]
    fn test_generate_register_data() { // Renamed test
        // Create an abstract context (used separately now)
        let _abstract_context = AbstractContext::new(ExecutionPhase::Planning);
        
        // Create a core Operation
        let identity = IdentityId::new();
        let target = ResourceId::from_str("test:resource:1").unwrap(); 
        let effect = Box::new(EmptyEffect::new("test_effect"));
        let operation = Operation::new(
            identity.clone(),
            OperationType::Create,
            target.clone(),
            vec![effect]
        );
        
        // Create target register context (passed separately if needed by generation fn)
        let _register_context = RegisterContext::new(
            ExecutionPhase::Execution,
            "test_namespace"
        );
        
        let register_data = generate_register_data_from_operation(&operation)
            .expect("Generation should succeed");
        
        // Verify the generated RegisterOperation data
        assert_eq!(register_data.register_id, operation.target);
        assert_eq!(register_data.operation, RegisterOperationType::Create);
        assert!(register_data.data.contains_key("operation_id"));
    }
} 
