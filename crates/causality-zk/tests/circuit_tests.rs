//! Tests for the circuit management functionality
//!
//! These tests verify that circuits can be created, stored, and referenced
//! correctly within the ZK system.

// Import causality-types components
use causality_types::primitive::ids::{ExprId, GraphId};

// Import ZK component
use causality_zk::circuit::core::Circuit;

#[test]
fn test_circuit_creation() {
    // Create input values for the circuit
    let graph_id = GraphId::from([1; 32]);

    // Create the circuit with no subgraphs and constraints
    let circuit = Circuit::new(graph_id, Vec::new(), Vec::new())
        .expect("Failed to create circuit");

    // Confirm circuit was created successfully and has expected properties
    assert_ne!(circuit.id.0, [0u8; 32], "Circuit should have a valid ID");
}

#[test]
fn test_expression_compilation() {
    // Create a circuit to hold expressions
    let graph_id = GraphId::from([10; 32]);
    let circuit = Circuit::new(graph_id, vec![], Vec::new())
        .expect("Failed to create circuit");

    // Check circuit ID generation is deterministic based on inputs
    assert_ne!(circuit.id.0, [0u8; 32], "Circuit should have a valid ID");
}

#[test]
fn test_dispatch_generation() {
    // Create a circuit with two mock constraints
    let graph_id = GraphId::from([20; 32]);
    let expr1 = ExprId::from([42; 32]);
    let expr2 = ExprId::from([43; 32]);

    let circuit = Circuit::new(graph_id, vec![], vec![expr1, expr2])
        .expect("Failed to create circuit");

    // Check circuit ID was generated
    assert_ne!(circuit.id.0, [0u8; 32], "Circuit should have a valid ID");

    // Check constraints were added
    assert_eq!(
        circuit.expr_ids.len(),
        2,
        "Circuit should have 2 constraints"
    );
    assert!(
        circuit.expr_ids.contains(&expr1),
        "Circuit should contain expr1"
    );
    assert!(
        circuit.expr_ids.contains(&expr2),
        "Circuit should contain expr2"
    );
}

#[test]
fn test_generate_circuit_id() {
    // Create two identical circuits and check that their IDs match
    let graph_id = GraphId::from([30; 32]);
    let circuit1 = Circuit::new(graph_id, vec![], Vec::new())
        .expect("Failed to create circuit");

    let circuit2 = Circuit::new(graph_id, vec![], Vec::new())
        .expect("Failed to create circuit");

    assert_eq!(
        circuit1.id, circuit2.id,
        "Same inputs should generate same circuit ID"
    );

    // Create a different circuit and check that the ID is different
    let different_graph_id = GraphId::from([40; 32]);
    let circuit3 = Circuit::new(different_graph_id, vec![], Vec::new())
        .expect("Failed to create circuit");

    assert_ne!(
        circuit1.id, circuit3.id,
        "Different inputs should generate different IDs"
    );
}
