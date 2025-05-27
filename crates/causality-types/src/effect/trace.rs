//! Execution Trace System
//!
//! Defines trace-related types for tracking execution history
//! and state changes in the Causality framework.

use std::collections::BTreeMap;

use crate::primitive::ids::{EffectId, ExprId, ResourceId};
use crate::expression::{ast::Expr as TypesExpr, value::ValueExpr};
use crate::resource::Resource;
use crate::system::serialization::{Decode, Encode, SimpleSerialize, DecodeError};
use crate::resource::state::ResourceState;

//-----------------------------------------------------------------------------
// Execution Trace Type
//-----------------------------------------------------------------------------

/// Represents a trace of an execution, primarily tracking the state changes of resources
/// and the sequence of executed effects.
///
/// This trace can be used for various purposes, including:
/// - Providing input to ZK provers.
/// - Debugging and auditing execution paths.
/// - Replaying or simulating parts of an execution.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionTrace {
    /// Ordered list of Effect IDs that were successfully executed.
    pub executed_effects: Vec<EffectId>,
    /// A map from ResourceId to its state at the conclusion of the traced execution segment.
    /// Using BTreeMap for deterministic serialization, which is important for provers.
    pub final_resource_states: BTreeMap<ResourceId, ResourceState>,
    /// New fields:
    pub effect_details: BTreeMap<EffectId, EffectDetail>,
    pub expr_definitions: BTreeMap<ExprId, TypesExpr>,
    // New fields for ZkContextProvider
    pub context_values: BTreeMap<String, ValueExpr>,
    pub resource_details: BTreeMap<ResourceId, Resource>, // Storing full Resource
                                                          // TODO: Extend with other relevant trace information as needed, e.g.:
                                                          // - Input values to the overall computation/transaction
                                                          // - Output values from the overall computation/transaction
                                                          // - Events emitted during execution
}

//-----------------------------------------------------------------------------
// Execution Trace Implementation
//-----------------------------------------------------------------------------

impl ExecutionTrace {
    /// Creates a new, empty execution trace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or updates the state of a resource in the trace.
    pub fn set_resource_state(
        &mut self,
        resource_id: ResourceId,
        state: ResourceState,
    ) {
        self.final_resource_states.insert(resource_id, state);
    }

    /// Gets the state of a resource from the trace, if present.
    pub fn get_resource_state(
        &self,
        resource_id: &ResourceId,
    ) -> Option<&ResourceState> {
        self.final_resource_states.get(resource_id)
    }

    /// Record an effect execution in the trace.
    pub fn record_effect_execution(&mut self, effect_id: EffectId) {
        self.executed_effects.push(effect_id);
    }
}

//-----------------------------------------------------------------------------
// Effect Trace Types
//-----------------------------------------------------------------------------

/// A record of an effect execution
/// 
/// Uses manual implementation since ID types already have proper serialization
#[derive(Debug, Clone)]
pub struct EffectTrace {
    /// The ID of the effect that was executed
    pub effect_id: EffectId,
    /// The ID of the resource that was affected
    pub resource_id: ResourceId,
    /// Timestamp when the effect was executed
    pub timestamp: u64,
}

impl Encode for EffectTrace {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.effect_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.resource_id.as_ssz_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes
    }
}

impl Decode for EffectTrace {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < (32 + 32 + 8) { // EffectId (32) + ResourceId (32) + u64 (8)
            return Err(DecodeError { message: format!("EffectTrace bytes too short: {}", bytes.len()) });
        }
        let mut offset = 0;
        let effect_id = EffectId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;
        let resource_id = ResourceId::from_ssz_bytes(&bytes[offset..offset+32])?;
        offset += 32;
        
        let mut timestamp_bytes = [0u8; 8];
        timestamp_bytes.copy_from_slice(&bytes[offset..offset+8]);
        let timestamp = u64::from_le_bytes(timestamp_bytes);

        Ok(EffectTrace { effect_id, resource_id, timestamp })
    }
}

impl SimpleSerialize for EffectTrace {}

//-----------------------------------------------------------------------------
// Trace Entry
//-----------------------------------------------------------------------------

/// A trace entry for a transaction
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// Unique identifier for this trace entry
    pub id: u64,
    /// The effect traces that were part of this transaction
    pub effects: Vec<EffectTrace>,
}

//-----------------------------------------------------------------------------
// ZK Execution Trace Type
//-----------------------------------------------------------------------------

/// Represents an execution trace optimized for ZK proof generation and verification
#[derive(Clone, Debug)]
pub struct ZkExecutionTrace {
    /// The input resources that were processed
    pub input_resources: Vec<ResourceState>,

    /// The output resources that were produced
    pub output_resources: Vec<ResourceState>,

    /// The effect IDs that were executed
    pub effect_ids: Vec<EffectId>,

    /// Additional metadata for the execution
    pub metadata: ZkExecutionMetadata,
}

/// Metadata associated with a ZK execution trace
#[derive(Clone, Debug)]
pub struct ZkExecutionMetadata {
    /// Unique identifier for this execution
    pub execution_id: String,

    /// Timestamp when the execution started
    pub timestamp: u64,

    /// Optional trace hash for verification
    pub trace_hash: Option<[u8; 32]>,
}

impl ZkExecutionTrace {
    /// Create a new empty execution trace
    pub fn new(execution_id: String, timestamp: u64) -> Self {
        Self {
            input_resources: Vec::new(),
            output_resources: Vec::new(),
            effect_ids: Vec::new(),
            metadata: ZkExecutionMetadata {
                execution_id,
                timestamp,
                trace_hash: None,
            },
        }
    }

    /// Add an input resource to the trace
    pub fn add_input_resource(&mut self, resource: ResourceState) {
        self.input_resources.push(resource);
    }

    /// Add an output resource to the trace
    pub fn add_output_resource(&mut self, resource: ResourceState) {
        self.output_resources.push(resource);
    }

    /// Add an effect ID to the trace
    pub fn add_effect(&mut self, effect_id: EffectId) {
        self.effect_ids.push(effect_id);
    }
}

// Effect extension struct for trace information
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectDetail {
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
    pub constraints: Vec<ExprId>,
    // Potentially other details like expressions executed within the effect if needed
    // pub expressions_executed: Vec<ExprId>,
}
