// Verification examples and test cases
// Original file: src/verification/examples.rs

// Verification Examples
//
// This module provides example implementations of the Verifiable trait
// for common entities in the Causality system.

use std::collections::HashMap;
use std::sync::Arc;

use borsh::{BorshSerialize, BorshDeserialize};
use chrono::Utc;
use crate::crypto::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};
use causality_types::{*};
use causality_crypto::ContentId;;
use crate::verification::{
    Verifiable,
    VerificationContext,
    VerificationError, 
    VerificationResult,
    VerificationOptions,
    VerificationDependency,
    VerificationCapability,
    UnifiedProof,
    ZkProofData,
    TemporalProofData,
    TemporalAttestation,
    DomainContext,
};

/// A simple resource operation that can be verified
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ResourceOperation {
    /// Operation ID
    pub id: String,
    
    /// Resource ID
    pub resource_id: ContentId,
    
    /// Domain ID
    pub domain_id: DomainId,
    
    /// Operation type
    pub operation_type: String,
    
    /// Operation parameters
    pub parameters: HashMap<String, String>,
    
    /// Operation timestamp
    pub timestamp: Timestamp,
    
    /// Controller label
    pub controller: String,
    
    /// Previous state hash (optional)
    pub previous_state: Option<String>,
    
    /// New state hash
    pub new_state: String,
}

// Implement ContentAddressed for ResourceOperation
impl ContentAddressed for ResourceOperation {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl ResourceOperation {
    /// Create a new resource operation
    pub fn new(
        resource_id: ContentId,
        domain_id: DomainId,
        operation_type: &str,
        controller: &str,
        previous_state: Option<&str>,
        new_state: &str,
    ) -> Self {
        let op = Self {
            id: String::new(), // Temporary value
            resource_id,
            domain_id,
            operation_type: operation_type.to_string(),
            parameters: HashMap::new(),
            timestamp: Timestamp::now(),
            controller: controller.to_string(),
            previous_state: previous_state.map(String::from),
            new_state: new_state.to_string(),
        };
        
        // Set the content-derived ID
        let mut result = op;
        result.id = format!("op:{}", result.content_id());
        result
    }
    
    /// Add a parameter to the operation
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Generate a test ZK proof for this operation
    fn generate_zk_proof(&self, context: &VerificationContext) -> Result<ZkProofData, VerificationError> {
        // In a real implementation, this would use the ZK prover to generate a proof
        // For this example, we'll create a dummy proof
        
        // Check if we have the required capability
        if !context.has_capability(&VerificationCapability::ZkProving) {
            return Err(VerificationError::MissingCapability("ZkProving".to_string()));
        }
        
        // Get the domain context
        let domain_context = context.get_domain_context(&self.domain_id)
            .ok_or_else(|| VerificationError::MissingContext(format!("Domain context for {}", self.domain_id)))?;
        
        // In a real implementation, we would use the domain parameters for the circuit
        let _domain_params = domain_context.parameters();
        
        // Generate a dummy proof
        Ok(ZkProofData {
            system: "groth16".to_string(),
            proof: vec![1, 2, 3, 4], // Dummy proof data
            public_inputs: vec![
                self.resource_id.to_string().as_bytes().to_vec(),
                self.new_state.as_bytes().to_vec(),
            ],
            verification_key_id: format!("vk-{}", self.domain_id),
            created_at: Utc::now(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("operation_type".to_string(), self.operation_type.clone());
                map
            },
        })
    }
    
    /// Generate a test temporal proof for this operation
    fn generate_temporal_proof(&self, context: &VerificationContext) -> Result<TemporalProofData, VerificationError> {
        // In a real implementation, this would use the time map to generate a proof
        // For this example, we'll create a dummy proof
        
        // Check if we have the required capability
        if !context.has_capability(&VerificationCapability::TimeMapAccess) {
            return Err(VerificationError::MissingCapability("TimeMapAccess".to_string()));
        }
        
        // Generate a dummy temporal proof
        Ok(TemporalProofData {
            domain_id: self.domain_id.clone(),
            block_height: 12345,
            block_hash: "0123456789abcdef0123456789abcdef".to_string(),
            timestamp: self.timestamp.clone(),
            attestations: vec![
                TemporalAttestation {
                    validator_id: "validator-1".to_string(),
                    signature: vec![1, 2, 3, 4],
                    public_key: vec![5, 6, 7, 8],
                    timestamp: self.timestamp.clone(),
                },
            ],
            metadata: HashMap::new(),
        })
    }
}

impl Verifiable for ResourceOperation {
    fn generate_proof(&self, context: &VerificationContext) -> Result<UnifiedProof, VerificationError> {
        // Create a unified proof with multiple components
        let mut proof = UnifiedProof::new(format!("proof-{}", self.id));
        
        // Add ZK proof if the context has ZK proving capability
        if context.has_capability(&VerificationCapability::ZkProving) {
            let zk_proof = self.generate_zk_proof(context)?;
            proof = proof.with_zk_components(zk_proof);
        }
        
        // Add temporal proof if the context has time map access
        if context.has_capability(&VerificationCapability::TimeMapAccess) {
            let temporal_proof = self.generate_temporal_proof(context)?;
            proof = proof.with_temporal_components(temporal_proof);
        }
        
        // Add metadata
        proof = proof
            .with_metadata("resource_id", &self.resource_id.to_string())
            .with_metadata("domain_id", &self.domain_id.to_string())
            .with_metadata("operation_type", &self.operation_type)
            .with_metadata("controller", &self.controller);
        
        Ok(proof)
    }
    
    fn verify(&self, context: &VerificationContext, options: &VerificationOptions) -> Result<VerificationResult, VerificationError> {
        // Generate the proof
        let proof = self.generate_proof(context)?;
        
        // In a real implementation, we would verify the proof using the appropriate verifiers
        // For this example, we'll simulate a successful verification
        
        Ok(VerificationResult {
            success: true,
            verification_time: Utc::now(),
            error_message: None,
            confidence: 1.0,
            provider_name: "example_provider".to_string(),
            verification_type: crate::verification::VerificationType::ZkProof,
            metadata: HashMap::new(),
        })
    }
    
    fn get_dependencies(&self) -> Vec<VerificationDependency> {
        // In a real implementation, this operation might depend on other resources
        // For this example, we'll return a simple dependency
        vec![
            VerificationDependency {
                resource_ids: vec![self.resource_id.clone()],
                domain_ids: vec![self.domain_id.clone()],
                verification_types: vec![crate::verification::VerificationType::ZkProof],
                metadata: HashMap::new(),
            }
        ]
    }
    
    fn required_capabilities(&self) -> Vec<VerificationCapability> {
        vec![
            VerificationCapability::ZkProving,
            VerificationCapability::TimeMapAccess,
        ]
    }
}

/// A simple fact assertion that can be verified
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct FactAssertion {
    /// Assertion ID
    pub id: String,
    
    /// Domain ID
    pub domain_id: DomainId,
    
    /// Statement being asserted
    pub statement: String,
    
    /// Controller making the assertion
    pub controller: String,
    
    /// Assertion timestamp
    pub timestamp: Timestamp,
    
    /// Related resources
    pub resources: Vec<ContentId>,
    
    /// Evidence hash
    pub evidence_hash: String,
}

// Implement ContentAddressed for FactAssertion
impl ContentAddressed for FactAssertion {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl FactAssertion {
    /// Create a new fact assertion
    pub fn new(
        domain_id: DomainId,
        statement: &str,
        controller: &str,
        evidence_hash: &str,
    ) -> Self {
        let assertion = Self {
            id: String::new(), // Temporary value
            domain_id,
            statement: statement.to_string(),
            controller: controller.to_string(),
            timestamp: Timestamp::now(),
            resources: Vec::new(),
            evidence_hash: evidence_hash.to_string(),
        };
        
        // Set the content-derived ID
        let mut result = assertion;
        result.id = format!("fact:{}", result.content_id());
        result
    }
    
    /// Add a related resource
    pub fn with_resource(mut self, resource_id: ContentId) -> Self {
        self.resources.push(resource_id);
        self
    }
}

impl Verifiable for FactAssertion {
    fn generate_proof(&self, context: &VerificationContext) -> Result<UnifiedProof, VerificationError> {
        // Create a unified proof
        let proof = UnifiedProof::new(format!("proof-{}", self.id))
            .with_temporal_components(TemporalProofData {
                domain_id: self.domain_id.clone(),
                block_height: 12345,
                block_hash: "0123456789abcdef0123456789abcdef".to_string(),
                timestamp: self.timestamp.clone(),
                attestations: vec![
                    TemporalAttestation {
                        validator_id: "validator-1".to_string(),
                        signature: vec![1, 2, 3, 4],
                        public_key: vec![5, 6, 7, 8],
                        timestamp: self.timestamp.clone(),
                    },
                ],
                metadata: HashMap::new(),
            })
            .with_metadata("statement", &self.statement)
            .with_metadata("controller", &self.controller)
            .with_metadata("evidence_hash", &self.evidence_hash);
        
        Ok(proof)
    }
    
    fn verify(&self, context: &VerificationContext, _options: &VerificationOptions) -> Result<VerificationResult, VerificationError> {
        // In a real implementation, we would verify the assertion
        // For this example, we'll simulate a successful verification
        
        Ok(VerificationResult {
            success: true,
            verification_time: Utc::now(),
            error_message: None,
            confidence: 0.9,
            provider_name: "example_provider".to_string(),
            verification_type: crate::verification::VerificationType::Temporal,
            metadata: HashMap::new(),
        })
    }
    
    fn get_dependencies(&self) -> Vec<VerificationDependency> {
        vec![
            VerificationDependency {
                resource_ids: self.resources.clone(),
                domain_ids: vec![self.domain_id.clone()],
                verification_types: vec![crate::verification::VerificationType::Temporal],
                metadata: HashMap::new(),
            }
        ]
    }
    
    fn required_capabilities(&self) -> Vec<VerificationCapability> {
        vec![
            VerificationCapability::TimeMapAccess,
            VerificationCapability::ControllerRegistryAccess,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification::VerificationStatus;
    use causality_core::{
        VerificationService,
        VerificationOptions as ServiceOptions,
        VerificationMode,
    };
    
    #[tokio::test]
    async fn test_resource_operation_verification() {
        // Create a resource operation
        let resource_id = ContentId::from("test-resource-123");
        let domain_id = DomainId::new("test-domain");
        
        let operation = ResourceOperation::new(
            resource_id,
            domain_id.clone(),
            "update",
            "controller-123",
            Some("previous-state-hash"),
            "new-state-hash",
        )
        .with_parameter("key1", "value1")
        .with_parameter("key2", "value2");
        
        // Create a verification context with all required capabilities
        let mut context = VerificationContext::new();
        
        context.add_capability(VerificationCapability::ZkProving);
        context.add_capability(VerificationCapability::TimeMapAccess);
        
        // Create a domain context
        let domain_context = DomainContext::new(domain_id.clone(), Timestamp::now())
            .with_parameter("circuit", "update_circuit");
        context.add_domain_context(domain_context);
        
        // Generate a proof
        let proof = operation.generate_proof(&context).unwrap();
        
        // Verify the proof has the expected components
        assert!(proof.zk_components.is_some());
        assert!(proof.temporal_components.is_some());
        
        // Verify metadata
        assert_eq!(proof.metadata.get("resource_id"), Some(&resource_id.to_string()));
        assert_eq!(proof.metadata.get("operation_type"), Some(&"update".to_string()));
        
        // Create a verification service and verify using it
        let service = VerificationService::default();
        service.set_default_context(context);
        
        // Create a mock provider for testing
        struct MockProvider;
        
        #[async_trait::async_trait]
        impl crate::verification::VerificationProvider for MockProvider {
            fn name(&self) -> &str {
                "mock_provider"
            }
            
            fn supported_verification_types(&self) -> Vec<crate::verification::VerificationType> {
                vec![
                    crate::verification::VerificationType::ZkProof,
                    crate::verification::VerificationType::Temporal,
                ]
            }
            
            fn capabilities(&self) -> Vec<crate::verification::ProviderCapability> {
                vec![]
            }
            
            fn can_verify(&self, proof: &UnifiedProof) -> bool {
                true
            }
            
            async fn verify(
                &self,
                proof: &UnifiedProof,
                context: &VerificationContext,
            ) -> Result<VerificationResult, VerificationError> {
                // Mock implementation that always succeeds
                Ok(VerificationResult {
                    success: true,
                    verification_time: Utc::now(),
                    error_message: None,
                    confidence: 1.0,
                    provider_name: "mock_provider".to_string(),
                    verification_type: crate::verification::VerificationType::ZkProof,
                    metadata: HashMap::new(),
                })
            }
        }
        
        service.register_provider(Arc::new(MockProvider));
        
        // Verify
        let options = ServiceOptions {
            mode: VerificationMode::All,
            ..Default::default()
        };
        
        let result = service.verify(&operation, &options, None).await.unwrap();
        
        // Check result
        assert_eq!(result.status, VerificationStatus::Verified);
        assert!(result.any_successful());
    }
} 
