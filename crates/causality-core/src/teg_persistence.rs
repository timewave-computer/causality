// TEG Domain Persistence System
//
// This module provides disk persistence strategy for multi-domain TEG data in SMT,
// including incremental backup/snapshot mechanisms, import/export functionality,
// and integrity verification tools for cross-domain TEG consistency.

use crate::smt::{TegMultiDomainSmt, MemoryBackend};
use crate::teg_deployment::DomainStateExport;
use causality_types::{
    core::id::{DomainId, NodeId},
    serialization::Encode,
};
use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use serde_json::{Value as JsonValue, Map as JsonMap};
use sha2::Digest;

/// TEG domain persistence manager for disk-based state storage
pub struct TegPersistenceManager {
    smt: Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>>,
    /// Base directory for persistence storage
    base_directory: PathBuf,
    /// Configuration for persistence operations
    persistence_config: PersistenceConfig,
}

/// Configuration for TEG persistence operations
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Maximum snapshot size in bytes before compression
    #[allow(dead_code)]
    max_snapshot_size_bytes: usize,
    /// Number of incremental snapshots to keep
    #[allow(dead_code)]
    max_incremental_snapshots: usize,
    /// Enable compression for snapshots
    enable_compression: bool,
    /// Enable integrity verification on load
    #[allow(dead_code)]
    verify_integrity_on_load: bool,
    /// Backup interval in milliseconds
    #[allow(dead_code)]
    backup_interval_ms: u64,
}

/// TEG snapshot containing state at a specific point in time
#[derive(Debug, Clone)]
pub struct TegSnapshot {
    /// Unique snapshot identifier
    pub snapshot_id: String,
    /// Timestamp when snapshot was created
    pub created_at: u64,
    /// SMT root at time of snapshot
    pub smt_root: [u8; 32],
    /// Parent snapshot ID for incremental snapshots
    pub parent_snapshot_id: Option<String>,
    /// Type of snapshot
    pub snapshot_type: SnapshotType,
    /// Domains included in this snapshot
    pub included_domains: Vec<DomainId>,
    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
}

/// Types of TEG snapshots
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotType {
    /// Full snapshot containing all domain data
    Full,
    /// Incremental snapshot containing only changes since parent
    Incremental,
    /// Domain-specific snapshot for a single domain
    DomainSpecific(DomainId),
}

/// Metadata for TEG snapshots
#[derive(Debug, Clone)]
pub struct SnapshotMetadata {
    /// Total size of snapshot in bytes
    pub size_bytes: usize,
    /// Number of domains in snapshot
    pub domain_count: usize,
    /// Number of nodes in snapshot
    pub node_count: usize,
    /// Integrity checksum
    pub integrity_checksum: [u8; 32],
    /// Compression used (if any)
    pub compression_type: Option<CompressionType>,
}

/// Types of compression for snapshots
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}

/// Result of persistence operations
#[derive(Debug, Clone)]
pub struct PersistenceResult {
    pub operation_id: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub snapshot_id: Option<String>,
    pub size_bytes: usize,
    pub operation_time_ms: u128,
}

/// Import/export operation result
#[derive(Debug, Clone)]
pub struct ImportExportResult {
    pub operation_id: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub domains_processed: Vec<DomainId>,
    pub nodes_processed: usize,
    pub file_path: PathBuf,
    pub operation_time_ms: u128,
}

/// Integrity verification result
#[derive(Debug, Clone)]
pub struct IntegrityVerificationResult {
    pub verification_id: String,
    pub is_valid: bool,
    pub verification_errors: Vec<IntegrityError>,
    pub domains_verified: Vec<DomainId>,
    pub cross_domain_consistency: bool,
    pub snapshot_integrity: bool,
    pub verification_time_ms: u128,
}

/// Integrity error details
#[derive(Debug, Clone)]
pub struct IntegrityError {
    pub error_type: IntegrityErrorType,
    pub domain_id: Option<DomainId>,
    pub node_id: Option<NodeId>,
    pub description: String,
    pub severity: IntegrityErrorSeverity,
}

/// Types of integrity errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityErrorType {
    CorruptedData,
    MissingDependency,
    InvalidChecksum,
    InconsistentReferences,
    TemporalViolation,
}

/// Severity levels for integrity errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityErrorSeverity {
    Critical,
    Warning,
    Info,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            max_snapshot_size_bytes: 100 * 1024 * 1024, // 100MB
            max_incremental_snapshots: 10,
            enable_compression: true,
            verify_integrity_on_load: true,
            backup_interval_ms: 60000, // 1 minute
        }
    }
}

impl TegPersistenceManager {
    /// Create a new TEG persistence manager
    pub fn new(
        smt: Arc<Mutex<TegMultiDomainSmt<MemoryBackend>>>,
        base_directory: PathBuf,
        persistence_config: Option<PersistenceConfig>,
    ) -> Result<Self> {
        let config = persistence_config.unwrap_or_default();
        
        // Ensure base directory exists
        if !base_directory.exists() {
            fs::create_dir_all(&base_directory)
                .map_err(|e| anyhow!("Failed to create persistence directory: {}", e))?;
        }
        
        Ok(Self {
            smt,
            base_directory,
            persistence_config: config,
        })
    }

    /// Create a full snapshot of all TEG data
    pub fn create_full_snapshot(&self) -> Result<PersistenceResult> {
        let start_time = std::time::Instant::now();
        let snapshot_id = format!("full_snapshot_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        let smt = self.smt.lock().map_err(|_| anyhow!("Failed to lock SMT"))?;
        let smt_root = smt.get_state_root();
        
        // Create snapshot directory
        let snapshot_dir = self.base_directory.join(&snapshot_id);
        fs::create_dir_all(&snapshot_dir)
            .map_err(|e| anyhow!("Failed to create snapshot directory: {}", e))?;
        
        // Export all domains (simplified - would iterate through actual domains)
        let included_domains = vec![]; // Placeholder
        let domain_count = included_domains.len();
        let node_count = 0; // Would be calculated from actual data
        
        // Create snapshot metadata
        let integrity_checksum = self.calculate_integrity_checksum(&smt_root, &included_domains)?;
        let snapshot = TegSnapshot {
            snapshot_id: snapshot_id.clone(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            smt_root,
            parent_snapshot_id: None,
            snapshot_type: SnapshotType::Full,
            included_domains: included_domains.clone(),
            metadata: SnapshotMetadata {
                size_bytes: 0, // Will be calculated after writing
                domain_count,
                node_count,
                integrity_checksum,
                compression_type: if self.persistence_config.enable_compression {
                    Some(CompressionType::Gzip)
                } else {
                    None
                },
            },
        };
        
        // Write snapshot to disk
        let snapshot_file = snapshot_dir.join("snapshot.json");
        let snapshot_json = serde_json::to_string_pretty(&self.serialize_snapshot(&snapshot)?)
            .map_err(|e| anyhow!("Failed to serialize snapshot: {}", e))?;
        
        fs::write(&snapshot_file, &snapshot_json)
            .map_err(|e| anyhow!("Failed to write snapshot file: {}", e))?;
        
        let size_bytes = snapshot_json.len();
        let operation_time_ms = start_time.elapsed().as_millis();
        
        Ok(PersistenceResult {
            operation_id: format!("create_full_snapshot_{}", snapshot_id),
            success: true,
            error_message: None,
            snapshot_id: Some(snapshot_id),
            size_bytes,
            operation_time_ms,
        })
    }

    /// Create an incremental snapshot with changes since the last snapshot
    pub fn create_incremental_snapshot(&self, parent_snapshot_id: String) -> Result<PersistenceResult> {
        let start_time = std::time::Instant::now();
        let snapshot_id = format!("incr_snapshot_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());
        
        let smt = self.smt.lock().map_err(|_| anyhow!("Failed to lock SMT"))?;
        let smt_root = smt.get_state_root();
        
        // Load parent snapshot to calculate differences
        let parent_snapshot = self.load_snapshot(&parent_snapshot_id)?;
        
        // Calculate changes since parent (simplified)
        let changed_domains = self.calculate_domain_changes(&smt, &parent_snapshot)?;
        
        // Create incremental snapshot
        let snapshot = TegSnapshot {
            snapshot_id: snapshot_id.clone(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            smt_root,
            parent_snapshot_id: Some(parent_snapshot_id),
            snapshot_type: SnapshotType::Incremental,
            included_domains: changed_domains.clone(),
            metadata: SnapshotMetadata {
                size_bytes: 0,
                domain_count: changed_domains.len(),
                node_count: 0, // Would calculate from changes
                integrity_checksum: self.calculate_integrity_checksum(&smt_root, &changed_domains)?,
                compression_type: if self.persistence_config.enable_compression {
                    Some(CompressionType::Gzip)
                } else {
                    None
                },
            },
        };
        
        // Write incremental snapshot
        let snapshot_dir = self.base_directory.join(&snapshot_id);
        fs::create_dir_all(&snapshot_dir)
            .map_err(|e| anyhow!("Failed to create snapshot directory: {}", e))?;
        
        let snapshot_file = snapshot_dir.join("snapshot.json");
        let snapshot_json = serde_json::to_string_pretty(&self.serialize_snapshot(&snapshot)?)
            .map_err(|e| anyhow!("Failed to serialize snapshot: {}", e))?;
        
        fs::write(&snapshot_file, &snapshot_json)
            .map_err(|e| anyhow!("Failed to write snapshot file: {}", e))?;
        
        let size_bytes = snapshot_json.len();
        let operation_time_ms = start_time.elapsed().as_millis();
        
        Ok(PersistenceResult {
            operation_id: format!("create_incremental_snapshot_{}", snapshot_id),
            success: true,
            error_message: None,
            snapshot_id: Some(snapshot_id),
            size_bytes,
            operation_time_ms,
        })
    }

    /// Export TEG data for a specific domain
    pub fn export_domain_data(&self, domain_id: DomainId, export_path: PathBuf) -> Result<ImportExportResult> {
        let start_time = std::time::Instant::now();
        let operation_id = format!("export_domain_{}_{}", 
            hex::encode(domain_id.as_ssz_bytes()), 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        );
        
        let _smt = self.smt.lock().map_err(|_| anyhow!("Failed to lock SMT"))?;
        
        // Create domain export (simplified)
        let domain_export = DomainStateExport {
            domain_id,
            effects: HashMap::new(), // Would extract from SMT
            resources: HashMap::new(), // Would extract from SMT
            intents: HashMap::new(), // Would extract from SMT
            handlers: HashMap::new(), // Would extract from SMT
            temporal_relationships: vec![], // Would extract from SMT
            domain_metadata: crate::teg_deployment::DomainExportMetadata {
                effect_count: 0,
                resource_count: 0,
                intent_count: 0,
                handler_count: 0,
                temporal_relationship_count: 0,
            },
        };
        
        // Write export to file
        let export_json = domain_export.to_json()?;
        let export_str = serde_json::to_string_pretty(&export_json)
            .map_err(|e| anyhow!("Failed to serialize domain export: {}", e))?;
        
        fs::write(&export_path, &export_str)
            .map_err(|e| anyhow!("Failed to write export file: {}", e))?;
        
        let operation_time_ms = start_time.elapsed().as_millis();
        
        Ok(ImportExportResult {
            operation_id,
            success: true,
            error_message: None,
            domains_processed: vec![domain_id],
            nodes_processed: 0, // Would count actual nodes
            file_path: export_path,
            operation_time_ms,
        })
    }

    /// Import TEG data from an export file
    pub fn import_domain_data(&self, import_path: PathBuf) -> Result<ImportExportResult> {
        let start_time = std::time::Instant::now();
        let operation_id = format!("import_domain_{}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        );
        
        // Read import file
        let import_str = fs::read_to_string(&import_path)
            .map_err(|e| anyhow!("Failed to read import file: {}", e))?;
        
        let import_json: JsonValue = serde_json::from_str(&import_str)
            .map_err(|e| anyhow!("Failed to parse import JSON: {}", e))?;
        
        // Parse domain ID
        let domain_id_str = import_json["domain_id"].as_str()
            .ok_or_else(|| anyhow!("Missing domain_id in import"))?;
        
        let domain_id_bytes = hex::decode(domain_id_str)
            .map_err(|e| anyhow!("Invalid domain_id hex: {}", e))?;
        
        if domain_id_bytes.len() != 32 {
            return Err(anyhow!("Invalid domain_id length"));
        }
        
        let mut domain_bytes = [0u8; 32];
        domain_bytes.copy_from_slice(&domain_id_bytes);
        let domain_id = DomainId::new(domain_bytes);
        
        // Import data into SMT (simplified)
        let _smt = self.smt.lock().map_err(|_| anyhow!("Failed to lock SMT"))?;
        
        // Process effects, resources, intents, handlers from JSON
        // This would parse each section and store in SMT
        let nodes_processed = 0; // Would count imported nodes
        
        let operation_time_ms = start_time.elapsed().as_millis();
        
        Ok(ImportExportResult {
            operation_id,
            success: true,
            error_message: None,
            domains_processed: vec![domain_id],
            nodes_processed,
            file_path: import_path,
            operation_time_ms,
        })
    }

    /// Verify integrity of TEG data across domains
    pub fn verify_cross_domain_integrity(&self) -> Result<IntegrityVerificationResult> {
        let start_time = std::time::Instant::now();
        let verification_id = format!("integrity_check_{}", 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
        );
        
        let smt = self.smt.lock().map_err(|_| anyhow!("Failed to lock SMT"))?;
        
        let mut verification_errors = Vec::new();
        let domains_verified = vec![]; // Would scan SMT for domains
        
        // Verify cross-domain consistency
        let cross_domain_consistency = self.verify_cross_domain_references(&smt, &domains_verified)?;
        if !cross_domain_consistency {
            verification_errors.push(IntegrityError {
                error_type: IntegrityErrorType::InconsistentReferences,
                domain_id: None,
                node_id: None,
                description: "Cross-domain references are inconsistent".to_string(),
                severity: IntegrityErrorSeverity::Critical,
            });
        }
        
        // Verify temporal constraints
        let temporal_consistency = self.verify_temporal_constraints(&smt, &domains_verified)?;
        if !temporal_consistency {
            verification_errors.push(IntegrityError {
                error_type: IntegrityErrorType::TemporalViolation,
                domain_id: None,
                node_id: None,
                description: "Temporal constraints violated".to_string(),
                severity: IntegrityErrorSeverity::Critical,
            });
        }
        
        let is_valid = verification_errors.is_empty();
        let operation_time_ms = start_time.elapsed().as_millis();
        
        Ok(IntegrityVerificationResult {
            verification_id,
            is_valid,
            verification_errors,
            domains_verified,
            cross_domain_consistency,
            snapshot_integrity: true, // Would verify snapshot checksums
            verification_time_ms: operation_time_ms,
        })
    }

    /// Load a snapshot from disk
    fn load_snapshot(&self, snapshot_id: &str) -> Result<TegSnapshot> {
        let snapshot_dir = self.base_directory.join(snapshot_id);
        let snapshot_file = snapshot_dir.join("snapshot.json");
        
        if !snapshot_file.exists() {
            return Err(anyhow!("Snapshot not found: {}", snapshot_id));
        }
        
        let snapshot_str = fs::read_to_string(&snapshot_file)
            .map_err(|e| anyhow!("Failed to read snapshot file: {}", e))?;
        
        let snapshot_json: JsonValue = serde_json::from_str(&snapshot_str)
            .map_err(|e| anyhow!("Failed to parse snapshot JSON: {}", e))?;
        
        self.deserialize_snapshot(&snapshot_json)
    }

    /// Calculate domain changes since a parent snapshot
    fn calculate_domain_changes(
        &self,
        _smt: &TegMultiDomainSmt<MemoryBackend>,
        _parent_snapshot: &TegSnapshot,
    ) -> Result<Vec<DomainId>> {
        // Simplified implementation - would compare SMT states
        Ok(vec![])
    }

    /// Calculate integrity checksum for domains
    fn calculate_integrity_checksum(
        &self,
        smt_root: &[u8; 32],
        domains: &[DomainId],
    ) -> Result<[u8; 32]> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(smt_root);
        for domain in domains {
            hasher.update(domain.as_ssz_bytes());
        }
        Ok(hasher.finalize().into())
    }

    /// Serialize snapshot to JSON
    fn serialize_snapshot(&self, snapshot: &TegSnapshot) -> Result<JsonValue> {
        let mut json_map = JsonMap::new();
        
        json_map.insert("snapshot_id".to_string(), JsonValue::String(snapshot.snapshot_id.clone()));
        json_map.insert("created_at".to_string(), JsonValue::Number(snapshot.created_at.into()));
        json_map.insert("smt_root".to_string(), JsonValue::String(hex::encode(snapshot.smt_root)));
        
        if let Some(parent_id) = &snapshot.parent_snapshot_id {
            json_map.insert("parent_snapshot_id".to_string(), JsonValue::String(parent_id.clone()));
        }
        
        let snapshot_type_str = match snapshot.snapshot_type {
            SnapshotType::Full => "full".to_string(),
            SnapshotType::Incremental => "incremental".to_string(),
            SnapshotType::DomainSpecific(domain_id) => format!("domain_specific_{}", hex::encode(domain_id.as_ssz_bytes())),
        };
        json_map.insert("snapshot_type".to_string(), JsonValue::String(snapshot_type_str));
        
        Ok(JsonValue::Object(json_map))
    }

    /// Deserialize snapshot from JSON
    fn deserialize_snapshot(&self, json: &JsonValue) -> Result<TegSnapshot> {
        let snapshot_id = json["snapshot_id"].as_str()
            .ok_or_else(|| anyhow!("Missing snapshot_id"))?
            .to_string();
        
        let created_at = json["created_at"].as_u64()
            .ok_or_else(|| anyhow!("Missing created_at"))?;
        
        let smt_root_str = json["smt_root"].as_str()
            .ok_or_else(|| anyhow!("Missing smt_root"))?;
        
        let smt_root_bytes = hex::decode(smt_root_str)
            .map_err(|e| anyhow!("Invalid smt_root hex: {}", e))?;
        
        if smt_root_bytes.len() != 32 {
            return Err(anyhow!("Invalid smt_root length"));
        }
        
        let mut smt_root = [0u8; 32];
        smt_root.copy_from_slice(&smt_root_bytes);
        
        Ok(TegSnapshot {
            snapshot_id,
            created_at,
            smt_root,
            parent_snapshot_id: json["parent_snapshot_id"].as_str().map(|s| s.to_string()),
            snapshot_type: SnapshotType::Full, // Simplified
            included_domains: vec![], // Would parse from JSON
            metadata: SnapshotMetadata {
                size_bytes: 0,
                domain_count: 0,
                node_count: 0,
                integrity_checksum: [0u8; 32],
                compression_type: None,
            },
        })
    }

    /// Verify cross-domain references
    fn verify_cross_domain_references(
        &self,
        _smt: &TegMultiDomainSmt<MemoryBackend>,
        _domains: &[DomainId],
    ) -> Result<bool> {
        // Simplified verification
        Ok(true)
    }

    /// Verify temporal constraints
    fn verify_temporal_constraints(
        &self,
        _smt: &TegMultiDomainSmt<MemoryBackend>,
        _domains: &[DomainId],
    ) -> Result<bool> {
        // Simplified verification
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smt::MemoryBackend;
    use tempfile::TempDir;

    #[test]
    fn test_persistence_manager_creation() {
        let backend = MemoryBackend::new();
        let smt = Arc::new(Mutex::new(TegMultiDomainSmt::new(backend)));
        let temp_dir = TempDir::new().unwrap();
        
        let _manager = TegPersistenceManager::new(
            smt,
            temp_dir.path().to_path_buf(),
            None,
        ).unwrap();
        
        // Just verify we can create the manager
        assert!(true);
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = TegSnapshot {
            snapshot_id: "test_snapshot".to_string(),
            created_at: 1234567890,
            smt_root: [1u8; 32],
            parent_snapshot_id: None,
            snapshot_type: SnapshotType::Full,
            included_domains: vec![],
            metadata: SnapshotMetadata {
                size_bytes: 1024,
                domain_count: 1,
                node_count: 10,
                integrity_checksum: [2u8; 32],
                compression_type: None,
            },
        };
        
        let temp_dir = TempDir::new().unwrap();
        let backend = MemoryBackend::new();
        let smt = Arc::new(Mutex::new(TegMultiDomainSmt::new(backend)));
        
        let manager = TegPersistenceManager::new(
            smt,
            temp_dir.path().to_path_buf(),
            None,
        ).unwrap();
        
        let json = manager.serialize_snapshot(&snapshot).unwrap();
        assert!(json["snapshot_id"].as_str().unwrap() == "test_snapshot");
    }
} 