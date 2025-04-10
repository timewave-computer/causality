// Edge definitions for the Temporal Effect Graph
// This file defines different types of edges that can connect nodes in the TEG.

use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::{EffectId, ResourceId};

/// Unique identifier for an edge in the graph
pub type EdgeId = String;

/// Enumeration of access modes for resource edges
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum AccessMode {
    /// Read-only access to the resource
    Read,
    /// Write access to the resource
    Write,
    /// Read and write access to the resource
    ReadWrite,
    /// Special access mode for resource creation
    Create,
    /// Special access mode for resource deletion
    Delete,
}

/// Temporal relation between effects
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum TemporalRelation {
    /// First effect must happen before second effect
    Before,
    /// First effect must happen after second effect
    After,
    /// Effects must happen simultaneously
    Simultaneous,
    /// Effects must not overlap in time
    NonOverlapping,
    /// Custom temporal relation with a name
    Custom(String),
}

/// Condition for continuation edges
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum Condition {
    /// Continue on successful completion
    Success,
    /// Continue on error
    Error,
    /// Continue on specific outcome (e.g. error code)
    Specific(String),
    /// Always continue
    Always,
}

/// Relationship type between resources
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum RelationshipType {
    /// Parent-child relationship (containment)
    ParentChild,
    /// Reference relationship (one resource references another)
    Reference,
    /// Dependency relationship (one resource depends on another)
    Dependency,
    /// Association relationship (resources are associated)
    Association,
    /// Transformation relationship (one resource transforms into another)
    Transformation,
    /// Custom relationship with a name
    Custom(String),
}

/// Types of edges in the Temporal Effect Graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum EdgeType {
    /// Sequential continuation between effects
    Continuation {
        /// Condition for following this continuation
        condition: Option<Condition>,
    },
    
    /// Resource access by an effect
    ResourceAccess {
        /// Mode of access (read, write, etc.)
        mode: AccessMode,
    },
    
    /// Dependency between effects
    Dependency,
    
    /// Temporal constraint between effects
    TemporalConstraint {
        /// Type of temporal relation
        relation: TemporalRelation,
    },
    
    /// Relationship between resources
    ResourceRelationship {
        /// Type of relationship
        relationship_type: RelationshipType,
    },
}

/// Represents an edge in the Temporal Effect Graph
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Edge {
    /// Unique identifier for this edge
    pub id: EdgeId,
    
    /// Source node ID (can be effect or resource)
    pub source: NodeId,
    
    /// Target node ID (can be effect or resource)
    pub target: NodeId,
    
    /// Type of the edge
    pub edge_type: EdgeType,
}

/// Identifier for a node in the graph (either effect or resource)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum NodeId {
    /// Effect node identifier
    Effect(EffectId),
    
    /// Resource node identifier
    Resource(ResourceId),
}
