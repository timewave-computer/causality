//! Witness generation for the Causality ZK system.
//!
//! This module provides functionality for building witnesses from execution traces
//! that can be used for ZK proof generation. Follows the approach described in
//! the Valence Coprocessor architecture document.
//!
//! See:
//! - [Valence Coprocessor Interaction](../../../docs/valence-coprocessor-interaction.md)
//! - [Expression Compilation](../../../docs/expression_compilation.md)

// Use alloc for no_std compatibility
extern crate alloc;
use alloc::vec::Vec;

use causality_types::{
    core::id::{EffectId, ExprId, ResourceId},
    serialization::{Encode, Decode, SimpleSerialize},
    trace::ExecutionTrace,
};
use core::fmt::{self, Debug, Formatter};

// Import from causality_types with minimal dependencies
use crate::core::{CircuitId, Error, WitnessId};
use causality_types::trace::ExecutionTrace as CanonicalExecutionTrace;

use sha2::{Sha256, Digest};

//-----------------------------------------------------------------------------
// Witness Data Types - Minimalistic no_std compatible types
//-----------------------------------------------------------------------------

/// Witness data for ZK proof generation
#[derive(Clone)]
pub struct WitnessData {
    /// Unique identifier for this witness
    pub id: WitnessId,

    /// Circuit that this witness is for
    pub circuit_id: CircuitId,

    /// The ordered sequence of effect IDs in the execution trace
    pub effect_ids: Vec<EffectId>,

    /// The input resources for each effect, indexed by effect position
    pub inputs: Vec<Vec<ResourceId>>,

    /// The output resources for each effect, indexed by effect position
    pub outputs: Vec<Vec<ResourceId>>,

    /// Constraint expressions to validate
    pub constraints: Vec<ExprId>,

    /// Additional private data needed for proof generation
    pub private_data: Vec<u8>,
}

impl SimpleSerialize for WitnessData {}

impl Encode for WitnessData {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        // Use proper SSZ serialization for all fields
        let mut bytes = Vec::new();
        
        // Serialize each field with proper SSZ encoding
        let id_bytes = self.id.0.as_ssz_bytes();
        bytes.extend((id_bytes.len() as u32).to_le_bytes());
        bytes.extend(id_bytes);
        
        let circuit_id_bytes = self.circuit_id.0.as_ssz_bytes();
        bytes.extend((circuit_id_bytes.len() as u32).to_le_bytes());
        bytes.extend(circuit_id_bytes);
        
        let effect_ids_bytes = self.effect_ids.as_ssz_bytes();
        bytes.extend((effect_ids_bytes.len() as u32).to_le_bytes());
        bytes.extend(effect_ids_bytes);
        
        let inputs_bytes = self.inputs.as_ssz_bytes();
        bytes.extend((inputs_bytes.len() as u32).to_le_bytes());
        bytes.extend(inputs_bytes);
        
        let outputs_bytes = self.outputs.as_ssz_bytes();
        bytes.extend((outputs_bytes.len() as u32).to_le_bytes());
        bytes.extend(outputs_bytes);
        
        let constraints_bytes = self.constraints.as_ssz_bytes();
        bytes.extend((constraints_bytes.len() as u32).to_le_bytes());
        bytes.extend(constraints_bytes);
        
        let private_data_bytes = self.private_data.as_ssz_bytes();
        bytes.extend((private_data_bytes.len() as u32).to_le_bytes());
        bytes.extend(private_data_bytes);
        
        bytes
    }
}

impl Decode for WitnessData {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, causality_types::serialization::DecodeError> {
        let mut offset = 0;
        
        // Helper function to read length-prefixed data
        let read_length_prefixed = |offset: &mut usize| -> Result<Vec<u8>, causality_types::serialization::DecodeError> {
            if *offset + 4 > bytes.len() {
                return Err(causality_types::serialization::DecodeError { 
                    message: "Not enough bytes for length prefix".to_string() 
                });
            }
            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&bytes[*offset..*offset + 4]);
            let len = u32::from_le_bytes(len_bytes) as usize;
            *offset += 4;
            
            if *offset + len > bytes.len() {
                return Err(causality_types::serialization::DecodeError { 
                    message: format!("Not enough bytes for data: need {}, have {}", len, bytes.len() - *offset)
                });
            }
            let data = bytes[*offset..*offset + len].to_vec();
            *offset += len;
            Ok(data)
        };
        
        // Decode id
        let id_bytes = read_length_prefixed(&mut offset)?;
        let id = WitnessId(<[u8; 32]>::from_ssz_bytes(&id_bytes)?);
        
        // Decode circuit_id
        let circuit_id_bytes = read_length_prefixed(&mut offset)?;
        let circuit_id = CircuitId(<[u8; 32]>::from_ssz_bytes(&circuit_id_bytes)?);
        
        // Decode effect_ids
        let effect_ids_bytes = read_length_prefixed(&mut offset)?;
        let effect_ids = Vec::<EffectId>::from_ssz_bytes(&effect_ids_bytes)?;
        
        // Decode inputs
        let inputs_bytes = read_length_prefixed(&mut offset)?;
        let inputs = Vec::<Vec<ResourceId>>::from_ssz_bytes(&inputs_bytes)?;
        
        // Decode outputs
        let outputs_bytes = read_length_prefixed(&mut offset)?;
        let outputs = Vec::<Vec<ResourceId>>::from_ssz_bytes(&outputs_bytes)?;
        
        // Decode constraints
        let constraints_bytes = read_length_prefixed(&mut offset)?;
        let constraints = Vec::<ExprId>::from_ssz_bytes(&constraints_bytes)?;
        
        // Decode private_data
        let private_data_bytes = read_length_prefixed(&mut offset)?;
        let private_data = Vec::<u8>::from_ssz_bytes(&private_data_bytes)?;
        
        Ok(WitnessData {
            id,
            circuit_id,
            effect_ids,
            inputs,
            outputs,
            constraints,
            private_data,
        })
    }
}

/// Public inputs for verification
#[derive(Clone)]
pub struct PublicInputs {
    /// Circuit identifier
    pub circuit_id: CircuitId,

    /// Expression IDs used in the circuit (for compiled expression verification)
    pub expr_ids: Vec<ExprId>,

    /// Output commitments
    pub output_commitments: Vec<[u8; 32]>,
}

impl SimpleSerialize for PublicInputs {}

impl Encode for PublicInputs {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.circuit_id.0.as_ssz_bytes());
        bytes.extend(self.expr_ids.as_ssz_bytes());
        bytes.extend(self.output_commitments.as_ssz_bytes());
        bytes
    }
}

// Implement Debug manually to avoid std dependency
impl Debug for WitnessData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("WitnessData")
            .field("id", &self.id)
            .field("circuit_id", &self.circuit_id)
            .field("effect_ids", &self.effect_ids.len())
            .field("inputs", &self.inputs.len())
            .field("outputs", &self.outputs.len())
            .field("constraints", &self.constraints.len())
            .finish()
    }
}

impl Debug for PublicInputs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PublicInputs")
            .field("circuit_id", &self.circuit_id)
            .field("expr_ids", &self.expr_ids)
            .field("output_commitments", &self.output_commitments.len())
            .finish()
    }
}

//-----------------------------------------------------------------------------
// Witness Construction
//-----------------------------------------------------------------------------

/// Trait for types that can be converted to witness data
pub trait AsWitness {
    /// Convert to witness data
    fn to_witness(&self) -> Result<WitnessData, Error>;

    /// Generate a witness ID based on content
    fn witness_id(&self) -> WitnessId;
}

/// Implement AsWitness for ExecutionTrace
impl AsWitness for ExecutionTrace {
    fn to_witness(&self) -> Result<WitnessData, Error> {
        // Extract inputs, outputs, and constraints from effect details
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut constraints = Vec::new();

        for effect_id in &self.executed_effects {
            if let Some(detail) = self.effect_details.get(effect_id) {
                inputs.push(detail.inputs.clone());
                outputs.push(detail.outputs.clone());
                constraints.extend(detail.constraints.clone());
            } else {
                // If no detail available, use empty vectors
                inputs.push(Vec::new());
                outputs.push(Vec::new());
            }
        }

        let data = serialize_execution_trace_simple(self);

        Ok(WitnessData {
            id: self.witness_id(),
            circuit_id: CircuitId::new(&data),
            effect_ids: self.executed_effects.clone(),
            inputs,
            outputs,
            constraints,
            private_data: data,
        })
    }

    fn witness_id(&self) -> WitnessId {
        // Generate a content-based ID using the trace data
        let serialized = serialize_execution_trace_simple(self);

        // Use SHA-256 for content addressing
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let hash: [u8; 32] = hasher.finalize().into();

        WitnessId(hash)
    }
}

/// Helper function to serialize regular execution trace
fn serialize_execution_trace_simple(trace: &ExecutionTrace) -> Vec<u8> {
    // Simple serialization - in production this would use proper SSZ
    let mut data = Vec::new();
    data.extend_from_slice(b"EXEC_TRACE_V2");
    data.extend_from_slice(&(trace.executed_effects.len() as u32).to_le_bytes());
    data.extend_from_slice(&(trace.final_resource_states.len() as u32).to_le_bytes());
    // Add more fields as needed
    data
}

//-----------------------------------------------------------------------------
// Witness Conversion Function
//-----------------------------------------------------------------------------

/// Convert an execution trace to witness data for ZK proof generation
///
/// This is the primary entry point for witness generation and follows the
/// approach described in docs/expression_compilation.md.
pub fn build_witness_from_trace(
    trace: &CanonicalExecutionTrace,
) -> Result<WitnessData, Error> {
    // Simply delegate to the AsWitness trait implementation
    trace.to_witness()
}

impl WitnessData {
    /// Get the constraint expression IDs for this witness
    pub fn get_constraint_expr_ids(&self) -> &[ExprId] {
        &self.constraints
    }

    /// Serialize the execution trace to bytes
    pub fn serialize(&self) -> Vec<u8> {
        // Simple serialization - in production this would use proper SSZ
        let mut data = Vec::new();
        data.extend_from_slice(b"EXEC_TRACE_V1");
        data.extend_from_slice(&(self.effect_ids.len() as u32).to_le_bytes());
        // Add more fields as needed
        data
    }
}

/// Helper function to serialize canonical execution trace to bytes
pub fn serialize_canonical_execution_trace(trace: &CanonicalExecutionTrace) -> Vec<u8> {
    // Simple serialization - in production this would use proper SSZ
    let mut data = Vec::new();
    data.extend_from_slice(b"EXEC_TRACE_V1");
    data.extend_from_slice(&(trace.executed_effects.len() as u32).to_le_bytes());
    // Add more fields as needed
    data
}

//-----------------------------------------------------------------------------
// Witness Type Registry - For type-safe witness handling
//-----------------------------------------------------------------------------

use frunk::{HCons, HNil};
use std::marker::PhantomData;

/// Marker trait for witness type lists using frunk HList.
pub trait WitnessTypesList {}

// Implementation for empty HList
impl WitnessTypesList for HNil {}

// Implementation for non-empty HList
impl<H, T: WitnessTypesList> WitnessTypesList for HCons<H, T> {}

/// Registry for witness types used in the ZK system.
/// This allows for type-safe operations on witness types.
pub struct WitnessRegistry<L: WitnessTypesList> {
    /// Phantom data to track the types list
    _types: PhantomData<L>,
}

impl<L: WitnessTypesList> Default for WitnessRegistry<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: WitnessTypesList> WitnessRegistry<L> {
    /// Create a new witness types registry
    pub fn new() -> Self {
        Self {
            _types: PhantomData,
        }
    }
}
