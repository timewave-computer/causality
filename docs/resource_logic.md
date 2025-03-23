# Resource Logic

This document outlines the resource logic system in Causality, detailing how business logic is attached to resources, processed, and validated within the unified resource system.

## Overview

Resource logic encapsulates the behavior, constraints, and operations associated with resources. It provides mechanisms for defining, executing, and validating resource-specific behaviors while ensuring consistency with system-wide constraints and verification requirements.

## Logic Components

### Logic Modules

Resource logic is organized into modules that implement specific behaviors:

```rust
/// A logic module for a specific resource type
pub trait ResourceLogicModule {
    /// Resource type this module applies to
    fn resource_type(&self) -> ResourceType;
    
    /// Validates operations on resources of this type
    fn validate_operation(&self, operation: &Operation, context: &ValidationContext) -> Result<ValidationOutcome>;
    
    /// Executes operations on resources of this type
    fn execute_operation(&self, operation: &Operation, context: &ExecutionContext) -> Result<ExecutionOutcome>;
    
    /// Computes effects of operations on resources of this type
    fn compute_effects(&self, operation: &Operation, context: &EffectContext) -> Result<Vec<Effect>>;
}
```

### Logic Registry

The logic registry manages logic modules for different resource types:

```rust
/// Registry for resource logic modules
pub struct ResourceLogicRegistry {
    /// Modules indexed by resource type
    modules: HashMap<ResourceType, Box<dyn ResourceLogicModule>>,
    
    /// Fallback module for unknown resource types
    fallback_module: Option<Box<dyn ResourceLogicModule>>,
}

impl ResourceLogicRegistry {
    /// Registers a new logic module
    pub fn register_module(&mut self, module: Box<dyn ResourceLogicModule>) -> Result<()> {
        let resource_type = module.resource_type();
        
        if self.modules.contains_key(&resource_type) {
            return Err(Error::ModuleAlreadyRegistered(resource_type));
        }
        
        self.modules.insert(resource_type, module);
        Ok(())
    }
    
    /// Gets the module for a specific resource type
    pub fn get_module(&self, resource_type: &ResourceType) -> Result<&Box<dyn ResourceLogicModule>> {
        self.modules.get(resource_type).ok_or_else(|| {
            if let Some(fallback) = &self.fallback_module {
                Ok(fallback)
            } else {
                Err(Error::ModuleNotFound(resource_type.clone()))
            }
        })
    }
}
```

## Logic Execution

### Operation Execution

Resource logic executes operations on resources:

```rust
/// Executes an operation using appropriate resource logic
pub fn execute_resource_operation(
    operation: &Operation,
    resource: &ResourceRegister,
    logic_registry: &ResourceLogicRegistry,
    execution_context: &mut ExecutionContext,
) -> Result<ExecutionOutcome> {
    // Get the appropriate logic module
    let module = logic_registry.get_module(&resource.resource_type)?;
    
    // Validate the operation before execution
    let validation_outcome = module.validate_operation(operation, &execution_context.to_validation_context())?;
    
    if !validation_outcome.is_valid() {
        return Err(Error::ValidationFailed(validation_outcome.errors));
    }
    
    // Execute the operation
    let execution_outcome = module.execute_operation(operation, execution_context)?;
    
    // Record the execution outcome in the context
    execution_context.record_outcome(execution_outcome.clone());
    
    Ok(execution_outcome)
}
```

### Effect Computation

Resource logic computes effects from operations:

```rust
/// Computes effects for an operation
pub fn compute_operation_effects(
    operation: &Operation,
    resource: &ResourceRegister,
    logic_registry: &ResourceLogicRegistry,
    effect_context: &EffectContext,
) -> Result<Vec<Effect>> {
    // Get the appropriate logic module
    let module = logic_registry.get_module(&resource.resource_type)?;
    
    // Compute effects
    module.compute_effects(operation, effect_context)
}
```

## Logic Implementation

### Standard Resource Logic

Causality provides standard logic implementations for common resource types:

```rust
/// Standard logic for data resources
pub struct DataResourceLogic {
    validators: Vec<Box<dyn Validator>>,
    transformers: Vec<Box<dyn Transformer>>,
}

impl ResourceLogicModule for DataResourceLogic {
    fn resource_type(&self) -> ResourceType {
        ResourceType::Data
    }
    
    fn validate_operation(&self, operation: &Operation, context: &ValidationContext) -> Result<ValidationOutcome> {
        let mut outcome = ValidationOutcome::default();
        
        for validator in &self.validators {
            let result = validator.validate(operation, context)?;
            outcome.merge(result);
            
            if outcome.has_blocking_errors() {
                break;
            }
        }
        
        Ok(outcome)
    }
    
    fn execute_operation(&self, operation: &Operation, context: &ExecutionContext) -> Result<ExecutionOutcome> {
        match operation.kind {
            OperationKind::Create => self.create_data(operation, context),
            OperationKind::Read => self.read_data(operation, context),
            OperationKind::Update => self.update_data(operation, context),
            OperationKind::Delete => self.delete_data(operation, context),
            _ => Err(Error::UnsupportedOperation(operation.kind)),
        }
    }
    
    fn compute_effects(&self, operation: &Operation, context: &EffectContext) -> Result<Vec<Effect>> {
        let mut effects = Vec::new();
        
        // Add standard effects based on operation type
        match operation.kind {
            OperationKind::Create => {
                effects.push(Effect::ResourceCreated {
                    resource_id: operation.resource_id.clone(),
                    attributes: operation.params.clone(),
                });
            },
            OperationKind::Update => {
                effects.push(Effect::ResourceUpdated {
                    resource_id: operation.resource_id.clone(),
                    updated_attributes: operation.params.clone(),
                });
            },
            OperationKind::Delete => {
                effects.push(Effect::ResourceDeleted {
                    resource_id: operation.resource_id.clone(),
                });
            },
            _ => {},
        }
        
        // Apply transformers to generate additional effects
        for transformer in &self.transformers {
            let additional_effects = transformer.transform(operation, context)?;
            effects.extend(additional_effects);
        }
        
        Ok(effects)
    }
}
```

### Custom Resource Logic

Custom logic can be implemented for specific business requirements:

```rust
/// Custom logic for a financial transaction resource
pub struct FinancialTransactionLogic {
    account_service: Arc<dyn AccountService>,
    transaction_rules: Vec<Box<dyn TransactionRule>>,
    notification_service: Arc<dyn NotificationService>,
}

impl ResourceLogicModule for FinancialTransactionLogic {
    fn resource_type(&self) -> ResourceType {
        ResourceType::FinancialTransaction
    }
    
    fn validate_operation(&self, operation: &Operation, context: &ValidationContext) -> Result<ValidationOutcome> {
        let mut outcome = ValidationOutcome::default();
        
        // Basic operation validation
        if operation.kind != OperationKind::Create && operation.kind != OperationKind::Read {
            outcome.add_error(ValidationError::new(
                "FinancialTransactions can only be created or read, not modified or deleted",
                ErrorSeverity::Blocking,
            ));
            return Ok(outcome);
        }
        
        // Validate against transaction rules
        for rule in &self.transaction_rules {
            let rule_result = rule.validate(operation, context)?;
            outcome.merge(rule_result);
            
            if outcome.has_blocking_errors() {
                break;
            }
        }
        
        Ok(outcome)
    }
    
    fn execute_operation(&self, operation: &Operation, context: &ExecutionContext) -> Result<ExecutionOutcome> {
        match operation.kind {
            OperationKind::Create => self.create_transaction(operation, context),
            OperationKind::Read => self.read_transaction(operation, context),
            _ => Err(Error::UnsupportedOperation(operation.kind)),
        }
    }
    
    fn compute_effects(&self, operation: &Operation, context: &EffectContext) -> Result<Vec<Effect>> {
        let mut effects = Vec::new();
        
        if operation.kind == OperationKind::Create {
            // Extract transaction parameters
            let amount = operation.params.get("amount")
                .ok_or_else(|| Error::MissingParameter("amount".to_string()))?
                .as_f64()
                .ok_or_else(|| Error::InvalidParameterType("amount".to_string()))?;
                
            let from_account = operation.params.get("from_account")
                .ok_or_else(|| Error::MissingParameter("from_account".to_string()))?
                .as_str()
                .ok_or_else(|| Error::InvalidParameterType("from_account".to_string()))?;
                
            let to_account = operation.params.get("to_account")
                .ok_or_else(|| Error::MissingParameter("to_account".to_string()))?
                .as_str()
                .ok_or_else(|| Error::InvalidParameterType("to_account".to_string()))?;
            
            // Add resource creation effect
            effects.push(Effect::ResourceCreated {
                resource_id: operation.resource_id.clone(),
                attributes: operation.params.clone(),
            });
            
            // Add account balance effects
            effects.push(Effect::AccountDebited {
                account_id: from_account.to_string(),
                amount,
                transaction_id: operation.resource_id.clone(),
            });
            
            effects.push(Effect::AccountCredited {
                account_id: to_account.to_string(),
                amount,
                transaction_id: operation.resource_id.clone(),
            });
            
            // Add notification effect
            effects.push(Effect::NotificationSent {
                recipient_id: from_account.to_string(),
                notification_type: "transaction_completed".to_string(),
                notification_params: json!({
                    "transaction_id": operation.resource_id.to_string(),
                    "amount": amount,
                    "to_account": to_account,
                }),
            });
        }
        
        Ok(effects)
    }
}
```

## Logic Composition

Resource logic can be composed from smaller, reusable components:

```rust
/// Validates required fields are present in an operation
pub struct RequiredFieldValidator {
    required_fields: Vec<String>,
}

impl Validator for RequiredFieldValidator {
    fn validate(&self, operation: &Operation, _context: &ValidationContext) -> Result<ValidationOutcome> {
        let mut outcome = ValidationOutcome::default();
        
        for field in &self.required_fields {
            if !operation.params.contains_key(field) {
                outcome.add_error(ValidationError::new(
                    &format!("Required field '{}' is missing", field),
                    ErrorSeverity::Blocking,
                ));
            }
        }
        
        Ok(outcome)
    }
}

/// Enforces field type constraints
pub struct FieldTypeValidator {
    field_types: HashMap<String, FieldType>,
}

impl Validator for FieldTypeValidator {
    fn validate(&self, operation: &Operation, _context: &ValidationContext) -> Result<ValidationOutcome> {
        let mut outcome = ValidationOutcome::default();
        
        for (field, expected_type) in &self.field_types {
            if let Some(value) = operation.params.get(field) {
                if !self.check_type(value, expected_type) {
                    outcome.add_error(ValidationError::new(
                        &format!("Field '{}' has incorrect type", field),
                        ErrorSeverity::Blocking,
                    ));
                }
            }
        }
        
        Ok(outcome)
    }
}

/// Composite validator combining multiple validators
pub struct CompositeValidator {
    validators: Vec<Box<dyn Validator>>,
}

impl Validator for CompositeValidator {
    fn validate(&self, operation: &Operation, context: &ValidationContext) -> Result<ValidationOutcome> {
        let mut outcome = ValidationOutcome::default();
        
        for validator in &self.validators {
            let result = validator.validate(operation, context)?;
            outcome.merge(result);
            
            if outcome.has_blocking_errors() {
                break;
            }
        }
        
        Ok(outcome)
    }
}
```

## Logic Extensibility

Causality provides extension points for logic customization:

### Logic Plugins

Logic plugins allow for dynamic extension of resource logic:

```rust
/// Plugin for extending resource logic
pub trait ResourceLogicPlugin {
    /// Initializes the plugin
    fn initialize(&mut self, registry: &mut ResourceLogicRegistry) -> Result<()>;
    
    /// Returns the plugin name
    fn name(&self) -> &str;
    
    /// Returns the plugin version
    fn version(&self) -> &str;
    
    /// Shuts down the plugin
    fn shutdown(&mut self) -> Result<()>;
}

/// Plugin manager for resource logic plugins
pub struct ResourceLogicPluginManager {
    plugins: Vec<Box<dyn ResourceLogicPlugin>>,
}

impl ResourceLogicPluginManager {
    /// Loads a plugin from a dynamic library
    pub fn load_plugin(&mut self, path: &Path, registry: &mut ResourceLogicRegistry) -> Result<()> {
        let plugin = load_plugin_from_path(path)?;
        plugin.initialize(registry)?;
        self.plugins.push(plugin);
        Ok(())
    }
}
```

### Logic Scripting

Scripting support for defining resource logic:

```rust
/// Script-based resource logic
pub struct ScriptedResourceLogic {
    resource_type: ResourceType,
    validation_script: Script,
    execution_script: Script,
    effect_script: Script,
    script_engine: Arc<dyn ScriptEngine>,
}

impl ResourceLogicModule for ScriptedResourceLogic {
    fn resource_type(&self) -> ResourceType {
        self.resource_type.clone()
    }
    
    fn validate_operation(&self, operation: &Operation, context: &ValidationContext) -> Result<ValidationOutcome> {
        let script_context = self.create_script_context(operation, context);
        
        let result = self.script_engine.execute_script(
            &self.validation_script,
            &script_context,
        )?;
        
        parse_validation_outcome(result)
    }
    
    fn execute_operation(&self, operation: &Operation, context: &ExecutionContext) -> Result<ExecutionOutcome> {
        let script_context = self.create_script_context(operation, context);
        
        let result = self.script_engine.execute_script(
            &self.execution_script,
            &script_context,
        )?;
        
        parse_execution_outcome(result)
    }
    
    fn compute_effects(&self, operation: &Operation, context: &EffectContext) -> Result<Vec<Effect>> {
        let script_context = self.create_script_context(operation, context);
        
        let result = self.script_engine.execute_script(
            &self.effect_script,
            &script_context,
        )?;
        
        parse_effects(result)
    }
}
```

## Logic Integration

### Integration with Resource Lifecycle

Resource logic is integrated with the resource lifecycle:

```rust
/// Resource lifecycle manager with logic integration
pub struct ResourceLifecycleManager {
    logic_registry: ResourceLogicRegistry,
    state_manager: ResourceStateManager,
}

impl ResourceLifecycleManager {
    /// Creates a new resource
    pub async fn create_resource(
        &self,
        create_params: &CreateResourceParams,
        context: &mut ExecutionContext,
    ) -> Result<ResourceId> {
        // Generate resource ID
        let resource_id = generate_resource_id(
            &create_params.resource_type,
            &context.domain_id,
            create_params.namespace.clone(),
        );
        
        // Create initial register state
        let register = ResourceRegister::new(
            resource_id.clone(),
            create_params.initial_state.clone(),
            create_params.attributes.clone(),
        );
        
        // Create operation
        let operation = Operation {
            id: OperationId::new(),
            kind: OperationKind::Create,
            resource_id: resource_id.clone(),
            params: create_params.attributes.clone(),
            metadata: create_params.metadata.clone(),
        };
        
        // Get logic module and validate
        let module = self.logic_registry.get_module(&create_params.resource_type)?;
        let validation_outcome = module.validate_operation(&operation, &context.to_validation_context())?;
        
        if !validation_outcome.is_valid() {
            return Err(Error::ValidationFailed(validation_outcome.errors));
        }
        
        // Execute logic
        let execution_outcome = module.execute_operation(&operation, context)?;
        
        // Compute effects
        let effect_context = EffectContext::from_execution_context(context);
        let effects = module.compute_effects(&operation, &effect_context)?;
        
        // Apply effects
        self.apply_effects(&effects, context).await?;
        
        // Save register state
        self.state_manager.save_register(&register).await?;
        
        Ok(resource_id)
    }
    
    /// Executes an operation on a resource
    pub async fn execute_resource_operation(
        &self,
        operation: &Operation,
        context: &mut ExecutionContext,
    ) -> Result<ExecutionOutcome> {
        // Load the resource register
        let register = self.state_manager.load_register(&operation.resource_id).await?;
        
        // Get logic module and validate
        let module = self.logic_registry.get_module(&register.resource_type)?;
        let validation_outcome = module.validate_operation(operation, &context.to_validation_context())?;
        
        if !validation_outcome.is_valid() {
            return Err(Error::ValidationFailed(validation_outcome.errors));
        }
        
        // Execute logic
        let execution_outcome = module.execute_operation(operation, context)?;
        
        // Compute effects
        let effect_context = EffectContext::from_execution_context(context);
        let effects = module.compute_effects(operation, &effect_context)?;
        
        // Apply effects
        self.apply_effects(&effects, context).await?;
        
        Ok(execution_outcome)
    }
}
```

### Integration with Effects System

Resource logic connects with the effects system:

```rust
/// Applies effects generated by resource logic
async fn apply_effects(
    effects: &[Effect],
    context: &mut ExecutionContext,
) -> Result<()> {
    let effect_manager = context.get_effect_manager()?;
    
    for effect in effects {
        effect_manager.enqueue_effect(effect.clone(), context.operation_id.clone())?;
    }
    
    // Process immediate effects
    let immediate_effects: Vec<_> = effects.iter()
        .filter(|e| e.is_immediate())
        .cloned()
        .collect();
        
    if !immediate_effects.is_empty() {
        effect_manager.process_immediate_effects(&immediate_effects, context).await?;
    }
    
    Ok(())
}
```

## Cross-Domain Logic

Resource logic supports cross-domain scenarios:

```rust
/// Executes resource logic across domain boundaries
pub async fn execute_cross_domain_operation(
    operation: &Operation,
    source_domain: &DomainId,
    target_domain: &DomainId,
    logic_registry: &ResourceLogicRegistry,
    context: &mut ExecutionContext,
) -> Result<ExecutionOutcome> {
    // Validate cross-domain capabilities
    validate_cross_domain_capabilities(operation, source_domain, target_domain, context)?;
    
    // Create cross-domain context
    let mut cross_domain_context = ExecutionContext::new_cross_domain(
        context.clone(),
        source_domain.clone(),
        target_domain.clone(),
    );
    
    // Load resource from target domain
    let target_domain_client = context.get_domain_client(target_domain)?;
    let resource = target_domain_client.get_resource(&operation.resource_id).await?;
    
    // Get appropriate logic module
    let module = logic_registry.get_module(&resource.resource_type)?;
    
    // Execute operation in cross-domain context
    let execution_outcome = module.execute_operation(operation, &mut cross_domain_context)?;
    
    // Synchronize contexts
    context.merge_cross_domain_context(cross_domain_context);
    
    Ok(execution_outcome)
}
```

## Logic Verification

Resource logic includes verification mechanisms:

```rust
/// Verifies execution of resource logic
pub fn verify_logic_execution(
    operation: &Operation,
    execution_outcome: &ExecutionOutcome,
    effects: &[Effect],
    verification_context: &VerificationContext,
) -> Result<VerificationOutcome> {
    // Get the resource type
    let resource_type = get_resource_type_from_operation(operation, verification_context)?;
    
    // Get the verification logic
    let verification_logic = verification_context.get_verification_logic(&resource_type)?;
    
    // Verify the execution outcome
    let outcome_verification = verification_logic.verify_execution_outcome(
        operation,
        execution_outcome,
        verification_context,
    )?;
    
    if !outcome_verification.is_valid() {
        return Ok(outcome_verification);
    }
    
    // Verify the effects
    let effects_verification = verification_logic.verify_effects(
        operation,
        effects,
        verification_context,
    )?;
    
    if !effects_verification.is_valid() {
        return Ok(effects_verification);
    }
    
    // Combined verification is valid
    Ok(VerificationOutcome::valid())
}
```

## Logic Migration

Support for evolving resource logic over time:

```rust
/// Migrates resource logic to a new version
pub async fn migrate_resource_logic(
    resource_type: &ResourceType,
    from_version: &Version,
    to_version: &Version,
    migration_context: &MigrationContext,
) -> Result<MigrationOutcome> {
    // Get the migration strategy
    let migration_strategy = migration_context.get_migration_strategy(resource_type, from_version, to_version)?;
    
    // Execute migration
    let outcome = match migration_strategy {
        MigrationStrategy::InPlace => {
            migrate_in_place(resource_type, from_version, to_version, migration_context).await?
        },
        MigrationStrategy::CopyAndTransform => {
            migrate_copy_and_transform(resource_type, from_version, to_version, migration_context).await?
        },
        MigrationStrategy::Versioned => {
            migrate_versioned(resource_type, from_version, to_version, migration_context).await?
        },
    };
    
    // Update logic registry with new version
    update_logic_registry(resource_type, to_version, migration_context).await?;
    
    Ok(outcome)
}
```

## Conclusion

The resource logic system provides a flexible and extensible framework for defining, executing, and verifying behavior associated with resources in Causality. Through integration with the resource lifecycle, effects system, and cross-domain operations, it ensures that resource behavior is consistent, verifiable, and capable of evolving over time. The composition-based approach and extension mechanisms allow for a wide range of logic implementations, from simple data validation to complex business processes spanning multiple domains. 