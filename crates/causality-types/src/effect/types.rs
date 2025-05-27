//! Effect type definition for the unified core type system

use std::fmt;
use crate::primitive::ids::{EntityId, DomainId, ExprId, HandlerId, AsId, NodeId, ResourceId};
use crate::primitive::string::Str;
use crate::primitive::time::Timestamp;
use crate::primitive::trait_::{HasInputs, HasOutputs, AsResource, AsIdentifiable, HasDomainId, HasExpression, HasTimestamp, AsEffect};
use crate::resource::flow::{ResourceFlow, ResourcePattern};
use crate::graph::r#trait::AsNode;
use crate::graph::optimization::{TypedDomain, EffectCostModel, ResourceUsageEstimate};
use crate::system::serialization::{Encode, Decode, SimpleSerialize, DecodeError};

/// Effect represents a computational effect in the causality system
#[derive(Debug, Clone, PartialEq)]
pub struct Effect {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub effect_type: Str,
    pub inputs: Vec<ResourceFlow>,
    pub outputs: Vec<ResourceFlow>,
    pub expression: Option<ExprId>,
    pub timestamp: Timestamp,
    pub resources: Vec<ResourceFlow>,
    pub nullifiers: Vec<ResourceFlow>,
    pub scoped_by: HandlerId,
    pub intent_id: Option<ExprId>,
    
    /// Source typed domain for this effect
    pub source_typed_domain: TypedDomain,
    /// Target typed domain for this effect
    pub target_typed_domain: TypedDomain,
    /// Cost estimation metadata
    pub cost_model: Option<EffectCostModel>,
    /// Resource usage prediction
    pub resource_usage_estimate: Option<ResourceUsageEstimate>,
    /// ProcessDataflowBlock instance this effect originates from
    pub originating_dataflow_instance: Option<ResourceId>,
}

impl Effect {
    /// Create a new Effect with required parameters
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
            resources: Vec::new(),
            nullifiers: Vec::new(),
            scoped_by: HandlerId::null(),
            intent_id: None,
            source_typed_domain: TypedDomain::default(),
            target_typed_domain: TypedDomain::default(),
            cost_model: None,
            resource_usage_estimate: None,
            originating_dataflow_instance: None,
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

    /// Builder method to set source typed domain
    pub fn with_source_typed_domain(mut self, source_typed_domain: TypedDomain) -> Self {
        self.source_typed_domain = source_typed_domain;
        self
    }
    
    /// Builder method to set target typed domain
    pub fn with_target_typed_domain(mut self, target_typed_domain: TypedDomain) -> Self {
        self.target_typed_domain = target_typed_domain;
        self
    }
    
    /// Builder method to set cost model
    pub fn with_cost_model(mut self, cost_model: EffectCostModel) -> Self {
        self.cost_model = Some(cost_model);
        self
    }
    
    /// Builder method to set resource usage estimate
    pub fn with_resource_usage_estimate(mut self, resource_usage_estimate: ResourceUsageEstimate) -> Self {
        self.resource_usage_estimate = Some(resource_usage_estimate);
        self
    }
    
    /// Builder method to set originating dataflow instance
    pub fn with_originating_dataflow_instance(mut self, originating_dataflow_instance: ResourceId) -> Self {
        self.originating_dataflow_instance = Some(originating_dataflow_instance);
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
            resources: Vec::new(),
            nullifiers: Vec::new(),
            scoped_by: HandlerId::null(),
            intent_id: None,
            source_typed_domain: TypedDomain::default(),
            target_typed_domain: TypedDomain::default(),
            cost_model: None,
            resource_usage_estimate: None,
            originating_dataflow_instance: None,
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
        bytes.extend_from_slice(&self.timestamp.as_ssz_bytes());
        bytes
    }
}

impl Decode for Effect {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let id = EntityId::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let name = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += name.as_ssz_bytes().len();
        
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..])?;
        offset += domain_id.as_ssz_bytes().len();
        
        let effect_type = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += effect_type.as_ssz_bytes().len();
        
        let timestamp = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(Effect {
            id,
            name,
            domain_id,
            effect_type,
            inputs: Vec::new(), // Simplified for now
            outputs: Vec::new(), // Simplified for now
            expression: None,
            timestamp,
            resources: Vec::new(),
            nullifiers: Vec::new(),
            scoped_by: HandlerId::null(),
            intent_id: None,
            source_typed_domain: TypedDomain::default(),
            target_typed_domain: TypedDomain::default(),
            cost_model: None,
            resource_usage_estimate: None,
            originating_dataflow_instance: None,
        })
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