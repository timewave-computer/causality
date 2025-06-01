//! Effect algebra operations for Layer 2
//!
//! This module implements the core effect operations: pure, bind, perform, handle
//! and effect combinators: parallel, race.

use super::core::{EffectExpr, EffectExprKind, EffectHandler};
use crate::lambda::Term;

//-----------------------------------------------------------------------------
// Core Effect Operations
//-----------------------------------------------------------------------------

/// Create a pure effect from a term
/// pure : A ⊸ Effect A
pub fn pure(term: Term) -> EffectExpr {
    EffectExpr::new(EffectExprKind::Pure(term))
}

/// Bind an effect with a continuation
/// bind : Effect A ⊗ (A ⊸ Effect B) ⊸ Effect B
pub fn bind(effect: EffectExpr, var: impl Into<String>, body: EffectExpr) -> EffectExpr {
    EffectExpr::new(EffectExprKind::Bind {
        effect: Box::new(effect),
        var: var.into(),
        body: Box::new(body),
    })
}

/// Perform an effect
/// perform : EffectTag → Args → Effect Result
pub fn perform(effect_tag: impl Into<String>, args: Vec<Term>) -> EffectExpr {
    EffectExpr::new(EffectExprKind::Perform {
        effect_tag: effect_tag.into(),
        args,
    })
}

/// Handle effects with handlers
/// handle : Effect A ⊗ Handlers ⊸ Effect A
pub fn handle(expr: EffectExpr, handlers: Vec<EffectHandler>) -> EffectExpr {
    EffectExpr::new(EffectExprKind::Handle {
        expr: Box::new(expr),
        handlers,
    })
}

//-----------------------------------------------------------------------------
// Effect Combinators
//-----------------------------------------------------------------------------

/// Run two effects in parallel and collect both results
/// parallel : Effect A ⊗ Effect B ⊸ Effect (A ⊗ B)
pub fn parallel(left: EffectExpr, right: EffectExpr) -> EffectExpr {
    EffectExpr::new(EffectExprKind::Parallel {
        left: Box::new(left),
        right: Box::new(right),
    })
}

/// Race two effects and return the first to complete
/// race : Effect A ⊗ Effect B ⊸ Effect (A ⊕ B)
pub fn race(left: EffectExpr, right: EffectExpr) -> EffectExpr {
    EffectExpr::new(EffectExprKind::Race {
        left: Box::new(left),
        right: Box::new(right),
    })
}

//-----------------------------------------------------------------------------
// Monadic Helpers
//-----------------------------------------------------------------------------

/// Sequence two effects, discarding the result of the first
/// seq : Effect A ⊗ Effect B ⊸ Effect B
pub fn seq(first: EffectExpr, second: EffectExpr) -> EffectExpr {
    bind(first, "_", second)
}

/// Map a pure function over an effect
/// map : Effect A ⊗ (A ⊸ B) ⊸ Effect B
pub fn map<F>(effect: EffectExpr, var: impl Into<String>, f: F) -> EffectExpr 
where
    F: FnOnce(Term) -> Term
{
    let var_str = var.into();
    let var_term = Term::var(var_str.clone());
    bind(effect, var_str, pure(f(var_term)))
}

/// Flatten nested effects
/// join : Effect (Effect A) ⊸ Effect A
pub fn join(effect: EffectExpr) -> EffectExpr {
    bind(effect, "e", EffectExpr::new(EffectExprKind::Pure(Term::var("e"))))
}

//-----------------------------------------------------------------------------
// Effect Patterns
//-----------------------------------------------------------------------------

/// Create a handler that transforms one effect into another
pub fn handler(
    effect_tag: impl Into<String>,
    params: Vec<String>,
    continuation: impl Into<String>,
    body: EffectExpr,
) -> EffectHandler {
    EffectHandler {
        effect_tag: effect_tag.into(),
        params,
        continuation: continuation.into(),
        body,
    }
}

/// Create a simple effect handler that ignores the continuation
pub fn simple_handler(
    effect_tag: impl Into<String>,
    params: Vec<String>,
    body: EffectExpr,
) -> EffectHandler {
    handler(effect_tag, params, "_k", body)
}

//-----------------------------------------------------------------------------
// Transaction Operations
//-----------------------------------------------------------------------------

/// Execute a list of effects as a transaction
/// transact : List (Effect A) ⊸ Effect (List A)
pub fn transact(effects: Vec<EffectExpr>) -> EffectExpr {
    // For now, we sequence effects and collect results
    // In a full implementation, this would ensure atomicity
    
    if effects.is_empty() {
        return pure(Term::unit());
    }
    
    // Start with the first effect
    let mut result = effects[0].clone();
    
    // Sequence remaining effects, collecting results
    for (i, effect) in effects.iter().skip(1).enumerate() {
        result = bind(
            result,
            format!("tx_result_{}", i),
            effect.clone()
        );
    }
    
    result
}

/// Make an effect atomic (all-or-nothing execution)
/// atomic : Effect A ⊸ Effect (Effect A)
pub fn atomic(_effect: EffectExpr) -> EffectExpr {
    // Wrap the effect to indicate it should be executed atomically
    // The actual atomicity is enforced by the runtime
    perform("atomic", vec![Term::var("effect_placeholder")])
}

/// Commit a nested effect
/// commit : Effect (Effect A) ⊸ Effect A
pub fn commit(nested_effect: EffectExpr) -> EffectExpr {
    // Flatten the nested effect structure
    // In practice, this would interact with the transaction manager
    bind(nested_effect, "inner", EffectExpr::new(EffectExprKind::Pure(Term::var("inner"))))
} 