//! Sample execution traces for testing
//!
//! This module contains example execution traces for testing witness generation
//! and circuit execution.

// Import required types
use causality_types::primitive::ids::{EffectId, ResourceId};
use causality_types::resource::state::ResourceState;
use causality_types::effect::trace::{ExecutionTrace, EffectDetail};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Sample Resource IDs and Effect ID
//-----------------------------------------------------------------------------

/// Create a sample resource ID for testing

#[allow(dead_code)]
pub fn sample_resource_id(index: u8) -> ResourceId {
    let mut bytes = [0u8; 32];
    bytes[0] = index;
    bytes[1] = 42; // Some consistent pattern for test resources
    ResourceId::from(bytes)
}

/// Create a sample effect ID for testing
#[allow(dead_code)]
pub fn sample_effect_id(index: u8) -> EffectId {
    let mut bytes = [0u8; 32];
    bytes[0] = index;
    bytes[1] = 99; // Some consistent pattern for test effects
    EffectId::from(bytes)
}

//-----------------------------------------------------------------------------
// Sample Execution Trace
//-----------------------------------------------------------------------------

/// Create a simple execution trace with a single effect

#[allow(dead_code)]
pub fn simple_single_effect_trace() -> ExecutionTrace {
    // Create resource IDs
    let input_resource = sample_resource_id(1);
    let output_resource = sample_resource_id(2);

    // Create effect ID
    let effect1 = sample_effect_id(1);

    // Create a basic trace
    let mut resource_states = BTreeMap::new();
    resource_states.insert(input_resource, ResourceState::Consumed);
    resource_states.insert(output_resource, ResourceState::Available);

    // Create effect details
    let mut effect_details = BTreeMap::new();
    effect_details.insert(effect1, EffectDetail {
        inputs: vec![input_resource],
        outputs: vec![output_resource],
        constraints: vec![],
    });

    // Return trace
    ExecutionTrace {
        executed_effects: vec![effect1],
        final_resource_states: resource_states,
        effect_details,
        expr_definitions: BTreeMap::new(),
        context_values: BTreeMap::new(),
        resource_details: BTreeMap::new(),
    }
}

/// Create a trace with multiple chained effects
#[allow(dead_code)]
pub fn multi_effect_trace() -> ExecutionTrace {
    // Create resource IDs
    let input1 = sample_resource_id(1);
    let intermediate = sample_resource_id(2);
    let output1 = sample_resource_id(3);

    // Create effect IDs
    let effect1 = sample_effect_id(1);
    let effect2 = sample_effect_id(2);

    // Create a basic trace
    let mut resource_states = BTreeMap::new();
    resource_states.insert(input1, ResourceState::Consumed);
    resource_states.insert(intermediate, ResourceState::Consumed);
    resource_states.insert(output1, ResourceState::Available);

    // Create effect details
    let mut effect_details = BTreeMap::new();
    effect_details.insert(effect1, EffectDetail {
        inputs: vec![input1],
        outputs: vec![intermediate],
        constraints: vec![],
    });
    effect_details.insert(effect2, EffectDetail {
        inputs: vec![intermediate],
        outputs: vec![output1],
        constraints: vec![],
    });

    // Return trace
    ExecutionTrace {
        executed_effects: vec![effect1, effect2],
        final_resource_states: resource_states,
        effect_details,
        expr_definitions: BTreeMap::new(),
        context_values: BTreeMap::new(),
        resource_details: BTreeMap::new(),
    }
}

/// Create a complex trace with multiple resources and effects
#[allow(dead_code)]
pub fn complex_trace() -> ExecutionTrace {
    // Create resource IDs for inputs, intermediates, outputs
    let input1 = sample_resource_id(1);
    let input2 = sample_resource_id(2);
    let input3 = sample_resource_id(3);

    let intermediate1 = sample_resource_id(4);
    let intermediate2 = sample_resource_id(5);

    let output1 = sample_resource_id(6);
    let output2 = sample_resource_id(7);

    // Create effect IDs
    let effect1 = sample_effect_id(1);
    let effect2 = sample_effect_id(2);
    let effect3 = sample_effect_id(3);

    // Create a basic trace
    let mut resource_states = BTreeMap::new();

    // Mark resource states appropriately
    resource_states.insert(input1, ResourceState::Consumed);
    resource_states.insert(input2, ResourceState::Consumed);
    resource_states.insert(input3, ResourceState::Consumed);

    resource_states.insert(intermediate1, ResourceState::Consumed);
    resource_states.insert(intermediate2, ResourceState::Consumed);

    resource_states.insert(output1, ResourceState::Available);
    resource_states.insert(output2, ResourceState::Available);

    // Create effect details
    let mut effect_details = BTreeMap::new();
    // Effect 1: Takes 2 inputs, produces 1 intermediate
    effect_details.insert(effect1, EffectDetail {
        inputs: vec![input1, input2],
        outputs: vec![intermediate1],
        constraints: vec![],
    });
    // Effect 2: Takes 1 input and 1 intermediate, produces 1 intermediate
    effect_details.insert(effect2, EffectDetail {
        inputs: vec![input3, intermediate1],
        outputs: vec![intermediate2],
        constraints: vec![],
    });
    // Effect 3: Takes 1 intermediate, produces 2 outputs
    effect_details.insert(effect3, EffectDetail {
        inputs: vec![intermediate2],
        outputs: vec![output1, output2],
        constraints: vec![],
    });

    // Return trace
    ExecutionTrace {
        executed_effects: vec![effect1, effect2, effect3],
        final_resource_states: resource_states,
        effect_details,
        expr_definitions: BTreeMap::new(),
        context_values: BTreeMap::new(),
        resource_details: BTreeMap::new(),
    }
}
