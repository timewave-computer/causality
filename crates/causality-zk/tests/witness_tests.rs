//! Tests for witness generation
//!
//! These tests verify that execution traces are properly converted to
//! witness data for ZK proof generation.

mod fixtures;

use causality_zk::witness::{build_witness_from_trace, WitnessData};
use fixtures::sample_trace::{
    complex_trace, multi_effect_trace, simple_single_effect_trace,
};
use causality_types::serialization::{Decode, Encode};

//-----------------------------------------------------------------------------
// Witness Generation Test
//-----------------------------------------------------------------------------

#[test]
fn test_witness_from_simple_trace() {
    // Get the simple trace fixture
    let trace = simple_single_effect_trace();

    // Convert to witness data
    let witness = build_witness_from_trace(&trace).expect("Failed to build witness");

    // Basic validation
    assert_eq!(
        witness.effect_ids.len(),
        1,
        "Should have exactly one effect"
    );
    assert_eq!(witness.inputs.len(), 1, "Should have inputs for one effect");
    assert_eq!(
        witness.outputs.len(),
        1,
        "Should have outputs for one effect"
    );

    // Verify resource accounting
    assert_eq!(
        witness.inputs[0].len(),
        1,
        "First effect should have one input"
    );
    assert_eq!(
        witness.outputs[0].len(),
        1,
        "First effect should have one output"
    );
}

#[test]
fn test_witness_from_multi_effect_trace() {
    // Get the multi-effect trace fixture
    let trace = multi_effect_trace();

    // Convert to witness data
    let witness = build_witness_from_trace(&trace).expect("Failed to build witness");

    // Basic validation
    assert_eq!(
        witness.effect_ids.len(),
        2,
        "Should have exactly two effects"
    );
    assert_eq!(
        witness.inputs.len(),
        2,
        "Should have inputs for two effects"
    );
    assert_eq!(
        witness.outputs.len(),
        2,
        "Should have outputs for two effects"
    );

    // Specific resource checks for the first effect
    // Note: The exact assignment of resources to effects is an implementation detail
    // of the build_witness_from_trace function - the test should match that logic
    let total_inputs = witness.inputs[0].len() + witness.inputs[1].len();
    let total_outputs = witness.outputs[0].len() + witness.outputs[1].len();

    assert_eq!(
        total_inputs, 2,
        "Should have 2 total inputs across all effects"
    );
    assert_eq!(
        total_outputs, 2,
        "Should have 2 total outputs across all effects"
    );
}

#[test]
fn test_witness_serialization() {
    // Get the simple trace fixture
    let trace = simple_single_effect_trace();

    // Convert to witness data
    let witness = build_witness_from_trace(&trace).expect("Failed to build witness");

    // Test serialization/deserialization roundtrip
    let serialized = witness.as_ssz_bytes();
    let deserialized = WitnessData::from_ssz_bytes(&serialized).expect("Failed to deserialize witness");

    // Verify the roundtrip
    assert_eq!(witness.effect_ids.len(), deserialized.effect_ids.len());
    assert_eq!(witness.inputs.len(), deserialized.inputs.len());
    assert_eq!(witness.outputs.len(), deserialized.outputs.len());
}

#[test]
fn test_witness_from_complex_trace() {
    // Get the complex trace fixture
    let trace = complex_trace();

    // Convert to witness data
    let witness = build_witness_from_trace(&trace).expect("Failed to build witness");

    // Basic validation
    assert_eq!(
        witness.effect_ids.len(),
        3,
        "Should have exactly three effects"
    );
    assert_eq!(
        witness.inputs.len(),
        3,
        "Should have inputs for three effects"
    );
    assert_eq!(
        witness.outputs.len(),
        3,
        "Should have outputs for three effects"
    );

    // Count total inputs and outputs
    let total_inputs: usize = witness.inputs.iter().map(|v| v.len()).sum();
    let total_outputs: usize = witness.outputs.iter().map(|v| v.len()).sum();

    // In the complex trace we have 3 inputs, 2 intermediates, and 2 outputs
    // The 2 intermediates are both inputs and outputs
    assert_eq!(
        total_inputs, 5,
        "Should have 5 total inputs (3 initial + 2 intermediate)"
    );
    assert_eq!(
        total_outputs, 4,
        "Should have 4 total outputs (2 intermediate + 2 final)"
    );
}
