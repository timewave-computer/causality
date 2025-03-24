# ADR-012: AST and Resource Graph Correspondence

## Status

Accepted

## Implementation Status

Implemented. The AST-resource graph correspondence system has been fully implemented in `/src/ast/resource_graph.rs` with integration in the resource allocation system through `/src/resource/static_alloc.rs`. The implementation includes all core components described in the decision:

1. **AST Node Tagging**: Implemented in `AstNodeId` and `AstNodeType` structures
2. **Resource Attribution**: Implemented via `AttributedResourceGrant` with source node references
3. **Correlation Tracking**: Fully implemented in `CorrelationTracker` and `GraphCorrelation` classes
4. **Divergence Analysis**: Implemented with `DivergencePoint` and `DivergenceType` structures
5. **Resource Delta Tracking**: Implemented via the `Delta` structure with support for conservation validation
6. **Controller Transition Tracking**: Implemented with `ControllerTransition` structure
7. **Visual Debugging**: Initial support via DOT graph generation in `graph_to_dot` function

The implementation closely follows the specifications in the ADR and includes a comprehensive test suite that verifies the core functionality. The resource allocation system has been extended to optionally accept an `AstContext` with allocations, enabling bidirectional tracking between AST nodes and resource grants.

## Context

In our content-addressable execution system implemented in Rust, we're implementing two conceptually different but related graphs:

1. **Abstract Syntax Tree (AST)** - The static representation of program structure derived from source code
2. **Resource Relationship Graph** - The dynamic graph of resource allocations and hierarchical relationships that emerges during execution

These two graphs model different aspects of program behavior, yet they interact in ways that affect debugging, optimization, and observability. We need to understand this relationship to build effective developer tooling and ensure efficient resource utilization.

## Problem

The relationship between a program's AST and its resource allocation pattern is complex and not well-defined in our current Rust architecture. This creates several challenges:

1. **Debugging Complexity**: When programs behave unexpectedly, developers must mentally map between syntactic structure and runtime resource behavior with limited tooling support.

2. **Resource Attribution**: It's difficult to attribute resource consumption to specific parts of the program, making optimization challenging.

3. **Execution Visualization**: Our time-travel debugging tools need a coherent way to visualize both program structure and resource flow.

4. **Effect Tracing**: When effects propagate through the system, the path they take through the resource graph may diverge significantly from what's suggested by the AST.

5. **Resource Balancing**: With our formal resource model we need to track which AST nodes create and consume resources to verify delta calculations.

6. **Controller Tracing**: For cross-domain resources, we need to track how controller labels evolve through the execution graph.

## Decision

We will explicitly model and track the relationship between AST nodes and resource allocations by implementing a **bidirectional mapping system** in Rust with the following components:

1. **AST Node Tagging**: Add unique identifiers to AST nodes during parsing/compilation.

2. **Resource Attribution**: Enhance the `ResourceGrant` type to include a reference to the AST node responsible for the allocation and track formalized resource properties:

```rust
#[derive(Debug, Clone)]
pub struct ResourceGrant {
    // existing fields
    pub grant_id: GrantId,
    pub memory_bytes: usize,
    pub cpu_millis: usize,
    pub io_operations: usize,
    pub effect_count: usize,
    
    // New fields for AST correspondence
    pub source_ast_node_id: Option<AstNodeId>,
    pub source_location: Option<SourceLocation>,
    pub resource: Resource,
    pub controller_label: Option<ControllerLabel>,
    pub consumption_delta: Delta,
}
```

3. **Graph Correlation API**: Implement an API for analyzing the relationship between AST and resource graphs:

```rust
#[derive(Debug)]
pub struct GraphCorrelation {
    pub ast_to_resources: HashMap<AstNodeId, Vec<ResourceGrantId>>,
    pub resource_to_ast: HashMap<ResourceGrantId, AstNodeId>,
    pub divergence_points: Vec<DivergencePoint>,
    pub resource_deltas: HashMap<AstNodeId, Delta>,
    pub controller_transitions: Vec<ControllerTransition>,
}
```

4. **Divergence Analysis**: Create algorithms to identify points where the AST and resource graph structures diverge significantly, highlighting potential areas of interest for optimization or debugging.

5. **Unified Visualization**: Develop a visualization approach that can render both graphs and their relationships, making it easier to understand program behavior holistically.

6. **Resource Delta Tracking**: Add mechanisms to track resource deltas by AST node to ensure conservation laws are maintained.

7. **Controller Transition Tracking**: Monitor how controller labels transform as resources move through the execution graph.

## Implementation Strategy

### 1. AST Node Tagging

During parsing and AST construction, we'll add:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AstNodeId(String);

impl AstNodeId {
    /// Create a new AST node ID from a string
    pub fn new(id: String) -> Self {
        AstNodeId(id)
    }
    
    /// Get the string representation of this AST node ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstNodeType {
    Program,
    Function,
    Sequence,
    Parallel,
    Conditional,
    Loop,
    Effect,
    ResourceOperation,
    ControllerOperation,
    // Other node types
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub node_id: AstNodeId,
    pub node_type: AstNodeType,
    pub source_location: SourceLocation,
    pub children: Vec<AstNode>,
    // Other node-specific fields
}

impl AstNode {
    pub fn new(node_type: AstNodeType, source_location: SourceLocation) -> Self {
        AstNode {
            node_id: AstNodeId::new(Uuid::new_v4().to_string()),
            node_type,
            source_location,
            children: Vec::new(),
        }
    }
    
    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }
}
```

This provides a stable identifier we can reference throughout execution.

### 2. Resource Allocation Instrumentation

Modify the resource allocator to capture the source of allocation requests and track formal resource properties:

```rust
#[derive(Debug, Clone)]
pub struct AstContext {
    pub ast_node_id: AstNodeId,
    pub source_location: Option<SourceLocation>,
    pub execution_path: Vec<AstNodeId>,
}

#[async_trait]
pub trait ResourceAllocator: Send + Sync {
    /// Allocate resources
    async fn allocate(&self, request: &ResourceRequest) -> Result<ResourceGrant>;
    
    /// Allocate resources with AST context
    async fn allocate_with_context(&self, request: &ResourceRequest, context: &AstContext) -> Result<ResourceGrant>;
    
    /// Release resources
    fn release(&self, grant: &ResourceGrant) -> Result<()>;
    
    /// Check usage
    fn check_usage(&self, grant: &ResourceGrant) -> ResourceUsage;
    
    /// Subdivide resources
    async fn subdivide(&self, grant: ResourceGrant, requests: Vec<ResourceRequest>) -> Result<Vec<ResourceGrant>>;
    
    /// Subdivide with context
    async fn subdivide_with_context(
        &self,
        grant: ResourceGrant,
        requests: Vec<(ResourceRequest, AstContext)>
    ) -> Result<Vec<ResourceGrant>>;
    
    /// Validate grant
    fn validate_grant(&self, grant: &ResourceGrant) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ResourceRequest {
    pub memory_bytes: usize,
    pub cpu_millis: usize,
    pub io_operations: usize,
    pub effect_count: usize,
    
    // New fields for formalized resources
    pub resource_type: ResourceType,
    pub resource_logic: Logic,
    pub fungibility_domain: Label,
    pub quantity: Quantity,
    pub metadata: Value,
    pub controller_label: Option<ControllerLabel>,
}
```

### 3. Correlation Tracking

Implement dedicated data structures for tracking the relationship:

```rust
#[derive(Debug)]
pub struct CorrelationTracker {
    // Maps AST node IDs to resource grants
    ast_to_resources: RwLock<HashMap<AstNodeId, HashSet<GrantId>>>,
    
    // Maps resource grant IDs to AST nodes
    resource_to_ast: RwLock<HashMap<GrantId, AstNodeId>>,
    
    // Maps AST nodes to resource deltas
    delta_map: RwLock<HashMap<AstNodeId, Delta>>,
    
    // Tracks controller transitions
    controller_transitions: RwLock<Vec<ControllerTransition>>,
    
    // Tracks divergence points
    divergence_points: RwLock<Vec<DivergencePoint>>,
}

impl CorrelationTracker {
    pub fn new() -> Self {
        CorrelationTracker {
            ast_to_resources: RwLock::new(HashMap::new()),
            resource_to_ast: RwLock::new(HashMap::new()),
            delta_map: RwLock::new(HashMap::new()),
            controller_transitions: RwLock::new(Vec::new()),
            divergence_points: RwLock::new(Vec::new()),
        }
    }
    
    /// Record a resource allocation by an AST node
    pub fn record_allocation(
        &self,
        ast_node_id: AstNodeId,
        grant_id: GrantId,
        resource: &Resource
    ) -> Result<()> {
        // Update ast_to_resources map
        let mut ast_to_resources = self.ast_to_resources.write().unwrap();
        ast_to_resources.entry(ast_node_id.clone())
            .or_insert_with(HashSet::new)
            .insert(grant_id.clone());
        
        // Update resource_to_ast map
        let mut resource_to_ast = self.resource_to_ast.write().unwrap();
        resource_to_ast.insert(grant_id, ast_node_id);
        
        Ok(())
    }
    
    /// Record a resource delta for an AST node
    pub fn record_delta(&self, ast_node_id: AstNodeId, delta: Delta) -> Result<()> {
        let mut delta_map = self.delta_map.write().unwrap();
        delta_map.insert(ast_node_id, delta);
        Ok(())
    }
    
    /// Record a controller transition
    pub fn record_controller_transition(
        &self,
        resource_id: ResourceGrantId, 
        source_controller: ControllerID,
        target_controller: ControllerID,
        ast_node_id: AstNodeId
    ) -> Result<()> {
        let transition = ControllerTransition {
            resource_id,
            source_controller,
            target_controller,
            ast_node_id,
        };
        
        let mut transitions = self.controller_transitions.write().unwrap();
        transitions.push(transition);
        
        Ok(())
    }
    
    /// Get all resources allocated by a given AST node
    pub fn resources_for_ast_node(&self, ast_node_id: &AstNodeId) -> HashSet<GrantId> {
        let ast_to_resources = self.ast_to_resources.read().unwrap();
        match ast_to_resources.get(ast_node_id) {
            Some(resources) => resources.clone(),
            None => HashSet::new(),
        }
    }
    
    /// Find the AST node responsible for a resource allocation
    pub fn ast_node_for_resource(&self, grant_id: &GrantId) -> Option<AstNodeId> {
        let resource_to_ast = self.resource_to_ast.read().unwrap();
        resource_to_ast.get(grant_id).cloned()
    }
    
    /// Compute total delta for a subtree
    pub fn subtree_delta(&self, ast_node_id: &AstNodeId) -> Delta {
        // Implementation would need to walk the AST
        // and sum deltas for all child nodes
        unimplemented!()
    }
}
```

### 4. Divergence Analysis

Implement metrics and algorithms to quantify structural differences:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DivergenceType {
    LoopUnrolling,         // A loop in AST becomes multiple allocations
    ConcurrentExecution,   // A single node forks into parallel branches
    HigherOrderDivergence, // Function passed to another context
    EffectHandlerJump,     // Effect handler causes non-local execution
    ResourceResharing,     // Resources reallocated to different AST nodes
    ControllerTransition,  // Resource moves between controllers
}

#[derive(Debug, Clone)]
pub struct DivergencePoint {
    pub ast_node: AstNodeId,
    pub resource_nodes: Vec<ResourceGrantId>,
    pub divergence_type: DivergenceType,
    pub divergence_magnitude: f32,
    pub delta_imbalance: Option<Delta>,
}

#[derive(Debug, Clone)]
pub struct ControllerTransition {
    pub resource_id: ResourceGrantId,
    pub source_controller: ControllerID,
    pub target_controller: ControllerID,
    pub ast_node: AstNodeId,
}

impl CorrelationTracker {
    /// Find points where the graphs diverge significantly
    pub fn find_divergence_points(&self) -> Vec<DivergencePoint> {
        // Implementation would analyze the AST and resource graphs
        // to identify structural differences
        unimplemented!()
    }
    
    /// Find controller transitions in the resource graph
    pub fn find_controller_transitions(&self) -> Vec<ControllerTransition> {
        self.controller_transitions.read().unwrap().clone()
    }
}
```

### 5. Visual Debugging Tools

Extend our time-travel debugger to visualize both graphs:

```rust
pub mod visualization {
    use super::*;
    
    #[derive(Debug)]
    pub enum VisualizationFormat {
        Json,
        Dot,
        Ascii,
        Html,
    }
    
    pub struct Visualizer {
        correlation_tracker: Arc<CorrelationTracker>,
    }
    
    impl Visualizer {
        pub fn new(correlation_tracker: Arc<CorrelationTracker>) -> Self {
            Visualizer { correlation_tracker }
        }
        
        /// Generate a combined visualization
        pub fn visualize_correlation(
            &self,
            execution_context: &ExecutionContext,
            format: VisualizationFormat
        ) -> String {
            // Implementation would generate visualization in the specified format
            unimplemented!()
        }
        
        /// Highlight active AST nodes based on resource usage
        pub fn highlight_active_nodes(&self, resource_usage: &ResourceUsage) -> Vec<AstNodeId> {
            // Implementation would identify which AST nodes are active
            // based on current resource usage
            unimplemented!()
        }
        
        /// Visualize resource flow with controller transitions
        pub fn visualize_resource_flow(
            &self,
            resource_id: &ResourceGrantId,
            format: VisualizationFormat
        ) -> String {
            // Implementation would trace the flow of a resource
            // through the system, including controller transitions
            unimplemented!()
        }
        
        /// Visualize resource delta balance
        pub fn visualize_delta_balance(
            &self,
            ast_node_id: &AstNodeId,
            format: VisualizationFormat
        ) -> String {
            // Implementation would show resource deltas for a subtree
            unimplemented!()
        }
    }
}
```

### 6. Resource Delta Validation

Implement validation to ensure resource conservation across AST nodes:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Delta {
    pub amount: i64,
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, Value>,
}

impl Delta {
    pub fn new(amount: i64, resource_type: ResourceType) -> Self {
        Delta {
            amount,
            resource_type,
            metadata: HashMap::new(),
        }
    }
    
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }
}

impl CorrelationTracker {
    /// Validate resource conservation for a subtree
    pub fn validate_subtree_deltas(&self, ast_node_id: &AstNodeId) -> Result<()> {
        let delta = self.subtree_delta(ast_node_id);
        
        if !delta.is_zero() {
            Err(Error::ResourceImbalance(format!(
                "Resource delta for subtree at {:?} is not zero: {:?}",
                ast_node_id, delta
            )))
        } else {
            Ok(())
        }
    }
    
    /// Validate resource conservation for the entire program
    pub fn validate_program_deltas(&self, program: &Program) -> Result<()> {
        // Implementation would validate deltas for the entire program
        unimplemented!()
    }
}
```

## Expected Correspondence Patterns

Based on language constructs, we expect certain predictable patterns of correspondence:

1. **Sequential Code**: Direct 1:1 mapping between AST and resource graph.

2. **Conditional Branches**: AST shows all branches, resource graph shows only taken paths.

3. **Loops**: Single AST node, multiple resource nodes (one per iteration).

4. **Higher-Order Functions**: Complex many-to-many relationships as functions move between contexts.

5. **Concurrent Primitives** (`fork`, `race`, `parallel`): AST shows the operation, resource graph shows the actual parallelism.

6. **Effect Handlers**: Resource graph may show non-local jumps not evident in the AST.

7. **Controller Transitions**: Resource graph shows controller label changes as resources cross Domains.

8. **Resource Creation/Consumption**: Resource graph tracks deltas to ensure conservation across operations.

## Examples

### Example 1: Sequential Execution

```rust
let a = heavy_computation1(input); // AST Node A
let b = heavy_computation2(a);     // AST Node B
combine_results(a, b)              // AST Node C
```

Resource graph will typically match AST structure:
```
ResourceGrant(for A) → ResourceGrant(for B) → ResourceGrant(for C)
```

With delta tracking:
```
Node A: delta = 0 (creates and consumes internal resources)
Node B: delta = 0 (creates and consumes internal resources)
Node C: delta = 0 (creates and consumes internal resources)
Total program delta = 0 (conservation verified)
```

### Example 2: Loop Unrolling Divergence

```rust
for item in items {
    process_item(item);              // AST Node D (loop body)
}
```

AST shows a single loop node, but resource graph shows multiple allocations:
```
ResourceGrant(for D, item1) → ResourceGrant(for D, item2) → ResourceGrant(for D, item3)
```

This creates a 1:N relationship between the AST and resource graph, with delta tracking:
```
Node D (iteration 1): delta = 0
Node D (iteration 2): delta = 0
Node D (iteration 3): delta = 0
Total loop delta = 0 (conservation verified across all iterations)
```

### Example 3: Higher-Order Function Divergence

```rust
let processor = build_processor(config);   // AST Node E
let mapper = create_mapper(rules);         // AST Node F
run_pipeline(processor, mapper, input);    // AST Node G
```

When `run_pipeline` executes, it creates a complex resource graph where resources are allocated based on AST nodes from multiple sources:

```
ResourceGrant(for G) → ResourceGrant(for E, inside G) → ResourceGrant(for F, inside E)
```

This creates M:N relationships that are difficult to visualize without explicit tracking.

### Example 4: Cross-domain Resource Transfer

```rust
// Transfer token from Ethereum to Solana
transfer_cross_domain(token, amount); // AST Node H
```

The resource graph would show:
```
ResourceGrant(for H, on Ethereum) → ResourceGrant(for H, on Solana)
```

With controller tracking:
```
ControllerTransition {
  resource_id: "token123",
  source_controller: EthereumController,
  target_controller: SolanaController,
  ast_node: NodeH
}
```

And delta validation:
```
Node H (on Ethereum): delta = -amount
Node H (on Solana): delta = +amount
Total transfer delta = 0 (conservation verified across Domains)
```

## Integration with Existing Code

The implementation will integrate with:

1. **AST Module**: Add the `AstNodeId` and related tracking in `src/ast.rs`
2. **Resource Allocation**: Enhance the `ResourceAllocator` trait in `src/resource/allocator.rs`
3. **Resource Grants**: Add AST tracking to `ResourceGrant` in `src/resource/request.rs`
4. **Correlation Tracking**: Implement in `src/ast/resource_graph.rs`
5. **Visualization**: Add to debugging tools in a new module

## Benefits

1. **Improved Debugging**: Developers can see which parts of their code are consuming resources.

2. **Better Performance Analysis**: Identify code structures that cause unexpected resource patterns.

3. **Enhanced Observability**: Monitor resource utilization with context about the responsible code.

4. **Optimized JIT**: Target optimization efforts at high-impact AST nodes that consume disproportionate resources.

5. **Effect Transparency**: Make non-local control flow from effects more visible and understandable.

6. **Resource Conservation Verification**: Ensure that resources are conserved across program execution.

7. **Controller Transition Tracking**: Identify and monitor cross-domain resource transfers.

## Drawbacks

1. **Execution Overhead**: Tracking this correspondence adds some runtime overhead.

2. **Implementation Complexity**: Maintaining bidirectional mapping adds complexity.

3. **Storage Requirements**: Additional metadata increases memory usage.

4. **Visualization Challenges**: Representing two interrelated graphs is non-trivial.

5. **Delta Computation Overhead**: Calculating and validating resource deltas adds computational cost.

## Alternatives Considered

### 1. Implicit Correlation Only

We could rely on execution order alone to correlate AST nodes with resource allocations, without explicit tracking.

**Rejected because**: This approach breaks down with concurrent execution, higher-order functions, and effect handlers. The correlation becomes too tenuous to be useful.

### 2. Enhanced AST Only

We could enhance the AST to include resource estimates and usage patterns directly, merging the concepts.

**Rejected because**: This conflates static and dynamic properties, making the AST less reusable and more complex. It also doesn't handle runtime adaptations well.

### 3. Sampling-Based Approach

We could use statistical sampling to infer correlations without comprehensive tracking.

**Rejected because**: While efficient, this provides incomplete information that may miss critical patterns, especially in non-deterministic executions.

### 4. Separate Resource Tracking

We could keep resource formalization entirely separate from AST tracking.

**Rejected because**: This would miss the opportunity to link resource conservation to program structure, making it harder to identify which parts of the program are causing resource imbalances.

## Implementation Plan

1. **Phase 1**: Implement AST node tagging and basic resource attribution (2 weeks)
   - Add `AstNodeId` and related structures to AST module
   - Extend `ResourceGrant` with AST node references
   - Modify `ResourceAllocator` to accept AST context

2. **Phase 2**: Build correlation tracking infrastructure (2 weeks)
   - Implement `CorrelationTracker` with necessary data structures
   - Add methods for recording allocations and retrieving correlations
   - Create basic testing infrastructure

3. **Phase 3**: Develop divergence analysis algorithms (3 weeks)
   - Implement algorithms to detect AST-resource graph divergence
   - Create data structures for divergence points
   - Build test cases for different divergence patterns

4. **Phase 4**: Integrate resource formalization and delta tracking (2 weeks)
   - Add delta tracking to correlation tracker
   - Implement delta calculation and validation
   - Create tests for resource conservation

5. **Phase 5**: Implement controller transition tracking (2 weeks)
   - Add controller transition recording
   - Implement cross-domain resource tracking
   - Create validation tests for controller transitions

6. **Phase 6**: Create visualization tools for the debugger (3 weeks)
   - Implement visualization formats (JSON, DOT, ASCII, HTML)
   - Create resource flow visualization
   - Build delta balance visualization

7. **Phase 7**: Optimization and performance tuning (2 weeks)
   - Identify and optimize performance bottlenecks
   - Reduce memory overhead of tracking structures
   - Improve concurrency handling

## Open Questions

1. **Granularity**: At what level of AST granularity should we track correlation? Expression, statement, function?
2. **Persistence**: Should correlation data be persisted as part of execution traces or computed on demand?
3. **Concurrent Execution**: How do we handle attribution when resources are shared across concurrent branches?
4. **Effect System Integration**: What additional metadata is needed to properly track effect handling?
5. **Resource Reuse**: How do we handle cases where resources are recycled and reused by different AST nodes?
6. **Delta Precision**: How precise should delta tracking be for complex operations? Should we use exact or approximate tracking?
7. **Controller Validation**: How do we validate controller transitions when resources cross Domains?
8. **Performance Impact**: What is the performance impact of tracking resource deltas and controller transitions?

## Conclusion

By explicitly modeling the correspondence between AST structure and resource allocation patterns in Rust, we can provide developers with deeper insights into program behavior, especially for complex scenarios involving concurrency, higher-order functions, and effects. This approach bridges the gap between static program understanding and dynamic execution behavior, making our content-addressable execution system more transparent and debuggable.

With the integration of resource formalization, we gain additional capabilities to track resource conservation and controller transitions, enhancing our ability to reason about cross-domain operations and ensure resource integrity throughout the system. 