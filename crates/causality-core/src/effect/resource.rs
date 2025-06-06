//! Resource algebra operations for Layer 2
//!
//! This module implements the resource algebra operations: produce, transform, combine, split

use super::core::EffectExpr;
use super::operations::{perform, bind};
use crate::lambda::Term;

//-----------------------------------------------------------------------------
// Resource Operations as Effects
//-----------------------------------------------------------------------------

/// Produce a new resource given a capability and data
/// produce : Capability ⊗ Data ⊸ Effect (Resource Data)
pub fn produce(capability: Term, data: Term) -> EffectExpr {
    perform("produce", vec![capability, data])
}

/// Transform a resource with a function
/// transform : Resource A ⊗ (A ⊸ B) ⊸ Effect (Resource B)
pub fn transform(resource: Term, f: Term) -> EffectExpr {
    perform("transform", vec![resource, f])
}

/// Combine two resources into a tensor product
/// combine : Resource A ⊗ Resource B ⊸ Effect (Resource (A ⊗ B))
pub fn combine(res_a: Term, res_b: Term) -> EffectExpr {
    perform("combine", vec![res_a, res_b])
}

/// Split a sum resource into its components
/// split : Resource (A ⊕ B) ⊸ Effect ((Resource A) ⊕ (Resource B))
pub fn split(sum_resource: Term) -> EffectExpr {
    perform("split", vec![sum_resource])
}

//-----------------------------------------------------------------------------
// Higher-level Resource Patterns
//-----------------------------------------------------------------------------

/// Transfer a resource from one owner to another
pub fn transfer(resource: Term, from: Term, to: Term) -> EffectExpr {
    // First transform the resource to update ownership
    let update_owner = Term::lambda("r", Term::var("r")); // Placeholder
    bind(
        transform(resource, update_owner),
        "updated",
        perform("transfer", vec![Term::var("updated"), from, to])
    )
}

/// Split a fungible resource into two parts
pub fn split_fungible(resource: Term, amount1: Term, amount2: Term) -> EffectExpr {
    perform("split_fungible", vec![resource, amount1, amount2])
}

/// Merge two compatible resources
pub fn merge(res1: Term, res2: Term) -> EffectExpr {
    bind(
        combine(res1, res2),
        "combined",
        perform("merge", vec![Term::var("combined")])
    )
}

//-----------------------------------------------------------------------------
// Resource Capabilities
//-----------------------------------------------------------------------------

/// Check if an actor has a capability for a resource operation
pub fn has_capability(actor: Term, capability: Term) -> EffectExpr {
    perform("has_capability", vec![actor, capability])
}

/// Grant a capability to an actor
pub fn grant_capability(actor: Term, capability: Term) -> EffectExpr {
    perform("grant_capability", vec![actor, capability])
}

/// Revoke a capability from an actor
pub fn revoke_capability(actor: Term, capability: Term) -> EffectExpr {
    perform("revoke_capability", vec![actor, capability])
}

//-----------------------------------------------------------------------------
// Resource Invariants
//-----------------------------------------------------------------------------

/// Assert a conservation law holds for a resource transformation
pub fn assert_conservation(_inputs: Vec<Term>, _outputs: Vec<Term>) -> EffectExpr {
    let inputs_term = Term::var("inputs"); // Placeholder for list
    let outputs_term = Term::var("outputs"); // Placeholder for list
    perform("assert_conservation", vec![inputs_term, outputs_term])
}

/// Check if a resource satisfies a predicate
pub fn check_resource(resource: Term, predicate: Term) -> EffectExpr {
    perform("check_resource", vec![resource, predicate])
} 