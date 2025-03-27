# Implementing the Resource System

*This guide is derived from the [System Specification](../../../spec/spec.md) and the [Resource System](../../architecture/core/resource-system.md) architecture document.*

*Last updated: 2023-08-25*

## Introduction

This guide provides practical steps and code examples for implementing and working with the Resource System in Causality. You'll learn how to:

1. Create and manage resources using the unified ResourceRegister model
2. Implement resource accessors for different resource types
3. Manage resource lifecycle states
4. Work with cross-domain resources
5. Track relationships between resources
6. Integrate resources with the capability and effect systems

## Prerequisites

Before implementing resources, ensure you understand:
- Content addressing fundamentals
- Basic effect system principles
- Capability-based security model

## Resource Creation and Management

### 1. Defining a Resource Type

Let's start by defining a custom resource type:

```rust
use causality_core::resource::{Resource, ResourceRegister, ResourceId};
use causality_core::content::{ContentAddressed, ContentHash};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResource {
    // Core resource fields
    id: ResourceId,
    owner: Address,
    
    // Document-specific fields
    title: String,
    content: String,
    version: u32,
    metadata: HashMap<String, String>,
    
    // Content hash
    content_hash: ContentHash,
}

impl DocumentResource {
    pub fn new(
        id: ResourceId,
        owner: Address,
        title: String,
        content: String,
    ) -> Self {
        let mut doc = Self {
            id,
            owner,
            title,
            content,
            version: 1,
            metadata: HashMap::new(),
            content_hash: ContentHash::default(),
        };
        
        // Calculate content hash
        doc.content_hash = doc.calculate_content_hash().expect("Failed to calculate hash");
        
        doc
    }
    
    pub fn update_content(&mut self, new_content: String) -> Result<(), ResourceError> {
        self.content = new_content;
        self.version += 1;
        
        // Recalculate content hash
        self.content_hash = self.calculate_content_hash()?;
        
        Ok(())
    }
    
    pub fn add_metadata(&mut self, key: &str, value: &str) -> Result<(), ResourceError> {
        self.metadata.insert(key.to_string(), value.to_string());
        
        // Recalculate content hash
        self.content_hash = self.calculate_content_hash()?;
        
        Ok(())
    }
}

impl Resource for DocumentResource {
    fn resource_id(&self) -> &ResourceId {
        &self.id
    }
    
    fn owner(&self) -> &Address {
        &self.owner
    }
    
    fn resource_type() -> &'static str {
        "document"
    }
}

impl ContentAddressed for DocumentResource {
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
        let mut hasher = ContentHasher::new();
        
        // Hash type information
        hasher.update("DocumentResource");
        
        // Hash core fields
        hasher.update(self.id.as_bytes());
        hasher.update(self.owner.as_bytes());
        
        // Hash document-specific fields
        hasher.update(self.title.as_bytes());
        hasher.update(self.content.as_bytes());
        hasher.update(&self.version.to_le_bytes());
        
        // Hash metadata
        let mut sorted_keys: Vec<&String> = self.metadata.keys().collect();
        sorted_keys.sort();
        
        for key in sorted_keys {
            if let Some(value) = self.metadata.get(key) {
                hasher.update(key.as_bytes());
                hasher.update(value.as_bytes());
            }
        }
        
        Ok(hasher.finalize())
    }
    
    fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
    
    fn with_content_hash(mut self, hash: ContentHash) -> Self {
        self.content_hash = hash;
        self
    }
}
```

### 2. Creating a Resource Manager

Now, let's create a ResourceManager to handle our resources:

```rust
use causality_core::resource::{ResourceManager, ResourceManagerConfig};
use causality_core::effect::EffectSystem;
use causality_core::capability::CapabilityManager;

async fn setup_resource_manager(
    effect_system: &EffectSystem,
    capability_manager: &CapabilityManager,
) -> Result<ResourceManager, ResourceError> {
    // Create resource manager configuration
    let config = ResourceManagerConfig::default()
        .with_effect_system(effect_system.clone())
        .with_capability_manager(capability_manager.clone())
        .with_storage_strategy(StorageStrategy::Hybrid {
            on_chain_fields: HashSet::from([
                "owner".to_string(),
                "title".to_string(),
                "version".to_string(),
            ]),
            remaining_commitment: None,
        });
    
    // Create resource manager
    let resource_manager = ResourceManager::new(config)?;
    
    // Register resource types
    resource_manager.register_resource_type::<DocumentResource>()?;
    
    Ok(resource_manager)
}
```

### 3. Implementing a Resource Accessor

The Resource Accessor Pattern provides type-safe access to resources:

```rust
use async_trait::async_trait;
use causality_core::resource::{ResourceAccessor, ResourceId, ResourceQuery, ResourceError};

// Define a custom accessor for document resources
#[async_trait]
pub trait DocumentAccessor: ResourceAccessor<Resource = DocumentResource> {
    // Document-specific operations
    async fn search_by_title(&self, query: &str) -> Result<Vec<DocumentResource>, ResourceError>;
    async fn get_latest_version(&self, id: &ResourceId) -> Result<Option<DocumentResource>, ResourceError>;
    async fn update_content(&self, id: &ResourceId, new_content: &str) -> Result<(), ResourceError>;
}

// Implement the document accessor
pub struct LocalDocumentAccessor {
    resources: RwLock<HashMap<ResourceId, DocumentResource>>,
}

impl LocalDocumentAccessor {
    pub fn new() -> Self {
        Self {
            resources: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ResourceAccessor for LocalDocumentAccessor {
    type Resource = DocumentResource;
    
    async fn get(&self, id: &ResourceId) -> Result<Option<Self::Resource>, ResourceError> {
        let resources = self.resources.read().map_err(|_| ResourceError::LockError)?;
        Ok(resources.get(id).cloned())
    }
    
    async fn query(&self, query: &ResourceQuery) -> Result<Vec<Self::Resource>, ResourceError> {
        let resources = self.resources.read().map_err(|_| ResourceError::LockError)?;
        
        let results: Vec<DocumentResource> = resources.values()
            .filter(|doc| query.matches(doc))
            .cloned()
            .collect();
        
        Ok(results)
    }
    
    async fn create(&self, resource: Self::Resource) -> Result<ResourceId, ResourceError> {
        let resource_id = resource.resource_id().clone();
        
        let mut resources = self.resources.write().map_err(|_| ResourceError::LockError)?;
        resources.insert(resource_id.clone(), resource);
        
        Ok(resource_id)
    }
    
    async fn update(&self, id: &ResourceId, resource: Self::Resource) -> Result<(), ResourceError> {
        let mut resources = self.resources.write().map_err(|_| ResourceError::LockError)?;
        
        if !resources.contains_key(id) {
            return Err(ResourceError::NotFound);
        }
        
        resources.insert(id.clone(), resource);
        
        Ok(())
    }
    
    async fn delete(&self, id: &ResourceId) -> Result<(), ResourceError> {
        let mut resources = self.resources.write().map_err(|_| ResourceError::LockError)?;
        
        if resources.remove(id).is_none() {
            return Err(ResourceError::NotFound);
        }
        
        Ok(())
    }
}

#[async_trait]
impl DocumentAccessor for LocalDocumentAccessor {
    async fn search_by_title(&self, query: &str) -> Result<Vec<DocumentResource>, ResourceError> {
        let resources = self.resources.read().map_err(|_| ResourceError::LockError)?;
        
        let results: Vec<DocumentResource> = resources.values()
            .filter(|doc| doc.title.contains(query))
            .cloned()
            .collect();
        
        Ok(results)
    }
    
    async fn get_latest_version(&self, id: &ResourceId) -> Result<Option<DocumentResource>, ResourceError> {
        let resources = self.resources.read().map_err(|_| ResourceError::LockError)?;
        
        // Find documents with the same base ID but different versions
        let base_id = id.base_id()?;
        let versions: Vec<DocumentResource> = resources.values()
            .filter(|doc| doc.resource_id().base_id().unwrap_or_default() == base_id)
            .cloned()
            .collect();
        
        // Return the one with the highest version
        Ok(versions.into_iter().max_by_key(|doc| doc.version))
    }
    
    async fn update_content(&self, id: &ResourceId, new_content: &str) -> Result<(), ResourceError> {
        let mut resources = self.resources.write().map_err(|_| ResourceError::LockError)?;
        
        if let Some(mut doc) = resources.get(id).cloned() {
            doc.update_content(new_content.to_string())?;
            resources.insert(id.clone(), doc);
            Ok(())
        } else {
            Err(ResourceError::NotFound)
        }
    }
}
```

### 4. Registering and Using a Resource Accessor

Now, let's register and use our custom accessor:

```rust
async fn register_document_accessor(
    resource_manager: &ResourceManager,
) -> Result<(), ResourceError> {
    // Create the document accessor
    let document_accessor = Arc::new(LocalDocumentAccessor::new());
    
    // Register the accessor with the resource manager
    resource_manager.register_accessor::<dyn DocumentAccessor>(document_accessor)?;
    
    Ok(())
}

async fn use_document_accessor(
    resource_manager: &ResourceManager,
    domain_id: &DomainId,
) -> Result<(), ResourceError> {
    // Get the document accessor for the domain
    let document_accessor = resource_manager.get_accessor::<dyn DocumentAccessor>(domain_id)?;
    
    // Create a new document
    let doc_id = ResourceId::new("document", domain_id.clone(), "my-first-doc")?;
    let document = DocumentResource::new(
        doc_id.clone(),
        Address::from_string("user123")?,
        "My First Document".to_string(),
        "This is the content of my first document.".to_string(),
    );
    
    // Create the document
    document_accessor.create(document).await?;
    
    // Update the document content
    document_accessor.update_content(
        &doc_id,
        "This is the updated content of my first document."
    ).await?;
    
    // Search for documents by title
    let search_results = document_accessor.search_by_title("First").await?;
    println!("Found {} documents", search_results.len());
    
    // Get the latest version of the document
    let latest = document_accessor.get_latest_version(&doc_id).await?;
    if let Some(doc) = latest {
        println!("Latest version: {}", doc.version);
    }
    
    Ok(())
}
```

## Resource Lifecycle Management

### 1. Creating a Resource Lifecycle Manager

The lifecycle manager enforces valid state transitions:

```rust
use causality_core::resource::lifecycle::{ResourceLifecycleManager, ResourceLifecycleEvent};

async fn setup_lifecycle_manager(
    resource_manager: &ResourceManager,
) -> Result<ResourceLifecycleManager, ResourceError> {
    // Define supported state transitions
    let transitions = vec![
        (RegisterState::Active, RegisterState::Locked, true),
        (RegisterState::Active, RegisterState::Frozen, true),
        (RegisterState::Active, RegisterState::PendingDeletion, true),
        (RegisterState::Locked, RegisterState::Active, true),
        (RegisterState::Frozen, RegisterState::Active, true),
        (RegisterState::PendingDeletion, RegisterState::Tombstone, true),
    ];
    
    // Create lifecycle manager configuration
    let config = ResourceLifecycleConfig::default()
        .with_allowed_transitions(transitions)
        .with_transition_validation(true)
        .with_event_logging(true);
    
    // Create lifecycle manager
    let lifecycle_manager = ResourceLifecycleManager::new(
        resource_manager.clone(),
        config,
    )?;
    
    Ok(lifecycle_manager)
}
```

### 2. Using the Lifecycle Manager

Let's use the lifecycle manager to handle state transitions:

```rust
async fn manage_document_lifecycle(
    lifecycle_manager: &ResourceLifecycleManager,
    document_id: &ResourceId,
) -> Result<(), ResourceError> {
    // Lock the document for editing
    lifecycle_manager.transition(
        document_id,
        RegisterState::Locked {
            operation_id: OperationId::new(),
            expiry: system_time() + Duration::from_secs(300),
        },
        "Locking for content update",
    ).await?;
    
    // Perform operations while document is locked
    // ...
    
    // Unlock the document
    lifecycle_manager.transition(
        document_id,
        RegisterState::Active,
        "Unlocking after content update",
    ).await?;
    
    // Later, mark for deletion
    lifecycle_manager.transition(
        document_id,
        RegisterState::PendingDeletion {
            scheduled_time: system_time() + Duration::from_secs(86400), // 1 day
        },
        "Document marked for deletion",
    ).await?;
    
    // Get lifecycle events for the document
    let events = lifecycle_manager.get_events(document_id).await?;
    for event in events {
        println!("Event: {:?} at {}", event.event_type, event.timestamp);
    }
    
    Ok(())
}
```

## Resource Relationships

### 1. Creating a Relationship Tracker

The relationship tracker manages dependencies between resources:

```rust
use causality_core::resource::relationship::{RelationshipTracker, RelationshipType};

async fn setup_relationship_tracker(
    resource_manager: &ResourceManager,
) -> Result<RelationshipTracker, ResourceError> {
    // Create relationship tracker configuration
    let config = RelationshipTrackerConfig::default()
        .with_validation(true)
        .with_bidirectional_tracking(true);
    
    // Create relationship tracker
    let relationship_tracker = RelationshipTracker::new(
        resource_manager.clone(),
        config,
    )?;
    
    // Register custom relationship types
    relationship_tracker.register_relationship_type(
        "DocumentVersion",
        RelationshipConstraints::new()
            .with_source_type("document")
            .with_target_type("document")
            .with_cardinality(RelationshipCardinality::OneToMany),
    )?;
    
    Ok(relationship_tracker)
}
```

### 2. Working with Relationships

Let's create and query relationships between resources:

```rust
async fn manage_document_relationships(
    relationship_tracker: &RelationshipTracker,
    source_id: &ResourceId,
    target_id: &ResourceId,
) -> Result<(), ResourceError> {
    // Create a dependency relationship
    let relationship_id = relationship_tracker.create_relationship(
        source_id,
        target_id,
        RelationshipType::Dependency,
        HashMap::new(),
    ).await?;
    
    // Check if a relationship exists
    let relationship_exists = relationship_tracker.has_relationship(
        source_id,
        target_id,
        RelationshipType::Dependency,
    ).await?;
    
    println!("Relationship exists: {}", relationship_exists);
    
    // Find all dependencies of a resource
    let dependencies = relationship_tracker.find_related_resources(
        source_id,
        RelationshipType::Dependency,
        RelationshipDirection::Outgoing,
    ).await?;
    
    println!("Found {} dependencies", dependencies.len());
    
    // Find all resources dependent on this resource
    let dependents = relationship_tracker.find_related_resources(
        source_id,
        RelationshipType::Dependency,
        RelationshipDirection::Incoming,
    ).await?;
    
    println!("Found {} dependents", dependents.len());
    
    // Delete a relationship
    relationship_tracker.delete_relationship(&relationship_id).await?;
    
    Ok(())
}
```

## Cross-Domain Resource Management

### 1. Creating a Cross-Domain Resource

Let's create a resource that spans multiple domains:

```rust
async fn create_cross_domain_resource(
    resource_manager: &ResourceManager,
    source_domain: &DomainId,
    target_domain: &DomainId,
) -> Result<(ResourceId, ResourceId), ResourceError> {
    // Create the source resource
    let source_resource = DocumentResource::new(
        ResourceId::new("document", source_domain.clone(), "cross-domain-doc")?,
        Address::from_string("user123")?,
        "Cross-Domain Document".to_string(),
        "This document exists across domains.".to_string(),
    );
    
    // Add cross-domain metadata
    let mut source_resource = source_resource.clone();
    source_resource.add_metadata("target_domain", target_domain.to_string().as_str())?;
    
    // Create the source resource
    let source_id = resource_manager.register(source_resource).await?;
    
    // Create the target resource (mirror)
    let target_resource = DocumentResource::new(
        ResourceId::new("document", target_domain.clone(), "cross-domain-doc")?,
        Address::from_string("user123")?,
        "Cross-Domain Document".to_string(),
        "This document exists across domains.".to_string(),
    );
    
    // Add cross-domain metadata
    let mut target_resource = target_resource.clone();
    target_resource.add_metadata("source_domain", source_domain.to_string().as_str())?;
    
    // Create the target resource
    let target_id = resource_manager.register(target_resource).await?;
    
    // Create a cross-domain relationship
    let relationship_tracker = resource_manager.get_relationship_tracker()?;
    relationship_tracker.create_relationship(
        &source_id,
        &target_id,
        RelationshipType::Mirror,
        HashMap::new(),
    ).await?;
    
    Ok((source_id, target_id))
}
```

### 2. Implementing a Cross-Domain Operation

Now, let's implement a cross-domain update operation:

```rust
async fn update_cross_domain_document(
    resource_manager: &ResourceManager,
    source_id: &ResourceId,
    target_id: &ResourceId,
    new_content: &str,
) -> Result<(), ResourceError> {
    // Get the relationship tracker
    let relationship_tracker = resource_manager.get_relationship_tracker()?;
    
    // Verify the cross-domain relationship
    let relationship_exists = relationship_tracker.has_relationship(
        source_id,
        target_id,
        RelationshipType::Mirror,
    ).await?;
    
    if !relationship_exists {
        return Err(ResourceError::RelationshipNotFound);
    }
    
    // Begin a cross-domain transaction
    let transaction = resource_manager.begin_transaction()?;
    
    // Update the source document
    let source_domain = source_id.domain();
    let source_accessor = resource_manager.get_accessor::<dyn DocumentAccessor>(source_domain)?;
    source_accessor.update_content(source_id, new_content).await?;
    
    // Update the target document
    let target_domain = target_id.domain();
    let target_accessor = resource_manager.get_accessor::<dyn DocumentAccessor>(target_domain)?;
    target_accessor.update_content(target_id, new_content).await?;
    
    // Commit the transaction
    transaction.commit().await?;
    
    Ok(())
}
```

## Integration with the Effect System

### 1. Defining Resource Storage Effects

Let's define storage effects for resources:

```rust
use causality_core::effect::{Effect, EffectExecutor, EffectContext, EffectResult};

// Define a storage effect for document resources
pub enum DocumentStorageEffect {
    // Store a document
    StoreDocument {
        document: DocumentResource,
        domain_id: DomainId,
    },
    
    // Read a document
    ReadDocument {
        document_id: ResourceId,
        domain_id: DomainId,
    },
}

impl Effect for DocumentStorageEffect {
    async fn execute(&self, context: &dyn EffectContext) -> Result<EffectResult, EffectError> {
        match self {
            DocumentStorageEffect::StoreDocument { document, domain_id } => {
                // Get the document accessor for the domain
                let document_accessor = context.resource_manager()
                    .get_accessor::<dyn DocumentAccessor>(domain_id)?;
                
                // Store the document
                let document_id = document_accessor.create(document.clone()).await?;
                
                Ok(EffectResult::Value(document_id))
            },
            DocumentStorageEffect::ReadDocument { document_id, domain_id } => {
                // Get the document accessor for the domain
                let document_accessor = context.resource_manager()
                    .get_accessor::<dyn DocumentAccessor>(domain_id)?;
                
                // Read the document
                let document = document_accessor.get(document_id).await?
                    .ok_or(EffectError::ResourceNotFound)?;
                
                Ok(EffectResult::Value(document))
            },
        }
    }
}
```

### 2. Using Storage Effects

Now, let's use the storage effects:

```rust
async fn use_storage_effects(
    effect_system: &EffectSystem,
    domain_id: &DomainId,
) -> Result<(), EffectError> {
    // Create a document
    let doc_id = ResourceId::new("document", domain_id.clone(), "effect-doc")?;
    let document = DocumentResource::new(
        doc_id.clone(),
        Address::from_string("user123")?,
        "Effect Document".to_string(),
        "This document is managed through effects.".to_string(),
    );
    
    // Store the document using an effect
    let store_effect = DocumentStorageEffect::StoreDocument {
        document: document.clone(),
        domain_id: domain_id.clone(),
    };
    
    let result = effect_system.execute_effect(store_effect).await?;
    println!("Document stored: {:?}", result);
    
    // Read the document using an effect
    let read_effect = DocumentStorageEffect::ReadDocument {
        document_id: doc_id.clone(),
        domain_id: domain_id.clone(),
    };
    
    let result = effect_system.execute_effect(read_effect).await?;
    if let EffectResult::Value(document) = result {
        println!("Document read: {:?}", document);
    }
    
    Ok(())
}
```

## Integration with the Capability System

### 1. Defining Resource Capabilities

Let's define capabilities for resource operations:

```rust
use causality_core::capability::{Capability, CapabilityType, CapabilityConstraint};

// Define resource-specific capability types
pub enum DocumentCapability {
    // Read a document
    Read(ResourceId),
    
    // Edit a document
    Edit(ResourceId),
    
    // Delete a document
    Delete(ResourceId),
    
    // Manage document relationships
    ManageRelationships(ResourceId),
}

impl Into<CapabilityType> for DocumentCapability {
    fn into(self) -> CapabilityType {
        match self {
            DocumentCapability::Read(id) => {
                CapabilityType::Resource(
                    ResourceCapability::Read,
                    id,
                )
            },
            DocumentCapability::Edit(id) => {
                CapabilityType::Resource(
                    ResourceCapability::Write,
                    id,
                )
            },
            DocumentCapability::Delete(id) => {
                CapabilityType::Resource(
                    ResourceCapability::Delete,
                    id,
                )
            },
            DocumentCapability::ManageRelationships(id) => {
                CapabilityType::Custom(
                    "document.manage_relationships".to_string(),
                    id.to_string(),
                )
            },
        }
    }
}
```

### 2. Creating and Verifying Capabilities

Now, let's create and verify capabilities:

```rust
async fn manage_document_capabilities(
    capability_manager: &CapabilityManager,
    document_id: &ResourceId,
    owner: &Address,
    delegate: &Address,
) -> Result<(), CapabilityError> {
    // Grant read capability to the delegate
    let read_capability = Capability::new(
        DocumentCapability::Read(document_id.clone()).into(),
        Vec::new(), // No constraints
    )?;
    
    capability_manager.grant(
        &owner.to_resource_id()?,
        &delegate.to_resource_id()?,
        read_capability.clone(),
    ).await?;
    
    // Grant edit capability with constraints
    let edit_constraints = vec![
        CapabilityConstraint::Time(
            TimeConstraint::ExpiresAt(
                system_time() + Duration::from_secs(86400), // 1 day
            ),
        ),
        CapabilityConstraint::Usage(
            UsageConstraint::MaxUsage(5), // Max 5 edits
        ),
    ];
    
    let edit_capability = Capability::new(
        DocumentCapability::Edit(document_id.clone()).into(),
        edit_constraints,
    )?;
    
    capability_manager.grant(
        &owner.to_resource_id()?,
        &delegate.to_resource_id()?,
        edit_capability.clone(),
    ).await?;
    
    // Verify read capability
    let has_read = capability_manager.verify(
        &delegate.to_resource_id()?,
        DocumentCapability::Read(document_id.clone()).into(),
    ).await?;
    
    println!("Has read capability: {}", has_read);
    
    // Verify edit capability
    let has_edit = capability_manager.verify(
        &delegate.to_resource_id()?,
        DocumentCapability::Edit(document_id.clone()).into(),
    ).await?;
    
    println!("Has edit capability: {}", has_edit);
    
    // Revoke edit capability
    capability_manager.revoke(
        &owner.to_resource_id()?,
        &delegate.to_resource_id()?,
        DocumentCapability::Edit(document_id.clone()).into(),
    ).await?;
    
    Ok(())
}
```

## Resource Storage Optimization with Deferred Hashing

As described in ADR-030, we can optimize resource hashing in ZK environments:

```rust
// Context for deferred hashing
struct DeferredHashingContext {
    deferred_hash_inputs: Vec<DeferredHashInput>,
    hash_outputs: HashMap<DeferredHashId, ContentHash>,
}

impl DeferredHashingContext {
    fn new() -> Self {
        Self {
            deferred_hash_inputs: Vec::new(),
            hash_outputs: HashMap::new(),
        }
    }
    
    fn request_hash(&mut self, data: &[u8]) -> DeferredHashId {
        let id = DeferredHashId::new();
        self.deferred_hash_inputs.push(DeferredHashInput {
            id: id.clone(),
            data: data.to_vec(),
        });
        id
    }
    
    fn compute_deferred_hashes(&mut self) {
        for input in &self.deferred_hash_inputs {
            let hash = poseidon_hash(&input.data);
            self.hash_outputs.insert(input.id.clone(), ContentHash(hash));
        }
    }
    
    fn get_hash(&self, id: &DeferredHashId) -> Option<&ContentHash> {
        self.hash_outputs.get(id)
    }
}

// Resource accessor that uses deferred hashing
struct DeferredHashingDocumentAccessor {
    inner: Arc<dyn DocumentAccessor>,
    hashing_context: Arc<RwLock<DeferredHashingContext>>,
}

impl DeferredHashingDocumentAccessor {
    fn new(inner: Arc<dyn DocumentAccessor>) -> Self {
        Self {
            inner,
            hashing_context: Arc::new(RwLock::new(DeferredHashingContext::new())),
        }
    }
    
    fn compute_hashes(&self) -> Result<(), ResourceError> {
        let mut context = self.hashing_context.write().map_err(|_| ResourceError::LockError)?;
        context.compute_deferred_hashes();
        Ok(())
    }
}

#[async_trait]
impl ResourceAccessor for DeferredHashingDocumentAccessor {
    type Resource = DocumentResource;
    
    async fn get(&self, id: &ResourceId) -> Result<Option<Self::Resource>, ResourceError> {
        self.inner.get(id).await
    }
    
    async fn create(&self, mut resource: Self::Resource) -> Result<ResourceId, ResourceError> {
        // Instead of computing the hash now, defer it
        let mut context = self.hashing_context.write().map_err(|_| ResourceError::LockError)?;
        
        // Serialize the resource
        let serialized = serde_json::to_vec(&resource)
            .map_err(|e| ResourceError::SerializationError(e.to_string()))?;
        
        // Request a deferred hash
        let hash_id = context.request_hash(&serialized);
        
        // For now, use a placeholder hash
        resource = resource.with_content_hash(ContentHash::placeholder());
        
        // Create the resource
        let resource_id = self.inner.create(resource).await?;
        
        Ok(resource_id)
    }
    
    // Implement the rest of the accessor methods
    // ...
}
```

## Best Practices

When working with the Resource System, follow these best practices:

1. **Content Hash Management**: Always ensure content hashes are correctly calculated and updated
   ```rust
   // Recalculate hash after modifying fields
   resource.title = new_title;
   resource.content_hash = resource.calculate_content_hash()?;
   ```

2. **Resource Lifecycle Management**: Use the lifecycle manager for all state transitions
   ```rust
   // Always use the lifecycle manager
   lifecycle_manager.transition(resource_id, new_state, reason).await?;
   ```

3. **Transaction Management**: Use transactions for multi-step operations
   ```rust
   // Start a transaction
   let tx = resource_manager.begin_transaction()?;
   
   // Perform multiple operations
   // ...
   
   // Commit or rollback
   tx.commit().await?;
   ```

4. **Capability Validation**: Always verify capabilities before performing operations
   ```rust
   // Check capability before operation
   if !capability_manager.verify(subject, capability).await? {
       return Err(ResourceError::Unauthorized);
   }
   ```

5. **Storage Strategy Selection**: Choose the appropriate storage strategy for each resource type
   ```rust
   // For sensitive data, use CommitmentBased
   let strategy = StorageStrategy::CommitmentBased {
       commitment: None,
       nullifier: None,
   };
   
   // For public data, use FullyOnChain
   let strategy = StorageStrategy::FullyOnChain {
       visibility: StateVisibility::Public,
   };
   ```

6. **Resource Relationships**: Explicitly track dependencies between resources
   ```rust
   // Create explicit relationships for dependencies
   relationship_tracker.create_relationship(
       source_id,
       target_id,
       RelationshipType::Dependency,
       metadata,
   ).await?;
   ```

7. **Optimized Content Addressing**: Use deferred hashing in ZK environments
   ```rust
   // Use deferred hashing context
   let hash_id = hashing_context.request_hash(&serialized);
   // Compute hashes later
   hashing_context.compute_deferred_hashes();
   ```

8. **Resource Type Registration**: Register all resource types with the resource manager
   ```rust
   // Register resource types
   resource_manager.register_resource_type::<DocumentResource>()?;
   resource_manager.register_resource_type::<ImageResource>()?;
   ```

9. **Cross-Domain Consistency**: Maintain consistency in cross-domain resources
   ```rust
   // Use transactions for cross-domain updates
   let tx = resource_manager.begin_transaction()?;
   source_accessor.update(source_id, source_resource).await?;
   target_accessor.update(target_id, target_resource).await?;
   tx.commit().await?;
   ```

10. **Resource Accessors**: Use the appropriate accessor for each resource type
    ```rust
    // Get type-specific accessor
    let document_accessor = resource_manager.get_accessor::<dyn DocumentAccessor>(domain_id)?;
    ```

## Troubleshooting

### Common Issues and Solutions

1. **Content Hash Mismatch**
   ```
   Problem: Resource validation fails due to content hash mismatch
   Solution: Ensure all fields affecting the hash are included in calculate_content_hash()
   ```

2. **Invalid State Transition**
   ```
   Problem: Resource lifecycle state transition fails
   Solution: Check allowed transitions in lifecycle manager configuration
   ```

3. **Resource Not Found**
   ```
   Problem: Resource operations fail with NotFound error
   Solution: Verify resource ID and domain, and check if the resource exists
   ```

4. **Capability Verification Failure**
   ```
   Problem: Operation fails due to capability verification
   Solution: Check if the capability has been granted and hasn't expired
   ```

5. **Cross-Domain Relationship Issues**
   ```
   Problem: Cross-domain operations fail
   Solution: Verify both resources exist and have the correct relationship
   ```

## Where to Go Next

1. Explore the [Resource System Architecture](../../architecture/core/resource-system.md) for more details on the design
2. Learn about [The Effect System](../../architecture/core/effect-system.md) to understand how storage effects work
3. See the [Capability System](../../architecture/core/capability-system.md) for more on resource authorization
4. Dive into [Content Addressing](../../architecture/core/content-addressing.md) for details on hash calculation

## Reference

### Related ADRs
- [ADR-003: Resource System](../../../spec/adr_003_resource.md)
- [ADR-021: Resource Register Unification](../../../spec/adr_021_resource_register_unification.md)
- [ADR-030: Deferred Hashing Out of VM](../../../spec/adr_030_deffered_hashing_out_of_vm.md)

### Related Architecture Documents
- [Resource System](../../architecture/core/resource-system.md)
- [Role-Based Resources](../../architecture/core/role-based-resources.md)
- [Capability System](../../architecture/core/capability-system.md) 