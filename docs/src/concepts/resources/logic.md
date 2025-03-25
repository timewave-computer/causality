<!-- Logic for resources -->
<!-- Original file: docs/src/resource_logic.md -->

# ResourceRegister Logic

This document outlines the ResourceRegister logic system in Causality, detailing how business logic is attached to unified ResourceRegisters, processed, and validated through the three-layer effect architecture and unified verification framework.

## Overview

ResourceRegister logic encapsulates the behavior, constraints, and operations associated with resources. It provides mechanisms for defining, executing, and validating resource-specific behaviors while ensuring consistency with system-wide constraints and verification requirements. With the unified ResourceRegister model, logic is applied consistently across the entire system.

## Logic Components

### Content-Addressed Logic Modules

ResourceRegister logic is organized into content-addressed modules that implement specific behaviors:

```rust
/// A logic module for a specific ResourceRegister type
pub trait ResourceLogic: ContentAddressed + Send + Sync {
    /// Resource type this logic applies to
    fn resource_type(&self) -> ResourceType;
    
    /// Validate a proposed operation on this resource
    fn validate_operation(&self, resource: &ResourceRegister, operation: &Operation<C>) -> Result<(), LogicError>;
    
    /// Apply an operation to this resource, transforming it as needed
    fn apply_operation(&self, resource: &mut ResourceRegister, operation: &Operation<C>) -> Result<Vec<Effect>, LogicError>;
    
    /// Check conservation rules for this resource
    fn check_conservation(&self, resources: &[ResourceRegister], delta: &ResourceDelta) -> Result<(), ConservationError>;
    
    /// Get constraints for operations on this resource
    fn get_constraints(&self, operation_type: OperationType) -> Vec<Constraint>;
    
    /// Get the resource schema
    fn get_schema(&self) -> ResourceSchema;
    
    /// Returns the type identifier for this logic
    fn logic_type(&self) -> LogicType;
}
```

### Content-Addressed Logic Registry

The logic registry manages content-addressed logic modules for different resource types:

```rust
/// Registry for resource logic modules
pub struct ResourceLogicRegistry {
    /// Modules indexed by resource type
    modules: HashMap<ResourceType, ContentRef<Box<dyn ResourceLogic>>>,
    
    /// Fallback module for unknown resource types
    fallback_module: Option<ContentRef<Box<dyn ResourceLogic>>>,
    
    /// Storage for logic modules
    storage: Arc<dyn ContentAddressedStorage>,
}

impl ResourceLogicRegistry {
    /// Registers a new logic module
    pub fn register_module(&mut self, module: Box<dyn ResourceLogic>) -> Result<ContentHash, RegistrationError> {
        let resource_type = module.resource_type();
        
        if self.modules.contains_key(&resource_type) {
            return Err(RegistrationError::ModuleAlreadyRegistered(resource_type));
        }
        
        // Store the module in content-addressed storage
        let content_hash = self.storage.store(&module)?;
        
        // Create a content reference
        let module_ref = ContentRef::<Box<dyn ResourceLogic>> {
            hash: content_hash.clone(),
            reference_type: ReferenceType::ContentAddressed,
            required_capabilities: HashSet::new(),
            relationship_type: None,
            domain_id: DomainId::default(),
            phantom: PhantomData,
        };
        
        self.modules.insert(resource_type, module_ref);
        
        Ok(content_hash)
    }
    
    /// Gets the module for a specific resource type
    pub fn get_module(&self, resource_type: &ResourceType) -> Result<Box<dyn ResourceLogic>, RetrievalError> {
        if let Some(module_ref) = self.modules.get(resource_type) {
            return module_ref.resolve(&self.storage);
        }
        
        if let Some(fallback_ref) = &self.fallback_module {
            return fallback_ref.resolve(&self.storage);
        }
        
        Err(RetrievalError::ModuleNotFound(resource_type.clone()))
    }
}
```

## Three-Layer Effect Architecture Integration

ResourceRegister logic integrates with the three-layer effect architecture:

### 1. Algebraic Effect Layer (First Layer)

```rust
/// Base trait for all resource logic effects
pub trait ResourceEffect: Effect {
    /// The resource this effect applies to
    fn resource(&self) -> RegisterId;
    
    /// The domains this effect interacts with
    fn domains(&self) -> Vec<DomainId>;
}
```

### 2. Effect Constraints Layer (Second Layer)

```rust
/// Constraints for resource transformation effects
pub trait ResourceTransformationEffect: ResourceEffect {
    /// The source resource state
    fn source_state(&self) -> RegisterState;
    
    /// The target resource state
    fn target_state(&self) -> RegisterState;
    
    /// The transformation parameters
    fn parameters(&self) -> HashMap<String, Value>;
    
    /// Validate the transformation
    fn validate_transformation(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        // Get the resource
        let resource = context.get_resource(&self.resource())?;
        
        // Get the resource logic
        let logic = context.get_resource_logic(&resource.resource_type)?;
        
        // Validate using resource logic
        logic.validate_operation(&resource, self.to_operation())?;
        
        Ok(())
    }
}

/// Constraints for resource creation effects
pub trait ResourceCreationEffect: ResourceEffect {
    /// The initial resource state
    fn initial_state(&self) -> RegisterState;
    
    /// The resource type
    fn resource_type(&self) -> ResourceType;
    
    /// The initial attributes
    fn attributes(&self) -> HashMap<String, Value>;
    
    /// Validate the creation
    fn validate_creation(&self, context: &ValidationContext) -> Result<(), ValidationError> {
        // Get the resource logic for this type
        let logic = context.get_resource_logic(&self.resource_type())?;
        
        // Create a temporary resource for validation
        let temp_resource = ResourceRegister::new(
            self.resource(),
            self.initial_state(),
            self.attributes(),
        );
        
        // Validate using resource logic
        logic.validate_operation(&temp_resource, self.to_operation())?;
        
        Ok(())
    }
}
```

### 3. Domain Implementation Layer (Third Layer - TEL)

```rust
effect EthereumResourceTransfer implements ResourceTransformationEffect {
    // State fields
    resource_id: RegisterId
    from_account: Address
    to_account: Address
    quantity: u64
    domain: DomainId
    
    // Implementation of required accessors
    fn resource() -> RegisterId { return this.resource_id; }
    fn domains() -> Vec<DomainId> { return [this.domain]; }
    fn source_state() -> RegisterState { return RegisterState::Active; }
    fn target_state() -> RegisterState { return RegisterState::Transferred; }
    
    // Domain-specific validation
    fn validate_ethereum_transfer(context) -> Result<(), ValidationError> {
        // Check gas price
        let current_gas = context.observe("ethereum.gas_price");
        require(current_gas < 100, "Gas price too high");
        
        // Check resource ownership
        let resource = context.get_resource(this.resource_id);
        require(resource.owner == this.from_account, "Not the resource owner");
        
        return Ok(());
    }
    
    // Execution logic
    fn execute(context) -> Result<TransactionHash, EffectError> {
        // Get resource logic
        let logic = context.get_resource_logic(context.get_resource(this.resource_id).resource_type);
        
        // Create operation
        let operation = Operation::new(OperationType::Transfer)
            .with_input(ContentRef::new(context.get_resource(this.resource_id)))
            .with_parameters({"from": this.from_account, "to": this.to_account, "quantity": this.quantity})
            .with_context(EthereumContext::new(this.domain));
        
        // Apply the operation using resource logic
        let result = logic.apply_operation(context.get_resource_mut(this.resource_id), &operation)?;
        
        // Return the transaction hash
        return Ok(TransactionHash(result.transaction_hash));
    }
}
```

## Unified Operation Model Integration

ResourceRegister logic integrates with the unified operation model:

```rust
/// Execute an operation on a ResourceRegister using appropriate resource logic
pub async fn execute_resource_operation<C: ExecutionContext>(
    operation: Operation<C>,
    logic_registry: &ResourceLogicRegistry,
    execution_service: &ExecutionService,
) -> Result<OperationResult, ExecutionError> {
    // Extract resource from operation
    let resource_ref = operation.inputs.first()
        .ok_or(ExecutionError::InvalidOperation("Operation has no inputs"))?;
    
    // Get the resource
    let mut resource = resource_ref.resolve(execution_service.storage())?;
    
    // Get the appropriate logic module
    let logic = logic_registry.get_module(&resource.resource_type)?;
    
    // Validate the operation before execution
    logic.validate_operation(&resource, &operation)?;
    
    // Apply the operation
    let effects = logic.apply_operation(&mut resource, &operation)?;
    
    // Verify conservation rules
    if let Some(resource_delta) = ResourceDelta::from_operation(&operation) {
        let resources = get_affected_resources(&operation, execution_service)?;
        logic.check_conservation(&resources, &resource_delta)?;
    }
    
    // Execute the effects
    let effect_results = execution_service.execute_effects(&effects, &operation.context).await?;
    
    // Store the updated resource
    let new_content_hash = execution_service.storage().store(&resource)?;
    
    // Create the result
    let result = OperationResult {
        operation_id: operation.id.clone(),
        success: true,
        new_content_hash,
        effect_results,
        output_resources: vec![ContentRef::new(&resource)],
        metadata: HashMap::new(),
    };
    
    Ok(result)
}
```

## Content-Addressed Logic Implementation

Causality provides content-addressed logic implementations for common resource types:

```rust
/// Standard logic for data resources
#[derive(Serialize, Deserialize)]
pub struct DataResourceLogic {
    validators: Vec<ContentRef<Box<dyn Validator>>>,
    transformers: Vec<ContentRef<Box<dyn Transformer>>>,
    content_hash: ContentHash,
}

impl ContentAddressed for DataResourceLogic {
    fn content_hash(&self) -> ContentHash {
        self.content_hash.clone()
    }
    
    fn verify(&self) -> bool {
        let calculated_hash = calculate_content_hash(self).unwrap();
        self.content_hash == calculated_hash
    }
}

impl ResourceLogic for DataResourceLogic {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Data
    }
    
    fn validate_operation(&self, resource: &ResourceRegister, operation: &Operation<C>) -> Result<(), LogicError> {
        // Resolve validators from content store
        let validators = resolve_validators(&self.validators, operation.context.storage())?;
        
        // Run all validators
        for validator in &validators {
            validator.validate(resource, operation)?;
        }
        
        Ok(())
    }
    
    fn apply_operation(&self, resource: &mut ResourceRegister, operation: &Operation<C>) -> Result<Vec<Effect>, LogicError> {
        match operation.op_type {
            OperationType::Create => self.create_data(resource, operation),
            OperationType::Read => self.read_data(resource, operation),
            OperationType::Update => self.update_data(resource, operation),
            OperationType::Delete => self.delete_data(resource, operation),
            _ => Err(LogicError::UnsupportedOperation(operation.op_type.clone())),
        }
    }
    
    fn check_conservation(&self, resources: &[ResourceRegister], delta: &ResourceDelta) -> Result<(), ConservationError> {
        // For data resources, check that the count is conserved
        let before_count = resources.iter()
            .filter(|r| r.state == RegisterState::Active)
            .count();
            
        let after_count = before_count + delta.created_count - delta.deleted_count;
        
        if delta.expected_final_count != after_count {
            return Err(ConservationError::CountMismatch {
                expected: delta.expected_final_count,
                actual: after_count,
            });
        }
        
        Ok(())
    }
    
    // Other required methods implementation...
}
```

### Custom Content-Addressed Logic

Custom logic with content addressing for business requirements:

```rust
/// Custom logic for a financial transaction resource
#[derive(Serialize, Deserialize)]
pub struct FinancialTransactionLogic {
    account_service_ref: ContentRef<dyn AccountService>,
    transaction_rules: Vec<ContentRef<Box<dyn TransactionRule>>>,
    notification_service_ref: ContentRef<dyn NotificationService>,
    content_hash: ContentHash,
}

impl ContentAddressed for FinancialTransactionLogic {
    fn content_hash(&self) -> ContentHash {
        self.content_hash.clone()
    }
    
    fn verify(&self) -> bool {
        let calculated_hash = calculate_content_hash(self).unwrap();
        self.content_hash == calculated_hash
    }
}

impl ResourceLogic for FinancialTransactionLogic {
    fn resource_type(&self) -> ResourceType {
        ResourceType::FinancialTransaction
    }
    
    fn validate_operation(&self, resource: &ResourceRegister, operation: &Operation<C>) -> Result<(), LogicError> {
        // Basic operation validation
        if operation.op_type != OperationType::Create && operation.op_type != OperationType::Read {
            return Err(LogicError::InvalidOperation(
                "FinancialTransactions can only be created or read, not modified or deleted".to_string()
            ));
        }
        
        // Resolve transaction rules from content store
        let rules = resolve_transaction_rules(&self.transaction_rules, operation.context.storage())?;
        
        // Validate against transaction rules
        for rule in &rules {
            rule.validate(resource, operation)?;
        }
        
        Ok(())
    }
    
    fn apply_operation(&self, resource: &mut ResourceRegister, operation: &Operation<C>) -> Result<Vec<Effect>, LogicError> {
        match operation.op_type {
            OperationType::Create => self.create_transaction(resource, operation),
            OperationType::Read => self.read_transaction(resource, operation),
            _ => Err(LogicError::UnsupportedOperation(operation.op_type.clone())),
        }
    }
    
    // Other required methods implementation...
}

```

## Logic Composition with Content Addressing

ResourceRegister logic components use content addressing for immutability and verification:

```rust
/// Validates required fields with content addressing
#[derive(Serialize, Deserialize)]
pub struct RequiredFieldValidator {
    required_fields: Vec<String>,
    content_hash: ContentHash,
}

impl ContentAddressed for RequiredFieldValidator {
    fn content_hash(&self) -> ContentHash {
        self.content_hash.clone()
    }
    
    fn verify(&self) -> bool {
        let calculated_hash = calculate_content_hash(self).unwrap();
        self.content_hash == calculated_hash
    }
}

impl Validator for RequiredFieldValidator {
    fn validate(&self, resource: &ResourceRegister, operation: &Operation<C>) -> Result<(), ValidationError> {
        for field in &self.required_fields {
            if !operation.parameters.contains_key(field) {
                return Err(ValidationError::MissingField(field.clone()));
            }
        }
        
        Ok(())
    }
}

/// Composite validator with content addressing
#[derive(Serialize, Deserialize)]
pub struct CompositeValidator {
    validators: Vec<ContentRef<Box<dyn Validator>>>,
    content_hash: ContentHash,
}

impl ContentAddressed for CompositeValidator {
    fn content_hash(&self) -> ContentHash {
        self.content_hash.clone()
    }
    
    fn verify(&self) -> bool {
        let calculated_hash = calculate_content_hash(self).unwrap();
        self.content_hash == calculated_hash
    }
}

impl Validator for CompositeValidator {
    fn validate(&self, resource: &ResourceRegister, operation: &Operation<C>) -> Result<(), ValidationError> {
        // Resolve validators from content store
        let validators = resolve_validators(&self.validators, operation.context.storage())?;
        
        // Run all validators
        for validator in &validators {
            validator.validate(resource, operation)?;
        }
        
        Ok(())
    }
}
```

## Logic Verification through Unified Verification Framework

ResourceRegister logic integrates with the unified verification framework:

```rust
impl Verifiable for ResourceLogicExecution<C> {
    type Proof = UnifiedProof;
    type Subject = LogicExecutionValidity;
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Get the resource logic
        let logic = context.get_resource_logic(&self.resource.resource_type)?;
        
        // Generate logical proof for resource logic execution
        let logical_proof = generate_resource_logic_proof(self, logic, context)?;
        
        // Generate conservation proof if needed
        let conservation_proof = if let Some(delta) = &self.resource_delta {
            Some(generate_conservation_proof(self, logic, delta, context)?)
        } else {
            None
        };
        
        // Create unified proof
        let proof = UnifiedProof {
            logical_components: Some(logical_proof),
            conservation_components: conservation_proof,
            // Other components as needed
            ..Default::default()
        };
        
        Ok(proof)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Verify logical execution
        let logical_valid = if let Some(logical_proof) = &proof.logical_components {
            verify_resource_logic_execution(self, logical_proof, context)?
        } else {
            return Err(VerificationError::MissingProofComponent("logical_components"));
        };
        
        // Verify conservation if needed
        let conservation_valid = if let (Some(delta), Some(conservation_proof)) = (&self.resource_delta, &proof.conservation_components) {
            verify_conservation(self, conservation_proof, delta, context)?
        } else if self.resource_delta.is_some() {
            return Err(VerificationError::MissingProofComponent("conservation_components"));
        } else {
            true
        };
        
        Ok(logical_valid && conservation_valid)
    }
}
```

## Capability-Based Logic Access

Access to ResourceRegister logic is controlled through capabilities:

```rust
/// Capability for access to resource logic
pub struct ResourceLogicCapability {
    /// The capability ID
    pub id: CapabilityId,
    
    /// The rights this capability grants
    pub rights: HashSet<Right>,
    
    /// Resource logic types this capability applies to
    pub logic_types: HashSet<ResourceType>,
    
    /// Constraints on using this capability
    pub constraints: CapabilityConstraints,
    
    /// How this capability can be delegated
    pub delegation_rules: DelegationRules,
}

/// Validates access to resource logic
pub fn validate_logic_access(
    entity: &EntityId,
    resource_type: &ResourceType,
    operation: &Operation<C>,
    capabilities: &[Capability],
) -> Result<bool, AuthError> {
    // Extract resource logic capabilities
    let logic_capabilities = capabilities.iter()
        .filter_map(|cap| {
            if let Capability::ResourceLogic(logic_cap) = cap {
                Some(logic_cap)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    
    // Check if any capability grants access to this resource type
    for cap in &logic_capabilities {
        if cap.logic_types.contains(resource_type) && 
           cap.rights.contains(&right_for_operation(operation.op_type)) {
            return Ok(true);
        }
    }
    
    Ok(false)
}
```

## Cross-Domain Logic with Unified Operation Model

ResourceRegister logic for cross-domain operations uses the unified operation model:

```rust
/// Execute a cross-domain resource operation
pub async fn execute_cross_domain_resource_operation<C: ExecutionContext>(
    operation: Operation<CrossDomainContext>,
    logic_registry: &ResourceLogicRegistry,
    execution_service: &ExecutionService,
) -> Result<OperationResult, ExecutionError> {
    // Extract source and target domains from context
    let source_domain = &operation.context.source_domain;
    let target_domain = &operation.context.target_domain;
    
    // Validate cross-domain capabilities
    validate_cross_domain_capabilities(
        &operation.context.entity_id,
        source_domain,
        target_domain,
        &operation.op_type,
        &operation.context.capabilities,
    )?;
    
    // Create source domain operation
    let source_operation = operation.refine_to::<DomainContext>()?
        .with_context(DomainContext::new(source_domain.clone()));
    
    // Execute source operation
    let source_result = execute_resource_operation(
        source_operation,
        logic_registry,
        execution_service,
    ).await?;
    
    // Create target domain operation
    let target_operation = operation.refine_to::<DomainContext>()?
        .with_context(DomainContext::new(target_domain.clone()));
    
    // Execute target operation
    let target_result = execute_resource_operation(
        target_operation,
        logic_registry,
        execution_service,
    ).await?;
    
    // Create cross-domain relationship
    let relationship_operation = Operation::new(OperationType::CreateRelationship)
        .with_input(source_result.output_resources[0].clone())
        .with_output(target_result.output_resources[0].clone())
        .with_parameter("type", RelationshipType::CrossDomain)
        .with_parameter("direction", RelationshipDirection::Bidirectional)
        .with_context(CrossDomainContext::new(source_domain.clone(), target_domain.clone()));
    
    let relationship_result = execution_service.execute_operation(relationship_operation).await?;
    
    // Combine results
    let cross_domain_result = OperationResult {
        operation_id: operation.id.clone(),
        success: source_result.success && target_result.success,
        new_content_hash: relationship_result.new_content_hash,
        effect_results: [
            source_result.effect_results,
            target_result.effect_results,
            relationship_result.effect_results,
        ].concat(),
        output_resources: [
            source_result.output_resources,
            target_result.output_resources,
        ].concat(),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("source_domain".to_string(), source_domain.id.clone().into());
            metadata.insert("target_domain".to_string(), target_domain.id.clone().into());
            metadata.insert("relationship_id".to_string(), 
                relationship_result.output_resources[0].hash.to_string().into());
            metadata
        },
    };
    
    Ok(cross_domain_result)
}
```

## Content-Addressed Logic Migration

Support for evolving ResourceRegister logic with content addressing:

```rust
/// Migrates resource logic to a new version with content addressing
pub async fn migrate_resource_logic(
    resource_type: &ResourceType,
    from_version: &ContentHash,
    to_version: &ContentHash,
    migration_context: &MigrationContext,
) -> Result<MigrationOutcome, MigrationError> {
    // Get the existing logic
    let existing_logic = migration_context.storage.get::<Box<dyn ResourceLogic>>(from_version)?;
    
    // Get the new logic
    let new_logic = migration_context.storage.get::<Box<dyn ResourceLogic>>(to_version)?;
    
    // Validate compatibility
    validate_logic_compatibility(&existing_logic, &new_logic)?;
    
    // Get all resources using this logic
    let affected_resources = find_resources_by_logic(resource_type, migration_context).await?;
    
    // Determine migration strategy based on compatibility level
    let compatibility = determine_compatibility_level(&existing_logic, &new_logic)?;
    let strategy = select_migration_strategy(compatibility, migration_context);
    
    // Execute migration
    let outcome = match strategy {
        MigrationStrategy::InPlace => {
            migrate_in_place(&affected_resources, &new_logic, migration_context).await?
        },
        MigrationStrategy::CopyAndTransform => {
            migrate_copy_and_transform(&affected_resources, &existing_logic, &new_logic, migration_context).await?
        },
        MigrationStrategy::Versioned => {
            migrate_versioned(&affected_resources, &existing_logic, &new_logic, migration_context).await?
        },
    };
    
    // Update logic registry with new version
    update_logic_registry(resource_type, to_version, migration_context).await?;
    
    Ok(outcome)
}
```

## Conclusion

The ResourceRegister logic system provides a robust framework for defining, executing, and verifying behavior associated with resources in Causality. Through integration with the unified ResourceRegister model, three-layer effect architecture, unified operation model, and universal content addressing, it ensures that resource behavior is consistent, verifiable, and immutable. The capability-based access model and content-addressed implementation provide strong security and integrity guarantees, while the compositional approach allows for flexible and extensible logic definitions that support diverse business requirements. 