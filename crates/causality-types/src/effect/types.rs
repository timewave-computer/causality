//! Effect type definition for the unified core type system

use std::fmt;
use crate::primitive::ids::{EntityId, DomainId, ExprId, AsId, NodeId};
use crate::primitive::string::Str;
use crate::primitive::time::Timestamp;
use crate::primitive::trait_::{HasInputs, HasOutputs, AsResource, AsIdentifiable, HasDomainId, HasExpression, HasTimestamp, AsEffect};
use crate::resource::flow::{ResourceFlow, ResourcePattern};
use crate::graph::r#trait::AsNode;
use crate::system::serialization::{Encode, DecodeWithLength, DecodeError, SimpleSerialize, Decode};

/// Effect represents a computational effect (algebraic effect specification) in the causality system
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub effect_type: Str,
    pub inputs: Vec<ResourceFlow>,
    pub outputs: Vec<ResourceFlow>,
    pub expression: Option<ExprId>,
    pub timestamp: Timestamp, // Timestamp of the effect specification's creation
    pub hint: Option<ExprId>,   // For preferences like target handler, cost, performance, etc.
}

impl Effect {
    /// Create a new Effect specification with required parameters
    pub fn new(
        id: EntityId,
        name: Str,
        domain_id: DomainId,
        effect_type: Str,
    ) -> Self {
        Self {
            id,
            name,
            domain_id,
            effect_type,
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

    /// Builder method to set hint expression
    pub fn with_hint(mut self, hint: ExprId) -> Self {
        self.hint = Some(hint);
        self
    }
}

impl Default for Effect {
    fn default() -> Self {
        Self {
            id: EntityId::null(),
            name: Str::from("default_effect"),
            domain_id: DomainId::null(),
            effect_type: Str::from("default"),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expression: None,
            timestamp: Timestamp::now(),
            hint: None,
        }
    }
}

impl AsNode for Effect {
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

impl HasInputs for Effect {
    fn inputs(&self) -> &[ResourceFlow] {
        &self.inputs
    }
}

impl HasOutputs for Effect {
    fn outputs(&self) -> &[ResourceFlow] {
        &self.outputs
    }
}

impl AsResource for Effect {
    fn resource_type(&self) -> &Str {
        &self.effect_type
    }
    
    fn quantity(&self) -> u64 {
        // Effects represent a single unit of computation/transformation
        1
    }
    
    fn matches_pattern(&self, pattern: &ResourcePattern) -> bool {
        pattern.resource_type == self.effect_type &&
        pattern.domain_id.as_ref().map_or(true, |d| *d == self.domain_id)
    }
}

impl Encode for Effect {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.as_ssz_bytes());
        bytes.extend_from_slice(&self.name.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.effect_type.as_ssz_bytes());

        // Encode inputs (Vec<ResourceFlow>)
        bytes.extend_from_slice(&self.inputs.as_ssz_bytes());

        // Encode outputs (Vec<ResourceFlow>)
        bytes.extend_from_slice(&self.outputs.as_ssz_bytes());

        // Encode expression (Option<ExprId>)
        bytes.extend_from_slice(&self.expression.as_ssz_bytes());

        // Encode timestamp (Timestamp)
        bytes.extend_from_slice(&self.timestamp.as_ssz_bytes());

        // Encode hint (Option<ExprId>)
        bytes.extend_from_slice(&self.hint.as_ssz_bytes());

        bytes
    }
}

impl DecodeWithLength for Effect {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let mut offset = 0;

        // id: EntityId (fixed size: 32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError::new("Effect: Input bytes too short for id"));
        }
        let id = EntityId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;

        // name: Str (variable size)
        let (name, consumed) = Str::from_ssz_bytes_with_length(&bytes[offset..])?;
        offset += consumed;

        // domain_id: DomainId (fixed size: 32 bytes)
        if bytes.len() < offset + 32 {
            return Err(DecodeError::new("Effect: Input bytes too short for domain_id"));
        }
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;

        // effect_type: Str (variable size)
        let (effect_type, consumed) = Str::from_ssz_bytes_with_length(&bytes[offset..])?;
        offset += consumed;

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
            return Err(DecodeError::new("Effect: Input bytes too short for timestamp"));
        }
        let timestamp = Timestamp::from_ssz_bytes(&bytes[offset..offset + 48])?;
        offset += 48;

        // hint: Option<ExprId> (variable size)
        let (hint, consumed) = Option::<ExprId>::from_ssz_bytes_with_length(&bytes[offset..])?;
        offset += consumed;

        Ok((
            Effect {
                id,
                name,
                domain_id,
                effect_type,
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

impl Decode for Effect {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (effect, _) = Self::from_ssz_bytes_with_length(bytes)?;
        Ok(effect)
    }
}

impl SimpleSerialize for Effect {}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Effect(id={}, name={}, type={})", self.id, self.name, self.effect_type)
    }
}

impl AsIdentifiable for Effect {
    fn id(&self) -> &EntityId {
        &self.id
    }
    
    fn name(&self) -> &Str {
        &self.name
    }
}

impl HasDomainId for Effect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
}

impl HasExpression for Effect {
    fn expression(&self) -> Option<&ExprId> {
        self.expression.as_ref()
    }
}

impl HasTimestamp for Effect {
    fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl AsEffect for Effect {
    fn effect_type(&self) -> &Str {
        &self.effect_type
    }
}