//! Unified Handler type definition for the Causality framework.
//!
//! This module contains the single, canonical Handler type that replaces
//! all previous scattered handler definitions.

use crate::primitive::ids::{DomainId, EntityId, NodeId, AsId, ExprId};
use crate::primitive::string::Str;
use crate::primitive::time::Timestamp;
use crate::primitive::trait_::{AsIdentifiable, HasDomainId, HasExpression, HasTimestamp, AsHandler};
use crate::graph::r#trait::AsNode;
use crate::system::serialization::{Decode, Encode, SimpleSerialize, DecodeError};

/// Unified Handler type representing logic for processing effects or intents
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handler {
    /// Unique identifier for this handler
    pub id: EntityId,
    
    /// Human-readable name or description
    pub name: Str,
    
    /// Domain this handler belongs to
    pub domain_id: DomainId,
    
    /// Type of objects this handler can process (e.g., "token_transfer", "stake_delegation")
    pub handles_type: Str,
    
    /// Handler priority for conflict resolution (higher = more priority)
    pub priority: u32,
    
    /// TEL expression defining the handler's processing logic
    pub expression: Option<ExprId>,
    
    /// When this handler was created or became active
    pub timestamp: Timestamp,
}

impl Handler {
    /// Create a new Handler instance with required parameters
    pub fn new(
        id: EntityId,
        name: Str,
        domain_id: DomainId,
        handles_type: Str,
    ) -> Self {
        Self {
            id,
            name,
            domain_id,
            handles_type,
            priority: 1,
            expression: None,
            timestamp: Timestamp::now(),
        }
    }

    /// Builder method to set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Builder method to set expression
    pub fn with_expression(mut self, expression: ExprId) -> Self {
        self.expression = Some(expression);
        self
    }

    /// Builder method to set timestamp
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = timestamp;
        self
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            id: EntityId::new([0u8; 32]),
            name: Str::from("default_handler"),
            domain_id: DomainId::new([0u8; 32]),
            handles_type: Str::from("default"),
            priority: 1,
            expression: None,
            timestamp: Timestamp::now(),
        }
    }
}

impl AsNode for Handler {
    fn to_node_id(&self) -> NodeId {
        // Convert EntityId to NodeId - they use the same internal format
        NodeId::new(self.id.inner())
    }

    fn from_node_id(id: NodeId) -> Option<Self> {
        Some(Self {
            id: EntityId::new(id.inner()),
            ..Default::default()
        })
    }
}

//-----------------------------------------------------------------------------
// Trait Implementations
//-----------------------------------------------------------------------------

impl AsIdentifiable for Handler {
    fn id(&self) -> &EntityId {
        &self.id
    }
    
    fn name(&self) -> &Str {
        &self.name
    }
}

impl HasDomainId for Handler {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
}

impl HasExpression for Handler {
    fn expression(&self) -> Option<&ExprId> {
        self.expression.as_ref()
    }
}

impl HasTimestamp for Handler {
    fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl AsHandler for Handler {
    fn handles_type(&self) -> &Str {
        &self.handles_type
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn can_handle(&self, _target: &dyn std::any::Any) -> bool {
        // Default implementation - can be customized based on handler type
        // This would typically examine the target object's type and other properties
        true
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization
//-----------------------------------------------------------------------------

impl Encode for Handler {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Encode each field in order
        bytes.extend_from_slice(&self.id.as_ssz_bytes());
        bytes.extend_from_slice(&self.name.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.handles_type.as_ssz_bytes());
        bytes.extend_from_slice(&self.priority.to_le_bytes());
        
        // Handle optional expression
        if let Some(ref expr) = self.expression {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&expr.as_ssz_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        bytes.extend_from_slice(&self.timestamp.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for Handler {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode each field in order
        let id = EntityId::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let name = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += name.as_ssz_bytes().len();
        
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..])?;
        offset += domain_id.as_ssz_bytes().len();
        
        let handles_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += handles_type.as_ssz_bytes().len();
        
        if offset + 4 > bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for priority".to_string() });
        }
        let priority = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]);
        offset += 4;
        
        // Handle optional expression
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for expression marker".to_string() });
        }
        
        let expression = if bytes[offset] == 1 {
            offset += 1;
            Some(ExprId::from_ssz_bytes(&bytes[offset..])?)
        } else {
            offset += 1;
            None
        };
        
        if let Some(ref expr) = expression {
            offset += expr.as_ssz_bytes().len();
        }
        
        let timestamp = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(Handler {
            id,
            name,
            domain_id,
            handles_type,
            priority,
            expression,
            timestamp,
        })
    }
}

impl SimpleSerialize for Handler {} 