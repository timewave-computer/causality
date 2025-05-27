//-----------------------------------------------------------------------------
// Resource NodeId Conversion Utilities
//-----------------------------------------------------------------------------

use crate::primitive::ids::{AsId, NodeId, ResourceId};
// use std::any::TypeId; // Unused

/// Convert a ResourceId to NodeId and vice versa
pub trait ResourceNodeIdConverter {
    fn to_node_id(resource_id: &ResourceId) -> NodeId;
    fn to_resource_id(node_id: &NodeId) -> ResourceId;
}

/// Default implementation of ResourceNodeIdConverter
pub struct DefaultResourceNodeIdConverter;

impl ResourceNodeIdConverter for DefaultResourceNodeIdConverter {
    fn to_node_id(resource_id: &ResourceId) -> NodeId {
        // Just reinterpret the bytes as a NodeId
        NodeId::new(resource_id.inner())
    }
    
    fn to_resource_id(node_id: &NodeId) -> ResourceId {
        // Just reinterpret the bytes as a ResourceId
        ResourceId::new(node_id.inner())
    }
}
