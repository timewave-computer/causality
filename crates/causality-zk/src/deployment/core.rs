//! Deployment infrastructure for the Valence Coprocessor
//!
//! This module handles the deployment of compiled circuits to the Valence Coprocessor,
//! including program registration, key management, and secure storage.

extern crate alloc;
use alloc::{string::String, vec::Vec};

use causality_types::serialization::{SimpleSerialize, Encode, Decode, DecodeError};

#[cfg(feature = "host")]
use crate::circuit::core::{Circuit, CircuitTarget, CompiledCircuit};
use crate::core::CircuitId;

//-----------------------------------------------------------------------------
// Program Registration
//-----------------------------------------------------------------------------

/// Registration info for Valence Coprocessor
#[derive(Clone)]
pub struct ProgramRegistration {
    /// Circuit ID for the program
    pub circuit_id: CircuitId,

    /// Program name for the registry
    pub program_name: String,

    /// Version of the program
    pub version: String,

    /// Description of the program
    pub description: String,

    /// Endpoint for the Valence Coprocessor
    #[cfg(feature = "host")]
    pub endpoint: String,
}

impl SimpleSerialize for ProgramRegistration {}

impl Encode for ProgramRegistration {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.circuit_id.0.as_ssz_bytes());
        bytes.extend(self.program_name.as_ssz_bytes());
        bytes.extend(self.version.as_ssz_bytes());
        bytes.extend(self.description.as_ssz_bytes());
        #[cfg(feature = "host")]
        bytes.extend(self.endpoint.as_ssz_bytes());
        bytes
    }
}

impl ProgramRegistration {
    /// Create a new program registration
    #[cfg(feature = "host")]
    pub fn new(
        circuit_id: CircuitId,
        program_name: String,
        version: String,
        description: String,
        endpoint: String,
    ) -> Self {
        Self {
            circuit_id,
            program_name,
            version,
            description,
            endpoint,
        }
    }

    /// Create a new program registration (no_host version)
    #[cfg(not(feature = "host"))]
    pub fn new(
        circuit_id: CircuitId,
        program_name: String,
        version: String,
        description: String,
    ) -> Self {
        Self {
            circuit_id,
            program_name,
            version,
            description,
        }
    }

    /// Register the program with the Valence Coprocessor
    #[cfg(feature = "host")]
    pub async fn register(
        &self,
        _compiled: &CompiledCircuit,
    ) -> Result<String, Error> {
        // Placeholder for real implementation that would call the Valence API
        // This would make HTTP requests to the Coprocessor API

        // Return a program ID (this would be provided by the Valence Coprocessor)
        Ok(format!("program-{:?}", self.circuit_id))
    }
}

//-----------------------------------------------------------------------------
// Key Management
//-----------------------------------------------------------------------------

/// Verification key for ZK proofs
#[derive(Debug, Clone)]
pub struct VerificationKey {
    /// The raw bytes of the verification key.
    pub bytes: Vec<u8>,
}

impl SimpleSerialize for VerificationKey {}

impl Encode for VerificationKey {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.bytes.as_ssz_bytes()
    }
}

impl Decode for VerificationKey {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let key_bytes = Vec::<u8>::from_ssz_bytes(bytes)?;
        Ok(Self { bytes: key_bytes })
    }
}

impl VerificationKey {
    /// Create a new verification key
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

//-----------------------------------------------------------------------------
// Secure Storage
//-----------------------------------------------------------------------------

/// Key store for verification keys using SMT storage
#[cfg(feature = "host")]
pub struct KeyStore {
    /// SMT storage backend
    smt: std::sync::Arc<parking_lot::Mutex<causality_core::smt::DomainSmt<causality_core::smt::MemoryBackend>>>,
    /// Domain ID for storage isolation
    domain_id: causality_types::primitive::ids::DomainId,
}

#[cfg(feature = "host")]
impl KeyStore {
    /// Create a new key store with SMT storage
    pub fn new(domain_id: causality_types::primitive::ids::DomainId) -> Self {
        let backend = causality_core::smt::MemoryBackend::new();
        let smt = causality_core::smt::DomainSmt::new(backend);
        
        Self {
            smt: std::sync::Arc::new(parking_lot::Mutex::new(smt)),
            domain_id,
        }
    }

    /// Store a verification key
    pub fn store_key(&self, circuit_id: &CircuitId, key: &VerificationKey) -> Result<(), Error> {
        // Use SMT storage
        let mut smt_guard = self.smt.lock();
        
        // Serialize the key using SSZ
        let serialized_key = key.as_ssz_bytes();
        
        // Generate SMT key from circuit ID
        let key_path = format!("vkey-{:?}", circuit_id);
        
        // Store in SMT with domain context
        smt_guard.store_data(&self.domain_id, &key_path, &serialized_key)
            .map_err(|e| {
                Error::Serialization(format!("Failed to store key in SMT: {}", e))
            })?;
            
        Ok(())
    }

    /// Retrieve a verification key
    pub fn get_key(&self, circuit_id: &CircuitId) -> Result<VerificationKey, Error> {
        let smt_guard = self.smt.lock();
        
        // Generate SMT key from circuit ID
        let key_path = format!("vkey-{:?}", circuit_id);
        
        // Retrieve from SMT
        let serialized_key = smt_guard.get_data(&self.domain_id, &key_path)
            .map_err(|e| {
                Error::Serialization(format!("Failed to get key from SMT: {}", e))
            })?
            .ok_or_else(|| {
                Error::Serialization(format!("Verification key not found: {:?}", circuit_id))
            })?;
        
        // Deserialize the key
        VerificationKey::from_ssz_bytes(&serialized_key).map_err(|e| {
            Error::Serialization(format!("Failed to deserialize key: {}", e))
        })
    }

    /// List all verification keys in the store
    pub fn list_keys(&self) -> Result<Vec<CircuitId>, Error> {
        let smt_guard = self.smt.lock();
        
        // Get all verification key keys from SMT
        let key_paths = smt_guard.list_keys(&self.domain_id, "vkey-")
            .map_err(|e| {
                Error::Serialization(format!("Failed to list verification keys from SMT: {}", e))
            })?;
        
        let mut circuit_ids = Vec::new();
        
        // Extract circuit IDs from the key paths
        for key_path in key_paths {
            if key_path.starts_with("vkey-") {
                // Parse circuit ID from key path
                // This is a simplified approach - in production would need proper parsing
                if let Some(id_str) = key_path.strip_prefix("vkey-") {
                    // For now, create a placeholder CircuitId based on the path
                    // In practice, we'd need to properly parse the ID from the string
                    let mut id_bytes = [0u8; 32];
                    let path_bytes = id_str.as_bytes();
                    let copy_len = std::cmp::min(32, path_bytes.len());
                    id_bytes[..copy_len].copy_from_slice(&path_bytes[..copy_len]);
                    circuit_ids.push(CircuitId(id_bytes));
                }
            }
        }
        
        Ok(circuit_ids)
    }
}

//-----------------------------------------------------------------------------
// Deployment Pipeline
//-----------------------------------------------------------------------------

/// Deployment manager for circuits
#[cfg(feature = "host")]
pub struct DeploymentManager {
    /// Key store for verification keys
    key_store: KeyStore,

    /// Default endpoint for the Valence Coprocessor
    default_endpoint: String,
}

#[cfg(feature = "host")]
impl DeploymentManager {
    /// Create a new deployment manager
    pub fn new(domain_id: causality_types::primitive::ids::DomainId, default_endpoint: String) -> Self {
        Self {
            key_store: KeyStore::new(domain_id),
            default_endpoint,
        }
    }

    /// Deploy a circuit to the Valence Coprocessor
    pub async fn deploy_circuit(
        &self,
        circuit: &Circuit,
        program_name: String,
        version: String,
        description: String,
        endpoint: Option<String>,
    ) -> Result<(String, VerificationKey), Error> {
        // 1. Compile the circuit for WASM
        let wasm_compiled = CompiledCircuit {
            circuit_id: circuit.id,
            target: CircuitTarget::Wasm,
            bytecode: Vec::new(), // Would be actual bytecode in real implementation
        };

        // 2. Compile the circuit for RISC-V
        let _risc_v_compiled = CompiledCircuit {
            circuit_id: circuit.id,
            target: CircuitTarget::RiscV,
            bytecode: Vec::new(), // Would be actual bytecode in real implementation
        };

        // 3. Register the program
        let endpoint = endpoint.unwrap_or_else(|| self.default_endpoint.clone());
        let registration = ProgramRegistration::new(
            circuit.id,
            program_name,
            version,
            description,
            endpoint,
        );

        let program_id = registration.register(&wasm_compiled).await?;

        // 4. Generate verification key
        // This would normally come from the Valence Coprocessor
        let key_data = Vec::new(); // Placeholder for actual key data
        let verification_key = VerificationKey::new(key_data);

        // 5. Store the verification key
        self.key_store.store_key(&circuit.id, &verification_key)?;

        Ok((program_id, verification_key))
    }

    /// Get a verification key for a circuit
    pub fn get_verification_key(
        &self,
        circuit_id: &CircuitId,
    ) -> Result<VerificationKey, Error> {
        self.key_store.get_key(circuit_id)
    }

    /// List all deployed circuits
    pub fn list_deployed_circuits(&self) -> Result<Vec<CircuitId>, Error> {
        self.key_store.list_keys()
    }
}
