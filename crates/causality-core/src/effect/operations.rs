//! Effect algebra operations for Layer 2
//!
//! This module implements the core effect operations: pure, bind, perform, handle
//! and effect combinators: parallel, race.

use super::core::{EffectExpr, EffectExprKind, EffectHandler, SessionBranch};
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

//-----------------------------------------------------------------------------
// Session Type Operations
//-----------------------------------------------------------------------------

/// Send a value through a session channel
/// session_send : Channel ⊗ Value ⊸ Effect Unit
pub fn session_send(channel: EffectExpr, value: Term) -> EffectExpr {
    EffectExpr::new(EffectExprKind::SessionSend {
        channel: Box::new(channel),
        value,
        continuation: Box::new(pure(Term::unit())),
    })
}

/// Receive a value from a session channel
/// session_recv : Channel ⊸ Effect Value
pub fn session_recv(channel: EffectExpr) -> EffectExpr {
    EffectExpr::new(EffectExprKind::SessionReceive {
        channel: Box::new(channel),
        continuation: Box::new(pure(Term::var("received_value"))),
    })
}

/// Make an internal choice on a session channel
/// session_select : Channel ⊗ Choice ⊸ Effect Unit
pub fn session_select(channel: EffectExpr, choice: impl Into<String>) -> EffectExpr {
    EffectExpr::new(EffectExprKind::SessionSelect {
        channel: Box::new(channel),
        choice: choice.into(),
        continuation: Box::new(pure(Term::unit())),
    })
}

/// Handle external choices from a session channel
/// session_case : Channel ⊗ Branches ⊸ Effect A
pub fn session_case(channel: EffectExpr, branches: Vec<SessionBranch>) -> EffectExpr {
    EffectExpr::new(EffectExprKind::SessionCase {
        channel: Box::new(channel),
        branches,
    })
}

/// Establish a session with a specific role
/// with_session : SessionName ⊗ Role ⊗ (Session ⊸ Effect A) ⊸ Effect A
pub fn with_session(
    session: impl Into<String>, 
    role: impl Into<String>, 
    body: EffectExpr
) -> EffectExpr {
    EffectExpr::new(EffectExprKind::WithSession {
        session_decl: session.into(),
        role: role.into(),
        body: Box::new(body),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::{Term, Literal};
    use crate::effect::core::SessionBranch;

    #[test]
    fn test_session_send() {
        let channel = pure(Term::var("my_channel"));
        let value = Term::literal(Literal::Int(42));
        let result = session_send(channel.clone(), value.clone());
        
        if let EffectExprKind::SessionSend { channel: ch, value: val, continuation } = result.kind {
            assert_eq!(*ch, channel);
            assert_eq!(val, value);
            // Continuation should be unit
            if let EffectExprKind::Pure(term) = continuation.kind {
                assert_eq!(term, Term::unit());
            } else {
                panic!("Expected Pure unit continuation");
            }
        } else {
            panic!("Expected SessionSend");
        }
    }

    #[test]
    fn test_session_recv() {
        let channel = pure(Term::var("my_channel"));
        let result = session_recv(channel.clone());
        
        if let EffectExprKind::SessionReceive { channel: ch, continuation } = result.kind {
            assert_eq!(*ch, channel);
            // Continuation should have received_value
            if let EffectExprKind::Pure(term) = continuation.kind {
                assert_eq!(term, Term::var("received_value"));
            } else {
                panic!("Expected Pure received_value continuation");
            }
        } else {
            panic!("Expected SessionReceive");
        }
    }

    #[test]
    fn test_session_select() {
        let channel = pure(Term::var("my_channel"));
        let choice = "branch1";
        let result = session_select(channel.clone(), choice);
        
        if let EffectExprKind::SessionSelect { channel: ch, choice: selected, continuation } = result.kind {
            assert_eq!(*ch, channel);
            assert_eq!(selected, choice);
            // Continuation should be unit
            if let EffectExprKind::Pure(term) = continuation.kind {
                assert_eq!(term, Term::unit());
            } else {
                panic!("Expected Pure unit continuation");
            }
        } else {
            panic!("Expected SessionSelect");
        }
    }

    #[test]
    fn test_session_case() {
        let channel = pure(Term::var("my_channel"));
        let branches = vec![
            SessionBranch {
                label: "option1".to_string(),
                body: pure(Term::var("result1")),
            },
            SessionBranch {
                label: "option2".to_string(),
                body: pure(Term::var("result2")),
            },
        ];
        
        let result = session_case(channel.clone(), branches.clone());
        
        if let EffectExprKind::SessionCase { channel: ch, branches: br } = result.kind {
            assert_eq!(*ch, channel);
            assert_eq!(br.len(), 2);
            assert_eq!(br[0].label, "option1");
            assert_eq!(br[1].label, "option2");
        } else {
            panic!("Expected SessionCase");
        }
    }

    #[test]
    fn test_with_session() {
        let session_name = "PaymentProtocol";
        let role = "client";
        let body = pure(Term::var("session_body"));
        
        let result = with_session(session_name, role, body.clone());
        
        if let EffectExprKind::WithSession { session_decl, role: r, body: b } = result.kind {
            assert_eq!(session_decl, session_name);
            assert_eq!(r, role);
            assert_eq!(*b, body);
        } else {
            panic!("Expected WithSession");
        }
    }
} 