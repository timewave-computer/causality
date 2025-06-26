//! Causality tracking operations for Layer 2
//!
//! This module implements causality operations: check, depend, sequence, verify

use super::core::EffectExpr;
use super::operations::{pure, bind, perform};
use crate::lambda::Term;

//-----------------------------------------------------------------------------
// Causality Operations
//-----------------------------------------------------------------------------

/// Check conservation laws between input and output resources
/// check : List (Resource A) ⊗ List (Resource A) ⊸ Effect 1
pub fn check(inputs: Term, outputs: Term) -> EffectExpr {
    perform("check", vec![inputs, outputs])
}

/// Establish a causal dependency between two resources
/// depend : Resource A ⊗ Resource B ⊸ Effect (A, B, Proof)
pub fn depend(resource_a: Term, resource_b: Term) -> EffectExpr {
    perform("depend", vec![resource_a, resource_b])
}

/// Sequence two causal proofs to establish transitivity
/// sequence : (A, B, Proof) ⊗ (B, C, Proof) ⊸ Effect (A, C, Proof)
pub fn sequence(proof_ab: Term, proof_bc: Term) -> EffectExpr {
    perform("sequence", vec![proof_ab, proof_bc])
}

/// Verify a causal relationship proof
/// verify : (A, B, Proof) ⊸ Effect Proof
pub fn verify(causal_triple: Term) -> EffectExpr {
    perform("verify", vec![causal_triple])
}

//-----------------------------------------------------------------------------
// Causal Patterns
//-----------------------------------------------------------------------------

/// Create a causal chain from a list of resources
pub fn causal_chain(resources: Vec<Term>) -> EffectExpr {
    match resources.as_slice() {
        [] => pure(Term::unit()),
        [single] => pure(single.clone()),
        [first, second] => depend(first.clone(), second.clone()),
        [first, second, rest @ ..] => {
            bind(
                depend(first.clone(), second.clone()),
                "proof_1_2",
                chain_rest(second.clone(), rest.to_vec(), Term::var("proof_1_2"))
            )
        }
    }
}

fn chain_rest(prev: Term, rest: Vec<Term>, acc_proof: Term) -> EffectExpr {
    match rest.as_slice() {
        [] => pure(acc_proof),
        [next, tail @ ..] => {
            bind(
                depend(prev.clone(), next.clone()),
                "next_proof",
                bind(
                    sequence(acc_proof, Term::var("next_proof")),
                    "combined_proof",
                    chain_rest(next.clone(), tail.to_vec(), Term::var("combined_proof"))
                )
            )
        }
    }
}

/// Assert that a transformation preserves causality
pub fn assert_causality_preserved(
    input_deps: Vec<(Term, Term)>,
    output_deps: Vec<(Term, Term)>
) -> EffectExpr {
    // Create proofs for all input dependencies
    let _input_proofs = input_deps.into_iter()
        .map(|(a, b)| depend(a, b))
        .collect::<Vec<_>>();
    
    // Create proofs for all output dependencies
    let _output_proofs = output_deps.into_iter()
        .map(|(a, b)| depend(a, b))
        .collect::<Vec<_>>();
    
    // For now, just check that we can create all dependencies
    // In a real implementation, we'd verify the transformation preserves the causal structure
    perform("assert_causality", vec![])
}

//-----------------------------------------------------------------------------
// Temporal Ordering
//-----------------------------------------------------------------------------

/// Establish a happens-before relationship
pub fn happens_before(event_a: Term, event_b: Term) -> EffectExpr {
    perform("happens_before", vec![event_a, event_b])
}

/// Check if two events are concurrent (no causal relationship)
pub fn concurrent(event_a: Term, event_b: Term) -> EffectExpr {
    perform("concurrent", vec![event_a, event_b])
}

/// Create a causal barrier that ensures all prior effects complete
pub fn causal_barrier(effects: Vec<Term>) -> EffectExpr {
    if effects.is_empty() {
        pure(Term::unit())
    } else {
        // Create a proper causal barrier by sequencing all effects
        let mut barrier_effect = pure(Term::unit());
        
        for effect in effects {
            barrier_effect = bind(
                barrier_effect,
                "_",
                bind(
                    perform("wait_for_completion", vec![effect]),
                    "_",
                    pure(Term::unit())
                )
            );
        }
        
        // Add final synchronization barrier
        bind(
            barrier_effect,
            "_",
            perform("synchronize_barrier", vec![])
        )
    }
}

//-----------------------------------------------------------------------------
// Causal Consistency
//-----------------------------------------------------------------------------

/// Verify that a set of operations maintains causal consistency
pub fn verify_causal_consistency(operations: Vec<Term>) -> EffectExpr {
    if operations.is_empty() {
        pure(Term::unit())
    } else {
        // Build dependency graph from operations
        let operations_term = if operations.is_empty() {
            Term::unit()
        } else {
            // Represent list as nested tensor products: (op1, (op2, (op3, unit)))
            operations.into_iter().fold(Term::unit(), |acc, op| {
                Term::tensor(op, acc)
            })
        };
        
        let dependency_check = perform("build_dependency_graph", vec![
            Term::lambda("ops", operations_term.clone())
        ]);
        
        // Verify no cycles exist in the dependency graph
        let cycle_check = bind(
            dependency_check,
            "dep_graph",
            perform("check_acyclic", vec![Term::var("dep_graph")])
        );
        
        // Verify all causality constraints are satisfied
        bind(
            cycle_check,
            "acyclic_proof",
            bind(
                perform("verify_constraints", vec![
                    Term::lambda("ops", operations_term),
                    Term::var("acyclic_proof")
                ]),
                "constraint_proof",
                pure(Term::var("constraint_proof"))
            )
        )
    }
}

/// Create a causal snapshot of the current state
pub fn causal_snapshot() -> EffectExpr {
    perform("snapshot", vec![])
}

/// Restore state to a previous causal snapshot
pub fn restore_snapshot(snapshot: Term) -> EffectExpr {
    perform("restore", vec![snapshot])
} 