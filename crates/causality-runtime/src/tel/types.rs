//! TEL Type Definitions
//!
//! Defines common types for the TEL interpreter in causality-runtime.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use causality_types::serialization::{SimpleSerialize, Encode, Decode, DecodeError};
use causality_types::primitive::ids::{EffectId, NodeId};
use causality_types::expr::ValueExpr;
use causality_types::graph::traits::{
    AsContainsEdgeType, AsContainsNodeType, AsEdgeTypesList, AsNodeTypesList,
};
use causality_types::resource::Resource;
use causality_types::tel::Edge;
use causality_types::core::{Effect, Handler};
use frunk::{HCons, HNil};

//-----------------------------------------------------------------------------
// Interpreter Types
//-----------------------------------------------------------------------------

// InterpreterMode is now imported from causality_types

//-----------------------------------------------------------------------------
// Execution Events
//-----------------------------------------------------------------------------

/// Represents an event in the execution history of an effect graph.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionEvent {
    /// An effect was processed by a handler.
    EffectProcessed {
        effect_id: EffectId,
        output: Option<ValueExpr>,
    },
    /// The interpreter encountered a blocking condition.
    BlockedOnInputs {
        effect_id: EffectId,
        missing_inputs: Vec<NodeId>, // Or some other identifier for what's missing
    },
    /// The interpreter chose an effect to process next.
    EffectChosen { effect_id: EffectId },
    /// A resource was created.
    ResourceCreated { resource_id: NodeId }, // Using NodeId for now
    /// A resource was consumed/nullified.
    ResourceConsumed {
        resource_id: NodeId,
        nullifier_id: Option<NodeId>,
    }, // Using NodeId
}

impl Encode for ExecutionEvent {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            ExecutionEvent::EffectProcessed { effect_id, output } => {
                bytes.push(0u8); // variant tag
                bytes.extend(effect_id.as_ssz_bytes());
                // Encode the Option<ValueExpr>
                match output {
                    Some(value) => {
                        bytes.push(1u8); // Some tag
                        bytes.extend(value.as_ssz_bytes());
                    }
                    None => {
                        bytes.push(0u8); // None tag
                    }
                }
            }
            ExecutionEvent::BlockedOnInputs { effect_id, missing_inputs } => {
                bytes.push(1u8); // variant tag
                bytes.extend(effect_id.as_ssz_bytes());
                bytes.extend((missing_inputs.len() as u32).to_le_bytes());
                for input in missing_inputs {
                    bytes.extend(input.as_ssz_bytes());
                }
            }
            ExecutionEvent::EffectChosen { effect_id } => {
                bytes.push(2u8); // variant tag
                bytes.extend(effect_id.as_ssz_bytes());
            }
            ExecutionEvent::ResourceCreated { resource_id } => {
                bytes.push(3u8); // variant tag
                bytes.extend(resource_id.as_ssz_bytes());
            }
            ExecutionEvent::ResourceConsumed { resource_id, nullifier_id } => {
                bytes.push(4u8); // variant tag
                bytes.extend(resource_id.as_ssz_bytes());
                match nullifier_id {
                    Some(id) => {
                        bytes.push(1u8); // Some tag
                        bytes.extend(id.as_ssz_bytes());
                    }
                    None => {
                        bytes.push(0u8); // None tag
                    }
                }
            }
        }
        bytes
    }
}

impl Decode for ExecutionEvent {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError { message: "Empty bytes for ExecutionEvent".to_string() });
        }
        
        let mut offset = 0;
        let variant_tag = bytes[offset];
        offset += 1;
        
        match variant_tag {
            0 => { // EffectProcessed
                let effect_id = EffectId::from_ssz_bytes(&bytes[offset..])?;
                offset += effect_id.as_ssz_bytes().len();
                
                if offset >= bytes.len() {
                    return Err(DecodeError { message: "Insufficient data for output option".to_string() });
                }
                
                let output = if bytes[offset] == 1 {
                    offset += 1;
                    Some(ValueExpr::from_ssz_bytes(&bytes[offset..])?)
                } else {
                    None
                };
                
                Ok(ExecutionEvent::EffectProcessed { effect_id, output })
            }
            1 => { // BlockedOnInputs
                let effect_id = EffectId::from_ssz_bytes(&bytes[offset..])?;
                offset += effect_id.as_ssz_bytes().len();
                
                let len = u32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]) as usize;
                offset += 4;
                
                let mut missing_inputs = Vec::with_capacity(len);
                for _ in 0..len {
                    let input = NodeId::from_ssz_bytes(&bytes[offset..])?;
                    offset += input.as_ssz_bytes().len();
                    missing_inputs.push(input);
                }
                
                Ok(ExecutionEvent::BlockedOnInputs { effect_id, missing_inputs })
            }
            2 => { // EffectChosen
                let effect_id = EffectId::from_ssz_bytes(&bytes[offset..])?;
                Ok(ExecutionEvent::EffectChosen { effect_id })
            }
            3 => { // ResourceCreated
                let resource_id = NodeId::from_ssz_bytes(&bytes[offset..])?;
                Ok(ExecutionEvent::ResourceCreated { resource_id })
            }
            4 => { // ResourceConsumed
                let resource_id = NodeId::from_ssz_bytes(&bytes[offset..])?;
                offset += resource_id.as_ssz_bytes().len();
                
                let nullifier_id = if bytes[offset] == 1 {
                    offset += 1;
                    Some(NodeId::from_ssz_bytes(&bytes[offset..])?)
                } else {
                    None
                };
                
                Ok(ExecutionEvent::ResourceConsumed { resource_id, nullifier_id })
            }
            _ => Err(DecodeError { message: format!("Invalid ExecutionEvent variant: {}", variant_tag) })
        }
    }
}

impl SimpleSerialize for ExecutionEvent {}

//-----------------------------------------------------------------------------
// Trace Identifiers
//-----------------------------------------------------------------------------

/// Placeholder for a transaction trace identifier.
/// TODO: Define this properly, possibly from causality-types or causality-api if it exists there.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub struct TransactionTraceId(pub [u8; 32]);

impl TransactionTraceId {
    pub fn new_v4() -> Self {
        // Using a simple fixed ID for placeholder. Replace with actual UUID v4 logic.
        Self([0u8; 32])
    }
}

impl Encode for TransactionTraceId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Decode for TransactionTraceId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 32 {
            return Err(DecodeError { message: format!("Expected 32 bytes for TransactionTraceId, got {}", bytes.len()) });
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(TransactionTraceId(array))
    }
}

impl SimpleSerialize for TransactionTraceId {}

//-----------------------------------------------------------------------------
// TEL Node Types
//-----------------------------------------------------------------------------

// Define HList for TEL Node Types as a newtype to satisfy orphan rules
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TelNodeTypes(pub HCons<Effect, HCons<Resource, HCons<Handler, HNil>>>);
impl AsNodeTypesList for TelNodeTypes {}

// Implement AsContainsNodeType for specific types within TelNodeTypes
impl AsContainsNodeType<Effect> for TelNodeTypes {
    fn is_present() -> bool {
        true
    }
}
impl AsContainsNodeType<Resource> for TelNodeTypes {
    fn is_present() -> bool {
        true
    }
}
impl AsContainsNodeType<Handler> for TelNodeTypes {
    fn is_present() -> bool {
        true
    }
}

//-----------------------------------------------------------------------------
// TEL Edge Types
//-----------------------------------------------------------------------------

// Define HList for TEL Edge Types as a newtype
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TelEdgeTypes(pub HCons<Edge, HNil>);
impl AsEdgeTypesList for TelEdgeTypes {}

// Implement AsContainsEdgeType for specific types within TelEdgeTypes
impl AsContainsEdgeType<Edge> for TelEdgeTypes {
    fn is_present() -> bool {
        true
    } // Edge is the head
}

// Placeholder for a more concrete Graph structure if needed.
// For now, this could represent a collection of TEL effects and edges.
// pub struct TelGraph { ... }
