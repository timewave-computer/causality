# ADR-025: Unified Operation Model

## Status

Proposed

## Context

Our current architecture distinguishes between three interrelated but separate operational concepts:

1. **Effects**: Abstract operations on resources that express user intent (e.g., transfer tokens, update data)
2. **Operations**: Concrete implementations of effects on registers that define specific implementation logic
3. **ResourceRegister Operations**: Physical state changes that represent the actual on-chain transformations

This separation has evolved organically as the system has grown and now presents several challenges:

- **Cognitive Overhead**: Developers must understand and navigate between three related but distinct concepts
- **Redundant Transformation Logic**: We maintain separate transformation pipelines from effects → operations → register operations
- **Inconsistent Validation**: Validation rules are applied differently at each layer
- **Complex ZK Circuit Integration**: Proof generation needs to bridge these concepts, creating additional complexity
- **Debugging Difficulty**: Tracing an operation through these transformations is unnecessarily complex

The resource formalization model (ADR-018) and ZK-based register system (ADR-006) have already moved us toward a more coherent model for resources and their state transitions. It's time to extend this coherence to the operational model.

## Decision

We will unify these three concepts into a single comprehensive **Operation** model that can represent the operation at different levels of abstraction, with different execution contexts:

```rust
/// Unified Operation model that spans abstraction levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation<C: ExecutionContext> {
    /// Unique identifier for this operation
    pub id: OperationId,
    
    /// The operation type
    pub op_type: OperationType,
    
    /// Abstract representation (what the operation logically does)
    pub abstract_representation: Effect,
    
    /// Concrete implementation (how it's implemented on registers)
    pub concrete_implementation: RegisterOperation,
    
    /// Physical execution details (actual on-chain state changes)
    pub physical_execution: Option<ResourceRegisterOperation>,
    
    /// Execution context (where and how it executes)
    pub context: C,
    
    /// Input resources/registers this operation reads from
    pub inputs: Vec<ResourceRef>,
    
    /// Output resources/registers this operation writes to
    pub outputs: Vec<ResourceRef>,
    
    /// Authorization for this operation
    pub authorization: Authorization,
    
    /// ZK proof for this operation (if applicable)
    pub proof: Option<Proof>,
    
    /// Temporal facts this operation depends on
    pub temporal_facts: Vec<FactId>,
    
    /// Resource conservation details (ΔTX = 0 enforcement)
    pub conservation: ResourceConservation,
    
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}
```

This unified model will be parameterized by an execution context, allowing different contexts for different stages of the operation lifecycle:

```rust
/// Trait for operation execution contexts
pub trait ExecutionContext: Clone + Debug + Serialize + Deserialize {
    /// The environment this context operates in
    fn environment(&self) -> ExecutionEnvironment;
    
    /// The domain this context is associated with (if any)
    fn domain(&self) -> Option<DomainId>;
    
    /// The execution phase this context represents
    fn phase(&self) -> ExecutionPhase;
    
    /// Whether this context requires a ZK proof
    fn requires_proof(&self) -> bool;
    
    /// Get capability requirements for this context
    fn capability_requirements(&self) -> Vec<Capability>;
}

/// Execution phases for operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionPhase {
    /// Planning phase (intent formation)
    Planning,
    
    /// Validation phase (checking preconditions)
    Validation,
    
    /// Authorization phase (verifying permissions)
    Authorization,
    
    /// Execution phase (applying changes)
    Execution,
    
    /// Verification phase (confirming effects)
    Verification,
    
    /// Finalization phase (recording outcomes)
    Finalization,
}

/// Execution environments
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionEnvironment {
    /// Abstract environment (logical operations)
    Abstract,
    
    /// Program execution environment
    Program,
    
    /// Register-based environment
    Register,
    
    /// Physical on-chain environment
    OnChain(DomainId),
    
    /// ZK verification environment
    ZkVm,
}
```

The new architecture will provide:

### 1. Progressive Refinement

An operation will start as an abstract intent and progressively refine through the execution pipeline:

```rust
// Operations refine from abstract to concrete
let abstract_op = Operation::<AbstractContext>::from_effect(transfer_effect);
let program_op = abstract_op.refine_to::<ProgramContext>();
let register_op = program_op.refine_to::<RegisterContext>();
let physical_op = register_op.refine_to::<PhysicalContext>();
```

### 2. Unified Validation Pipeline

Validation will be applied consistently across all operation phases:

```rust
/// Validate an operation at any level of abstraction
pub fn validate_operation<C: ExecutionContext>(
    operation: &Operation<C>,
    validator: &dyn OperationValidator<C>
) -> Result<ValidationResult, ValidationError> {
    // Validate basic operation structure
    validator.validate_structure(operation)?;
    
    // Validate resource references
    validator.validate_resource_refs(operation)?;
    
    // Validate authorization
    validator.validate_authorization(operation)?;
    
    // Validate conservation laws
    validator.validate_conservation(operation)?;
    
    // Validate temporal consistency
    validator.validate_temporal_consistency(operation)?;
    
    // Context-specific validation
    validator.validate_context_specific(operation)?;
    
    Ok(ValidationResult::valid())
}
```

### 3. Simplified ZK Proof Generation

ZK proof generation will work directly with operations:

```rust
/// Generate a ZK proof for an operation
pub async fn generate_operation_proof<C: ExecutionContext>(
    operation: &Operation<C>,
    prover: &dyn Prover
) -> Result<Proof, ProverError> {
    // Create circuit inputs from operation
    let (public_inputs, witness) = create_circuit_inputs(operation)?;
    
    // Select appropriate circuit for operation type
    let circuit = select_circuit_for_operation(operation)?;
    
    // Generate the proof
    let proof = prover.generate_proof(circuit, public_inputs, witness).await?;
    
    Ok(proof)
}
```

### 4. Consistent Developer API

Developers will work with a single operation concept that adapts to their specific needs:

```rust
// Create an operation (works at any abstraction level)
let operation = Operation::new(OperationType::Transfer)
    .with_input(token_resource)
    .with_output(token_resource.with_owner(recipient))
    .with_authorization(signature_auth)
    .with_fact_dependencies(vec![balance_fact_id]);

// Execute the operation (execution context determines behavior)
let result = execute_operation(operation, context).await?;
```

### 5. Traceable Lifecycle

The operation lifecycle will be fully traceable:

```rust
/// Record operation lifecycle events
pub async fn record_operation_event<C: ExecutionContext>(
    operation: &Operation<C>,
    event: OperationLifecycleEvent
) -> Result<(), LogError> {
    let entry = LogEntry {
        operation_id: operation.id.clone(),
        event_type: event,
        timestamp: Utc::now(),
        context: operation.context.clone(),
        metadata: operation.metadata.clone(),
    };
    
    log_system().append_entry(entry).await
}
```

## Implementation Strategy

We'll implement this unification through the following phases:

### 1. Core Unified Model

Implement the core `Operation<C>` structure and basic context types:

```rust
/// Implementation for abstract context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractContext {
    pub intent_type: IntentType,
    pub originator: UserId,
    pub execution_requirements: ExecutionRequirements,
}

impl ExecutionContext for AbstractContext {
    fn environment(&self) -> ExecutionEnvironment {
        ExecutionEnvironment::Abstract
    }
    
    fn domain(&self) -> Option<DomainId> {
        None // Abstract context isn't tied to a specific domain
    }
    
    fn phase(&self) -> ExecutionPhase {
        ExecutionPhase::Planning
    }
    
    fn requires_proof(&self) -> bool {
        false // Abstract operations don't require proofs
    }
    
    fn capability_requirements(&self) -> Vec<Capability> {
        vec![] // Abstract operations don't require capabilities
    }
}

// Similar implementations for other contexts...
```

### 2. Transformation Functions

Implement transformation functions between different context types:

```rust
/// Extension methods for operations
impl<C: ExecutionContext> Operation<C> {
    /// Refine this operation to a more concrete context
    pub fn refine_to<D: ExecutionContext>(&self) -> Result<Operation<D>, RefinementError> {
        // Create the new context
        let new_context = D::from_previous_context(&self.context)?;
        
        // Refine the abstract representation if needed
        let abstract_representation = self.abstract_representation.clone();
        
        // Refine the concrete implementation
        let concrete_implementation = refine_implementation(
            &self.concrete_implementation,
            &self.context,
            &new_context
        )?;
        
        // Refine the physical execution if present
        let physical_execution = if let Some(physical) = &self.physical_execution {
            Some(refine_physical_implementation(physical, &self.context, &new_context)?)
        } else {
            None
        };
        
        // Create the refined operation
        Ok(Operation {
            id: self.id.clone(),
            op_type: self.op_type.clone(),
            abstract_representation,
            concrete_implementation,
            physical_execution,
            context: new_context,
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            authorization: self.authorization.clone(),
            proof: self.proof.clone(),
            temporal_facts: self.temporal_facts.clone(),
            conservation: self.conservation.clone(),
            metadata: self.metadata.clone(),
        })
    }
    
    // Other transformation methods...
}
```

### 3. Execution Pipeline Integration

Integrate the unified operation model into the execution pipeline:

```rust
/// Execute an operation in a specific context
pub async fn execute_operation<C: ExecutionContext>(
    operation: Operation<C>,
    executor: &dyn Executor<C>
) -> Result<OperationResult, ExecutionError> {
    // Validate the operation
    validate_operation(&operation, executor.validator())?;
    
    // Generate proof if required
    let operation_with_proof = if operation.context.requires_proof() {
        let proof = generate_operation_proof(&operation, executor.prover()).await?;
        operation.with_proof(proof)
    } else {
        operation
    };
    
    // Execute the operation
    let result = executor.execute(operation_with_proof).await?;
    
    // Record the execution
    record_operation_event(&operation_with_proof, OperationLifecycleEvent::Executed).await?;
    
    Ok(result)
}
```

### 4. ZK Circuit Integration

Update ZK circuits to work with the unified operation model:

```rust
/// Create circuit inputs from an operation
pub fn create_circuit_inputs<C: ExecutionContext>(
    operation: &Operation<C>
) -> Result<(PublicInputs, WitnessInputs), CircuitError> {
    let mut public_inputs = HashMap::new();
    public_inputs.insert("operation_type".to_string(), operation.op_type.to_string());
    public_inputs.insert("operation_id".to_string(), operation.id.to_string());
    
    // Add inputs and outputs
    for (i, input) in operation.inputs.iter().enumerate() {
        public_inputs.insert(format!("input_{}", i), input.to_public_input()?);
    }
    
    for (i, output) in operation.outputs.iter().enumerate() {
        public_inputs.insert(format!("output_{}", i), output.to_public_input()?);
    }
    
    // Create witness inputs
    let mut witness = HashMap::new();
    witness.insert("authorization".to_string(), operation.authorization.to_witness()?);
    
    // Add context-specific witness data
    operation.context.add_witness_data(&mut witness)?;
    
    Ok((public_inputs, witness))
}
```

### 5. Migration Strategy

To migrate existing code:

1. **Create Adapter Classes**: Build adapters that translate between old and new models during the migration phase
2. **Progressive Replacement**: Gradually replace old effect/operation handling with unified operations
3. **Compatibility Shims**: Provide compatibility shims for existing interfaces until migration is complete

```rust
/// Adapter for legacy Effect to unified Operation
pub fn effect_to_operation(effect: &Effect) -> Operation<AbstractContext> {
    Operation {
        id: OperationId(Uuid::new_v4().to_string()),
        op_type: effect_type_to_operation_type(&effect.effect_type),
        abstract_representation: effect.clone(),
        concrete_implementation: empty_register_operation(), // Will be filled during refinement
        physical_execution: None, // Will be filled during execution
        context: AbstractContext {
            intent_type: intent_type_from_effect(effect),
            originator: effect.originator.clone(),
            execution_requirements: ExecutionRequirements::default(),
        },
        inputs: extract_inputs_from_effect(effect),
        outputs: extract_outputs_from_effect(effect),
        authorization: effect.authorization.clone(),
        proof: None,
        temporal_facts: effect.fact_dependencies.iter().map(|dep| dep.fact_id.clone()).collect(),
        conservation: ResourceConservation::from_effect(effect),
        metadata: effect.metadata.clone(),
    }
}
```

## Consequences

### Positive

1. **Simplified Conceptual Model**: Developers work with a single unified operation concept that spans abstractions
2. **Streamlined Execution Pipeline**: One concept being transformed rather than three separate concepts
3. **Integrated ZK Proof Generation**: Direct path from operation to ZK proof
4. **Consistent Validation**: Same validation framework applies at all levels
5. **Better Traceability**: Operations maintain identity throughout their lifecycle
6. **Improved Developer Experience**: Reduced cognitive load and learning curve
7. **More Comprehensive Logging**: Single operation ID can be traced through all stages of execution
8. **Enhanced Composability**: Operations can be composed more cleanly
9. **Future-Proof Architecture**: The context pattern allows for extension to new environments

### Negative

1. **Migration Complexity**: Significant refactoring of existing code required
2. **Performance Overhead**: Carrying complete operation information through all stages may introduce overhead
3. **Learning Curve**: Developers familiar with the current system will need to adapt
4. **Generic Programming Complexity**: Using generics and contexts adds implementation complexity

### Mitigation Strategies

1. **Phased Migration**: Implement the unification in phases, starting with core components
2. **Performance Optimization**: Use lazy loading and caching strategies to minimize overhead
3. **Comprehensive Documentation**: Create clear documentation with migration guides
4. **Training Sessions**: Conduct training sessions to help developers understand the new model
5. **Compatibility Layer**: Provide a compatibility layer for gradual migration

## Implementation Plan

We'll implement this ADR in the following phases:

1. **Phase 1: Core Model Definition** (3 weeks)
   - Define the unified `Operation<C>` structure and basic contexts
   - Implement transformation functions between contexts
   - Create basic validation framework

2. **Phase 2: Execution Integration** (4 weeks)
   - Update execution pipeline to use unified operations
   - Integrate ZK proof generation with unified model
   - Implement context-specific execution logic

3. **Phase 3: API Surface Updates** (3 weeks)
   - Create developer-friendly APIs for the unified model
   - Build fluent interfaces for operation construction
   - Update documentation and examples

4. **Phase 4: Migration** (6 weeks)
   - Create adapters for legacy systems
   - Gradually replace existing effect/operation handling
   - Update tests to use the new model

5. **Phase 5: Optimization** (2 weeks)
   - Profile and optimize performance
   - Implement lazy loading strategies
   - Reduce memory footprint where possible

## Example Usage

### Example 1: Creating and Executing a Token Transfer

```rust
// Create a transfer operation
let operation = Operation::new(OperationType::Transfer)
    .with_input(ResourceRef::new("token:ETH").with_owner(sender))
    .with_output(ResourceRef::new("token:ETH").with_owner(recipient))
    .with_amount(Amount::new(1, 18)) // 1 ETH with 18 decimals
    .with_authorization(SignatureAuthorization::new(sender_signature))
    .with_temporal_facts(vec![balance_fact_id]);

// Execute through the refined pipeline
let abstract_op = operation.with_context(AbstractContext::new());
let program_op = abstract_op.refine_to::<ProgramContext>()?;
let register_op = program_op.refine_to::<RegisterContext>()?;

// Execute in register context
let result = execute_operation(register_op, register_executor).await?;

// Get the on-chain representation if needed
let physical_op = result.operation.refine_to::<PhysicalContext>()?;
```

### Example 2: Cross-Domain Transfer with ZK Proof

```rust
// Create a cross-domain transfer operation
let operation = Operation::new(OperationType::CrossDomainTransfer)
    .with_input(ResourceRef::new("token:USDC").on_domain("ethereum"))
    .with_output(ResourceRef::new("token:USDC").on_domain("solana"))
    .with_amount(Amount::new(100, 6)) // 100 USDC with 6 decimals
    .with_authorization(MerkleProofAuthorization::new(merkle_proof))
    .with_temporal_facts(vec![eth_balance_fact_id, sol_state_fact_id]);

// Execute in ZK context which requires a proof
let zk_op = operation
    .refine_to::<AbstractContext>()?
    .refine_to::<ProgramContext>()?
    .refine_to::<ZkContext>()?;

// This will automatically generate a proof during execution
let result = execute_operation(zk_op, zk_executor).await?;

// The result contains the proof that can be verified on-chain
let proof = result.proof.unwrap();
```

### Example 3: Composing Operations

```rust
// Create component operations
let swap_op = Operation::new(OperationType::Swap)
    .with_input(ResourceRef::new("token:ETH"))
    .with_output(ResourceRef::new("token:USDC"))
    .with_amount(Amount::new(1, 18)) // 1 ETH with 18 decimals
    .with_min_output(Amount::new(1800, 6)) // Minimum 1800 USDC
    .with_authorization(signature_auth.clone());

let stake_op = Operation::new(OperationType::Stake)
    .with_input(ResourceRef::new("token:USDC"))
    .with_output(ResourceRef::new("token:stUSDC"))
    .with_amount(Amount::new(1800, 6)) // 1800 USDC
    .with_authorization(signature_auth.clone());

// Compose into a single operation
let composed_op = Operation::compose(
    vec![swap_op, stake_op],
    CompositionStrategy::Sequential,
    AbstractContext::new()
);

// Execute as a single atomic operation
let result = execution_system.execute(composed_op).await?;
```

## Alternatives Considered

### 1. Adapter Pattern Without Full Unification

We could create adapters between the current concepts without fully unifying them.

**Rejected because**: This would add complexity without addressing the fundamental issue of having three separate concepts.

### 2. Interface-Based Unification

We could define interfaces that all three concepts implement, without changing their underlying structure.

**Rejected because**: This would lead to interface bloat and wouldn't solve the transformation complexity.

### 3. Partial Unification (Effects + Operations)

We could unify effects and operations while keeping resource register operations separate.

**Rejected because**: This would only partially solve the problem and still require transformations between abstraction levels.

## Conclusion

The unified operation model offers a more coherent approach to representing operations in our system. By treating operations as a single concept that can be refined across different contexts, we simplify our architecture, streamline our execution pipeline, and improve the developer experience. While the migration will require significant effort, the long-term benefits in terms of maintainability, extensibility, and conceptual clarity justify this investment.

The model also aligns well with our existing architectural principles, particularly the resource formalization model (ADR-018) and ZK-based register system (ADR-006). It represents a natural evolution of our system toward a more unified, principled architecture.