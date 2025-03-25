<!-- Architecture for storage -->
<!-- Original file: docs/src/storage_architecture.md -->

# Storage Architecture in Causality

## Overview

This document describes the storage architecture of the Causality system. The storage subsystem is responsible for persisting all system data, including resources, facts, operations, transactions, and state information. The architecture is designed to provide durability, consistency, performance, and security, while supporting the temporal nature of the Causality system.

## Core Concepts

### Storage Model

The storage system is built around these core components:

```rust
pub struct StorageSystem {
    /// Storage providers for different storage types
    providers: HashMap<StorageType, Box<dyn StorageProvider>>,
    
    /// Storage policy manager
    policy_manager: Arc<StoragePolicyManager>,
    
    /// Storage transaction manager
    transaction_manager: Arc<StorageTransactionManager>,
    
    /// Storage encryption manager
    encryption_manager: Arc<StorageEncryptionManager>,
    
    /// Storage metrics collector
    metrics_collector: Arc<StorageMetricsCollector>,
}

pub enum StorageType {
    /// Resource storage
    Resource,
    
    /// Fact storage
    Fact,
    
    /// Operation storage
    Operation,
    
    /// Transaction storage
    Transaction,
    
    /// Registry storage
    Registry,
    
    /// Metadata storage
    Metadata,
    
    /// Blob storage
    Blob,
}
```

### Storage Provider Interface

Storage providers implement a common interface:

```rust
pub trait StorageProvider: Send + Sync {
    /// Get the type of storage this provider handles
    fn storage_type(&self) -> StorageType;
    
    /// Store a data item
    fn store(&self, key: &StorageKey, value: &[u8], options: &StorageOptions) 
        -> Result<StorageMetadata, StorageError>;
    
    /// Retrieve a data item
    fn retrieve(&self, key: &StorageKey, options: &RetrieveOptions) 
        -> Result<StorageItem, StorageError>;
    
    /// Check if a key exists
    fn exists(&self, key: &StorageKey) -> Result<bool, StorageError>;
    
    /// Delete a data item
    fn delete(&self, key: &StorageKey, options: &DeleteOptions) 
        -> Result<(), StorageError>;
    
    /// List keys matching a pattern
    fn list_keys(&self, pattern: &KeyPattern) 
        -> Result<Vec<StorageKey>, StorageError>;
    
    /// Begin a storage transaction
    fn begin_transaction(&self) -> Result<StorageTransaction, StorageError>;
    
    /// Commit a storage transaction
    fn commit_transaction(&self, transaction: &StorageTransaction) 
        -> Result<(), StorageError>;
    
    /// Rollback a storage transaction
    fn rollback_transaction(&self, transaction: &StorageTransaction) 
        -> Result<(), StorageError>;
}
```

## Storage System Architecture

### Provider Registry

Manages storage providers:

```rust
pub struct StorageProviderRegistry {
    providers: RwLock<HashMap<StorageType, Box<dyn StorageProvider>>>,
}

impl StorageProviderRegistry {
    /// Register a storage provider
    pub fn register_provider(
        &self,
        provider: Box<dyn StorageProvider>,
    ) -> Result<(), RegistryError> {
        let storage_type = provider.storage_type();
        
        let mut providers = self.providers.write().unwrap();
        
        // Check if we already have a provider for this type
        if providers.contains_key(&storage_type) {
            return Err(RegistryError::AlreadyExists(
                format!("Provider for storage type {:?} already registered", storage_type)
            ));
        }
        
        // Register the provider
        providers.insert(storage_type, provider);
        
        Ok(())
    }
    
    /// Get a storage provider by type
    pub fn get_provider(
        &self,
        storage_type: StorageType,
    ) -> Result<Arc<dyn StorageProvider>, RegistryError> {
        let providers = self.providers.read().unwrap();
        
        providers.get(&storage_type)
            .map(|p| Arc::new(p.as_ref().clone()) as Arc<dyn StorageProvider>)
            .ok_or_else(|| RegistryError::NotFound(
                format!("No provider registered for storage type {:?}", storage_type)
            ))
    }
}
```

### Transaction Management

Handling storage transactions:

```rust
pub struct StorageTransactionManager {
    /// Active transactions
    active_transactions: RwLock<HashMap<TransactionId, StorageTransaction>>,
    
    /// Transaction observers
    observers: Vec<Box<dyn TransactionObserver>>,
}

impl StorageTransactionManager {
    /// Begin a new storage transaction
    pub fn begin_transaction(&self) -> Result<StorageTransaction, TransactionError> {
        let transaction_id = TransactionId::generate();
        
        let transaction = StorageTransaction {
            id: transaction_id,
            operations: Vec::new(),
            status: TransactionStatus::Active,
            start_time: system.current_time(),
            commit_time: None,
        };
        
        // Store in active transactions
        {
            let mut active = self.active_transactions.write().unwrap();
            active.insert(transaction_id, transaction.clone());
        }
        
        // Notify observers
        for observer in &self.observers {
            observer.on_transaction_begin(&transaction)?;
        }
        
        Ok(transaction)
    }
    
    /// Commit a storage transaction
    pub fn commit_transaction(
        &self,
        transaction_id: TransactionId,
    ) -> Result<(), TransactionError> {
        // Get the transaction
        let transaction = {
            let active = self.active_transactions.read().unwrap();
            active.get(&transaction_id)
                .ok_or_else(|| TransactionError::NotFound(transaction_id))?
                .clone()
        };
        
        // Check if transaction is active
        if transaction.status != TransactionStatus::Active {
            return Err(TransactionError::InvalidStatus(
                format!("Transaction is not active: {:?}", transaction.status)
            ));
        }
        
        // Commit on all affected providers
        let providers = self.get_affected_providers(&transaction)?;
        
        for provider in providers {
            provider.commit_transaction(&transaction)?;
        }
        
        // Update transaction status
        {
            let mut active = self.active_transactions.write().unwrap();
            if let Some(tx) = active.get_mut(&transaction_id) {
                tx.status = TransactionStatus::Committed;
                tx.commit_time = Some(system.current_time());
            }
        }
        
        // Notify observers
        for observer in &self.observers {
            observer.on_transaction_commit(&transaction)?;
        }
        
        // Remove from active transactions after some time
        let transaction_manager = self.clone();
        system.scheduler().schedule_after(
            Duration::from_secs(300), // Keep for 5 minutes for query purposes
            move || {
                let _ = transaction_manager.remove_transaction(transaction_id);
            },
        );
        
        Ok(())
    }
    
    /// Rollback a storage transaction
    pub fn rollback_transaction(
        &self,
        transaction_id: TransactionId,
    ) -> Result<(), TransactionError> {
        // Similar implementation to commit but with rollback semantics
        // ...
        
        Ok(())
    }
}
```

## Storage Implementations

### Key-Value Store

Basic key-value storage:

```rust
pub struct KeyValueStorageProvider {
    /// The underlying key-value store
    store: Arc<dyn KeyValueStore>,
    
    /// Storage type this provider handles
    storage_type: StorageType,
    
    /// Cache for frequently accessed items
    cache: Option<Arc<StorageCache>>,
}

impl StorageProvider for KeyValueStorageProvider {
    fn storage_type(&self) -> StorageType {
        self.storage_type
    }
    
    fn store(&self, key: &StorageKey, value: &[u8], options: &StorageOptions) 
        -> Result<StorageMetadata, StorageError> {
        let store_options = self.convert_options(options);
        
        let result = self.store.put(key.to_string(), value.to_vec(), &store_options)?;
        
        // Update cache if enabled
        if let Some(cache) = &self.cache {
            if options.cacheable {
                cache.put(key.clone(), value.to_vec(), options.ttl)?;
            }
        }
        
        Ok(StorageMetadata {
            key: key.clone(),
            size: value.len(),
            stored_at: system.current_time(),
            expires_at: options.ttl.map(|ttl| system.current_time() + ttl),
            checksum: calculate_checksum(value),
        })
    }
    
    fn retrieve(&self, key: &StorageKey, options: &RetrieveOptions) 
        -> Result<StorageItem, StorageError> {
        // Check cache first if enabled
        if let Some(cache) = &self.cache {
            if let Some(cached_value) = cache.get(key)? {
                return Ok(StorageItem {
                    key: key.clone(),
                    value: cached_value,
                    metadata: StorageMetadata {
                        key: key.clone(),
                        size: cached_value.len(),
                        stored_at: system.current_time(), // Approximate
                        expires_at: None,                 // Unknown from cache
                        checksum: calculate_checksum(&cached_value),
                    },
                    from_cache: true,
                });
            }
        }
        
        // Not in cache, retrieve from store
        let retrieve_options = self.convert_retrieve_options(options);
        let value = self.store.get(key.to_string(), &retrieve_options)?;
        
        if let Some(v) = value {
            // Update cache if enabled
            if let Some(cache) = &self.cache {
                cache.put(key.clone(), v.clone(), None)?;
            }
            
            return Ok(StorageItem {
                key: key.clone(),
                value: v.clone(),
                metadata: StorageMetadata {
                    key: key.clone(),
                    size: v.len(),
                    stored_at: system.current_time(), // Approximate, could get from store metadata
                    expires_at: None,
                    checksum: calculate_checksum(&v),
                },
                from_cache: false,
            });
        }
        
        Err(StorageError::NotFound(key.clone()))
    }
    
    // Other implementation methods...
}
```

### Document Store

Storage for structured documents:

```rust
pub struct DocumentStorageProvider {
    /// The underlying document store
    store: Arc<dyn DocumentStore>,
    
    /// Storage type this provider handles
    storage_type: StorageType,
    
    /// Schema registry
    schema_registry: Arc<SchemaRegistry>,
}

impl StorageProvider for DocumentStorageProvider {
    // Implementation of StorageProvider methods
    // ...
    
    // Document-specific methods
    
    /// Store a document
    pub fn store_document<T: Serialize>(
        &self,
        collection: &str,
        id: &str,
        document: &T,
        options: &DocumentOptions,
    ) -> Result<DocumentMetadata, StorageError> {
        // Serialize document
        let doc_data = serde_json::to_vec(document)?;
        
        // Create storage key
        let key = StorageKey::new(format!("{}/{}", collection, id));
        
        // Store using base implementation
        let metadata = self.store(&key, &doc_data, &options.into())?;
        
        // Convert to document metadata
        Ok(DocumentMetadata {
            collection: collection.to_string(),
            id: id.to_string(),
            key: key,
            size: metadata.size,
            stored_at: metadata.stored_at,
            expires_at: metadata.expires_at,
            checksum: metadata.checksum,
            schema_id: options.schema_id.clone(),
        })
    }
    
    /// Retrieve a document
    pub fn retrieve_document<T: DeserializeOwned>(
        &self,
        collection: &str,
        id: &str,
        options: &RetrieveOptions,
    ) -> Result<T, StorageError> {
        // Create storage key
        let key = StorageKey::new(format!("{}/{}", collection, id));
        
        // Retrieve using base implementation
        let item = self.retrieve(&key, options)?;
        
        // Deserialize document
        let document = serde_json::from_slice(&item.value)?;
        
        Ok(document)
    }
    
    /// Query documents
    pub fn query_documents<T: DeserializeOwned>(
        &self,
        collection: &str,
        query: &DocumentQuery,
        options: &QueryOptions,
    ) -> Result<Vec<T>, StorageError> {
        let store_query = self.convert_query(collection, query);
        let store_options = self.convert_query_options(options);
        
        let results = self.store.query(collection, &store_query, &store_options)?;
        
        // Deserialize results
        let documents = results.iter()
            .map(|doc_data| serde_json::from_slice(doc_data))
            .collect::<Result<Vec<T>, _>>()?;
        
        Ok(documents)
    }
}
```

## Storage Security

### Encryption

Secure storage with encryption:

```rust
pub struct StorageEncryptionManager {
    /// Encryption key provider
    key_provider: Arc<dyn EncryptionKeyProvider>,
    
    /// Encryption engines for different algorithms
    engines: HashMap<EncryptionAlgorithm, Box<dyn EncryptionEngine>>,
    
    /// Default encryption algorithm
    default_algorithm: EncryptionAlgorithm,
}

impl StorageEncryptionManager {
    /// Encrypt data for storage
    pub fn encrypt(
        &self,
        data: &[u8],
        context: &EncryptionContext,
    ) -> Result<EncryptedData, EncryptionError> {
        // Get the algorithm to use
        let algorithm = context.algorithm.unwrap_or(self.default_algorithm);
        
        // Get the encryption engine
        let engine = self.engines.get(&algorithm)
            .ok_or_else(|| EncryptionError::UnsupportedAlgorithm(algorithm))?;
        
        // Get the encryption key
        let key_id = context.key_id.clone().unwrap_or_else(|| {
            self.key_provider.get_default_key_id(algorithm)
        });
        
        let key = self.key_provider.get_encryption_key(key_id.clone(), algorithm)?;
        
        // Encrypt the data
        let encrypted = engine.encrypt(data, &key, context.additional_data.as_deref())?;
        
        Ok(EncryptedData {
            ciphertext: encrypted,
            algorithm,
            key_id,
            additional_data: context.additional_data.clone(),
            created_at: system.current_time(),
        })
    }
    
    /// Decrypt data from storage
    pub fn decrypt(
        &self,
        encrypted_data: &EncryptedData,
        additional_data: Option<&[u8]>,
    ) -> Result<Vec<u8>, EncryptionError> {
        // Get the encryption engine
        let engine = self.engines.get(&encrypted_data.algorithm)
            .ok_or_else(|| EncryptionError::UnsupportedAlgorithm(encrypted_data.algorithm))?;
        
        // Get the decryption key
        let key = self.key_provider.get_encryption_key(
            encrypted_data.key_id.clone(),
            encrypted_data.algorithm,
        )?;
        
        // Use additional data from context or encrypted data
        let aad = additional_data.or_else(|| encrypted_data.additional_data.as_deref());
        
        // Decrypt the data
        let decrypted = engine.decrypt(&encrypted_data.ciphertext, &key, aad)?;
        
        Ok(decrypted)
    }
}
```

### Access Control

Controlling access to stored data:

```rust
pub struct StorageAccessController {
    /// Access policy provider
    policy_provider: Arc<dyn AccessPolicyProvider>,
    
    /// Identity resolver
    identity_resolver: Arc<IdentityResolver>,
    
    /// Access decision cache
    decision_cache: Option<Arc<AccessDecisionCache>>,
}

impl StorageAccessController {
    /// Check if an identity has access to a storage key
    pub fn check_access(
        &self,
        identity: &Identity,
        key: &StorageKey,
        access_type: AccessType,
    ) -> Result<AccessDecision, AccessControlError> {
        // Check cache first if enabled
        if let Some(cache) = &self.decision_cache {
            if let Some(decision) = cache.get_decision(identity, key, access_type)? {
                return Ok(decision);
            }
        }
        
        // Resolve identity to get attributes
        let identity_attributes = self.identity_resolver.resolve_attributes(identity)?;
        
        // Get applicable policies
        let policies = self.policy_provider.get_policies_for_key(key)?;
        
        // Evaluate policies
        let mut decision = AccessDecision::default();
        
        for policy in policies {
            let policy_decision = policy.evaluate(
                identity,
                &identity_attributes,
                key,
                access_type,
            )?;
            
            // Apply decision according to policy type
            decision = decision.combine(policy_decision);
            
            // If we have a definitive deny, stop evaluating
            if matches!(decision, AccessDecision::Deny { .. }) {
                break;
            }
        }
        
        // Default to deny if no policies matched
        let final_decision = if decision.is_undetermined() {
            AccessDecision::Deny {
                reason: "No applicable policies found".to_string(),
            }
        } else {
            decision
        };
        
        // Update cache if enabled
        if let Some(cache) = &self.decision_cache {
            cache.cache_decision(identity.clone(), key.clone(), access_type, final_decision.clone())?;
        }
        
        Ok(final_decision)
    }
}
```

## Storage Observers and Metrics

### Storage Metrics

Collecting storage metrics:

```rust
pub struct StorageMetricsCollector {
    /// Metrics registry
    metrics_registry: Arc<MetricsRegistry>,
    
    /// Storage operation counters
    counters: HashMap<String, Counter>,
    
    /// Storage operation timers
    timers: HashMap<String, Timer>,
    
    /// Storage size gauges
    gauges: HashMap<String, Gauge>,
}

impl StorageMetricsCollector {
    /// Record a storage operation
    pub fn record_operation(
        &self,
        operation_type: &str,
        storage_type: StorageType,
        result: &Result<(), StorageError>,
        duration: Duration,
    ) {
        // Increment operation counter
        let counter_name = format!("storage.{}.{}.count", storage_type, operation_type);
        if let Some(counter) = self.counters.get(&counter_name) {
            counter.increment(1);
        }
        
        // Record operation duration
        let timer_name = format!("storage.{}.{}.duration", storage_type, operation_type);
        if let Some(timer) = self.timers.get(&timer_name) {
            timer.record(duration);
        }
        
        // Record success/failure
        let result_counter_name = match result {
            Ok(_) => format!("storage.{}.{}.success", storage_type, operation_type),
            Err(_) => format!("storage.{}.{}.error", storage_type, operation_type),
        };
        
        if let Some(counter) = self.counters.get(&result_counter_name) {
            counter.increment(1);
        }
    }
    
    /// Update storage size
    pub fn update_size(
        &self,
        storage_type: StorageType,
        size: u64,
    ) {
        let gauge_name = format!("storage.{}.size", storage_type);
        if let Some(gauge) = self.gauges.get(&gauge_name) {
            gauge.set(size as f64);
        }
    }
}
```

### Storage Observers

Observing storage operations:

```rust
pub trait StorageObserver: Send + Sync {
    /// Called when an item is stored
    fn on_store(
        &self,
        key: &StorageKey,
        metadata: &StorageMetadata,
        provider_type: StorageType,
    ) -> Result<(), ObserverError>;
    
    /// Called when an item is retrieved
    fn on_retrieve(
        &self,
        key: &StorageKey,
        item: &StorageItem,
        provider_type: StorageType,
    ) -> Result<(), ObserverError>;
    
    /// Called when an item is deleted
    fn on_delete(
        &self,
        key: &StorageKey,
        provider_type: StorageType,
    ) -> Result<(), ObserverError>;
    
    /// Called when a storage transaction begins
    fn on_transaction_begin(
        &self,
        transaction: &StorageTransaction,
    ) -> Result<(), ObserverError>;
    
    /// Called when a storage transaction commits
    fn on_transaction_commit(
        &self,
        transaction: &StorageTransaction,
    ) -> Result<(), ObserverError>;
    
    /// Called when a storage transaction rolls back
    fn on_transaction_rollback(
        &self,
        transaction: &StorageTransaction,
    ) -> Result<(), ObserverError>;
}
```

## Configuration and Setup

### Storage System Configuration

Setting up the storage system:

```rust
pub struct StorageSystemConfig {
    /// Provider configurations by storage type
    providers: HashMap<StorageType, ProviderConfig>,
    
    /// Cache configuration
    cache_config: Option<CacheConfig>,
    
    /// Encryption configuration
    encryption_config: Option<EncryptionConfig>,
    
    /// Transaction configuration
    transaction_config: TransactionConfig,
    
    /// Metrics configuration
    metrics_config: Option<MetricsConfig>,
}

pub struct StorageSystemBuilder {
    config: StorageSystemConfig,
    provider_factories: HashMap<String, Box<dyn ProviderFactory>>,
    key_provider: Option<Box<dyn EncryptionKeyProvider>>,
    metrics_registry: Option<Arc<MetricsRegistry>>,
}

impl StorageSystemBuilder {
    pub fn new() -> Self {
        Self {
            config: StorageSystemConfig::default(),
            provider_factories: HashMap::new(),
            key_provider: None,
            metrics_registry: None,
        }
    }
    
    /// Set configuration
    pub fn with_config(mut self, config: StorageSystemConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Register a provider factory
    pub fn register_provider_factory(
        mut self,
        name: impl Into<String>,
        factory: Box<dyn ProviderFactory>,
    ) -> Self {
        self.provider_factories.insert(name.into(), factory);
        self
    }
    
    /// Set encryption key provider
    pub fn with_key_provider(
        mut self,
        provider: Box<dyn EncryptionKeyProvider>,
    ) -> Self {
        self.key_provider = Some(provider);
        self
    }
    
    /// Set metrics registry
    pub fn with_metrics_registry(
        mut self,
        registry: Arc<MetricsRegistry>,
    ) -> Self {
        self.metrics_registry = Some(registry);
        self
    }
    
    /// Build the storage system
    pub fn build(self) -> Result<StorageSystem, StorageError> {
        // Create components
        let encryption_manager = self.build_encryption_manager()?;
        let metrics_collector = self.build_metrics_collector()?;
        let transaction_manager = Arc::new(StorageTransactionManager::new(
            self.config.transaction_config.clone(),
        ));
        let policy_manager = Arc::new(StoragePolicyManager::new());
        
        // Create providers
        let mut providers = HashMap::new();
        
        for (storage_type, provider_config) in &self.config.providers {
            let factory = self.provider_factories.get(&provider_config.provider_type)
                .ok_or_else(|| StorageError::Configuration(
                    format!("No factory found for provider type: {}", provider_config.provider_type)
                ))?;
            
            let provider = factory.create_provider(
                *storage_type,
                provider_config,
                &encryption_manager,
                &transaction_manager,
                &metrics_collector,
            )?;
            
            providers.insert(*storage_type, provider);
        }
        
        // Create storage system
        Ok(StorageSystem {
            providers,
            policy_manager,
            transaction_manager,
            encryption_manager,
            metrics_collector,
        })
    }
}
```

## Usage Examples

### Basic Storage Operations

```rust
// Get a reference to the storage system
let storage = system.storage_system();

// Get a provider for resource storage
let resource_storage = storage.get_provider(StorageType::Resource)?;

// Store a resource
let resource_key = StorageKey::new(format!("resources/{}", resource_id));
let resource_data = serde_json::to_vec(&resource)?;

let options = StorageOptions::new()
    .with_encryption(true)
    .with_compression(true)
    .with_ttl(None);

let metadata = resource_storage.store(&resource_key, &resource_data, &options)?;

println!("Stored resource with metadata: {:?}", metadata);

// Retrieve the resource
let retrieve_options = RetrieveOptions::new();
let item = resource_storage.retrieve(&resource_key, &retrieve_options)?;

let retrieved_resource: Resource = serde_json::from_slice(&item.value)?;
println!("Retrieved resource: {:?}", retrieved_resource);
```

### Transactional Storage

```rust
// Get providers
let resource_storage = storage.get_provider(StorageType::Resource)?;
let fact_storage = storage.get_provider(StorageType::Fact)?;

// Begin transaction
let tx = storage.begin_transaction()?;

// Store multiple items atomically
let resource_key = StorageKey::new(format!("resources/{}", resource_id));
let resource_data = serde_json::to_vec(&resource)?;

let fact_key = StorageKey::new(format!("facts/{}", fact_id));
let fact_data = serde_json::to_vec(&fact)?;

let options = StorageOptions::new()
    .with_transaction(&tx)
    .with_encryption(true);

// Store items within transaction
resource_storage.store(&resource_key, &resource_data, &options)?;
fact_storage.store(&fact_key, &fact_data, &options)?;

// Commit transaction
storage.commit_transaction(tx.id)?;
```

### Document Query

```rust
// Get document storage
let doc_storage = storage.get_document_storage(StorageType::Resource)?;

// Query documents
let query = DocumentQuery::new()
    .with_field_equals("type", "document")
    .with_field_greater_than("created_at", Timestamp::from_iso8601("2023-01-01T00:00:00Z")?)
    .with_limit(100);

let options = QueryOptions::new()
    .with_sort_field("created_at", SortOrder::Descending);

let documents: Vec<Document> = doc_storage.query_documents("resources", &query, &options)?;

println!("Found {} documents", documents.len());
```

## Implementation Status

The current implementation status of the Storage Architecture:

- ✅ Core storage interfaces
- ✅ Storage provider abstraction 
- ✅ Transaction support
- ⚠️ Storage security (partially implemented)
- ⚠️ Key-Value provider (partially implemented)
- ⚠️ Document store provider (partially implemented)
- ❌ Blob storage provider (not yet implemented)
- ❌ Query optimization (not yet implemented)

## Future Enhancements

Planned future enhancements for the Storage Architecture:

1. **Distributed Storage**: Support for distributed storage across multiple nodes
2. **Storage Replication**: Automatic replication for high availability
3. **Time-Based Storage**: Enhanced support for temporal data storage
4. **Advanced Querying**: Improved query capabilities for document stores
5. **Storage Versioning**: Version control for stored items
6. **Storage Policies**: Policy-based storage management
7. **Dynamic Scaling**: Automatic scaling of storage resources
8. **Enhanced Encryption**: Additional encryption algorithms and key rotation
9. **Storage Analytics**: Advanced analytics on storage usage patterns 