<!-- Pipeline for validation -->
<!-- Original file: docs/src/validation_pipeline.md -->

# Validation Pipeline Architecture

## Overview

The Causality Validation Pipeline provides a comprehensive framework for validating operations, resources, and transactions across domains. The pipeline ensures that all changes to system state maintain consistency, respect constraints, and adhere to security policies.

```
┌───────────────────────────────────────────────────────────────┐
│                 Validation Pipeline Architecture              │
│                                                               │
│  ┌───────────┐   ┌───────────┐   ┌───────────┐   ┌───────────┐│
│  │  Input    │   │  Semantic │   │ Capability│   │ Temporal  ││
│  │Validation ├──►│Validation ├──►│Validation ├──►│Validation ││
│  └───────────┘   └───────────┘   └───────────┘   └───────────┘│
│        │               │               │               │      │
│        ▼               ▼               ▼               ▼      │
│  ┌───────────┐   ┌───────────┐   ┌───────────┐   ┌───────────┐│
│  │ Resource  │   │Relationship   │ Domain    │   │  Custom   ││
│  │Validation ├──►│Validation ├──►│Validation ├──►│Validation ││
│  └───────────┘   └───────────┘   └───────────┘   └───────────┘│
│        │               │               │               │      │
│        └───────────────┼───────────────┼───────────────┘      │
│                        ▼               ▼                      │
│                ┌───────────────┐ ┌───────────────┐            │
│                │ Validation    │ │ Validation    │            │
│                │ Results       │ │ Context       │            │
│                └───────────────┘ └───────────────┘            │
└───────────────────────────────────────────────────────────────┘
```

## Core Concepts

### Validation Stages

The validation pipeline is organized into sequential stages, each focusing on a specific aspect of validation:

1. **Input Validation**: Verifies that operation inputs are well-formed
2. **Semantic Validation**: Ensures operations conform to their semantic definitions
3. **Capability Validation**: Checks that operations have necessary capabilities
4. **Temporal Validation**: Validates temporal consistency
5. **Resource Validation**: Verifies resource state transitions
6. **Relationship Validation**: Validates relationships between resources
7. **Domain Validation**: Applies domain-specific validation rules
8. **Custom Validation**: Extensible hooks for custom validation logic

### Validation Pipeline Structure

```rust
pub struct ValidationPipeline {
    stages: Vec<Box<dyn ValidationStage>>,
    context_provider: Box<dyn ValidationContextProvider>,
    result_aggregator: Box<dyn ValidationResultAggregator>,
}

pub trait ValidationStage: Send + Sync {
    fn name(&self) -> &'static str;
    fn validate(&self, 
                item: &dyn Validatable, 
                context: &ValidationContext) -> ValidationResult;
    fn priority(&self) -> u32;
}
```

### Validatable Items

```rust
pub trait Validatable {
    fn validation_type(&self) -> ValidationType;
    fn validation_id(&self) -> ValidationId;
    fn metadata(&self) -> &ValidationMetadata;
}

impl Validatable for ResourceOperation {
    fn validation_type(&self) -> ValidationType {
        ValidationType::Operation
    }
    
    fn validation_id(&self) -> ValidationId {
        ValidationId::Operation(self.id)
    }
    
    fn metadata(&self) -> &ValidationMetadata {
        &self.validation_metadata
    }
}
```

### Validation Results

```rust
pub struct ValidationResult {
    is_valid: bool,
    stage_name: String,
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
    metadata: ValidationMetadata,
}

pub struct ValidationError {
    code: ValidationErrorCode,
    message: String,
    path: Option<String>,
    severity: ErrorSeverity,
    context: HashMap<String, String>,
}
```

## Validation Stages in Detail

### Input Validation Stage

The Input Validation Stage verifies that all operation inputs are well-formed and meet basic requirements.

```rust
pub struct InputValidationStage {
    validators: HashMap<OperationType, Box<dyn InputValidator>>,
}

impl ValidationStage for InputValidationStage {
    fn validate(&self, 
                item: &dyn Validatable, 
                context: &ValidationContext) -> ValidationResult {
        if let Some(op) = item.as_operation() {
            if let Some(validator) = self.validators.get(&op.operation_type) {
                return validator.validate_input(op, context);
            }
        }
        
        // Default validation if no specific validator found
        // ...
    }
}
```

Key functions:
- Type checking for operation parameters
- Format validation for identifiers and references
- Size limits for data fields
- Schema validation for structured data

### Semantic Validation Stage

The Semantic Validation Stage ensures that operations conform to their semantic definitions.

```rust
pub struct SemanticValidationStage {
    semantic_validators: HashMap<ResourceType, Box<dyn SemanticValidator>>,
}

impl ValidationStage for SemanticValidationStage {
    fn validate(&self, 
                item: &dyn Validatable, 
                context: &ValidationContext) -> ValidationResult {
        if let Some(op) = item.as_operation() {
            // Get resource type from operation
            let resource_type = context.get_resource_type(op.resource_id)?;
            
            // Apply semantic validator for this resource type
            if let Some(validator) = self.semantic_validators.get(&resource_type) {
                return validator.validate_semantics(op, context);
            }
        }
        
        // Default validation if no specific validator found
        // ...
    }
}
```

Key functions:
- Operation-specific validation rules
- State transition validation
- Invariant checking
- Business logic validation

### Capability Validation Stage

The Capability Validation Stage verifies that operations have the necessary capabilities.

```rust
pub struct CapabilityValidationStage {
    capability_verifier: CapabilityVerifier,
}

impl ValidationStage for CapabilityValidationStage {
    fn validate(&self, 
                item: &dyn Validatable, 
                context: &ValidationContext) -> ValidationResult {
        if let Some(op) = item.as_operation() {
            // Extract auth context from operation
            let auth_context = op.auth_context();
            
            // Verify capabilities against operation
            return self.capability_verifier.verify(op, auth_context, context);
        }
        
        // Default validation for non-operations
        // ...
    }
}
```

Key functions:
- Capability resolution and verification
- Permission checking
- Constraint validation
- Delegation chain verification

### Temporal Validation Stage

The Temporal Validation Stage ensures temporal consistency of operations.

```rust
pub struct TemporalValidationStage {
    temporal_validator: TemporalValidator,
}

impl ValidationStage for TemporalValidationStage {
    fn validate(&self, 
                item: &dyn Validatable, 
                context: &ValidationContext) -> ValidationResult {
        if let Some(op) = item.as_operation() {
            // Extract temporal context from operation
            let temporal_context = op.temporal_context();
            
            // Verify temporal consistency
            return self.temporal_validator.validate_consistency(op, temporal_context, context);
        }
        
        // Default validation for non-operations
        // ...
    }
}
```

Key functions:
- Causal dependency verification
- Temporal order validation
- Clock drift checking
- Fact existence validation

### Resource Validation Stage

The Resource Validation Stage validates resource state transitions.

```rust
pub struct ResourceValidationStage {
    resource_validators: HashMap<ResourceType, Box<dyn ResourceValidator>>,
}

impl ValidationStage for ResourceValidationStage {
    fn validate(&self, 
                item: &dyn Validatable, 
                context: &ValidationContext) -> ValidationResult {
        if let Some(op) = item.as_operation() {
            // Get resource type and current state
            let resource_type = context.get_resource_type(op.resource_id)?;
            let current_state = context.get_resource_state(op.resource_id)?;
            
            // Apply resource validator for this type
            if let Some(validator) = self.resource_validators.get(&resource_type) {
                return validator.validate_transition(op, current_state, context);
            }
        }
        
        // Default validation if no specific validator found
        // ...
    }
}
```

Key functions:
- State transition validation
- Resource lifecycle checks
- Resource integrity validation
- Schema compliance checking

### Relationship Validation Stage

The Relationship Validation Stage validates relationships between resources.

```rust
pub struct RelationshipValidationStage {
    relationship_validator: RelationshipValidator,
}

impl ValidationStage for RelationshipValidationStage {
    fn validate(&self, 
                item: &dyn Validatable, 
                context: &ValidationContext) -> ValidationResult {
        if let Some(op) = item.as_operation() {
            // Get related resources
            let related_resources = context.get_related_resources(op.resource_id)?;
            
            // Validate relationships
            return self.relationship_validator.validate_relationships(
                op, 
                related_resources, 
                context
            );
        }
        
        // Default validation for non-operations
        // ...
    }
}
```

Key functions:
- Relationship integrity checks
- Reference validation
- Ownership verification
- Hierarchical relationship validation

### Domain Validation Stage

The Domain Validation Stage applies domain-specific validation rules.

```rust
pub struct DomainValidationStage {
    domain_validators: HashMap<DomainId, Box<dyn DomainValidator>>,
}

impl ValidationStage for DomainValidationStage {
    fn validate(&self, 
                item: &dyn Validatable, 
                context: &ValidationContext) -> ValidationResult {
        if let Some(op) = item.as_operation() {
            // Get domain for this operation
            let domain_id = context.get_resource_domain(op.resource_id)?;
            
            // Apply domain validator
            if let Some(validator) = self.domain_validators.get(&domain_id) {
                return validator.validate_for_domain(op, context);
            }
        }
        
        // Default validation if no domain validator found
        // ...
    }
}
```

Key functions:
- Domain policy enforcement
- Domain-specific rules
- Cross-domain validation
- Domain boundary checks

### Custom Validation Stage

The Custom Validation Stage provides extensibility for custom validation logic.

```rust
pub struct CustomValidationStage {
    custom_validators: Vec<Box<dyn CustomValidator>>,
}

impl ValidationStage for CustomValidationStage {
    fn validate(&self, 
                item: &dyn Validatable, 
                context: &ValidationContext) -> ValidationResult {
        // Apply all registered custom validators
        let mut result = ValidationResult::new_valid("custom");
        
        for validator in &self.custom_validators {
            let validator_result = validator.validate(item, context);
            result.merge(validator_result);
            
            // Short-circuit on critical errors
            if result.has_critical_errors() {
                break;
            }
        }
        
        result
    }
}
```

Key functions:
- Application-specific validation
- Business rule validation
- Integration-specific validation
- Policy enforcement

## Validation Context

The Validation Context provides access to all information needed for validation.

```rust
pub struct ValidationContext {
    resource_accessor: Box<dyn ResourceAccessor>,
    capability_accessor: Box<dyn CapabilityAccessor>,
    fact_accessor: Box<dyn FactAccessor>,
    domain_accessor: Box<dyn DomainAccessor>,
    schema_registry: Box<dyn SchemaRegistry>,
    parameters: HashMap<String, Value>,
}

impl ValidationContext {
    // Resource state access
    pub fn get_resource(&self, id: ResourceId) -> Result<Option<Resource>>;
    
    // Capability access
    pub fn get_capabilities(&self, principal_id: PrincipalId) -> Result<Vec<Capability>>;
    
    // Temporal fact access
    pub fn get_facts(&self, resource_id: ResourceId) -> Result<Vec<TemporalFact>>;
    
    // Domain access
    pub fn get_resource_domain(&self, resource_id: ResourceId) -> Result<DomainId>;
    
    // Schema access
    pub fn get_resource_schema(&self, resource_type: ResourceType) -> Result<Option<Schema>>;
    
    // Parameter access
    pub fn get_parameter<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
}
```

## Result Aggregation

The Result Aggregator combines results from all validation stages.

```rust
pub struct ValidationResultAggregator {
    error_severity_threshold: ErrorSeverity,
}

impl ValidationResultAggregator {
    pub fn aggregate(&self, results: Vec<ValidationResult>) -> AggregatedValidationResult {
        let mut aggregated = AggregatedValidationResult::new();
        
        for result in results {
            aggregated.add_stage_result(result);
            
            // Short-circuit on critical errors
            if aggregated.has_error_above_threshold(self.error_severity_threshold) {
                break;
            }
        }
        
        aggregated
    }
}

pub struct AggregatedValidationResult {
    is_valid: bool,
    stage_results: HashMap<String, ValidationResult>,
    error_count: usize,
    warning_count: usize,
}
```

## Usage Example

```rust
// Create the validation pipeline
let pipeline = ValidationPipeline::builder()
    .add_stage(InputValidationStage::new())
    .add_stage(SemanticValidationStage::new())
    .add_stage(CapabilityValidationStage::new())
    .add_stage(TemporalValidationStage::new())
    .add_stage(ResourceValidationStage::new())
    .add_stage(RelationshipValidationStage::new())
    .add_stage(DomainValidationStage::new())
    .add_stage(CustomValidationStage::new())
    .with_context_provider(DefaultContextProvider::new())
    .with_result_aggregator(DefaultResultAggregator::new())
    .build();

// Create the validation context
let context = ValidationContext::new()
    .with_resource_accessor(resource_service.accessor())
    .with_capability_accessor(capability_service.accessor())
    .with_fact_accessor(temporal_service.accessor())
    .with_domain_accessor(domain_service.accessor())
    .with_schema_registry(registry_service.schema_registry())
    .with_parameter("allow_legacy", Value::Bool(true));

// Validate an operation
let operation = ResourceOperation::create_resource(/* params */);
let result = pipeline.validate(&operation, &context);

if result.is_valid {
    // Proceed with operation execution
} else {
    // Handle validation errors
    for error in result.errors() {
        log::error!("Validation error: {} (code: {})", error.message, error.code);
    }
}
```

## Validation for Different Items

The validation pipeline supports different types of validatable items:

### Operation Validation

```rust
let operation = ResourceOperation::new(/* params */);
let result = pipeline.validate(&operation, &context);
```

### Transaction Validation

```rust
let transaction = Transaction::new(/* params */);
let result = pipeline.validate(&transaction, &context);
```

### Resource Validation

```rust
let resource = Resource::new(/* params */);
let result = pipeline.validate(&resource, &context);
```

## Transaction Validation

Transactions require special validation that considers all operations as a unit:

```rust
pub struct TransactionValidator {
    pipeline: ValidationPipeline,
    transaction_stages: Vec<Box<dyn TransactionValidationStage>>,
}

impl TransactionValidator {
    pub fn validate_transaction(
        &self,
        transaction: &Transaction,
        context: &ValidationContext
    ) -> TransactionValidationResult {
        // First validate each operation individually
        let mut operation_results = Vec::new();
        for operation in &transaction.operations {
            let result = self.pipeline.validate(operation, context);
            operation_results.push(result);
        }
        
        // Then validate the transaction as a whole
        let mut transaction_result = TransactionValidationResult::new(transaction.id);
        
        for stage in &self.transaction_stages {
            let stage_result = stage.validate_transaction(transaction, &operation_results, context);
            transaction_result.add_stage_result(stage_result);
            
            // Short-circuit on critical errors
            if transaction_result.has_critical_errors() {
                break;
            }
        }
        
        transaction_result
    }
}
```

Transaction-specific validation includes:

1. **Cross-operation consistency**: Ensuring operations don't conflict
2. **Transaction atomicity**: Validating that all-or-nothing semantics can be maintained
3. **Transaction fees**: Validating fee calculations and payments
4. **Transaction size limits**: Enforcing limits on transaction size
5. **Operation ordering**: Validating the sequence of operations

## Cross-Domain Validation

For cross-domain operations, validation must span multiple domains:

```rust
pub struct CrossDomainValidationCoordinator {
    domain_validators: HashMap<DomainId, Box<dyn DomainValidator>>,
    projection_validator: Box<dyn CapabilityProjectionValidator>,
}

impl CrossDomainValidationCoordinator {
    pub fn validate_cross_domain_operation(
        &self,
        operation: &CrossDomainOperation,
        context: &ValidationContext
    ) -> CrossDomainValidationResult {
        // Validate in source domain
        let source_domain = operation.source_domain();
        let source_validator = self.domain_validators.get(&source_domain)?;
        let source_result = source_validator.validate_for_domain(operation, context);
        
        if !source_result.is_valid {
            return CrossDomainValidationResult::from_source_failure(source_result);
        }
        
        // Validate in each target domain
        let mut target_results = HashMap::new();
        for target_domain in operation.target_domains() {
            // Validate capability projection
            let projection_result = self.projection_validator.validate_projection(
                operation,
                source_domain,
                target_domain,
                context
            );
            
            if !projection_result.is_valid {
                target_results.insert(target_domain, projection_result);
                continue;
            }
            
            // Validate in target domain
            let target_validator = self.domain_validators.get(&target_domain)?;
            let target_result = target_validator.validate_for_domain(operation, context);
            target_results.insert(target_domain, target_result);
        }
        
        CrossDomainValidationResult::new(source_result, target_results)
    }
}
```

## Validation Registration and Extension

The validation system supports runtime registration of validators:

```rust
// Register a resource validator
validation_service.register_resource_validator(
    ResourceType::new("token"),
    Box::new(TokenValidator::new())
);

// Register a custom validator
validation_service.register_custom_validator(
    Box::new(ComplianceValidator::new())
);

// Register a domain validator
validation_service.register_domain_validator(
    DomainId::new("evm"),
    Box::new(EvmValidator::new())
);
```

## Error Handling and Reporting

Validation errors are structured for clear reporting:

```rust
pub struct ValidationError {
    code: ValidationErrorCode,
    message: String,
    path: Option<String>,
    severity: ErrorSeverity,
    context: HashMap<String, String>,
}

// Example of generating a validation error
fn validate_balance(account: &Account, amount: u64) -> Result<(), ValidationError> {
    if account.balance < amount {
        return Err(ValidationError::new(
            ValidationErrorCode::InsufficientFunds,
            format!("Insufficient funds: have {}, need {}", account.balance, amount),
            Some("account.balance"),
            ErrorSeverity::Error,
        )
        .with_context("account_id", account.id.to_string())
        .with_context("required_amount", amount.to_string()));
    }
    
    Ok(())
}
```

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Validation Pipeline | Complete | Core pipeline architecture |
| Input Validation | Complete | Basic type and format validation |
| Semantic Validation | Complete | Resource operation validation |
| Capability Validation | Complete | Authorization validation |
| Temporal Validation | Complete | Causal consistency validation |
| Resource Validation | Complete | Resource state validation |
| Relationship Validation | In Progress | Basic validation implemented |
| Domain Validation | In Progress | Framework in place |
| Custom Validation | In Progress | Extension points defined |
| Transaction Validation | In Progress | Basic functionality working |
| Cross-Domain Validation | In Progress | Framework in development |

## Future Enhancements

1. **Parallel Validation**: Enable parallel execution of validation stages
2. **Validation Caching**: Cache validation results for performance
3. **Policy-Based Validation**: Define validation rules using a policy language
4. **Machine Learning Validation**: Use ML for anomaly detection
5. **Formal Verification**: Integrate formal verification techniques
6. **Interactive Error Resolution**: Guidance for resolving validation errors

## References

- [Architecture Overview](architecture.md)
- [Resource Validation](resource_validation.md)
- [Capability Model](capability_model.md)
- [Temporal Validation](temporal_validation.md)
- [Cross-Domain Operations](cross_domain_operations.md)
- [Transaction Model](transaction_model.md)
- [Security Architecture](security.md) 