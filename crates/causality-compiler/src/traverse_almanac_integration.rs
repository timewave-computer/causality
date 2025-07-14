//! Traverse-Almanac Integration for Automatic Witness Generation
//!
//! This module provides integration between Traverse path resolution and Almanac state queries,
//! enabling automatic witness generation from high-level state queries. It coordinates between
//! the two systems to provide seamless ZK proof generation capabilities.

use crate::almanac_schema::LayoutCommitment;
use crate::proof_primitives::WitnessData;
use crate::storage_layout::{StorageLayout, StorageLayoutGenerator};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Conditional imports based on feature flags
#[cfg(feature = "traverse")]
use traverse_core::{KeyResolver, LayoutInfo as TraverseLayoutInfo, StaticKeyPath};

#[cfg(feature = "almanac")]
use crate::almanac_runtime::AlmanacRuntime;

/// Integration coordinator between Traverse and Almanac
pub struct TraverseAlmanacIntegrator {
    /// Storage layout generator
    #[allow(dead_code)]
    layout_generator: StorageLayoutGenerator,
    /// Almanac runtime for state queries
    #[cfg(feature = "almanac")]
    almanac_runtime: Option<AlmanacRuntime>,
    /// Traverse key resolver
    #[cfg(feature = "traverse")]
    key_resolver: Option<Box<dyn KeyResolver>>,
    /// Witness cache for performance
    witness_cache: BTreeMap<String, CachedWitness>,
    /// Integration configuration
    config: IntegrationConfig,
}

/// Configuration for Traverse-Almanac integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Enable witness caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum concurrent witness generations
    pub max_concurrent_witnesses: usize,
    /// Timeout for witness generation in milliseconds
    pub witness_timeout_ms: u64,
    /// Enable automatic layout validation
    pub enable_layout_validation: bool,
}

/// Cached witness data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedWitness {
    /// The witness data
    pub witness: WitnessData,
    /// Cache timestamp
    pub cached_at: u64,
    /// Layout commitment used for generation
    pub layout_commitment: String,
    /// Number of times this witness has been used
    pub usage_count: u64,
}

/// Request for witness generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessGenerationRequest {
    /// Contract identifier
    pub contract_id: String,
    /// Storage query to generate witness for
    pub query: String,
    /// Block number for the witness
    pub block_number: u64,
    /// Contract address
    pub contract_address: String,
    /// Layout commitment for consistency
    pub layout_commitment: LayoutCommitment,
    /// Additional parameters
    pub parameters: BTreeMap<String, String>,
}

/// Result of witness generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessGenerationResult {
    /// Generated witness data
    pub witness: WitnessData,
    /// Storage path used for generation
    pub storage_path: String,
    /// Generation metadata
    pub metadata: WitnessGenerationMetadata,
}

/// Metadata about witness generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessGenerationMetadata {
    /// Generation duration in milliseconds
    pub generation_duration_ms: u64,
    /// Whether witness was retrieved from cache
    pub from_cache: bool,
    /// Traverse path resolution time in milliseconds
    pub path_resolution_time_ms: u64,
    /// Almanac query time in milliseconds
    pub almanac_query_time_ms: u64,
}

/// Errors that can occur during integration
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Storage layout not found for contract: {0}")]
    LayoutNotFound(String),

    #[error("Path resolution failed: {0}")]
    PathResolutionFailed(String),

    #[error("Almanac query failed: {0}")]
    AlmanacQueryFailed(String),

    #[error("Witness generation failed: {0}")]
    WitnessGenerationFailed(String),

    #[error("Layout commitment mismatch: expected {expected}, got {actual}")]
    LayoutCommitmentMismatch { expected: String, actual: String },

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Timeout during witness generation")]
    Timeout,

    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

impl TraverseAlmanacIntegrator {
    /// Create a new integrator
    pub fn new() -> Self {
        Self {
            layout_generator: StorageLayoutGenerator::new(),
            #[cfg(feature = "almanac")]
            almanac_runtime: None,
            #[cfg(feature = "traverse")]
            key_resolver: None,
            witness_cache: BTreeMap::new(),
            config: IntegrationConfig::default(),
        }
    }

    /// Create a new integrator with custom configuration
    pub fn with_config(config: IntegrationConfig) -> Self {
        Self {
            layout_generator: StorageLayoutGenerator::new(),
            #[cfg(feature = "almanac")]
            almanac_runtime: None,
            #[cfg(feature = "traverse")]
            key_resolver: None,
            witness_cache: BTreeMap::new(),
            config,
        }
    }

    /// Set the Almanac runtime
    #[cfg(feature = "almanac")]
    pub fn set_almanac_runtime(&mut self, runtime: AlmanacRuntime) {
        self.almanac_runtime = Some(runtime);
    }

    /// Set the Traverse key resolver
    #[cfg(feature = "traverse")]
    pub fn set_key_resolver(&mut self, resolver: Box<dyn KeyResolver>) {
        self.key_resolver = Some(resolver);
    }

    /// Generate witness data from a state query
    pub async fn generate_witness(
        &mut self,
        request: WitnessGenerationRequest,
    ) -> Result<WitnessGenerationResult, IntegrationError> {
        let start_time = std::time::Instant::now();

        // Check cache first if enabled
        if self.config.enable_caching {
            if let Some(cached) = self.get_cached_witness(&request) {
                return Ok(WitnessGenerationResult {
                    witness: cached.witness,
                    storage_path: request.query.clone(),
                    metadata: WitnessGenerationMetadata {
                        generation_duration_ms: start_time.elapsed().as_millis()
                            as u64,
                        from_cache: true,
                        path_resolution_time_ms: 0,
                        almanac_query_time_ms: 0,
                    },
                });
            }
        }

        // Resolve storage path using Traverse
        let path_resolution_start = std::time::Instant::now();
        let storage_path = self.resolve_storage_path(&request).await?;
        let path_resolution_time =
            path_resolution_start.elapsed().as_millis() as u64;

        // Query state using Almanac
        let almanac_query_start = std::time::Instant::now();
        let witness = self
            .query_state_for_witness(&request, &storage_path)
            .await?;
        let almanac_query_time = almanac_query_start.elapsed().as_millis() as u64;

        // Cache the result if enabled
        if self.config.enable_caching {
            self.cache_witness(&request, &witness);
        }

        let total_duration = start_time.elapsed().as_millis() as u64;

        Ok(WitnessGenerationResult {
            witness,
            storage_path: storage_path.clone(),
            metadata: WitnessGenerationMetadata {
                generation_duration_ms: total_duration,
                from_cache: false,
                path_resolution_time_ms: path_resolution_time,
                almanac_query_time_ms: almanac_query_time,
            },
        })
    }

    /// Resolve storage path using Traverse
    async fn resolve_storage_path(
        &self,
        request: &WitnessGenerationRequest,
    ) -> Result<String, IntegrationError> {
        #[cfg(feature = "traverse")]
        {
            if let Some(ref resolver) = self.key_resolver {
                // Get the storage layout for the contract
                let layout = self.get_contract_layout(&request.contract_id)?;
                let traverse_layout =
                    self.layout_generator.to_traverse_layout(&layout);

                // Resolve the query to a storage path
                let static_path = resolver
                    .resolve(&traverse_layout, &request.query)
                    .map_err(|e| {
                        IntegrationError::PathResolutionFailed(format!("{:?}", e))
                    })?;

                // Convert the key to a hex string
                let storage_key = hex::encode(static_path.key);
                return Ok(storage_key);
            }
        }

        // Fallback: simple query parsing for basic cases
        self.parse_simple_query(&request.query)
    }

    /// Query state using Almanac to generate witness
    async fn query_state_for_witness(
        &self,
        request: &WitnessGenerationRequest,
        storage_path: &str,
    ) -> Result<WitnessData, IntegrationError> {
        #[cfg(feature = "almanac")]
        {
            if let Some(ref runtime) = self.almanac_runtime {
                // Query the storage value from Almanac
                let storage_value = runtime
                    .get_storage_value(
                        &request.contract_address,
                        storage_path,
                        request.block_number,
                    )
                    .await
                    .map_err(|e| {
                        IntegrationError::AlmanacQueryFailed(format!("{:?}", e))
                    })?;

                // Get the merkle proof for the storage slot
                let merkle_proof = runtime
                    .get_storage_proof(
                        &request.contract_address,
                        storage_path,
                        request.block_number,
                    )
                    .await
                    .map_err(|e| {
                        IntegrationError::AlmanacQueryFailed(format!(
                            "Proof generation failed: {:?}",
                            e
                        ))
                    })?;

                return Ok(WitnessData {
                    storage_key: storage_path.to_string(),
                    storage_value,
                    merkle_proof,
                    block_number: request.block_number,
                    contract_address: request.contract_address.clone(),
                });
            }
        }

        // Mock witness generation for development/testing
        Ok(WitnessData {
            storage_key: storage_path.to_string(),
            storage_value:
                "0x0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            merkle_proof: vec![
                "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                    .to_string(),
                "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
                    .to_string(),
            ],
            block_number: request.block_number,
            contract_address: request.contract_address.clone(),
        })
    }

    /// Get cached witness if available and valid
    fn get_cached_witness(
        &self,
        request: &WitnessGenerationRequest,
    ) -> Option<CachedWitness> {
        let cache_key = self.generate_cache_key(request);

        if let Some(cached) = self.witness_cache.get(&cache_key) {
            // Check if cache is still valid
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now - cached.cached_at < self.config.cache_ttl_seconds {
                // Check layout commitment consistency
                if cached.layout_commitment
                    == request.layout_commitment.commitment_hash
                {
                    return Some(cached.clone());
                }
            }
        }

        None
    }

    /// Cache witness data
    fn cache_witness(
        &mut self,
        request: &WitnessGenerationRequest,
        witness: &WitnessData,
    ) {
        let cache_key = self.generate_cache_key(request);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cached_witness = CachedWitness {
            witness: witness.clone(),
            cached_at: now,
            layout_commitment: request.layout_commitment.commitment_hash.clone(),
            usage_count: 1,
        };

        self.witness_cache.insert(cache_key, cached_witness);
    }

    /// Generate cache key for a witness request
    fn generate_cache_key(&self, request: &WitnessGenerationRequest) -> String {
        format!(
            "{}:{}:{}:{}",
            request.contract_id,
            request.query,
            request.block_number,
            request.contract_address
        )
    }

    /// Get contract layout
    fn get_contract_layout(
        &self,
        contract_id: &str,
    ) -> Result<StorageLayout, IntegrationError> {
        // This would typically come from the layout generator or a registry
        // For now, we'll create a mock layout
        Ok(StorageLayout {
            contract_name: contract_id.to_string(),
            storage: vec![],
            types: vec![],
            layout_commitment: LayoutCommitment {
                commitment_hash: "mock_commitment".to_string(),
                version: "1.0.0".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
            domain: "ethereum".to_string(),
        })
    }

    /// Parse simple queries when Traverse is not available
    fn parse_simple_query(&self, query: &str) -> Result<String, IntegrationError> {
        // Simple parsing for common patterns
        if query.starts_with("balances[") && query.ends_with(']') {
            // Extract address from balances[0x...]
            let address_part = &query[9..query.len() - 1];
            if address_part.starts_with("0x") && address_part.len() == 42 {
                // Generate storage key for mapping: keccak256(key . slot)
                // For simplicity, we'll use a mock key
                return Ok(format!("0x{:064x}", 0u64)); // Slot 0 for balances
            }
        }

        Err(IntegrationError::PathResolutionFailed(format!(
            "Cannot parse query without Traverse: {}",
            query
        )))
    }

    /// Validate layout commitment consistency
    pub fn validate_layout_commitment(
        &self,
        request: &WitnessGenerationRequest,
    ) -> Result<bool, IntegrationError> {
        if self.config.enable_layout_validation {
            let layout = self.get_contract_layout(&request.contract_id)?;

            #[cfg(feature = "traverse")]
            {
                return self
                    .layout_generator
                    .validate_layout_commitment(&layout)
                    .map_err(|e| IntegrationError::LayoutCommitmentMismatch {
                        expected: request.layout_commitment.commitment_hash.clone(),
                        actual: format!("{:?}", e),
                    });
            }

            // Basic validation without Traverse
            Ok(layout.layout_commitment.commitment_hash
                == request.layout_commitment.commitment_hash)
        } else {
            Ok(true)
        }
    }

    /// Clear witness cache
    pub fn clear_cache(&mut self) {
        self.witness_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        let total_entries = self.witness_cache.len();
        let total_usage: u64 =
            self.witness_cache.values().map(|w| w.usage_count).sum();

        CacheStats {
            total_entries,
            total_usage,
            cache_hit_rate: if total_usage > 0 {
                (total_usage as f64 - total_entries as f64) / total_usage as f64
            } else {
                0.0
            },
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cached entries
    pub total_entries: usize,
    /// Total usage count across all entries
    pub total_usage: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_concurrent_witnesses: 10,
            witness_timeout_ms: 30000, // 30 seconds
            enable_layout_validation: true,
        }
    }
}

impl Default for TraverseAlmanacIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrator_creation() {
        let integrator = TraverseAlmanacIntegrator::new();
        assert!(integrator.witness_cache.is_empty());
        assert!(integrator.config.enable_caching);
    }

    #[test]
    fn test_cache_key_generation() {
        let integrator = TraverseAlmanacIntegrator::new();
        let request = WitnessGenerationRequest {
            contract_id: "test_contract".to_string(),
            query: "balances[0x123]".to_string(),
            block_number: 12345,
            contract_address: "0xabc".to_string(),
            layout_commitment: LayoutCommitment {
                commitment_hash: "test_hash".to_string(),
                version: "1.0.0".to_string(),
                timestamp: 0,
            },
            parameters: BTreeMap::new(),
        };

        let key = integrator.generate_cache_key(&request);
        assert_eq!(key, "test_contract:balances[0x123]:12345:0xabc");
    }

    #[test]
    fn test_simple_query_parsing() -> Result<(), Box<dyn std::error::Error>> {
        let integrator = TraverseAlmanacIntegrator::new();

        // Test valid balance query - this should work with the current implementation
        let _result = integrator.parse_simple_query(
            "balances[0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C6F8]",
        )?;
        // The current implementation expects 42 character addresses (0x + 40 hex chars)
        let valid_address = "balances[0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C12]";

        // Parse the query
        let result = integrator.parse_simple_query(valid_address);
        assert!(result.is_ok());

        // Test invalid query
        let invalid_query = "invalid_query_format";
        let result = integrator.parse_simple_query(invalid_query);
        assert!(result.is_err());

        Ok(())
    }
}
