# Resource Register

This document outlines the Resource Register system in Causality, which serves as the foundational structure for managing resources throughout their lifecycle.

## Overview

The Resource Register is a core data structure in Causality that maintains the state, metadata, and relationships of resources in a unified model. It provides a consistent interface for resource management regardless of the underlying resource type, while enabling verification, traceability, and cross-domain operations.

## Register Structure

The Resource Register consists of several components that together form a comprehensive representation of a resource:

```rust
/// The unified resource register model
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceRegister {
    /// Unique identifier for the resource
    pub id: ResourceId,
    
    /// Current state of the resource in the register
    pub state: RegisterState,
    
    /// Resource type information
    pub resource_type: ResourceType,
    
    /// Resource attributes (data/properties)
    pub attributes: AttributeMap,
    
    /// Metadata about the resource
    pub metadata: MetadataMap,
    
    /// Capability information for this resource
    pub capabilities: CapabilitySet,
    
    /// Temporal information about the resource
    pub temporal_info: TemporalInfo,
    
    /// Ownership information for the resource
    pub ownership: OwnershipInfo,
    
    /// Verification information for the resource
    pub verification: VerificationInfo,
}
```

### Register State

The state field tracks the lifecycle state of a resource:

```rust
/// Possible states of a resource in the register
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterState {
    /// Resource is being initialized but not yet active
    Initializing,
    
    /// Resource is active and available for use
    Active,
    
    /// Resource is temporarily unavailable
    Suspended,
    
    /// Resource is read-only
    Frozen,
    
    /// Resource is being migrated 
    Migrating,
    
    /// Resource is being replaced
    Replacing,
    
    /// Resource is marked for deletion
    MarkedForDeletion,
    
    /// Resource has been deleted
    Deleted,
    
    /// Resource is in error state
    Error(String),
    
    /// Custom state with associated data
    Custom(String, Value),
}
```

### Attributes

Attributes store the actual data and properties of the resource:

```rust
/// Map of resource attributes
pub type AttributeMap = HashMap<String, Value>;

/// Creates default attributes for a resource type
pub fn create_default_attributes(resource_type: &ResourceType) -> AttributeMap {
    let mut attributes = AttributeMap::new();
    
    // Add default attributes based on resource type
    match resource_type {
        ResourceType::Data => {
            attributes.insert("content".to_string(), Value::Null);
            attributes.insert("format".to_string(), Value::String("text".to_string()));
            attributes.insert("size".to_string(), Value::Number(0.into()));
        },
        ResourceType::Compute => {
            attributes.insert("cpu".to_string(), Value::Number(1.into()));
            attributes.insert("memory".to_string(), Value::Number(512.into()));
            attributes.insert("status".to_string(), Value::String("idle".to_string()));
        },
        ResourceType::Service => {
            attributes.insert("endpoint".to_string(), Value::String("".to_string()));
            attributes.insert("status".to_string(), Value::String("stopped".to_string()));
            attributes.insert("version".to_string(), Value::String("1.0.0".to_string()));
        },
        // Other default attributes for other resource types
        _ => {},
    }
    
    attributes
}
```

### Metadata

Metadata provides additional information about the resource:

```rust
/// Map of resource metadata
pub type MetadataMap = HashMap<String, Value>;

/// Creates default metadata for a resource
pub fn create_default_metadata(resource_id: &ResourceId) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    
    // Add standard metadata fields
    metadata.insert("created_at".to_string(), json!(current_timestamp()));
    metadata.insert("updated_at".to_string(), json!(current_timestamp()));
    metadata.insert("origin_domain".to_string(), json!(resource_id.origin_domain.to_string()));
    metadata.insert("version".to_string(), json!(resource_id.version.to_string()));
    
    metadata
}
```

### Capabilities

The capabilities field defines what operations are permitted on the resource:

```rust
/// Set of capabilities for a resource
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Capabilities that are granted by default
    pub default_capabilities: Vec<Capability>,
    
    /// Capabilities granted to specific entities
    pub granted_capabilities: HashMap<EntityId, Vec<Capability>>,
    
    /// Capability requirements for operations
    pub capability_requirements: HashMap<OperationType, Vec<Capability>>,
}
```

### Temporal Information

Temporal information tracks time-related aspects of the resource:

```rust
/// Temporal information for a resource
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalInfo {
    /// When the resource was created
    pub created_at: TimeSnapshot,
    
    /// When the resource was last updated
    pub updated_at: TimeSnapshot,
    
    /// Time-to-live for the resource (if any)
    pub ttl: Option<Duration>,
    
    /// Scheduled operations
    pub scheduled_operations: Vec<ScheduledOperation>,
    
    /// Historical states
    pub state_history: Vec<StateHistoryEntry>,
}
```

### Ownership Information

Ownership tracks who owns and can manage the resource:

```rust
/// Ownership information for a resource
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OwnershipInfo {
    /// Primary owner of the resource
    pub owner: EntityId,
    
    /// Secondary owners (if any)
    pub co_owners: Vec<EntityId>,
    
    /// Whether ownership can be transferred
    pub transferable: bool,
    
    /// Delegation information
    pub delegations: Vec<OwnershipDelegation>,
}
```

### Verification Information

Verification information ensures the integrity and authenticity of the resource:

```rust
/// Verification information for a resource
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationInfo {
    /// Verification status
    pub status: VerificationStatus,
    
    /// Verification method used
    pub method: VerificationMethod,
    
    /// Verification evidence
    pub evidence: Vec<VerificationEvidence>,
    
    /// Last verification time
    pub last_verified: Option<TimeSnapshot>,
}
```

## Register Operations

The Resource Register supports various operations:

### Creation

```rust
/// Creates a new resource register
pub fn create_resource_register(
    resource_id: ResourceId,
    resource_type: ResourceType,
    initial_attributes: Option<AttributeMap>,
    owner: EntityId,
    creation_context: &CreationContext,
) -> Result<ResourceRegister> {
    // Create attributes (using defaults if not provided)
    let attributes = initial_attributes.unwrap_or_else(|| create_default_attributes(&resource_type));
    
    // Create metadata
    let metadata = create_default_metadata(&resource_id);
    
    // Create capabilities
    let capabilities = create_default_capabilities(&resource_type, &owner);
    
    // Create temporal info
    let current_time = creation_context.current_time.clone();
    let temporal_info = TemporalInfo {
        created_at: current_time.clone(),
        updated_at: current_time,
        ttl: None,
        scheduled_operations: Vec::new(),
        state_history: Vec::new(),
    };
    
    // Create ownership info
    let ownership = OwnershipInfo {
        owner,
        co_owners: Vec::new(),
        transferable: true,
        delegations: Vec::new(),
    };
    
    // Create verification info
    let verification = VerificationInfo {
        status: VerificationStatus::Unverified,
        method: VerificationMethod::None,
        evidence: Vec::new(),
        last_verified: None,
    };
    
    // Create the register
    let register = ResourceRegister {
        id: resource_id,
        state: RegisterState::Initializing,
        resource_type,
        attributes,
        metadata,
        capabilities,
        temporal_info,
        ownership,
        verification,
    };
    
    Ok(register)
}
```

### State Transitions

```rust
/// Transitions a resource register to a new state
pub fn transition_register_state(
    register: &mut ResourceRegister,
    new_state: RegisterState,
    transition_context: &TransitionContext,
) -> Result<()> {
    let old_state = register.state.clone();
    
    // Validate the transition
    validate_state_transition(&old_state, &new_state, register, transition_context)?;
    
    // Record the old state in history
    register.temporal_info.state_history.push(StateHistoryEntry {
        from_state: old_state,
        to_state: new_state.clone(),
        timestamp: transition_context.current_time.clone(),
        reason: transition_context.reason.clone(),
        initiated_by: transition_context.initiator.clone(),
    });
    
    // Update the state
    register.state = new_state;
    
    // Update the updated_at timestamp
    register.temporal_info.updated_at = transition_context.current_time.clone();
    
    // Apply state-specific changes
    apply_state_specific_changes(register, transition_context)?;
    
    Ok(())
}
```

### Attribute Updates

```rust
/// Updates attributes in a resource register
pub fn update_register_attributes(
    register: &mut ResourceRegister,
    attribute_updates: &AttributeMap,
    update_context: &UpdateContext,
) -> Result<()> {
    // Validate the updates
    validate_attribute_updates(register, attribute_updates, update_context)?;
    
    // Apply the updates
    for (key, value) in attribute_updates {
        register.attributes.insert(key.clone(), value.clone());
    }
    
    // Update the updated_at timestamp
    register.temporal_info.updated_at = update_context.current_time.clone();
    
    // Update metadata
    register.metadata.insert("updated_at".to_string(), json!(update_context.current_time.to_string()));
    register.metadata.insert("last_modifier".to_string(), json!(update_context.initiator.to_string()));
    
    Ok(())
}
```

### Capability Management

```rust
/// Grants capabilities to an entity for a resource
pub fn grant_capabilities(
    register: &mut ResourceRegister,
    entity: &EntityId,
    capabilities: &[Capability],
    grant_context: &CapabilityContext,
) -> Result<()> {
    // Validate the capability grant
    validate_capability_grant(register, entity, capabilities, grant_context)?;
    
    // Get or create the entity's capabilities
    let entity_capabilities = register.capabilities.granted_capabilities
        .entry(entity.clone())
        .or_insert_with(Vec::new);
    
    // Add new capabilities (avoiding duplicates)
    for capability in capabilities {
        if !entity_capabilities.contains(capability) {
            entity_capabilities.push(capability.clone());
        }
    }
    
    // Update the updated_at timestamp
    register.temporal_info.updated_at = grant_context.current_time.clone();
    
    Ok(())
}
```

### Verification

```rust
/// Verifies a resource register
pub fn verify_resource_register(
    register: &mut ResourceRegister,
    verification_method: VerificationMethod,
    verification_context: &VerificationContext,
) -> Result<VerificationStatus> {
    // Perform verification
    let (status, evidence) = perform_verification(register, &verification_method, verification_context)?;
    
    // Update verification information
    register.verification.status = status.clone();
    register.verification.method = verification_method;
    register.verification.evidence.push(evidence);
    register.verification.last_verified = Some(verification_context.current_time.clone());
    
    // Update the updated_at timestamp
    register.temporal_info.updated_at = verification_context.current_time.clone();
    
    Ok(status)
}
```

## Register Storage

The Resource Register is stored in a persistent storage system:

```rust
/// Storage interface for resource registers
pub trait ResourceRegisterStorage {
    /// Stores a resource register
    async fn store_register(&self, register: &ResourceRegister) -> Result<()>;
    
    /// Retrieves a resource register by ID
    async fn load_register(&self, id: &ResourceId) -> Result<Option<ResourceRegister>>;
    
    /// Updates a resource register
    async fn update_register(&self, register: &ResourceRegister) -> Result<()>;
    
    /// Deletes a resource register
    async fn delete_register(&self, id: &ResourceId) -> Result<()>;
    
    /// Lists resource registers matching a filter
    async fn list_registers(&self, filter: &RegisterFilter) -> Result<Vec<ResourceRegister>>;
}
```

## Register Serialization

Resource Registers can be serialized for storage and communication:

```rust
/// Serializes a resource register to JSON
pub fn serialize_register_to_json(register: &ResourceRegister) -> Result<String> {
    serde_json::to_string(register).map_err(|e| Error::SerializationError(e.to_string()))
}

/// Deserializes a resource register from JSON
pub fn deserialize_register_from_json(json: &str) -> Result<ResourceRegister> {
    serde_json::from_str(json).map_err(|e| Error::DeserializationError(e.to_string()))
}

/// Serializes a resource register to a compact binary format
pub fn serialize_register_to_binary(register: &ResourceRegister) -> Result<Vec<u8>> {
    bincode::serialize(register).map_err(|e| Error::SerializationError(e.to_string()))
}

/// Deserializes a resource register from a binary format
pub fn deserialize_register_from_binary(data: &[u8]) -> Result<ResourceRegister> {
    bincode::deserialize(data).map_err(|e| Error::DeserializationError(e.to_string()))
}
```

## Cross-Domain Register Management

Resource Registers can be managed across domain boundaries:

```rust
/// Transfers a resource register to another domain
pub async fn transfer_register_to_domain(
    register: &ResourceRegister,
    target_domain: &DomainId,
    transfer_context: &TransferContext,
) -> Result<ResourceId> {
    // Validate the transfer
    validate_cross_domain_transfer(register, target_domain, transfer_context)?;
    
    // Create a representation for the target domain
    let target_register = create_target_domain_register(register, target_domain, transfer_context)?;
    
    // Get the target domain client
    let target_domain_client = transfer_context.get_domain_client(target_domain)?;
    
    // Send the register to the target domain
    let target_resource_id = target_domain_client.import_register(&target_register).await?;
    
    // Record the transfer in the source domain
    record_register_transfer(register, target_domain, &target_resource_id, transfer_context).await?;
    
    Ok(target_resource_id)
}
```

## Register Validation

Resource Registers are validated to ensure integrity:

```rust
/// Validates a resource register
pub fn validate_resource_register(
    register: &ResourceRegister,
    validation_context: &ValidationContext,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();
    
    // Validate ID
    if let Err(e) = validate_resource_id(&register.id) {
        result.add_error(ValidationError::new(
            format!("Invalid resource ID: {}", e),
            ErrorSeverity::Critical,
        ));
    }
    
    // Validate resource type
    if let Err(e) = validate_resource_type(&register.resource_type) {
        result.add_error(ValidationError::new(
            format!("Invalid resource type: {}", e),
            ErrorSeverity::Critical,
        ));
    }
    
    // Validate attributes
    for validator in validation_context.attribute_validators {
        if let Err(e) = validator.validate_attributes(&register.attributes, &register.resource_type) {
            result.add_error(ValidationError::new(
                format!("Attribute validation failed: {}", e),
                ErrorSeverity::Error,
            ));
        }
    }
    
    // Validate capabilities
    if let Err(e) = validate_capability_set(&register.capabilities, validation_context) {
        result.add_error(ValidationError::new(
            format!("Capability validation failed: {}", e),
            ErrorSeverity::Warning,
        ));
    }
    
    // Validate temporal information
    if let Err(e) = validate_temporal_info(&register.temporal_info, validation_context) {
        result.add_error(ValidationError::new(
            format!("Temporal validation failed: {}", e),
            ErrorSeverity::Warning,
        ));
    }
    
    // Validate ownership
    if let Err(e) = validate_ownership_info(&register.ownership, validation_context) {
        result.add_error(ValidationError::new(
            format!("Ownership validation failed: {}", e),
            ErrorSeverity::Error,
        ));
    }
    
    // Validate custom rules
    for validator in validation_context.custom_validators {
        if let Err(e) = validator.validate_register(register) {
            result.add_error(ValidationError::new(
                format!("Custom validation failed: {}", e),
                ErrorSeverity::Error,
            ));
        }
    }
    
    Ok(result)
}
```

## Register Snapshots

Resource Registers can be snapshotted for historical tracking:

```rust
/// Creates a snapshot of a resource register
pub fn create_register_snapshot(
    register: &ResourceRegister,
    snapshot_reason: &str,
    snapshot_context: &SnapshotContext,
) -> Result<RegisterSnapshot> {
    let snapshot = RegisterSnapshot {
        register: register.clone(),
        snapshot_time: snapshot_context.current_time.clone(),
        snapshot_reason: snapshot_reason.to_string(),
        snapshot_id: generate_snapshot_id(register, snapshot_context),
        initiator: snapshot_context.initiator.clone(),
    };
    
    Ok(snapshot)
}

/// Retrieves a historical snapshot of a register
pub async fn get_register_snapshot(
    resource_id: &ResourceId,
    snapshot_id: &SnapshotId,
    storage: &dyn ResourceRegisterStorage,
) -> Result<Option<RegisterSnapshot>> {
    storage.get_snapshot(resource_id, snapshot_id).await
}

/// Lists snapshots for a register
pub async fn list_register_snapshots(
    resource_id: &ResourceId,
    filter: &SnapshotFilter,
    storage: &dyn ResourceRegisterStorage,
) -> Result<Vec<SnapshotMetadata>> {
    storage.list_snapshots(resource_id, filter).await
}
```

## Integration with Resource System

The Resource Register integrates with the broader resource system:

```rust
/// Resource Manager using Resource Registers
pub struct ResourceManager {
    register_storage: Box<dyn ResourceRegisterStorage>,
    relationship_manager: RelationshipManager,
    capability_manager: CapabilityManager,
    validation_pipeline: ValidationPipeline,
}

impl ResourceManager {
    /// Creates a new resource
    pub async fn create_resource(
        &self,
        resource_type: ResourceType,
        attributes: AttributeMap,
        owner: EntityId,
        context: &OperationContext,
    ) -> Result<ResourceId> {
        // Generate resource ID
        let resource_id = generate_resource_id(&resource_type, &context.domain_id, None);
        
        // Create the register
        let mut register = create_resource_register(
            resource_id.clone(),
            resource_type,
            Some(attributes),
            owner,
            &CreationContext::from_operation_context(context),
        )?;
        
        // Validate the register
        let validation_result = self.validation_pipeline.validate_register(
            &register,
            &ValidationContext::from_operation_context(context),
        )?;
        
        if !validation_result.is_valid() {
            return Err(Error::ValidationFailed(validation_result));
        }
        
        // Store the register
        self.register_storage.store_register(&register).await?;
        
        // Activate the resource
        transition_register_state(
            &mut register, 
            RegisterState::Active,
            &TransitionContext::new(
                context.current_time.clone(),
                "Resource activation after creation".to_string(),
                context.initiator.clone(),
            ),
        )?;
        
        // Update the stored register
        self.register_storage.update_register(&register).await?;
        
        Ok(resource_id)
    }
    
    /// Retrieves a resource
    pub async fn get_resource(
        &self,
        resource_id: &ResourceId,
        context: &OperationContext,
    ) -> Result<Option<ResourceRegister>> {
        // Load the register
        let register_option = self.register_storage.load_register(resource_id).await?;
        
        if let Some(register) = register_option {
            // Check access
            if !self.capability_manager.check_capability(
                &context.initiator,
                &register,
                &Capability::Read,
                context,
            )? {
                return Err(Error::AccessDenied);
            }
            
            Ok(Some(register))
        } else {
            Ok(None)
        }
    }
    
    /// Updates a resource
    pub async fn update_resource(
        &self,
        resource_id: &ResourceId,
        attribute_updates: AttributeMap,
        context: &OperationContext,
    ) -> Result<()> {
        // Load the register
        let register_option = self.register_storage.load_register(resource_id).await?;
        
        if let Some(mut register) = register_option {
            // Check access
            if !self.capability_manager.check_capability(
                &context.initiator,
                &register,
                &Capability::Update,
                context,
            )? {
                return Err(Error::AccessDenied);
            }
            
            // Update attributes
            update_register_attributes(
                &mut register,
                &attribute_updates,
                &UpdateContext::from_operation_context(context),
            )?;
            
            // Validate the updated register
            let validation_result = self.validation_pipeline.validate_register(
                &register,
                &ValidationContext::from_operation_context(context),
            )?;
            
            if !validation_result.is_valid() {
                return Err(Error::ValidationFailed(validation_result));
            }
            
            // Update the stored register
            self.register_storage.update_register(&register).await?;
            
            Ok(())
        } else {
            Err(Error::ResourceNotFound(resource_id.clone()))
        }
    }
}
```

## Conclusion

The Resource Register provides a unified model for managing resources in Causality. It encapsulates all aspects of a resource's state, attributes, capabilities, and lifecycle, enabling consistent management across different resource types and domains. The register's rich metadata and verification features support auditability, security, and verification requirements, while its flexible structure allows for extension to new resource types and use cases. 