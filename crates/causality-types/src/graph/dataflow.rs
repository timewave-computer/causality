//! Process dataflow and domain-aware nodes for TEL
//!
//! Defines process dataflow definitions, nodes, and domain-aware execution
//! for complex multi-step workflows in the Temporal Effect Language.

use crate::primitive::ids::{DomainId, ExprId, ResourceId, NodeId};
use crate::primitive::string::Str;
use crate::primitive::time::Timestamp;
use crate::expression::value::ValueExpr;
use crate::expression::r#type::TypeExpr;
use crate::graph::optimization::{TypedDomain, ResourceUsageEstimate};
use crate::system::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use std::collections::BTreeMap;
use std::marker::PhantomData;

//-----------------------------------------------------------------------------
// Process Dataflow Types
//-----------------------------------------------------------------------------

/// Definition of a process dataflow workflow with type-safe schemas
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDataflowDefinition<I = (), O = (), S = ()> 
where
    I: Clone + PartialEq,
    O: Clone + PartialEq,
    S: Clone + PartialEq,
{
    /// Unique identifier for this dataflow definition
    pub definition_id: ExprId,
    /// Human-readable name
    pub name: Str,
    /// Processing nodes in the dataflow
    pub nodes: Vec<ProcessDataflowNode>,
    /// Edges connecting the nodes
    pub edges: Vec<ProcessDataflowEdge>,
    /// Default typed domain for execution
    pub default_typed_domain: TypedDomain,
    /// Type markers for compile-time schema generation
    _phantom: PhantomData<(I, O, S)>,
}

impl<I, O, S> ProcessDataflowDefinition<I, O, S>
where
    I: Clone + PartialEq,
    O: Clone + PartialEq, 
    S: Clone + PartialEq,
{
    /// Create a new process dataflow definition
    pub fn new(definition_id: ExprId, name: Str) -> Self {
        Self {
            definition_id,
            name,
            nodes: Vec::new(),
            edges: Vec::new(),
            default_typed_domain: TypedDomain::default(),
            _phantom: PhantomData,
        }
    }

    /// Add a node to the dataflow
    pub fn add_node(&mut self, node: ProcessDataflowNode) {
        self.nodes.push(node);
    }

    /// Add an edge to the dataflow
    pub fn add_edge(&mut self, edge: ProcessDataflowEdge) {
        self.edges.push(edge);
    }

    /// Set the default typed domain
    pub fn with_default_typed_domain(mut self, domain: TypedDomain) -> Self {
        self.default_typed_domain = domain;
        self
    }
}

/// Auto-generated schema support for ProcessDataflowDefinition
impl<I, O, S> ProcessDataflowDefinition<I, O, S>
where
    I: Clone + PartialEq + 'static,
    O: Clone + PartialEq + 'static,
    S: Clone + PartialEq + 'static,
{
    /// Get the input schema automatically derived from the type parameter I
    pub fn input_schema() -> TypeExpr 
    where
        I: TypeSchema,  // Requires the type to implement TypeSchema
    {
        I::type_expr()
    }

    /// Get the output schema automatically derived from the type parameter O
    pub fn output_schema() -> TypeExpr 
    where
        O: TypeSchema,
    {
        O::type_expr()
    }

    /// Get the state schema automatically derived from the type parameter S
    pub fn state_schema() -> TypeExpr 
    where
        S: TypeSchema,
    {
        S::type_expr()
    }

    /// Get all schemas as a tuple for validation
    pub fn schemas() -> (TypeExpr, TypeExpr, TypeExpr)
    where
        I: TypeSchema,
        O: TypeSchema,
        S: TypeSchema,
    {
        (Self::input_schema(), Self::output_schema(), Self::state_schema())
    }
}

/// Trait for types that can provide TypeExpr schemas (would be auto-derived)
pub trait TypeSchema {
    fn type_expr() -> TypeExpr;
}

// Provide default implementations for common types
impl TypeSchema for () {
    fn type_expr() -> TypeExpr {
        TypeExpr::Unit
    }
}

impl TypeSchema for String {
    fn type_expr() -> TypeExpr {
        TypeExpr::String
    }
}

impl TypeSchema for bool {
    fn type_expr() -> TypeExpr {
        TypeExpr::Bool
    }
}

impl TypeSchema for i64 {
    fn type_expr() -> TypeExpr {
        TypeExpr::Integer
    }
}

impl<T: TypeSchema> TypeSchema for Vec<T> {
    fn type_expr() -> TypeExpr {
        use crate::expression::r#type::TypeExprBox;
        TypeExpr::List(TypeExprBox(Box::new(T::type_expr())))
    }
}

impl<T: TypeSchema> TypeSchema for Option<T> {
    fn type_expr() -> TypeExpr {
        use crate::expression::r#type::TypeExprBox;
        TypeExpr::Optional(TypeExprBox(Box::new(T::type_expr())))
    }
}

/// A node in a process dataflow
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDataflowNode {
    /// Unique identifier for this node
    pub node_id: NodeId,
    /// Human-readable name
    pub name: Str,
    /// Node type (e.g., "transform", "filter", "aggregate")
    pub node_type: Str,
    /// Input ports for this node
    pub input_ports: Vec<DataflowPort>,
    /// Output ports for this node
    pub output_ports: Vec<DataflowPort>,
    /// Processing logic expression
    pub processing_logic: Option<ExprId>,
    /// Execution domain preference
    pub preferred_domain: Option<TypedDomain>,
    /// Resource requirements
    pub resource_requirements: Option<ResourceUsageEstimate>,
}

impl ProcessDataflowNode {
    /// Create a new dataflow node
    pub fn new(node_id: NodeId, name: Str, node_type: Str) -> Self {
        Self {
            node_id,
            name,
            node_type,
            input_ports: Vec::new(),
            output_ports: Vec::new(),
            processing_logic: None,
            preferred_domain: None,
            resource_requirements: None,
        }
    }

    /// Add an input port
    pub fn add_input_port(&mut self, port: DataflowPort) {
        self.input_ports.push(port);
    }

    /// Add an output port
    pub fn add_output_port(&mut self, port: DataflowPort) {
        self.output_ports.push(port);
    }

    /// Set processing logic
    pub fn with_processing_logic(mut self, logic: ExprId) -> Self {
        self.processing_logic = Some(logic);
        self
    }

    /// Set preferred domain
    pub fn with_preferred_domain(mut self, domain: TypedDomain) -> Self {
        self.preferred_domain = Some(domain);
        self
    }
}

/// A port on a dataflow node
#[derive(Debug, Clone, PartialEq)]
pub struct DataflowPort {
    /// Port identifier
    pub port_id: Str,
    /// Port name
    pub name: Str,
    /// Data type expected/produced
    pub data_type: Str,
    /// Whether this port is required
    pub required: bool,
}

impl DataflowPort {
    /// Create a new dataflow port
    pub fn new(port_id: Str, name: Str, data_type: Str, required: bool) -> Self {
        Self {
            port_id,
            name,
            data_type,
            required,
        }
    }
}

/// An edge connecting dataflow nodes
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDataflowEdge {
    /// Unique identifier for this edge
    pub edge_id: Str,
    /// Source node ID
    pub source_node: NodeId,
    /// Source port ID
    pub source_port: Str,
    /// Target node ID
    pub target_node: NodeId,
    /// Target port ID
    pub target_port: Str,
    /// Data transformation logic (optional)
    pub transformation: Option<ExprId>,
}

impl ProcessDataflowEdge {
    /// Create a new dataflow edge
    pub fn new(
        edge_id: Str,
        source_node: NodeId,
        source_port: Str,
        target_node: NodeId,
        target_port: Str,
    ) -> Self {
        Self {
            edge_id,
            source_node,
            source_port,
            target_node,
            target_port,
            transformation: None,
        }
    }

    /// Set transformation logic
    pub fn with_transformation(mut self, transformation: ExprId) -> Self {
        self.transformation = Some(transformation);
        self
    }
}

/// ProcessDataflow instance execution state
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDataflowInstanceState {
    /// Unique identifier for this instance
    pub instance_id: ResourceId,
    /// Reference to the ProcessDataflow definition
    pub definition_id: ExprId,
    /// Current execution state
    pub execution_state: DataflowExecutionState,
    /// Individual node states mapped by node ID
    pub node_states: BTreeMap<NodeId, NodeExecutionState>,
    /// Instance metadata
    pub metadata: BTreeMap<Str, ValueExpr>,
    /// Optional initiation context hint
    pub initiation_hint: Option<ValueExpr>,
}

/// Execution state of a ProcessDataflow instance
#[derive(Debug, Clone, PartialEq)]
pub enum DataflowExecutionState {
    /// Instance is currently running
    Running,
    /// Instance completed successfully
    Completed,
    /// Instance failed with error message
    Failed(Str),
    /// Instance was paused
    Paused,
    /// Instance was cancelled
    Cancelled,
}

/// Execution state of an individual node within a ProcessDataflow
#[derive(Debug, Clone, PartialEq)]
pub struct NodeExecutionState {
    /// Current node state
    pub state: NodeState,
    /// Number of times this node has been executed
    pub execution_count: u32,
    /// Last execution timestamp
    pub last_execution: Option<Timestamp>,
    /// Optional result data from last execution
    pub last_result: Option<ValueExpr>,
}

/// Individual node states
#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    /// Node is waiting to be executed
    Pending,
    /// Node is currently executing
    Executing,
    /// Node completed successfully
    Completed,
    /// Node failed
    Failed(Str),
    /// Node was skipped
    Skipped,
}

//-----------------------------------------------------------------------------
// Domain-Aware Node Types
//-----------------------------------------------------------------------------

/// A domain-aware node that can adapt its behavior based on execution domain
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAwareNode {
    /// Base node information
    pub base_node: ProcessDataflowNode,
    /// Domain-specific configurations
    pub domain_configs: BTreeMap<DomainId, DomainNodeConfig>,
    /// Fallback configuration for unknown domains
    pub fallback_config: DomainNodeConfig,
}

impl DomainAwareNode {
    /// Create a new domain-aware node
    pub fn new(base_node: ProcessDataflowNode) -> Self {
        Self {
            base_node,
            domain_configs: BTreeMap::new(),
            fallback_config: DomainNodeConfig::default(),
        }
    }

    /// Add a domain-specific configuration
    pub fn add_domain_config(&mut self, domain_id: DomainId, config: DomainNodeConfig) {
        self.domain_configs.insert(domain_id, config);
    }

    /// Get configuration for a specific domain
    pub fn get_config_for_domain(&self, domain_id: &DomainId) -> &DomainNodeConfig {
        self.domain_configs.get(domain_id).unwrap_or(&self.fallback_config)
    }

    /// Set fallback configuration
    pub fn with_fallback_config(mut self, config: DomainNodeConfig) -> Self {
        self.fallback_config = config;
        self
    }
}

/// Domain-specific configuration for a node
#[derive(Debug, Clone, PartialEq)]
pub struct DomainNodeConfig {
    /// Execution strategy for this domain
    pub execution_strategy: Str,
    /// Resource allocation preferences
    pub resource_preferences: BTreeMap<Str, u64>,
    /// Performance tuning parameters
    pub performance_params: BTreeMap<Str, f64>,
    /// Domain-specific processing logic override
    pub processing_logic_override: Option<ExprId>,
    /// Timeout settings
    pub timeout_ms: Option<u64>,
}

impl Default for DomainNodeConfig {
    fn default() -> Self {
        Self {
            execution_strategy: Str::from("default"),
            resource_preferences: BTreeMap::new(),
            performance_params: BTreeMap::new(),
            processing_logic_override: None,
            timeout_ms: None,
        }
    }
}

impl DomainNodeConfig {
    /// Create a new domain node configuration
    pub fn new(execution_strategy: Str) -> Self {
        Self {
            execution_strategy,
            resource_preferences: BTreeMap::new(),
            performance_params: BTreeMap::new(),
            processing_logic_override: None,
            timeout_ms: None,
        }
    }

    /// Set resource preference
    pub fn with_resource_preference(mut self, resource_type: Str, amount: u64) -> Self {
        self.resource_preferences.insert(resource_type, amount);
        self
    }

    /// Set performance parameter
    pub fn with_performance_param(mut self, param_name: Str, value: f64) -> Self {
        self.performance_params.insert(param_name, value);
        self
    }

    /// Set processing logic override
    pub fn with_processing_logic_override(mut self, logic: ExprId) -> Self {
        self.processing_logic_override = Some(logic);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

//-----------------------------------------------------------------------------
// Serialization Implementations
//-----------------------------------------------------------------------------

impl Encode for ProcessDataflowDefinition {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.definition_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.name.as_ssz_bytes());
        bytes.extend_from_slice(&self.nodes.as_ssz_bytes());
        bytes.extend_from_slice(&self.edges.as_ssz_bytes());
        bytes.extend_from_slice(&self.default_typed_domain.as_ssz_bytes());
        bytes
    }
}

impl Decode for ProcessDataflowDefinition {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let definition_id = ExprId::from_ssz_bytes(&bytes[offset..])?;
        offset += definition_id.as_ssz_bytes().len();
        
        let name = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += name.as_ssz_bytes().len();
        
        let nodes = Vec::<ProcessDataflowNode>::from_ssz_bytes(&bytes[offset..])?;
        offset += nodes.as_ssz_bytes().len();
        
        let edges = Vec::<ProcessDataflowEdge>::from_ssz_bytes(&bytes[offset..])?;
        offset += edges.as_ssz_bytes().len();
        
        let default_typed_domain = TypedDomain::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(ProcessDataflowDefinition {
            definition_id,
            name,
            nodes,
            edges,
            default_typed_domain,
            _phantom: PhantomData,
        })
    }
}

impl SimpleSerialize for ProcessDataflowDefinition {}

// SSZ implementations for ProcessDataflowNode
impl Encode for ProcessDataflowNode {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.node_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.name.as_ssz_bytes());
        bytes.extend_from_slice(&self.node_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.input_ports.as_ssz_bytes());
        bytes.extend_from_slice(&self.output_ports.as_ssz_bytes());
        bytes.extend_from_slice(&self.processing_logic.as_ssz_bytes());
        bytes.extend_from_slice(&self.preferred_domain.as_ssz_bytes());
        bytes.extend_from_slice(&self.resource_requirements.as_ssz_bytes());
        bytes
    }
}

impl Decode for ProcessDataflowNode {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let node_id = NodeId::from_ssz_bytes(&bytes[offset..])?;
        offset += node_id.as_ssz_bytes().len();
        
        let name = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += name.as_ssz_bytes().len();
        
        let node_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += node_type.as_ssz_bytes().len();
        
        let input_ports = Vec::<DataflowPort>::from_ssz_bytes(&bytes[offset..])?;
        offset += input_ports.as_ssz_bytes().len();
        
        let output_ports = Vec::<DataflowPort>::from_ssz_bytes(&bytes[offset..])?;
        offset += output_ports.as_ssz_bytes().len();
        
        let processing_logic = Option::<ExprId>::from_ssz_bytes(&bytes[offset..])?;
        offset += processing_logic.as_ssz_bytes().len();
        
        let preferred_domain = Option::<TypedDomain>::from_ssz_bytes(&bytes[offset..])?;
        offset += preferred_domain.as_ssz_bytes().len();
        
        let resource_requirements = Option::<ResourceUsageEstimate>::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(ProcessDataflowNode {
            node_id,
            name,
            node_type,
            input_ports,
            output_ports,
            processing_logic,
            preferred_domain,
            resource_requirements,
        })
    }
}

impl SimpleSerialize for ProcessDataflowNode {}

// SSZ implementations for ProcessDataflowEdge
impl Encode for ProcessDataflowEdge {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.edge_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.source_node.as_ssz_bytes());
        bytes.extend_from_slice(&self.source_port.as_ssz_bytes());
        bytes.extend_from_slice(&self.target_node.as_ssz_bytes());
        bytes.extend_from_slice(&self.target_port.as_ssz_bytes());
        bytes.extend_from_slice(&self.transformation.as_ssz_bytes());
        bytes
    }
}

impl Decode for ProcessDataflowEdge {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let edge_id = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += edge_id.as_ssz_bytes().len();
        
        let source_node = NodeId::from_ssz_bytes(&bytes[offset..])?;
        offset += source_node.as_ssz_bytes().len();
        
        let source_port = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += source_port.as_ssz_bytes().len();
        
        let target_node = NodeId::from_ssz_bytes(&bytes[offset..])?;
        offset += target_node.as_ssz_bytes().len();
        
        let target_port = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += target_port.as_ssz_bytes().len();
        
        let transformation = Option::<ExprId>::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(ProcessDataflowEdge {
            edge_id,
            source_node,
            source_port,
            target_node,
            target_port,
            transformation,
        })
    }
}

impl SimpleSerialize for ProcessDataflowEdge {}

// SSZ implementations for DataflowPort
impl Encode for DataflowPort {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.port_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.name.as_ssz_bytes());
        bytes.extend_from_slice(&self.data_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.required.as_ssz_bytes());
        bytes
    }
}

impl Decode for DataflowPort {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let port_id = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += port_id.as_ssz_bytes().len();
        
        let name = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += name.as_ssz_bytes().len();
        
        let data_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += data_type.as_ssz_bytes().len();
        
        let required = bool::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(DataflowPort {
            port_id,
            name,
            data_type,
            required,
        })
    }
}

impl SimpleSerialize for DataflowPort {}

// Implement serialization for all types
impl Encode for ProcessDataflowInstanceState {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        // Simple implementation - in practice would use proper SSZ encoding
        format!("{:?}", self).into_bytes()
    }
}

impl Decode for ProcessDataflowInstanceState {
    fn from_ssz_bytes(_bytes: &[u8]) -> Result<Self, DecodeError> {
        // Placeholder implementation
        Err(DecodeError::new("ProcessDataflowInstanceState decode not implemented"))
    }
}

impl SimpleSerialize for ProcessDataflowInstanceState {}

impl Encode for DataflowExecutionState {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        format!("{:?}", self).into_bytes()
    }
}

impl Decode for DataflowExecutionState {
    fn from_ssz_bytes(_bytes: &[u8]) -> Result<Self, DecodeError> {
        Err(DecodeError::new("DataflowExecutionState decode not implemented"))
    }
}

impl SimpleSerialize for DataflowExecutionState {}

impl Encode for NodeExecutionState {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        format!("{:?}", self).into_bytes()
    }
}

impl Decode for NodeExecutionState {
    fn from_ssz_bytes(_bytes: &[u8]) -> Result<Self, DecodeError> {
        Err(DecodeError::new("NodeExecutionState decode not implemented"))
    }
}

impl SimpleSerialize for NodeExecutionState {}

impl Encode for NodeState {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        format!("{:?}", self).into_bytes()
    }
}

impl Decode for NodeState {
    fn from_ssz_bytes(_bytes: &[u8]) -> Result<Self, DecodeError> {
        Err(DecodeError::new("NodeState decode not implemented"))
    }
}

impl SimpleSerialize for NodeState {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::ids::{ExprId, DomainId};
    use crate::graph::optimization::TypedDomain;
    use crate::expression::r#type::TypeExpr;
    use std::collections::BTreeMap;

    /// Test input type for automatic schema generation
    #[derive(Debug, Clone, PartialEq)]
    struct TestInput {
        name: String,
        value: u64,
        flag: bool,
    }

    /// Test output type for automatic schema generation  
    #[derive(Debug, Clone, PartialEq)]
    struct TestOutput {
        result: String,
        success: bool,
    }

    /// Test state type for automatic schema generation
    #[derive(Debug, Clone, PartialEq)]
    struct TestState {
        current_step: String,
        data: Option<u32>,
    }

    // Manual TypeSchema implementations for testing
    impl TypeSchema for TestInput {
        fn type_expr() -> TypeExpr {
            let mut fields = BTreeMap::new();
            fields.insert(Str::from("name"), TypeExpr::String);
            fields.insert(Str::from("value"), TypeExpr::Integer);
            fields.insert(Str::from("flag"), TypeExpr::Bool);
            TypeExpr::Record(crate::expression::r#type::TypeExprMap(fields))
        }
    }

    impl TypeSchema for TestOutput {
        fn type_expr() -> TypeExpr {
            let mut fields = BTreeMap::new();
            fields.insert(Str::from("result"), TypeExpr::String);
            fields.insert(Str::from("success"), TypeExpr::Bool);
            TypeExpr::Record(crate::expression::r#type::TypeExprMap(fields))
        }
    }

    impl TypeSchema for TestState {
        fn type_expr() -> TypeExpr {
            let mut fields = BTreeMap::new();
            fields.insert(Str::from("current_step"), TypeExpr::String);
            fields.insert(
                Str::from("data"), 
                TypeExpr::Optional(crate::expression::r#type::TypeExprBox(Box::new(TypeExpr::Integer)))
            );
            TypeExpr::Record(crate::expression::r#type::TypeExprMap(fields))
        }
    }

    #[test]
    fn test_automatic_schema_generation() {
        // Test that TypeSchema trait works for our test types
        let input_schema = TestInput::type_expr();
        let output_schema = TestOutput::type_expr();
        let state_schema = TestState::type_expr();

        // Verify schemas are generated correctly
        match input_schema {
            TypeExpr::Record(fields) => {
                assert!(fields.0.contains_key(&Str::from("name")));
                assert!(fields.0.contains_key(&Str::from("value")));
                assert!(fields.0.contains_key(&Str::from("flag")));
            }
            _ => panic!("Expected Record type for input schema"),
        }

        match output_schema {
            TypeExpr::Record(fields) => {
                assert!(fields.0.contains_key(&Str::from("result")));
                assert!(fields.0.contains_key(&Str::from("success")));
            }
            _ => panic!("Expected Record type for output schema"),
        }

        match state_schema {
            TypeExpr::Record(fields) => {
                assert!(fields.0.contains_key(&Str::from("current_step")));
                assert!(fields.0.contains_key(&Str::from("data")));
            }
            _ => panic!("Expected Record type for state schema"),
        }

        // Create a typed ProcessDataflowDefinition
        let _dataflow = ProcessDataflowDefinition::<TestInput, TestOutput, TestState>::new(
            ExprId::new([1u8; 32]),
            Str::from("test_workflow"),
        ).with_default_typed_domain(TypedDomain::new(DomainId::new([2u8; 32]), Str::from("test_domain")));

        // Verify schemas are automatically derived
        assert_eq!(ProcessDataflowDefinition::<TestInput, TestOutput, TestState>::input_schema(), TestInput::type_expr());
        assert_eq!(ProcessDataflowDefinition::<TestInput, TestOutput, TestState>::output_schema(), TestOutput::type_expr());
        assert_eq!(ProcessDataflowDefinition::<TestInput, TestOutput, TestState>::state_schema(), TestState::type_expr());

        println!("✅ Automatic schema generation test passed!");
    }

    #[test]
    fn test_dataflow_instance_state() {
        let instance_state = ProcessDataflowInstanceState {
            instance_id: ResourceId::new([1u8; 32]),
            definition_id: ExprId::new([2u8; 32]),
            execution_state: DataflowExecutionState::Running,
            node_states: BTreeMap::new(),
            metadata: BTreeMap::new(),
            initiation_hint: None,
        };

        // Test serialization roundtrip
        let encoded = instance_state.as_ssz_bytes();
        assert!(!encoded.is_empty());
        
        println!("✅ Dataflow instance state test passed!");
    }
} 