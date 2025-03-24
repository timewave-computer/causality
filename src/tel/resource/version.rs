// Resource Versioning System for TEL
//
// This module implements version control for resources in the
// Temporal Effect Language (TEL), providing history tracking,
// differential storage, and version comparison capabilities.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use borsh::{BorshSerialize, BorshDeserialize};
use crypto::{
    hash::{ContentId, HashError, HashFactory, HashOutput},
    ContentAddressed,
};

use crate::tel::{
    types::{ResourceId, Address, Domain, Timestamp},
    error::{TelError, TelResult},
    resource::{
        Register,
        RegisterId,
        RegisterContents,
        ResourceOperation,
    },
};

/// Version identifier for a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct VersionId(pub Uuid);

impl VersionId {
    /// Create a new version ID
    pub fn new() -> Self {
        // Generate a unique string based on the current time to hash
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
            
        let version_data = format!("version-{}", now);
        
        // Generate a content ID
        let hasher = HashFactory::default().create_hasher().unwrap();
        let hash = hasher.hash(version_data.as_bytes());
        let content_id = ContentId::from(hash);
        
        // Create version ID from the content_id
        Self::from_content_id(&content_id)
    }
    
    /// Create from a ContentId
    pub fn from_content_id(content_id: &ContentId) -> Self {
        // Create a UUID from the first 16 bytes of the content hash
        let hash_bytes = content_id.hash().as_bytes();
        let mut uuid_bytes = [0u8; 16];
        
        // Copy the first 16 bytes (or fewer if hash is shorter)
        let copy_len = std::cmp::min(hash_bytes.len(), 16);
        uuid_bytes[..copy_len].copy_from_slice(&hash_bytes[..copy_len]);
        
        Self(Uuid::from_bytes(uuid_bytes))
    }
    
    /// Convert from a string
    pub fn from_str(s: &str) -> TelResult<Self> {
        match Uuid::parse_str(s) {
            Ok(uuid) => Ok(Self(uuid)),
            Err(_) => Err(TelError::InvalidId(format!("Invalid version ID: {}", s)))
        }
    }
}

impl ContentAddressed for VersionId {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
        let bytes = self.0.as_bytes();
        hasher.hash(bytes)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        if bytes.len() < 16 {
            return Err(HashError::InvalidLength);
        }
        
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&bytes[..16]);
        
        Ok(Self(Uuid::from_bytes(uuid_bytes)))
    }
}

impl std::fmt::Display for VersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Types of changes that can be applied to a resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// Initial creation of the resource
    Created,
    
    /// Update to the contents of the resource
    ContentUpdated,
    
    /// Resource owner changed
    OwnershipChanged(Address),
    
    /// Resource state changed
    StateChanged,
    
    /// Resource was marked for deletion
    MarkedForDeletion,
    
    /// Resource domain changed
    DomainChanged(Domain),
    
    /// Resource metadata changed
    MetadataChanged,
}

/// Represents a tracked change to a resource
#[derive(Debug, Clone)]
pub struct ResourceChange {
    /// Unique ID for this change
    pub id: VersionId,
    
    /// ID of the resource that was changed
    pub register_id: ContentId,
    
    /// Type of change that was applied
    pub change_type: ChangeType,
    
    /// Operation that caused this change
    pub operation_id: Option<Uuid>,
    
    /// Previous contents (if contents were changed)
    pub previous_contents: Option<RegisterContents>,
    
    /// New contents (if contents were changed)
    pub new_contents: Option<RegisterContents>,
    
    /// Timestamp of the change
    pub timestamp: Timestamp,
    
    /// User that initiated the change
    pub initiator: Address,
    
    /// Parent version ID
    pub parent_version: Option<VersionId>,
}

impl ResourceChange {
    /// Create a new resource change
    pub fn new(
        register_id: ContentId,
        change_type: ChangeType,
        operation_id: Option<Uuid>,
        previous_contents: Option<RegisterContents>,
        new_contents: Option<RegisterContents>,
        initiator: Address,
        parent_version: Option<VersionId>,
    ) -> Self {
        Self {
            id: VersionId::new(),
            register_id,
            change_type,
            operation_id,
            previous_contents,
            new_contents,
            timestamp: SystemTime::now().into(),
            initiator,
            parent_version,
        }
    }
}

/// Configurable options for the versioning system
#[derive(Debug, Clone)]
pub struct VersioningConfig {
    /// Whether versioning is enabled
    pub enabled: bool,
    
    /// Maximum number of versions to keep per resource
    pub max_versions_per_resource: usize,
    
    /// Whether to store full content snapshots or just diffs
    pub store_full_contents: bool,
    
    /// Whether to track metadata changes
    pub track_metadata_changes: bool,
    
    /// Time to keep historical versions (None = keep forever)
    pub version_retention: Option<Duration>,
}

impl Default for VersioningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_versions_per_resource: 10,
            store_full_contents: true,
            track_metadata_changes: true,
            version_retention: Some(Duration::from_secs(60 * 60 * 24 * 30)), // 30 days
        }
    }
}

/// Results of a version pruning operation
#[derive(Debug, Clone)]
pub struct PruningResults {
    /// Number of versions pruned
    pub versions_pruned: usize,
    
    /// Number of resources affected
    pub resources_affected: usize,
    
    /// Bytes freed
    pub bytes_freed: usize,
}

/// Options for restoring a resource to a previous version
#[derive(Debug, Clone)]
pub struct RestoreOptions {
    /// Whether to create a new version for the restore operation
    pub create_restore_version: bool,
    
    /// Optional initiator to attribute the restore to
    pub initiator: Option<Address>,
}

impl Default for RestoreOptions {
    fn default() -> Self {
        Self {
            create_restore_version: true,
            initiator: None,
        }
    }
}

/// Query options for listing versions
#[derive(Debug, Clone)]
pub struct VersionQueryOptions {
    /// Maximum number of versions to return
    pub limit: Option<usize>,
    
    /// Offset for pagination
    pub offset: Option<usize>,
    
    /// Filter by initiator
    pub initiator: Option<Address>,
    
    /// Filter by timestamp range (from)
    pub from_time: Option<Timestamp>,
    
    /// Filter by timestamp range (to)
    pub to_time: Option<Timestamp>,
    
    /// Filter by change type
    pub change_type: Option<ChangeType>,
}

impl Default for VersionQueryOptions {
    fn default() -> Self {
        Self {
            limit: Some(10),
            offset: None,
            initiator: None,
            from_time: None,
            to_time: None,
            change_type: None,
        }
    }
}

/// Core management system for resource versions
pub struct VersionManager {
    /// Configuration for the versioning system
    config: VersioningConfig,
    
    /// Version history for each resource
    versions: Arc<RwLock<HashMap<RegisterId, VecDeque<ResourceChange>>>>,
    
    /// Last pruning timestamp
    last_pruning: RwLock<SystemTime>,
}

impl VersionManager {
    /// Create a new version manager with the given configuration
    pub fn new(config: VersioningConfig) -> Self {
        Self {
            config,
            versions: Arc::new(RwLock::new(HashMap::new())),
            last_pruning: RwLock::new(SystemTime::now()),
        }
    }
    
    /// Create a new version manager with default configuration
    pub fn default() -> Self {
        Self::new(VersioningConfig::default())
    }
    
    /// Track a resource creation
    pub fn track_creation(
        &self,
        register: &Register,
        operation_id: Option<Uuid>,
        initiator: Address,
    ) -> TelResult<VersionId> {
        if !self.config.enabled {
            return Ok(VersionId::new());
        }
        
        let change = ResourceChange::new(
            register.id.clone(),
            ChangeType::Created,
            operation_id,
            None,
            Some(register.contents.clone()),
            initiator,
            None,
        );
        
        let version_id = change.id.clone();
        self.add_change(change)?;
        
        Ok(version_id)
    }
    
    /// Track a content update
    pub fn track_update(
        &self,
        register_id: &RegisterId,
        previous_contents: &RegisterContents,
        new_contents: &RegisterContents,
        operation_id: Option<Uuid>,
        initiator: Address,
    ) -> TelResult<VersionId> {
        if !self.config.enabled {
            return Ok(VersionId::new());
        }
        
        let parent = self.get_latest_version(register_id);
        
        let change = ResourceChange::new(
            register_id.clone(),
            ChangeType::ContentUpdated,
            operation_id,
            Some(previous_contents.clone()),
            Some(new_contents.clone()),
            initiator,
            parent.map(|p| p.id.clone()),
        );
        
        let version_id = change.id.clone();
        self.add_change(change)?;
        
        Ok(version_id)
    }
    
    /// Track an ownership change
    pub fn track_ownership_change(
        &self,
        register_id: &RegisterId,
        previous_owner: &Address,
        new_owner: &Address,
        operation_id: Option<Uuid>,
        initiator: Address,
    ) -> TelResult<VersionId> {
        if !self.config.enabled {
            return Ok(VersionId::new());
        }
        
        let parent = self.get_latest_version(register_id);
        
        let change = ResourceChange::new(
            register_id.clone(),
            ChangeType::OwnershipChanged(previous_owner.clone()),
            operation_id,
            None,
            None,
            initiator,
            parent.map(|p| p.id.clone()),
        );
        
        let version_id = change.id.clone();
        self.add_change(change)?;
        
        Ok(version_id)
    }
    
    /// Track a state change
    pub fn track_state_change(
        &self,
        register_id: &RegisterId,
        operation_id: Option<Uuid>,
        initiator: Address,
    ) -> TelResult<VersionId> {
        if !self.config.enabled {
            return Ok(VersionId::new());
        }
        
        let parent = self.get_latest_version(register_id);
        
        let change = ResourceChange::new(
            register_id.clone(),
            ChangeType::StateChanged,
            operation_id,
            None,
            None,
            initiator,
            parent.map(|p| p.id.clone()),
        );
        
        let version_id = change.id.clone();
        self.add_change(change)?;
        
        Ok(version_id)
    }
    
    /// Track marking a resource for deletion
    pub fn track_deletion_mark(
        &self,
        register_id: &RegisterId,
        operation_id: Option<Uuid>,
        initiator: Address,
    ) -> TelResult<VersionId> {
        if !self.config.enabled {
            return Ok(VersionId::new());
        }
        
        let parent = self.get_latest_version(register_id);
        
        let change = ResourceChange::new(
            register_id.clone(),
            ChangeType::MarkedForDeletion,
            operation_id,
            None,
            None,
            initiator,
            parent.map(|p| p.id.clone()),
        );
        
        let version_id = change.id.clone();
        self.add_change(change)?;
        
        Ok(version_id)
    }
    
    /// Get all versions for a resource
    pub fn get_versions(
        &self,
        register_id: &RegisterId,
        options: Option<VersionQueryOptions>,
    ) -> TelResult<Vec<ResourceChange>> {
        let options = options.unwrap_or_default();
        
        let versions = self.versions.read().map_err(|_| {
            TelError::ResourceError("Failed to acquire read lock on versions".to_string())
        })?;
        
        let resource_versions = versions.get(register_id).cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        
        let mut filtered = resource_versions.into_iter()
            .filter(|v| {
                let initiator_match = options.initiator.as_ref()
                    .map(|i| &v.initiator == i)
                    .unwrap_or(true);
                    
                let from_time_match = options.from_time.as_ref()
                    .map(|t| &v.timestamp >= t)
                    .unwrap_or(true);
                    
                let to_time_match = options.to_time.as_ref()
                    .map(|t| &v.timestamp <= t)
                    .unwrap_or(true);
                    
                let change_type_match = options.change_type.as_ref()
                    .map(|c| &v.change_type == c)
                    .unwrap_or(true);
                    
                initiator_match && from_time_match && to_time_match && change_type_match
            })
            .collect::<Vec<_>>();
            
        // Apply offset and limit
        if let Some(offset) = options.offset {
            if offset < filtered.len() {
                filtered = filtered.into_iter().skip(offset).collect();
            } else {
                filtered.clear();
            }
        }
        
        if let Some(limit) = options.limit {
            filtered.truncate(limit);
        }
        
        Ok(filtered)
    }
    
    /// Get a specific version by its ID
    pub fn get_version(
        &self,
        version_id: &VersionId,
    ) -> TelResult<Option<ResourceChange>> {
        let versions = self.versions.read().map_err(|_| {
            TelError::ResourceError("Failed to acquire read lock on versions".to_string())
        })?;
        
        for versions_list in versions.values() {
            for version in versions_list {
                if version.id == *version_id {
                    return Ok(Some(version.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Get the latest version for a resource
    pub fn get_latest_version(
        &self,
        register_id: &RegisterId,
    ) -> Option<ResourceChange> {
        if let Ok(versions) = self.versions.read() {
            if let Some(resource_versions) = versions.get(register_id) {
                if !resource_versions.is_empty() {
                    return Some(resource_versions.front().unwrap().clone());
                }
            }
        }
        
        None
    }
    
    /// Restore a resource to a specific version
    pub fn restore_version(
        &self,
        version_id: &VersionId,
        options: RestoreOptions,
        register_fn: impl FnOnce(&RegisterContents) -> TelResult<()>,
    ) -> TelResult<Option<VersionId>> {
        let version = match self.get_version(version_id)? {
            Some(v) => v,
            None => return Err(TelError::ResourceError(format!("Version {} not found", version_id))),
        };
        
        // If we don't have contents to restore, we can't do a restore
        let contents = match &version.new_contents {
            Some(c) => c,
            None => return Err(TelError::ResourceError("Cannot restore version with no content".to_string())),
        };
        
        // Call the register update function with the contents
        register_fn(contents)?;
        
        // If we should create a new version for this restore, do so
        if options.create_restore_version {
            let initiator = options.initiator.unwrap_or(version.initiator.clone());
            
            let current_version = self.get_latest_version(&version.register_id);
            let current_contents = current_version
                .and_then(|v| v.new_contents.clone())
                .or_else(|| current_version.and_then(|v| v.previous_contents.clone()));
            
            let previous = current_contents.clone();
            
            let change = ResourceChange::new(
                version.register_id.clone(),
                ChangeType::ContentUpdated,
                None, // No operation ID for a restore
                previous,
                Some(contents.clone()),
                initiator,
                current_version.map(|v| v.id.clone()),
            );
            
            let restore_version_id = change.id.clone();
            self.add_change(change)?;
            
            return Ok(Some(restore_version_id));
        }
        
        Ok(None)
    }
    
    /// Run pruning to remove old versions
    pub fn prune_versions(&self) -> TelResult<PruningResults> {
        if !self.config.enabled {
            return Ok(PruningResults {
                versions_pruned: 0,
                resources_affected: 0,
                bytes_freed: 0,
            });
        }
        
        let mut versions = self.versions.write().map_err(|_| {
            TelError::ResourceError("Failed to acquire write lock on versions".to_string())
        })?;
        
        let mut results = PruningResults {
            versions_pruned: 0,
            resources_affected: 0,
            bytes_freed: 0,
        };
        
        // Get current time for age-based pruning
        let now = SystemTime::now();
        
        for (_, resource_versions) in versions.iter_mut() {
            let original_len = resource_versions.len();
            
            // Apply max versions limit
            if resource_versions.len() > self.config.max_versions_per_resource {
                let to_remove = resource_versions.len() - self.config.max_versions_per_resource;
                
                // Remove oldest versions
                for _ in 0..to_remove {
                    if let Some(v) = resource_versions.pop_back() {
                        // Estimate size of version for bytes freed calculation
                        results.bytes_freed += self.estimate_version_size(&v);
                        results.versions_pruned += 1;
                    }
                }
            }
            
            // Apply age-based retention if configured
            if let Some(retention) = self.config.version_retention {
                let cutoff = now - retention;
                
                // Only keep versions that are newer than the cutoff
                // but always keep at least one version
                while resource_versions.len() > 1 {
                    if let Some(oldest) = resource_versions.back() {
                        if oldest.timestamp.as_system_time() < cutoff {
                            if let Some(v) = resource_versions.pop_back() {
                                results.bytes_freed += self.estimate_version_size(&v);
                                results.versions_pruned += 1;
                            }
                        } else {
                            // If the oldest is newer than cutoff, then all are newer
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            
            if original_len != resource_versions.len() {
                results.resources_affected += 1;
            }
        }
        
        // Update last pruning time
        if let Ok(mut last_pruning) = self.last_pruning.write() {
            *last_pruning = now;
        }
        
        Ok(results)
    }
    
    /// Compare two versions of a resource
    pub fn compare_versions(
        &self,
        version1: &VersionId,
        version2: &VersionId,
    ) -> TelResult<VersionDiff> {
        let v1 = match self.get_version(version1)? {
            Some(v) => v,
            None => return Err(TelError::ResourceError(format!("Version {} not found", version1))),
        };
        
        let v2 = match self.get_version(version2)? {
            Some(v) => v,
            None => return Err(TelError::ResourceError(format!("Version {} not found", version2))),
        };
        
        // Make sure they're for the same resource
        if v1.register_id != v2.register_id {
            return Err(TelError::ResourceError("Cannot compare versions from different resources".to_string()));
        }
        
        let mut diff = VersionDiff {
            register_id: v1.register_id.clone(),
            version1: version1.clone(),
            version2: version2.clone(),
            content_changed: false,
            owner_changed: false,
            state_changed: false,
            domain_changed: false,
            metadata_changed: false,
            time_difference: (v2.timestamp.as_system_time().duration_since(v1.timestamp.as_system_time()))
                .unwrap_or_default(),
        };
        
        // Determine what changed
        match (&v1.change_type, &v2.change_type) {
            (ChangeType::ContentUpdated, ChangeType::ContentUpdated) => {
                diff.content_changed = true;
            },
            (ChangeType::OwnershipChanged(_), ChangeType::OwnershipChanged(_)) => {
                diff.owner_changed = true;
            },
            (ChangeType::StateChanged, ChangeType::StateChanged) => {
                diff.state_changed = true;
            },
            (ChangeType::DomainChanged(_), ChangeType::DomainChanged(_)) => {
                diff.domain_changed = true;
            },
            (ChangeType::MetadataChanged, ChangeType::MetadataChanged) => {
                diff.metadata_changed = true;
            },
            _ => {
                // Mixed changes, check what's different by examining contents
                if v1.new_contents != v2.new_contents {
                    diff.content_changed = true;
                }
                
                // Check ownership changes
                if let (ChangeType::OwnershipChanged(_), _) | (_, ChangeType::OwnershipChanged(_)) = (&v1.change_type, &v2.change_type) {
                    diff.owner_changed = true;
                }
                
                // Check state changes
                if let (ChangeType::StateChanged, _) | (_, ChangeType::StateChanged) = (&v1.change_type, &v2.change_type) {
                    diff.state_changed = true;
                }
                
                // Check domain changes
                if let (ChangeType::DomainChanged(_), _) | (_, ChangeType::DomainChanged(_)) = (&v1.change_type, &v2.change_type) {
                    diff.domain_changed = true;
                }
                
                // Check metadata changes
                if let (ChangeType::MetadataChanged, _) | (_, ChangeType::MetadataChanged) = (&v1.change_type, &v2.change_type) {
                    diff.metadata_changed = true;
                }
            }
        }
        
        Ok(diff)
    }
    
    /// Check if a version exists
    pub fn version_exists(&self, version_id: &VersionId) -> bool {
        if let Ok(Some(_)) = self.get_version(version_id) {
            true
        } else {
            false
        }
    }
    
    /// Get resource version history as a tree
    pub fn get_version_tree(
        &self,
        register_id: &RegisterId,
    ) -> TelResult<VersionTree> {
        let versions = self.get_versions(register_id, None)?;
        
        if versions.is_empty() {
            return Ok(VersionTree {
                register_id: register_id.clone(),
                root: None,
            });
        }
        
        // Build a map of version ID to node index
        let mut version_map: HashMap<VersionId, usize> = HashMap::new();
        let mut nodes: Vec<VersionTreeNode> = Vec::with_capacity(versions.len());
        
        for version in &versions {
            let node = VersionTreeNode {
                version_id: version.id.clone(),
                change_type: version.change_type.clone(),
                timestamp: version.timestamp.clone(),
                initiator: version.initiator.clone(),
                children: Vec::new(),
            };
            
            version_map.insert(version.id.clone(), nodes.len());
            nodes.push(node);
        }
        
        // Connect parent-child relationships
        for (i, version) in versions.iter().enumerate() {
            if let Some(parent_id) = &version.parent_version {
                if let Some(parent_idx) = version_map.get(parent_id) {
                    nodes[*parent_idx].children.push(i);
                }
            }
        }
        
        // Find the root node (the one without a parent)
        let root_idx = versions.iter()
            .position(|v| v.parent_version.is_none())
            .unwrap_or(0);
        
        Ok(VersionTree {
            register_id: register_id.clone(),
            root: Some(VersionTreeNode {
                version_id: versions[root_idx].id.clone(),
                change_type: versions[root_idx].change_type.clone(),
                timestamp: versions[root_idx].timestamp.clone(),
                initiator: versions[root_idx].initiator.clone(),
                children: nodes[root_idx].children.clone(),
            }),
        })
    }
    
    /// Add a change to the version history
    fn add_change(&self, change: ResourceChange) -> TelResult<()> {
        let mut versions = self.versions.write().map_err(|_| {
            TelError::ResourceError("Failed to acquire write lock on versions".to_string())
        })?;
        
        let register_id = change.register_id.clone();
        let resource_versions = versions.entry(register_id).or_insert_with(VecDeque::new);
        
        // Add the new version at the front (newest first)
        resource_versions.push_front(change);
        
        // Check if we need to prune
        if resource_versions.len() > self.config.max_versions_per_resource {
            // Just trim one for now, full pruning happens separately
            resource_versions.pop_back();
        }
        
        Ok(())
    }
    
    /// Estimate the size of a version in bytes
    fn estimate_version_size(&self, version: &ResourceChange) -> usize {
        let mut size = 0;
        
        // Fixed-size components
        size += std::mem::size_of::<VersionId>();
        size += std::mem::size_of::<RegisterId>();
        size += std::mem::size_of::<Timestamp>();
        size += std::mem::size_of::<Address>();
        
        // Optional fields
        if version.operation_id.is_some() {
            size += std::mem::size_of::<Uuid>();
        }
        
        if version.parent_version.is_some() {
            size += std::mem::size_of::<VersionId>();
        }
        
        // Content sizes (approximate)
        if let Some(contents) = &version.previous_contents {
            size += self.estimate_contents_size(contents);
        }
        
        if let Some(contents) = &version.new_contents {
            size += self.estimate_contents_size(contents);
        }
        
        size
    }
    
    /// Estimate the size of register contents
    fn estimate_contents_size(&self, contents: &RegisterContents) -> usize {
        match contents {
            RegisterContents::Binary(data) => data.len(),
            RegisterContents::Text(text) => text.len(),
            RegisterContents::Json(json) => {
                if let Ok(json_str) = serde_json::to_string(json) {
                    json_str.len()
                } else {
                    0
                }
            },
            RegisterContents::Struct(_) => 100, // A rough approximation
            _ => 64, // Default approximation for other types
        }
    }
}

/// Represents a difference between two versions
#[derive(Debug, Clone)]
pub struct VersionDiff {
    /// ID of the resource
    pub register_id: ContentId,
    
    /// First version ID
    pub version1: VersionId,
    
    /// Second version ID
    pub version2: VersionId,
    
    /// Whether content changed
    pub content_changed: bool,
    
    /// Whether owner changed
    pub owner_changed: bool,
    
    /// Whether state changed
    pub state_changed: bool,
    
    /// Whether domain changed
    pub domain_changed: bool,
    
    /// Whether metadata changed
    pub metadata_changed: bool,
    
    /// Time difference between versions
    pub time_difference: Duration,
}

/// Node in a version tree
#[derive(Debug, Clone)]
pub struct VersionTreeNode {
    /// Version ID
    pub version_id: VersionId,
    
    /// Type of change
    pub change_type: ChangeType,
    
    /// Timestamp
    pub timestamp: Timestamp,
    
    /// Initiator
    pub initiator: Address,
    
    /// Indices of child nodes
    pub children: Vec<usize>,
}

/// Tree representation of a resource's version history
#[derive(Debug, Clone)]
pub struct VersionTree {
    /// ID of the resource
    pub register_id: ContentId,
    
    /// Root node (first version)
    pub root: Option<VersionTreeNode>,
} 
