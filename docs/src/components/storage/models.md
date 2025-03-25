<!-- Models for storage -->
<!-- Original file: docs/src/storage_models.md -->

# Storage Models in Causality

## Overview

This document describes the storage models used within the Causality architecture. The storage models define the structures, formats, schemas, and organization of data stored throughout the system. These models are designed to support the temporal nature of the Causality system while providing efficient access patterns and ensuring data integrity.

## Core Storage Models

### StorageItem Model

The fundamental unit of storage:

```rust
pub struct StorageItem {
    /// Storage key
    key: StorageKey,
    
    /// Item data
    value: Vec<u8>,
    
    /// Metadata about this item
    metadata: StorageMetadata,
    
    /// Whether this item was retrieved from cache
    from_cache: bool,
}

pub struct StorageMetadata {
    /// Storage key
    key: StorageKey,
    
    /// Size in bytes
    size: usize,
    
    /// When the item was stored
    stored_at: Timestamp,
    
    /// When the item expires (if applicable)
    expires_at: Option<Timestamp>,
    
    /// Data checksum
    checksum: Checksum,
}
```

### Storage Key Model

Keys for addressing storage items:

```rust
pub struct StorageKey {
    /// Key value
    value: String,
    
    /// Key namespace
    namespace: Option<String>,
    
    /// Key version
    version: Option<Version>,
}

impl StorageKey {
    /// Create a new storage key
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            namespace: None,
            version: None,
        }
    }
    
    /// With namespace
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }
    
    /// With version
    pub fn with_version(mut self, version: Version) -> Self {
        self.version = Some(version);
        self
    }
    
    /// Get formatted key string
    pub fn formatted_key(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(ns) = &self.namespace {
            parts.push(ns.clone());
        }
        
        parts.push(self.value.clone());
        
        if let Some(ver) = &self.version {
            parts.push(format!("v{}", ver));
        }
        
        parts.join("/")
    }
}
```

## Domain-Specific Storage Models

### Resource Storage Model

Model for storing resources:

```rust
pub struct ResourceStorageModel {
    /// Resource ID
    id: ResourceId,
    
    /// Resource type
    resource_type: ResourceType,
    
    /// Resource data
    data: Vec<u8>,
    
    /// Resource attributes
    attributes: HashMap<String, Vec<u8>>,
    
    /// Resource state
    state: ResourceState,
    
    /// Resource version
    version: Version,
    
    /// Resource metadata
    metadata: ResourceMetadata,
    
    /// Creation timestamp
    created_at: Timestamp,
    
    /// Last updated timestamp
    updated_at: Timestamp,
}

impl ResourceStorageModel {
    /// Convert to storage item
    pub fn to_storage_item(&self) -> Result<StorageItem, StorageError> {
        // Serialize the resource
        let data = self.serialize()?;
        
        // Create storage key
        let key = StorageKey::new(format!("resources/{}", self.id))
            .with_namespace("causality")
            .with_version(self.version);
        
        // Create metadata
        let metadata = StorageMetadata {
            key: key.clone(),
            size: data.len(),
            stored_at: system.current_time(),
            expires_at: None,
            checksum: calculate_checksum(&data),
        };
        
        Ok(StorageItem {
            key,
            value: data,
            metadata,
            from_cache: false,
        })
    }
    
    /// Create from storage item
    pub fn from_storage_item(item: StorageItem) -> Result<Self, StorageError> {
        // Deserialize the resource
        Self::deserialize(&item.value)
    }
    
    /// Serialize to bytes
    fn serialize(&self) -> Result<Vec<u8>, StorageError> {
        bincode::serialize(self).map_err(|e| StorageError::Serialization(e.to_string()))
    }
    
    /// Deserialize from bytes
    fn deserialize(data: &[u8]) -> Result<Self, StorageError> {
        bincode::deserialize(data).map_err(|e| StorageError::Deserialization(e.to_string()))
    }
}
```

### Fact Storage Model

Model for storing temporal facts:

```rust
pub struct FactStorageModel {
    /// Fact ID
    id: FactId,
    
    /// Fact type
    fact_type: FactType,
    
    /// Fact data
    data: Vec<u8>,
    
    /// Fact timestamp
    timestamp: Timestamp,
    
    /// Source domain
    source_domain: DomainId,
    
    /// Fact provenance
    provenance: FactProvenance,
    
    /// Fact status
    status: FactStatus,
    
    /// Related resource ID (if applicable)
    resource_id: Option<ResourceId>,
    
    /// Related operation ID (if applicable)
    operation_id: Option<OperationId>,
    
    /// Creation timestamp
    created_at: Timestamp,
}

impl FactStorageModel {
    /// Convert to storage item
    pub fn to_storage_item(&self) -> Result<StorageItem, StorageError> {
        // Serialize the fact
        let data = self.serialize()?;
        
        // Create storage key
        let key = StorageKey::new(format!("facts/{}", self.id))
            .with_namespace("causality");
        
        // Create metadata
        let metadata = StorageMetadata {
            key: key.clone(),
            size: data.len(),
            stored_at: system.current_time(),
            expires_at: None,
            checksum: calculate_checksum(&data),
        };
        
        Ok(StorageItem {
            key,
            value: data,
            metadata,
            from_cache: false,
        })
    }
    
    /// Create from storage item
    pub fn from_storage_item(item: StorageItem) -> Result<Self, StorageError> {
        // Deserialize the fact
        Self::deserialize(&item.value)
    }
    
    /// Create secondary indices for fact
    pub fn create_indices(&self) -> Vec<(StorageKey, Vec<u8>)> {
        let mut indices = Vec::new();
        
        // Time-based index
        let time_key = StorageKey::new(format!("fact_time/{}/{}", 
            self.timestamp.to_rfc3339(),
            self.id,
        )).with_namespace("causality_index");
        
        indices.push((time_key, Vec::new())); // Empty value, just for indexing
        
        // Type-based index
        let type_key = StorageKey::new(format!("fact_type/{}/{}/{}", 
            self.fact_type,
            self.timestamp.to_rfc3339(),
            self.id,
        )).with_namespace("causality_index");
        
        indices.push((type_key, Vec::new()));
        
        // Resource-based index (if applicable)
        if let Some(resource_id) = &self.resource_id {
            let resource_key = StorageKey::new(format!("fact_resource/{}/{}/{}", 
                resource_id,
                self.timestamp.to_rfc3339(),
                self.id,
            )).with_namespace("causality_index");
            
            indices.push((resource_key, Vec::new()));
        }
        
        indices
    }
    
    // Serialization methods similar to ResourceStorageModel
}
```

### Operation Storage Model

Model for storing operations:

```rust
pub struct OperationStorageModel {
    /// Operation ID
    id: OperationId,
    
    /// Operation type
    operation_type: OperationType,
    
    /// Target resource ID
    resource_id: ResourceId,
    
    /// Operation parameters (serialized)
    parameters: Vec<u8>,
    
    /// Operation status
    status: OperationStatus,
    
    /// Authentication information (serialized)
    auth_info: Vec<u8>,
    
    /// Operation metadata (serialized)
    metadata: Vec<u8>,
    
    /// Operation result (if completed)
    result: Option<Vec<u8>>,
    
    /// Creation timestamp
    created_at: Timestamp,
    
    /// Completion timestamp (if completed)
    completed_at: Option<Timestamp>,
}

// Implementation similar to ResourceStorageModel
```

### Transaction Storage Model

Model for storing transactions:

```rust
pub struct TransactionStorageModel {
    /// Transaction ID
    id: TransactionId,
    
    /// Transaction operations (serialized)
    operations: Vec<u8>,
    
    /// Transaction status
    status: TransactionStatus,
    
    /// Authentication information (serialized)
    auth_info: Vec<u8>,
    
    /// Transaction metadata (serialized)
    metadata: Vec<u8>,
    
    /// Transaction result (if completed)
    result: Option<Vec<u8>>,
    
    /// Creation timestamp
    created_at: Timestamp,
    
    /// Completion timestamp (if completed)
    completed_at: Option<Timestamp>,
    
    /// Expiration timestamp (if applicable)
    expires_at: Option<Timestamp>,
}

// Implementation similar to ResourceStorageModel
```

## Schema Management

### Schema Registry

Registry for storage schemas:

```rust
pub struct SchemaRegistry {
    /// Registered schemas
    schemas: RwLock<HashMap<SchemaId, Schema>>,
    
    /// Schema validators
    validators: HashMap<SchemaType, Box<dyn SchemaValidator>>,
}

impl SchemaRegistry {
    /// Register a schema
    pub fn register_schema(
        &self,
        schema: Schema,
    ) -> Result<SchemaId, RegistryError> {
        let schema_id = schema.id;
        
        let mut schemas = self.schemas.write().unwrap();
        
        // Check if already exists
        if schemas.contains_key(&schema_id) {
            return Err(RegistryError::AlreadyExists(
                format!("Schema with ID {} already exists", schema_id)
            ));
        }
        
        // Register schema
        schemas.insert(schema_id, schema);
        
        Ok(schema_id)
    }
    
    /// Get a schema by ID
    pub fn get_schema(
        &self,
        schema_id: SchemaId,
    ) -> Result<Schema, RegistryError> {
        let schemas = self.schemas.read().unwrap();
        
        schemas.get(&schema_id)
            .cloned()
            .ok_or_else(|| RegistryError::NotFound(
                format!("Schema with ID {} not found", schema_id)
            ))
    }
    
    /// Validate data against a schema
    pub fn validate_against_schema(
        &self,
        schema_id: SchemaId,
        data: &[u8],
    ) -> Result<ValidationResult, ValidationError> {
        // Get the schema
        let schema = self.get_schema(schema_id)?;
        
        // Get the validator
        let validator = self.validators.get(&schema.schema_type)
            .ok_or_else(|| ValidationError::UnsupportedSchemaType(schema.schema_type))?;
        
        // Validate data
        validator.validate(&schema, data)
    }
}
```

## Data Organization

### Collection and Partition Model

Organizing data in collections and partitions:

```rust
pub struct CollectionConfig {
    /// Collection name
    name: String,
    
    /// Collection schema
    schema: Option<SchemaId>,
    
    /// Partitioning strategy
    partitioning: PartitioningStrategy,
    
    /// Indexing configuration
    indexing: IndexingConfig,
    
    /// Retention policy
    retention: RetentionPolicy,
}

pub enum PartitioningStrategy {
    /// No partitioning
    None,
    
    /// Time-based partitioning
    TimeBasedPartitioning {
        /// Time field
        time_field: String,
        
        /// Partition interval
        interval: PartitionInterval,
    },
    
    /// Hash-based partitioning
    HashBasedPartitioning {
        /// Field to hash
        hash_field: String,
        
        /// Number of partitions
        partition_count: u32,
    },
    
    /// Range-based partitioning
    RangeBasedPartitioning {
        /// Range field
        range_field: String,
        
        /// Partition ranges
        ranges: Vec<RangeDefinition>,
    },
}

pub struct DocumentStore {
    /// Store configuration
    config: DocumentStoreConfig,
    
    /// Collection configurations
    collections: HashMap<String, CollectionConfig>,
    
    /// Storage provider
    storage_provider: Arc<dyn StorageProvider>,
    
    /// Schema registry
    schema_registry: Arc<SchemaRegistry>,
}

impl DocumentStore {
    /// Create a collection
    pub fn create_collection(
        &self,
        config: CollectionConfig,
    ) -> Result<(), StorageError> {
        // Validate collection config
        self.validate_collection_config(&config)?;
        
        // Store collection metadata
        let metadata_key = StorageKey::new(
            format!("_collections/{}", config.name)
        ).with_namespace("causality_metadata");
        
        let metadata = serde_json::to_vec(&config)?;
        
        self.storage_provider.store(
            &metadata_key,
            &metadata,
            &StorageOptions::new().with_encryption(true),
        )?;
        
        // Update in-memory collections
        self.collections.insert(config.name.clone(), config);
        
        Ok(())
    }
    
    /// Store a document in a collection
    pub fn store_document<T: Serialize>(
        &self,
        collection: &str,
        id: &str,
        document: &T,
        options: &DocumentStoreOptions,
    ) -> Result<DocumentMetadata, StorageError> {
        // Get collection config
        let config = self.get_collection_config(collection)?;
        
        // Serialize document
        let doc_data = serde_json::to_vec(document)?;
        
        // Validate against schema if configured
        if let Some(schema_id) = &config.schema {
            let result = self.schema_registry.validate_against_schema(
                *schema_id,
                &doc_data,
            )?;
            
            if !result.is_valid() {
                return Err(StorageError::SchemaValidation(
                    format!("Document failed schema validation: {}", 
                        result.error_messages().join(", ")
                    )
                ));
            }
        }
        
        // Determine partition
        let partition = self.determine_partition(collection, &doc_data, &config)?;
        
        // Create storage key
        let key = if let Some(partition) = partition {
            StorageKey::new(format!("{}/{}/{}", collection, partition, id))
        } else {
            StorageKey::new(format!("{}/{}", collection, id))
        };
        
        // Store document
        let metadata = self.storage_provider.store(
            &key,
            &doc_data,
            &options.into(),
        )?;
        
        // Create indices
        self.create_indices(collection, id, &doc_data, &config)?;
        
        // Return document metadata
        Ok(DocumentMetadata {
            collection: collection.to_string(),
            id: id.to_string(),
            key,
            size: metadata.size,
            stored_at: metadata.stored_at,
            expires_at: metadata.expires_at,
            checksum: metadata.checksum,
            schema_id: config.schema,
        })
    }
    
    // Other methods for retrieving, querying, etc.
}
```

## Data Versioning

### Version Management

Managing versions of stored data:

```rust
pub struct VersionedStorageProvider {
    /// Base storage provider
    storage_provider: Arc<dyn StorageProvider>,
    
    /// Version history store
    version_store: Arc<dyn StorageProvider>,
}

impl StorageProvider for VersionedStorageProvider {
    // Implement StorageProvider trait
    
    /// Store a versioned item
    fn store(&self, key: &StorageKey, value: &[u8], options: &StorageOptions) 
        -> Result<StorageMetadata, StorageError> {
        // Check if versioning is enabled for this key
        if !self.is_versioning_enabled(key) {
            // If not, delegate to base provider
            return self.storage_provider.store(key, value, options);
        }
        
        // Check if item already exists
        let current = self.storage_provider.retrieve(key, &RetrieveOptions::new());
        
        // Generate version
        let version = match &current {
            Ok(item) => {
                // Extract version from metadata or key
                let current_version = self.extract_version(&item.key)
                    .unwrap_or_else(|| Version::new(1, 0, 0));
                
                // Increment version
                current_version.increment_patch()
            }
            Err(_) => {
                // New item, start at version 1.0.0
                Version::new(1, 0, 0)
            }
        };
        
        // Create versioned key
        let versioned_key = key.clone().with_version(version);
        
        // Store current version
        let metadata = self.storage_provider.store(&versioned_key, value, options)?;
        
        // If we had a previous version, store it in history
        if let Ok(current_item) = current {
            // Create history key
            let history_key = StorageKey::new(format!("history/{}", key.formatted_key()))
                .with_version(self.extract_version(&current_item.key)
                    .unwrap_or_else(|| Version::new(1, 0, 0)));
            
            // Store in history
            self.version_store.store(
                &history_key,
                &current_item.value,
                &options,
            )?;
        }
        
        // Update version index
        self.update_version_index(key, version)?;
        
        Ok(metadata)
    }
    
    /// Get version history
    pub fn get_version_history(
        &self,
        key: &StorageKey,
        limit: Option<usize>,
    ) -> Result<Vec<VersionInfo>, StorageError> {
        // Get version index
        let index_key = StorageKey::new(format!("version_index/{}", key.formatted_key()));
        
        let index_item = self.version_store.retrieve(
            &index_key,
            &RetrieveOptions::new(),
        )?;
        
        // Deserialize version index
        let versions: Vec<VersionInfo> = serde_json::from_slice(&index_item.value)?;
        
        // Apply limit if specified
        let limited_versions = match limit {
            Some(limit) => versions.into_iter().take(limit).collect(),
            None => versions,
        };
        
        Ok(limited_versions)
    }
    
    /// Retrieve a specific version
    pub fn retrieve_version(
        &self,
        key: &StorageKey,
        version: Version,
    ) -> Result<StorageItem, StorageError> {
        // If it's the current version, get from main storage
        if self.is_current_version(key, &version)? {
            return self.storage_provider.retrieve(
                &key.clone().with_version(version),
                &RetrieveOptions::new(),
            );
        }
        
        // Otherwise, get from history
        let history_key = StorageKey::new(format!("history/{}", key.formatted_key()))
            .with_version(version);
        
        self.version_store.retrieve(
            &history_key,
            &RetrieveOptions::new(),
        )
    }
}
```

## Blob Storage

### Large Object Storage

Managing large binary objects:

```rust
pub struct BlobStorageProvider {
    /// Base storage provider
    storage_provider: Arc<dyn StorageProvider>,
    
    /// Chunk size for splitting large objects
    chunk_size: usize,
    
    /// Blob metadata store
    metadata_store: Arc<dyn StorageProvider>,
}

impl StorageProvider for BlobStorageProvider {
    // Implement StorageProvider trait
    
    /// Store a blob
    fn store(&self, key: &StorageKey, value: &[u8], options: &StorageOptions) 
        -> Result<StorageMetadata, StorageError> {
        // If smaller than chunk size, store directly
        if value.len() <= self.chunk_size {
            return self.storage_provider.store(key, value, options);
        }
        
        // Split into chunks
        let chunks = self.split_into_chunks(value);
        
        // Store each chunk
        let mut chunk_metadata = Vec::with_capacity(chunks.len());
        
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_key = StorageKey::new(format!("{}/chunks/{}", key.formatted_key(), i));
            
            let metadata = self.storage_provider.store(
                &chunk_key,
                chunk,
                options,
            )?;
            
            chunk_metadata.push(metadata);
        }
        
        // Create blob metadata
        let blob_metadata = BlobMetadata {
            key: key.clone(),
            total_size: value.len(),
            chunk_count: chunks.len(),
            chunk_size: self.chunk_size,
            chunks: chunk_metadata,
            checksum: calculate_checksum(value),
            stored_at: system.current_time(),
            expires_at: options.ttl.map(|ttl| system.current_time() + ttl),
        };
        
        // Store blob metadata
        let metadata_key = StorageKey::new(format!("{}/metadata", key.formatted_key()));
        
        let serialized_metadata = serde_json::to_vec(&blob_metadata)?;
        
        self.metadata_store.store(
            &metadata_key,
            &serialized_metadata,
            options,
        )?;
        
        // Return metadata
        Ok(StorageMetadata {
            key: key.clone(),
            size: value.len(),
            stored_at: system.current_time(),
            expires_at: options.ttl.map(|ttl| system.current_time() + ttl),
            checksum: blob_metadata.checksum,
        })
    }
    
    /// Retrieve a blob
    fn retrieve(&self, key: &StorageKey, options: &RetrieveOptions) 
        -> Result<StorageItem, StorageError> {
        // Try to retrieve directly (may be small blob)
        let direct_result = self.storage_provider.retrieve(key, options);
        
        if direct_result.is_ok() {
            return direct_result;
        }
        
        // Retrieve blob metadata
        let metadata_key = StorageKey::new(format!("{}/metadata", key.formatted_key()));
        
        let metadata_item = self.metadata_store.retrieve(
            &metadata_key,
            options,
        )?;
        
        // Deserialize metadata
        let blob_metadata: BlobMetadata = serde_json::from_slice(&metadata_item.value)?;
        
        // Allocate buffer for full blob
        let mut blob_data = Vec::with_capacity(blob_metadata.total_size);
        
        // Retrieve each chunk
        for i in 0..blob_metadata.chunk_count {
            let chunk_key = StorageKey::new(format!("{}/chunks/{}", key.formatted_key(), i));
            
            let chunk_item = self.storage_provider.retrieve(
                &chunk_key,
                options,
            )?;
            
            // Append chunk data
            blob_data.extend_from_slice(&chunk_item.value);
        }
        
        // Verify checksum
        let calculated_checksum = calculate_checksum(&blob_data);
        
        if calculated_checksum != blob_metadata.checksum {
            return Err(StorageError::ChecksumMismatch(
                format!("Blob checksum mismatch for key {}", key)
            ));
        }
        
        // Return assembled blob
        Ok(StorageItem {
            key: key.clone(),
            value: blob_data,
            metadata: StorageMetadata {
                key: key.clone(),
                size: blob_metadata.total_size,
                stored_at: blob_metadata.stored_at,
                expires_at: blob_metadata.expires_at,
                checksum: blob_metadata.checksum,
            },
            from_cache: false,
        })
    }
}
```

## Usage Examples

### Basic Document Storage

```rust
// Get document storage
let doc_storage = system.get_document_storage(StorageType::Resource)?;

// Define document
#[derive(Serialize, Deserialize)]
struct Document {
    title: String,
    content: String,
    tags: Vec<String>,
    created_at: Timestamp,
}

// Create document
let document = Document {
    title: "Storage System Overview".to_string(),
    content: "This document provides an overview...".to_string(),
    tags: vec!["storage".to_string(), "architecture".to_string()],
    created_at: system.current_time(),
};

// Store document
let document_id = "doc-12345";
let metadata = doc_storage.store_document(
    "documents",
    document_id,
    &document,
    &DocumentOptions::new()
        .with_encryption(true)
        .with_compression(true),
)?;

println!("Stored document with metadata: {:?}", metadata);

// Retrieve document
let retrieved: Document = doc_storage.retrieve_document(
    "documents",
    document_id,
    &RetrieveOptions::new(),
)?;

println!("Retrieved document: {}", retrieved.title);
```

### Versioned Storage

```rust
// Get versioned storage
let versioned_storage = system.get_versioned_storage(StorageType::Resource)?;

// Store initial version
let resource_key = StorageKey::new(format!("resources/{}", resource_id));
let resource_data_v1 = serde_json::to_vec(&resource_v1)?;

versioned_storage.store(
    &resource_key,
    &resource_data_v1,
    &StorageOptions::new().with_versioning(true),
)?;

// Update resource
let resource_data_v2 = serde_json::to_vec(&resource_v2)?;

versioned_storage.store(
    &resource_key,
    &resource_data_v2,
    &StorageOptions::new().with_versioning(true),
)?;

// Get version history
let history = versioned_storage.get_version_history(&resource_key, Some(10))?;

println!("Version history:");
for version in &history {
    println!("  Version: {}, Created: {}", version.version, version.created_at);
}

// Retrieve specific version
let v1 = history.last().unwrap().version;
let v1_resource = versioned_storage.retrieve_version(&resource_key, v1)?;

let resource_v1_restored: Resource = serde_json::from_slice(&v1_resource.value)?;
println!("Restored version {}: {}", v1, resource_v1_restored.name);
```

### Blob Storage

```rust
// Get blob storage
let blob_storage = system.get_blob_storage()?;

// Store a large file
let file_data = std::fs::read("large_binary_file.dat")?;
let blob_key = StorageKey::new("binaries/large_binary_file.dat");

blob_storage.store(
    &blob_key,
    &file_data,
    &StorageOptions::new()
        .with_encryption(true)
        .with_metadata(HashMap::from([
            ("content-type".to_string(), "application/octet-stream".to_string()),
            ("description".to_string(), "Large binary data file".to_string()),
        ])),
)?;

// Retrieve the file
let retrieved_blob = blob_storage.retrieve(
    &blob_key,
    &RetrieveOptions::new(),
)?;

assert_eq!(file_data, retrieved_blob.value);
```

## Implementation Status

The current implementation status of Storage Models:

- ✅ Core storage models
- ✅ Resource storage model
- ✅ Fact storage model
- ⚠️ Operation storage model (partially implemented)
- ⚠️ Transaction storage model (partially implemented)
- ⚠️ Schema registry (partially implemented)
- ❌ Versioned storage (not yet implemented)
- ❌ Blob storage (not yet implemented)

## Future Enhancements

Planned future enhancements for Storage Models:

1. **Time-Based Storage**: Enhanced storage models optimized for temporal data
2. **Hierarchical Storage Models**: Support for nested/hierarchical data structures
3. **Polymorphic Storage Models**: Improved handling of polymorphic data types
4. **Storage Model Evolution**: Mechanisms for data migration during schema changes
5. **Compression Strategies**: Optimized compression based on data characteristics
6. **Distributed Storage Models**: Enhanced models for distributed data storage
7. **Cryptographic Storage**: Advanced encryption and secure storage models
8. **Analytical Storage Models**: Models optimized for analytical workloads 