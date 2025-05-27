//! Unified Transaction type definition for the Causality framework.
//!
//! This module contains the single, canonical Transaction type that replaces
//! all previous scattered transaction definitions.

use crate::primitive::ids::{DomainId, EntityId};
use crate::primitive::string::Str;
use crate::primitive::time::Timestamp;
use crate::primitive::trait_::{
    AsIdentifiable, HasDomainId, HasInputs, HasOutputs, HasTimestamp, AsTransaction
};
use crate::resource::flow::ResourceFlow;
use crate::system::serialization::{Decode, Encode, SimpleSerialize, DecodeError};

/// Unified Transaction type representing a collection of effects and intents
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Unique identifier for this transaction
    pub id: EntityId,
    
    /// Human-readable name or description
    pub name: Str,
    
    /// Domain this transaction belongs to
    pub domain_id: DomainId,
    
    /// All effects included in this transaction
    pub effects: Vec<EntityId>,
    
    /// All intents satisfied by this transaction
    pub intents: Vec<EntityId>,
    
    /// Aggregated resources consumed by all effects
    pub inputs: Vec<ResourceFlow>,
    
    /// Aggregated resources produced by all effects
    pub outputs: Vec<ResourceFlow>,
    
    /// When this transaction was created or executed
    pub timestamp: Timestamp,
}

impl Transaction {
    /// Create a new Transaction instance with required parameters
    pub fn new(
        id: EntityId,
        name: Str,
        domain_id: DomainId,
    ) -> Self {
        Self {
            id,
            name,
            domain_id,
            effects: Vec::new(),
            intents: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            timestamp: Timestamp::now(),
        }
    }

    /// Builder method to set effects
    pub fn with_effects(mut self, effects: Vec<EntityId>) -> Self {
        self.effects = effects;
        self
    }

    /// Builder method to set intents
    pub fn with_intents(mut self, intents: Vec<EntityId>) -> Self {
        self.intents = intents;
        self
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

    /// Builder method to set timestamp
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = timestamp;
        self
    }
}

//-----------------------------------------------------------------------------
// Trait Implementations
//-----------------------------------------------------------------------------

impl AsIdentifiable for Transaction {
    fn id(&self) -> &EntityId {
        &self.id
    }
    
    fn name(&self) -> &Str {
        &self.name
    }
}

impl HasDomainId for Transaction {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
}

impl HasInputs for Transaction {
    fn inputs(&self) -> &[ResourceFlow] {
        &self.inputs
    }
}

impl HasOutputs for Transaction {
    fn outputs(&self) -> &[ResourceFlow] {
        &self.outputs
    }
}

impl HasTimestamp for Transaction {
    fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}

impl AsTransaction for Transaction {
    fn effects(&self) -> &[EntityId] {
        &self.effects
    }
    
    fn intents(&self) -> &[EntityId] {
        &self.intents
    }
    
    fn is_valid(&self) -> bool {
        // Default implementation - basic validation
        // Real implementation would validate effect/intent relationships,
        // resource conservation, etc.
        !self.effects.is_empty()
        // Note: u64 quantities are always non-negative, so no need to check >= 0
    }
}

//-----------------------------------------------------------------------------
// SSZ Serialization
//-----------------------------------------------------------------------------

impl Encode for Transaction {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Encode each field in order
        bytes.extend_from_slice(&self.id.as_ssz_bytes());
        bytes.extend_from_slice(&self.name.as_ssz_bytes());
        bytes.extend_from_slice(&self.domain_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.effects.as_ssz_bytes());
        bytes.extend_from_slice(&self.intents.as_ssz_bytes());
        bytes.extend_from_slice(&self.inputs.as_ssz_bytes());
        bytes.extend_from_slice(&self.outputs.as_ssz_bytes());
        bytes.extend_from_slice(&self.timestamp.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for Transaction {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode each field in order
        let id = EntityId::from_ssz_bytes(&bytes[offset..])?;
        offset += id.as_ssz_bytes().len();
        
        let name = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += name.as_ssz_bytes().len();
        
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..])?;
        offset += domain_id.as_ssz_bytes().len();
        
        let effects = Vec::<EntityId>::from_ssz_bytes(&bytes[offset..])?;
        offset += effects.as_ssz_bytes().len();
        
        let intents = Vec::<EntityId>::from_ssz_bytes(&bytes[offset..])?;
        offset += intents.as_ssz_bytes().len();
        
        let inputs = Vec::<ResourceFlow>::from_ssz_bytes(&bytes[offset..])?;
        offset += inputs.as_ssz_bytes().len();
        
        let outputs = Vec::<ResourceFlow>::from_ssz_bytes(&bytes[offset..])?;
        offset += outputs.as_ssz_bytes().len();
        
        let timestamp = Timestamp::from_ssz_bytes(&bytes[offset..])?;
        
        Ok(Transaction {
            id,
            name,
            domain_id,
            effects,
            intents,
            inputs,
            outputs,
            timestamp,
        })
    }
}

impl SimpleSerialize for Transaction {} 