//! Runtime API for ZK integration
//!
//! This module provides the integration API between the Causality runtime
//! and the ZK subsystem, connecting execution traces to proof generation
//! and verification.

extern crate alloc;
use alloc::vec::Vec;

use causality_types::{
    core::id::{EffectId, GraphId},
    serialization::{Encode, Decode, SimpleSerialize},
};

use crate::core::hash_from_serializable;
use crate::core::{CircuitId, Error, ProofId};
#[cfg(feature = "host")]
use crate::deployment::DeploymentManager;

// Import the ZkCoprocessorApi trait and related types

// Use the canonical ExecutionTrace from causality_types
// ResourceState will come via ExecutionTrace which uses causality_types::state::ResourceState

// Assuming ExecutionTrace is used to produce WitnessData
// For generate_witnesses
// For verify_proof

//-----------------------------------------------------------------------------
// Proof Storage
//-----------------------------------------------------------------------------

/// ZK proof with associated metadata
#[derive(Clone)]
pub struct StoredProof {
    /// Unique identifier for this proof
    pub id: ProofId,

    /// The circuit that generated the proof
    pub circuit_id: CircuitId,

    /// The graph execution that was proven
    pub graph_id: GraphId,

    /// Effect IDs that were verified in this proof
    pub effect_ids: Vec<EffectId>,

    /// Proof data in binary format
    pub proof_data: Vec<u8>,

    /// Public inputs for verification
    pub public_inputs: Vec<u8>,
}

impl SimpleSerialize for StoredProof {}

impl Encode for StoredProof {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.0.as_ssz_bytes());
        bytes.extend(self.circuit_id.0.as_ssz_bytes());
        bytes.extend(self.graph_id.as_ssz_bytes());
        bytes.extend(self.effect_ids.as_ssz_bytes());
        bytes.extend(self.proof_data.as_ssz_bytes());
        bytes.extend(self.public_inputs.as_ssz_bytes());
        bytes
    }
}

impl Decode for StoredProof {
    fn from_ssz_bytes(_bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        // Simplified implementation
        Err(causality_types::serialization::DecodeError::new("StoredProof deserialization not implemented"))
    }
}

impl StoredProof {
    /// Create a new stored proof with a generated ID
    pub fn new(
        circuit_id: CircuitId,
        graph_id: GraphId,
        effect_ids: Vec<EffectId>,
        proof_data: Vec<u8>,
        public_inputs: Vec<u8>,
    ) -> Result<Self, Error> {
        // Create an object with a temporary ID
        let mut proof = Self {
            id: ProofId([0; 32]),
            circuit_id,
            graph_id,
            effect_ids,
            proof_data,
            public_inputs,
        };

        // Generate a deterministic ID based on contents
        let id = hash_from_serializable(&proof)?;
        proof.id = ProofId(id);

        Ok(proof)
    }
}

//-----------------------------------------------------------------------------
// Proof Storage Repository
//-----------------------------------------------------------------------------

/// Repository for storing and retrieving proofs using SMT storage
#[cfg(feature = "host")]
pub struct ProofRepository {
    /// SMT storage backend
    smt: Arc<parking_lot::Mutex<causality_core::smt::DomainSmt<causality_core::smt::MemoryBackend>>>,
    /// Domain ID for storage isolation
    domain_id: causality_types::primitive::ids::DomainId,
}

#[cfg(feature = "host")]
impl ProofRepository {
    /// Create a new proof repository with SMT storage
    pub fn new(domain_id: causality_types::primitive::ids::DomainId) -> Self {
        let backend = causality_core::smt::MemoryBackend::new();
        let smt = causality_core::smt::DomainSmt::new(backend);
        
        Self {
            smt: Arc::new(parking_lot::Mutex::new(smt)),
            domain_id,
        }
    }

    /// Store a proof
    pub fn store_proof(&self, proof: &StoredProof) -> Result<(), Error> {
        // Use SMT storage
        let mut smt_guard = self.smt.lock();
        
        // Serialize the proof using SSZ
        let serialized_proof = proof.as_ssz_bytes();
        
        // Generate SMT key from proof ID
        let proof_key = format!("proof-{:?}", proof.id);
        
        // Store in SMT with domain context
        smt_guard.store_data(&self.domain_id, &proof_key, &serialized_proof)
            .map_err(|e| {
                Error::Serialization(format!("Failed to store proof in SMT: {}", e))
            })?;
            
        Ok(())
    }

    /// Get a proof by ID
    pub fn get_proof(&self, proof_id: &ProofId) -> Result<StoredProof, Error> {
        let smt_guard = self.smt.lock();
        
        // Generate SMT key from proof ID
        let proof_key = format!("proof-{:?}", proof_id);
        
        // Retrieve from SMT
        let serialized_proof = smt_guard.get_data(&self.domain_id, &proof_key)
            .map_err(|e| {
                Error::Serialization(format!("Failed to get proof from SMT: {}", e))
            })?
            .ok_or_else(|| {
                Error::Serialization(format!("Proof not found: {:?}", proof_id))
            })?;
        
        // Deserialize the proof
        StoredProof::from_ssz_bytes(&serialized_proof).map_err(|e| {
            Error::Serialization(format!("Failed to deserialize proof: {}", e))
        })
    }

    /// List all proofs for a specific graph execution
    pub fn list_proofs_for_graph(
        &self,
        graph_id: &GraphId,
    ) -> Result<Vec<ProofId>, Error> {
        let smt_guard = self.smt.lock();
        
        // Get all proof keys from SMT
        let proof_keys = smt_guard.list_keys(&self.domain_id, "proof-")
            .map_err(|e| {
                Error::Serialization(format!("Failed to list proofs from SMT: {}", e))
            })?;
        
        let mut matching_proofs = Vec::new();
        
        // Filter proofs by graph ID
        for proof_key in proof_keys {
            if proof_key.starts_with("proof-") {
                // Get the proof data to check graph ID
                if let Ok(Some(serialized_proof)) = smt_guard.get_data(&self.domain_id, &proof_key) {
                    if let Ok(proof) = StoredProof::from_ssz_bytes(&serialized_proof) {
                        if proof.graph_id == *graph_id {
                            matching_proofs.push(proof.id);
                        }
                    }
                }
            }
        }
        
        Ok(matching_proofs)
    }
}

//-----------------------------------------------------------------------------
// Runtime Integration API
//-----------------------------------------------------------------------------

/// API for integrating the Causality runtime with ZK proofs
#[cfg(feature = "host")]
pub struct ZkRuntimeApi {
    /// Repository for proofs
    proof_repo: ProofRepository,
    /// Client for interacting with the ZK Coprocessor
    coprocessor_api: Arc<dyn ZkCoprocessorApi>,
    /// Manager for circuit deployments and verification keys
    deployment_manager: Arc<DeploymentManager>,
}

#[cfg(feature = "host")]
impl ZkRuntimeApi {
    /// Create a new ZK runtime API instance
    pub fn new(
        domain_id: causality_types::primitive::ids::DomainId,
        default_coprocessor_endpoint: String,
        coprocessor_api: Arc<dyn ZkCoprocessorApi>,
    ) -> Self {
        Self {
            proof_repo: ProofRepository::new(domain_id.clone()),
            coprocessor_api,
            deployment_manager: Arc::new(DeploymentManager::new(
                domain_id,
                default_coprocessor_endpoint,
            )),
        }
    }

    /// Generate witnesses from an execution trace
    pub fn generate_witnesses(
        &self,
        trace: &ExecutionTrace,
    ) -> Result<WitnessData, Error> {
        process_execution_trace(trace)
    }

    /// Generate a proof for an execution trace using the ZK Coprocessor API.
    pub async fn generate_proof(
        &self,
        trace: &ExecutionTrace, // This is causality_api::coprocessor::ExecutionTrace due to `use` statement above
        circuit_id: CircuitId,  // This is causality_zk::core::CircuitId
        graph_id: GraphId,      // This is causality_types::primitive::ids::GraphId
    ) -> Result<StoredProof, Error> {
        // 1. Generate witnesses (still a local ZK concern, but uses the ExecutionTrace)
        //    Note: `generate_witnesses` expects `&ExecutionTrace`. If `trace` is consumed by
        //    `submit_trace_for_proving`, it might need to be cloned or `generate_witnesses` adapted.
        //    For now, assume `trace` can be referenced for `generate_witnesses` and then moved.
        let witness_data = self.generate_witnesses(trace)?;

        // 2. Submit trace to coprocessor
        //    The `trace` argument to this function is now assumed to be the one from `causality_api::coprocessor`.
        //    If it was a different `ExecutionTrace` type before, conversion would be needed here.
        let proof_request_id = self
            .coprocessor_api
            .submit_trace_for_proving(trace.clone())
            .await // Clone trace if needed
            .map_err(|e| {
                Error::Coprocessor(format!(
                    "Failed to submit trace for proving: {:?}",
                    e
                ))
            })?;

        // 3. Poll for proof completion (simplified: loop until Completed or Failed)
        //    In a real scenario, this would involve delays and timeouts.
        let mut coprocessor_proof: Option<CoprocessorApiProof> = None;
        loop {
            match self
                .coprocessor_api
                .check_proof_status(&proof_request_id)
                .await
            {
                Ok(CoprocessorProofStatus::Completed) => {
                    coprocessor_proof = Some(
                        self.coprocessor_api
                            .get_proof(&proof_request_id)
                            .await
                            .map_err(|e| {
                                Error::Coprocessor(format!(
                                    "Failed to get proof: {:?}",
                                    e
                                ))
                            })?,
                    );
                    break;
                }
                Ok(CoprocessorProofStatus::Failed) => {
                    return Err(Error::Coprocessor(
                        "Proof generation failed on coprocessor".to_string(),
                    ));
                }
                Ok(CoprocessorProofStatus::Pending)
                | Ok(CoprocessorProofStatus::InProgress) => {
                    // TODO: Add proper polling delay, e.g., tokio::time::sleep(Duration::from_secs(1)).await;
                    // For now, this will be a busy loop if not using a mock that resolves quickly.
                    // If using tokio, ensure it's available: `#[cfg(feature = "host")]` might need tokio features.
                    // Consider adding a small sleep if tokio is available and we are in host mode.
                    #[cfg(feature = "host")]
                    // ZkRuntimeApi and this method are already cfg(feature = "host")
                    {
                        // Assuming tokio is available under host feature. Cargo.toml has it as optional.
                        // Ensure `tokio` crate is accessible here. It might need to be a direct dependency of causality-zk for this.
                        // For now, let this be conditional on a feature that implies tokio is present.
                        // A common pattern is to have a `async_std` or `tokio_runtime` feature for the crate.
                        // The `host` feature in causality-zk enables `sp1-sdk` and `tokio` as optional deps.
                        tokio::time::sleep(std::time::Duration::from_millis(50))
                            .await;
                        // Small sleep
                    }
                }
                Ok(CoprocessorProofStatus::Rejected) => {
                    return Err(Error::Coprocessor(
                        "Proof request rejected by coprocessor".to_string(),
                    ));
                }
                Err(e) => {
                    return Err(Error::Coprocessor(format!(
                        "Failed to check proof status: {:?}",
                        e
                    )));
                }
            }
        }

        let final_coprocessor_proof = coprocessor_proof.ok_or_else(|| {
            Error::Coprocessor("Proof not retrieved after completion".to_string())
        })?;

        // 4. Create and store StoredProof
        //    The `StoredProof` requires `proof_data: Vec<u8>` and `public_inputs: Vec<u8>`.
        //    `CoprocessorApiProof` has `data: Vec<u8>`. We need public inputs.
        //    The `ZkCoprocessorApi` doesn't currently expose a way to get public inputs separately for a job.
        //    Let's assume for now that `witness_data.public_inputs` (if it exists) or a transformation of `trace` can provide this.
        //    Or, `CoprocessorApiProof` should include public inputs, or `get_proof` should return a tuple (Proof, PublicInputs).
        //    For now, let's use a placeholder for public_inputs for StoredProof.
        //    This highlights a potential gap in ZkCoprocessorApi or its current types.

        //    The circuit_id for StoredProof comes from this function's arguments.
        //    The CoprocessorApiProof also has a circuit_id. We should ensure they are consistent or decide which one is canonical.
        //    For now, we use the circuit_id passed to this function.

        let stored_proof = StoredProof::new(
            circuit_id,                           // from function args
            graph_id,                             // from function args
            witness_data.effect_ids.clone(),      // from generated witness
            final_coprocessor_proof.data.clone(), // from coprocessor proof
            Vec::new(), // Placeholder for public_inputs. This needs to be addressed.
                        // One option: witness_data could contain a field for this after generation.
                        // Or, CoprocessorApiProof includes it.
        )?;

        self.proof_repo.store_proof(&stored_proof)?;

        Ok(stored_proof)
    }

    /// Verify a proof using the ZK Coprocessor API.
    pub async fn verify_proof(
        &self,
        proof_to_verify: &StoredProof, // This is the StoredProof to verify
        _key: &VerificationKey, // _key is unused if we fetch based on proof_to_verify.circuit_id
    ) -> Result<bool, Error> {
        // Use the circuit_id from the StoredProof being verified
        let circuit_id_from_proof = proof_to_verify.circuit_id.clone();

        let verification_key_data = self
            .deployment_manager
            .get_verification_key(&circuit_id_from_proof)
            .map_err(|e| {
                Error::GenericError(format!(
                    "Failed to get verification key for {:?}: {}",
                    circuit_id_from_proof, e
                ))
            })?;

        // Serialize public_inputs from StoredProof for verify_with_key if it expects &[u8]
        // StoredProof.public_inputs is already Vec<u8>, so no further serialization needed for that part.

        // Perform local verification using the key and data from proof_to_verify
        verify_with_key(
            &verification_key_data.key_data,
            &proof_to_verify.proof_data,
            &proof_to_verify.public_inputs,
            &circuit_id_from_proof,
        )
        .map_err(|e| {
            Error::GenericError(format!(
                "Local proof verification failed for {:?}: {}",
                proof_to_verify.id, e
            ))
        })?;

        // Convert StoredProof fields to CoprocessorApiProof and CoprocessorApiPublicInputs for the API call
        let api_proof = CoprocessorApiProof {
            program_id: proof_to_verify.circuit_id.to_string(),
            data: proof_to_verify.proof_data.clone(),
            verification_key: verification_key_data.key_data.clone(), // Pass the fetched VK data
            public_inputs: None,
        };

        let api_public_inputs = CoprocessorApiPublicInputs {
            data: proof_to_verify.public_inputs.clone(),
        };

        // Call the coprocessor API to verify the proof
        self.coprocessor_api
            .verify_proof(&api_proof, &api_public_inputs)
            .await
            .map_err(|e| {
                Error::Coprocessor(format!(
                    "Proof verification failed via coprocessor: {:?}",
                    e
                ))
            })
    }

    /// Get verification result for a graph
    pub fn get_verification_result(
        &self,
        graph_id: &GraphId,
        effect_ids: &[EffectId],
    ) -> Result<VerificationResult, Error> {
        // Look for a proof for this graph
        let proof_ids = self.proof_repo.list_proofs_for_graph(graph_id)?;

        if proof_ids.is_empty() {
            // No proofs found
            return Ok(VerificationResult::failure(
                *graph_id,
                Vec::new(),
                Vec::new(),
                Some("No proofs found for this graph".to_string()),
            ));
        }

        // For simplicity, just check the first proof
        let proof = self.proof_repo.get_proof(&proof_ids[0])?;

        // Check if all effect IDs are verified
        let mut verified_effects = Vec::new();
        let mut missing_effects = Vec::new();

        for effect_id in effect_ids {
            if proof.effect_ids.contains(effect_id) {
                verified_effects.push(*effect_id);
            } else {
                missing_effects.push(*effect_id);
            }
        }

        if missing_effects.is_empty() {
            // All effects verified
            Ok(VerificationResult::success(*graph_id, verified_effects))
        } else {
            // Some effects not verified
            Ok(VerificationResult::failure(
                *graph_id,
                verified_effects,
                Vec::new(), // Failed constraint indices
                Some(format!("Some effects not verified: {:?}", missing_effects)),
            ))
        }
    }

    async fn verify_proof_async(
        &self,
        stored_proof: &StoredProof, // Input is StoredProof
    ) -> Result<VerificationResult, Error> {
        // Serialize public inputs for the verification key usage
        let public_inputs_bytes = ssz::to_vec(&stored_proof.public_inputs)
            .map_err(|e| {
                Error::Serialization(format!(
                    "Failed to serialize public inputs: {}",
                    e
                ))
            })?;

        // TODO: Correctly derive/fetch CircuitId from stored_proof.circuit_id (which is already a CircuitId) or a program_id if that's the source.
        // For now, using stored_proof.circuit_id directly if it makes sense, or a HACK if it's complex.
        // StoredProof has circuit_id: CircuitId. So we can use that.
        let circuit_id_for_verification = stored_proof.circuit_id.clone(); // Use CircuitId from StoredProof

        let verification_key_data = self
            .deployment_manager // This should work if self is ValenceZkRuntime
            .get_verification_key(&circuit_id_for_verification)
            .map_err(|e| Error::GenericError(format!("Failed to get verification key: {}", e)))?;

        // Perform local verification using the key
        // This is a placeholder for that logic.
        verify_with_key(
            &verification_key_data.key_data,
            &stored_proof.proof_data, // Use proof_data from StoredProof
            &public_inputs_bytes,     // Use the serialized public inputs
            &circuit_id_for_verification,
        )
        .map_err(|e| {
            Error::GenericError(format!("Local proof verification failed: {}", e))
        })?;

        // If local verification passes, assume remote verification (if any) also passed
        // or construct result based on coprocessor's potential receipt.
        // For now, returning true if local passed.
        Ok(VerificationResult {
            graph_id: stored_proof.graph_id.clone(), // Populate from StoredProof
            success: true,
            verified_effects: stored_proof.effect_ids.clone(), // Populate from StoredProof
            failed_constraints: Vec::new(), // Assuming no failed constraints if local verification passes
            error_message: None,
        })
    }
}
