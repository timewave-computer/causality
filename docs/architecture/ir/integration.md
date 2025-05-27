# TEG Integration Guide for External Systems

This document provides guidance for integrating the Temporal Effect Graph (TEG) with external systems, including visualizers, analyzers, and execution engines.

## Interface Specifications

### Core API Interface

The primary interface for external systems is the `TemporalEffectGraph` API, which provides access to the graph structure, nodes, and relationships:

```rust
// Main entry points for external systems
pub struct TemporalEffectGraph {
    // Core methods for external consumption
    pub fn effect_nodes(&self) -> impl Iterator<Item = &EffectNode>;
    pub fn resource_nodes(&self) -> impl Iterator<Item = &ResourceNode>;
    pub fn get_effect_node(&self, id: &EffectId) -> Option<&EffectNode>;
    pub fn get_resource_node(&self, id: &ResourceId) -> Option<&ResourceNode>;
    pub fn get_effect_dependencies(&self, id: &EffectId) -> impl Iterator<Item = &EffectId>;
    pub fn get_effect_continuations(&self, id: &EffectId) -> impl Iterator<Item = (&EffectId, &Option<Condition>)>;
    
    // Serialization methods
    pub fn serialize(&self) -> Result<Vec<u8>>;
    pub fn deserialize(data: &[u8]) -> Result<Self>;
    pub fn to_json(&self) -> Result<serde_json::Value>;
    pub fn from_json(json: &serde_json::Value) -> Result<Self>;
    
    // Content addressing
    pub fn content_hash(&self) -> ContentHash;
    pub fn verify_hash(&self, hash: &ContentHash) -> bool;
}
```

### Serialization Format

The TEG serialization format is based on Borsh and includes:

- Effect nodes with their metadata, parameters, and relationships
- Resource nodes with their state and metadata
- Edge relationships and dependencies
- Content addressing information

JSON serialization is also available for systems that prefer to work with JSON.

## Example Usage Patterns

### Loading and Analyzing a TEG

```rust
// Load a TEG from serialized data
let serialized_teg = get_serialized_teg_from_somewhere();
let teg = TemporalEffectGraph::deserialize(&serialized_teg)?;

// Basic graph analysis
let effect_count = teg.effect_nodes().count();
let resource_count = teg.resource_nodes().count();
println!("TEG contains {} effects and {} resources", effect_count, resource_count);

// Find all log effects
let log_effects: Vec<_> = teg.effect_nodes()
    .filter(|node| node.effect_type() == "log")
    .collect();
println!("Found {} log effects", log_effects.len());

// Analyze effect dependencies
for effect in teg.effect_nodes() {
    let dependencies = teg.get_effect_dependencies(&effect.id()).count();
    println!("Effect {} has {} dependencies", effect.id(), dependencies);
}
```

### Executing a TEG

```rust
// Create an executor with the appropriate configuration
let executor = TegExecutor::new(config);

// Execute the TEG
let result = executor.execute(&teg)?;

// Extract results
println!("Execution result: {:?}", result);
```

### Visualizing a TEG

```rust
// Convert to a format suitable for visualization
let dot_graph = teg_to_dot(&teg);
write_to_file("graph.dot", &dot_graph)?;

// Or use the JSON representation for web-based visualization
let json = teg.to_json()?;
send_to_visualization_tool(json);
```

## Extension Mechanisms

### Custom Effect Types

You can extend the TEG with custom effect types by implementing the appropriate traits:

```rust
pub trait CustomEffectType {
    fn effect_type(&self) -> &str;
    fn parameters_schema(&self) -> Schema;
    fn return_type_schema(&self) -> Schema;
    fn validate_parameters(&self, params: &HashMap<String, Value>) -> Result<()>;
}

// Register your custom effect type with the system
teg_registry.register_effect_type(MyCustomEffect::new());
```

### Custom Resource Types

Similarly, custom resource types can be defined:

```rust
pub trait CustomResourceType {
    fn resource_type(&self) -> &str;
    fn state_schema(&self) -> Schema;
    fn validate_state(&self, state: &Value) -> Result<()>;
    fn initial_state(&self) -> Value;
}

// Register your custom resource type
teg_registry.register_resource_type(MyCustomResource::new());
```

### Custom Transformations

You can implement custom graph transformations:

```rust
pub trait Transformation {
    fn transform(&self, teg: &mut TemporalEffectGraph) -> Result<()>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

// Apply the transformation
let my_transformation = MyCustomTransformation::new();
my_transformation.transform(&mut teg)?;
```

## Integration Guidelines

### Best Practices

1. **Content Addressing**: Always maintain content addressing when modifying a TEG to ensure semantic integrity
2. **Incremental Updates**: Use the diff and patch operations for incremental updates rather than full serialization
3. **Resource Safety**: Ensure resource safety by validating that all resource operations are properly ordered
4. **Effect Capabilities**: Verify that effects have the necessary capabilities before execution
5. **Error Handling**: Implement proper error handling using the Result type for all operations

### Performance Considerations

1. For large graphs (>1000 nodes), use pagination when querying the graph
2. Consider using the indexed access methods rather than iterating over all nodes
3. For frequent updates, use the transaction-based API to batch modifications
4. Cache content hashes when performing frequent verification operations
5. Use the compact binary serialization format for storage and transmission

### Security Considerations

1. Validate all input TEGs before execution to prevent malicious graphs
2. Check resource access patterns to prevent unauthorized access
3. Verify effect capability requirements against available capabilities
4. Use content addressing to ensure graph integrity
5. Implement rate limiting for resource-intensive operations

## GraphQL API

For web-based integrations, a GraphQL API is provided:

```graphql
type EffectNode {
  id: ID!
  effectType: String!
  parameters: JSON!
  dependencies: [EffectNode!]!
  continuations: [EffectContinuation!]!
  metadata: JSON!
  contentHash: String!
}

type ResourceNode {
  id: ID!
  resourceType: String!
  state: JSON!
  metadata: JSON!
  contentHash: String!
}

type TemporalEffectGraph {
  effectNodes: [EffectNode!]!
  resourceNodes: [ResourceNode!]!
  metadata: JSON!
  contentHash: String!
}

type Query {
  getGraph(id: ID!): TemporalEffectGraph
  getEffectNode(id: ID!): EffectNode
  getResourceNode(id: ID!): ResourceNode
  searchEffects(type: String, limit: Int, offset: Int): [EffectNode!]!
  searchResources(type: String, limit: Int, offset: Int): [ResourceNode!]!
}

type Mutation {
  executeGraph(id: ID!): ExecutionResult!
  optimizeGraph(id: ID!, optimizations: [String!]!): TemporalEffectGraph!
  transformGraph(id: ID!, transformation: String!): TemporalEffectGraph!
}
```

## Database Integration

For database storage, the following schema is recommended:

```sql
CREATE TABLE teg_graphs (
  id TEXT PRIMARY KEY,
  content_hash TEXT NOT NULL,
  serialized_data BLOB NOT NULL,
  metadata JSONB,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE teg_effect_nodes (
  id TEXT PRIMARY KEY,
  graph_id TEXT REFERENCES teg_graphs(id),
  effect_type TEXT NOT NULL,
  parameters JSONB NOT NULL,
  content_hash TEXT NOT NULL,
  metadata JSONB,
  INDEX idx_graph_effect (graph_id, effect_type)
);

CREATE TABLE teg_resource_nodes (
  id TEXT PRIMARY KEY,
  graph_id TEXT REFERENCES teg_graphs(id),
  resource_type TEXT NOT NULL,
  state JSONB NOT NULL,
  content_hash TEXT NOT NULL,
  metadata JSONB,
  INDEX idx_graph_resource (graph_id, resource_type)
);

CREATE TABLE teg_relationships (
  source_id TEXT NOT NULL,
  target_id TEXT NOT NULL,
  relationship_type TEXT NOT NULL,
  metadata JSONB,
  PRIMARY KEY (source_id, target_id, relationship_type)
);
```

## Further Reading

- [TEG Category Theory](./adjunction.md) - Mathematical foundation of the TEG
- [Optimization Guide](./optimization.md) - Details on available optimizations
- [Content Addressing](./content_addressing.md) - How content addressing ensures semantic integrity
- [API Reference](./api_reference.md) - Complete API documentation 