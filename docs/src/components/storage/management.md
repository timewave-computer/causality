<!-- Documentation about storage management -->
<!-- Original file: docs/src/storage_management.md -->

# Storage Management in Causality

## Overview

This document describes the storage management architecture in the Causality system. Storage management handles the organization, segmentation, rotation, and lifecycle management of stored data across the system. It provides mechanisms for efficient data access, ensures durability, implements retention policies, and optimizes storage utilization. The architecture is designed to handle the temporal nature of Causality data, supporting efficient retrieval by time ranges, content hashes, and other criteria.

## Core Concepts

### Storage Management System

The storage management system provides unified control over all storage operations:

```rust
pub struct StorageManagementSystem {
    /// Storage provider registry
    provider_registry: Arc<StorageProviderRegistry>,
    
    /// Segment management
    segment_manager: Arc<LogSegmentManager>,
    
    /// Storage lifecycle management
    lifecycle_manager: Arc<StorageLifecycleManager>,
    
    /// Storage metrics
    metrics_collector: Arc<StorageMetricsCollector>,
    
    /// Storage policy engine
    policy_engine: Arc<StoragePolicyEngine>,
}

impl StorageManagementSystem {
    /// Create a new storage management system
    pub fn new(
        provider_registry: Arc<StorageProviderRegistry>,
        segment_manager: Arc<LogSegmentManager>,
        lifecycle_manager: Arc<StorageLifecycleManager>,
        metrics_collector: Arc<StorageMetricsCollector>,
        policy_engine: Arc<StoragePolicyEngine>,
    ) -> Self {
        Self {
            provider_registry,
            segment_manager,
            lifecycle_manager,
            metrics_collector,
            policy_engine,
        }
    }
    
    /// Get a storage provider by type
    pub fn get_provider(&self, storage_type: StorageType) -> Result<Arc<dyn StorageProvider>, StorageError> {
        self.provider_registry.get_provider(storage_type)
    }
    
    /// Store an object in the appropriate storage
    pub fn store<T: ContentAddressed>(
        &self,
        object: &T,
        options: StoreOptions,
    ) -> Result<StorageMetadata, StorageError> {
        // Determine the storage type based on the object type and options
        let storage_type = self.determine_storage_type(object, &options)?;
        
        // Get the provider for this storage type
        let provider = self.get_provider(storage_type)?;
        
        // Apply storage policies
        let effective_options = self.policy_engine.apply_policies(
            object,
            storage_type,
            options,
        )?;
        
        // Store the object
        let (content_id, metadata) = provider.store_raw(object, &effective_options)?;
        
        // If segmentation is enabled, handle segmentation
        if effective_options.use_segmentation {
            self.segment_manager.handle_object_segmentation(
                storage_type,
                content_id,
                metadata.size,
            )?;
        }
        
        // Register with lifecycle manager
        self.lifecycle_manager.register_object(
            content_id.clone(),
            storage_type,
            &effective_options,
        )?;
        
        // Collect metrics
        self.metrics_collector.record_store_operation(
            storage_type,
            metadata.size,
        );
        
        Ok(metadata)
    }
    
    /// Retrieve an object from storage
    pub fn retrieve<T: ContentAddressed>(
        &self,
        content_id: &ContentId,
        options: RetrieveOptions,
    ) -> Result<T, StorageError> {
        // Try to determine the storage type from content ID
        let storage_type = self.resolve_storage_type(content_id)?;
        
        // Get the provider for this storage type
        let provider = self.get_provider(storage_type)?;
        
        // Retrieve the object
        let object = provider.retrieve::<T>(content_id, &options)?;
        
        // Collect metrics
        self.metrics_collector.record_retrieve_operation(
            storage_type,
            size_of(&object),
        );
        
        Ok(object)
    }
    
    /// Check if an object exists in storage
    pub fn exists(&self, content_id: &ContentId) -> Result<bool, StorageError> {
        // Try each storage provider to find the object
        for storage_type in StorageType::all() {
            if let Ok(provider) = self.get_provider(storage_type) {
                if provider.exists(content_id)? {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
}
```

### Segment Management

Segment management handles the organization of data into manageable chunks:

```rust
pub struct LogSegmentManager {
    /// Options for the segment manager
    options: SegmentManagerOptions,
    
    /// Currently active segment for writing
    active_segment: Arc<Mutex<LogSegment>>,
    
    /// Cached segments (recently used)
    cached_segments: Arc<RwLock<HashMap<String, Arc<Mutex<LogSegment>>>>>,
    
    /// Index of all segments
    segment_index: Arc<RwLock<BTreeMap<Timestamp, SegmentIndexEntry>>>,
    
    /// Storage configuration
    storage_config: StorageConfig,
    
    /// Last rotation timestamp
    last_rotation: Arc<Mutex<DateTime<Utc>>>,
}

impl LogSegmentManager {
    /// Create a new segment manager with the given options
    pub fn new(options: SegmentManagerOptions, storage_config: StorageConfig) -> Result<Self> {
        // Create base directory if it doesn't exist
        if !options.base_dir.exists() {
            fs::create_dir_all(&options.base_dir)?;
        }
        
        // Create initial active segment
        let segment_id = generate_segment_id();
        let segment_path = options.base_dir.join(format!("{}.log", segment_id));
        let mut segment = LogSegment::new(segment_id);
        segment.set_path(&segment_path);
        
        let active_segment = Arc::new(Mutex::new(segment));
        
        Ok(LogSegmentManager {
            options,
            active_segment,
            cached_segments: Arc::new(RwLock::new(HashMap::new())),
            segment_index: Arc::new(RwLock::new(BTreeMap::new())),
            storage_config,
            last_rotation: Arc::new(Mutex::new(Utc::now())),
        })
    }
    
    /// Check if the active segment needs rotation
    fn check_rotation(&self) -> Result<()> {
        let should_rotate = {
            let active = self.active_segment.lock()?;
            
            // Check rotation criteria
            for criteria in &self.options.rotation_criteria {
                match criteria {
                    RotationCriteria::EntryCount(max_entries) => {
                        if active.entry_count() >= *max_entries {
                            return Ok(true);
                        }
                    },
                    RotationCriteria::Size(_) => {
                        if active.is_full(&self.storage_config) {
                            return Ok(true);
                        }
                    },
                    RotationCriteria::TimeInterval(duration) => {
                        let last_rotation = self.last_rotation.lock()?;
                        let now = Utc::now();
                        if now.signed_duration_since(*last_rotation) >= *duration {
                            return Ok(true);
                        }
                    },
                    RotationCriteria::Custom(func) => {
                        if func(&active) {
                            return Ok(true);
                        }
                    },
                }
            }
            
            false
        };
        
        if should_rotate {
            self.rotate_segment()?;
        }
        
        Ok(())
    }
    
    /// Rotate the active segment
    fn rotate_segment(&self) -> Result<()> {
        // Create a new segment
        let segment_id = generate_segment_id();
        let segment_path = self.options.base_dir.join(format!("{}.log", segment_id));
        let mut new_segment = LogSegment::new(segment_id);
        new_segment.set_path(&segment_path);
        
        // Swap the active segment
        let old_segment = {
            let mut active_lock = self.active_segment.lock()?;
            
            // Mark the old segment as read-only
            active_lock.mark_readonly();
            
            // Flush the old segment
            active_lock.flush()?;
            
            // Update the index with the old segment info
            let old_info = active_lock.info().clone();
            
            // Extract the segment before updating
            let old_segment = std::mem::replace(&mut *active_lock, new_segment);
            
            // Update the last rotation timestamp
            let mut last_rotation = self.last_rotation.lock()?;
            *last_rotation = Utc::now();
            
            // Return the old segment
            old_segment
        };
        
        // Add the old segment to the cache
        self.add_to_cache(old_segment)?;
        
        // Manage cache size
        self.manage_cache()?;
        
        Ok(())
    }
}
```

### Storage Lifecycle Management

Lifecycle management controls data retention, archiving, and deletion:

```rust
pub struct StorageLifecycleManager {
    /// Lifecycle policies by storage type
    policies: HashMap<StorageType, LifecyclePolicy>,
    
    /// Registry of managed objects
    object_registry: Arc<RwLock<ObjectRegistry>>,
    
    /// Archival manager
    archival_manager: Arc<ArchivalManager>,
    
    /// Garbage collector
    garbage_collector: Arc<StorageGarbageCollector>,
    
    /// Scheduler for lifecycle tasks
    scheduler: Arc<TaskScheduler>,
}

impl StorageLifecycleManager {
    /// Create a new storage lifecycle manager
    pub fn new(
        archival_manager: Arc<ArchivalManager>,
        garbage_collector: Arc<StorageGarbageCollector>,
        scheduler: Arc<TaskScheduler>,
    ) -> Self {
        Self {
            policies: HashMap::new(),
            object_registry: Arc::new(RwLock::new(ObjectRegistry::new())),
            archival_manager,
            garbage_collector,
            scheduler,
        }
    }
    
    /// Register a lifecycle policy for a storage type
    pub fn register_policy(
        &mut self,
        storage_type: StorageType,
        policy: LifecyclePolicy,
    ) -> Result<(), LifecycleError> {
        self.policies.insert(storage_type, policy);
        Ok(())
    }
    
    /// Register an object for lifecycle management
    pub fn register_object(
        &self,
        content_id: ContentId,
        storage_type: StorageType,
        options: &StoreOptions,
    ) -> Result<(), LifecycleError> {
        // Get the policy for this storage type
        let policy = self.policies.get(&storage_type)
            .ok_or_else(|| LifecycleError::MissingPolicy(storage_type))?;
        
        // Create a lifecycle record for the object
        let record = LifecycleRecord {
            content_id: content_id.clone(),
            storage_type,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 0,
            size: options.content_size.unwrap_or(0),
            metadata: options.metadata.clone().unwrap_or_default(),
        };
        
        // Add to the registry
        {
            let mut registry = self.object_registry.write()?;
            registry.add_object(record)?;
        }
        
        // Schedule lifecycle transitions based on policy
        self.schedule_transitions(&content_id, storage_type, policy)?;
        
        Ok(())
    }
    
    /// Record an access to an object
    pub fn record_access(
        &self,
        content_id: &ContentId,
        storage_type: StorageType,
    ) -> Result<(), LifecycleError> {
        let mut registry = self.object_registry.write()?;
        
        // Update access information
        registry.update_access(content_id, storage_type, SystemTime::now())?;
        
        Ok(())
    }
    
    /// Schedule lifecycle transitions for an object
    fn schedule_transitions(
        &self,
        content_id: &ContentId,
        storage_type: StorageType,
        policy: &LifecyclePolicy,
    ) -> Result<(), LifecycleError> {
        let now = SystemTime::now();
        
        // Schedule archival if configured
        if let Some(archive_after) = policy.archive_after {
            let archive_time = now + archive_after;
            let id = content_id.clone();
            let s_type = storage_type;
            let archival_manager = self.archival_manager.clone();
            
            self.scheduler.schedule_task(
                archive_time,
                move || {
                    let _ = archival_manager.archive_object(&id, s_type);
                },
            )?;
        }
        
        // Schedule deletion if configured
        if let Some(delete_after) = policy.delete_after {
            let delete_time = now + delete_after;
            let id = content_id.clone();
            let s_type = storage_type;
            let garbage_collector = self.garbage_collector.clone();
            
            self.scheduler.schedule_task(
                delete_time,
                move || {
                    let _ = garbage_collector.collect_object(&id, s_type);
                },
            )?;
        }
        
        Ok(())
    }
    
    /// Run maintenance tasks
    pub fn run_maintenance(&self) -> Result<MaintenanceStats, LifecycleError> {
        let mut stats = MaintenanceStats::default();
        
        // Perform archival maintenance
        let archive_stats = self.archival_manager.run_maintenance()?;
        stats.archived_objects = archive_stats.archived_objects;
        stats.archive_size = archive_stats.archived_bytes;
        
        // Perform garbage collection
        let gc_stats = self.garbage_collector.run_collection()?;
        stats.deleted_objects = gc_stats.collected_objects;
        stats.reclaimed_space = gc_stats.reclaimed_bytes;
        
        Ok(stats)
    }
}
```

### Storage Policies

Policies control how data is stored and managed:

```rust
pub struct StoragePolicyEngine {
    /// Storage policies by storage type
    type_policies: HashMap<StorageType, Vec<StoragePolicy>>,
    
    /// Object class policies
    class_policies: HashMap<ObjectClass, Vec<StoragePolicy>>,
    
    /// Global policies that apply to all objects
    global_policies: Vec<StoragePolicy>,
    
    /// Content-based policy matcher
    content_matcher: Arc<ContentPolicyMatcher>,
}

impl StoragePolicyEngine {
    /// Create a new storage policy engine
    pub fn new(content_matcher: Arc<ContentPolicyMatcher>) -> Self {
        Self {
            type_policies: HashMap::new(),
            class_policies: HashMap::new(),
            global_policies: Vec::new(),
            content_matcher,
        }
    }
    
    /// Register a policy for a specific storage type
    pub fn register_type_policy(
        &mut self,
        storage_type: StorageType,
        policy: StoragePolicy,
    ) {
        self.type_policies.entry(storage_type)
            .or_insert_with(Vec::new)
            .push(policy);
    }
    
    /// Register a policy for a specific object class
    pub fn register_class_policy(
        &mut self,
        object_class: ObjectClass,
        policy: StoragePolicy,
    ) {
        self.class_policies.entry(object_class)
            .or_insert_with(Vec::new)
            .push(policy);
    }
    
    /// Register a global policy
    pub fn register_global_policy(&mut self, policy: StoragePolicy) {
        self.global_policies.push(policy);
    }
    
    /// Apply policies to determine effective storage options
    pub fn apply_policies<T: ContentAddressed>(
        &self,
        object: &T,
        storage_type: StorageType,
        base_options: StoreOptions,
    ) -> Result<StoreOptions, StorageError> {
        let mut effective_options = base_options.clone();
        
        // Apply content-based policies
        let content_policies = self.content_matcher.match_object(object)?;
        for policy in content_policies {
            effective_options = policy.apply(object, storage_type, effective_options)?;
        }
        
        // Apply global policies
        for policy in &self.global_policies {
            effective_options = policy.apply(object, storage_type, effective_options)?;
        }
        
        // Apply type-specific policies
        if let Some(policies) = self.type_policies.get(&storage_type) {
            for policy in policies {
                effective_options = policy.apply(object, storage_type, effective_options)?;
            }
        }
        
        // Apply class-specific policies if object class is known
        if let Some(class) = effective_options.object_class {
            if let Some(policies) = self.class_policies.get(&class) {
                for policy in policies {
                    effective_options = policy.apply(object, storage_type, effective_options)?;
                }
            }
        }
        
        // Validate final options
        self.validate_options(&effective_options)?;
        
        Ok(effective_options)
    }
    
    /// Validate storage options
    fn validate_options(&self, options: &StoreOptions) -> Result<(), StorageError> {
        // Check that retention policy is valid
        if let Some(policy) = &options.retention_policy {
            if let Some(expire_after) = policy.expire_after {
                if expire_after.as_secs() == 0 {
                    return Err(StorageError::InvalidPolicy(
                        "Expiration time must be greater than zero".to_string()
                    ));
                }
            }
        }
        
        // Check that replication factor is valid
        if let Some(factor) = options.replication_factor {
            if factor == 0 {
                return Err(StorageError::InvalidPolicy(
                    "Replication factor must be greater than zero".to_string()
                ));
            }
        }
        
        Ok(())
    }
}
```

### Garbage Collection

The garbage collector manages object deletion:

```rust
pub struct StorageGarbageCollector {
    /// Storage providers
    providers: HashMap<StorageType, Arc<dyn StorageProvider>>,
    
    /// Object registry for lifecycle management
    object_registry: Arc<RwLock<ObjectRegistry>>,
    
    /// Garbage collection options
    options: GarbageCollectionOptions,
    
    /// Worker pool for parallel collection
    worker_pool: Arc<ThreadPool>,
}

impl StorageGarbageCollector {
    /// Create a new storage garbage collector
    pub fn new(
        providers: HashMap<StorageType, Arc<dyn StorageProvider>>,
        object_registry: Arc<RwLock<ObjectRegistry>>,
        options: GarbageCollectionOptions,
    ) -> Self {
        let worker_pool = Arc::new(
            ThreadPool::new(options.worker_threads)
        );
        
        Self {
            providers,
            object_registry,
            options,
            worker_pool,
        }
    }
    
    /// Collect a specific object
    pub fn collect_object(
        &self,
        content_id: &ContentId,
        storage_type: StorageType,
    ) -> Result<bool, GarbageCollectionError> {
        // Check if the object can be collected
        if !self.can_collect(content_id, storage_type)? {
            return Ok(false);
        }
        
        // Get the provider for this storage type
        let provider = self.providers.get(&storage_type)
            .ok_or_else(|| GarbageCollectionError::UnsupportedStorageType(storage_type))?;
        
        // Delete the object
        provider.delete(content_id)?;
        
        // Remove from registry
        {
            let mut registry = self.object_registry.write()?;
            registry.remove_object(content_id, storage_type)?;
        }
        
        Ok(true)
    }
    
    /// Check if an object can be collected
    fn can_collect(
        &self,
        content_id: &ContentId,
        storage_type: StorageType,
    ) -> Result<bool, GarbageCollectionError> {
        let registry = self.object_registry.read()?;
        
        // Check if the object is registered
        if let Some(record) = registry.get_object(content_id, storage_type)? {
            // Check if it has dependencies
            if self.has_dependencies(content_id, storage_type)? {
                return Ok(false);
            }
            
            // Check if it's archived - archived objects should not be collected directly
            if record.is_archived() {
                return Ok(false);
            }
            
            // Check if it meets retention policies
            if !self.meets_retention_criteria(record)? {
                return Ok(false);
            }
            
            Ok(true)
        } else {
            // Object not in registry, can be collected
            Ok(true)
        }
    }
    
    /// Run a collection cycle
    pub fn run_collection(&self) -> Result<GarbageCollectionStats, GarbageCollectionError> {
        let mut stats = GarbageCollectionStats::default();
        let start_time = Instant::now();
        
        // Get objects eligible for collection
        let collection_candidates = self.get_collection_candidates()?;
        
        // Process candidates in parallel
        let results = Arc::new(Mutex::new(Vec::new()));
        let counter = Arc::new(AtomicUsize::new(0));
        
        for candidate in collection_candidates {
            let provider = self.providers.get(&candidate.storage_type).cloned();
            if provider.is_none() {
                continue;
            }
            
            let provider = provider.unwrap();
            let worker_results = results.clone();
            let worker_counter = counter.clone();
            let content_id = candidate.content_id.clone();
            let storage_type = candidate.storage_type;
            let registry = self.object_registry.clone();
            
            self.worker_pool.execute(move || {
                // Check if we can collect this object
                let can_collect = provider.can_delete(&content_id).unwrap_or(false);
                
                if can_collect {
                    // Delete the object
                    if let Ok(metadata) = provider.delete(&content_id) {
                        // Update stats
                        worker_counter.fetch_add(1, Ordering::SeqCst);
                        
                        // Remove from registry
                        let mut registry = registry.write().unwrap();
                        if let Ok(()) = registry.remove_object(&content_id, storage_type) {
                            // Record result
                            let mut results = worker_results.lock().unwrap();
                            results.push(GarbageCollectionResult {
                                content_id,
                                storage_type,
                                size: metadata.size,
                                success: true,
                            });
                        }
                    }
                }
            });
        }
        
        // Wait for all workers to complete
        self.worker_pool.join();
        
        // Compile statistics
        let collection_results = results.lock().unwrap();
        
        for result in collection_results.iter() {
            if result.success {
                stats.collected_objects += 1;
                stats.reclaimed_bytes += result.size as u64;
            }
        }
        
        stats.duration = start_time.elapsed();
        
        Ok(stats)
    }
}
```

### Archival Management

Archival management handles moving data to cold storage:

```rust
pub struct ArchivalManager {
    /// Active storage providers
    active_providers: HashMap<StorageType, Arc<dyn StorageProvider>>,
    
    /// Archive storage provider
    archive_provider: Arc<dyn ArchiveProvider>,
    
    /// Object registry for lifecycle management
    object_registry: Arc<RwLock<ObjectRegistry>>,
    
    /// Archival options
    options: ArchivalOptions,
}

impl ArchivalManager {
    /// Create a new archival manager
    pub fn new(
        active_providers: HashMap<StorageType, Arc<dyn StorageProvider>>,
        archive_provider: Arc<dyn ArchiveProvider>,
        object_registry: Arc<RwLock<ObjectRegistry>>,
        options: ArchivalOptions,
    ) -> Self {
        Self {
            active_providers,
            archive_provider,
            object_registry,
            options,
        }
    }
    
    /// Archive a specific object
    pub fn archive_object(
        &self,
        content_id: &ContentId,
        storage_type: StorageType,
    ) -> Result<ArchivalResult, ArchivalError> {
        // Check if the object is eligible for archival
        if !self.is_eligible_for_archival(content_id, storage_type)? {
            return Ok(ArchivalResult {
                content_id: content_id.clone(),
                storage_type,
                archived: false,
                size: 0,
                archive_key: None,
            });
        }
        
        // Get the provider for the active storage
        let provider = self.active_providers.get(&storage_type)
            .ok_or_else(|| ArchivalError::UnsupportedStorageType(storage_type))?;
        
        // Retrieve the object data
        let data = provider.get_bytes(content_id)?;
        
        // Get object metadata
        let metadata = provider.get_metadata(content_id)?;
        
        // Create archive options
        let archive_options = ArchiveOptions {
            content_id: content_id.clone(),
            storage_type,
            original_size: data.len(),
            original_metadata: metadata.clone(),
            archive_format: self.options.archive_format,
            compression_level: self.options.compression_level,
        };
        
        // Archive the object
        let archive_key = self.archive_provider.archive(
            content_id,
            &data,
            &archive_options,
        )?;
        
        // Update the object registry
        {
            let mut registry = self.object_registry.write()?;
            registry.mark_archived(
                content_id,
                storage_type,
                archive_key.clone(),
                SystemTime::now(),
            )?;
        }
        
        // If configured to delete after archival, delete the original
        if self.options.delete_after_archive {
            provider.delete(content_id)?;
        }
        
        Ok(ArchivalResult {
            content_id: content_id.clone(),
            storage_type,
            archived: true,
            size: data.len(),
            archive_key: Some(archive_key),
        })
    }
    
    /// Run archival maintenance
    pub fn run_maintenance(&self) -> Result<ArchivalStats, ArchivalError> {
        let mut stats = ArchivalStats::default();
        let start_time = Instant::now();
        
        // Get objects eligible for archival
        let archival_candidates = self.get_archival_candidates()?;
        
        // Process each candidate
        for candidate in archival_candidates {
            match self.archive_object(&candidate.content_id, candidate.storage_type) {
                Ok(result) => {
                    if result.archived {
                        stats.archived_objects += 1;
                        stats.archived_bytes += result.size as u64;
                    }
                },
                Err(e) => {
                    stats.errors += 1;
                    log::error!("Failed to archive object: {}", e);
                }
            }
        }
        
        stats.duration = start_time.elapsed();
        
        Ok(stats)
    }
    
    /// Check if an object is eligible for archival
    fn is_eligible_for_archival(
        &self,
        content_id: &ContentId,
        storage_type: StorageType,
    ) -> Result<bool, ArchivalError> {
        let registry = self.object_registry.read()?;
        
        // Check if the object is registered
        if let Some(record) = registry.get_object(content_id, storage_type)? {
            // If already archived, not eligible
            if record.is_archived() {
                return Ok(false);
            }
            
            // Check access patterns
            let now = SystemTime::now();
            if let Ok(last_access_age) = now.duration_since(record.last_accessed) {
                if last_access_age < self.options.min_age_for_archival {
                    return Ok(false);
                }
            }
            
            // Check size - skip small objects if configured
            if record.size < self.options.min_size_for_archival {
                return Ok(false);
            }
            
            // Check if storage type is eligible for archival
            if !self.options.eligible_storage_types.contains(&storage_type) {
                return Ok(false);
            }
            
            Ok(true)
        } else {
            // Object not in registry, not eligible
            Ok(false)
        }
    }
}
```

## Storage Provider Integration

The storage management system integrates with various storage providers:

```rust
pub struct ManagedStorageProvider {
    /// Base storage provider
    base_provider: Arc<dyn StorageProvider>,
    
    /// Storage type
    storage_type: StorageType,
    
    /// Storage management system
    management_system: Arc<StorageManagementSystem>,
    
    /// Metrics collector
    metrics_collector: Arc<StorageMetricsCollector>,
}

impl StorageProvider for ManagedStorageProvider {
    fn storage_type(&self) -> StorageType {
        self.storage_type
    }
    
    fn store<T: ContentAddressed>(
        &self,
        object: &T,
        options: &StoreOptions,
    ) -> Result<StorageMetadata, StorageError> {
        // Start metrics
        let timer = self.metrics_collector.start_timer(
            self.storage_type,
            StorageOperation::Store,
        );
        
        // Apply management policies
        let effective_options = self.management_system.policy_engine.apply_policies(
            object,
            self.storage_type,
            options.clone(),
        )?;
        
        // Store the object in the base provider
        let metadata = self.base_provider.store(object, &effective_options)?;
        
        // Register with lifecycle manager
        self.management_system.lifecycle_manager.register_object(
            metadata.content_id.clone(),
            self.storage_type,
            &effective_options,
        )?;
        
        // Record metrics
        timer.stop();
        self.metrics_collector.record_store_operation(
            self.storage_type,
            metadata.size,
        );
        
        Ok(metadata)
    }
    
    fn retrieve<T: ContentAddressed>(
        &self,
        content_id: &ContentId,
        options: &RetrieveOptions,
    ) -> Result<T, StorageError> {
        // Start metrics
        let timer = self.metrics_collector.start_timer(
            self.storage_type,
            StorageOperation::Retrieve,
        );
        
        // Check if object is archived
        let is_archived = self.management_system.lifecycle_manager.is_archived(
            content_id,
            self.storage_type,
        )?;
        
        if is_archived && !options.allow_archived {
            return Err(StorageError::ObjectArchived(content_id.clone()));
        }
        
        // Try to retrieve from base provider
        let result = self.base_provider.retrieve::<T>(content_id, options);
        
        // Record access regardless of result
        if result.is_ok() {
            self.management_system.lifecycle_manager.record_access(
                content_id,
                self.storage_type,
            )?;
        }
        
        // Record metrics
        timer.stop();
        if let Ok(ref object) = result {
            self.metrics_collector.record_retrieve_operation(
                self.storage_type,
                size_of(object),
            );
        }
        
        result
    }
    
    // Other methods delegate to base provider...
}
```

## Cross-Domain Storage Synchronization

Handling storage across domain boundaries:

```rust
pub struct CrossDomainStorageManager {
    /// Storage management for each domain
    domain_managers: HashMap<DomainId, Arc<StorageManagementSystem>>,
    
    /// Cross-domain synchronization service
    sync_service: Arc<SynchronizationService>,
    
    /// Cross-domain policy engine
    policy_engine: Arc<CrossDomainPolicyEngine>,
}

impl CrossDomainStorageManager {
    /// Create a new cross-domain storage manager
    pub fn new(
        sync_service: Arc<SynchronizationService>,
        policy_engine: Arc<CrossDomainPolicyEngine>,
    ) -> Self {
        Self {
            domain_managers: HashMap::new(),
            sync_service,
            policy_engine,
        }
    }
    
    /// Register a storage management system for a domain
    pub fn register_domain(
        &mut self,
        domain_id: DomainId,
        management_system: Arc<StorageManagementSystem>,
    ) {
        self.domain_managers.insert(domain_id, management_system);
    }
    
    /// Store an object with cross-domain replication
    pub fn store_with_replication<T: ContentAddressed>(
        &self,
        object: &T,
        primary_domain: &DomainId,
        replica_domains: &[DomainId],
        options: StoreOptions,
    ) -> Result<CrossDomainStorageResult, StorageError> {
        // Check primary domain exists
        let primary_manager = self.domain_managers.get(primary_domain)
            .ok_or_else(|| StorageError::UnsupportedDomain(primary_domain.clone()))?;
        
        // Check if replication is allowed by policy
        self.policy_engine.validate_replication(
            primary_domain,
            replica_domains,
            &options.object_class.unwrap_or_default(),
        )?;
        
        // Store in primary domain
        let primary_metadata = primary_manager.store(object, options.clone())?;
        
        // Prepare replication
        let mut replica_results = Vec::new();
        
        // Create replication options - adjust as needed for each domain
        let replication_options = options.with_replication_source(
            primary_domain.clone(),
            primary_metadata.content_id.clone(),
        );
        
        // Replicate to target domains
        for domain_id in replica_domains {
            if let Some(domain_manager) = self.domain_managers.get(domain_id) {
                match domain_manager.store(object, replication_options.clone()) {
                    Ok(metadata) => {
                        replica_results.push(DomainStorageResult {
                            domain_id: domain_id.clone(),
                            content_id: metadata.content_id,
                            success: true,
                            error: None,
                        });
                        
                        // Register cross-domain relationship
                        self.sync_service.register_replica(
                            primary_domain,
                            primary_metadata.content_id.clone(),
                            domain_id,
                            metadata.content_id,
                        )?;
                    },
                    Err(e) => {
                        replica_results.push(DomainStorageResult {
                            domain_id: domain_id.clone(),
                            content_id: ContentId::default(),
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        }
        
        Ok(CrossDomainStorageResult {
            primary_domain: primary_domain.clone(),
            primary_content_id: primary_metadata.content_id,
            replica_results,
        })
    }
    
    /// Synchronize storage across domains
    pub fn synchronize_domains(
        &self,
        source_domain: &DomainId,
        target_domains: &[DomainId],
        object_filter: Option<ObjectFilter>,
    ) -> Result<SynchronizationResult, StorageError> {
        // Get the source domain management system
        let source_manager = self.domain_managers.get(source_domain)
            .ok_or_else(|| StorageError::UnsupportedDomain(source_domain.clone()))?;
        
        // Get objects to synchronize
        let objects_to_sync = if let Some(filter) = object_filter {
            source_manager.lifecycle_manager.get_objects_matching_filter(&filter)?
        } else {
            source_manager.lifecycle_manager.get_all_objects()?
        };
        
        // Track synchronization results
        let mut results = SynchronizationResult {
            source_domain: source_domain.clone(),
            target_domains: target_domains.to_vec(),
            total_objects: objects_to_sync.len(),
            synced_objects: 0,
            failed_objects: 0,
            total_bytes: 0,
            domain_results: HashMap::new(),
        };
        
        // Initialize domain results
        for domain_id in target_domains {
            results.domain_results.insert(domain_id.clone(), DomainSyncResult {
                synced_objects: 0,
                failed_objects: 0,
                total_bytes: 0,
            });
        }
        
        // Synchronize each object
        for object_record in objects_to_sync {
            let content_id = object_record.content_id;
            let storage_type = object_record.storage_type;
            
            // Get the object data
            let object_data = match source_manager.get_provider(storage_type)?.get_bytes(&content_id) {
                Ok(data) => data,
                Err(_) => {
                    results.failed_objects += 1;
                    continue;
                }
            };
            
            // Sync to each target domain
            for domain_id in target_domains {
                if let Some(target_manager) = self.domain_managers.get(domain_id) {
                    // Check if replication is allowed by policy
                    if !self.policy_engine.is_replication_allowed(
                        source_domain,
                        domain_id,
                        &object_record.metadata.object_class.unwrap_or_default(),
                    ) {
                        continue;
                    }
                    
                    // Create storage options for the target domain
                    let target_options = StoreOptions::new()
                        .with_replication_source(
                            source_domain.clone(),
                            content_id.clone(),
                        )
                        .with_object_class(
                            object_record.metadata.object_class.clone(),
                        );
                    
                    // Store in target domain
                    match target_manager.get_provider(storage_type)?.store_bytes(
                        &content_id,
                        &object_data,
                        &target_options,
                    ) {
                        Ok(_) => {
                            // Update domain result
                            if let Some(domain_result) = results.domain_results.get_mut(domain_id) {
                                domain_result.synced_objects += 1;
                                domain_result.total_bytes += object_data.len() as u64;
                            }
                            
                            // Register cross-domain relationship
                            self.sync_service.register_replica(
                                source_domain,
                                content_id.clone(),
                                domain_id,
                                content_id.clone(), // Same content ID in both domains
                            )?;
                            
                            // Update overall results
                            results.synced_objects += 1;
                            results.total_bytes += object_data.len() as u64;
                        },
                        Err(_) => {
                            // Update domain result
                            if let Some(domain_result) = results.domain_results.get_mut(domain_id) {
                                domain_result.failed_objects += 1;
                            }
                            
                            // Update overall results
                            results.failed_objects += 1;
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
}
```

## Storage Metrics

Metrics collection for storage operations:

```rust
pub struct StorageMetricsCollector {
    /// Storage operation counters
    counters: HashMap<String, Counter>,
    
    /// Storage operation timers
    timers: HashMap<String, Timer>,
    
    /// Storage size gauges
    gauges: HashMap<String, Gauge>,
    
    /// Metrics registry
    metrics_registry: Arc<MetricsRegistry>,
}

impl StorageMetricsCollector {
    /// Create a new storage metrics collector
    pub fn new(metrics_registry: Arc<MetricsRegistry>) -> Self {
        let mut collector = Self {
            counters: HashMap::new(),
            timers: HashMap::new(),
            gauges: HashMap::new(),
            metrics_registry,
        };
        
        // Initialize metrics
        collector.initialize_metrics();
        
        collector
    }
    
    /// Initialize storage metrics
    fn initialize_metrics(&mut self) {
        // Create metrics for each storage type
        for storage_type in StorageType::all() {
            let type_name = storage_type.to_string();
            
            // Operation counters
            for op in &["store", "retrieve", "delete", "exists", "list"] {
                let counter_name = format!("storage.{}.{}.count", type_name, op);
                let counter = self.metrics_registry.create_counter(
                    &counter_name,
                    &format!("Number of {} operations on {} storage", op, type_name),
                );
                self.counters.insert(counter_name, counter);
                
                // Success/failure counters
                let success_counter = self.metrics_registry.create_counter(
                    &format!("storage.{}.{}.success", type_name, op),
                    &format!("Number of successful {} operations on {} storage", op, type_name),
                );
                self.counters.insert(format!("storage.{}.{}.success", type_name, op), success_counter);
                
                let error_counter = self.metrics_registry.create_counter(
                    &format!("storage.{}.{}.error", type_name, op),
                    &format!("Number of failed {} operations on {} storage", op, type_name),
                );
                self.counters.insert(format!("storage.{}.{}.error", type_name, op), error_counter);
            }
            
            // Operation timers
            for op in &["store", "retrieve", "delete", "exists", "list"] {
                let timer_name = format!("storage.{}.{}.duration", type_name, op);
                let timer = self.metrics_registry.create_timer(
                    &timer_name,
                    &format!("Duration of {} operations on {} storage", op, type_name),
                );
                self.timers.insert(timer_name, timer);
            }
            
            // Size gauge
            let size_gauge = self.metrics_registry.create_gauge(
                &format!("storage.{}.size", type_name),
                &format!("Total size of {} storage in bytes", type_name),
            );
            self.gauges.insert(format!("storage.{}.size", type_name), size_gauge);
            
            // Count gauge
            let count_gauge = self.metrics_registry.create_gauge(
                &format!("storage.{}.count", type_name),
                &format!("Total number of objects in {} storage", type_name),
            );
            self.gauges.insert(format!("storage.{}.count", type_name), count_gauge);
        }
        
        // Lifecycle metrics
        let archived_counter = self.metrics_registry.create_counter(
            "storage.lifecycle.archived",
            "Number of objects archived",
        );
        self.counters.insert("storage.lifecycle.archived".to_string(), archived_counter);
        
        let deleted_counter = self.metrics_registry.create_counter(
            "storage.lifecycle.deleted",
            "Number of objects deleted by lifecycle management",
        );
        self.counters.insert("storage.lifecycle.deleted".to_string(), deleted_counter);
        
        // Cache metrics
        let cache_hit_counter = self.metrics_registry.create_counter(
            "storage.cache.hits",
            "Number of cache hits",
        );
        self.counters.insert("storage.cache.hits".to_string(), cache_hit_counter);
        
        let cache_miss_counter = self.metrics_registry.create_counter(
            "storage.cache.misses",
            "Number of cache misses",
        );
        self.counters.insert("storage.cache.misses".to_string(), cache_miss_counter);
    }
    
    /// Record a storage operation
    pub fn record_operation(
        &self,
        storage_type: StorageType,
        operation: StorageOperation,
        result: &Result<(), StorageError>,
        duration: Duration,
    ) {
        let type_name = storage_type.to_string();
        let op_name = operation.to_string();
        
        // Increment operation counter
        if let Some(counter) = self.counters.get(&format!("storage.{}.{}.count", type_name, op_name)) {
            counter.increment(1);
        }
        
        // Record operation duration
        if let Some(timer) = self.timers.get(&format!("storage.{}.{}.duration", type_name, op_name)) {
            timer.record(duration);
        }
        
        // Record success/failure
        let result_counter_name = match result {
            Ok(_) => format!("storage.{}.{}.success", type_name, op_name),
            Err(_) => format!("storage.{}.{}.error", type_name, op_name),
        };
        
        if let Some(counter) = self.counters.get(&result_counter_name) {
            counter.increment(1);
        }
    }
    
    /// Update storage size metric
    pub fn update_size(&self, storage_type: StorageType, size: u64) {
        let type_name = storage_type.to_string();
        
        if let Some(gauge) = self.gauges.get(&format!("storage.{}.size", type_name)) {
            gauge.set(size as f64);
        }
    }
    
    /// Start a timer for an operation
    pub fn start_timer(
        &self,
        storage_type: StorageType,
        operation: StorageOperation,
    ) -> OperationTimer {
        let start_time = Instant::now();
        
        OperationTimer {
            collector: self,
            storage_type,
            operation,
            start_time,
            completed: false,
        }
    }
}
```

## Usage Examples

### Basic Storage Management

```rust
// Get the storage management system
let storage_mgmt = system.storage_management_system();

// Store a resource with lifecycle options
let resource = Resource::new("resource-1", "Sample resource");

let options = StoreOptions::new()
    .with_storage_type(StorageType::Resource)
    .with_retention_policy(RetentionPolicy {
        keep_for: Duration::from_days(30),
        archive_after: Some(Duration::from_days(7)),
        expire_after: Some(Duration::from_days(30)),
    });

let metadata = storage_mgmt.store(&resource, options)?;

println!("Stored resource with ID: {}", metadata.content_id);

// Later, retrieve the resource
let retrieved_resource = storage_mgmt.retrieve::<Resource>(
    &metadata.content_id,
    RetrieveOptions::default()
)?;

println!("Retrieved resource: {}", retrieved_resource.name);
```

### Storage Lifecycle Management

```rust
// Get the lifecycle manager
let lifecycle = storage_mgmt.lifecycle_manager();

// Check if an object is eligible for archival
let is_eligible = lifecycle.is_eligible_for_archival(
    &content_id,
    StorageType::Resource,
)?;

if is_eligible {
    // Archive the object
    let result = lifecycle.archive_object(
        &content_id,
        StorageType::Resource,
    )?;
    
    println!("Archived object: {}, size: {} bytes", 
        result.content_id, result.size);
}

// Run lifecycle maintenance to automatically process eligible objects
let stats = lifecycle.run_maintenance()?;

println!("Maintenance completed:");
println!("  Archived objects: {}", stats.archived_objects);
println!("  Archived bytes: {}", stats.archive_size);
println!("  Deleted objects: {}", stats.deleted_objects);
println!("  Reclaimed space: {}", stats.reclaimed_space);
```

### Cross-Domain Storage Replication

```rust
// Get cross-domain storage manager
let cross_domain = system.cross_domain_storage_manager();

// Define domains
let source_domain = DomainId::new("domain-a");
let target_domains = vec![
    DomainId::new("domain-b"),
    DomainId::new("domain-c"),
];

// Store with replication
let resource = Resource::new("cross-domain-resource", "Replicated resource");

let result = cross_domain.store_with_replication(
    &resource,
    &source_domain,
    &target_domains,
    StoreOptions::default(),
)?;

println!("Stored in primary domain: {}", result.primary_domain);
println!("Primary content ID: {}", result.primary_content_id);

for replica in &result.replica_results {
    if replica.success {
        println!("Replica in domain {}: {}", replica.domain_id, replica.content_id);
    } else {
        println!("Failed to replicate to domain {}: {}", 
            replica.domain_id, replica.error.unwrap_or_default());
    }
}

// Synchronize domains
let sync_result = cross_domain.synchronize_domains(
    &source_domain,
    &target_domains,
    None, // Synchronize all objects
)?;

println!("Synchronization completed:");
println!("  Total objects: {}", sync_result.total_objects);
println!("  Synced objects: {}", sync_result.synced_objects);
println!("  Failed objects: {}", sync_result.failed_objects);
println!("  Total bytes: {}", sync_result.total_bytes);
```

### Storage Metrics and Monitoring

```rust
// Get the metrics collector
let metrics = storage_mgmt.metrics_collector();

// Get storage metrics for a specific type
let resource_size = metrics.get_storage_size(StorageType::Resource);
let resource_count = metrics.get_storage_count(StorageType::Resource);

println!("Resource storage:");
println!("  Total objects: {}", resource_count);
println!("  Total size: {} bytes", resource_size);

// Get operation statistics
let store_ops = metrics.get_operation_count(
    StorageType::Resource,
    StorageOperation::Store,
);

let retrieve_ops = metrics.get_operation_count(
    StorageType::Resource,
    StorageOperation::Retrieve,
);

println!("Operations:");
println!("  Store operations: {}", store_ops);
println!("  Retrieve operations: {}", retrieve_ops);

// Get timing statistics
let avg_store_time = metrics.get_operation_average_time(
    StorageType::Resource,
    StorageOperation::Store,
);

let avg_retrieve_time = metrics.get_operation_average_time(
    StorageType::Resource,
    StorageOperation::Retrieve,
);

println!("Timing:");
println!("  Average store time: {:?}", avg_store_time);
println!("  Average retrieve time: {:?}", avg_retrieve_time);
```

## Implementation Status

The current implementation status of Storage Management:

- ✅ Core storage management interfaces
- ✅ Segment management for log storage
- ⚠️ Storage lifecycle management (partially implemented)
- ⚠️ Archival management (partially implemented)
- ⚠️ Garbage collection (partially implemented)
- ❌ Cross-domain storage synchronization (not yet implemented)
- ❌ Storage metrics collection (not yet implemented)

## Future Enhancements

Planned future enhancements for Storage Management:

1. **Distributed Storage Management**: Coordinated storage management across multiple nodes
2. **Advanced Archival Strategies**: Tiered archival policies based on data importance
3. **Predictive Caching**: ML-based prediction of access patterns for cache optimization
4. **Storage Deduplication**: Automatic detection and management of duplicate content
5. **Storage Compression Analysis**: Automatic selection of optimal compression algorithms
6. **Storage SLAs**: Service level agreements for storage operations
7. **Adaptive Storage Policies**: Self-tuning storage policies based on usage patterns
8. **Backup Integration**: Integration with external backup systems
9. **Content-Aware Storage**: Storage optimizations based on content analysis