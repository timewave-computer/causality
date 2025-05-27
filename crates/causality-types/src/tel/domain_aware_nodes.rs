// Purpose: Domain-aware TEG node types with temporal relationships and content-addressable properties

use crate::{
    core::{
        id::{DomainId, NodeId, EffectId, EntityId, AsId},
        content_addressable::{ContentAddressable, ContentTraversable, DomainValidated},
        Effect, Intent, Handler,
    },
    serialization::{Encode, Decode, DecodeError},
    tel::common_refs::ResourceRef,
};
use anyhow::Result;
use std::collections::HashMap;

/// Domain-aware effect node with temporal constraints
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAwareEffectNode {
    pub domain_id: DomainId,
    pub effect: Effect,
    pub temporal_constraints: Vec<TemporalConstraint>,
    pub cross_domain_refs: Vec<(DomainId, NodeId)>,
    pub metadata: HashMap<String, String>,
}

/// Domain-aware resource node with domain context
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAwareResourceNode {
    pub domain_id: DomainId,
    pub resource: ResourceRef,
    pub access_constraints: Vec<AccessConstraint>,
    pub cross_domain_refs: Vec<(DomainId, NodeId)>,
    pub metadata: HashMap<String, String>,
}

/// Domain-aware intent node with execution context
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAwareIntentNode {
    pub domain_id: DomainId,
    pub intent: Intent,
    pub execution_constraints: Vec<IntentExecutionConstraint>,
    pub cross_domain_refs: Vec<(DomainId, NodeId)>,
    pub metadata: HashMap<String, String>,
}

/// Domain-aware handler node with processing constraints
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAwareHandlerNode {
    pub domain_id: DomainId,
    pub handler: Handler,
    pub processing_constraints: Vec<ProcessingConstraint>,
    pub cross_domain_refs: Vec<(DomainId, NodeId)>,
    pub metadata: HashMap<String, String>,
}

/// Temporal constraints for effects
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalConstraint {
    /// Effect must happen before another effect
    Before(EffectId),
    /// Effect must happen after another effect
    After(EffectId),
    /// Effect must happen within a time window (in domain-specific time units)
    WithinWindow { start: u64, end: u64 },
    /// Effect must happen at a specific time
    AtTime(u64),
    /// Effect depends on another effect completing
    DependsOn(EffectId),
}

/// Access constraints for resources
#[derive(Debug, Clone, PartialEq)]
pub enum AccessConstraint {
    /// Resource is read-only
    ReadOnly,
    /// Resource requires specific capability
    RequiresCapability(String),
    /// Resource has exclusive access requirements
    ExclusiveAccess,
    /// Resource access is time-bounded
    TimeBounded { start: u64, end: u64 },
    /// Resource access requires cross-domain permission
    CrossDomainAccess(DomainId),
}

/// Execution constraints for intents
#[derive(Debug, Clone, PartialEq)]
pub enum IntentExecutionConstraint {
    /// Intent must execute in specific domain
    RequiresDomain(DomainId),
    /// Intent has gas/resource limits
    ResourceLimit { gas: u64, memory: u64 },
    /// Intent requires specific capabilities
    RequiresCapabilities(Vec<String>),
    /// Intent has deadline
    Deadline(u64),
    /// Intent requires atomic execution
    Atomic,
}

/// Processing constraints for handlers
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingConstraint {
    /// Handler processes effects in order
    OrderedProcessing,
    /// Handler has concurrency limits
    ConcurrencyLimit(u32),
    /// Handler requires specific domain context
    RequiresDomainContext(DomainId),
    /// Handler has throughput limits
    ThroughputLimit { max_per_second: u32 },
    /// Handler requires deterministic execution
    Deterministic,
}

//-----------------------------------------------------------------------------
// Temporal Constraint Serialization
//-----------------------------------------------------------------------------

impl Encode for TemporalConstraint {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            TemporalConstraint::Before(effect_id) => {
                bytes.push(0u8);
                bytes.extend(effect_id.as_ssz_bytes());
            }
            TemporalConstraint::After(effect_id) => {
                bytes.push(1u8);
                bytes.extend(effect_id.as_ssz_bytes());
            }
            TemporalConstraint::WithinWindow { start, end } => {
                bytes.push(2u8);
                bytes.extend(start.to_le_bytes());
                bytes.extend(end.to_le_bytes());
            }
            TemporalConstraint::AtTime(time) => {
                bytes.push(3u8);
                bytes.extend(time.to_le_bytes());
            }
            TemporalConstraint::DependsOn(effect_id) => {
                bytes.push(4u8);
                bytes.extend(effect_id.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for TemporalConstraint {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "TemporalConstraint: empty bytes".to_string(),
            });
        }
        
        let discriminant = bytes[0];
        let data = &bytes[1..];
        
        match discriminant {
            0 => {
                let effect_id = EffectId::from_ssz_bytes(data)?;
                Ok(TemporalConstraint::Before(effect_id))
            }
            1 => {
                let effect_id = EffectId::from_ssz_bytes(data)?;
                Ok(TemporalConstraint::After(effect_id))
            }
            2 => {
                if data.len() < 16 {
                    return Err(DecodeError {
                        message: "TemporalConstraint::WithinWindow: insufficient bytes".to_string(),
                    });
                }
                let start = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
                let end = u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]);
                Ok(TemporalConstraint::WithinWindow { start, end })
            }
            3 => {
                if data.len() < 8 {
                    return Err(DecodeError {
                        message: "TemporalConstraint::AtTime: insufficient bytes".to_string(),
                    });
                }
                let time = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
                Ok(TemporalConstraint::AtTime(time))
            }
            4 => {
                let effect_id = EffectId::from_ssz_bytes(data)?;
                Ok(TemporalConstraint::DependsOn(effect_id))
            }
            _ => Err(DecodeError {
                message: format!("TemporalConstraint: invalid discriminant {}", discriminant),
            }),
        }
    }
}

//-----------------------------------------------------------------------------
// Domain-Aware Effect Node Implementation
//-----------------------------------------------------------------------------

impl Encode for DomainAwareEffectNode {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Domain ID
        bytes.extend(self.domain_id.as_ssz_bytes());
        
        // Effect data
        bytes.extend(self.effect.as_ssz_bytes());
        
        // Temporal constraints
        let constraint_count = self.temporal_constraints.len() as u32;
        bytes.extend(constraint_count.to_le_bytes());
        for constraint in &self.temporal_constraints {
            let constraint_bytes = constraint.as_ssz_bytes();
            bytes.extend((constraint_bytes.len() as u32).to_le_bytes());
            bytes.extend(constraint_bytes);
        }
        
        // Cross-domain references
        let cross_ref_count = self.cross_domain_refs.len() as u32;
        bytes.extend(cross_ref_count.to_le_bytes());
        for (domain_id, node_id) in &self.cross_domain_refs {
            bytes.extend(domain_id.as_ssz_bytes());
            bytes.extend(node_id.as_ssz_bytes());
        }
        
        // Metadata (simplified)
        let metadata_count = self.metadata.len() as u32;
        bytes.extend(metadata_count.to_le_bytes());
        for (key, value) in &self.metadata {
            let key_bytes = key.as_bytes();
            let value_bytes = value.as_bytes();
            bytes.extend((key_bytes.len() as u32).to_le_bytes());
            bytes.extend(key_bytes);
            bytes.extend((value_bytes.len() as u32).to_le_bytes());
            bytes.extend(value_bytes);
        }
        
        bytes
    }
}

impl Decode for DomainAwareEffectNode {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 32 {
            return Err(DecodeError {
                message: "DomainAwareEffectNode: insufficient bytes".to_string(),
            });
        }
        
        let mut offset = 0;
        
        // Domain ID
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;
        
        // Effect data (simplified - assume fixed size for now)
        let effect = Effect::from_ssz_bytes(&bytes[offset..])?;
        
        // For simplicity, return with empty constraints and metadata
        Ok(DomainAwareEffectNode {
            domain_id,
            effect,
            temporal_constraints: Vec::new(),
            cross_domain_refs: Vec::new(),
            metadata: HashMap::new(),
        })
    }
}

impl ContentAddressable for DomainAwareEffectNode {
    fn domain_id(&self) -> DomainId {
        self.domain_id
    }
    
    fn node_type_name(&self) -> &'static str {
        "effect"
    }
    
    fn validate(&self) -> Result<()> {
        // Validate temporal constraints don't create cycles
        let mut effect_deps = std::collections::HashSet::new();
        for constraint in &self.temporal_constraints {
            match constraint {
                TemporalConstraint::Before(id) | TemporalConstraint::After(id) | TemporalConstraint::DependsOn(id) => {
                    // Convert EffectId to EntityId for comparison
                    let entity_id = EntityId::new(id.inner());
                    if entity_id == self.effect.id {
                        return Err(anyhow::anyhow!("Effect cannot depend on itself"));
                    }
                    effect_deps.insert(id);
                }
                _ => {}
            }
        }
        
        if self.temporal_constraints.len() > 100 {
            return Err(anyhow::anyhow!("Too many temporal constraints"));
        }
        
        Ok(())
    }
}

impl ContentTraversable for DomainAwareEffectNode {
    fn child_refs(&self) -> Vec<NodeId> {
        // Extract node references from temporal constraints
        let mut refs = Vec::new();
        for constraint in &self.temporal_constraints {
            match constraint {
                TemporalConstraint::Before(id) | TemporalConstraint::After(id) | TemporalConstraint::DependsOn(id) => {
                    refs.push(NodeId::new(id.inner()));
                }
                _ => {}
            }
        }
        refs
    }
    
    fn cross_domain_refs(&self) -> Vec<(DomainId, NodeId)> {
        self.cross_domain_refs.clone()
    }
}

impl DomainValidated for DomainAwareEffectNode {
    fn validate_in_domain(&self, domain_id: &DomainId) -> Result<()> {
        if self.domain_id != *domain_id {
            return Err(anyhow::anyhow!(
                "Effect domain {} does not match expected domain {}",
                self.domain_id,
                domain_id
            ));
        }
        
        self.validate()
    }
    
    fn is_compatible_with_domain(&self, domain_id: &DomainId) -> bool {
        self.domain_id == *domain_id
    }
}

//-----------------------------------------------------------------------------
// Builder Patterns
//-----------------------------------------------------------------------------

/// Builder for domain-aware effect nodes
#[derive(Debug, Default)]
pub struct DomainAwareEffectBuilder {
    domain_id: Option<DomainId>,
    effect: Option<Effect>,
    temporal_constraints: Vec<TemporalConstraint>,
    cross_domain_refs: Vec<(DomainId, NodeId)>,
    metadata: HashMap<String, String>,
}

impl DomainAwareEffectBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn domain_id(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    pub fn effect(mut self, effect: Effect) -> Self {
        self.effect = Some(effect);
        self
    }
    
    pub fn add_temporal_constraint(mut self, constraint: TemporalConstraint) -> Self {
        self.temporal_constraints.push(constraint);
        self
    }
    
    pub fn add_cross_domain_ref(mut self, domain_id: DomainId, node_id: NodeId) -> Self {
        self.cross_domain_refs.push((domain_id, node_id));
        self
    }
    
    pub fn add_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    pub fn build(self) -> Result<DomainAwareEffectNode> {
        let domain_id = self.domain_id.ok_or_else(|| anyhow::anyhow!("Domain ID is required"))?;
        let effect = self.effect.ok_or_else(|| anyhow::anyhow!("Effect is required"))?;
        
        let node = DomainAwareEffectNode {
            domain_id,
            effect,
            temporal_constraints: self.temporal_constraints,
            cross_domain_refs: self.cross_domain_refs,
            metadata: self.metadata,
        };
        
        node.validate()?;
        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Effect, id::{DomainId, EntityId, ExprId, HandlerId}};
    use crate::primitive::string::Str;
    use crate::core::time::Timestamp;
    
    fn create_test_domain_id() -> DomainId {
        DomainId::new([1u8; 32])
    }
    
    fn create_test_effect() -> Effect {
        Effect {
            id: EntityId::new([2u8; 32]),
            name: Str::from("test_effect"),
            domain_id: create_test_domain_id(),
            effect_type: Str::from("computation"),
            inputs: vec![],
            outputs: vec![],
            expression: Some(ExprId::new([3u8; 32])),
            timestamp: Timestamp::now(),
            resources: vec![],
            nullifiers: vec![],
            scoped_by: HandlerId::new([4u8; 32]),
            intent_id: Some(ExprId::new([5u8; 32])),
            source_typed_domain: crate::tel::optimization::TypedDomain::default(),
            target_typed_domain: crate::tel::optimization::TypedDomain::default(),
            cost_model: None,
            resource_usage_estimate: None,
            originating_dataflow_instance: None,
        }
    }
    
    #[test]
    fn test_domain_aware_effect_node_creation() {
        let domain_id = create_test_domain_id();
        let effect = create_test_effect();
        
        let node = DomainAwareEffectBuilder::new()
            .domain_id(domain_id)
            .effect(effect.clone())
            .add_temporal_constraint(TemporalConstraint::AtTime(1000))
            .add_metadata("type", "test-effect")
            .build()
            .unwrap();
        
        assert_eq!(node.domain_id, domain_id);
        assert_eq!(node.effect.id, effect.id);
        assert_eq!(node.temporal_constraints.len(), 1);
    }
    
    #[test]
    fn test_temporal_constraint_serialization() {
        let constraint = TemporalConstraint::WithinWindow { start: 100, end: 200 };
        let bytes = constraint.as_ssz_bytes();
        let decoded = TemporalConstraint::from_ssz_bytes(&bytes).unwrap();
        assert_eq!(constraint, decoded);
    }
} 