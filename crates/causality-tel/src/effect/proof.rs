// TEL effect proof integration
// Original file: src/tel/effect/proof.rs

// Proof generation and verification for TEL effects
//
// This module provides functionality for generating and verifying
// proofs for resource effects, enabling zero-knowledge verification
// of effect execution.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use std::io::{Read, Write};
use crypto;

use crate::tel::{
    error::{TelError, TelResult},
    types::{Proof, ResourceId, Address, OperationId, Timestamp},
    resource::{
        ResourceOperation,
        ResourceOperationType,
        RegisterContents,
    },
    effect::ResourceEffect,
};
use causality_crypto::ContentId;
use :ResourceRegister:causality_core::resource::Resource::{ResourceRegister, RegisterState};
use crate::operation::{Operation, RegisterOperationType};

/// Format for effect proofs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectProofFormat {
    /// Groth16 zero-knowledge proof format
    Groth16,
    /// Plonk zero-knowledge proof format
    Plonk,
    /// Stark zero-knowledge proof format
    Stark,
    /// BulletProofs zero-knowledge proof format
    BulletProofs,
    /// Custom format (application specific)
    Custom(u32),
}

impl Default for EffectProofFormat {
    fn default() -> Self {
        Self::Groth16
    }
}

/// Metadata for effect proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectProofMetadata {
    /// Resource ID this proof is associated with (legacy format)
    pub resource_id: Option<ResourceId>,
    /// ResourceRegister ID (unified format)
    pub register_id: Option<ContentId>,
    /// Creator of the proof
    pub creator: Address,
    /// Timestamp when the proof was created
    pub created_at: Timestamp,
    /// Format of the proof
    pub format: EffectProofFormat,
    /// Optional auxiliary data
    pub aux_data: Option<Vec<u8>>,
    /// Whether this is for a unified ResourceRegister 
    pub is_resource_register: bool,
}

impl EffectProofMetadata {
    /// Create new metadata for an effect proof
    pub fn new(
        resource_id: Option<ResourceId>,
        creator: Address,
        format: EffectProofFormat,
    ) -> Self {
        Self {
            resource_id,
            register_id: None,
            creator,
            created_at: Timestamp::now(),
            format,
            aux_data: None,
            is_resource_register: false,
        }
    }

    /// Create new metadata for a ResourceRegister effect proof
    pub fn new_for_register(
        register_id: ContentId,
        creator: Address,
        format: EffectProofFormat,
    ) -> Self {
        Self {
            resource_id: None,
            register_id: Some(register_id),
            creator,
            created_at: Timestamp::now(),
            format,
            aux_data: None,
            is_resource_register: true,
        }
    }

    /// Add auxiliary data to the metadata
    pub fn with_aux_data(mut self, data: Vec<u8>) -> Self {
        self.aux_data = Some(data);
        self
    }
}

/// Generator for effect proofs
pub struct EffectProofGenerator {
    /// Default proof format
    default_format: EffectProofFormat,
    /// Default creator address
    default_creator: Address,
}

impl EffectProofGenerator {
    /// Create a new proof generator
    pub fn new(default_format: EffectProofFormat, default_creator: Address) -> Self {
        Self {
            default_format,
            default_creator,
        }
    }

    /// Generate a proof for a resource effect
    pub fn generate_proof(
        &self,
        effect: &ResourceEffect,
        metadata: Option<EffectProofMetadata>,
    ) -> TelResult<Proof> {
        // Extract resource ID if available from the operation
        let resource_id = match &effect.operation.op_type {
            ResourceOperationType::Create { .. } => None,
            ResourceOperationType::Update { resource_id, .. } => Some(*resource_id),
            ResourceOperationType::Delete { resource_id } => Some(*resource_id),
            ResourceOperationType::Transfer { resource_id, .. } => Some(*resource_id),
            ResourceOperationType::Lock { resource_id } => Some(*resource_id),
            ResourceOperationType::Unlock { resource_id } => Some(*resource_id),
            ResourceOperationType::Custom { resource_id, .. } => *resource_id,
        };

        // Use provided metadata or create default
        let metadata = metadata.unwrap_or_else(|| EffectProofMetadata::new(
            resource_id,
            self.default_creator.clone(),
            self.default_format,
        ));

        // Serialize the effect and metadata for the proof
        let serialized_effect = self.serialize_effect(effect)?;
        let serialized_metadata = self.serialize_metadata(&metadata)?;

        // Combine the data and create a proof
        let mut proof_data = Vec::new();
        proof_data.extend_from_slice(&serialized_effect);
        proof_data.extend_from_slice(&serialized_metadata);

        // In a real implementation, this would call into a cryptographic library
        // to generate an actual zero-knowledge proof. For this implementation,
        // we'll use a content-derived ID based on the hash of the proof data.
        let hasher = crypto::hash::HashFactory::default().create_hasher().unwrap();
        let content_id = crypto::hash::ContentId::from(hasher.hash(&proof_data));
        let proof_id = content_id.to_string();
        
        Ok(Proof::new(
            &proof_id,
            proof_data,
        ))
    }

    /// Generate a proof for a ResourceRegister operation
    pub fn generate_register_proof(
        &self,
        register: &ResourceRegister,
        operation_type: RegisterOperationType,
        metadata: Option<EffectProofMetadata>,
    ) -> TelResult<Proof> {
        // Use provided metadata or create default
        let metadata = metadata.unwrap_or_else(|| EffectProofMetadata::new_for_register(
            register.id.clone(),
            self.default_creator.clone(),
            self.default_format,
        ));

        // Create a simplified Operation for serialization
        let operation = Operation {
            id: OperationId::new(),
            operation_type: crate::operation::OperationType::Register(operation_type),
            timestamp: Timestamp::now(),
            metadata: HashMap::new(),
        };

        // Serialize the register, operation, and metadata
        let serialized_register = self.serialize_register(register)?;
        let serialized_operation = serde_json::to_vec(&operation)
            .map_err(|e| TelError::SerializationError(
                format!("Failed to serialize operation: {}", e)
            ))?;
        let serialized_metadata = self.serialize_metadata(&metadata)?;

        // Combine the data for the proof
        let mut proof_data = Vec::new();
        proof_data.extend_from_slice(&serialized_register);
        proof_data.extend_from_slice(&serialized_operation);
        proof_data.extend_from_slice(&serialized_metadata);

        // In a real implementation, this would generate an actual ZK proof
        // based on the data. For now, we just create a content-derived ID.
        let hasher = crypto::hash::HashFactory::default().create_hasher().unwrap();
        let content_id = crypto::hash::ContentId::from(hasher.hash(&proof_data));
        let proof_id = content_id.to_string();
        
        Ok(Proof::new(
            &proof_id,
            proof_data,
        ))
    }

    /// Serialize a ResourceRegister for proof generation
    fn serialize_register(&self, register: &ResourceRegister) -> TelResult<Vec<u8>> {
        match serde_json::to_vec(register) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(TelError::SerializationError(format!(
                "Failed to serialize ResourceRegister: {}", e
            ))),
        }
    }

    /// Serialize an effect for proof generation
    fn serialize_effect(&self, effect: &ResourceEffect) -> TelResult<Vec<u8>> {
        match serde_json::to_vec(effect) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(TelError::SerializationError(format!(
                "Failed to serialize effect: {}", e
            ))),
        }
    }

    /// Serialize metadata for proof generation
    fn serialize_metadata(&self, metadata: &EffectProofMetadata) -> TelResult<Vec<u8>> {
        match serde_json::to_vec(metadata) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(TelError::SerializationError(format!(
                "Failed to serialize metadata: {}", e
            ))),
        }
    }
}

/// Effect proof verifier
pub struct EffectProofVerifier {
    /// Supported proof formats
    supported_formats: Vec<EffectProofFormat>,
}

impl EffectProofVerifier {
    /// Create a new proof verifier
    pub fn new(supported_formats: Vec<EffectProofFormat>) -> Self {
        Self {
            supported_formats,
        }
    }

    /// Create a verifier with default supported formats
    pub fn default() -> Self {
        Self {
            supported_formats: vec![
                EffectProofFormat::Groth16,
                EffectProofFormat::Plonk,
            ],
        }
    }

    /// Verify a proof for a resource effect
    pub fn verify_proof(
        &self,
        effect: &ResourceEffect,
        proof: &Proof,
    ) -> TelResult<bool> {
        // Extract the metadata from the proof
        let metadata = self.extract_metadata(proof)?;

        // Check if the format is supported
        if !self.supported_formats.contains(&metadata.format) {
            return Err(TelError::UnsupportedOperation(format!(
                "Proof format {:?} is not supported by this verifier",
                metadata.format
            )));
        }

        // In a real implementation, this would validate the cryptographic proof
        // against the effect data. For this implementation, we'll use a simple
        // validation that checks if the proof data contains the serialized effect.
        let serialized_effect = self.serialize_effect(effect)?;
        let proof_data = proof.data();

        // Simple verification check: proof data must contain the serialized effect
        Ok(proof_data.windows(serialized_effect.len()).any(|window| {
            window == serialized_effect.as_slice()
        }))
    }

    /// Verify a proof for a ResourceRegister operation
    pub fn verify_register_proof(
        &self,
        register: &ResourceRegister,
        operation_type: RegisterOperationType,
        proof: &Proof,
    ) -> TelResult<bool> {
        // Extract the metadata from the proof
        let metadata = self.extract_metadata(proof)?;

        // Check if the format is supported
        if !self.supported_formats.contains(&metadata.format) {
            return Err(TelError::UnsupportedOperation(format!(
                "Proof format {:?} is not supported by this verifier",
                metadata.format
            )));
        }

        // Verify this is actually a ResourceRegister proof
        if !metadata.is_resource_register {
            return Err(TelError::ValidationError(
                "This is not a ResourceRegister proof".to_string()
            ));
        }

        // Check if the register ID matches
        if let Some(register_id) = &metadata.register_id {
            if register_id != &register.id {
                return Err(TelError::ValidationError(format!(
                    "Register ID mismatch: expected {:?}, got {:?}",
                    register.id, register_id
                )));
            }
        } else {
            return Err(TelError::ValidationError(
                "Proof metadata missing register ID".to_string()
            ));
        }

        // In a real implementation, this would validate the cryptographic proof
        // against the register data and operation. For this implementation, we'll
        // use a simple validation that checks if the proof data contains the
        // serialized register data.
        let proof_generator = EffectProofGenerator::new(
            EffectProofFormat::default(),
            Address::default(),
        );
        let serialized_register = proof_generator.serialize_register(register)?;
        let proof_data = proof.data();

        // Simple verification check: proof data must contain the serialized register
        Ok(proof_data.windows(serialized_register.len()).any(|window| {
            window == serialized_register.as_slice()
        }))
    }

    /// Extract metadata from a proof
    fn extract_metadata(&self, proof: &Proof) -> TelResult<EffectProofMetadata> {
        // In a real implementation, this would properly extract and deserialize
        // the metadata portion of the proof. For this implementation, we'll
        // use a simple approach to find and parse the metadata portion.
        
        // Get the entire proof data
        let proof_data = proof.data();
        
        // Try to deserialize the metadata from the latter half of the proof data
        // This is a very simplified approach and would be more sophisticated in reality
        if proof_data.len() < 10 {
            return Err(TelError::ValidationError(
                "Proof data too short to contain valid metadata".to_string()
            ));
        }
        
        // Simplified: try to parse the metadata from the latter half of the proof
        let midpoint = proof_data.len() / 2;
        match serde_json::from_slice::<EffectProofMetadata>(&proof_data[midpoint..]) {
            Ok(metadata) => Ok(metadata),
            Err(_) => {
                // Try with different offsets if the first attempt fails
                for offset in 1..5 {
                    let pos = if midpoint > offset * 10 { midpoint - offset * 10 } else { 0 };
                    if let Ok(metadata) = serde_json::from_slice::<EffectProofMetadata>(&proof_data[pos..]) {
                        return Ok(metadata);
                    }
                }
                
                Err(TelError::ValidationError(
                    "Failed to extract metadata from proof".to_string()
                ))
            }
        }
    }

    /// Serialize an effect for proof verification
    fn serialize_effect(&self, effect: &ResourceEffect) -> TelResult<Vec<u8>> {
        match serde_json::to_vec(effect) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(TelError::SerializationError(format!(
                "Failed to serialize effect: {}", e
            ))),
        }
    }
}

/// Serialization and deserialization for effect proofs
pub mod serialization {
    use super::*;
    use std::io::{Read, Write};
    
    /// Serialized effect proof format
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SerializedEffectProof {
        /// Unique identifier for the proof
        pub id: String,
        /// Proof metadata
        pub metadata: EffectProofMetadata,
        /// The actual proof data
        pub proof_data: Vec<u8>,
        /// Effect data that this proof is for
        pub effect_data: Option<Vec<u8>>,
    }
    
    impl SerializedEffectProof {
        /// Create a new serialized effect proof
        pub fn new(
            id: String,
            metadata: EffectProofMetadata,
            proof_data: Vec<u8>,
            effect_data: Option<Vec<u8>>,
        ) -> Self {
            Self {
                id,
                metadata,
                proof_data,
                effect_data,
            }
        }
        
        /// Convert to a binary format
        pub fn to_bytes(&self) -> TelResult<Vec<u8>> {
            match serde_json::to_vec(self) {
                Ok(bytes) => Ok(bytes),
                Err(e) => Err(TelError::SerializationError(format!(
                    "Failed to serialize effect proof: {}", e
                ))),
            }
        }
        
        /// Write to a writer
        pub fn write_to<W: Write>(&self, writer: &mut W) -> TelResult<()> {
            let bytes = self.to_bytes()?;
            match writer.write_all(&bytes) {
                Ok(_) => Ok(()),
                Err(e) => Err(TelError::SerializationError(format!(
                    "Failed to write effect proof: {}", e
                ))),
            }
        }
        
        /// Read from bytes
        pub fn from_bytes(bytes: &[u8]) -> TelResult<Self> {
            match serde_json::from_slice(bytes) {
                Ok(proof) => Ok(proof),
                Err(e) => Err(TelError::SerializationError(format!(
                    "Failed to deserialize effect proof: {}", e
                ))),
            }
        }
        
        /// Read from a reader
        pub fn read_from<R: Read>(reader: &mut R) -> TelResult<Self> {
            let mut bytes = Vec::new();
            match reader.read_to_end(&mut bytes) {
                Ok(_) => Self::from_bytes(&bytes),
                Err(e) => Err(TelError::SerializationError(format!(
                    "Failed to read effect proof: {}", e
                ))),
            }
        }
        
        /// Convert to a proof
        pub fn to_proof(&self) -> Proof {
            Proof::new(&self.id, self.proof_data.clone())
        }
        
        /// Create from a proof and metadata
        pub fn from_proof(
            proof: &Proof,
            metadata: EffectProofMetadata,
            effect_data: Option<Vec<u8>>,
        ) -> Self {
            Self::new(
                proof.id().to_string(),
                metadata,
                proof.data().to_vec(),
                effect_data,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_tel::Domain;
    
    #[test]
    fn test_proof_generation() {
        // Create an effect
        let operation = ResourceOperation::new(
            ResourceOperationType::Create {
                owner: Address::random(),
                domain: Domain::new("test"),
                initial_data: RegisterContents::Text("Hello, World!".to_string()),
            },
        );
        
        let effect = ResourceEffect::new(operation);
        
        // Create a proof generator
        let generator = EffectProofGenerator::new(
            EffectProofFormat::Groth16,
            Address::random(),
        );
        
        // Generate a proof
        let proof = generator.generate_proof(&effect, None).unwrap();
        
        // Check the proof
        assert!(!proof.data().is_empty());
    }
    
    #[test]
    fn test_proof_verification() {
        // Create an effect
        let operation = ResourceOperation::new(
            ResourceOperationType::Create {
                owner: Address::random(),
                domain: Domain::new("test"),
                initial_data: RegisterContents::Text("Hello, World!".to_string()),
            },
        );
        
        let effect = ResourceEffect::new(operation);
        
        // Create a proof generator
        let generator = EffectProofGenerator::new(
            EffectProofFormat::Groth16,
            Address::random(),
        );
        
        // Generate and verify a proof
        let proof = generator.generate_proof(&effect, None).unwrap();
        
        // Create a verifier
        let verifier = EffectProofVerifier::default();
        
        // Verify the proof
        let result = verifier.verify_proof(&effect, &proof).unwrap();
        assert!(result);
        
        // Test with unsupported format
        let verifier = EffectProofVerifier::new(
            vec![EffectProofFormat::Plonk],
        );
        
        // This should fail because Groth16 isn't supported
        let result = verifier.verify_proof(&effect, &proof);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_proof_serialization() {
        use super::serialization::SerializedEffectProof;
        
        let generator = EffectProofGenerator::new(
            EffectProofFormat::Groth16,
            Address::random(),
        );
        
        let operation = ResourceOperation::new(
            ResourceOperationType::Create {
                owner: Address::random(),
                domain: Domain::new("test"),
                initial_data: RegisterContents::Text("Hello, World!".to_string()),
            },
        );
        
        let effect = ResourceEffect::new(operation);
        let effect_bytes = serde_json::to_vec(&effect).unwrap();
        
        let metadata = EffectProofMetadata::new(
            None,
            Address::random(),
            EffectProofFormat::Groth16,
        );
        
        let proof = generator.generate_proof(&effect, Some(metadata.clone())).unwrap();
        
        let serialized = SerializedEffectProof::from_proof(
            &proof,
            metadata,
            Some(effect_bytes),
        );
        
        let bytes = serialized.to_bytes().unwrap();
        let deserialized = SerializedEffectProof::from_bytes(&bytes).unwrap();
        
        assert_eq!(serialized.id, deserialized.id);
        assert_eq!(serialized.metadata.format, deserialized.metadata.format);
    }
} 
