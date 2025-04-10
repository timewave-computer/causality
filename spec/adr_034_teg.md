# ADR-034: Temporal Effect Graph (TEG) Intermediate Representation

## Status

Proposed

## Context

The Causality system enables cross-domain operations with temporal consistency, resource safety, and verifiable execution. As the system grows in complexity and supports multiple input languages (including TEL) as well as multiple target execution environments, we need a well-defined intermediate representation (IR) to:

1. Decouple the frontend language concerns from backend execution details
2. Enable optimization passes and static analysis 
3. Support cross-domain reasoning about effects and resources
4. Facilitate transpilation to multiple target environments
5. Provide a common validation layer between compilation and execution

Currently, there's no formal intermediate representation between the compilation phase in the TEL crate and the execution phase in the Engine crate. This tight coupling limits our ability to evolve either component independently and complicates the implementation of cross-cutting concerns like optimization.

Our algebraic effects model (defined in ADR-001 and extended in ADR-023) provides a natural foundation for an IR, but we need to formalize how this representation should be structured, serialized, and processed.

## Decision

We will implement a **Temporal Effect Graph (TEG)** as our intermediate representation. This graph-based IR will capture both the temporal relationships between effects and the resource state transitions they produce.

The TEG will be implemented in a new `temporality-ir` crate, distinct from both `temporality-tel` and `temporality-engine`, creating a clean separation of concerns and clear ownership.

### Category Theoretic Model

The TEG implements a formal categorical adjunction between the TEL language and algebraic effects execution. Specifically:

1. **TEL Category**: Represents TEL combinators
   - Objects: Types in TEL language (including resources)
   - Morphisms: TEL combinators transforming between types
   - Composition: Sequential combination of combinators
   - Identity: The identity combinator
   
2. **TEG Category**: Represents temporal effect graphs
   - Objects: Types in the effect system (including resource states)
   - Morphisms: Effect graph computations between types
   - Composition: Effect sequencing via continuations
   - Identity: The pure effect (identity function)

These categories are connected by two functors:
- F: TEL → TEG (translates combinators to graphs)
- G: TEG → TEL (translates graphs to combinators)

These establish a natural isomorphism: Hom_TEL(A, G(B)) ≅ Hom_TEG(F(A), B), which formalizes the bidirectional translation property. This is essential for ensuring semantic preservation during both compilation and execution.

The resource graph forms a symmetric monoidal structure over both categories, with:
- Resource types as objects
- Resource transformations as morphisms
- Combining resources as tensor product
- Order independence when combining resources (symmetry)

Capabilities are modeled as a presheaf over resources, mapping each resource configuration to its authorization requirements. The content addressing system ensures that the functors preserve identity and isomorphisms, critical for maintaining semantic equivalence across translations.

### Core Structure

The TEG represents programs as a directed graph where:

1. **Nodes** represent effects and resources
2. **Edges** represent dependencies, continuations, and resource access patterns
3. **Context** provide additional details like temporal constraints and capabilities

This structure captures the essential elements of the three-layer algebraic effect architecture:

```
┌───────────────────────────────────────────────────────────┐
│                   Temporal Effect Graph                   │
│                                                           │
│  ┌──────────────┐                    ┌─────────────────┐  │
│  │ Effect       │                    │ Resource        │  │
│  │ Nodes        │◄────Access────────►│ Nodes           │  │
│  └──────┬───────┘                    └────────┬────────┘  │
│         │                                     │           │
│    Depends On                           Relationships     │
│         │                                     │           │
│         ▼                                     ▼           │
│  ┌──────────────┐                     ┌─────────────────┐ │
│  │ Effect       │                     │ Resource        │ │
│  │ Continuations│◄───Authorization────┤ Capabilities    │ │
│  └──────────────┘                     └─────────────────┘ │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

### Core Components

```rust
pub struct EffectNode {
    id: EffectId,
    effect_type: String,
    parameters: HashMap<String, Value>,
    required_capabilities: Vec<CapabilityId>,
    resources_accessed: Vec<ResourceId>,
    fact_dependencies: Vec<FactId>,
    domain_id: DomainId,
    content_hash: ContentHash,
}

pub struct ResourceNode {
    id: ResourceId,
    resource_type: String,
    state: ResourceState,
    metadata: HashMap<String, Value>,
    domain_id: DomainId,
    content_hash: ContentHash,
}

pub struct TemporalEffectGraph {
    effect_nodes: HashMap<EffectId, EffectNode>,
    resource_nodes: HashMap<ResourceId, ResourceNode>,
    effect_dependencies: HashMap<EffectId, Vec<EffectId>>,
    effect_continuations: HashMap<EffectId, Vec<(EffectId, Option<Condition>)>>,
    resource_relationships: HashMap<ResourceId, Vec<(ResourceId, RelationshipType)>>,
    temporal_constraints: HashMap<EffectId, Vec<TemporalConstraint>>,
    capability_authorizations: HashMap<EffectId, Vec<CapabilityId>>,
    domains: HashSet<DomainId>,
    metadata: HashMap<String, Value>,
    content_hash: ContentHash,
}
```

### Serialization

We will use Borsh for serialization, maintaining consistency with the rest of the Temporality system. Borsh provides:

1. Deterministic, content-addressable serialization
2. High performance for both encoding and decoding
3. Compact binary representation 
4. Straightforward schema evolution

```rust
impl BorshSerialize for TemporalEffectGraph {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Implementation that ensures deterministic ordering
    }
}

impl BorshDeserialize for TemporalEffectGraph {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        // Implementation with validation
    }
}
```

### Builder API

The IR will include a fluid API for constructing and manipulating programs:

```rust
// Example usage of builder pattern
let mut teg = TemporalEffectGraph::new();

// Add resources
let account1 = teg.add_resource()
    .id("account1")
    .type("account")
    .state(ResourceState::Active)
    .metadata("owner", "0xalice")
    .domain("ethereum-mainnet")
    .build();

// Add effects
let transfer = teg.add_effect()
    .id("transfer")
    .type("transfer")
    .parameter("from", "0xalice")
    .parameter("to", "0xbob")
    .parameter("asset", "ETH")
    .parameter("amount", "1.0")
    .requires_capability("transfer_cap")
    .accesses_resource(account1)
    .domain("ethereum-mainnet")
    .build();

// Add dependencies and continuations
teg.add_effect_dependency(deposit, transfer);
teg.add_effect_continuation(transfer, confirm, Some(Condition::Success));
```

### Integration with Content Addressing

The TEG will fully integrate with our content addressing system:

```rust
impl ContentAddressed for TemporalEffectGraph {
    fn content_hash(&self) -> Result<ContentHash, ContentHashError> {
        // Implementation that generates deterministic hash
    }
    
    fn verify(&self, expected_hash: &ContentHash) -> Result<bool, ContentHashError> {
        // Verification implementation 
    }
}
```

## Rationale: Graph Representation for Algebraic Effects

The decision to use a graph-based IR stems from several key observations:

1. **Algebraic effects are inherently compositional** - They can be composed sequentially, in parallel, or with conditional branching. A graph structure naturally represents these compositions as edges between effect nodes.

2. **Resources and capabilities form relationships** - The capability system grants permissions over resources in specific ways. A graph can model these authorization relationships directly.

3. **Cross-domain operations have causal dependencies** - Operations across domains need to maintain causal ordering. A graph with causal context can enforce these constraints.

4. **Static analysis benefits from graph traversal** - Many optimization and validation passes involve graph traversal algorithms that can efficiently operate on this representation.

5. **Graph representations scale to complex programs** - As programs grow in complexity, a graph structure allows for local reasoning about subgraphs while maintaining global constraints.

6. **Category theory provides formal validation** - The TEG implements a categorical adjunction between TEL and algebraic effects, ensuring that the translation preserves semantics.

We considered alternative representations like:

- **Hierarchical AST**: Too restrictive for representing cross-cutting concerns like resource access patterns
- **Linear IR**: Doesn't capture the parallelism and conditional branching of effects adequately
- **State transition system**: Lacks explicit representation of continuations

The graph-based approach provides the right balance of expressiveness and analyzability, while aligning with the algebraic effects paradigm at the core of our system. It also mirrors how the team already describes and models cross-chain programs today.

## Consequences

### What This Enables

1. **Multi-target Code Generation**
   - The TEG enables generation of code for various targets (Ethereum, CosmWasm, etc.)
   - Effect handlers can be implemented differently per target while maintaining semantics

2. **Advanced Optimization**
   - Graph-based optimizations like dead effect elimination
   - Cross-domain operation batching and reordering
   - Resource access pattern optimization

3. **Static Verification**
   - Resource safety verification through graph analysis
   - Capability validation before execution
   - Temporal consistency checking

4. **Decoupled Evolution**
   - The TEL compiler can evolve independently of the execution engine
   - New frontends can target the TEG without affecting execution
   - Backend optimizations can be applied without affecting compilation

5. **Developer Tooling**
   - Visualization of effect chains and resource relationships
   - Debugging tools that operate on the TEG
   - Static analysis reporting

6. **Zero-Knowledge Circuit Generation**
   - The graph structure maps naturally to circuit generation for ZK proofs
   - Effects can be compiled to RISC-V for ZK-VM execution

7. **Categorical Reasoning**
   - Formal verification of translations using category theory
   - Property-based testing based on categorical laws
   - Composition guarantees derived from the adjunction

### Positive Consequences

1. **Cleaner Separation of Concerns**: 
   - Compilers focus on generating valid TEGs
   - Execution engines focus on running TEGs efficiently
   - Optimization passes operate on the TEG independently

2. **Enhanced Testability**:
   - The TEG can be unit tested independently of both compilation and execution
   - Test cases can be written directly against the TEG API
   - Property-based testing becomes possible at the IR level

3. **Improved Performance**:
   - Optimizations can be applied at the IR level, benefiting all frontends
   - Caching of analysis results becomes possible

4. **Mathematical Foundations**:
   - The category theory foundation provides formal verification opportunities
   - Composition laws can be derived from categorical properties
   - Semantic equivalence can be proven mathematically

### Negative Consequences

1. **Additional Layer**:
   - Introduces another translation step in the pipeline
   - May increase initial development time
   - Developers need to understand the IR model

2. **Serialization Overhead**:
   - Programs need to be serialized/deserialized between phases
   - Memory overhead for maintaining the graph structure

3. **Complexity**:
   - Graph algorithms are generally more complex than AST traversals
   - Maintaining the graph invariants requires care
   - The categorical model adds another layer of abstraction

## Implementation Strategy

### New Crate Structure

We will implement the TEG in a new `temporality-ir` crate:

```
temporality-ir/
├── src/
│   ├── graph/           # Core graph implementation
│   ├── effect_node.rs   # Effect node definition
│   ├── resource_node.rs # Resource node definition
│   ├── builder.rs       # Builder API
│   ├── validation.rs    # Validation logic
│   ├── optimization.rs  # Optimization passes
│   ├── serialization.rs # Borsh serialization
│   └── lib.rs           # Public exports
└── tests/               # Extensive test suite
```

The new dependency structure will be:

```
temporality-tel → temporality-ir ← temporality-engine → temporality-core
```

This creates a clear boundary between compilation and execution concerns.

### Migration Strategy

We will implement this in phases:

1. **Phase 1**: Create the core IR structure with minimal functionality
2. **Phase 2**: Implement the TEL → TEG translator (functor F)
3. **Phase 3**: Update the engine to consume TEG instead of directly executing TEL
4. **Phase 4**: Add optimization passes and validation
5. **Phase 5**: Extend with multi-target code generation
6. **Phase 6**: Implement the TEG → TEL translator (functor G) for bidirectional transformation

## References

- ADR-001: Rust Algebraic Effects Library
- ADR-023: Three-Layer Effect Architecture with TEL Integration
- ADR-024: Time Map and Temporal Facts
- ADR-032: Agent-Based Resource System
- System Contract: Resource and Effect sections
- IR Theory Document: Category Theoretic Model of TEG

# Addendum to ADR-034: Preserving Combinator Patterns in TEG Translation

## Context

While the Temporal Effect Graph (TEG) provides significant benefits as our intermediate representation, there's a concern that the elegant AST combinator system developed for TEL could become less useful in the translation process. The functional composition patterns that make the TEL system so powerful don't inherently map to the edge-and-node structure of a graph representation.

## Decision

Rather than treating the translation from TEL to TEG as a one-way process that abandons the combinator pattern, we will implement the category theoretical adjunction described in our IR theory document. This ensures that we maintain the investment in our combinator system while leveraging the benefits of a graph-based IR.

## Implementation Approach

We will implement a bridge between the TEL combinators and the TEG construction through a consistent pattern that implements the functors F: TEL → TEG and G: TEG → TEL:

```rust
// Functor F: TEL → TEG
pub trait ToTEGFragment {
    /// Convert a TEL combinator to a TEG fragment (implements functor F)
    fn to_teg_fragment(&self) -> TEGFragment;
}

/// A composable fragment of a Temporal Effect Graph
pub struct TEGFragment {
    effect_nodes: HashMap<EffectId, EffectNode>,
    resource_nodes: HashMap<ResourceId, ResourceNode>,
    effect_dependencies: HashMap<EffectId, Vec<EffectId>>,
    effect_continuations: HashMap<EffectId, Vec<(EffectId, Option<Condition>)>>,
    resource_relationships: HashMap<ResourceId, Vec<(ResourceId, RelationshipType)>>,
    // Additional graph components as needed
    
    // Entry and exit points for composition
    entry_points: Vec<EffectId>,
    exit_points: Vec<EffectId>,
}

impl TEGFragment {
    /// Compose this fragment with another, connecting exit points to entry points
    /// Implements the composition in the TEG category
    pub fn sequence(self, next: TEGFragment) -> TEGFragment {
        // Implementation that connects this fragment's exit points
        // to the next fragment's entry points
    }
    
    /// Compose this fragment with another in parallel
    /// Implements the monoidal structure in the TEG category
    pub fn parallel(self, other: TEGFragment) -> TEGFragment {
        // Implementation that maintains independent paths
    }
    
    /// Apply a conditional branch
    pub fn branch(
        self, 
        condition: Condition, 
        then_fragment: TEGFragment,
        else_fragment: Option<TEGFragment>
    ) -> TEGFragment {
        // Implementation that creates conditional paths
    }
}

// Functor G: TEG → TEL
pub trait ToTELCombinator {
    /// Convert a TEG fragment back to a TEL combinator (implements functor G)
    fn to_tel_combinator(&self) -> Combinator;
}

// Example implementations for existing combinators
impl ToTEGFragment for SequenceCombinator {
    fn to_teg_fragment(&self) -> TEGFragment {
        let first_fragment = self.first.to_teg_fragment();
        let second_fragment = self.second.to_teg_fragment();
        first_fragment.sequence(second_fragment)
    }
}

impl ToTEGFragment for ParallelCombinator {
    fn to_teg_fragment(&self) -> TEGFragment {
        let left_fragment = self.left.to_teg_fragment();
        let right_fragment = self.right.to_teg_fragment();
        left_fragment.parallel(right_fragment)
    }
}

impl TemporalEffectGraph {
    /// Incorporate a fragment into this graph
    pub fn incorporate(&mut self, fragment: TEGFragment) -> Result<(), GraphError> {
        // Implementation that merges the fragment into the full graph
    }
    
    /// Convert the entire graph back to a TEL combinator (implements functor G)
    pub fn to_tel_combinator(&self) -> Result<Combinator, ConversionError> {
        // Implementation that converts the graph back to a TEL combinator
    }
}
```

This approach implements the categorical adjunction by ensuring:

1. **Functorial properties**: F and G preserve identity and composition
2. **Natural isomorphism**: Hom_TEL(A, G(B)) ≅ Hom_TEG(F(A), B)
3. **Preservation of structure**: Both functors respect the monoidal structure of resources

## Example Usage

With this approach, existing TEL code can leverage the TEG without losing its compositional elegance:

```purescript
-- Using the existing TEL combinators
let transfer_op = combinators::helpers::perform("transfer", vec![
    Combinator::Ref(from.to_string()),
    Combinator::Ref(to.to_string()),
    Combinator::Ref(amount.to_string())
]);
let verify_op = combinators::helpers::perform("observe", vec![
    Combinator::Ref(balance_query.to_string())
]);
let program = Combinator::Sequence(vec![transfer_op, verify_op]);

-- Convert to a TEG (applying functor F)
let mut teg = TemporalEffectGraph::new();
let fragment = program.to_teg_fragment();
teg.incorporate(fragment)?;

-- Existing graph-style operations still work
let optimization_pass = SomeOptimization::new();
optimization_pass.run(&mut teg)?;

-- Convert back to TEL if needed (applying functor G)
let optimized_program = teg.to_tel_combinator()?;
```

## Benefits

This approach offers significant benefits:

1. **Mathematical foundation**: The adjunction provides formal guarantees about the translation
2. **Progressive adoption**: Teams can gradually migrate to graph-based analysis without rewriting combinator code
3. **Best of both worlds**: We get the elegance of functional composition plus the power of graph analysis
4. **Separation of concerns**: Combinator composition can focus on semantics while graph traversals focus on analysis
5. **Tooling integration**: Existing TEL tools can be extended to visualize or analyze the resulting TEG fragments
6. **Bidirectional transformation**: Changes in either representation can be propagated to the other
7. **Semantic preservation**: The adjunction ensures that meaning is preserved in both directions

## Considerations

The main implementation consideration is ensuring that the functors F and G satisfy the adjunction property. This requires careful design of the fragment composition rules and the conversion back to combinators.

The translation layer adds a small amount of overhead compared to direct graph construction, but the benefits of maintaining the combinator pattern far outweigh this cost.

## Conclusion

By implementing the categorical adjunction between TEL and TEG, we maintain the elegant functional composition patterns that make TEL powerful, while still gaining the analytical benefits of a graph-based intermediate representation. 

Rather than losing value in the translation, our combinator system becomes more powerful, gaining the ability to produce both AST nodes and graph fragments while maintaining its compositional semantics, all with the formal guarantees provided by category theory.