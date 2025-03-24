# Storage Commitment in Causality

## Overview

This document describes the storage commitment mechanism in the Causality system. Storage commitments provide cryptographic proofs that specific content exists in the system's storage, allowing for verification of data integrity, completeness, and consistency across distributed nodes. These commitments leverage the universal content-addressing architecture (as defined in ADR-028) to create a tamper-evident storage layer that integrates with cross-domain operations, verification frameworks, and zero-knowledge proofs.

## Core Concepts

### Storage Commitment Model

At the core of the storage commitment system is the `StorageCommitment` structure, which creates a binding relationship between content hashes and their metadata:

```rust
pub struct StorageCommitment {
    /// The content hash this commitment refers to
    pub content_id: ContentId,
    
    /// Commitment timestamp
    pub timestamp: Timestamp,
    
    /// Commitment metadata
    pub metadata: CommitmentMetadata,
    
    /// Commitment proof
    pub proof: CommitmentProof,
    
    /// Authority that issued this commitment
    pub authority: Option<Authority>,
}

pub struct CommitmentMetadata {
    /// Storage type
    pub storage_type: StorageType,
    
    /// Object type
    pub object_type: Option<String>,
    
    /// Retention policy
    pub retention: Option<RetentionPolicy>,
    
    /// Replication requirements
    pub replication: Option<ReplicationPolicy>,
    
    /// Domain information
    pub domain: Option<DomainId>,
}

pub enum CommitmentProof {
    /// Merkle proof to the root
    MerkleProof(MerkleProof),
    
    /// Sparse Merkle Tree proof
    SmtProof(SmtProof),
    
    /// Zero-knowledge proof
    ZkProof(ZkProof),
}
```

### Storage Commitment Repository

The commitment repository manages storage commitments across the system:

```rust
pub trait StorageCommitmentRepository: Send + Sync {
    /// Create a new storage commitment
    fn create_commitment(
        &self,
        content_id: ContentId,
        metadata: CommitmentMetadata,
    ) -> Result<StorageCommitment, CommitmentError>;
    
    /// Get a commitment by content ID
    fn get_commitment(
        &self,
        content_id: &ContentId,
    ) -> Result<StorageCommitment, CommitmentError>;
    
    /// Verify a commitment
    fn verify_commitment(
        &self,
        commitment: &StorageCommitment,
    ) -> Result<VerificationResult, VerificationError>;
    
    /// List commitments matching criteria
    fn list_commitments(
        &self,
        filter: CommitmentFilter,
    ) -> Result<Vec<StorageCommitment>, CommitmentError>;
    
    /// Revoke a commitment
    fn revoke_commitment(
        &self,
        content_id: &ContentId,
        reason: RevocationReason,
    ) -> Result<(), CommitmentError>;
}
```

### Storage Commitment Manager

The StorageCommitmentManager provides high-level operations for working with commitments:

```rust
pub struct StorageCommitmentManager {
    /// Commitment repository
    repository: Arc<dyn StorageCommitmentRepository>,
    
    /// Storage system
    storage: Arc<dyn ContentAddressedStorage>,
    
    /// Proof generator
    proof_generator: Arc<dyn CommitmentProofGenerator>,
    
    /// Authority manager
    authority_manager: Arc<AuthorityManager>,
}

impl StorageCommitmentManager {
    /// Commit an object to storage with proof
    pub fn commit_object<T: ContentAddressed>(
        &self,
        object: &T,
        metadata: CommitmentMetadata,
    ) -> Result<StorageCommitment, CommitmentError> {
        // Store the object
        let content_id = self.storage.store(object)?;
        
        // Generate proof
        let proof = self.proof_generator.generate_proof(&content_id, &metadata)?;
        
        // Create commitment
        let commitment = StorageCommitment {
            content_id,
            timestamp: system.current_time(),
            metadata,
            proof,
            authority: self.authority_manager.get_current_authority(),
        };
        
        // Store commitment
        self.repository.create_commitment(commitment.content_id, commitment.metadata)?;
        
        Ok(commitment)
    }
    
    /// Verify object existence and commitment
    pub fn verify_object_commitment<T: ContentAddressed>(
        &self,
        content_id: &ContentId,
    ) -> Result<VerificationResult, VerificationError> {
        // Check if object exists in storage
        if !self.storage.contains(content_id) {
            return Err(VerificationError::ObjectNotFound);
        }
        
        // Get the commitment
        let commitment = self.repository.get_commitment(content_id)?;
        
        // Verify the commitment
        self.repository.verify_commitment(&commitment)
    }
}
```

## Commitment Proof Generation

### Merkle Tree Commitments

For collections of objects, Merkle tree commitments provide efficient proofs of inclusion:

```rust
pub struct MerkleTreeCommitmentProofGenerator {
    /// Merkle tree implementation
    tree: Arc<MerkleTree>,
}

impl CommitmentProofGenerator for MerkleTreeCommitmentProofGenerator {
    fn generate_proof(
        &self,
        content_id: &ContentId,
        metadata: &CommitmentMetadata,
    ) -> Result<CommitmentProof, CommitmentError> {
        // Get the path to the leaf
        let path = self.tree.get_path(content_id)?;
        
        // Generate the merkle proof
        let proof = self.tree.create_proof(content_id, &path)?;
        
        Ok(CommitmentProof::MerkleProof(proof))
    }
    
    fn verify_proof(
        &self,
        content_id: &ContentId,
        proof: &CommitmentProof,
    ) -> Result<bool, VerificationError> {
        match proof {
            CommitmentProof::MerkleProof(merkle_proof) => {
                self.tree.verify_proof(content_id, merkle_proof)
            },
            _ => Err(VerificationError::UnsupportedProofType),
        }
    }
}
```

### Sparse Merkle Tree Commitments

For key-value storage models, Sparse Merkle Trees provide efficient verification:

```rust
pub struct SparseTreeCommitmentProofGenerator {
    /// Sparse Merkle tree implementation
    tree: Arc<MerkleSmt>,
}

impl CommitmentProofGenerator for SparseTreeCommitmentProofGenerator {
    fn generate_proof(
        &self,
        content_id: &ContentId,
        metadata: &CommitmentMetadata,
    ) -> Result<CommitmentProof, CommitmentError> {
        // Convert content ID to key format
        let key = self.to_key_format(content_id);
        
        // Get the SMT proof
        let proof = self.tree.create_proof(&key)?;
        
        Ok(CommitmentProof::SmtProof(proof))
    }
    
    fn verify_proof(
        &self,
        content_id: &ContentId,
        proof: &CommitmentProof,
    ) -> Result<bool, VerificationError> {
        match proof {
            CommitmentProof::SmtProof(smt_proof) => {
                let key = self.to_key_format(content_id);
                self.tree.verify_proof(&key, smt_proof)
            },
            _ => Err(VerificationError::UnsupportedProofType),
        }
    }
}
```

### ZK Proof Commitments

For privacy-sensitive storage operations, zero-knowledge proofs ensure verifiability without revealing content:

```rust
pub struct ZkCommitmentProofGenerator {
    /// ZK prover
    prover: Arc<dyn ZkProver>,
    
    /// Storage circuit
    circuit: Arc<StorageCircuit>,
}

impl CommitmentProofGenerator for ZkCommitmentProofGenerator {
    fn generate_proof(
        &self,
        content_id: &ContentId,
        metadata: &CommitmentMetadata,
    ) -> Result<CommitmentProof, CommitmentError> {
        // Prepare public and private inputs
        let public_inputs = self.prepare_public_inputs(content_id, metadata)?;
        let private_inputs = self.prepare_private_inputs(content_id, metadata)?;
        
        // Generate the ZK proof
        let proof = self.prover.prove(&self.circuit, &public_inputs, &private_inputs)?;
        
        Ok(CommitmentProof::ZkProof(proof))
    }
    
    fn verify_proof(
        &self,
        content_id: &ContentId,
        proof: &CommitmentProof,
    ) -> Result<bool, VerificationError> {
        match proof {
            CommitmentProof::ZkProof(zk_proof) => {
                let public_inputs = self.prepare_public_inputs(content_id, &metadata)?;
                self.prover.verify(zk_proof, &public_inputs)
            },
            _ => Err(VerificationError::UnsupportedProofType),
        }
    }
}
```

## Integration with Content-Addressed Storage

The storage commitment system integrates seamlessly with the content-addressed storage architecture:

```rust
pub struct CommitmentAwareStorage {
    /// Base content-addressed storage
    storage: Arc<dyn ContentAddressedStorage>,
    
    /// Commitment manager
    commitment_manager: Arc<StorageCommitmentManager>,
}

impl ContentAddressedStorage for CommitmentAwareStorage {
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentId, StorageError> {
        // Store in base storage
        let content_id = self.storage.store(object)?;
        
        // Create default commitment metadata
        let metadata = CommitmentMetadata {
            storage_type: StorageType::default(),
            object_type: Some(std::any::type_name::<T>().to_string()),
            retention: None,
            replication: None,
            domain: None,
        };
        
        // Create commitment
        let _ = self.commitment_manager.commit_object(object, metadata)?;
        
        Ok(content_id)
    }
    
    fn contains(&self, id: &ContentId) -> bool {
        self.storage.contains(id)
    }
    
    fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError> {
        // Verify commitment before returning data
        let verification = self.commitment_manager.verify_object_commitment(id)
            .map_err(|e| StorageError::IoError(format!("Commitment verification failed: {}", e)))?;
            
        if !verification.is_valid {
            return Err(StorageError::HashMismatch(
                format!("Commitment verification failed: {}", verification.message)
            ));
        }
        
        // Return data from base storage
        self.storage.get_bytes(id)
    }
    
    // Other method implementations delegate to base storage
    // ...
}
```

## Root of Trust and Authority Model

Storage commitments require a root of trust to validate the authority issuing commitments:

```rust
pub struct AuthorityManager {
    /// Trusted authorities
    authorities: RwLock<HashMap<AuthorityId, Authority>>,
    
    /// Current authority
    current_authority: RwLock<Option<Authority>>,
    
    /// Authority key manager
    key_manager: Arc<KeyManager>,
}

impl AuthorityManager {
    /// Add a trusted authority
    pub fn add_trusted_authority(
        &self,
        authority: Authority,
    ) -> Result<(), AuthorityError> {
        let mut authorities = self.authorities.write().unwrap();
        authorities.insert(authority.id, authority);
        Ok(())
    }
    
    /// Set the current authority
    pub fn set_current_authority(
        &self,
        authority_id: AuthorityId,
    ) -> Result<(), AuthorityError> {
        let authorities = self.authorities.read().unwrap();
        
        let authority = authorities.get(&authority_id)
            .ok_or_else(|| AuthorityError::UnknownAuthority(authority_id))?;
            
        let mut current = self.current_authority.write().unwrap();
        *current = Some(authority.clone());
        
        Ok(())
    }
    
    /// Get the current authority
    pub fn get_current_authority(&self) -> Option<Authority> {
        let current = self.current_authority.read().unwrap();
        current.clone()
    }
    
    /// Verify an authority signature
    pub fn verify_authority_signature(
        &self,
        authority_id: &AuthorityId,
        message: &[u8],
        signature: &Signature,
    ) -> Result<bool, AuthorityError> {
        let authorities = self.authorities.read().unwrap();
        
        let authority = authorities.get(authority_id)
            .ok_or_else(|| AuthorityError::UnknownAuthority(*authority_id))?;
            
        Ok(self.key_manager.verify_signature(
            &authority.public_key,
            message,
            signature,
        )?)
    }
}
```

## Cross-Domain Commitment Verification

Storage commitments can be verified across domain boundaries:

```rust
pub struct CrossDomainCommitmentVerifier {
    /// Verifier for the source domain
    source_verifier: Arc<dyn CommitmentVerifier>,
    
    /// Verifier for the target domain
    target_verifier: Arc<dyn CommitmentVerifier>,
    
    /// Domain adapter
    domain_adapter: Arc<dyn DomainAdapter>,
}

impl CrossDomainCommitmentVerifier {
    /// Verify a commitment across domains
    pub fn verify_cross_domain(
        &self,
        commitment: &StorageCommitment,
        source_domain: &DomainId,
        target_domain: &DomainId,
    ) -> Result<CrossDomainVerificationResult, VerificationError> {
        // Verify in source domain
        let source_result = self.source_verifier.verify_commitment(commitment)?;
        
        if !source_result.is_valid {
            return Ok(CrossDomainVerificationResult {
                is_valid: false,
                source_verification: source_result,
                target_verification: None,
                cross_domain_verification: None,
            });
        }
        
        // Translate commitment to target domain format
        let translated_commitment = self.domain_adapter.translate_commitment(
            commitment,
            source_domain,
            target_domain,
        )?;
        
        // Verify in target domain
        let target_result = self.target_verifier.verify_commitment(&translated_commitment)?;
        
        // Verify the cross-domain relationship
        let cross_domain_result = self.domain_adapter.verify_commitment_relationship(
            commitment,
            &translated_commitment,
            source_domain,
            target_domain,
        )?;
        
        Ok(CrossDomainVerificationResult {
            is_valid: target_result.is_valid && cross_domain_result.is_valid,
            source_verification: source_result,
            target_verification: Some(target_result),
            cross_domain_verification: Some(cross_domain_result),
        })
    }
}
```

## Temporal Aspects of Storage Commitments

Storage commitments include temporal constraints for time-bound validity:

```rust
pub struct TemporalStorageCommitment {
    /// Base commitment
    pub base: StorageCommitment,
    
    /// Valid from timestamp
    pub valid_from: Timestamp,
    
    /// Valid until timestamp
    pub valid_until: Option<Timestamp>,
    
    /// Temporal proof
    pub temporal_proof: TemporalProof,
}

impl TemporalStorageCommitment {
    /// Check if the commitment is currently valid
    pub fn is_valid_at(&self, time: Timestamp) -> bool {
        if time < self.valid_from {
            return false;
        }
        
        if let Some(valid_until) = self.valid_until {
            if time > valid_until {
                return false;
            }
        }
        
        true
    }
    
    /// Verify the temporal constraints
    pub fn verify_temporal_constraints(
        &self,
        time: Timestamp,
    ) -> Result<TemporalVerificationResult, VerificationError> {
        // Verify base commitment
        let base_result = self.base.verify()?;
        
        if !base_result.is_valid {
            return Ok(TemporalVerificationResult {
                is_valid: false,
                base_verification: base_result,
                temporal_constraints_valid: false,
                temporal_proof_valid: false,
            });
        }
        
        // Check temporal constraints
        let constraints_valid = self.is_valid_at(time);
        
        // Verify temporal proof
        let proof_valid = self.temporal_proof.verify(
            &self.base.content_id,
            self.valid_from,
            self.valid_until,
            time,
        )?;
        
        Ok(TemporalVerificationResult {
            is_valid: base_result.is_valid && constraints_valid && proof_valid,
            base_verification: base_result,
            temporal_constraints_valid: constraints_valid,
            temporal_proof_valid: proof_valid,
        })
    }
}
```

## Usage Examples

### Basic Storage Commitment

```rust
// Get the storage commitment manager
let commitment_manager = system.storage_commitment_manager();

// Create an object to store
let resource = Resource::new("resource-1", "Sample resource");

// Create commitment metadata
let metadata = CommitmentMetadata::new(StorageType::Resource)
    .with_object_type("Resource")
    .with_retention(RetentionPolicy::Permanent)
    .with_domain(domain_id);

// Commit the object to storage
let commitment = commitment_manager.commit_object(&resource, metadata)?;

println!("Created commitment with ID: {}", commitment.content_id);

// Later, verify the commitment
let verification = commitment_manager.verify_object_commitment(&commitment.content_id)?;

if verification.is_valid {
    println!("Commitment verified: {}", verification.message);
} else {
    println!("Commitment verification failed: {}", verification.message);
}
```

### Cross-Domain Commitment Verification

```rust
// Create a cross-domain verifier
let cross_domain_verifier = CrossDomainCommitmentVerifier::new(
    source_domain_verifier,
    target_domain_verifier,
    domain_adapter,
);

// Verify a commitment across domains
let verification = cross_domain_verifier.verify_cross_domain(
    &commitment,
    &source_domain_id,
    &target_domain_id,
)?;

if verification.is_valid {
    println!("Cross-domain verification successful");
} else {
    if let Some(cross_domain_result) = &verification.cross_domain_verification {
        println!("Cross-domain verification failed: {}", cross_domain_result.message);
    } else if let Some(target_result) = &verification.target_verification {
        println!("Target domain verification failed: {}", target_result.message);
    } else {
        println!("Source domain verification failed: {}", verification.source_verification.message);
    }
}
```

### Temporal Commitment

```rust
// Create a temporal commitment manager
let temporal_manager = TemporalStorageCommitmentManager::new(
    commitment_manager,
    temporal_proof_generator,
);

// Create a temporal commitment
let temporal_commitment = temporal_manager.create_temporal_commitment(
    &resource,
    metadata,
    system.current_time(),
    system.current_time() + Duration::from_days(30),
)?;

// Verify at a specific time
let time = system.current_time() + Duration::from_days(15);
let verification = temporal_manager.verify_at_time(&temporal_commitment, time)?;

println!("Temporal verification at {}: {}", time, verification.is_valid);
```

## Implementation Status

The current implementation status of Storage Commitments:

- ✅ Core content-addressed storage interface
- ✅ Basic commitment structures and repository
- ⚠️ Merkle tree commitment proofs (partially implemented)
- ⚠️ Sparse Merkle tree commitment proofs (partially implemented)
- ⚠️ Authority model (partially implemented)
- ❌ ZK proof commitments (not yet implemented)
- ❌ Cross-domain commitment verification (not yet implemented)
- ❌ Temporal commitments (not yet implemented)

## Future Enhancements

Planned future enhancements for Storage Commitments:

1. **Decentralized Authority Model**: Support for decentralized trust models using threshold signatures
2. **Commitment Aggregation**: Create aggregate commitments for sets of related objects
3. **Quantum-Resistant Commitments**: Integration of post-quantum commitment schemes
4. **Witness-Based Commitments**: Support for witness-based commitment schemes for private data
5. **Selective Disclosure**: Allow selective disclosure of committed data through ZK proofs
6. **Automated Verification Policies**: Define and enforce automatic verification policies
7. **Replicated Commitment Verification**: Verify commitments across replicated storage nodes
8. **Commitment-Based Recovery**: Use commitments to recover from storage failures