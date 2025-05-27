//! Intent type definitions for the Causality framework.
//! Intents represent desired outcomes or goals that can be satisfied by effects.

use crate::primitive::{
    ids::{NodeId, AsId, EntityId, DomainId, ExprId},
    string::Str,
    time::Timestamp,
    trait_::{AsIdentifiable, HasDomainId, HasInputs, HasOutputs, HasExpression, HasTimestamp, AsIntent, AsEffect},
};
use crate::graph::r#trait::AsNode;
use crate::resource::flow::ResourceFlow;
use crate::graph::optimization::{TypedDomain, ProcessDataflowInitiationHint, EffectCompatibility, ResourcePreference};
use crate::system::serialization::{Encode, Decode, SimpleSerialize, DecodeError};

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
    
    /// Optimization hint expression for strategy guidance
    pub optimization_hint: Option<ExprId>,
    
    /// Effect compatibility metadata for optimization
    pub compatibility_metadata: Vec<EffectCompatibility>,
    
    /// Resource preferences for optimization strategies
    pub resource_preferences: Vec<ResourcePreference>,
    
    /// Target typed domain for execution
    pub target_typed_domain: Option<TypedDomain>,
    
    /// ProcessDataflowBlock initiation hint
    pub process_dataflow_hint: Option<ProcessDataflowInitiationHint>,
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
            optimization_hint: None,
            compatibility_metadata: Vec::new(),
            resource_preferences: Vec::new(),
            target_typed_domain: None,
            process_dataflow_hint: None,
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

    /// Builder method to set optimization hint
    pub fn with_optimization_hint(mut self, optimization_hint: ExprId) -> Self {
        self.optimization_hint = Some(optimization_hint);
        self
    }

    /// Builder method to set compatibility metadata
    pub fn with_compatibility_metadata(mut self, compatibility_metadata: Vec<EffectCompatibility>) -> Self {
        self.compatibility_metadata = compatibility_metadata;
        self
    }

    /// Builder method to set resource preferences
    pub fn with_resource_preferences(mut self, resource_preferences: Vec<ResourcePreference>) -> Self {
        self.resource_preferences = resource_preferences;
        self
    }

    /// Builder method to set target typed domain
    pub fn with_target_typed_domain(mut self, target_typed_domain: TypedDomain) -> Self {
        self.target_typed_domain = Some(target_typed_domain);
        self
    }

    /// Builder method to set process dataflow hint
    pub fn with_process_dataflow_hint(mut self, process_dataflow_hint: ProcessDataflowInitiationHint) -> Self {
        self.process_dataflow_hint = Some(process_dataflow_hint);
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
            optimization_hint: None,
            compatibility_metadata: Vec::new(),
            resource_preferences: Vec::new(),
            target_typed_domain: None,
            process_dataflow_hint: None,
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

impl HasDomainId for Intent {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
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
        
        // Encode each field in order
        bytes.extend_from_slice(&self.id.as_ssz_bytes());
        bytes.extend_from_slice(&self.name.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.priority.to_le_bytes());
        bytes.extend_from_slice(&self.inputs.as_ssz_bytes());
        bytes.extend_from_slice(&self.outputs.as_ssz_bytes());
        
        // Handle optional expression
        if let Some(ref expr) = self.expression {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&expr.as_ssz_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        bytes.extend_from_slice(&self.timestamp.as_ssz_bytes());
        
        // Handle optional optimization hint
        if let Some(ref hint) = self.optimization_hint {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&hint.as_ssz_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        // Handle compatibility metadata
        bytes.extend_from_slice(&self.compatibility_metadata.as_ssz_bytes());
        
        // Handle resource preferences
        bytes.extend_from_slice(&self.resource_preferences.as_ssz_bytes());
        
        // Handle optional target typed domain
        if let Some(ref domain) = self.target_typed_domain {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&domain.as_ssz_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        // Handle optional process dataflow hint
        if let Some(ref hint) = self.process_dataflow_hint {
            bytes.push(1); // Some marker
            bytes.extend_from_slice(&hint.as_ssz_bytes());
        } else {
            bytes.push(0); // None marker
        }
        
        bytes
    }
}

impl Decode for Intent {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode each field in order
        let id = EntityId::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let name = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += name.as_ssz_bytes().len();
        
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..])?;
        offset += domain_id.as_ssz_bytes().len();
        
        if offset + 4 > bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for priority".to_string() });
        }
        let priority = u32::from_le_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]
        ]);
        offset += 4;
        
        let inputs = Vec::<ResourceFlow>::from_ssz_bytes(&bytes[offset..])?;
        offset += inputs.as_ssz_bytes().len();
        
        let outputs = Vec::<ResourceFlow>::from_ssz_bytes(&bytes[offset..])?;
        offset += outputs.as_ssz_bytes().len();
        
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
        offset += timestamp.as_ssz_bytes().len();
        
        // Handle optional optimization hint
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for optimization hint marker".to_string() });
        }
        
        let optimization_hint = if bytes[offset] == 1 {
            offset += 1;
            Some(ExprId::from_ssz_bytes(&bytes[offset..])?)
        } else {
            offset += 1;
            None
        };
        
        if let Some(ref hint) = optimization_hint {
            offset += hint.as_ssz_bytes().len();
        }
        
        // Handle compatibility metadata
        let compatibility_metadata = Vec::<EffectCompatibility>::from_ssz_bytes(&bytes[offset..])?;
        offset += compatibility_metadata.as_ssz_bytes().len();
        
        // Handle resource preferences
        let resource_preferences = Vec::<ResourcePreference>::from_ssz_bytes(&bytes[offset..])?;
        offset += resource_preferences.as_ssz_bytes().len();
        
        // Handle optional target typed domain
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for target typed domain marker".to_string() });
        }
        
        let target_typed_domain = if bytes[offset] == 1 {
            offset += 1;
            Some(TypedDomain::from_ssz_bytes(&bytes[offset..])?)
        } else {
            offset += 1;
            None
        };
        
        if let Some(ref domain) = target_typed_domain {
            offset += domain.as_ssz_bytes().len();
        }
        
        // Handle optional process dataflow hint
        if offset >= bytes.len() {
            return Err(DecodeError { message: "Insufficient bytes for process dataflow hint marker".to_string() });
        }
        
        let process_dataflow_hint = if bytes[offset] == 1 {
            // offset += 1; // Not needed since we don't use offset after this
            Some(ProcessDataflowInitiationHint::from_ssz_bytes(&bytes[offset + 1..])?)
        } else {
            // offset += 1; // Not needed since we don't use offset after this
            None
        };
        
        if let Some(ref hint) = process_dataflow_hint {
            let _ = hint.as_ssz_bytes().len(); // Consume the length but don't update offset since it's not used
        }
        
        Ok(Intent {
            id,
            name,
            domain_id,
            priority,
            inputs,
            outputs,
            expression,
            timestamp,
            optimization_hint,
            compatibility_metadata,
            resource_preferences,
            target_typed_domain,
            process_dataflow_hint,
        })
    }
}

impl SimpleSerialize for Intent {} 