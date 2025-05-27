//! Process dataflow and domain-aware nodes for TEL
//!
//! Defines process dataflow definitions, nodes, and domain-aware execution
//! for complex multi-step workflows in the Temporal Effect Language.

use crate::primitive::ids::{EntityId, DomainId, ExprId, ResourceId, NodeId};
use crate::primitive::string::Str;
use crate::primitive::time::Timestamp;
use crate::expression::value::ValueExpr;
use crate::graph::optimization::{TypedDomain, ProcessDataflowInitiationHint, ResourceUsageEstimate};
use crate::system::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Process Dataflow Types
//-----------------------------------------------------------------------------

/// Definition of a process dataflow workflow
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDataflowDefinition {
    /// Unique identifier for this dataflow definition
    pub definition_id: ExprId,
    /// Human-readable name
    pub name: Str,
    /// Input schema definition
    pub input_schema: BTreeMap<Str, Str>,
    /// Output schema definition
    pub output_schema: BTreeMap<Str, Str>,
    /// State schema definition
    pub state_schema: BTreeMap<Str, Str>,
    /// Processing nodes in the dataflow
    pub nodes: Vec<ProcessDataflowNode>,
    /// Edges connecting the nodes
    pub edges: Vec<ProcessDataflowEdge>,
    /// Default typed domain for execution
    pub default_typed_domain: TypedDomain,
}

impl ProcessDataflowDefinition {
    /// Create a new process dataflow definition
    pub fn new(definition_id: ExprId, name: Str) -> Self {
        Self {
            definition_id,
            name,
            input_schema: BTreeMap::new(),
            output_schema: BTreeMap::new(),
            state_schema: BTreeMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            default_typed_domain: TypedDomain::default(),
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

/// Instance state for a running process dataflow
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDataflowInstanceState {
    /// Instance identifier
    pub instance_id: ResourceId,
    /// Definition this instance is based on
    pub definition_id: ExprId,
    /// Current execution state
    pub execution_state: DataflowExecutionState,
    /// Current data values at each node
    pub node_states: BTreeMap<NodeId, ValueExpr>,
    /// Execution metadata
    pub metadata: BTreeMap<Str, ValueExpr>,
    /// Initiation hint used for this instance
    pub initiation_hint: Option<ProcessDataflowInitiationHint>,
}

/// Execution state of a dataflow instance
#[derive(Debug, Clone, PartialEq)]
pub enum DataflowExecutionState {
    /// Dataflow is initializing
    Initializing,
    /// Dataflow is running
    Running,
    /// Dataflow is paused
    Paused,
    /// Dataflow completed successfully
    Completed,
    /// Dataflow failed with error
    Failed(Str),
    /// Dataflow was cancelled
    Cancelled,
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
        bytes.extend_from_slice(&self.input_schema.as_ssz_bytes());
        bytes.extend_from_slice(&self.output_schema.as_ssz_bytes());
        bytes.extend_from_slice(&self.state_schema.as_ssz_bytes());
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
        
        let input_schema = BTreeMap::<Str, Str>::from_ssz_bytes(&bytes[offset..])?;
        offset += input_schema.as_ssz_bytes().len();
        
        let output_schema = BTreeMap::<Str, Str>::from_ssz_bytes(&bytes[offset..])?;
        offset += output_schema.as_ssz_bytes().len();
        
        let state_schema = BTreeMap::<Str, Str>::from_ssz_bytes(&bytes[offset..])?;
        offset += state_schema.as_ssz_bytes().len();
        
        let nodes = Vec::<ProcessDataflowNode>::from_ssz_bytes(&bytes[offset..])?;
        offset += nodes.as_ssz_bytes().len();
        
        let edges = Vec::<ProcessDataflowEdge>::from_ssz_bytes(&bytes[offset..])?;
        offset += edges.as_ssz_bytes().len();
        
        let default_typed_domain = TypedDomain::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(ProcessDataflowDefinition {
            definition_id,
            name,
            input_schema,
            output_schema,
            state_schema,
            nodes,
            edges,
            default_typed_domain,
        })
    }
}

impl SimpleSerialize for ProcessDataflowDefinition {}

// TODO: Additional serialization implementations would be needed for other types
// but are omitted here for brevity. In a full implementation, all types would
// have proper Encode/Decode implementations. 