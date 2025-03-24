// Verification Proof Module
//
// This module defines the proof types used in the unified verification framework.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::types::{*};
use crate::crypto::hash::ContentId;;

/// ZK proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofData {
    /// The proving system used
    pub system: String,
    /// The actual proof bytes
    pub proof: Vec<u8>,
    /// Public inputs to the proof
    pub public_inputs: Vec<Vec<u8>>,
    /// The verification key identifier
    pub verification_key_id: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Temporal proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalProofData {
    /// Domain ID
    pub domain_id: DomainId,
    /// The block height
    pub block_height: u64,
    /// The block hash
    pub block_hash: String,
    /// The timestamp
    pub timestamp: Timestamp,
    /// Signatures or attestations from validators
    pub attestations: Vec<TemporalAttestation>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Temporal attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAttestation {
    /// Validator ID
    pub validator_id: String,
    /// Signature
    pub signature: Vec<u8>,
    /// Public key
    pub public_key: Vec<u8>,
    /// Timestamp of the attestation
    pub timestamp: Timestamp,
}

/// Ancestral proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AncestralProofData {
    /// The resource ID
    pub resource_id: ContentId,
    /// The controller label
    pub controller_label: String,
    /// Ancestral path
    pub path: Vec<ControllerRelationship>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Controller relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerRelationship {
    /// Parent controller label
    pub parent: String,
    /// Child controller label
    pub child: String,
    /// Relationship type
    pub relationship_type: String,
    /// Timestamp of the relationship creation
    pub established_at: Timestamp,
    /// Signature of the parent
    pub parent_signature: Option<Vec<u8>>,
}

/// Logical proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalProofData {
    /// The operations verified
    pub operations: Vec<String>,
    /// The logical constraints satisfied
    pub constraints: Vec<ConstraintSatisfaction>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Constraint satisfaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSatisfaction {
    /// Constraint ID
    pub constraint_id: String,
    /// Whether the constraint is satisfied
    pub satisfied: bool,
    /// Satisfaction level (0.0 to 1.0)
    pub satisfaction_level: f64,
    /// Error message if not satisfied
    pub error: Option<String>,
}

/// Cross-domain proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainProofData {
    /// Source domain
    pub source_domain: DomainId,
    /// Destination domain
    pub destination_domain: DomainId,
    /// Cross-domain message or operation
    pub message: Vec<u8>,
    /// Signatures or attestations
    pub attestations: Vec<CrossDomainAttestation>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Cross-domain attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainAttestation {
    /// Attester ID
    pub attester_id: String,
    /// Domain ID
    pub domain_id: DomainId,
    /// Signature
    pub signature: Vec<u8>,
    /// Public key
    pub public_key: Vec<u8>,
    /// Timestamp of the attestation
    pub timestamp: Timestamp,
}

/// Signature for a proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Signer ID
    pub signer_id: String,
    /// Signature bytes
    pub signature: Vec<u8>,
    /// Public key
    pub public_key: Vec<u8>,
    /// Signature algorithm
    pub algorithm: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// A unified proof that can contain multiple verification aspects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedProof {
    /// Unique proof ID
    pub id: String,
    
    /// ZK proof components (if applicable)
    pub zk_components: Option<ZkProofData>,
    
    /// Temporal verification data (time map snapshot)
    pub temporal_components: Option<TemporalProofData>,
    
    /// Ancestral verification data (controller paths)
    pub ancestral_components: Option<AncestralProofData>,
    
    /// Logical verification data (effect validation)
    pub logical_components: Option<LogicalProofData>,
    
    /// Cross-domain verification data
    pub cross_domain_components: Option<CrossDomainProofData>,
    
    /// Metadata about this proof
    pub metadata: HashMap<String, String>,
    
    /// Proof generation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Signature over the proof contents (if applicable)
    pub signature: Option<Signature>,
}

impl UnifiedProof {
    /// Create a new unified proof
    pub fn new(id: String) -> Self {
        Self {
            id,
            zk_components: None,
            temporal_components: None,
            ancestral_components: None,
            logical_components: None,
            cross_domain_components: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            signature: None,
        }
    }
    
    /// Add ZK components
    pub fn with_zk_components(mut self, zk_components: ZkProofData) -> Self {
        self.zk_components = Some(zk_components);
        self
    }
    
    /// Add temporal components
    pub fn with_temporal_components(mut self, temporal_components: TemporalProofData) -> Self {
        self.temporal_components = Some(temporal_components);
        self
    }
    
    /// Add ancestral components
    pub fn with_ancestral_components(mut self, ancestral_components: AncestralProofData) -> Self {
        self.ancestral_components = Some(ancestral_components);
        self
    }
    
    /// Add logical components
    pub fn with_logical_components(mut self, logical_components: LogicalProofData) -> Self {
        self.logical_components = Some(logical_components);
        self
    }
    
    /// Add cross-domain components
    pub fn with_cross_domain_components(mut self, cross_domain_components: CrossDomainProofData) -> Self {
        self.cross_domain_components = Some(cross_domain_components);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Add signature
    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }
    
    /// Check if the proof has any components
    pub fn has_components(&self) -> bool {
        self.zk_components.is_some() ||
        self.temporal_components.is_some() ||
        self.ancestral_components.is_some() ||
        self.logical_components.is_some() ||
        self.cross_domain_components.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unified_proof() {
        let proof = UnifiedProof::new("proof-123".to_string())
            .with_metadata("version", "1.0")
            .with_zk_components(ZkProofData {
                system: "groth16".to_string(),
                proof: vec![1, 2, 3, 4],
                public_inputs: vec![vec![5, 6, 7, 8]],
                verification_key_id: "key-123".to_string(),
                created_at: Utc::now(),
                metadata: HashMap::new(),
            });
        
        assert_eq!(proof.id, "proof-123");
        assert_eq!(proof.metadata.get("version"), Some(&"1.0".to_string()));
        assert!(proof.zk_components.is_some());
        assert!(proof.has_components());
    }
} 
