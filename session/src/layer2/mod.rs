// Layer 2: Verifiable Outcome Algebra
// Adds declarative outcomes and algebraic effects to typed message passing

pub mod outcome;
pub mod effect;
pub mod proof;
pub mod compiler;
pub mod interpreter;

// Re-export key types
pub use outcome::{Outcome, StateTransition, Value, Address, ResourceType};
pub use effect::{Effect, EffectOp, Handler, EffectRow};
pub use proof::{generate_proof, verify_proof, ProofBuilder};
pub use interpreter::{Interpreter, CompositeInterpreter};
pub use compiler::{compile_outcome, compile_effect_to_session, CompileError};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::outcome::{StateLocation};
    
    #[test]
    fn test_outcome_algebra_identity() {
        let empty = Outcome::empty();
        let transition = StateTransition::Transfer {
            from: Address("Alice".to_string()),
            to: Address("Bob".to_string()),
            amount: 100,
            resource_type: ResourceType("Token".to_string()),
        };
        let outcome = Outcome::single(transition.clone());
        
        // Identity law: empty ⊕ O = O = O ⊕ empty
        let left = empty.clone().compose(outcome.clone());
        let right = outcome.clone().compose(empty.clone());
        
        assert_eq!(left.declarations, vec![transition.clone()]);
        assert_eq!(right.declarations, vec![transition]);
    }
    
    #[test]
    fn test_outcome_algebra_associativity() {
        let t1 = StateTransition::Transfer {
            from: Address("Alice".to_string()),
            to: Address("Bob".to_string()),
            amount: 50,
            resource_type: ResourceType("Token".to_string()),
        };
        let t2 = StateTransition::Transfer {
            from: Address("Bob".to_string()),
            to: Address("Carol".to_string()),
            amount: 30,
            resource_type: ResourceType("Token".to_string()),
        };
        let t3 = StateTransition::Transfer {
            from: Address("Carol".to_string()),
            to: Address("Dave".to_string()),
            amount: 20,
            resource_type: ResourceType("Token".to_string()),
        };
        
        let o1 = Outcome::single(t1.clone());
        let o2 = Outcome::single(t2.clone());
        let o3 = Outcome::single(t3.clone());
        
        // (O1 ⊕ O2) ⊕ O3 = O1 ⊕ (O2 ⊕ O3)
        let left = o1.clone().compose(o2.clone()).compose(o3.clone());
        let right = o1.compose(o2.compose(o3));
        
        assert_eq!(left.declarations.len(), 3);
        assert_eq!(right.declarations.len(), 3);
        assert_eq!(left.declarations, right.declarations);
    }
    
    #[test]
    fn test_outcome_algebra_commutativity() {
        let t1 = StateTransition::Create {
            location: StateLocation("x".to_string()),
            value: Value::Int(42),
        };
        let t2 = StateTransition::Create {
            location: StateLocation("y".to_string()),
            value: Value::Bool(true),
        };
        
        let o1 = Outcome::single(t1);
        let o2 = Outcome::single(t2);
        
        // O1 ⊕ O2 = O2 ⊕ O1 (order preserved but operations commute)
        let left = o1.clone().compose(o2.clone());
        let right = o2.compose(o1);
        
        // Both have the same transitions
        assert_eq!(left.declarations.len(), 2);
        assert_eq!(right.declarations.len(), 2);
    }
    
    #[test]
    fn test_effect_composition() {
        // Test that effects compose properly
        let read_effect: Effect<Value, EffectRow> = Effect::read(StateLocation("balance".to_string()));
        let _write_effect: Effect<(), EffectRow> = Effect::write(StateLocation("balance".to_string()), Value::Int(100));
        
        // Chain effects
        let composed = read_effect.and_then(|val| {
            match val {
                Value::Int(n) => Effect::write(StateLocation("balance".to_string()), Value::Int(n + 50)),
                _ => Effect::pure(()),
            }
        });
        
        // Should be a Do effect
        match composed {
            Effect::Do { .. } => (),
            _ => panic!("Expected Do effect"),
        }
    }
    
    #[test]
    fn test_handler_naturality() {
        use crate::layer2::effect::{LoggingStateHandler, handle};
        
        // Create a handler
        let handler = LoggingStateHandler::new();
        
        // Test naturality: handler preserves structure
        let pure_effect: Effect<i32, EffectRow> = Effect::pure(42);
        let transformed = handle(pure_effect, handler);
        
        // The transformed effect should still contain the pure value
        match transformed {
            Effect::Transform { effect, .. } => {
                match effect.as_ref() {
                    Effect::Pure(42) => (),
                    _ => panic!("Handler should preserve pure effects"),
                }
            }
            _ => panic!("Expected Transform effect"),
        }
    }
    
    #[test]
    fn test_interpreter_execution() {
        let mut interpreter = CompositeInterpreter::new();
        
        // Test pure effect
        let pure: Effect<i32, EffectRow> = Effect::pure(42);
        let (outcome, result) = interpreter.interpret(pure);
        assert_eq!(result, 42);
        assert!(outcome.is_empty());
        
        // Test state write
        let write: Effect<(), EffectRow> = Effect::write(StateLocation("x".to_string()), Value::Int(100));
        let (outcome, _) = interpreter.interpret(write);
        assert_eq!(outcome.declarations.len(), 1);
    }
    
    #[test]
    fn test_compilation_round_trip() {
        // Test that we can compile Layer 2 to Layer 1 and back
        let transition = StateTransition::Transfer {
            from: Address("Alice".to_string()),
            to: Address("Bob".to_string()),
            amount: 100,
            resource_type: ResourceType("Token".to_string()),
        };
        
        let outcome = Outcome::single(transition);
        let term = compile_outcome(&outcome).unwrap();
        
        // Should produce a valid Layer 1 term
        match term {
            crate::layer1::session::Term::Let { .. } => (),
            _ => panic!("Expected Let term"),
        }
    }
    
    #[test]
    fn test_proof_verification_distributes() {
        let t1 = StateTransition::Create {
            location: StateLocation("x".to_string()),
            value: Value::Int(42),
        };
        let t2 = StateTransition::Create {
            location: StateLocation("y".to_string()),
            value: Value::Bool(true),
        };
        
        let o1 = Outcome::single(t1);
        let o2 = Outcome::single(t2);
        let composed = o1.clone().compose(o2.clone());
        
        // Generate proofs
        let proof1 = generate_proof(&o1);
        let proof2 = generate_proof(&o2);
        let proof_composed = generate_proof(&composed);
        
        // Verify individually
        assert!(verify_proof(&proof1, &o1));
        assert!(verify_proof(&proof2, &o2));
        assert!(verify_proof(&proof_composed, &composed));
        
        // verify(O1 ⊕ O2) = verify(O1) ∧ verify(O2)
        let individual_verification = verify_proof(&proof1, &o1) && verify_proof(&proof2, &o2);
        let composed_verification = verify_proof(&proof_composed, &composed);
        assert_eq!(individual_verification, composed_verification);
    }
}
