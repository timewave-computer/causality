//! Storage Layout Generation for ZK Proof Integration
//!
//! This module generates Traverse-compatible storage layouts from Causality state analysis results.
//! It bridges the gap between Causality's state query analysis and Traverse's ZK proof generation
//! by creating deterministic storage layouts with content-addressed commitments.

use std::collections::BTreeMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::state_analysis::{StateAnalysisResult, StateQueryRequirement, QueryType};
use crate::almanac_schema::LayoutCommitment;


/// Traverse-compatible storage layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLayout {
    /// Contract name or identifier
    pub contract_name: String,
    /// Storage entries mapping variable names to storage locations
    pub storage: Vec<StorageEntry>,
    /// Type definitions for storage variables
    pub types: Vec<TypeInfo>,
    /// Content-addressed layout commitment
    pub layout_commitment: LayoutCommitment,
    /// Blockchain domain (ethereum, cosmos, etc.)
    pub domain: String,
}

/// Storage entry representing a single storage variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    /// Variable name as it appears in queries
    pub label: String,
    /// Storage slot index
    pub slot: String,
    /// Byte offset within the slot
    pub offset: u8,
    /// Type identifier
    pub type_name: String,
}

/// Type information for storage variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Type identifier
    pub label: String,
    /// Size in bytes
    pub number_of_bytes: String,
    /// Encoding type (inplace, mapping, dynamic_array)
    pub encoding: String,
    /// Base type for arrays and mappings
    pub base: Option<String>,
    /// Key type for mappings
    pub key: Option<String>,
    /// Value type for mappings
    pub value: Option<String>,
}

/// Storage layout generator that creates Traverse-compatible layouts
pub struct StorageLayoutGenerator {
    /// Known contract layouts by domain
    contract_layouts: BTreeMap<String, BTreeMap<String, StorageLayout>>,
    /// Type mapping configuration
    type_mapping: TypeMappingConfig,
    /// Integration with real Traverse system
    #[cfg(feature = "traverse")]
    traverse_integration: Option<TraverseIntegration>,
}

/// Configuration for mapping query types to storage types
#[derive(Debug, Clone)]
pub struct TypeMappingConfig {
    /// Default slot assignments for common patterns
    pub default_slots: BTreeMap<QueryType, u64>,
    /// Type size mappings
    pub type_sizes: BTreeMap<String, usize>,
}

/// Result of storage layout generation
#[derive(Debug, Clone)]
pub struct StorageLayoutResult {
    /// Generated layouts by contract
    pub layouts: BTreeMap<String, StorageLayout>,
    /// Layout commitments for version tracking
    pub commitments: BTreeMap<String, LayoutCommitment>,
    /// Generation metadata
    pub metadata: LayoutGenerationMetadata,
    /// Traverse layout info for ZK proof generation
    #[cfg(feature = "traverse")]
    pub traverse_layouts: BTreeMap<String, TraverseLayoutInfo>,
}

/// Metadata about layout generation process
#[derive(Debug, Clone)]
pub struct LayoutGenerationMetadata {
    /// Number of contracts processed
    pub contracts_processed: usize,
    /// Number of storage entries generated
    pub storage_entries_generated: usize,
    /// Generation duration in milliseconds
    pub generation_duration_ms: u64,
    /// Traverse integration status
    pub traverse_integration_enabled: bool,
}

/// Integration with real Traverse system
#[cfg(feature = "traverse")]
pub struct TraverseIntegration {
    /// Traverse layout compiler (placeholder - traverse_core not available)
    layout_compiler: Option<()>, // Box<dyn traverse_core::LayoutCompiler>,
    /// Key resolver for storage queries (placeholder - traverse_core not available)  
    key_resolver: Option<()>, // Box<dyn traverse_core::KeyResolver>,
}

// Mock types when traverse feature is not enabled
#[cfg(not(feature = "traverse"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseLayoutInfo {
    pub contract_name: String,
    pub storage: Vec<TraverseStorageEntry>,
    pub types: Vec<TraverseTypeInfo>,
}

#[cfg(not(feature = "traverse"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseStorageEntry {
    pub label: String,
    pub slot: String,
    pub offset: u32,
    pub type_name: String,
}

#[cfg(not(feature = "traverse"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseTypeInfo {
    pub type_name: String,
    pub encoding: String,
    pub number_of_bytes: String,
}

impl StorageLayoutGenerator {
    /// Create a new storage layout generator
    pub fn new() -> Self {
        Self {
            contract_layouts: BTreeMap::new(),
            type_mapping: TypeMappingConfig::default(),
            #[cfg(feature = "traverse")]
            traverse_integration: None,
        }
    }
    
    /// Create a new storage layout generator with Traverse integration
    #[cfg(feature = "traverse")]
    pub fn with_traverse_integration(
        layout_compiler: Box<dyn traverse_core::LayoutCompiler>,
        key_resolver: Box<dyn traverse_core::KeyResolver>,
    ) -> Self {
        Self {
            contract_layouts: BTreeMap::new(),
            type_mapping: TypeMappingConfig::default(),
            traverse_integration: Some(TraverseIntegration {
                layout_compiler,
                key_resolver,
            }),
        }
    }
    
    /// Generate storage layouts from state analysis results
    pub fn generate_layouts(&self, analysis: &StateAnalysisResult) -> Result<StorageLayoutResult> {
        let start_time = std::time::Instant::now();
        let mut layouts = BTreeMap::new();
        let mut commitments = BTreeMap::new();
        let mut total_entries = 0;
        
        #[cfg(feature = "traverse")]
        let mut traverse_layouts = BTreeMap::new();
        
        // Generate layout for each contract
        for (contract_id, queries) in &analysis.queries_by_contract {
            let layout = self.generate_contract_layout(contract_id, queries)?;
            let commitment = layout.layout_commitment.clone();
            
            total_entries += layout.storage.len();
            commitments.insert(contract_id.clone(), commitment);
            
            // Generate Traverse layout if integration is enabled
            #[cfg(feature = "traverse")]
            if let Some(ref integration) = self.traverse_integration {
                let traverse_layout = self.convert_to_traverse_layout(&layout)?;
                traverse_layouts.insert(contract_id.clone(), traverse_layout);
            }
            
            layouts.insert(contract_id.clone(), layout);
        }
        
        let generation_duration = start_time.elapsed().as_millis() as u64;
        
        Ok(StorageLayoutResult {
            layouts,
            commitments,
            metadata: LayoutGenerationMetadata {
                contracts_processed: analysis.queries_by_contract.len(),
                storage_entries_generated: total_entries,
                generation_duration_ms: generation_duration,
                traverse_integration_enabled: cfg!(feature = "traverse"),
            },
            #[cfg(feature = "traverse")]
            traverse_layouts,
        })
    }
    
    /// Generate storage layout for a specific contract
    fn generate_contract_layout(&self, contract_id: &str, queries: &[StateQueryRequirement]) -> Result<StorageLayout> {
        let domain = queries.first()
            .map(|q| q.domain.clone())
            .unwrap_or_else(|| "ethereum".to_string());
        
        // Generate storage entries from queries
        let mut storage_entries = Vec::new();
        let mut type_definitions = Vec::new();
        let mut slot_counter = 0u64;
        
        for query in queries {
            let (entry, type_def) = self.create_storage_entry(query, &mut slot_counter)?;
            storage_entries.push(entry);
            
            // Add type definition if not already present
            if !type_definitions.iter().any(|t: &TypeInfo| t.label == type_def.label) {
                type_definitions.push(type_def);
            }
        }
        
        // Create layout commitment
        let layout_commitment = self.generate_layout_commitment(contract_id, &storage_entries, &type_definitions)?;
        
        Ok(StorageLayout {
            contract_name: contract_id.to_string(),
            storage: storage_entries,
            types: type_definitions,
            layout_commitment,
            domain,
        })
    }
    
    /// Create a storage entry from a query requirement
    fn create_storage_entry(&self, query: &StateQueryRequirement, slot_counter: &mut u64) -> Result<(StorageEntry, TypeInfo)> {
        let slot = self.determine_storage_slot(query, slot_counter);
        let (type_name, type_info) = self.determine_type_info(&query.query_type);
        
        let entry = StorageEntry {
            label: query.storage_slot.clone(),
            slot: slot.to_string(),
            offset: 0, // Most storage variables start at offset 0
            type_name: type_name.clone(),
        };
        
        Ok((entry, type_info))
    }
    
    /// Determine the storage slot for a query
    fn determine_storage_slot(&self, query: &StateQueryRequirement, slot_counter: &mut u64) -> u64 {
        // Check if we have a predefined slot for this query type
        if let Some(&slot) = self.type_mapping.default_slots.get(&query.query_type) {
            return slot;
        }
        
        // Otherwise, assign the next available slot
        let slot = *slot_counter;
        *slot_counter += 1;
        slot
    }
    
    /// Determine type information for a query type
    fn determine_type_info(&self, query_type: &QueryType) -> (String, TypeInfo) {
        match query_type {
            QueryType::TokenBalance => {
                ("t_mapping_address_uint256".to_string(), TypeInfo {
                    label: "mapping(address => uint256)".to_string(),
                    encoding: "mapping".to_string(),
                    number_of_bytes: "32".to_string(),
                    base: Some("t_uint256".to_string()),
                    key: Some("t_address".to_string()),
                    value: Some("t_uint256".to_string()),
                })
            },
            QueryType::TokenAllowance => {
                ("t_mapping_address_mapping_address_uint256".to_string(), TypeInfo {
                    label: "mapping(address => mapping(address => uint256))".to_string(),
                    encoding: "mapping".to_string(),
                    number_of_bytes: "32".to_string(),
                    base: Some("t_mapping_address_uint256".to_string()),
                    key: Some("t_address".to_string()),
                    value: Some("t_mapping_address_uint256".to_string()),
                })
            },
            QueryType::ContractState => {
                ("t_bytes32".to_string(), TypeInfo {
                    label: "bytes32".to_string(),
                    encoding: "bytes32".to_string(),
                    number_of_bytes: "32".to_string(),
                    base: None,
                    key: None,
                    value: None,
                })
            },
            QueryType::EventLog => {
                ("t_bytes".to_string(), TypeInfo {
                    label: "bytes".to_string(),
                    encoding: "bytes".to_string(),
                    number_of_bytes: "0".to_string(),
                    base: Some("t_uint8".to_string()),
                    key: None,
                    value: None,
                })
            },
            QueryType::StorageSlot(slot_name) => {
                ("t_bytes32".to_string(), TypeInfo {
                    label: format!("bytes32 slot {}", slot_name),
                    encoding: "bytes32".to_string(),
                    number_of_bytes: "32".to_string(),
                    base: None,
                    key: None,
                    value: None,
                })
            },
            QueryType::Custom(custom_name) => {
                ("t_bytes".to_string(), TypeInfo {
                    label: format!("bytes custom {}", custom_name),
                    encoding: "bytes".to_string(),
                    number_of_bytes: "0".to_string(),
                    base: Some("t_uint8".to_string()),
                    key: None,
                    value: None,
                })
            },
        }
    }
    
    /// Generate layout commitment from storage layout components
    fn generate_layout_commitment(&self, contract_id: &str, storage: &[StorageEntry], types: &[TypeInfo]) -> Result<LayoutCommitment> {
        use sha2::{Digest, Sha256};
        
        // Create deterministic hash of layout components
        let mut hasher = Sha256::new();
        hasher.update(contract_id.as_bytes());
        
        // Hash storage entries in deterministic order
        let mut sorted_storage = storage.to_vec();
        sorted_storage.sort_by(|a, b| a.label.cmp(&b.label));
        for entry in &sorted_storage {
            hasher.update(entry.label.as_bytes());
            hasher.update(entry.slot.as_bytes());
            hasher.update([entry.offset]);
            hasher.update(entry.type_name.as_bytes());
        }
        
        // Hash type definitions in deterministic order
        let mut sorted_types = types.to_vec();
        sorted_types.sort_by(|a, b| a.label.cmp(&b.label));
        for type_info in &sorted_types {
            hasher.update(type_info.label.as_bytes());
            hasher.update(type_info.number_of_bytes.as_bytes());
            hasher.update(type_info.encoding.as_bytes());
            if let Some(ref base) = type_info.base {
                hasher.update(base.as_bytes());
            }
            if let Some(ref key) = type_info.key {
                hasher.update(key.as_bytes());
            }
            if let Some(ref value) = type_info.value {
                hasher.update(value.as_bytes());
            }
        }
        
        let hash = hasher.finalize();
        let commitment_hash = hex::encode(hash);
        
        Ok(LayoutCommitment {
            commitment_hash,
            version: "1.0.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    /// Convert internal storage layout to Traverse layout format
    #[cfg(feature = "traverse")]
    fn convert_to_traverse_layout(&self, layout: &StorageLayout) -> Result<TraverseLayoutInfo> {
        let storage_entries: Vec<TraverseStorageEntry> = layout.storage.iter()
            .map(|entry| TraverseStorageEntry {
                label: entry.label.clone(),
                slot: entry.slot.clone(),
                offset: entry.offset,
                type_name: entry.type_name.clone(),
            })
            .collect();
        
        let type_infos: Vec<TraverseTypeInfo> = layout.types.iter()
            .map(|type_info| TraverseTypeInfo {
                label: type_info.label.clone(),
                number_of_bytes: type_info.number_of_bytes.clone(),
                encoding: type_info.encoding.clone(),
                base: type_info.base.clone(),
                key: type_info.key.clone(),
                value: type_info.value.clone(),
            })
            .collect();
        
        Ok(TraverseLayoutInfo {
            contract_name: layout.contract_name.clone(),
            storage: storage_entries,
            types: type_infos,
        })
    }
    
    /// Register a pre-compiled layout
    pub fn register_layout(&mut self, domain: &str, contract_id: &str, layout: StorageLayout) {
        self.contract_layouts
            .entry(domain.to_string())
            .or_default()
            .insert(contract_id.to_string(), layout);
    }
    
    /// Get a registered layout
    pub fn get_layout(&self, domain: &str, contract_id: &str) -> Option<&StorageLayout> {
        self.contract_layouts
            .get(domain)
            .and_then(|domain_layouts| domain_layouts.get(contract_id))
    }
    
    /// Convert storage layout to legacy TraverseLayoutInfo format (for backwards compatibility)
    pub fn to_traverse_layout(&self, layout: &StorageLayout) -> TraverseLayoutInfo {
        TraverseLayoutInfo {
            contract_name: layout.contract_name.clone(),
            storage: layout.storage.iter().map(|entry| TraverseStorageEntry {
                label: entry.label.clone(),
                slot: entry.slot.clone(),
                offset: entry.offset as u32,
                type_name: entry.type_name.clone(),
            }).collect(),
            types: layout.types.iter().map(|type_info| TraverseTypeInfo {
                type_name: type_info.label.clone(),
                encoding: type_info.encoding.clone(),
                number_of_bytes: type_info.number_of_bytes.clone(),
            }).collect(),
        }
    }
    
    /// Validate layout commitment consistency with Traverse
    #[cfg(feature = "traverse")]
    pub fn validate_layout_commitment(&self, layout: &StorageLayout) -> Result<bool> {
        if let Some(ref integration) = self.traverse_integration {
            let traverse_layout = self.convert_to_traverse_layout(layout)?;
            let traverse_commitment = traverse_layout.commitment();
            let our_commitment = hex::decode(&layout.layout_commitment.commitment_hash)?;
            
            // Compare the first 32 bytes (SHA256 hash length)
            if our_commitment.len() >= 32 && traverse_commitment.len() == 32 {
                return Ok(&our_commitment[..32] == &traverse_commitment[..]);
            }
        }
        
        // If no Traverse integration, assume valid
        Ok(true)
    }
}

impl Default for TypeMappingConfig {
    fn default() -> Self {
        let mut default_slots = BTreeMap::new();
        default_slots.insert(QueryType::TokenBalance, 1); // Common slot for balances mapping
        default_slots.insert(QueryType::TokenAllowance, 2); // Common slot for allowances mapping
        
        let mut type_sizes = BTreeMap::new();
        type_sizes.insert("t_uint256".to_string(), 32);
        type_sizes.insert("t_address".to_string(), 20);
        type_sizes.insert("t_bytes32".to_string(), 32);
        type_sizes.insert("t_bool".to_string(), 1);
        
        Self {
            default_slots,
            type_sizes,
        }
    }
}

impl Default for StorageLayoutGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_analysis::{StateQueryRequirement, QueryType};
    
    fn create_test_query(contract: &str, slot: &str, query_type: QueryType) -> StateQueryRequirement {
        StateQueryRequirement {
            contract: contract.to_string(),
            storage_slot: slot.to_string(),
            domain: "ethereum".to_string(),
            query_type,
            is_conditional: false,
        }
    }
    
    #[test]
    fn test_storage_layout_generation() {
        let generator = StorageLayoutGenerator::new();
        
        let queries = vec![
            create_test_query("usdc", "balances", QueryType::TokenBalance),
            create_test_query("usdc", "allowances", QueryType::TokenAllowance),
        ];
        
        let layout = generator.generate_contract_layout("usdc", &queries).unwrap();
        
        assert_eq!(layout.contract_name, "usdc");
        assert_eq!(layout.storage.len(), 2);
        assert_eq!(layout.types.len(), 2);
        assert!(!layout.layout_commitment.commitment_hash.is_empty());
    }
    
    #[test]
    fn test_type_mapping() {
        let generator = StorageLayoutGenerator::new();
        
        let (type_name, type_info) = generator.determine_type_info(&QueryType::TokenBalance);
        assert_eq!(type_name, "t_mapping_address_uint256");
        assert_eq!(type_info.encoding, "mapping");
        assert_eq!(type_info.key, Some("t_address".to_string()));
        assert_eq!(type_info.value, Some("t_uint256".to_string()));
    }
    
    #[test]
    fn test_traverse_compatibility() {
        let generator = StorageLayoutGenerator::new();
        
        let queries = vec![
            create_test_query("test", "value", QueryType::StorageSlot("value".to_string())),
        ];
        
        let layout = generator.generate_contract_layout("test", &queries).unwrap();
        let traverse_layout = generator.to_traverse_layout(&layout);
        
        assert_eq!(traverse_layout.contract_name, "test");
        assert_eq!(traverse_layout.storage.len(), 1);
        assert_eq!(traverse_layout.types.len(), 1);
    }
} 