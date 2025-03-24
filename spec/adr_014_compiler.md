# ADR 014: Temporal Effect Language Compiler Architecture

## Status

Proposed

## Implementation Status

Partially implemented. The Temporal Effect Language (TEL) compiler architecture described in this ADR has been partially implemented with several key components in place:

1. **Handler System**: The handler interface (`TelHandler`, `ConstraintTelHandler`, etc.) has been implemented in `/src/tel/handlers.rs`
2. **Domain-Specific Adapters**: Domain-specific handlers (e.g., `EvmTransferHandler`) have been implemented
3. **TEL Compiler Interface**: A basic compiler interface (`TelCompiler`) and a standard implementation (`StandardTelCompiler`) exist
4. **Builder Pattern**: The builder pattern for TEL components is implemented in `/src/tel/builder.rs`
5. **Effect Types**: The core effect types and operations are implemented

However, several components described in the ADR are not yet fully implemented:

1. **Formal Type System**: The detailed type system with resource linearity constraints is not yet implemented
2. **AST Representation**: The full AST structure with effect tracking is partially implemented
3. **Validation System**: The effect validation and resource conservation validation is in early stages
4. **IR Generation**: The intermediate representation is not fully developed
5. **Optimization**: The optimization passes are not implemented
6. **Content-Addressable Integration**: Integration with the content-addressable storage is partial

The implementation appears to be following the direction set out in the ADR, but is still in development, with basic interfaces in place but more complex features like type checking, validation, and optimization still to be completed.

## Context

As Causality matures, we need a formal compiler architecture for the Temporal Effect Language (TEL) implemented in Rust. We need a clearly defined compiler pipeline that integrates with our content-addressable code system while preserving core properties required for cross-domain programming:

- Strong typing with effect tracking
- Resource linearity (preventing double-spend)
- Domain-aware causality validation
- Integration with content-addressable code
- Support for formalized resource models and controller patterns

A consistent and powerful compiler architecture is essential for ensuring programs behave predictably across multiple Domains and can be reasoned about statically before deployment. Furthermore, the formalized resource model outlined in ADR 018 introduces new requirements for the compiler to validate resource properties, conservation laws, and controller label integrity.

## Decision

We will implement a TEL compiler in Rust with a multi-stage pipeline that transforms source code into deployable artifacts while enforcing Causality's semantic guarantees. The compiler will integrate deeply with our content-addressable code system to enable incremental compilation, precise dependency management, and reproducible builds.

### Compiler Pipeline Architecture

The compiler will follow this pipeline structure:

```
┌─────────┐   ┌────────────┐   ┌──────────────┐   ┌───────────────┐   ┌───────────────┐
│ Parsing │──▶│ Type Check │──▶│ Effect Check │──▶│ IR Generation │──▶│ Optimization  │
└─────────┘   └────────────┘   └──────────────┘   └───────────────┘   └───────┬───────┘
                                                                              │
                                                                              ▼
                                                                     ┌───────────────┐
                                                                     │   Artifact    │
                                                                     │  Generation   │
                                                                     └───────────────┘
```

Each stage builds on the previous one, gradually transforming raw source into verified and optimized deployment artifacts.

### 1. Parser and AST

The TEL parser will use Rust's strong type system to create a well-defined AST that captures domain-specific concerns:

```rust
/// Core AST Structure
pub mod ast {
    use std::collections::HashMap;
    use crate::types::{ContentHash, Identifier, SourceLocation};
    use crate::resource::{ControllerID, ResourceID};
    
    /// Represents a TEL expression in the Abstract Syntax Tree
    #[derive(Debug, Clone)]
    pub enum TelExpression {
        Literal(LiteralValue),
        Variable(Identifier),
        Apply {
            function: Box<TelExpression>,
            arguments: Vec<TelExpression>,
        },
        Lambda {
            parameters: Vec<Pattern>,
            body: Box<TelExpression>,
        },
        Let {
            bindings: Vec<(Identifier, TelExpression)>,
            body: Box<TelExpression>,
        },
        If {
            condition: Box<TelExpression>,
            then_branch: Box<TelExpression>,
            else_branch: Box<TelExpression>,
        },
        Match {
            scrutinee: Box<TelExpression>,
            cases: Vec<(Pattern, TelExpression)>,
        },
        TimeExpr(TimeExpression),
        EffectExpr(EffectExpression),
        ResourceExpr(ResourceExpression),
        ControllerExpr(ControllerExpression),
        Do(Vec<DoStatement>),
        HashRef(ContentHash),
    }
    
    /// Time-related expressions
    #[derive(Debug, Clone)]
    pub enum TimeExpression {
        After {
            duration: Duration,
            expr: Box<TelExpression>,
        },
        Within {
            duration: Duration,
            expr: Box<TelExpression>,
        },
        At {
            time_point: TimePoint,
            expr: Box<TelExpression>,
        },
        Race(Vec<TelExpression>),
        Barrier {
            expressions: Vec<TelExpression>,
            condition: Condition,
        },
    }
    
    /// Effect expressions
    #[derive(Debug, Clone)]
    pub enum EffectExpression {
        Deposit {
            asset: Asset,
            amount: Amount,
            domain: Domain,
        },
        Withdraw {
            asset: Asset,
            amount: Amount,
            domain: Domain,
        },
        Transfer {
            asset: Asset,
            amount: Amount,
            source: Address,
            destination: Address,
        },
        Observe {
            fact_type: FactType,
            domain: Domain,
        },
        Invoke {
            program_id: ProgramID,
            entry_point: EntryPoint,
            arguments: Vec<TelExpression>,
        },
        Watch {
            condition: Condition,
            expr: Box<TelExpression>,
        },
        EndorseState {
            controller_id: ControllerID,
            state_root: StateRoot,
            proof: Proof,
        },
    }
    
    /// Resource expressions for formalized resources
    #[derive(Debug, Clone)]
    pub enum ResourceExpression {
        DefineResource {
            logic: ResourceLogic,
            fungibility_domain: FungibilityDomain,
            quantity: Quantity,
            value: Value,
        },
        NullifyResource {
            resource_id: ResourceID,
            nullifier_key: NullifierKey,
        },
        CreateResource {
            definition: ResourceDefinition,
            controller_label: ControllerLabel,
        },
        ComputeDelta(Vec<ResourceID>),
        ValidateResourceBalance(Vec<ResourceID>),
    }
    
    /// Controller expressions
    #[derive(Debug, Clone)]
    pub enum ControllerExpression {
        DefineController {
            controller_id: ControllerID,
            controller_type: ControllerType,
            finalization_rules: FinalizationRules,
        },
        EndorseController {
            controller_id: ControllerID,
            state_root: StateRoot,
            proof: Proof,
        },
        ValidateControllerDomain(ControllerLabel),
        ApplyEndorsements {
            controller_ids: Vec<ControllerID>,
            endorsements: Vec<Endorsement>,
        },
    }
    
    /// Pattern matching
    #[derive(Debug, Clone)]
    pub enum Pattern {
        VarPattern(Identifier),
        LiteralPattern(LiteralValue),
        ConstructorPattern {
            constructor: Identifier,
            fields: Vec<Pattern>,
        },
        WildcardPattern,
        AsPattern {
            pattern: Box<Pattern>,
            binding: Identifier,
        },
        OrPattern(Vec<Pattern>),
        GuardPattern {
            pattern: Box<Pattern>,
            guard: Box<TelExpression>,
        },
        ResourcePattern(ResourceID),
    }
    
    /// Represents an AST node with metadata
    #[derive(Debug, Clone)]
    pub struct AstNode {
        pub id: AstNodeId,
        pub expression: TelExpression,
        pub source_location: SourceLocation,
        pub type_info: Option<Type>,
        pub effect_set: Option<EffectSet>,
        pub resource_usage: Option<ResourceUsage>,
        pub domain_dependencies: Option<Vec<Domain>>,
        pub controller_info: Option<ControllerInfo>,
    }
}
```

The AST will preserve source location information for error reporting and debugging, with annotations for:
- Type information
- Effect sets (which effects each expression may trigger)
- Resource usage tracking and delta analysis
- Domain dependencies
- Controller label and ancestry tracking
- Endorsement relationships

### 2. Type System

TEL's type system will leverage Rust's strong typing with custom enums and structs:

```rust
pub mod types {
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize};
    
    /// Represents a type in the TEL type system
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Type {
        Base(BaseType),
        Var(TypeVar),
        Arrow {
            parameter: Box<Type>,
            result: Box<Type>,
        },
        Effect(Box<Type>),
        Domain(Box<Type>),
        Resource(ResourceType),
        List(Box<Type>),
        Tuple(Vec<Type>),
        Schema(SchemaType),
        Controller(ControllerType),
        ResourceLogic(LogicType),
        Delta(DeltaType),
        ControllerLabel(LabelType),
        Endorsement(EndorsementType),
    }
    
    /// Type checking context
    pub struct TypeContext {
        type_environment: HashMap<String, Type>,
        effect_environment: HashMap<String, EffectSet>,
        resource_environment: HashMap<String, ResourceType>,
        controller_environment: HashMap<String, ControllerType>,
        constraints: Vec<TypeConstraint>,
    }
    
    /// Type constraint for inference
    pub enum TypeConstraint {
        Equality(Type, Type),
        Subtype(Type, Type),
        EffectContainment(EffectSet, EffectSet),
        ResourceLinear(ResourceType),
        ResourceConservation(Vec<ResourceID>),
        ControllerValid(ControllerLabel),
        EndorsementValid(ControllerID, Vec<Endorsement>),
    }
    
    /// Type checker that implements inference with constraints
    pub struct TypeChecker {
        context: TypeContext,
    }
    
    impl TypeChecker {
        pub fn new() -> Self { /* ... */ }
        
        pub fn check_program(&mut self, program: &ast::Program) -> Result<Type, TypeError> { /* ... */ }
        
        pub fn infer_expression(&mut self, expr: &ast::TelExpression) -> Result<Type, TypeError> { /* ... */ }
        
        pub fn check_effect_flow(&mut self, expr: &ast::TelExpression) -> Result<EffectSet, TypeError> { /* ... */ }
        
        pub fn check_resource_linearity(&mut self, expr: &ast::TelExpression) -> Result<(), TypeError> { /* ... */ }
        
        pub fn check_controller_validity(&mut self, expr: &ast::TelExpression) -> Result<(), TypeError> { /* ... */ }
        
        pub fn solve_constraints(&mut self) -> Result<Substitution, TypeError> { /* ... */ }
    }
}
```

The type checker will enforce:
1. Resources are never duplicated or lost (linear types)
2. Effects are used consistently with their declared signatures
3. Domain-specific constraints are respected
4. Cross-domain causality is preserved
5. Resource delta calculations sum to zero (conservation laws)
6. Controller labels maintain valid ancestry Domains
7. Endorsements are properly validated
8. Dual validation constraints are satisfied

Type inference will use a modified Hindley-Milner algorithm extended with:
- Linear type constraints for resource tracking
- Effect tracking for causality
- Resource delta inference for conservation law checking
- Controller label propagation for ancestral validity
- Temporal constraint analysis for dual validation

### 3. Effect and Resource Validation

The validation phase will analyze both causal relationships between effects and resource conservation properties:

```rust
pub mod validation {
    use std::collections::{HashMap, HashSet};
    use crate::ast::{AstNodeId, TelExpression};
    use crate::types::{EffectSet, ResourceType};
    use crate::resource::{ResourceID, ControllerID};
    
    /// Represents the effect graph for validation
    pub struct EffectGraph {
        nodes: HashMap<EffectNodeID, EffectNode>,
        edges: HashMap<EffectNodeID, Vec<EffectNodeID>>,
        resource_flow: ResourceFlowMap,
        controller_map: ControllerMap,
        delta_tracker: ResourceDeltaTracker,
    }
    
    /// Validator for effect graphs
    pub struct EffectValidator {
        graph: EffectGraph,
    }
    
    impl EffectValidator {
        pub fn new(program: &ast::Program) -> Self { /* ... */ }
        
        /// Validate the entire effect graph
        pub fn validate_effect_graph(&self) -> Result<ValidatedGraph, ValidationError> { /* ... */ }
        
        /// Validate resource deltas
        pub fn validate_resource_deltas(&self) -> Result<ValidatedDeltas, DeltaError> { /* ... */ }
        
        /// Validate controller domains
        pub fn validate_controller_domains(&self) -> Result<ValidatedControllers, ControllerError> { /* ... */ }
        
        /// Perform dual validation (temporal + ancestral)
        pub fn validate_dual(&self) -> Result<DualValidationResult, ValidationError> { /* ... */ }
    }
    
    /// Resource delta validator
    pub struct DeltaValidator {
        delta_map: HashMap<AstNodeId, Delta>,
    }
    
    impl DeltaValidator {
        pub fn new(program: &ast::Program) -> Self { /* ... */ }
        
        /// Calculate and validate deltas for all nodes
        pub fn calculate_deltas(&mut self) -> Result<(), DeltaError> { /* ... */ }
        
        /// Verify conservation laws are maintained
        pub fn verify_conservation(&self) -> Result<(), DeltaError> { /* ... */ }
    }
    
    /// Controller validator
    pub struct ControllerValidator {
        controller_map: HashMap<ControllerID, ControllerInfo>,
    }
    
    impl ControllerValidator {
        pub fn new(program: &ast::Program) -> Self { /* ... */ }
        
        /// Validate controller ancestry
        pub fn validate_ancestry(&self) -> Result<(), ControllerError> { /* ... */ }
        
        /// Validate endorsements
        pub fn validate_endorsements(&self) -> Result<(), ControllerError> { /* ... */ }
    }
}
```

This phase will:
- Verify temporal consistency (correct ordering of events)
- Validate resource flows (no duplication or loss)
- Check that preconditions are satisfied
- Identify potential non-determinism
- Generate warnings for race conditions and hazards
- Calculate and verify resource deltas sum to zero
- Validate controller label ancestry Domains
- Verify endorsement applications are valid
- Implement dual validation (temporal + ancestral) for cross-domain operations

### 4. Intermediate Representation (IR)

The compiler will use a specialized IR optimized for effect-based programs with formalized resources:

```rust
pub mod ir {
    use std::collections::HashMap;
    use crate::types::{ContentHash, TimePoint, Domain};
    use crate::resource::{ResourceID, ControllerID};
    
    /// The core IR structure
    pub struct TelIR {
        pub effect_dag: EffectDAG,
        pub resource_mappings: ResourceMap,
        pub time_constraints: TimeMap,
        pub fact_dependencies: FactMap,
        pub resource_definitions: ResourceDefMap,
        pub controller_labels: ControllerLabelMap,
        pub delta_balances: DeltaMap,
        pub endorsements: EndorsementMap,
        pub dual_validation: DualValidationMap,
    }
    
    /// The effect DAG in the IR
    pub struct EffectDAG {
        pub nodes: HashMap<EffectID, Effect>,
        pub edges: HashMap<EffectID, Vec<EffectID>>,
    }
    
    /// Map of resource definitions
    pub struct ResourceDefMap {
        pub resources: HashMap<ResourceID, ResourceDefinition>,
        pub commitments: HashMap<ResourceID, Commitment>,
        pub nullifiers: HashMap<ResourceID, Nullifier>,
        pub kinds: HashMap<ResourceID, Kind>,
    }
    
    /// Map of controller labels
    pub struct ControllerLabelMap {
        pub labels: HashMap<ResourceID, ControllerLabel>,
        pub reductions: HashMap<Vec<ControllerID>, Vec<ControllerID>>,
    }
    
    /// IR generator
    pub struct IRGenerator {
        pub program: ast::Program,
        pub type_info: TypeInfo,
        pub validation_result: ValidationResult,
    }
    
    impl IRGenerator {
        pub fn new(program: ast::Program, type_info: TypeInfo, validation_result: ValidationResult) -> Self { /* ... */ }
        
        /// Generate the IR from the AST
        pub fn generate_ir(&self) -> Result<TelIR, IRGenerationError> { /* ... */ }
        
        /// Generate the effect DAG
        fn generate_effect_dag(&self) -> Result<EffectDAG, IRGenerationError> { /* ... */ }
        
        /// Generate resource mappings
        fn generate_resource_mappings(&self) -> Result<ResourceMap, IRGenerationError> { /* ... */ }
        
        /// Generate controller label mappings
        fn generate_controller_labels(&self) -> Result<ControllerLabelMap, IRGenerationError> { /* ... */ }
    }
}
```

The IR preserves the full effect graph while abstracting syntax details, making it ideal for optimization and code generation. The enhanced IR also captures resource definitions, controller labels, and delta calculations needed for the formalized resource model.

### 5. Optimization

The optimizer will apply several passes:

```rust
pub mod optimizer {
    use crate::ir::TelIR;
    
    /// The main optimizer
    pub struct Optimizer {
        pub ir: TelIR,
        pub optimization_level: OptimizationLevel,
    }
    
    /// Available optimization passes
    pub enum OptimizationPass {
        EffectFusion,
        DeadEffectElimination,
        ResourceLocalization,
        DomainBatching,
        FactDeduplication,
        ControllerLabelReduction,
        DeltaComputationFusion,
        DualValidationOptimization,
        ResourceCommitmentPrecomputation,
    }
    
    impl Optimizer {
        pub fn new(ir: TelIR, level: OptimizationLevel) -> Self { /* ... */ }
        
        /// Run all optimizations at the specified level
        pub fn optimize(&mut self) -> Result<TelIR, OptimizationError> { /* ... */ }
        
        /// Run a specific optimization pass
        pub fn run_pass(&mut self, pass: OptimizationPass) -> Result<(), OptimizationError> { /* ... */ }
        
        /// Fuse compatible effects
        fn effect_fusion(&mut self) -> Result<(), OptimizationError> { /* ... */ }
        
        /// Eliminate dead effects
        fn dead_effect_elimination(&mut self) -> Result<(), OptimizationError> { /* ... */ }
        
        /// Localize resources
        fn resource_localization(&mut self) -> Result<(), OptimizationError> { /* ... */ }
        
        /// Batch domain operations
        fn domain_batching(&mut self) -> Result<(), OptimizationError> { /* ... */ }
        
        /// Reduce controller labels via endorsements
        fn controller_label_reduction(&mut self) -> Result<(), OptimizationError> { /* ... */ }
    }
}
```

All optimizations must preserve causal ordering, effect semantics, resource conservation laws, and controller ancestry integrity. The optimizer must be particularly careful with optimizations that affect resource delta calculations or controller label validity.

### 6. Artifact Generation

The final stage generates deployable program artifacts:

```rust
pub mod codegen {
    use std::collections::HashSet;
    use crate::ir::TelIR;
    use crate::types::ContentHash;
    
    /// A compiled TEL program
    pub struct CompiledProgram {
        pub program_hash: ContentHash,
        pub effect_dag: EffectDAG,
        pub dependencies: HashSet<ContentHash>,
        pub schema: Schema,
        pub schema_evolution_rules: Vec<EvolutionRule>,
        pub compatible_protocol_versions: VersionRange,
        pub ir_debug_info: Option<IRDebugInfo>,
        pub resource_definitions: ResourceDefSet,
        pub controller_mappings: ControllerMappingSet,
        pub delta_verification: DeltaVerificationProof,
        pub dual_validation_evidence: DualValidationProof,
    }
    
    /// Code generator
    pub struct CodeGenerator {
        pub ir: TelIR,
        pub target: CodegenTarget,
    }
    
    impl CodeGenerator {
        pub fn new(ir: TelIR, target: CodegenTarget) -> Self { /* ... */ }
        
        /// Generate code for the specified target
        pub fn generate_code(&self) -> Result<CompiledProgram, CodegenError> { /* ... */ }
        
        /// Generate the effect DAG in binary format
        fn generate_effect_dag(&self) -> Result<Vec<u8>, CodegenError> { /* ... */ }
        
        /// Generate resource definitions
        fn generate_resource_definitions(&self) -> Result<Vec<u8>, CodegenError> { /* ... */ }
        
        /// Generate controller mappings
        fn generate_controller_mappings(&self) -> Result<Vec<u8>, CodegenError> { /* ... */ }
        
        /// Generate delta verification proofs
        fn generate_delta_proofs(&self) -> Result<Vec<u8>, CodegenError> { /* ... */ }
    }
}
```

The enhanced program artifact includes the formalized resource definitions, controller mappings, and proofs of both delta verification and dual validation. These additions ensure that any program operating with formalized resources can be verified for correctness both at compile time and runtime.

### Content-Addressable Integration

The compiler will leverage our content-addressable code system for:

```rust
pub mod content_addressing {
    use std::collections::HashMap;
    use crate::types::ContentHash;
    use crate::codegen::CompiledProgram;
    
    /// Content-addressed compiler
    pub struct ContentAddressedTelCompiler {
        pub storage: Arc<dyn ContentAddressableStorage>,
        pub dependencies: HashMap<String, ContentHash>,
    }
    
    impl ContentAddressedTelCompiler {
        pub fn new(storage: Arc<dyn ContentAddressableStorage>) -> Self { /* ... */ }
        
        /// Compile a TEL program
        pub fn compile(&self, source: &str, options: CompilationOptions) -> Result<ContentHash, CompilationError> { /* ... */ }
        
        /// Load a dependency
        pub fn load_dependency(&self, hash: &ContentHash) -> Result<CompiledProgram, CompilationError> { /* ... */ }
        
        /// Store a compiled artifact
        pub fn store_artifact(&self, program: &CompiledProgram) -> Result<ContentHash, CompilationError> { /* ... */ }
    }
    
    /// Trait for content-addressable storage
    pub trait ContentAddressableStorage: Send + Sync {
        fn store(&self, data: &[u8]) -> Result<ContentHash, StorageError>;
        fn load(&self, hash: &ContentHash) -> Result<Vec<u8>, StorageError>;
        fn exists(&self, hash: &ContentHash) -> Result<bool, StorageError>;
    }
}
```

Every compiler artifact receives a unique content hash, making compilation deterministic and verifiable.

### Compiler CLI Interface

```rust
pub mod cli {
    use clap::{App, Arg, SubCommand};
    use crate::compiler::ContentAddressedTelCompiler;
    
    /// CLI for the TEL compiler
    pub struct TelCompilerCli {
        compiler: ContentAddressedTelCompiler,
    }
    
    impl TelCompilerCli {
        pub fn new() -> Self { /* ... */ }
        
        /// Run the CLI
        pub fn run(&self) -> Result<(), CliError> { /* ... */ }
        
        /// Build a TEL program
        fn cmd_build(&self, source: &str, output: &str) -> Result<(), CliError> { /* ... */ }
        
        /// Check a program for errors
        fn cmd_check(&self, source: &str) -> Result<(), CliError> { /* ... */ }
        
        /// Generate a visualization
        fn cmd_viz(&self, source: &str, output: &str) -> Result<(), CliError> { /* ... */ }
        
        /// Generate a simulation scenario
        fn cmd_gen_scenario(&self, source: &str, output: &str) -> Result<(), CliError> { /* ... */ }
    }
}
```

## Consequences

### Positive

- **Rust's Type System**: Leveraging Rust's powerful type system for enhanced safety
- **Strong correctness guarantees**: Static verification prevents many runtime errors
- **Precise dependency management**: Content-addressable artifacts ensure reproducibility
- **Visual development**: Generated flow diagrams improve understanding
- **Incremental compilation**: Separate compilation improves development velocity
- **Deep integration**: Compiler leverages existing Causality architectural features
- **Conservation proofs**: Resource delta validation ensures conservation laws are respected
- **Controller Domain validation**: Ancestral validity proofs for cross-domain resources
- **Dual validation**: Combining temporal and ancestral validation for stronger security
- **Optimized controller Domains**: Endorsement-based reductions of controller ancestry

### Challenges

- **Lifecycle management**: Balancing ownership and borrowing in the compiler pipeline
- **Error propagation**: Designing a consistent error handling strategy across pipeline stages
- **Compiler complexity**: Linear types, effect analysis, and resource formalization require sophisticated techniques
- **Performance concerns**: Linear type checking and resource delta calculation may become bottlenecks for large programs
- **Extensibility**: We must design for effect system extensibility from the start
- **Learning curve**: Developers will need to understand new type system concepts and formalized resource model
- **Validation overhead**: Dual validation increases compilation and verification time
- **Controller abstraction complexity**: Reasoning about controller labels and endorsements adds cognitive load
- **Optimized pattern discovery**: Finding opportunities for controller Domain reduction requires advanced analysis

### Compiler Design Implications

The compiler architecture has several long-term implications:

- **Language evolution**: The design constrains how TEL can evolve
- **Performance characteristics**: Trade-offs between compilation time and runtime performance
- **Debugging experience**: Output quality shapes developer experience
- **Tooling requirements**: Rich diagnostics and visualizations are essential

## Implementation Plan

We will implement the compiler in phases:

1. **Core pipeline**: AST, basic type checking, and artifact generation (4 weeks)
   - Create AST module with expression types
   - Implement parser using a parser combinator library
   - Develop basic type checker with inference
   - Set up content-addressed artifact generation

2. **Effect validation**: Causality checking and resource flow analysis (3 weeks)
   - Implement effect graph construction
   - Create validation algorithms for causality
   - Build resource flow tracking

3. **Resource formalization**: Formal resource model implementation and delta validation (3 weeks)
   - Add resource definitions and delta tracking
   - Implement conservation validation
   - Create resource attribution system

4. **Controller integration**: Controller label tracking and ancestry validation (2 weeks)
   - Implement controller label data structures
   - Create ancestry validation algorithms
   - Build controller transition tracking

5. **Dual validation**: Combined temporal and ancestral validation (2 weeks)
   - Create dual validation algorithm
   - Implement verification proofs
   - Build validation evidence collection

6. **Optimization**: Performance-focused transformations including controller Domain reduction (3 weeks)
   - Implement effect fusion and dead code elimination
   - Create domain batching optimizations
   - Build controller label reduction algorithms

7. **Integration**: Content-addressable code system integration (2 weeks)
   - Implement dependency resolution
   - Create incremental compilation support
   - Build artifact storage and retrieval

8. **Tooling**: Visualizations and developer experience improvements (3 weeks)
   - Create flow diagram visualization tools
   - Implement debugging information generation
   - Build CLI tools and IDE integration

Each phase will include thorough testing and documentation to ensure both correctness and usability. We'll leverage Rust's strong testing capabilities and documentation tools to ensure high quality throughout.

### Integration with Existing Code

The implementation will integrate with:

1. The AST module (`src/ast.rs` and `src/ast/resource_graph.rs`)
2. The resource system (`src/resource.rs` and related modules)
3. The existing TEL modules (`src/tel/` directory)
4. The content addressing system (`src/effect_adapters/` modules including `hash.rs`, `repository.rs`, etc.)

We'll follow the file-based module structure rather than a directory-based one, creating new modules as needed to implement the compiler pipeline stages. 