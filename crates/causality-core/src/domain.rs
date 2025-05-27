//! TEG Domain System
//!
//! Defines domain-aware TEG operations with direct SMT integration.
//! This provides the foundation for multi-domain TEG graph management.

use std::sync::{Arc, Mutex};

// Importing from the correct modules
use causality_types::{
    core::id::DomainId,
    serialization::{Decode, Encode, SimpleSerialize, DecodeError},
};

use crate::smt::{Hash, MemoryBackend, TegMultiDomainSmt};
use serde_json::Value;

//-----------------------------------------------------------------------------
// Domain Data Structure
//-----------------------------------------------------------------------------

/// Domain data for TEG-aware operations with direct write capability
#[derive(Debug, Clone)]
pub struct TegDomainData {
    /// Human-readable domain name
    pub name: String,
    
    /// Domain metadata
    pub metadata: String,
    
    /// Domain-specific configuration for TEG operations
    pub teg_config: TegDomainConfig,
}

impl TegDomainData {
    /// Prefix for domain identifier hash generation
    pub const ID_PREFIX: &'static [u8] = b"teg-domain";
    
    /// Create a new TEG domain
    pub fn new(name: String) -> Self {
        Self {
            name,
            metadata: String::new(),
            teg_config: TegDomainConfig::default(),
        }
    }
    
    /// Add metadata to the domain
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Configure TEG-specific settings
    pub fn with_teg_config(mut self, config: TegDomainConfig) -> Self {
        self.teg_config = config;
        self
    }
    
    /// Generate a unique identifier for this domain
    pub fn identifier(&self) -> DomainId {
        Self::identifier_from_parts(&self.name)
    }
    
    /// Compute domain identifier from name
    pub fn identifier_from_parts(name: &str) -> DomainId {
        use crate::smt::Sha256Hasher;
        let hash = Sha256Hasher::digest(&[Self::ID_PREFIX, name.as_bytes()]);
        DomainId::from(hash)
    }
}

//-----------------------------------------------------------------------------
// TEG Domain Configuration
//-----------------------------------------------------------------------------

/// Configuration for TEG-specific domain operations
#[derive(Debug, Clone)]
pub struct TegDomainConfig {
    /// Enable direct writes to SMT (bypasses validation)
    pub enable_direct_writes: bool,
    
    /// Maximum nodes per TEG transaction
    pub max_nodes_per_transaction: u32,
    
    /// Enable temporal constraint validation
    pub validate_temporal_constraints: bool,
    
    /// Enable cross-domain references
    pub allow_cross_domain_refs: bool,
}

impl Default for TegDomainConfig {
    fn default() -> Self {
        Self {
            enable_direct_writes: true,
            max_nodes_per_transaction: 1000,
            validate_temporal_constraints: true,
            allow_cross_domain_refs: true,
        }
    }
}

//-----------------------------------------------------------------------------
// Manual Serialization Implementations
//-----------------------------------------------------------------------------

impl SimpleSerialize for TegDomainData {}

impl Encode for TegDomainData {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        
        // Encode name (length + data)
        buf.extend_from_slice(&(self.name.len() as u32).to_le_bytes());
        buf.extend_from_slice(self.name.as_bytes());
        
        // Encode metadata (length + data)
        buf.extend_from_slice(&(self.metadata.len() as u32).to_le_bytes());
        buf.extend_from_slice(self.metadata.as_bytes());
        
        // Encode config
        buf.extend_from_slice(&self.teg_config.as_ssz_bytes());
        
        buf
    }
}

impl Decode for TegDomainData {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode name
        if bytes.len() < offset + 4 {
            return Err(DecodeError { message: "Insufficient bytes for name length".to_string() });
        }
        let name_len = u32::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
        ]) as usize;
        offset += 4;
        
        if bytes.len() < offset + name_len {
            return Err(DecodeError { message: "Insufficient bytes for name data".to_string() });
        }
        let name = String::from_utf8(bytes[offset..offset+name_len].to_vec())
            .map_err(|_| DecodeError { message: "Invalid UTF-8 in name".to_string() })?;
        offset += name_len;
        
        // Decode metadata
        if bytes.len() < offset + 4 {
            return Err(DecodeError { message: "Insufficient bytes for metadata length".to_string() });
        }
        let metadata_len = u32::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
        ]) as usize;
        offset += 4;
        
        if bytes.len() < offset + metadata_len {
            return Err(DecodeError { message: "Insufficient bytes for metadata data".to_string() });
        }
        let metadata = String::from_utf8(bytes[offset..offset+metadata_len].to_vec())
            .map_err(|_| DecodeError { message: "Invalid UTF-8 in metadata".to_string() })?;
        offset += metadata_len;
        
        // Decode config
        let teg_config = TegDomainConfig::from_ssz_bytes(&bytes[offset..])
            .map_err(|_| DecodeError { message: "Failed to decode teg_config".to_string() })?;
        
        Ok(Self { name, metadata, teg_config })
    }
}

impl SimpleSerialize for TegDomainConfig {}

impl Encode for TegDomainConfig {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(if self.enable_direct_writes { 1 } else { 0 });
        buf.extend_from_slice(&self.max_nodes_per_transaction.to_le_bytes());
        buf.push(if self.validate_temporal_constraints { 1 } else { 0 });
        buf.push(if self.allow_cross_domain_refs { 1 } else { 0 });
        // Add padding to align to fixed size
        buf.extend_from_slice(&[0u8; 3]);
        buf
    }
}

impl Decode for TegDomainConfig {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 10 {
            return Err(DecodeError { message: "Insufficient bytes for TegDomainConfig".to_string() });
        }
        
        let enable_direct_writes = bytes[0] != 0;
        let max_nodes_per_transaction = u32::from_le_bytes([
            bytes[1], bytes[2], bytes[3], bytes[4]
        ]);
        let validate_temporal_constraints = bytes[5] != 0;
        let allow_cross_domain_refs = bytes[6] != 0;
        
        Ok(Self {
            enable_direct_writes,
            max_nodes_per_transaction,
            validate_temporal_constraints,
            allow_cross_domain_refs,
        })
    }
}

//-----------------------------------------------------------------------------
// TEG State Proof
//-----------------------------------------------------------------------------

/// State proof for TEG domain operations
#[derive(Debug, Clone)]
pub struct TegStateProof {
    /// Domain name
    pub domain: String,
    
    /// TEG graph root hash
    pub root: Hash,
    
    /// Serialized TEG data payload
    pub payload: Vec<u8>,
    
    /// Domain-specific proof data
    pub proof: Vec<u8>,
}

//-----------------------------------------------------------------------------
// TEG Domain Library Interface
//-----------------------------------------------------------------------------

/// Trait for TEG domain libraries with direct write capability
pub trait TegDomainLibrary {
    /// Domain identifier constant
    const ID: &'static str;
    
    /// Generate a state proof from arguments with direct SMT writes
    fn state_proof_direct(&self, args: Value, smt: &mut TegMultiDomainSmt<MemoryBackend>) -> anyhow::Result<TegStateProof>;
    
    /// Compute a state proof without direct writes (compatibility mode)
    fn state_proof(&self, args: Value) -> anyhow::Result<TegStateProof> {
        // Create a temporary SMT for compatibility
        let backend = MemoryBackend::new();
        let mut smt = TegMultiDomainSmt::new(backend);
        self.state_proof_direct(args, &mut smt)
    }
    
    /// Validate a TEG block with direct SMT access
    fn validate_block_direct(&self, args: Value, smt: &mut TegMultiDomainSmt<MemoryBackend>) -> anyhow::Result<TegValidatedBlock>;
    
    /// Get TEG witnesses for ZK circuit generation
    fn get_teg_witnesses(&self, args: Value) -> anyhow::Result<Vec<TegWitness>>;
}

//-----------------------------------------------------------------------------
// TEG Domain Circuit Interface
//-----------------------------------------------------------------------------

/// Trait for TEG domain circuit verification
pub trait TegDomainCircuit {
    /// Output type for circuit verification
    type Output;
    
    /// Verify a TEG state proof in ZK circuit
    fn verify(proof: &TegStateProof) -> anyhow::Result<Self::Output>;
    
    /// Verify temporal constraints in the proof
    fn verify_temporal_constraints(proof: &TegStateProof) -> anyhow::Result<bool>;
    
    /// Verify cross-domain references if enabled
    fn verify_cross_domain_refs(proof: &TegStateProof) -> anyhow::Result<bool>;
}

//-----------------------------------------------------------------------------
// TEG Validated Block
//-----------------------------------------------------------------------------

/// A validated TEG block with direct SMT integration
#[derive(Debug, Clone)]
pub struct TegValidatedBlock {
    /// Block number in the domain
    pub number: u64,
    
    /// TEG graph root hash for this block
    pub root: Hash,
    
    /// Serialized TEG data
    pub payload: Vec<u8>,
    
    /// Number of TEG nodes in this block
    pub node_count: u32,
    
    /// Number of TEG edges in this block
    pub edge_count: u32,
    
    /// Temporal constraint validation results
    pub temporal_valid: bool,
}

//-----------------------------------------------------------------------------
// TEG Witnesses
//-----------------------------------------------------------------------------

/// Witness data for TEG ZK proofs
#[derive(Debug, Clone)]
pub enum TegWitness {
    /// TEG state proof
    StateProof(TegStateProof),
    
    /// TEG node data
    NodeData(Vec<u8>),
    
    /// TEG edge data
    EdgeData(Vec<u8>),
    
    /// Temporal constraint data
    TemporalConstraint(Vec<u8>),
    
    /// Cross-domain reference data
    CrossDomainRef(Vec<u8>),
}

impl TegWitness {
    /// Get state proof if this witness contains one
    pub fn as_state_proof(&self) -> Option<&TegStateProof> {
        match self {
            TegWitness::StateProof(proof) => Some(proof),
            _ => None,
        }
    }
    
    /// Get node data if this witness contains it
    pub fn as_node_data(&self) -> Option<&[u8]> {
        match self {
            TegWitness::NodeData(data) => Some(data),
            _ => None,
        }
    }
    
    /// Get edge data if this witness contains it
    pub fn as_edge_data(&self) -> Option<&[u8]> {
        match self {
            TegWitness::EdgeData(data) => Some(data),
            _ => None,
        }
    }
}

//-----------------------------------------------------------------------------
// TEG Domain Registry
//-----------------------------------------------------------------------------

/// Registry for TEG domains with direct SMT integration
pub struct TegDomainRegistry {
    /// Shared SMT instance for all domains
    smt: Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>>,
    
    /// Domain configurations
    domains: Arc<Mutex<std::collections::HashMap<DomainId, TegDomainData>>>,
}

impl TegDomainRegistry {
    /// Create a new TEG domain registry
    pub fn new() -> Self {
        let backend = MemoryBackend::new();
        let smt = TegMultiDomainSmt::new(backend);
        
        Self {
            smt: Arc::new(Mutex::new(smt)),
            domains: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    /// Register a new TEG domain
    pub fn register_domain(&self, domain_data: TegDomainData) -> anyhow::Result<DomainId> {
        let domain_id = domain_data.identifier();
        
        // Store domain configuration
        {
            let mut domains = self.domains.lock().unwrap();
            domains.insert(domain_id, domain_data.clone());
        }
        
        // Initialize domain in SMT with configuration
        {
            let mut smt = self.smt.lock().unwrap();
            let config_key = format!("{}-domain-config", domain_id.namespace_prefix());
            let config_data = domain_data.as_ssz_bytes();
            smt.store_data(&config_key, &config_data)
                .map_err(|e| anyhow::anyhow!("Failed to store domain config: {}", e))?;
        }
        
        Ok(domain_id)
    }
    
    /// Get domain configuration
    pub fn get_domain(&self, domain_id: &DomainId) -> Option<TegDomainData> {
        let domains = self.domains.lock().unwrap();
        domains.get(domain_id).cloned()
    }
    
    /// Get shared SMT instance for direct writes
    pub fn get_smt(&self) -> Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>> {
        Arc::clone(&self.smt)
    }
    
    /// Store TEG data directly in SMT
    pub fn store_teg_data(
        &self,
        domain_id: &DomainId,
        key: &str,
        data: &[u8],
    ) -> anyhow::Result<()> {
        let mut smt = self.smt.lock().unwrap();
        let teg_key = format!("{}-teg-{}", domain_id.namespace_prefix(), key);
        smt.store_data(&teg_key, data)
            .map_err(|e| anyhow::anyhow!("Failed to store TEG data: {}", e))?;
        Ok(())
    }
    
    /// Get TEG data from SMT
    pub fn get_teg_data(
        &self,
        domain_id: &DomainId,
        key: &str,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let smt = self.smt.lock().unwrap();
        let teg_key = format!("{}-teg-{}", domain_id.namespace_prefix(), key);
        smt.get_data(&teg_key)
            .map_err(|e| anyhow::anyhow!("Failed to get TEG data: {}", e))
    }
    
    /// List all registered domains
    pub fn list_domains(&self) -> Vec<DomainId> {
        let domains = self.domains.lock().unwrap();
        domains.keys().cloned().collect()
    }
}

impl Default for TegDomainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

//-----------------------------------------------------------------------------
// Default Implementation for Basic TEG Domain
//-----------------------------------------------------------------------------

/// Basic TEG domain implementation for development
#[derive(Debug, Clone, Copy)]
pub struct BasicTegDomain;

impl TegDomainLibrary for BasicTegDomain {
    const ID: &'static str = "basic-teg";
    
    fn state_proof_direct(&self, args: Value, smt: &mut TegMultiDomainSmt<MemoryBackend>) -> anyhow::Result<TegStateProof> {
        // Extract TEG data from arguments
        let teg_data = args.get("teg_data")
            .ok_or_else(|| anyhow::anyhow!("TEG data not provided"))?;
        
        // Serialize TEG data
        let payload = serde_json::to_vec(teg_data)?;
        
        // Compute root hash
        let root = crate::smt::Sha256Hasher::hash(&payload);
        
        // Store in SMT directly (this is the key capability)
        let domain_id = TegDomainData::identifier_from_parts(Self::ID);
        let data_key = format!("{}-current-state", domain_id.namespace_prefix());
        smt.store_data(&data_key, &payload)
            .map_err(|e| anyhow::anyhow!("Failed to store state data: {}", e))?;
        
        Ok(TegStateProof {
            domain: Self::ID.to_string(),
            root,
            payload,
            proof: Vec::new(), // No additional proof needed for development
        })
    }
    
    fn validate_block_direct(&self, args: Value, smt: &mut TegMultiDomainSmt<MemoryBackend>) -> anyhow::Result<TegValidatedBlock> {
        let block_number = args.get("number")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        let teg_data = args.get("teg_data")
            .ok_or_else(|| anyhow::anyhow!("TEG data not provided"))?;
        
        let payload = serde_json::to_vec(teg_data)?;
        let root = crate::smt::Sha256Hasher::hash(&payload);
        
        // Store block in SMT
        let domain_id = TegDomainData::identifier_from_parts(Self::ID);
        let block_key = format!("{}-block-{}", domain_id.namespace_prefix(), block_number);
        smt.store_data(&block_key, &payload)
            .map_err(|e| anyhow::anyhow!("Failed to store block data: {}", e))?;
        
        Ok(TegValidatedBlock {
            number: block_number,
            root,
            payload,
            node_count: 1, // Simplified for development
            edge_count: 0,
            temporal_valid: true,
        })
    }
    
    fn get_teg_witnesses(&self, args: Value) -> anyhow::Result<Vec<TegWitness>> {
        let state_proof = self.state_proof(args)?;
        Ok(vec![TegWitness::StateProof(state_proof)])
    }
}

impl TegDomainCircuit for BasicTegDomain {
    type Output = Value;
    
    fn verify(proof: &TegStateProof) -> anyhow::Result<Self::Output> {
        // Verify that the payload hash matches the root
        let computed_root = crate::smt::Sha256Hasher::hash(&proof.payload);
        
        if computed_root != proof.root {
            anyhow::bail!("TEG state proof verification failed: root mismatch");
        }
        
        // Parse the payload back to Value for output
        let output: Value = serde_json::from_slice(&proof.payload)?;
        Ok(output)
    }
    
    fn verify_temporal_constraints(_proof: &TegStateProof) -> anyhow::Result<bool> {
        // Simplified temporal validation for development
        Ok(true)
    }
    
    fn verify_cross_domain_refs(_proof: &TegStateProof) -> anyhow::Result<bool> {
        // Simplified cross-domain validation for development
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_teg_domain_data_creation() {
        let domain = TegDomainData::new("test-domain".to_string())
            .with_metadata("Test domain for TEG".to_string());
        
        assert_eq!(domain.name, "test-domain");
        assert_eq!(domain.metadata, "Test domain for TEG");
        assert!(domain.teg_config.enable_direct_writes);
    }

    #[test]
    fn test_teg_domain_registry() {
        let registry = TegDomainRegistry::new();
        
        let domain_data = TegDomainData::new("test".to_string());
        let domain_id = registry.register_domain(domain_data.clone()).unwrap();
        
        let retrieved = registry.get_domain(&domain_id).unwrap();
        assert_eq!(retrieved.name, domain_data.name);
    }

    #[test]
    fn test_basic_teg_domain() {
        let domain = BasicTegDomain;
        let backend = MemoryBackend::new();
        let mut smt = TegMultiDomainSmt::new(backend);
        
        let args = json!({
            "teg_data": {
                "nodes": ["node1", "node2"],
                "edges": [{"from": "node1", "to": "node2"}]
            }
        });
        
        let proof = domain.state_proof_direct(args, &mut smt).unwrap();
        assert_eq!(proof.domain, "basic-teg");
        assert!(!proof.payload.is_empty());
        
        // Test verification
        let output = BasicTegDomain::verify(&proof).unwrap();
        assert!(output.get("nodes").is_some());
    }

    #[test]
    fn test_teg_domain_direct_storage() {
        let registry = TegDomainRegistry::new();
        let domain_data = TegDomainData::new("storage-test".to_string());
        let domain_id = registry.register_domain(domain_data).unwrap();
        
        let test_data = b"test teg data";
        registry.store_teg_data(&domain_id, "test-key", test_data).unwrap();
        
        let retrieved = registry.get_teg_data(&domain_id, "test-key").unwrap();
        assert_eq!(retrieved.unwrap(), test_data);
    }
} 