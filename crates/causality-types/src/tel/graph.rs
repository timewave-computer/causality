// Purpose: Defines graph-related structures for the Temporal Effect Language (TEL),
// specifically Edges and their kinds.

use crate::primitive::ids::{EdgeId, HandlerId, NodeId};
use crate::expr::value::ValueExpr; // Assuming ValueExpr is at this path
use crate::graph::r#trait::AsEdge;
use crate::tel::common_refs::ResourceRef;
use crate::serialization::{Decode, Encode, SimpleSerialize, DecodeError};

// Forward declaration for EdgeKind, will be defined in this file.
// pub enum EdgeKind { ... }

/// Defines the kind of relationship or dependency an Edge represents
/// in the Temporal Effect Language graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Represents a generic control flow dependency, indicating that the source
    /// node must be processed before the target node.
    /// (e.g., Intent to first Effect, or Effect to subsequent Effect).
    ControlFlow,

    /// Explicitly indicates that the target node must be processed immediately
    /// after the source node. More specific than generic ControlFlow.
    Next(NodeId),

    /// Indicates that the source node depends on the completion or state of another
    /// node (the target of this edge kind, though the edge itself still has a source and target field).
    /// This is for non-sequential dependencies.
    DependsOn(NodeId),

    /// Represents that the source node (typically an Effect) consumes the referenced Resource.
    Consumes(ResourceRef),

    /// Represents that the source node (typically an Effect) produces the referenced Resource.
    Produces(ResourceRef),

    /// The source node (typically an Effect) is processed by the referenced Handler.
    /// This is the primary mechanism for applying a handler's logic to an effect.
    Applies(HandlerId),

    /// The referenced Handler's logic is active and influences the processing scope
    /// of the source node (typically an Effect) and potentially its sub-effects.
    ScopedBy(HandlerId),

    /// The referenced Handler's logic takes precedence and overrides other normally
    /// applicable handlers for the source node (typically an Effect).
    Override(HandlerId),
}

impl Default for EdgeKind {
    fn default() -> Self {
        Self::ControlFlow
    }
}

// Manually implement Encode for EdgeKind
impl Encode for EdgeKind {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        match self {
            EdgeKind::ControlFlow => {
                result.push(0); // Tag for ControlFlow
            },
            EdgeKind::Next(node_id) => {
                result.push(1); // Tag for Next
                let node_bytes = node_id.as_ssz_bytes();
                result.extend_from_slice(&node_bytes);
            },
            EdgeKind::DependsOn(node_id) => {
                result.push(2); // Tag for DependsOn
                let node_bytes = node_id.as_ssz_bytes();
                result.extend_from_slice(&node_bytes);
            },
            EdgeKind::Consumes(resource) => {
                result.push(3); // Tag for Consumes
                let resource_bytes = resource.as_ssz_bytes();
                result.extend_from_slice(&resource_bytes);
            },
            EdgeKind::Produces(resource) => {
                result.push(4); // Tag for Produces
                let resource_bytes = resource.as_ssz_bytes();
                result.extend_from_slice(&resource_bytes);
            },
            EdgeKind::Applies(handler_id) => {
                result.push(5); // Tag for Applies
                let handler_bytes = handler_id.as_ssz_bytes();
                result.extend_from_slice(&handler_bytes);
            },
            EdgeKind::ScopedBy(handler_id) => {
                result.push(6); // Tag for ScopedBy
                let handler_bytes = handler_id.as_ssz_bytes();
                result.extend_from_slice(&handler_bytes);
            },
            EdgeKind::Override(handler_id) => {
                result.push(7); // Tag for Override
                let handler_bytes = handler_id.as_ssz_bytes();
                result.extend_from_slice(&handler_bytes);
            },
        }
        
        result
    }
}

// Manually implement Decode for EdgeKind
impl Decode for EdgeKind {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Expected at least 1 byte for EdgeKind".to_string(),
            });
        }
        
        let tag = bytes[0];
        let data = &bytes[1..]; // Skip the tag byte
        
        match tag {
            0 => Ok(EdgeKind::ControlFlow),
            1 => {
                let node_id = NodeId::from_ssz_bytes(data)?;
                Ok(EdgeKind::Next(node_id))
            },
            2 => {
                let node_id = NodeId::from_ssz_bytes(data)?;
                Ok(EdgeKind::DependsOn(node_id))
            },
            3 => {
                let resource = ResourceRef::from_ssz_bytes(data)?;
                Ok(EdgeKind::Consumes(resource))
            },
            4 => {
                let resource = ResourceRef::from_ssz_bytes(data)?;
                Ok(EdgeKind::Produces(resource))
            },
            5 => {
                let handler_id = HandlerId::from_ssz_bytes(data)?;
                Ok(EdgeKind::Applies(handler_id))
            },
            6 => {
                let handler_id = HandlerId::from_ssz_bytes(data)?;
                Ok(EdgeKind::ScopedBy(handler_id))
            },
            7 => {
                let handler_id = HandlerId::from_ssz_bytes(data)?;
                Ok(EdgeKind::Override(handler_id))
            },
            _ => Err(DecodeError {
                message: format!("Invalid EdgeKind tag: {}", tag),
            }),
        }
    }
}

// Implement SimpleSerialize for EdgeKind
impl SimpleSerialize for EdgeKind {}

/// Represents an Edge in the Temporal Effect Language graph.
///
/// Edges connect Effect nodes (or other potential node types) and define
/// the relationships and dependencies between them, such as execution order,
/// data flow, or handler application.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Edge {
    /// Unique identifier for this edge.
    pub id: EdgeId,

    /// The identifier of the source node of this edge.
    pub source: NodeId,

    /// The identifier of the target node of this edge.
    pub target: NodeId,

    /// The kind of relationship or dependency this edge represents.
    pub kind: EdgeKind,

    /// Optional metadata associated with this edge, structured as a ValueExpr.
    /// This can be used for edge-specific parameters, weights, or annotations.
    pub metadata: Option<ValueExpr>,
}

// Manually implement Encode for Edge
impl Encode for Edge {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Encode each field
        let id_bytes = self.id.as_ssz_bytes();
        result.extend_from_slice(&id_bytes);
        
        let source_bytes = self.source.as_ssz_bytes();
        result.extend_from_slice(&source_bytes);
        
        let target_bytes = self.target.as_ssz_bytes();
        result.extend_from_slice(&target_bytes);
        
        let kind_bytes = self.kind.as_ssz_bytes();
        result.extend_from_slice(&kind_bytes);
        
        // Encode metadata presence
        if let Some(ref metadata) = self.metadata {
            result.push(1); // Has metadata
            let metadata_bytes = metadata.as_ssz_bytes();
            result.extend_from_slice(&metadata_bytes);
        } else {
            result.push(0); // No metadata
        }
        
        result
    }
}

// Manually implement Decode for Edge
impl Decode for Edge {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 5 { // At minimum need id, source, target, kind, and metadata flag
            return Err(DecodeError {
                message: "Edge data too short".to_string(),
            });
        }
        
        let mut offset = 0;
        
        // Decode each field
        let id = EdgeId::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let source = NodeId::from_ssz_bytes(&bytes[offset..])?;
        offset += source.as_ssz_bytes().len();
        
        let target = NodeId::from_ssz_bytes(&bytes[offset..])?;
        offset += target.as_ssz_bytes().len();
        
        let kind = EdgeKind::from_ssz_bytes(&bytes[offset..])?;
        offset += kind.as_ssz_bytes().len();
        
        // Decode metadata if present
        let metadata = if bytes[offset] == 1 {
            offset += 1; // Skip the flag
            let meta = ValueExpr::from_ssz_bytes(&bytes[offset..])?;
            Some(meta)
        } else {
            None
        };
        
        Ok(Edge {
            id,
            source,
            target,
            kind,
            metadata,
        })
    }
}

// Implement SimpleSerialize for Edge
impl SimpleSerialize for Edge {}

impl AsEdge for Edge {
    fn to_edge_id(&self) -> EdgeId {
        self.id
    }

    fn from_edge_id(id: EdgeId) -> Option<Self> {
        // Similar to AsNode, full reconstruction is difficult.
        // Return a default Edge with this specific EdgeId.
        Some(Edge {
            id,
            ..Default::default()
        })
        // Consider implications for EdgeRegistry usage.
    }

    fn source(&self) -> NodeId {
        self.source
    }

    fn target(&self) -> NodeId {
        self.target
    }
}

//-----------------------------------------------------------------------------
// S-expression Serialization Support (for DSL output)
//-----------------------------------------------------------------------------

#[cfg(feature = "sexpr")]
use crate::expr::sexpr::{ToSexpr, FromSexpr, tagged_sexpr, validate_tag, extract_field};
#[cfg(feature = "sexpr")]
use crate::primitive::ids::AsId;
#[cfg(feature = "sexpr")]
use anyhow::{anyhow, Result};
#[cfg(feature = "sexpr")]
use lexpr::Value as SexprValue;

#[cfg(feature = "sexpr")]
impl ToSexpr for EdgeKind {
    fn to_sexpr(&self) -> SexprValue {
        match self {
            EdgeKind::ControlFlow => {
                tagged_sexpr("control-flow", vec![])
            }
            EdgeKind::Next(node_id) => {
                tagged_sexpr("next", vec![SexprValue::string(&*node_id.to_hex())])
            }
            EdgeKind::DependsOn(node_id) => {
                tagged_sexpr("depends-on", vec![SexprValue::string(&*node_id.to_hex())])
            }
            EdgeKind::Consumes(resource_ref) => {
                tagged_sexpr("consumes", vec![resource_ref.to_sexpr()])
            }
            EdgeKind::Produces(resource_ref) => {
                tagged_sexpr("produces", vec![resource_ref.to_sexpr()])
            }
            EdgeKind::Applies(handler_id) => {
                tagged_sexpr("applies", vec![SexprValue::string(&*handler_id.to_hex())])
            }
            EdgeKind::ScopedBy(handler_id) => {
                tagged_sexpr("scoped-by", vec![SexprValue::string(&*handler_id.to_hex())])
            }
            EdgeKind::Override(handler_id) => {
                tagged_sexpr("override", vec![SexprValue::string(&*handler_id.to_hex())])
            }
        }
    }
}

#[cfg(feature = "sexpr")]
impl FromSexpr for EdgeKind {
    fn from_sexpr(sexpr: &SexprValue) -> Result<Self> {
        let tag = crate::expr::sexpr::get_tag(sexpr)?;
        
        match tag {
            "control-flow" => Ok(EdgeKind::ControlFlow),
            "next" => {
                let elements = crate::expr::sexpr::get_list_elements(sexpr)
                    .ok_or_else(|| anyhow!("EdgeKind Next S-expression must be a list"))?;
                if elements.len() != 2 {
                    return Err(anyhow!("EdgeKind Next must have exactly one node ID"));
                }
                let node_hex = crate::expr::sexpr::get_string_value(&elements[1])
                    .ok_or_else(|| anyhow!("Node ID must be a string"))?;
                let node_id = NodeId::from_hex(node_hex).map_err(|_| anyhow!("Invalid node ID hex"))?;
                Ok(EdgeKind::Next(node_id))
            }
            "depends-on" => {
                let elements = crate::expr::sexpr::get_list_elements(sexpr)
                    .ok_or_else(|| anyhow!("EdgeKind DependsOn S-expression must be a list"))?;
                if elements.len() != 2 {
                    return Err(anyhow!("EdgeKind DependsOn must have exactly one node ID"));
                }
                let node_hex = crate::expr::sexpr::get_string_value(&elements[1])
                    .ok_or_else(|| anyhow!("Node ID must be a string"))?;
                let node_id = NodeId::from_hex(node_hex).map_err(|_| anyhow!("Invalid node ID hex"))?;
                Ok(EdgeKind::DependsOn(node_id))
            }
            "consumes" => {
                let elements = crate::expr::sexpr::get_list_elements(sexpr)
                    .ok_or_else(|| anyhow!("EdgeKind Consumes S-expression must be a list"))?;
                if elements.len() != 2 {
                    return Err(anyhow!("EdgeKind Consumes must have exactly one resource ref"));
                }
                let resource_ref = ResourceRef::from_sexpr(&elements[1])?;
                Ok(EdgeKind::Consumes(resource_ref))
            }
            "produces" => {
                let elements = crate::expr::sexpr::get_list_elements(sexpr)
                    .ok_or_else(|| anyhow!("EdgeKind Produces S-expression must be a list"))?;
                if elements.len() != 2 {
                    return Err(anyhow!("EdgeKind Produces must have exactly one resource ref"));
                }
                let resource_ref = ResourceRef::from_sexpr(&elements[1])?;
                Ok(EdgeKind::Produces(resource_ref))
            }
            "applies" => {
                let elements = crate::expr::sexpr::get_list_elements(sexpr)
                    .ok_or_else(|| anyhow!("EdgeKind Applies S-expression must be a list"))?;
                if elements.len() != 2 {
                    return Err(anyhow!("EdgeKind Applies must have exactly one handler ID"));
                }
                let handler_hex = crate::expr::sexpr::get_string_value(&elements[1])
                    .ok_or_else(|| anyhow!("Handler ID must be a string"))?;
                let handler_id = HandlerId::from_hex(handler_hex).map_err(|_| anyhow!("Invalid handler ID hex"))?;
                Ok(EdgeKind::Applies(handler_id))
            }
            "scoped-by" => {
                let elements = crate::expr::sexpr::get_list_elements(sexpr)
                    .ok_or_else(|| anyhow!("EdgeKind ScopedBy S-expression must be a list"))?;
                if elements.len() != 2 {
                    return Err(anyhow!("EdgeKind ScopedBy must have exactly one handler ID"));
                }
                let handler_hex = crate::expr::sexpr::get_string_value(&elements[1])
                    .ok_or_else(|| anyhow!("Handler ID must be a string"))?;
                let handler_id = HandlerId::from_hex(handler_hex).map_err(|_| anyhow!("Invalid handler ID hex"))?;
                Ok(EdgeKind::ScopedBy(handler_id))
            }
            "override" => {
                let elements = crate::expr::sexpr::get_list_elements(sexpr)
                    .ok_or_else(|| anyhow!("EdgeKind Override S-expression must be a list"))?;
                if elements.len() != 2 {
                    return Err(anyhow!("EdgeKind Override must have exactly one handler ID"));
                }
                let handler_hex = crate::expr::sexpr::get_string_value(&elements[1])
                    .ok_or_else(|| anyhow!("Handler ID must be a string"))?;
                let handler_id = HandlerId::from_hex(handler_hex).map_err(|_| anyhow!("Invalid handler ID hex"))?;
                Ok(EdgeKind::Override(handler_id))
            }
            _ => Err(anyhow!("Unknown EdgeKind type: {}", tag)),
        }
    }
}

#[cfg(feature = "sexpr")]
impl ToSexpr for Edge {
    fn to_sexpr(&self) -> SexprValue {
        let mut fields = Vec::new();
        
        fields.push(("id".to_string(), SexprValue::string(&*self.id.to_hex())));
        fields.push(("source".to_string(), SexprValue::string(&*self.source.to_hex())));
        fields.push(("target".to_string(), SexprValue::string(&*self.target.to_hex())));
        fields.push(("kind".to_string(), self.kind.to_sexpr()));
        
        if let Some(metadata) = &self.metadata {
            fields.push(("metadata".to_string(), metadata.to_sexpr()));
        } else {
            fields.push(("metadata".to_string(), SexprValue::Nil));
        }
        
        tagged_sexpr("define-edge", vec![
            crate::expr::sexpr::map_sexpr(fields)
        ])
    }
}

#[cfg(feature = "sexpr")]
impl FromSexpr for Edge {
    fn from_sexpr(sexpr: &SexprValue) -> Result<Self> {
        validate_tag(sexpr, "define-edge")?;
        
        let elements = crate::expr::sexpr::get_list_elements(sexpr)
            .ok_or_else(|| anyhow!("Edge S-expression must be a list"))?;
            
        if elements.len() != 2 {
            return Err(anyhow!("Edge S-expression must have exactly one map"));
        }
        
        let map = &elements[1];
        
        let id_str = extract_field(map, "id")?;
        let id_hex = crate::expr::sexpr::get_string_value(id_str)
            .ok_or_else(|| anyhow!("id must be a string"))?;
        let id = EdgeId::from_hex(id_hex).map_err(|_| anyhow!("Invalid id hex"))?;
        
        let source_str = extract_field(map, "source")?;
        let source_hex = crate::expr::sexpr::get_string_value(source_str)
            .ok_or_else(|| anyhow!("source must be a string"))?;
        let source = NodeId::from_hex(source_hex).map_err(|_| anyhow!("Invalid source hex"))?;
        
        let target_str = extract_field(map, "target")?;
        let target_hex = crate::expr::sexpr::get_string_value(target_str)
            .ok_or_else(|| anyhow!("target must be a string"))?;
        let target = NodeId::from_hex(target_hex).map_err(|_| anyhow!("Invalid target hex"))?;
        
        let kind_val = extract_field(map, "kind")?;
        let kind = EdgeKind::from_sexpr(kind_val)?;
        
        let metadata = match extract_field(map, "metadata") {
            Ok(val) if val.is_nil() => None,
            Ok(val) => {
                Some(crate::expr::value::ValueExpr::from_sexpr(val)?)
            }
            Err(_) => None,
        };
        
        Ok(Edge {
            id,
            source,
            target,
            kind,
            metadata,
        })
    }
}
