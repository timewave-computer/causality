//! Integration with causality-runtime
//!
//! This module provides the integration between causality-zk and causality-runtime,
//! enabling execution traces to be converted to ZK witnesses and providing
//! verification feedback to the runtime.

// Purpose: Core runtime functionality for ZK circuit execution.

use alloc::{vec::Vec, string::String};
use causality_types::{
    core::id::{ResourceId, EffectId, GraphId, ExprId},
    ResourceState,
    ExecutionTrace,
    serialization::{SimpleSerialize, Encode},
};

use crate::core::Error;
use crate::witness::core::{WitnessData, AsWitness};

extern crate alloc;
use alloc::{collections::BTreeMap};

//-----------------------------------------------------------------------------
// Execution Trace Processing
//-----------------------------------------------------------------------------

/// Process execution trace and convert to witness data
pub fn process_execution_trace(
    trace: &ExecutionTrace,
) -> Result<WitnessData, Error> {
    // Use the AsWitness trait implementation
    trace.to_witness()
}

/// Validate that an execution trace is suitable for ZK processing
fn validate_trace(trace: &ExecutionTrace) -> Result<(), Error> {
    // Check for minimum required data
    if trace.executed_effects.is_empty() {
        return Err(Error::InvalidInput(
            "Execution trace contains no effects".to_string(),
        ));
    }

    // Verify that all resources in the trace are properly formatted
    for state in trace.final_resource_states.values() {
        // In an actual implementation, we would validate resource format here
        // For now, we just check that it's a valid state
        match state {
            ResourceState::Available
            | ResourceState::Consumed
            | ResourceState::Locked => {}
        }
    }

    Ok(())
}

//-----------------------------------------------------------------------------
// Runtime Feedback
//-----------------------------------------------------------------------------

/// Verification result for an execution trace
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// The graph ID that was verified
    pub graph_id: GraphId,

    /// Whether the verification was successful
    pub success: bool,

    /// List of effect IDs that were verified
    pub verified_effects: Vec<EffectId>,

    /// List of constraints that failed (empty if successful)
    pub failed_constraints: Vec<usize>,

    /// Optional error message for failed verifications
    pub error_message: Option<String>,
}

impl VerificationResult {
    /// Create a successful verification result
    pub fn success(graph_id: GraphId, verified_effects: Vec<EffectId>) -> Self {
        Self {
            graph_id,
            success: true,
            verified_effects,
            failed_constraints: Vec::new(),
            error_message: None,
        }
    }

    /// Create a failed verification result
    pub fn failure(
        graph_id: GraphId,
        verified_effects: Vec<EffectId>,
        failed_constraints: Vec<usize>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            graph_id,
            success: false,
            verified_effects,
            failed_constraints,
            error_message,
        }
    }
}

impl SimpleSerialize for VerificationResult {}

impl Encode for VerificationResult {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.graph_id.as_ssz_bytes());
        buf.extend_from_slice(&self.success.as_ssz_bytes());
        buf.extend_from_slice(&self.verified_effects.as_ssz_bytes());
        
        // Convert Vec<usize> to Vec<u64> for SSZ serialization
        let failed_constraints_u64: Vec<u64> = self.failed_constraints.iter().map(|&x| x as u64).collect();
        buf.extend_from_slice(&failed_constraints_u64.as_ssz_bytes());
        
        buf.extend_from_slice(&self.error_message.as_ssz_bytes());
        buf
    }
}

/// Result of ZK execution
#[derive(Debug, Clone)]
pub struct ZkExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Verification result
    pub verification: VerificationResult,
    /// Generated witness data
    pub witness: Option<WitnessData>,
}

impl ZkExecutionResult {
    /// Create a successful execution result
    pub fn success(verification: VerificationResult, witness: WitnessData) -> Self {
        Self {
            success: true,
            verification,
            witness: Some(witness),
        }
    }

    /// Create a failed execution result
    pub fn failure(verification: VerificationResult) -> Self {
        Self {
            success: false,
            verification,
            witness: None,
        }
    }
}

//-----------------------------------------------------------------------------
// Resource Conversion Utilities
//-----------------------------------------------------------------------------

/// Convert runtime resources to ZK resources
pub fn convert_resources(
    resource_states: &BTreeMap<ResourceId, ResourceState>,
) -> Vec<ZkResource> {
    resource_states
        .iter()
        .map(|(id, state)| ZkResource::new(*id, *state))
        .collect()
}

/// Convert runtime effects to ZK effects
pub fn convert_effects(
    effect_ids: &[EffectId],
    resource_inputs: &[Vec<ResourceId>],
    resource_outputs: &[Vec<ResourceId>],
) -> Result<Vec<ZkEffect>, Error> {
    if effect_ids.len() != resource_inputs.len() || effect_ids.len() != resource_outputs.len() {
        return Err(Error::InvalidArgument(
            "Mismatched lengths in convert_effects".to_string(),
        ));
    }

    let mut effects = Vec::new();
    for i in 0..effect_ids.len() {
        let effect = ZkEffect {
            id: effect_ids[i],
            inputs: resource_inputs[i].clone(),
            outputs: resource_outputs[i].clone(),
            constraints: Vec::new(), // Add constraints if needed
        };
        effects.push(effect);
    }
    Ok(effects)
}

//-----------------------------------------------------------------------------
// ZK-Compatible Resource Types
//-----------------------------------------------------------------------------

/// Minimal resource representation for ZK circuits
#[derive(Clone)]
pub struct ZkResource {
    /// Resource identifier
    pub id: ResourceId,
    /// Current state
    pub state: ResourceState,
    /// Type identifier (optional, only if needed for validation)
    pub type_id: Option<[u8; 32]>,
}

impl SimpleSerialize for ZkResource {}

impl Encode for ZkResource {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.as_ssz_bytes());
        bytes.extend(self.state.as_ssz_bytes());
        bytes.extend(self.type_id.as_ssz_bytes());
        bytes
    }
}

impl ZkResource {
    /// Create a new ZkResource
    pub fn new(id: ResourceId, state: ResourceState) -> Self {
        Self {
            id,
            state,
            type_id: None,
        }
    }

    /// Create a ZkResource with specific type
    pub fn with_type(
        id: ResourceId,
        state: ResourceState,
        type_id: [u8; 32],
    ) -> Self {
        Self {
            id,
            state,
            type_id: Some(type_id),
        }
    }
}

//-----------------------------------------------------------------------------
// ZK-Compatible Effect Types
//-----------------------------------------------------------------------------

/// Minimal effect representation for ZK circuits
#[derive(Clone)]
pub struct ZkEffect {
    /// Effect identifier
    pub id: EffectId,
    /// Input resource IDs
    pub inputs: Vec<ResourceId>,
    /// Output resource IDs
    pub outputs: Vec<ResourceId>,
    /// Constraint expression IDs
    pub constraints: Vec<ExprId>,
}

impl SimpleSerialize for ZkEffect {}

impl Encode for ZkEffect {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.as_ssz_bytes());
        bytes.extend(self.inputs.as_ssz_bytes());
        bytes.extend(self.outputs.as_ssz_bytes());
        bytes.extend(self.constraints.as_ssz_bytes());
        bytes
    }
}

impl ZkEffect {
    /// Create a new ZkEffect
    pub fn new(id: EffectId) -> Self {
        Self {
            id,
            inputs: Vec::new(),
            outputs: Vec::new(),
            constraints: Vec::new(),
        }
    }
}

//-----------------------------------------------------------------------------
// Processing Functions
//-----------------------------------------------------------------------------

pub fn verify_execution_trace(trace: &ExecutionTrace) -> Result<WitnessData, Error> {
    trace.to_witness()
}
