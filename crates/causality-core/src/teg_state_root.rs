//! TEG State Root Management
//!
//! This module provides domain-specific state root tracking and management
//! for TEG nodes, graphs, and cross-domain operations. It enables cryptographic
//! verification of TEG state changes and supports multi-domain state coordination.

use anyhow::Result;
use std::collections::HashMap;

use causality_types::{
    core::id::{DomainId, GraphId},
    serialization::{Encode, Decode, SimpleSerialize},
};
use crate::smt::{TegMultiDomainSmt, MemoryBackend, Hash};
use std::sync::Arc;
use parking_lot::Mutex;
use sha2::{Digest, Sha256};

//-----------------------------------------------------------------------------
// TEG State Root Data Structures
//-----------------------------------------------------------------------------

/// Domain-specific state root for TEG data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TegDomainStateRoot {
    /// Domain this state root belongs to
    pub domain_id: DomainId,
    
    /// Overall TEG state root for the domain
    pub teg_root: Hash,
    
    /// State root for TEG nodes in this domain
    pub nodes_root: Hash,
    
    /// State root for TEG effects in this domain
    pub effects_root: Hash,
    
    /// State root for TEG resources in this domain
    pub resources_root: Hash,
    
    /// State root for TEG intents in this domain
    pub intents_root: Hash,
    
    /// State root for TEG handlers in this domain
    pub handlers_root: Hash,
    
    /// State root for TEG constraints in this domain
    pub constraints_root: Hash,
    
    /// State root for cross-domain references from this domain
    pub cross_domain_refs_root: Hash,
    
    /// Block height / sequence number for ordering
    pub sequence: u64,
    
    /// Timestamp when this state root was computed
    pub timestamp: u64,
}

/// TEG graph state root with hierarchical structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TegGraphStateRoot {
    /// Graph identifier
    pub graph_id: GraphId,
    
    /// Domain this graph belongs to
    pub domain_id: DomainId,
    
    /// Root hash of all nodes in this graph
    pub nodes_root: Hash,
    
    /// Root hash of all edges in this graph
    pub edges_root: Hash,
    
    /// Root hash of all effects in this graph
    pub effects_root: Hash,
    
    /// Root hash of all resources in this graph
    pub resources_root: Hash,
    
    /// Root hash of all intents in this graph
    pub intents_root: Hash,
    
    /// Root hash of all temporal constraints in this graph
    pub temporal_constraints_root: Hash,
    
    /// Combined state root for the entire graph
    pub combined_root: Hash,
    
    /// Sequence number for ordering graph state changes
    pub sequence: u64,
}

/// Cross-domain state reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossDomainStateReference {
    /// Source domain making the reference
    pub source_domain: DomainId,
    
    /// Target domain being referenced
    pub target_domain: DomainId,
    
    /// Target entity ID being referenced
    pub target_entity_id: Vec<u8>,
    
    /// Type of entity being referenced
    pub entity_type: String,
    
    /// State root of the target at the time of reference
    pub target_state_root: Hash,
    
    /// Proof that the referenced entity exists in target domain
    pub inclusion_proof: Vec<u8>,
    
    /// Sequence number for ordering
    pub sequence: u64,
}

/// TEG state root - top-level state hash for all TEG graphs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TegStateRoot {
    /// The root hash of all graph state roots
    pub root: Hash,
    
    /// Hash of the multi-domain SMT root
    pub smt_root: Hash,
}

impl SimpleSerialize for TegStateRoot {}

impl Encode for TegStateRoot {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.root.as_ssz_bytes());
        bytes.extend(self.smt_root.as_ssz_bytes());
        bytes
    }
}

impl Decode for TegStateRoot {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        if bytes.len() < 64 { // Hash is 32 bytes each
            return Err(causality_types::serialization::DecodeError {
                message: "Insufficient bytes for TegStateRoot".to_string(),
            });
        }
        
        let root = Hash::from_ssz_bytes(&bytes[0..32])?;
        let smt_root = Hash::from_ssz_bytes(&bytes[32..64])?;
        
        Ok(TegStateRoot { root, smt_root })
    }
}

//-----------------------------------------------------------------------------
// TEG State Root Manager
//-----------------------------------------------------------------------------

/// Manager for domain-specific TEG state roots
pub struct TegStateRootManager {
    /// SMT backend for persistent state root storage
    smt: Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>>,
    
    /// Cache of current domain state roots
    domain_roots: Arc<Mutex<HashMap<DomainId, TegDomainStateRoot>>>,
    
    /// Cache of current graph state roots
    graph_roots: Arc<Mutex<HashMap<GraphId, TegGraphStateRoot>>>,
    
    /// Cache of cross-domain references
    cross_domain_refs: Arc<Mutex<HashMap<String, CrossDomainStateReference>>>,
}

impl Default for TegStateRootManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TegStateRootManager {
    /// Create a new TEG state root manager
    pub fn new() -> Self {
        Self {
            smt: Arc::new(Mutex::new(TegMultiDomainSmt::new(MemoryBackend::new()))),
            domain_roots: Arc::new(Mutex::new(HashMap::new())),
            graph_roots: Arc::new(Mutex::new(HashMap::new())),
            cross_domain_refs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Create a new TEG state root manager with existing SMT
    pub fn with_smt(smt: Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>>) -> Self {
        Self {
            smt,
            domain_roots: Arc::new(Mutex::new(HashMap::new())),
            graph_roots: Arc::new(Mutex::new(HashMap::new())),
            cross_domain_refs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Initialize a new domain state root
    pub fn initialize_domain_state_root(&self, domain_id: DomainId) -> Result<TegDomainStateRoot, String> {
        let empty_hash = [0u8; 32];
        
        let domain_root = TegDomainStateRoot {
            domain_id,
            teg_root: empty_hash,
            nodes_root: empty_hash,
            effects_root: empty_hash,
            resources_root: empty_hash,
            intents_root: empty_hash,
            handlers_root: empty_hash,
            constraints_root: empty_hash,
            cross_domain_refs_root: empty_hash,
            sequence: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        // Store in cache
        {
            let mut cache = self.domain_roots.lock();
            cache.insert(domain_id, domain_root.clone());
        }
        
        // Store in SMT
        self.store_domain_state_root(&domain_root)?;
        
        Ok(domain_root)
    }
    
    /// Get the current state root for a domain
    pub fn get_domain_state_root(&self, domain_id: &DomainId) -> Result<Option<TegDomainStateRoot>, String> {
        // Check cache first
        {
            let cache = self.domain_roots.lock();
            if let Some(root) = cache.get(domain_id) {
                return Ok(Some(root.clone()));
            }
        }
        
        // Load from SMT
        let smt_key = domain_id.generate_domain_state_key();
        let smt = self.smt.lock();
        
        if let Some(data) = smt.get_data(&smt_key)
            .map_err(|e| format!("Failed to get domain state root: {}", e))? {
            
            let domain_root = TegDomainStateRoot::from_ssz_bytes(&data)
                .map_err(|e| format!("Failed to deserialize domain state root: {}", e.message))?;
            
            // Update cache
            {
                let mut cache = self.domain_roots.lock();
                cache.insert(*domain_id, domain_root.clone());
            }
            
            Ok(Some(domain_root))
        } else {
            Ok(None)
        }
    }
    
    /// Update the TEG nodes root for a domain
    pub fn update_domain_nodes_root(&self, domain_id: &DomainId, new_nodes_root: Hash) -> Result<(), String> {
        let mut domain_root = self.get_domain_state_root(domain_id)?
            .unwrap_or_else(|| {
                // Create new domain root if it doesn't exist
                
                TegDomainStateRoot {
                    domain_id: *domain_id,
                    teg_root: [0u8; 32],
                    nodes_root: [0u8; 32],
                    effects_root: [0u8; 32],
                    resources_root: [0u8; 32],
                    intents_root: [0u8; 32],
                    handlers_root: [0u8; 32],
                    constraints_root: [0u8; 32],
                    cross_domain_refs_root: [0u8; 32],
                    sequence: 0,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                }
            });
        
        domain_root.nodes_root = new_nodes_root;
        domain_root.sequence += 1;
        domain_root.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Recompute TEG root
        domain_root.teg_root = self.compute_combined_teg_root(&domain_root);
        
        self.store_domain_state_root(&domain_root)?;
        
        Ok(())
    }
    
    /// Update the TEG effects root for a domain
    pub fn update_domain_effects_root(&self, domain_id: &DomainId, new_effects_root: Hash) -> Result<(), String> {
        let mut domain_root = self.get_domain_state_root(domain_id)?
            .ok_or_else(|| format!("Domain {} not initialized", domain_id))?;
        
        domain_root.effects_root = new_effects_root;
        domain_root.sequence += 1;
        domain_root.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Recompute TEG root
        domain_root.teg_root = self.compute_combined_teg_root(&domain_root);
        
        self.store_domain_state_root(&domain_root)?;
        
        Ok(())
    }
    
    /// Update multiple TEG component roots atomically
    pub fn update_domain_roots_batch(&self, 
        domain_id: &DomainId,
        updates: Vec<(String, Hash)>
    ) -> Result<(), String> {
        let mut domain_root = self.get_domain_state_root(domain_id)?
            .ok_or_else(|| format!("Domain {} not initialized", domain_id))?;
        
        // Apply all updates
        for (component, new_root) in updates {
            match component.as_str() {
                "nodes" => domain_root.nodes_root = new_root,
                "effects" => domain_root.effects_root = new_root,
                "resources" => domain_root.resources_root = new_root,
                "intents" => domain_root.intents_root = new_root,
                "handlers" => domain_root.handlers_root = new_root,
                "constraints" => domain_root.constraints_root = new_root,
                "cross_domain_refs" => domain_root.cross_domain_refs_root = new_root,
                _ => return Err(format!("Unknown TEG component: {}", component)),
            }
        }
        
        domain_root.sequence += 1;
        domain_root.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Recompute TEG root
        domain_root.teg_root = self.compute_combined_teg_root(&domain_root);
        
        self.store_domain_state_root(&domain_root)?;
        
        Ok(())
    }
    
    /// Initialize a new graph state root
    pub fn initialize_graph_state_root(&self, graph_id: GraphId, domain_id: DomainId) -> Result<TegGraphStateRoot, String> {
        let empty_hash = [0u8; 32];
        
        let graph_root = TegGraphStateRoot {
            graph_id,
            domain_id,
            nodes_root: empty_hash,
            edges_root: empty_hash,
            effects_root: empty_hash,
            resources_root: empty_hash,
            intents_root: empty_hash,
            temporal_constraints_root: empty_hash,
            combined_root: empty_hash,
            sequence: 0,
        };
        
        // Store in cache
        {
            let mut cache = self.graph_roots.lock();
            cache.insert(graph_id, graph_root.clone());
        }
        
        // Store in SMT
        self.store_graph_state_root(&graph_root)?;
        
        Ok(graph_root)
    }
    
    /// Update a graph's state root
    pub fn update_graph_state_root(&self, 
        graph_id: &GraphId,
        component: &str,
        new_root: Hash
    ) -> Result<(), String> {
        let mut graph_root = self.get_graph_state_root(graph_id)?
            .ok_or_else(|| format!("Graph {} not initialized", graph_id))?;
        
        match component {
            "nodes" => graph_root.nodes_root = new_root,
            "edges" => graph_root.edges_root = new_root,
            "effects" => graph_root.effects_root = new_root,
            "resources" => graph_root.resources_root = new_root,
            "intents" => graph_root.intents_root = new_root,
            "temporal_constraints" => graph_root.temporal_constraints_root = new_root,
            _ => return Err(format!("Unknown graph component: {}", component)),
        }
        
        graph_root.sequence += 1;
        
        // Recompute combined root
        graph_root.combined_root = self.compute_combined_graph_root(&graph_root);
        
        self.store_graph_state_root(&graph_root)?;
        
        Ok(())
    }
    
    /// Create a cross-domain state reference
    pub fn create_cross_domain_reference(&self,
        source_domain: &DomainId,
        target_domain: &DomainId,
        target_entity_id: &[u8],
        entity_type: &str,
        inclusion_proof: Vec<u8>
    ) -> Result<String, String> {
        // Get target domain's current state root
        let target_state_root = self.get_domain_state_root(target_domain)?
            .ok_or_else(|| format!("Target domain {} not found", target_domain))?
            .teg_root;
        
        let cross_ref = CrossDomainStateReference {
            source_domain: *source_domain,
            target_domain: *target_domain,
            target_entity_id: target_entity_id.to_vec(),
            entity_type: entity_type.to_string(),
            target_state_root,
            inclusion_proof,
            sequence: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        let ref_key = source_domain.generate_cross_domain_ref_key(target_domain, target_entity_id);
        
        // Store in cache
        {
            let mut cache = self.cross_domain_refs.lock();
            cache.insert(ref_key.clone(), cross_ref.clone());
        }
        
        // Store in SMT
        self.store_cross_domain_reference(&ref_key, &cross_ref)?;
        
        Ok(ref_key)
    }
    
    /// Get a graph's state root
    pub fn get_graph_state_root(&self, graph_id: &GraphId) -> Result<Option<TegGraphStateRoot>, String> {
        // Check cache first
        {
            let cache = self.graph_roots.lock();
            if let Some(root) = cache.get(graph_id) {
                return Ok(Some(root.clone()));
            }
        }
        
        // Load from SMT - for now return None as we need domain context
        Ok(None)
    }
    
    /// Compute the combined TEG root for a domain
    fn compute_combined_teg_root(&self, domain_root: &TegDomainStateRoot) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(domain_root.nodes_root);
        hasher.update(domain_root.effects_root);
        hasher.update(domain_root.resources_root);
        hasher.update(domain_root.intents_root);
        hasher.update(domain_root.handlers_root);
        hasher.update(domain_root.constraints_root);
        hasher.update(domain_root.cross_domain_refs_root);
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    /// Compute the combined graph root
    fn compute_combined_graph_root(&self, graph_root: &TegGraphStateRoot) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(graph_root.nodes_root);
        hasher.update(graph_root.edges_root);
        hasher.update(graph_root.effects_root);
        hasher.update(graph_root.resources_root);
        hasher.update(graph_root.intents_root);
        hasher.update(graph_root.temporal_constraints_root);
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    /// Store domain state root in SMT
    fn store_domain_state_root(&self, domain_root: &TegDomainStateRoot) -> Result<(), String> {
        let smt_key = domain_root.domain_id.generate_domain_state_key();
        let serialized_data = domain_root.as_ssz_bytes();
        
        let mut smt = self.smt.lock();
        smt.store_data(&smt_key, &serialized_data)
            .map_err(|e| format!("Failed to store domain state root: {}", e))?;
        
        // Update cache
        {
            let mut cache = self.domain_roots.lock();
            cache.insert(domain_root.domain_id, domain_root.clone());
        }
        
        Ok(())
    }
    
    /// Store graph state root in SMT
    fn store_graph_state_root(&self, graph_root: &TegGraphStateRoot) -> Result<(), String> {
        let smt_key = format!("graph-{}", hex::encode(graph_root.graph_id.0));
        let serialized_data = graph_root.as_ssz_bytes();
        
        let mut smt = self.smt.lock();
        smt.store_data(&smt_key, &serialized_data)
            .map_err(|e| format!("Failed to store graph state root: {}", e))?;
        
        // Update cache
        {
            let mut cache = self.graph_roots.lock();
            cache.insert(graph_root.graph_id, graph_root.clone());
        }
        
        Ok(())
    }
    
    /// Store cross-domain reference in SMT
    fn store_cross_domain_reference(&self, ref_key: &str, cross_ref: &CrossDomainStateReference) -> Result<(), String> {
        let serialized_data = cross_ref.as_ssz_bytes();
        
        let mut smt = self.smt.lock();
        smt.store_data(ref_key, &serialized_data)
            .map_err(|e| format!("Failed to store cross-domain reference: {}", e))?;
        
        Ok(())
    }
}

//-----------------------------------------------------------------------------
// Serialization Implementations
//-----------------------------------------------------------------------------

impl SimpleSerialize for TegDomainStateRoot {}

impl Encode for TegDomainStateRoot {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.teg_root);
        bytes.extend_from_slice(&self.nodes_root);
        bytes.extend_from_slice(&self.effects_root);
        bytes.extend_from_slice(&self.resources_root);
        bytes.extend_from_slice(&self.intents_root);
        bytes.extend_from_slice(&self.handlers_root);
        bytes.extend_from_slice(&self.constraints_root);
        bytes.extend_from_slice(&self.cross_domain_refs_root);
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes
    }
}

impl Decode for TegDomainStateRoot {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        if bytes.len() < 32 + 32*8 + 8 + 8 {
            return Err(causality_types::serialization::DecodeError {
                message: "Insufficient data for TegDomainStateRoot".to_string(),
            });
        }
        
        let mut offset = 0;
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;
        
        let mut teg_root = [0u8; 32];
        teg_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut nodes_root = [0u8; 32];
        nodes_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut effects_root = [0u8; 32];
        effects_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut resources_root = [0u8; 32];
        resources_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut intents_root = [0u8; 32];
        intents_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut handlers_root = [0u8; 32];
        handlers_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut constraints_root = [0u8; 32];
        constraints_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut cross_domain_refs_root = [0u8; 32];
        cross_domain_refs_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let sequence = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        offset += 8;
        
        let timestamp = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        
        Ok(TegDomainStateRoot {
            domain_id,
            teg_root,
            nodes_root,
            effects_root,
            resources_root,
            intents_root,
            handlers_root,
            constraints_root,
            cross_domain_refs_root,
            sequence,
            timestamp,
        })
    }
}

impl SimpleSerialize for TegGraphStateRoot {}

impl Encode for TegGraphStateRoot {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.graph_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.nodes_root);
        bytes.extend_from_slice(&self.edges_root);
        bytes.extend_from_slice(&self.effects_root);
        bytes.extend_from_slice(&self.resources_root);
        bytes.extend_from_slice(&self.intents_root);
        bytes.extend_from_slice(&self.temporal_constraints_root);
        bytes.extend_from_slice(&self.combined_root);
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes
    }
}

impl Decode for TegGraphStateRoot {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        if bytes.len() < 32 + 32 + 32*7 + 8 {
            return Err(causality_types::serialization::DecodeError {
                message: "Insufficient data for TegGraphStateRoot".to_string(),
            });
        }
        
        let mut offset = 0;
        let graph_id = GraphId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;
        
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;
        
        let mut nodes_root = [0u8; 32];
        nodes_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut edges_root = [0u8; 32];
        edges_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut effects_root = [0u8; 32];
        effects_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut resources_root = [0u8; 32];
        resources_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut intents_root = [0u8; 32];
        intents_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut temporal_constraints_root = [0u8; 32];
        temporal_constraints_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let mut combined_root = [0u8; 32];
        combined_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let sequence = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        
        Ok(TegGraphStateRoot {
            graph_id,
            domain_id,
            nodes_root,
            edges_root,
            effects_root,
            resources_root,
            intents_root,
            temporal_constraints_root,
            combined_root,
            sequence,
        })
    }
}

impl SimpleSerialize for CrossDomainStateReference {}

impl Encode for CrossDomainStateReference {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.source_domain.as_ssz_bytes());
        bytes.extend_from_slice(&self.target_domain.as_ssz_bytes());
        bytes.extend_from_slice(&(self.target_entity_id.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.target_entity_id);
        bytes.extend_from_slice(&(self.entity_type.len() as u32).to_le_bytes());
        bytes.extend_from_slice(self.entity_type.as_bytes());
        bytes.extend_from_slice(&self.target_state_root);
        bytes.extend_from_slice(&(self.inclusion_proof.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.inclusion_proof);
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes
    }
}

impl Decode for CrossDomainStateReference {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        if bytes.len() < 32 + 32 + 4 + 4 + 32 + 4 + 8 {
            return Err(causality_types::serialization::DecodeError {
                message: "Insufficient data for CrossDomainStateReference".to_string(),
            });
        }
        
        let mut offset = 0;
        let source_domain = DomainId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;
        
        let target_domain = DomainId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;
        
        let entity_id_len = u32::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
        ]) as usize;
        offset += 4;
        
        let target_entity_id = bytes[offset..offset+entity_id_len].to_vec();
        offset += entity_id_len;
        
        let entity_type_len = u32::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
        ]) as usize;
        offset += 4;
        
        let entity_type = String::from_utf8(bytes[offset..offset+entity_type_len].to_vec())
            .map_err(|_| causality_types::serialization::DecodeError {
                message: "Invalid UTF-8 in entity_type".to_string(),
            })?;
        offset += entity_type_len;
        
        let mut target_state_root = [0u8; 32];
        target_state_root.copy_from_slice(&bytes[offset..offset+32]);
        offset += 32;
        
        let proof_len = u32::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
        ]) as usize;
        offset += 4;
        
        let inclusion_proof = bytes[offset..offset+proof_len].to_vec();
        offset += proof_len;
        
        let sequence = u64::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
            bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7]
        ]);
        
        Ok(CrossDomainStateReference {
            source_domain,
            target_domain,
            target_entity_id,
            entity_type,
            target_state_root,
            inclusion_proof,
            sequence,
        })
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::primitive::ids::GraphId;

    fn create_test_domain_id() -> DomainId {
        DomainId([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32])
    }

    fn create_test_graph_id() -> GraphId {
        GraphId([100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
                116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131])
    }

    #[test]
    fn test_teg_state_root_manager_creation() {
        let manager = TegStateRootManager::new();
        
        // Test with a non-existent domain
        let domain_id = create_test_domain_id();
        let result = manager.get_domain_state_root(&domain_id);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_domain_state_root_initialization() {
        let manager = TegStateRootManager::new();
        let domain_id = create_test_domain_id();
        
        // Initialize domain state root
        let domain_root = manager.initialize_domain_state_root(domain_id).unwrap();
        
        assert_eq!(domain_root.domain_id, domain_id);
        assert_eq!(domain_root.sequence, 0);
        assert_eq!(domain_root.teg_root, [0u8; 32]);
        assert_eq!(domain_root.nodes_root, [0u8; 32]);
        assert_eq!(domain_root.effects_root, [0u8; 32]);
        assert_eq!(domain_root.resources_root, [0u8; 32]);
        assert_eq!(domain_root.intents_root, [0u8; 32]);
        assert_eq!(domain_root.handlers_root, [0u8; 32]);
        assert_eq!(domain_root.constraints_root, [0u8; 32]);
        assert_eq!(domain_root.cross_domain_refs_root, [0u8; 32]);
        
        // Verify it can be retrieved
        let retrieved_root = manager.get_domain_state_root(&domain_id).unwrap().unwrap();
        assert_eq!(retrieved_root, domain_root);
    }

    #[test]
    fn test_domain_nodes_root_update() {
        let manager = TegStateRootManager::new();
        let domain_id = create_test_domain_id();
        
        // Initialize domain
        manager.initialize_domain_state_root(domain_id).unwrap();
        
        // Update nodes root
        let new_nodes_root = [42u8; 32];
        manager.update_domain_nodes_root(&domain_id, new_nodes_root).unwrap();
        
        // Verify update
        let updated_root = manager.get_domain_state_root(&domain_id).unwrap().unwrap();
        assert_eq!(updated_root.nodes_root, new_nodes_root);
        assert_eq!(updated_root.sequence, 1);
        assert_ne!(updated_root.teg_root, [0u8; 32]); // Should be recomputed
    }

    #[test]
    fn test_domain_effects_root_update() {
        let manager = TegStateRootManager::new();
        let domain_id = create_test_domain_id();
        
        // Initialize domain
        manager.initialize_domain_state_root(domain_id).unwrap();
        
        // Update effects root
        let new_effects_root = [100u8; 32];
        manager.update_domain_effects_root(&domain_id, new_effects_root).unwrap();
        
        // Verify update
        let updated_root = manager.get_domain_state_root(&domain_id).unwrap().unwrap();
        assert_eq!(updated_root.effects_root, new_effects_root);
        assert_eq!(updated_root.sequence, 1);
    }

    #[test]
    fn test_domain_batch_root_update() {
        let manager = TegStateRootManager::new();
        let domain_id = create_test_domain_id();
        
        // Initialize domain
        manager.initialize_domain_state_root(domain_id).unwrap();
        
        // Batch update multiple components
        let updates = vec![
            ("nodes".to_string(), [10u8; 32]),
            ("effects".to_string(), [20u8; 32]),
            ("resources".to_string(), [30u8; 32]),
            ("intents".to_string(), [40u8; 32]),
        ];
        
        manager.update_domain_roots_batch(&domain_id, updates).unwrap();
        
        // Verify updates
        let updated_root = manager.get_domain_state_root(&domain_id).unwrap().unwrap();
        assert_eq!(updated_root.nodes_root, [10u8; 32]);
        assert_eq!(updated_root.effects_root, [20u8; 32]);
        assert_eq!(updated_root.resources_root, [30u8; 32]);
        assert_eq!(updated_root.intents_root, [40u8; 32]);
        assert_eq!(updated_root.sequence, 1);
    }

    #[test]
    fn test_graph_state_root_initialization() {
        let manager = TegStateRootManager::new();
        let graph_id = create_test_graph_id();
        let domain_id = create_test_domain_id();
        
        // Initialize graph state root
        let graph_root = manager.initialize_graph_state_root(graph_id, domain_id).unwrap();
        
        assert_eq!(graph_root.graph_id, graph_id);
        assert_eq!(graph_root.domain_id, domain_id);
        assert_eq!(graph_root.sequence, 0);
        assert_eq!(graph_root.nodes_root, [0u8; 32]);
        assert_eq!(graph_root.edges_root, [0u8; 32]);
        assert_eq!(graph_root.effects_root, [0u8; 32]);
        assert_eq!(graph_root.resources_root, [0u8; 32]);
        assert_eq!(graph_root.intents_root, [0u8; 32]);
        assert_eq!(graph_root.temporal_constraints_root, [0u8; 32]);
        assert_eq!(graph_root.combined_root, [0u8; 32]);
    }

    #[test]
    fn test_cross_domain_reference_creation() {
        let manager = TegStateRootManager::new();
        let source_domain = create_test_domain_id();
        let mut target_domain_bytes = [0u8; 32];
        target_domain_bytes[0] = 99;
        let target_domain = DomainId(target_domain_bytes);
        
        // Initialize both domains
        manager.initialize_domain_state_root(source_domain).unwrap();
        manager.initialize_domain_state_root(target_domain).unwrap();
        
        // Create cross-domain reference
        let target_entity_id = [1, 2, 3, 4];
        let entity_type = "effect";
        let inclusion_proof = vec![10, 20, 30, 40];
        
        let ref_key = manager.create_cross_domain_reference(
            &source_domain,
            &target_domain,
            &target_entity_id,
            entity_type,
            inclusion_proof.clone()
        ).unwrap();
        
        // Verify reference key format
        assert!(ref_key.contains("cross-ref"));
        assert!(ref_key.contains(&hex::encode(&target_domain.0[..8])));
        assert!(ref_key.contains(&hex::encode(target_entity_id)));
    }

    #[test]
    fn test_teg_domain_state_root_serialization() {
        let domain_root = TegDomainStateRoot {
            domain_id: create_test_domain_id(),
            teg_root: [1u8; 32],
            nodes_root: [2u8; 32],
            effects_root: [3u8; 32],
            resources_root: [4u8; 32],
            intents_root: [5u8; 32],
            handlers_root: [6u8; 32],
            constraints_root: [7u8; 32],
            cross_domain_refs_root: [8u8; 32],
            sequence: 42,
            timestamp: 1234567890,
        };
        
        // Test serialization
        let serialized = domain_root.as_ssz_bytes();
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized = TegDomainStateRoot::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(deserialized, domain_root);
    }

    #[test]
    fn test_teg_graph_state_root_serialization() {
        let graph_root = TegGraphStateRoot {
            graph_id: create_test_graph_id(),
            domain_id: create_test_domain_id(),
            nodes_root: [10u8; 32],
            edges_root: [11u8; 32],
            effects_root: [12u8; 32],
            resources_root: [13u8; 32],
            intents_root: [14u8; 32],
            temporal_constraints_root: [15u8; 32],
            combined_root: [16u8; 32],
            sequence: 99,
        };
        
        // Test serialization
        let serialized = graph_root.as_ssz_bytes();
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized = TegGraphStateRoot::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(deserialized, graph_root);
    }

    #[test]
    fn test_cross_domain_state_reference_serialization() {
        let cross_ref = CrossDomainStateReference {
            source_domain: create_test_domain_id(),
            target_domain: DomainId([100u8; 32]),
            target_entity_id: vec![1, 2, 3, 4, 5],
            entity_type: "test_entity".to_string(),
            target_state_root: [50u8; 32],
            inclusion_proof: vec![10, 20, 30, 40, 50, 60],
            sequence: 777,
        };
        
        // Test serialization
        let serialized = cross_ref.as_ssz_bytes();
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized = CrossDomainStateReference::from_ssz_bytes(&serialized).unwrap();
        assert_eq!(deserialized, cross_ref);
    }

    #[test]
    fn test_combined_teg_root_computation() {
        let manager = TegStateRootManager::new();
        let domain_id = create_test_domain_id();
        
        // Initialize domain
        manager.initialize_domain_state_root(domain_id).unwrap();
        
        // Update multiple components with different values
        let updates = vec![
            ("nodes".to_string(), [1u8; 32]),
            ("effects".to_string(), [2u8; 32]),
            ("resources".to_string(), [3u8; 32]),
            ("intents".to_string(), [4u8; 32]),
            ("handlers".to_string(), [5u8; 32]),
            ("constraints".to_string(), [6u8; 32]),
            ("cross_domain_refs".to_string(), [7u8; 32]),
        ];
        
        manager.update_domain_roots_batch(&domain_id, updates).unwrap();
        
        // Get the updated root and verify TEG root is computed
        let updated_root = manager.get_domain_state_root(&domain_id).unwrap().unwrap();
        
        // TEG root should be different from all zeros and different from any component root
        assert_ne!(updated_root.teg_root, [0u8; 32]);
        assert_ne!(updated_root.teg_root, [1u8; 32]);
        assert_ne!(updated_root.teg_root, [2u8; 32]);
        
        // TEG root should be deterministic - same inputs should produce same output
        let second_domain = DomainId([50u8; 32]);
        manager.initialize_domain_state_root(second_domain).unwrap();
        
        let same_updates = vec![
            ("nodes".to_string(), [1u8; 32]),
            ("effects".to_string(), [2u8; 32]),
            ("resources".to_string(), [3u8; 32]),
            ("intents".to_string(), [4u8; 32]),
            ("handlers".to_string(), [5u8; 32]),
            ("constraints".to_string(), [6u8; 32]),
            ("cross_domain_refs".to_string(), [7u8; 32]),
        ];
        
        manager.update_domain_roots_batch(&second_domain, same_updates).unwrap();
        let second_root = manager.get_domain_state_root(&second_domain).unwrap().unwrap();
        
        // TEG roots should be the same for same component values
        assert_eq!(updated_root.teg_root, second_root.teg_root);
    }
} 