// Abstract syntax tree for pattern matching
// Original file: src/ast.rs

// Abstract Syntax Tree (AST) Module
//
// This module provides functionality for working with ASTs,
/// including AST-to-resource graph correspondence.

pub mod resource_graph;

pub use resource_graph::{
    AstNodeId, AstNodeType, AstContext, Delta,
    DivergenceType, DivergencePoint, ControllerTransition,
    GraphCorrelation, CorrelationTracker, AttributedResourceGrant
}; 