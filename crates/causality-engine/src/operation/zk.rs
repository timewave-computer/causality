// Zero-knowledge proof integration for operations
// Original file: src/operation/zk.rs

// Operation ZK Integration Module
//
// This module provides utilities for integrating the unified operation model
// with the ZK proving system, enabling operations to be proven and verified.

use std::collections::HashMap;
use std::sync::Arc;

use causality_types::{Error, Result};
use causality_core::{Proof, ProofRequest, ProofSystem, Circuit};
use crate::verification::UnifiedProof;

use super::{
    Operation, OperationType, ExecutionContext, ExecutionPhase, 
    AbstractContext, ZkContext, ResourceRef, ResourceRefType
};

/// Error during ZK proof generation
#[derive(Debug, thiserror::Error)]
pub enum ZkProofError {
    #[error("Missing required resource: {0}")]
    MissingResource(String),

    #[error("Unsupported operation type for proving: {0:?}")]
    UnsupportedOperationType(OperationType),

    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),

    #[error("Invalid circuit: {0}")]
    InvalidCircuit(String),

    #[error("Missing public input: {0}")]
    MissingPublicInput(String),

    #[error("Missing private input: {0}")]
    MissingPrivateInput(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// ZK proof generator for operations
pub struct OperationProofGenerator {
    proof_system: Arc<dyn ProofSystem>,
    circuit_selector: Arc<dyn CircuitSelector>,
}

impl OperationProofGenerator {
    /// Create a new operation proof generator
    pub fn new(
        proof_system: Arc<dyn ProofSystem>,
        circuit_selector: Arc<dyn CircuitSelector>,
    ) -> Self {
        Self {
            proof_system,
            circuit_selector,
        }
    }
    
    /// Generate a ZK proof for an operation
    pub async fn generate_proof<C: ExecutionContext>(
        &self,
        operation: &Operation<C>,
    ) -> Result<Proof, ZkProofError> {
        // Select the appropriate circuit for this operation
        let circuit = self.circuit_selector.select_circuit(operation)
            .ok_or_else(|| ZkProofError::UnsupportedOperationType(operation.op_type.clone()))?;
        
        // Generate public inputs from the operation
        let public_inputs = self.generate_public_inputs(operation, &circuit)?;
        
        // Generate private inputs from the operation
        let private_inputs = self.generate_private_inputs(operation, &circuit)?;
        
        // Create the proof request
        let proof_request = ProofRequest::new(
            circuit.id().to_string(),
            public_inputs,
            private_inputs,
        );
        
        // Generate the proof
        let proof = self.proof_system.generate_proof(proof_request)
            .await
            .map_err(|e| ZkProofError::ProofGenerationFailed(e.to_string()))?;
        
        Ok(proof)
    }
    
    /// Transform an abstract operation to a ZK operation with proof
    pub async fn transform_to_zk_operation(
        &self,
        operation: &Operation<AbstractContext>,
    ) -> Result<Operation<ZkContext>, ZkProofError> {
        // Create a ZK context
        let zk_context = ZkContext::new(
            ExecutionPhase::Verification,
            "default_circuit"
        );
        
        // Generate the ZK proof
        let zk_proof = self.generate_proof(operation).await?;
        
        // Create a unified proof wrapper
        let unified_proof = UnifiedProof::new(
            "zk",
            HashMap::from([
                ("proof_type".to_string(), "zk".to_string()),
                ("circuit".to_string(), zk_context.circuit_id.clone()),
            ]),
        );
        
        // Create the ZK operation
        let zk_operation = Operation {
            id: operation.id.clone(),
            op_type: operation.op_type.clone(),
            abstract_representation: operation.abstract_representation.clone(),
            concrete_implementation: operation.concrete_implementation.clone(),
            physical_execution: operation.physical_execution.clone(),
            context: zk_context,
            inputs: operation.inputs.clone(),
            outputs: operation.outputs.clone(),
            authorization: operation.authorization.clone(),
            proof: Some(unified_proof),
            zk_proof: Some(zk_proof),
            conservation: operation.conservation.clone(),
            metadata: operation.metadata.clone(),
        };
        
        Ok(zk_operation)
    }
    
    /// Generate public inputs for an operation proof
    fn generate_public_inputs<C: ExecutionContext>(
        &self,
        operation: &Operation<C>,
        circuit: &dyn Circuit,
    ) -> Result<HashMap<String, String>, ZkProofError> {
        let mut public_inputs = HashMap::new();
        
        // Add operation ID
        public_inputs.insert("operation_id".to_string(), operation.id.to_string());
        
        // Add operation type
        public_inputs.insert("operation_type".to_string(), format!("{:?}", operation.op_type));
        
        // Add resource IDs for inputs
        for (i, input) in operation.inputs.iter().enumerate() {
            public_inputs.insert(
                format!("input_{}_id", i),
                input.resource_id.to_string(),
            );
        }
        
        // Add resource IDs for outputs
        for (i, output) in operation.outputs.iter().enumerate() {
            public_inputs.insert(
                format!("output_{}_id", i),
                output.resource_id.to_string(),
            );
        }
        
        // Validate that we have the required public inputs for this circuit
        for required_input in circuit.required_public_inputs() {
            if !public_inputs.contains_key(&required_input) {
                return Err(ZkProofError::MissingPublicInput(required_input));
            }
        }
        
        Ok(public_inputs)
    }
    
    /// Generate private inputs for an operation proof
    fn generate_private_inputs<C: ExecutionContext>(
        &self,
        operation: &Operation<C>,
        circuit: &dyn Circuit,
    ) -> Result<HashMap<String, String>, ZkProofError> {
        let mut private_inputs = HashMap::new();
        
        // Add resource state information for inputs
        for (i, input) in operation.inputs.iter().enumerate() {
            if let Some(state) = &input.before_state {
                private_inputs.insert(
                    format!("input_{}_state", i),
                    state.clone(),
                );
            }
        }
        
        // Add resource state information for outputs
        for (i, output) in operation.outputs.iter().enumerate() {
            if let Some(state) = &output.after_state {
                private_inputs.insert(
                    format!("output_{}_state", i),
                    state.clone(),
                );
            }
        }
        
        // Add authorization data
        private_inputs.insert(
            "authorization_type".to_string(),
            format!("{:?}", operation.authorization.auth_type),
        );
        
        private_inputs.insert(
            "authorization_data".to_string(),
            hex::encode(&operation.authorization.data),
        );
        
        // Add additional metadata
        for (key, value) in &operation.metadata {
            private_inputs.insert(
                format!("metadata_{}", key),
                value.clone(),
            );
        }
        
        // Validate that we have the required private inputs for this circuit
        for required_input in circuit.required_private_inputs() {
            if !private_inputs.contains_key(&required_input) {
                return Err(ZkProofError::MissingPrivateInput(required_input));
            }
        }
        
        Ok(private_inputs)
    }
}

/// Trait for selecting the appropriate circuit for an operation
pub trait CircuitSelector: Send + Sync {
    /// Select a circuit for the given operation
    fn select_circuit<C: ExecutionContext>(&self, operation: &Operation<C>) -> Option<Arc<dyn Circuit>>;
}

/// Default circuit selector that selects circuits based on operation type
pub struct DefaultCircuitSelector {
    circuits: HashMap<OperationType, Arc<dyn Circuit>>,
}

impl DefaultCircuitSelector {
    /// Create a new default circuit selector
    pub fn new() -> Self {
        Self {
            circuits: HashMap::new(),
        }
    }
    
    /// Register a circuit for an operation type
    pub fn register_circuit(&mut self, op_type: OperationType, circuit: Arc<dyn Circuit>) -> &mut Self {
        self.circuits.insert(op_type, circuit);
        self
    }
}

impl CircuitSelector for DefaultCircuitSelector {
    fn select_circuit<C: ExecutionContext>(&self, operation: &Operation<C>) -> Option<Arc<dyn Circuit>> {
        self.circuits.get(&operation.op_type).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::EmptyEffect;
    use causality_crypto::ContentId;
    use causality_core::{Circuit, ProofRequest, ProofSystem};
    use async_trait::async_trait;
    
    #[tokio::test]
    async fn test_proof_generation() {
        // Create test operation
        let context = AbstractContext::new(ExecutionPhase::Planning);
        let effect = Box::new(EmptyEffect::new("test_effect"));
        
        let operation = Operation::new(
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
        });
        
        // Create proof generator
        let proof_system = Arc::new(MockProofSystem {});
        let circuit_selector = Arc::new(MockCircuitSelector {});
        let proof_generator = OperationProofGenerator::new(proof_system, circuit_selector);
        
        // Generate proof
        let proof = proof_generator.generate_proof(&operation).await.unwrap();
        
        // Verify the proof was generated
        assert_eq!(proof.circuit_id(), "mock_circuit");
    }
    
    #[tokio::test]
    async fn test_transform_to_zk_operation() {
        // Create test operation
        let context = AbstractContext::new(ExecutionPhase::Planning);
        let effect = Box::new(EmptyEffect::new("test_effect"));
        
        let operation = Operation::new(
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
        });
        
        // Create proof generator
        let proof_system = Arc::new(MockProofSystem {});
        let circuit_selector = Arc::new(MockCircuitSelector {});
        let proof_generator = OperationProofGenerator::new(proof_system, circuit_selector);
        
        // Transform to ZK operation
        let zk_operation = proof_generator.transform_to_zk_operation(&operation).await.unwrap();
        
        // Verify the transformation
        assert_eq!(zk_operation.id, operation.id);
        assert!(zk_operation.proof.is_some());
        assert!(zk_operation.zk_proof.is_some());
        assert_eq!(zk_operation.context.environment(), super::super::ExecutionEnvironment::ZkVm);
    }
    
    // Mock implementations for testing
    
    struct MockCircuit {}
    
    impl Circuit for MockCircuit {
        fn id(&self) -> &str {
            "mock_circuit"
        }
        
        fn required_public_inputs(&self) -> Vec<String> {
            vec!["operation_id".to_string(), "operation_type".to_string()]
        }
        
        fn required_private_inputs(&self) -> Vec<String> {
            Vec::new()
        }
    }
    
    struct MockProofSystem {}
    
    #[async_trait]
    impl ProofSystem for MockProofSystem {
        async fn generate_proof(&self, _request: ProofRequest) -> Result<Proof, Error> {
            Ok(Proof::new("mock_circuit", vec![1, 2, 3, 4]))
        }
        
        async fn verify_proof(&self, _proof: &Proof, _public_inputs: &HashMap<String, String>) -> Result<bool, Error> {
            Ok(true)
        }
    }
    
    struct MockCircuitSelector {}
    
    impl CircuitSelector for MockCircuitSelector {
        fn select_circuit<C: ExecutionContext>(&self, _operation: &Operation<C>) -> Option<Arc<dyn Circuit>> {
            Some(Arc::new(MockCircuit {}))
        }
    }
} 
