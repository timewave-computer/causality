//-----------------------------------------------------------------------------
// Graph Elements
//-----------------------------------------------------------------------------

use std::collections::BTreeMap;

use crate::primitive::ids::{EdgeId, NodeId};
use crate::expr::value::ValueExpr;
use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use crate::AsId;


// Define our own TypeId for graph element types
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash
)]
pub struct TypeId(pub [u8; 32]);

impl TypeId {
    /// Create a new TypeId from a string
    pub fn from_string(_s: &str) -> Self {
        // TODO: hashed value
        // let mut hasher = Sha256::new();
        // hasher.update(s.as_bytes());
        // let result = hasher.finalize();
        // let mut bytes = [0u8; 32];
        // bytes.copy_from_slice(&result);
        // Self(bytes)
        Self([0u8; 32])
    }
}

//-----------------------------------------------------------------------------
// Node Definition
//-----------------------------------------------------------------------------

/// Represents a node in the graph
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for this node
    pub id: NodeId,

    /// The type of this node
    pub type_id: TypeId,

    /// Properties associated with this node
    pub properties: BTreeMap<String, ValueExpr>,
}

impl Node {
    /// Create a new node with the given ID and type
    pub fn new(id: NodeId, type_id: TypeId) -> Self {
        Self {
            id,
            type_id,
            properties: BTreeMap::new(),
        }
    }

    /// Create a new node with properties
    pub fn with_properties(
        id: NodeId,
        type_id: TypeId,
        properties: BTreeMap<String, ValueExpr>,
    ) -> Self {
        Self {
            id,
            type_id,
            properties,
        }
    }
}

//-----------------------------------------------------------------------------
// Edge Definition
//-----------------------------------------------------------------------------

/// Edge represents a connection between two nodes in the graph
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// Unique identifier for this edge
    pub id: EdgeId,

    /// The type of this edge
    pub type_id: TypeId,

    /// The source node of this edge
    pub source_id: NodeId,

    /// The target node of this edge
    pub target_id: NodeId,

    /// Properties associated with this edge
    pub properties: BTreeMap<String, ValueExpr>,
}

impl Edge {
    /// Create a new edge
    pub fn new(
        id: EdgeId,
        type_id: TypeId,
        source_id: NodeId,
        target_id: NodeId,
    ) -> Self {
        Self {
            id,
            type_id,
            source_id,
            target_id,
            properties: BTreeMap::new(),
        }
    }

    /// Create a new edge with properties
    pub fn with_properties(
        id: EdgeId,
        type_id: TypeId,
        source_id: NodeId,
        target_id: NodeId,
        properties: BTreeMap<String, ValueExpr>,
    ) -> Self {
        Self {
            id,
            type_id,
            source_id,
            target_id,
            properties,
        }
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

// TypeId
impl Encode for TypeId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Decode for TypeId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 32 {
            return Err(DecodeError {
                message: format!("Invalid TypeId length {}, expected 32", bytes.len()),
            });
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(TypeId(array))
    }
}

impl SimpleSerialize for TypeId {}

// Node
impl Encode for Node {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.as_ssz_bytes());
        bytes.extend(self.type_id.as_ssz_bytes());
        bytes.extend(self.properties.as_ssz_bytes());
        bytes
    }
}

impl Decode for Node {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode NodeId (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError { message: "Not enough bytes for NodeId".to_string() });
        }
        let id = NodeId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;
        
        // Decode TypeId (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError { message: "Not enough bytes for TypeId".to_string() });
        }
        let type_id = TypeId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;
        
        // Decode properties
        let properties = BTreeMap::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(Node { id, type_id, properties })
    }
}

impl SimpleSerialize for Node {}

// Edge
impl Encode for Edge {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.as_ssz_bytes());
        bytes.extend(self.type_id.as_ssz_bytes());
        bytes.extend(self.source_id.as_ssz_bytes());
        bytes.extend(self.target_id.as_ssz_bytes());
        bytes.extend(self.properties.as_ssz_bytes());
        bytes
    }
}

impl Decode for Edge {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode EdgeId (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError { message: "Not enough bytes for EdgeId".to_string() });
        }
        let id = EdgeId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;
        
        // Decode TypeId (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError { message: "Not enough bytes for TypeId".to_string() });
        }
        let type_id = TypeId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;
        
        // Decode source NodeId (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError { message: "Not enough bytes for source NodeId".to_string() });
        }
        let source_id = NodeId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;
        
        // Decode target NodeId (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError { message: "Not enough bytes for target NodeId".to_string() });
        }
        let target_id = NodeId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;
        
        // Decode properties
        let properties = BTreeMap::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(Edge { id, type_id, source_id, target_id, properties })
    }
}

impl SimpleSerialize for Edge {}

impl Default for Edge {
    fn default() -> Self {
        Self {
            id: <EdgeId as AsId>::null(),
            type_id: TypeId([0u8; 32]),
            source_id: <NodeId as AsId>::null(),
            target_id: <NodeId as AsId>::null(),
            properties: BTreeMap::new(),
        }
    }
}
