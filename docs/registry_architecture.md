# Registry Architecture in Causality

## Overview

This document describes the Registry Architecture within the Causality framework. The Registry system serves as a central component for registering, discovering, and managing various system entities including resources, capabilities, interfaces, facts, and other critical components. It provides a unified approach to entity management across domains while enforcing appropriate validation, authorization, and consistency checks.

## Core Concepts

### Registry Model

The core registry model is built around a generalized concept of registerable entities:

```rust
/// A trait for any entity that can be registered in a registry
pub trait Registerable: Send + Sync {
    /// Get the unique identifier for this entity
    fn id(&self) -> EntityId;
    
    /// Get the type of this entity
    fn entity_type(&self) -> EntityType;
    
    /// Validate this entity
    fn validate(&self) -> Result<ValidationResult, ValidationError>;
    
    /// Get the serialized representation of this entity
    fn serialize(&self) -> Result<Vec<u8>, SerializationError>;
}

/// Registry for managing registerable entities
pub struct Registry<T: Registerable> {
    /// Registered entities
    entities: RwLock<HashMap<EntityId, T>>,
    
    /// Indexes for efficient querying
    indexes: RwLock<HashMap<String, Box<dyn Index<T>>>>,
    
    /// Validators for entity validation
    validators: Vec<Box<dyn Validator<T>>>,
    
    /// Observers for entity lifecycle events
    observers: Vec<Box<dyn RegistryObserver<T>>>,
    
    /// Registry configuration
    config: RegistryConfig,
}
```

### Registry Hub

The Registry Hub acts as a central coordination point for all registries:

```rust
pub struct RegistryHub {
    /// Resource registry
    resource_registry: Arc<ResourceRegistry>,
    
    /// Capability registry
    capability_registry: Arc<CapabilityRegistry>,
    
    /// Interface registry
    interface_registry: Arc<InterfaceRegistry>,
    
    /// Fact registry
    fact_registry: Arc<FactRegistry>,
    
    /// Relationship registry
    relationship_registry: Arc<RelationshipRegistry>,
    
    /// Operation registry
    operation_registry: Arc<OperationRegistry>,
    
    /// Transaction registry
    transaction_registry: Arc<TransactionRegistry>,
    
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    
    /// Registry configuration
    config: RegistryHubConfig,
}
```

## Registry Types

### Resource Registry

Manages resources within the system:

```rust
pub struct ResourceRegistry {
    /// Core registry functionality
    registry: Registry<Resource>,
    
    /// Resource state manager
    state_manager: Arc<ResourceStateManager>,
    
    /// Resource lifecycle manager
    lifecycle_manager: Arc<ResourceLifecycleManager>,
    
    /// Resource validation pipeline
    validation_pipeline: Arc<ResourceValidationPipeline>,
}

impl ResourceRegistry {
    /// Register a new resource
    pub fn register_resource(&self, resource: Resource) -> Result<ResourceId, RegistryError> {
        // Validate the resource before registration
        let validation_result = self.validation_pipeline.validate_resource(&resource)?;
        if !validation_result.is_valid() {
            return Err(RegistryError::ValidationFailed(validation_result));
        }
        
        // Register the resource
        let resource_id = resource.id();
        self.registry.register(resource.clone())?;
        
        // Initialize resource state
        self.state_manager.initialize_state(&resource)?;
        
        // Trigger lifecycle events
        self.lifecycle_manager.handle_creation(&resource)?;
        
        Ok(resource_id)
    }
    
    /// Get a resource by ID
    pub fn get_resource(&self, id: ResourceId) -> Result<Resource, RegistryError> {
        self.registry.get(&id.into())
    }
    
    /// Query resources based on criteria
    pub fn query_resources(&self, query: ResourceQuery) -> Result<Vec<Resource>, RegistryError> {
        self.registry.query(query)
    }
    
    /// Update a resource
    pub fn update_resource(&self, id: ResourceId, updates: ResourceUpdates) -> Result<Resource, RegistryError> {
        // Get the current resource
        let current = self.get_resource(id)?;
        
        // Apply updates
        let updated = current.apply_updates(updates)?;
        
        // Validate the updated resource
        let validation_result = self.validation_pipeline.validate_resource(&updated)?;
        if !validation_result.is_valid() {
            return Err(RegistryError::ValidationFailed(validation_result));
        }
        
        // Update in registry
        self.registry.update(updated.clone())?;
        
        // Trigger lifecycle events
        self.lifecycle_manager.handle_update(&current, &updated)?;
        
        Ok(updated)
    }
    
    /// Deregister a resource
    pub fn deregister_resource(&self, id: ResourceId) -> Result<(), RegistryError> {
        // Get the current resource
        let resource = self.get_resource(id)?;
        
        // Check if deregistration is allowed
        let can_deregister = self.lifecycle_manager.can_deregister(&resource)?;
        if !can_deregister.is_allowed() {
            return Err(RegistryError::DeregistrationNotAllowed(can_deregister.reason()));
        }
        
        // Trigger lifecycle events before deregistration
        self.lifecycle_manager.handle_deregistration(&resource)?;
        
        // Deregister from registry
        self.registry.deregister(&id.into())?;
        
        Ok(())
    }
}
```

### Capability Registry

Manages capabilities for authorization:

```rust
pub struct CapabilityRegistry {
    /// Core registry functionality
    registry: Registry<Capability>,
    
    /// Capability validation pipeline
    validation_pipeline: Arc<CapabilityValidationPipeline>,
    
    /// Capability resolver
    resolver: Arc<CapabilityResolver>,
}

impl CapabilityRegistry {
    /// Register a new capability
    pub fn register_capability(&self, capability: Capability) -> Result<CapabilityId, RegistryError> {
        // Validate the capability
        let validation_result = self.validation_pipeline.validate_capability(&capability)?;
        if !validation_result.is_valid() {
            return Err(RegistryError::ValidationFailed(validation_result));
        }
        
        // Register the capability
        let capability_id = capability.id();
        self.registry.register(capability)?;
        
        Ok(capability_id)
    }
    
    /// Check if a capability grants permission for an operation
    pub fn check_capability_for_operation(
        &self,
        capability_id: CapabilityId,
        operation: &Operation,
    ) -> Result<CapabilityCheckResult, RegistryError> {
        // Get the capability
        let capability = self.registry.get(&capability_id.into())?;
        
        // Resolve the capability for the operation
        self.resolver.resolve_for_operation(&capability, operation)
    }
    
    /// Find capabilities that grant permission for an operation
    pub fn find_capabilities_for_operation(
        &self,
        operation: &Operation,
        holder: CapabilityHolder,
    ) -> Result<Vec<Capability>, RegistryError> {
        // Query capabilities for the holder
        let query = CapabilityQuery::new()
            .with_holder(holder)
            .with_resource_id(operation.resource_id());
        
        let capabilities = self.registry.query(query)?;
        
        // Filter to those that grant permission for the operation
        let mut granted_capabilities = Vec::new();
        for capability in capabilities {
            let result = self.resolver.resolve_for_operation(&capability, operation)?;
            if result.is_granted() {
                granted_capabilities.push(capability);
            }
        }
        
        Ok(granted_capabilities)
    }
}
```

### Interface Registry

Manages resource interfaces:

```rust
pub struct InterfaceRegistry {
    /// Core registry functionality
    registry: Registry<ResourceInterface>,
    
    /// Interface validation pipeline
    validation_pipeline: Arc<InterfaceValidationPipeline>,
    
    /// Interface implementation registry
    implementation_registry: Arc<InterfaceImplementationRegistry>,
}

impl InterfaceRegistry {
    /// Register a new interface
    pub fn register_interface(&self, interface: ResourceInterface) -> Result<InterfaceId, RegistryError> {
        // Validate the interface
        let validation_result = self.validation_pipeline.validate_interface(&interface)?;
        if !validation_result.is_valid() {
            return Err(RegistryError::ValidationFailed(validation_result));
        }
        
        // Register the interface
        let interface_id = interface.id();
        self.registry.register(interface)?;
        
        Ok(interface_id)
    }
    
    /// Register a resource as implementing an interface
    pub fn register_implementation(
        &self,
        resource_id: ResourceId,
        interface_id: InterfaceId,
        implementation_data: ImplementationData,
    ) -> Result<ImplementationId, RegistryError> {
        // Check if interface exists
        let interface = self.registry.get(&interface_id.into())?;
        
        // Check if resource exists
        system.resource_registry().get_resource(resource_id)?;
        
        // Create implementation
        let implementation = InterfaceImplementation {
            id: ImplementationId::generate(),
            resource_id,
            interface_id,
            data: implementation_data,
        };
        
        // Validate implementation
        let validation_result = self.validation_pipeline.validate_implementation(&implementation, &interface)?;
        if !validation_result.is_valid() {
            return Err(RegistryError::ValidationFailed(validation_result));
        }
        
        // Register implementation
        self.implementation_registry.register_implementation(implementation)
    }
    
    /// Find resources implementing an interface
    pub fn find_implementers(&self, interface_id: InterfaceId) -> Result<Vec<ResourceId>, RegistryError> {
        self.implementation_registry.find_implementers(interface_id)
    }
    
    /// Check if a resource implements an interface
    pub fn implements_interface(
        &self,
        resource_id: ResourceId,
        interface_id: InterfaceId,
    ) -> Result<bool, RegistryError> {
        self.implementation_registry.has_implementation(resource_id, interface_id)
    }
}
```

### Fact Registry

Manages temporal facts:

```rust
pub struct FactRegistry {
    /// Core registry functionality
    registry: Registry<TemporalFact>,
    
    /// Fact validation pipeline
    validation_pipeline: Arc<FactValidationPipeline>,
    
    /// Fact indexers for efficient queries
    indexers: Vec<Box<dyn FactIndexer>>,
}

impl FactRegistry {
    /// Register a new fact
    pub fn register_fact(&self, fact: TemporalFact) -> Result<FactId, RegistryError> {
        // Validate the fact
        let validation_result = self.validation_pipeline.validate_fact(&fact)?;
        if !validation_result.is_valid() {
            return Err(RegistryError::ValidationFailed(validation_result));
        }
        
        // Register the fact
        let fact_id = fact.id();
        self.registry.register(fact)?;
        
        // Update indexes
        for indexer in &self.indexers {
            indexer.index_fact(&fact)?;
        }
        
        Ok(fact_id)
    }
    
    /// Get a fact by ID
    pub fn get_fact(&self, id: FactId) -> Option<TemporalFact> {
        self.registry.get(&id.into()).ok()
    }
    
    /// Query facts based on criteria
    pub fn query_facts(
        &self,
        filter: FactFilter,
        pagination: Option<Pagination>,
    ) -> Result<Vec<TemporalFact>, RegistryError> {
        self.registry.query(filter.into_query(pagination))
    }
}
```

### Relationship Registry

Manages relationships between resources:

```rust
pub struct RelationshipRegistry {
    /// Core registry functionality
    registry: Registry<Relationship>,
    
    /// Relationship validation pipeline
    validation_pipeline: Arc<RelationshipValidationPipeline>,
    
    /// Relationship graph for efficient traversal
    relationship_graph: Arc<RelationshipGraph>,
}

impl RelationshipRegistry {
    /// Register a new relationship
    pub fn register_relationship(&self, relationship: Relationship) -> Result<RelationshipId, RegistryError> {
        // Validate the relationship
        let validation_result = self.validation_pipeline.validate_relationship(&relationship)?;
        if !validation_result.is_valid() {
            return Err(RegistryError::ValidationFailed(validation_result));
        }
        
        // Register the relationship
        let relationship_id = relationship.id();
        self.registry.register(relationship.clone())?;
        
        // Update relationship graph
        self.relationship_graph.add_relationship(&relationship)?;
        
        Ok(relationship_id)
    }
    
    /// Find relationships for a resource
    pub fn find_relationships_for_resource(
        &self,
        resource_id: ResourceId,
        relationship_type: Option<RelationshipType>,
        role: Option<RelationshipRole>,
    ) -> Result<Vec<Relationship>, RegistryError> {
        let query = RelationshipQuery::new()
            .with_resource(resource_id)
            .with_type_opt(relationship_type)
            .with_role_opt(role);
            
        self.registry.query(query)
    }
    
    /// Find related resources
    pub fn find_related_resources(
        &self,
        resource_id: ResourceId,
        params: RelatedResourceParams,
    ) -> Result<Vec<RelatedResource>, RegistryError> {
        self.relationship_graph.find_related_resources(resource_id, params)
    }
    
    /// Traverse relationships to find path between resources
    pub fn find_path(
        &self,
        from_resource: ResourceId,
        to_resource: ResourceId,
        params: PathParams,
    ) -> Result<Option<RelationshipPath>, RegistryError> {
        self.relationship_graph.find_path(from_resource, to_resource, params)
    }
}
```

## Registry Infrastructure

### Registry Storage

Core storage abstractions for registries:

```rust
/// Storage backend for registry data
pub trait RegistryStorage<T: Registerable>: Send + Sync {
    /// Store an entity
    fn store(&self, entity: &T) -> Result<(), StorageError>;
    
    /// Retrieve an entity by ID
    fn retrieve(&self, id: &EntityId) -> Result<T, StorageError>;
    
    /// Delete an entity
    fn delete(&self, id: &EntityId) -> Result<(), StorageError>;
    
    /// List entities matching criteria
    fn list(&self, criteria: &StorageCriteria) -> Result<Vec<T>, StorageError>;
    
    /// Count entities matching criteria
    fn count(&self, criteria: &StorageCriteria) -> Result<usize, StorageError>;
}

/// Transactional registry storage
pub trait TransactionalStorage<T: Registerable>: RegistryStorage<T> {
    /// Begin a transaction
    fn begin_transaction(&self) -> Result<Transaction, StorageError>;
    
    /// Store within a transaction
    fn store_in_transaction(&self, transaction: &Transaction, entity: &T) -> Result<(), StorageError>;
    
    /// Delete within a transaction
    fn delete_in_transaction(&self, transaction: &Transaction, id: &EntityId) -> Result<(), StorageError>;
    
    /// Commit a transaction
    fn commit_transaction(&self, transaction: Transaction) -> Result<(), StorageError>;
    
    /// Rollback a transaction
    fn rollback_transaction(&self, transaction: Transaction) -> Result<(), StorageError>;
}
```

### Registry Indexing

Infrastructure for efficient registry queries:

```rust
/// Index for efficient entity lookup
pub trait Index<T: Registerable>: Send + Sync {
    /// Index name
    fn name(&self) -> &str;
    
    /// Add or update an entity in the index
    fn index_entity(&self, entity: &T) -> Result<(), IndexError>;
    
    /// Remove an entity from the index
    fn remove_entity(&self, id: &EntityId) -> Result<(), IndexError>;
    
    /// Find entities matching the criteria
    fn find(&self, criteria: &IndexCriteria) -> Result<Vec<EntityId>, IndexError>;
}

/// Implementation of a B-tree based index
pub struct BTreeIndex<T: Registerable> {
    /// Index name
    name: String,
    
    /// Field extractor for getting index values from entities
    field_extractor: Box<dyn Fn(&T) -> Result<Vec<IndexValue>, IndexError> + Send + Sync>,
    
    /// The actual index
    index: RwLock<BTreeMap<IndexValue, HashSet<EntityId>>>,
}
```

### Registry Events

Events for registry change notifications:

```rust
/// Event types for registry changes
pub enum RegistryEvent<T: Registerable> {
    /// Entity registered
    Registered {
        /// The newly registered entity
        entity: T,
    },
    
    /// Entity updated
    Updated {
        /// Previous version of the entity
        previous: T,
        /// New version of the entity
        updated: T,
    },
    
    /// Entity deregistered
    Deregistered {
        /// The deregistered entity
        entity: T,
    },
}

/// Observer for registry events
pub trait RegistryObserver<T: Registerable>: Send + Sync {
    /// Handle a registry event
    fn on_event(&self, event: &RegistryEvent<T>) -> Result<(), ObserverError>;
    
    /// Filter for which events this observer wants
    fn event_filter(&self) -> EventFilter;
}
```

## Cross-Domain Registry

### Domain Registry

Tracks available domains and their registries:

```rust
pub struct DomainRegistry {
    /// Core registry functionality
    registry: Registry<Domain>,
    
    /// Cross-domain messenger
    messenger: Arc<CrossDomainMessenger>,
}

impl DomainRegistry {
    /// Register a new domain
    pub fn register_domain(&self, domain: Domain) -> Result<DomainId, RegistryError> {
        let domain_id = domain.id();
        self.registry.register(domain)?;
        Ok(domain_id)
    }
    
    /// Get registry address for a domain
    pub fn get_registry_address(&self, domain_id: DomainId) -> Result<RegistryAddress, RegistryError> {
        let domain = self.registry.get(&domain_id.into())?;
        Ok(domain.registry_address())
    }
}
```

### Federated Registry Queries

Query across multiple registry domains:

```rust
/// Execute a query across multiple domains
pub fn query_across_domains<T: Registerable>(
    query: &Query<T>,
    domains: &[DomainId],
) -> Result<Vec<T>, RegistryError> {
    let mut results = Vec::new();
    
    // Local domain
    let local_domain = system.domain_id();
    if domains.contains(&local_domain) {
        let local_results = system.registry_hub().get_registry::<T>().query(query.clone())?;
        results.extend(local_results);
    }
    
    // Remote domains
    for &domain_id in domains {
        if domain_id == local_domain {
            continue; // Already handled above
        }
        
        // Get registry address
        let address = system.domain_registry().get_registry_address(domain_id)?;
        
        // Create cross-domain query message
        let message = CrossDomainMessage::RegistryQuery {
            query_type: query.query_type(),
            query_data: query.serialize()?,
            requester_domain: local_domain,
            timestamp: system.current_time(),
        };
        
        // Send query and get response
        let response = system.cross_domain_messenger()
            .send_and_wait_response(domain_id, message, Duration::from_secs(10))?;
        
        // Process response
        if let CrossDomainMessage::RegistryQueryResponse { results: remote_results, .. } = response {
            for result_data in remote_results {
                let entity = T::deserialize(&result_data)?;
                results.push(entity);
            }
        }
    }
    
    Ok(results)
}
```

## Registry Operations

### Registry Transactions

Enables atomic registry operations:

```rust
pub struct RegistryTransaction {
    /// Transaction ID
    id: TransactionId,
    
    /// Operations to perform
    operations: Vec<RegistryOperation>,
    
    /// Transaction status
    status: TransactionStatus,
    
    /// Timestamp of creation
    created_at: Timestamp,
    
    /// Timestamp of last update
    updated_at: Timestamp,
}

impl RegistryTransaction {
    /// Create a new transaction
    pub fn new() -> Self {
        Self {
            id: TransactionId::generate(),
            operations: Vec::new(),
            status: TransactionStatus::Created,
            created_at: system.current_time(),
            updated_at: system.current_time(),
        }
    }
    
    /// Add a registry operation
    pub fn add_operation(&mut self, operation: RegistryOperation) -> &mut Self {
        self.operations.push(operation);
        self.updated_at = system.current_time();
        self
    }
    
    /// Execute the transaction
    pub fn execute(self) -> Result<TransactionResult, TransactionError> {
        system.registry_hub().execute_transaction(self)
    }
}
```

### Registry Synchronization

Synchronizes registry data across domains:

```rust
pub struct RegistrySynchronizer {
    /// Synchronization strategies
    strategies: HashMap<EntityType, Box<dyn SyncStrategy>>,
    
    /// Cross-domain messenger
    messenger: Arc<CrossDomainMessenger>,
}

impl RegistrySynchronizer {
    /// Synchronize an entity with another domain
    pub fn synchronize_entity<T: Registerable>(
        &self,
        entity_id: EntityId,
        target_domain: DomainId,
    ) -> Result<SyncResult, SyncError> {
        // Get the entity
        let entity = system.registry_hub().get_registry::<T>().get(&entity_id)?;
        
        // Get synchronization strategy
        let strategy = self.strategies.get(&entity.entity_type())
            .ok_or(SyncError::UnsupportedEntityType(entity.entity_type()))?;
        
        // Execute synchronization
        strategy.synchronize_entity(&entity, target_domain)
    }
}
```

## Usage Examples

### Registering a Resource

```rust
// Create a new resource
let resource = Resource::new(
    ResourceType::new("document"),
    ResourceAttributes::new()
        .with_attribute("name", "Example Document")
        .with_attribute("owner", user_id.to_string())
        .with_attribute("status", "draft"),
);

// Register the resource
let resource_id = system.registry_hub()
    .resource_registry()
    .register_resource(resource)?;

println!("Registered resource with ID: {}", resource_id);
```

### Querying Resources with Filters

```rust
// Create a query for resources
let query = ResourceQuery::new()
    .with_type("document")
    .with_attribute_equals("status", "published")
    .with_attribute_contains("tags", "important")
    .with_owner(user_id)
    .with_limit(10);

// Execute the query
let resources = system.registry_hub()
    .resource_registry()
    .query_resources(query)?;

println!("Found {} matching resources", resources.len());
```

### Creating Resource Relationships

```rust
// Define a relationship between resources
let relationship = Relationship::new(
    RelationshipType::new("references"),
    ResourceEndpoint::new(source_resource_id, "source"),
    ResourceEndpoint::new(target_resource_id, "target"),
    RelationshipAttributes::new()
        .with_attribute("created_at", system.current_time().to_string())
        .with_attribute("strength", "strong"),
);

// Register the relationship
let relationship_id = system.registry_hub()
    .relationship_registry()
    .register_relationship(relationship)?;

println!("Created relationship with ID: {}", relationship_id);
```

### Transactional Registry Operations

```rust
// Create a transaction for multiple registry operations
let mut transaction = RegistryTransaction::new();

// Add operations to the transaction
transaction
    .add_operation(RegistryOperation::RegisterResource {
        resource: document_resource.clone(),
    })
    .add_operation(RegistryOperation::RegisterResource {
        resource: metadata_resource.clone(),
    })
    .add_operation(RegistryOperation::RegisterRelationship {
        relationship: Relationship::new(
            RelationshipType::new("describes"),
            ResourceEndpoint::new(metadata_resource.id(), "metadata"),
            ResourceEndpoint::new(document_resource.id(), "document"),
            RelationshipAttributes::default(),
        ),
    });

// Execute the transaction
let result = transaction.execute()?;

println!("Transaction completed with {} operations", result.completed_operations);
```

## Registry Security

### Access Control for Registry Operations

```rust
/// Check if an operation is allowed on a registry
fn check_registry_access(
    registry_type: RegistryType,
    operation_type: RegistryOperationType,
    auth_context: &AuthContext,
) -> Result<bool, SecurityError> {
    // Get the registry security policy
    let policy = system.security_policies().get_registry_policy(registry_type)?;
    
    // Check if the operation is allowed
    let access_check = policy.check_access(
        operation_type,
        auth_context.identity(),
        auth_context.capabilities(),
    )?;
    
    Ok(access_check.is_allowed())
}
```

### Registry Audit Logging

```rust
/// Log a registry audit event
fn log_registry_audit(
    registry_type: RegistryType,
    operation_type: RegistryOperationType,
    entity_id: Option<EntityId>,
    auth_context: &AuthContext,
    result: &OperationResult,
) -> Result<(), AuditError> {
    // Create audit event
    let audit_event = AuditEvent {
        event_type: AuditEventType::RegistryOperation {
            registry_type,
            operation_type,
        },
        timestamp: system.current_time(),
        identity: auth_context.identity().clone(),
        entity_id,
        result: result.clone(),
        metadata: AuditMetadata::new()
            .with_field("source_ip", auth_context.get_metadata("source_ip"))
            .with_field("session_id", auth_context.get_metadata("session_id")),
    };
    
    // Log the audit event
    system.audit_logger().log_event(audit_event)
}
```

## Implementation Status

The current implementation status of the Registry Architecture:

- ✅ Core registry abstractions
- ✅ Resource registry
- ✅ Capability registry
- ✅ Interface registry
- ✅ Fact registry
- ✅ Relationship registry
- ⚠️ Registry transactions (partially implemented)
- ⚠️ Registry synchronization (partially implemented)
- ⚠️ Cross-domain registry operations (partially implemented)
- ❌ Advanced registry query optimization (not yet implemented)
- ❌ Registry replication (not yet implemented)

## Future Enhancements

Planned future enhancements for the Registry Architecture:

1. **Distributed Registry**: Fully distributed registry implementation with consensus
2. **Query Optimization**: Advanced query optimization for complex queries
3. **Caching Layer**: Multi-level caching for registry data
4. **Schema Evolution**: Support for evolving entity schemas over time
5. **Registry Sharding**: Sharding strategies for large-scale registry data
6. **Subscription API**: Real-time notifications for registry changes
7. **Registry Analytics**: Advanced analytics on registry usage and patterns
8. **Custom Indexes**: User-definable indexes for domain-specific queries 