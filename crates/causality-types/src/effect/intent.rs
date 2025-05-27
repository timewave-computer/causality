//! Intent type definitions for the Causality framework.
//! Intents represent desired outcomes or goals that can be satisfied by effects.

use crate::primitive::{
    ids::{NodeId, AsId, EntityId, ExprId, DomainId},
    string::Str,
    time::Timestamp,
    trait_::{AsIdentifiable, HasInputs, HasOutputs, HasExpression, HasTimestamp, AsIntent, AsEffect, HasDomainId},
};
use crate::graph::r#trait::AsNode;
use crate::resource::flow::ResourceFlow;
use crate::system::serialization::{Encode, DecodeWithLength, DecodeError, SimpleSerialize, Decode};

/// Unified Intent type representing a desired outcome or goal in the system
#[derive(Debug, Clone, PartialEq)]
pub struct Intent {
    /// Unique identifier for this intent
    pub id: EntityId,
    
    /// Human-readable name or description
    pub name: Str,
    
    /// Domain this intent belongs to
    pub domain_id: DomainId,
    
    /// Intent priority level for resolution ordering
    pub priority: u32,
    
    /// Resources required to satisfy this intent
    pub inputs: Vec<ResourceFlow>,
    
    /// Resources that would be produced when this intent is satisfied
    pub outputs: Vec<ResourceFlow>,
    
    /// TEL expression for validation, constraints, or satisfaction logic
    pub expression: Option<ExprId>,
    
    /// When this intent was created or became active
    pub timestamp: Timestamp,
    
    /// Hint expression for optimization guidance or preferences
    pub hint: Option<ExprId>,
}

impl Intent {
    /// Create a new Intent instance with required parameters
    pub fn new(
        id: EntityId,
        name: Str,
        domain_id: DomainId,
        priority: u32,
    ) -> Self {
        Self {
            id,
            name,
            domain_id,
            priority,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expression: None,
            timestamp: Timestamp::now(),
            hint: None,
        }
    }

    /// Builder method to set inputs
    pub fn with_inputs(mut self, inputs: Vec<ResourceFlow>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Builder method to set outputs
    pub fn with_outputs(mut self, outputs: Vec<ResourceFlow>) -> Self {
        self.outputs = outputs;
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

    /// Builder method to set hint
    pub fn with_hint(mut self, hint: ExprId) -> Self {
        self.hint = Some(hint);
        self
    }
}

impl Default for Intent {
    fn default() -> Self {
        Self {
            id: EntityId::new([0u8; 32]),
            name: Str::from("default_intent"),
            domain_id: DomainId::new([0u8; 32]),
            priority: 1,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expression: None,
            timestamp: Timestamp::now(),
            hint: None,
        }
    }
}

impl AsNode for Intent {
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

impl AsIdentifiable for Intent {
    fn id(&self) -> &EntityId {
        &self.id
    }
    
    fn name(&self) -> &Str {
        &self.name
    }
}

impl HasInputs for Intent {
    fn inputs(&self) -> &[ResourceFlow] {
        &self.inputs
    }
}

impl HasOutputs for Intent {
    fn outputs(&self) -> &[ResourceFlow] {
        &self.outputs
    }
}

impl HasExpression for Intent {
    fn expression(&self) -> Option<&ExprId> {
        self.expression.as_ref()
    }
}

impl HasTimestamp for Intent {
    fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl HasDomainId for Intent {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
}

impl AsIntent for Intent {
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn is_satisfied_by(&self, effects: &[&dyn AsEffect]) -> bool {
        // Default implementation - check if effects provide required outputs
        // This can be customized based on intent-specific logic
        let required_outputs = &self.outputs;
        let provided_outputs: Vec<_> = effects.iter()
            .flat_map(|effect| effect.outputs())
            .collect();
            
        // Simple check: all required outputs are provided
        required_outputs.iter().all(|required| {
            provided_outputs.iter().any(|provided| {
                // This is a simplified check - real implementation would need
                // more sophisticated resource matching logic
                required.resource_type == provided.resource_type &&
                required.quantity <= provided.quantity
            })
        })
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization
//-----------------------------------------------------------------------------

impl Encode for Intent {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // id: EntityId
        bytes.extend_from_slice(&self.id.as_ssz_bytes());

        // name: Str
        bytes.extend_from_slice(&self.name.as_ssz_bytes());

        // domain_id: DomainId
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());

        // priority: u32
        bytes.extend_from_slice(&self.priority.as_ssz_bytes());

        // inputs: Vec<ResourceFlow>
        bytes.extend_from_slice(&self.inputs.as_ssz_bytes());

        // outputs: Vec<ResourceFlow>
        bytes.extend_from_slice(&self.outputs.as_ssz_bytes());

        // expression: Option<ExprId>
        bytes.extend_from_slice(&self.expression.as_ssz_bytes());

        // timestamp: Timestamp
        bytes.extend_from_slice(&self.timestamp.as_ssz_bytes());

        // hint: Option<ExprId>
        bytes.extend_from_slice(&self.hint.as_ssz_bytes());
        
        bytes
    }
}

impl DecodeWithLength for Intent {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let mut offset = 0;

        // id: EntityId (fixed size: 32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError::new("Intent: Input bytes too short for id"));
        }
        let id = EntityId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;

        // name: Str (variable size)
        let (name, consumed) = Str::from_ssz_bytes_with_length(&bytes[offset..])?;
        offset += consumed;

        // domain_id: DomainId (fixed size: 32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError::new("Intent: Input bytes too short for domain_id"));
        }
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;

        // priority: u32 (fixed size: 4 bytes)
        if bytes.len() < offset + 4 {
            return Err(DecodeError::new("Intent: Input bytes too short for priority"));
        }
        let priority = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]);
        offset += 4;

        // inputs: Vec<ResourceFlow> (variable size)
        let (inputs, consumed) = <Vec<ResourceFlow>>::from_ssz_bytes_with_length(&bytes[offset..])?;
        offset += consumed;

        // outputs: Vec<ResourceFlow> (variable size)
        let (outputs, consumed) = <Vec<ResourceFlow>>::from_ssz_bytes_with_length(&bytes[offset..])?;
        offset += consumed;

        // expression: Option<ExprId> (variable size)
        let (expression, consumed) = Option::<ExprId>::from_ssz_bytes_with_length(&bytes[offset..])?;
        offset += consumed;

        // timestamp: Timestamp (fixed size: 48 bytes)
        if bytes.len() < offset + 48 {
            return Err(DecodeError::new("Intent: Input bytes too short for timestamp"));
        }
        let timestamp = Timestamp::from_ssz_bytes(&bytes[offset..offset + 48])?;
        offset += 48;

        // hint: Option<ExprId> (variable size)
        let (hint, consumed) = Option::<ExprId>::from_ssz_bytes_with_length(&bytes[offset..])?;
        offset += consumed;

        Ok((
            Intent {
                id,
                name,
                domain_id,
                priority,
                inputs,
                outputs,
                expression,
                timestamp,
                hint,
            },
            offset,
        ))
    }
}

impl Decode for Intent {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (intent, _) = Self::from_ssz_bytes_with_length(bytes)?;
        Ok(intent)
    }
}

impl SimpleSerialize for Intent {}