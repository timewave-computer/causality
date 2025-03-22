// Resource snapshot module for TEL
//
// This module implements a snapshot system for resources,
// allowing state recovery and point-in-time access to resource
// states.

use std::sync::{Arc, RwLock};
use std::collections::{HashMap, BTreeMap};
use std::time::{Duration, SystemTime};
use std::path::PathBuf;

use crate::tel::{
    error::{TelError, TelResult},
    types::{ResourceId, Timestamp, Domain, Address},
    resource::{
        ResourceManager,
        Register,
        RegisterId,
        RegisterContents,
        RegisterState,
    },
};

/// Snapshot identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SnapshotId(String);

impl SnapshotId {
    /// Create a new snapshot ID
    pub fn new() -> Self {
        use uuid::Uuid;
        Self(format!("snapshot-{}", Uuid::new_v4()))
    }
    
    /// Create a snapshot ID from a string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    /// Get the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Storage backend for snapshots
pub trait SnapshotStorage: Send + Sync {
    /// Store a snapshot
    fn store_snapshot(&self, id: &SnapshotId, data: &[u8]) -> TelResult<()>;
    
    /// Load a snapshot
    fn load_snapshot(&self, id: &SnapshotId) -> TelResult<Option<Vec<u8>>>;
    
    /// List available snapshots
    fn list_snapshots(&self) -> TelResult<Vec<SnapshotId>>;
    
    /// Delete a snapshot
    fn delete_snapshot(&self, id: &SnapshotId) -> TelResult<bool>;
}

/// File-based snapshot storage
pub struct FileSnapshotStorage {
    /// Base directory for snapshot storage
    base_dir: PathBuf,
}

impl FileSnapshotStorage {
    /// Create a new file-based snapshot storage
    pub fn new(base_dir: PathBuf) -> Self {
        // Ensure the directory exists
        std::fs::create_dir_all(&base_dir).ok();
        
        Self { base_dir }
    }
    
    /// Get the path for a snapshot file
    fn snapshot_path(&self, id: &SnapshotId) -> PathBuf {
        self.base_dir.join(format!("{}.snapshot", id.as_str()))
    }
}

impl SnapshotStorage for FileSnapshotStorage {
    fn store_snapshot(&self, id: &SnapshotId, data: &[u8]) -> TelResult<()> {
        let path = self.snapshot_path(id);
        std::fs::write(&path, data).map_err(|e| 
            TelError::InternalError(format!("Failed to write snapshot to {}: {}", path.display(), e))
        )
    }
    
    fn load_snapshot(&self, id: &SnapshotId) -> TelResult<Option<Vec<u8>>> {
        let path = self.snapshot_path(id);
        if !path.exists() {
            return Ok(None);
        }
        
        std::fs::read(&path).map(Some).map_err(|e| 
            TelError::InternalError(format!("Failed to read snapshot from {}: {}", path.display(), e))
        )
    }
    
    fn list_snapshots(&self) -> TelResult<Vec<SnapshotId>> {
        let entries = std::fs::read_dir(&self.base_dir).map_err(|e| 
            TelError::InternalError(format!("Failed to read snapshot directory: {}", e))
        )?;
        
        let mut snapshots = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| 
                TelError::InternalError(format!("Failed to read directory entry: {}", e))
            )?;
            
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "snapshot") {
                if let Some(file_stem) = path.file_stem() {
                    if let Some(name) = file_stem.to_str() {
                        snapshots.push(SnapshotId::from_string(name.to_string()));
                    }
                }
            }
        }
        
        Ok(snapshots)
    }
    
    fn delete_snapshot(&self, id: &SnapshotId) -> TelResult<bool> {
        let path = self.snapshot_path(id);
        if !path.exists() {
            return Ok(false);
        }
        
        std::fs::remove_file(&path).map(|_| true).map_err(|e| 
            TelError::InternalError(format!("Failed to delete snapshot {}: {}", path.display(), e))
        )
    }
}

/// In-memory snapshot storage (for testing)
pub struct MemorySnapshotStorage {
    /// Stored snapshots
    snapshots: RwLock<HashMap<SnapshotId, Vec<u8>>>,
}

impl MemorySnapshotStorage {
    /// Create a new in-memory snapshot storage
    pub fn new() -> Self {
        Self {
            snapshots: RwLock::new(HashMap::new()),
        }
    }
}

impl SnapshotStorage for MemorySnapshotStorage {
    fn store_snapshot(&self, id: &SnapshotId, data: &[u8]) -> TelResult<()> {
        let mut snapshots = self.snapshots.write().map_err(|_| 
            TelError::InternalError("Failed to acquire snapshots lock".to_string())
        )?;
        
        snapshots.insert(id.clone(), data.to_vec());
        
        Ok(())
    }
    
    fn load_snapshot(&self, id: &SnapshotId) -> TelResult<Option<Vec<u8>>> {
        let snapshots = self.snapshots.read().map_err(|_| 
            TelError::InternalError("Failed to acquire snapshots lock".to_string())
        )?;
        
        Ok(snapshots.get(id).cloned())
    }
    
    fn list_snapshots(&self) -> TelResult<Vec<SnapshotId>> {
        let snapshots = self.snapshots.read().map_err(|_| 
            TelError::InternalError("Failed to acquire snapshots lock".to_string())
        )?;
        
        Ok(snapshots.keys().cloned().collect())
    }
    
    fn delete_snapshot(&self, id: &SnapshotId) -> TelResult<bool> {
        let mut snapshots = self.snapshots.write().map_err(|_| 
            TelError::InternalError("Failed to acquire snapshots lock".to_string())
        )?;
        
        Ok(snapshots.remove(id).is_some())
    }
}

/// Metadata for a resource snapshot
#[derive(Debug, Clone)]
pub struct SnapshotMetadata {
    /// ID of the snapshot
    pub id: SnapshotId,
    /// Time the snapshot was created
    pub created_at: Timestamp,
    /// Description of the snapshot
    pub description: String,
    /// Creator of the snapshot
    pub creator: Option<Address>,
    /// Domain the snapshot belongs to
    pub domain: Option<Domain>,
    /// Number of resources in the snapshot
    pub resource_count: usize,
    /// Tags for the snapshot
    pub tags: Vec<String>,
}

/// Configuration for snapshot scheduling
#[derive(Debug, Clone)]
pub struct SnapshotScheduleConfig {
    /// Whether automatic snapshots are enabled
    pub enabled: bool,
    /// Interval between snapshots
    pub interval: Duration,
    /// Maximum number of automatic snapshots to keep
    pub max_snapshots: usize,
    /// Whether to include resources from all domains
    pub all_domains: bool,
    /// Specific domains to include
    pub domains: Vec<Domain>,
}

impl Default for SnapshotScheduleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(3600), // 1 hour
            max_snapshots: 24, // Keep 24 hours worth of snapshots
            all_domains: true,
            domains: Vec::new(),
        }
    }
}

/// Manager for resource snapshots
pub struct SnapshotManager {
    /// The resource manager
    resource_manager: Arc<ResourceManager>,
    /// Storage backend for snapshots
    storage: Box<dyn SnapshotStorage>,
    /// Snapshot schedule configuration
    schedule_config: RwLock<SnapshotScheduleConfig>,
    /// Metadata for snapshots
    snapshot_metadata: RwLock<HashMap<SnapshotId, SnapshotMetadata>>,
    /// Last automatic snapshot time
    last_auto_snapshot: RwLock<Option<SystemTime>>,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(
        resource_manager: Arc<ResourceManager>,
        storage: Box<dyn SnapshotStorage>,
        schedule_config: SnapshotScheduleConfig,
    ) -> Self {
        Self {
            resource_manager,
            storage,
            schedule_config: RwLock::new(schedule_config),
            snapshot_metadata: RwLock::new(HashMap::new()),
            last_auto_snapshot: RwLock::new(None),
        }
    }
    
    /// Create a snapshot of all resources
    pub fn create_snapshot(
        &self,
        description: String,
        creator: Option<&Address>,
        domain: Option<&Domain>,
        tags: Vec<String>,
    ) -> TelResult<SnapshotId> {
        // Generate a new snapshot ID
        let snapshot_id = SnapshotId::new();
        
        // Create the snapshot data
        let snapshot_data = self.create_snapshot_data(domain)?;
        
        // Store the snapshot
        self.storage.store_snapshot(&snapshot_id, &snapshot_data)?;
        
        // Store metadata
        let metadata = SnapshotMetadata {
            id: snapshot_id.clone(),
            created_at: Timestamp::now(),
            description,
            creator: creator.cloned(),
            domain: domain.cloned(),
            resource_count: snapshot_data.len() / 100, // Approximate count
            tags,
        };
        
        let mut snapshot_metadata = self.snapshot_metadata.write().map_err(|_| 
            TelError::InternalError("Failed to acquire snapshot metadata lock".to_string())
        )?;
        
        snapshot_metadata.insert(snapshot_id.clone(), metadata);
        
        Ok(snapshot_id)
    }
    
    /// Create snapshot data for resources
    fn create_snapshot_data(&self, domain: Option<&Domain>) -> TelResult<Vec<u8>> {
        // Get all registers
        let registers = if let Some(domain) = domain {
            self.resource_manager.get_registers_by_domain(domain)?
        } else {
            self.resource_manager.get_all_registers()?
        };
        
        // Serialize the registers
        // In a real implementation, this would use a binary serialization format
        // For the purposes of this implementation, we'll use JSON
        let json = serde_json::to_vec(&registers)
            .map_err(|e| TelError::InternalError(format!("Failed to serialize snapshot: {}", e)))?;
            
        Ok(json)
    }
    
    /// Restore a snapshot
    pub fn restore_snapshot(
        &self,
        id: &SnapshotId,
        options: RestoreOptions,
    ) -> TelResult<RestoreResult> {
        // Load the snapshot
        let snapshot_data = self.storage.load_snapshot(id)?
            .ok_or_else(|| TelError::ResourceSnapshotNotFound(id.as_str().to_string()))?;
            
        // Deserialize the snapshot
        let registers: Vec<Register> = serde_json::from_slice(&snapshot_data)
            .map_err(|e| TelError::InternalError(format!("Failed to deserialize snapshot: {}", e)))?;
            
        // Apply the restore
        match options.mode {
            RestoreMode::Full => {
                // Clear existing registers if needed
                if options.clear_existing {
                    self.resource_manager.clear_all_registers()?;
                }
                
                // Restore all registers
                for register in registers {
                    self.resource_manager.restore_register(&register)?;
                }
                
                Ok(RestoreResult {
                    restored_registers: registers.len(),
                    skipped_registers: 0,
                    errors: Vec::new(),
                })
            },
            RestoreMode::Selective { ref register_ids } => {
                let mut result = RestoreResult {
                    restored_registers: 0,
                    skipped_registers: 0,
                    errors: Vec::new(),
                };
                
                // Filter registers by ID
                for register in registers {
                    if register_ids.contains(&register.id) {
                        match self.resource_manager.restore_register(&register) {
                            Ok(_) => {
                                result.restored_registers += 1;
                            },
                            Err(e) => {
                                result.errors.push((
                                    register.id.clone(),
                                    format!("Failed to restore register: {}", e),
                                ));
                            }
                        }
                    } else {
                        result.skipped_registers += 1;
                    }
                }
                
                Ok(result)
            },
            RestoreMode::DomainOnly { ref domain } => {
                let mut result = RestoreResult {
                    restored_registers: 0,
                    skipped_registers: 0,
                    errors: Vec::new(),
                };
                
                // Filter registers by domain
                for register in registers {
                    if register.domain == *domain {
                        match self.resource_manager.restore_register(&register) {
                            Ok(_) => {
                                result.restored_registers += 1;
                            },
                            Err(e) => {
                                result.errors.push((
                                    register.id.clone(),
                                    format!("Failed to restore register: {}", e),
                                ));
                            }
                        }
                    } else {
                        result.skipped_registers += 1;
                    }
                }
                
                Ok(result)
            },
        }
    }
    
    /// List available snapshots
    pub fn list_snapshots(&self) -> TelResult<Vec<SnapshotMetadata>> {
        let snapshot_ids = self.storage.list_snapshots()?;
        
        let snapshot_metadata = self.snapshot_metadata.read().map_err(|_| 
            TelError::InternalError("Failed to acquire snapshot metadata lock".to_string())
        )?;
        
        let mut result = Vec::new();
        for id in snapshot_ids {
            if let Some(metadata) = snapshot_metadata.get(&id) {
                result.push(metadata.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Get metadata for a snapshot
    pub fn get_snapshot_metadata(&self, id: &SnapshotId) -> TelResult<Option<SnapshotMetadata>> {
        let snapshot_metadata = self.snapshot_metadata.read().map_err(|_| 
            TelError::InternalError("Failed to acquire snapshot metadata lock".to_string())
        )?;
        
        Ok(snapshot_metadata.get(id).cloned())
    }
    
    /// Delete a snapshot
    pub fn delete_snapshot(&self, id: &SnapshotId) -> TelResult<bool> {
        // Delete from storage
        let result = self.storage.delete_snapshot(id)?;
        
        if result {
            // Delete metadata
            let mut snapshot_metadata = self.snapshot_metadata.write().map_err(|_| 
                TelError::InternalError("Failed to acquire snapshot metadata lock".to_string())
            )?;
            
            snapshot_metadata.remove(id);
        }
        
        Ok(result)
    }
    
    /// Configure automatic snapshot scheduling
    pub fn configure_schedule(&self, config: SnapshotScheduleConfig) -> TelResult<()> {
        let mut schedule_config = self.schedule_config.write().map_err(|_| 
            TelError::InternalError("Failed to acquire schedule config lock".to_string())
        )?;
        
        *schedule_config = config;
        
        Ok(())
    }
    
    /// Check if an automatic snapshot should be created
    pub fn check_automatic_snapshot(&self) -> TelResult<bool> {
        let schedule_config = self.schedule_config.read().map_err(|_| 
            TelError::InternalError("Failed to acquire schedule config lock".to_string())
        )?;
        
        if !schedule_config.enabled {
            return Ok(false);
        }
        
        let mut last_auto_snapshot = self.last_auto_snapshot.write().map_err(|_| 
            TelError::InternalError("Failed to acquire last auto snapshot lock".to_string())
        )?;
        
        let now = SystemTime::now();
        let should_snapshot = match *last_auto_snapshot {
            None => true,
            Some(last_time) => {
                now.duration_since(last_time)
                    .map(|duration| duration >= schedule_config.interval)
                    .unwrap_or(true)
            }
        };
        
        if should_snapshot {
            *last_auto_snapshot = Some(now);
        }
        
        Ok(should_snapshot)
    }
    
    /// Create an automatic snapshot
    pub fn create_automatic_snapshot(&self) -> TelResult<Option<SnapshotId>> {
        if !self.check_automatic_snapshot()? {
            return Ok(None);
        }
        
        let schedule_config = self.schedule_config.read().map_err(|_| 
            TelError::InternalError("Failed to acquire schedule config lock".to_string())
        )?;
        
        // Create the snapshot
        let domain = if !schedule_config.all_domains && !schedule_config.domains.is_empty() {
            Some(&schedule_config.domains[0])
        } else {
            None
        };
        
        let snapshot_id = self.create_snapshot(
            "Automatic snapshot".to_string(),
            None,
            domain,
            vec!["automatic".to_string()],
        )?;
        
        // Prune old snapshots if needed
        self.prune_automatic_snapshots(schedule_config.max_snapshots)?;
        
        Ok(Some(snapshot_id))
    }
    
    /// Prune old automatic snapshots
    fn prune_automatic_snapshots(&self, max_snapshots: usize) -> TelResult<usize> {
        let snapshot_metadata = self.snapshot_metadata.read().map_err(|_| 
            TelError::InternalError("Failed to acquire snapshot metadata lock".to_string())
        )?;
        
        // Find automatic snapshots
        let mut auto_snapshots: Vec<_> = snapshot_metadata.values()
            .filter(|meta| meta.tags.contains(&"automatic".to_string()))
            .collect();
            
        // Sort by creation time (oldest first)
        auto_snapshots.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        // Delete old snapshots
        let to_delete = if auto_snapshots.len() > max_snapshots {
            auto_snapshots.len() - max_snapshots
        } else {
            0
        };
        
        let mut deleted = 0;
        for i in 0..to_delete {
            if self.delete_snapshot(&auto_snapshots[i].id)? {
                deleted += 1;
            }
        }
        
        Ok(deleted)
    }
}

/// Mode for restoring a snapshot
#[derive(Debug, Clone)]
pub enum RestoreMode {
    /// Restore all resources from the snapshot
    Full,
    /// Restore only specific registers
    Selective {
        /// Register IDs to restore
        register_ids: Vec<RegisterId>,
    },
    /// Restore only registers from a specific domain
    DomainOnly {
        /// Domain to restore
        domain: Domain,
    },
}

/// Options for restoring a snapshot
#[derive(Debug, Clone)]
pub struct RestoreOptions {
    /// Restore mode
    pub mode: RestoreMode,
    /// Whether to clear existing registers before restoring
    pub clear_existing: bool,
}

/// Result of a restore operation
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// Number of registers restored
    pub restored_registers: usize,
    /// Number of registers skipped
    pub skipped_registers: usize,
    /// Errors encountered during restore
    pub errors: Vec<(RegisterId, String)>,
} 