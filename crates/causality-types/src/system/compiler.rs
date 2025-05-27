//! Compiler Output Types
//!
//! This module defines the data structures produced by the Causality compiler,
//! including compiled subgraphs and temporal effect graphs (TEGs) that are
//! ready for loading into the runtime system.

use std::collections::{BTreeMap, BTreeSet};
use crate::primitive::ids::{ExprId, HandlerId, NodeId, EdgeId, SubgraphId};
use crate::effect::handler::Handler;
use crate::expression::ast::Expr as TypesExpr;

//-----------------------------------------------------------------------------
// Compiled Subgraph
//-----------------------------------------------------------------------------

/// Represents a compiled subgraph from the TEG.
/// This structure is intended for consumption by the runtime.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompiledSubgraph {
    /// Nodes within this subgraph
    pub nodes: BTreeSet<NodeId>,
    
    /// Edges connecting nodes within this subgraph or to other subgraphs
    pub edges: BTreeSet<EdgeId>,
    
    /// Entry points into this subgraph
    pub entry_points: BTreeSet<NodeId>,
    
    /// Exit points from this subgraph
    pub exit_points: BTreeSet<NodeId>,
    
    /// Metadata about this subgraph
    pub metadata: BTreeMap<String, String>,
}

impl CompiledSubgraph {
    /// Create a new empty compiled subgraph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the subgraph
    pub fn add_node(&mut self, node_id: NodeId) {
        self.nodes.insert(node_id);
    }

    /// Add an edge to the subgraph
    pub fn add_edge(&mut self, edge_id: EdgeId) {
        self.edges.insert(edge_id);
    }

    /// Mark a node as an entry point
    pub fn add_entry_point(&mut self, node_id: NodeId) {
        self.entry_points.insert(node_id);
    }

    /// Mark a node as an exit point
    pub fn add_exit_point(&mut self, node_id: NodeId) {
        self.exit_points.insert(node_id);
    }

    /// Add metadata to the subgraph
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Check if the subgraph contains a specific node
    pub fn contains_node(&self, node_id: &NodeId) -> bool {
        self.nodes.contains(node_id)
    }

    /// Check if the subgraph contains a specific edge
    pub fn contains_edge(&self, edge_id: &EdgeId) -> bool {
        self.edges.contains(edge_id)
    }

    /// Get the number of nodes in this subgraph
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges in this subgraph
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

//-----------------------------------------------------------------------------
// Compiled TEG (Temporal Effect Graph)
//-----------------------------------------------------------------------------

/// Represents a fully compiled TEG program, ready for loading into the runtime.
/// This is the primary output of the Causality compiler and contains all the
/// information needed by the runtime to execute the program.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompiledTeg {
    /// All expressions defined or referenced in the TEG program.
    /// The runtime will load these into its state manager.
    pub expressions: BTreeMap<ExprId, TypesExpr>,

    /// All handlers defined in the TEG program.
    /// The runtime will load these into its state manager.
    pub handlers: BTreeMap<HandlerId, Handler>,

    /// The structure of the graph, broken down into subgraphs.
    /// The runtime uses this to understand the overall topology.
    pub subgraphs: BTreeMap<SubgraphId, CompiledSubgraph>,

    /// Global metadata for the program
    pub metadata: CompiledTegMetadata,
}

/// Metadata associated with a compiled TEG program
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompiledTegMetadata {
    /// Program identifier
    pub program_id: Option<String>,
    
    /// Program name
    pub program_name: Option<String>,
    
    /// Program version
    pub version: Option<String>,
    
    /// Compilation timestamp
    pub compiled_at: Option<String>,
    
    /// Compiler version used
    pub compiler_version: Option<String>,
    
    /// Global configuration parameters relevant at runtime
    pub global_config: BTreeMap<String, String>,
    
    /// Dependencies required by this program
    pub dependencies: Vec<String>,
}

impl CompiledTeg {
    /// Create a new empty compiled TEG
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an expression to the compiled TEG
    pub fn add_expression(&mut self, expr_id: ExprId, expr: TypesExpr) {
        self.expressions.insert(expr_id, expr);
    }

    /// Add a handler to the compiled TEG
    pub fn add_handler(&mut self, handler_id: HandlerId, handler: Handler) {
        self.handlers.insert(handler_id, handler);
    }

    /// Add a subgraph to the compiled TEG
    pub fn add_subgraph(&mut self, subgraph_id: SubgraphId, subgraph: CompiledSubgraph) {
        self.subgraphs.insert(subgraph_id, subgraph);
    }

    /// Set the metadata for the compiled TEG
    pub fn set_metadata(&mut self, metadata: CompiledTegMetadata) {
        self.metadata = metadata;
    }

    /// Get an expression by ID
    pub fn get_expression(&self, expr_id: &ExprId) -> Option<&TypesExpr> {
        self.expressions.get(expr_id)
    }

    /// Get a handler by ID
    pub fn get_handler(&self, handler_id: &HandlerId) -> Option<&Handler> {
        self.handlers.get(handler_id)
    }

    /// Get a subgraph by ID
    pub fn get_subgraph(&self, subgraph_id: &SubgraphId) -> Option<&CompiledSubgraph> {
        self.subgraphs.get(subgraph_id)
    }

    /// Get all expression IDs
    pub fn expression_ids(&self) -> impl Iterator<Item = &ExprId> {
        self.expressions.keys()
    }

    /// Get all handler IDs
    pub fn handler_ids(&self) -> impl Iterator<Item = &HandlerId> {
        self.handlers.keys()
    }

    /// Get all subgraph IDs
    pub fn subgraph_ids(&self) -> impl Iterator<Item = &SubgraphId> {
        self.subgraphs.keys()
    }

    /// Get total number of expressions
    pub fn expression_count(&self) -> usize {
        self.expressions.len()
    }

    /// Get total number of handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// Get total number of subgraphs
    pub fn subgraph_count(&self) -> usize {
        self.subgraphs.len()
    }

    /// Validate the compiled TEG for consistency
    pub fn validate(&self) -> Result<(), CompilerValidationError> {
        // Check that all referenced expressions exist
        for handler in self.handlers.values() {
            if let Some(expr_id) = &handler.expression {
                if !self.expressions.contains_key(expr_id) {
                    return Err(CompilerValidationError::MissingExpression(*expr_id));
                }
            }
        }

        // Check that subgraphs don't reference non-existent nodes
        // (This would require more context about the node system)

        Ok(())
    }
}

impl CompiledTegMetadata {
    /// Create new metadata with basic information
    pub fn new(program_name: String, version: String) -> Self {
        Self {
            program_name: Some(program_name),
            version: Some(version),
            ..Default::default()
        }
    }

    /// Set the program ID
    pub fn with_program_id(mut self, program_id: String) -> Self {
        self.program_id = Some(program_id);
        self
    }

    /// Set the compilation timestamp
    pub fn with_compiled_at(mut self, timestamp: String) -> Self {
        self.compiled_at = Some(timestamp);
        self
    }

    /// Set the compiler version
    pub fn with_compiler_version(mut self, version: String) -> Self {
        self.compiler_version = Some(version);
        self
    }

    /// Add a global configuration parameter
    pub fn with_config(mut self, key: String, value: String) -> Self {
        self.global_config.insert(key, value);
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

//-----------------------------------------------------------------------------
// Compilation Error Types
//-----------------------------------------------------------------------------

/// Errors that can occur during compilation validation
#[derive(Debug, Clone, PartialEq)]
pub enum CompilerValidationError {
    /// Referenced expression does not exist
    MissingExpression(ExprId),
    
    /// Referenced handler does not exist
    MissingHandler(HandlerId),
    
    /// Referenced subgraph does not exist
    MissingSubgraph(SubgraphId),
    
    /// Circular dependency detected
    CircularDependency(Vec<ExprId>),
    
    /// Invalid graph structure
    InvalidGraphStructure(String),
    
    /// Missing required metadata
    MissingMetadata(String),
}

impl std::fmt::Display for CompilerValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerValidationError::MissingExpression(id) => {
                write!(f, "Missing expression: {:?}", id)
            }
            CompilerValidationError::MissingHandler(id) => {
                write!(f, "Missing handler: {:?}", id)
            }
            CompilerValidationError::MissingSubgraph(id) => {
                write!(f, "Missing subgraph: {:?}", id)
            }
            CompilerValidationError::CircularDependency(cycle) => {
                write!(f, "Circular dependency detected: {:?}", cycle)
            }
            CompilerValidationError::InvalidGraphStructure(msg) => {
                write!(f, "Invalid graph structure: {}", msg)
            }
            CompilerValidationError::MissingMetadata(field) => {
                write!(f, "Missing required metadata field: {}", field)
            }
        }
    }
}

impl std::error::Error for CompilerValidationError {}

//-----------------------------------------------------------------------------
// Builder Pattern Support
//-----------------------------------------------------------------------------

/// Builder for constructing CompiledTeg instances
pub struct CompiledTegBuilder {
    teg: CompiledTeg,
}

impl CompiledTegBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            teg: CompiledTeg::new(),
        }
    }

    /// Add an expression
    pub fn with_expression(mut self, expr_id: ExprId, expr: TypesExpr) -> Self {
        self.teg.add_expression(expr_id, expr);
        self
    }

    /// Add a handler
    pub fn with_handler(mut self, handler_id: HandlerId, handler: Handler) -> Self {
        self.teg.add_handler(handler_id, handler);
        self
    }

    /// Add a subgraph
    pub fn with_subgraph(mut self, subgraph_id: SubgraphId, subgraph: CompiledSubgraph) -> Self {
        self.teg.add_subgraph(subgraph_id, subgraph);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: CompiledTegMetadata) -> Self {
        self.teg.set_metadata(metadata);
        self
    }

    /// Build the CompiledTeg
    pub fn build(self) -> CompiledTeg {
        self.teg
    }

    /// Build and validate the CompiledTeg
    pub fn build_and_validate(self) -> Result<CompiledTeg, CompilerValidationError> {
        let teg = self.teg;
        teg.validate()?;
        Ok(teg)
    }
}

impl Default for CompiledTegBuilder {
    fn default() -> Self {
        Self::new()
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_compiled_subgraph() {
        let mut subgraph = CompiledSubgraph::new();
        let node_id = NodeId::new([1u8; 32]);
        let edge_id = EdgeId::new([2u8; 32]);

        subgraph.add_node(node_id);
        subgraph.add_edge(edge_id);
        subgraph.add_entry_point(node_id);
        subgraph.add_metadata("key".to_string(), "value".to_string());

        assert!(subgraph.contains_node(&node_id));
        assert!(subgraph.contains_edge(&edge_id));
        assert_eq!(subgraph.node_count(), 1);
        assert_eq!(subgraph.edge_count(), 1);
        assert_eq!(subgraph.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_compiled_teg_builder() {
        let metadata = CompiledTegMetadata::new("test_program".to_string(), "1.0.0".to_string())
            .with_program_id("test_id".to_string());

        let teg = CompiledTegBuilder::new()
            .with_metadata(metadata)
            .build();

        assert_eq!(teg.metadata.program_name, Some("test_program".to_string()));
        assert_eq!(teg.metadata.version, Some("1.0.0".to_string()));
        assert_eq!(teg.metadata.program_id, Some("test_id".to_string()));
    }

    #[test]
    fn test_compiled_teg_validation() {
        let teg = CompiledTeg::new();
        assert!(teg.validate().is_ok());

        // Test with missing expression reference would require more setup
        // This is a basic validation test
    }

    #[test]
    fn test_metadata_builder() {
        let metadata = CompiledTegMetadata::new("test".to_string(), "1.0".to_string())
            .with_compiler_version("0.1.0".to_string())
            .with_config("debug".to_string(), "true".to_string())
            .with_dependency("std".to_string());

        assert_eq!(metadata.program_name, Some("test".to_string()));
        assert_eq!(metadata.compiler_version, Some("0.1.0".to_string()));
        assert_eq!(metadata.global_config.get("debug"), Some(&"true".to_string()));
        assert_eq!(metadata.dependencies, vec!["std".to_string()]);
    }
} 