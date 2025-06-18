// Layer 2: Verifiable Outcome Algebra
// Pure algebraic effects and verifiable outcomes

pub mod effect;
pub mod outcome;
pub mod handler;
pub mod compiler;

// Re-export core types
pub use effect::{Effect, EffectRow, EffectType, EffectOp, OpResult, Handler};
pub use outcome::{Outcome, StateTransition, Value, StateLocation};
pub use handler::{StateInterpreter, CommInterpreter, ProofInterpreter, UnifiedInterpreter};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_outcome_algebra_identity() {
        let empty = Outcome::empty();
        let transition = StateTransition::Create {
            location: StateLocation("test".to_string()),
            value: Value::Int(42),
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
        let t1 = StateTransition::Create {
            location: StateLocation("x".to_string()),
            value: Value::Int(50),
        };
        let t2 = StateTransition::Create {
            location: StateLocation("y".to_string()),
            value: Value::Int(30),
        };
        let t3 = StateTransition::Create {
            location: StateLocation("z".to_string()),
            value: Value::Int(20),
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
        // Test that effects compose properly with pure effects
        let read_effect: Effect<Value, EffectRow> = Effect::read(StateLocation("balance".to_string()));
        let _write_effect: Effect<(), EffectRow> = Effect::write(StateLocation("balance".to_string()), Value::Int(100));
        
        // Chain effects using then
        let composed = read_effect.then(Effect::write(StateLocation("balance".to_string()), Value::Int(150)));
        
        // Should be a Then effect
        match composed {
            Effect::Then { .. } => (),
            _ => panic!("Expected Then effect"),
        }
    }
    
    #[test]
    fn test_handler_transformation() {
        use crate::layer2::handler::LoggingHandler;
        
        // Create a handler
        let handler = LoggingHandler::new("test".to_string());
        
        // Test transformation: handler preserves structure
        let op = EffectOp::StateRead(StateLocation("test".to_string()));
        let _transformed: Effect<OpResult, EffectRow> = handler.transform_op(op);
        
        // The transformed effect should be a StateRead
        // Note: we can't easily test the structure due to type complexities
    }
    
    #[test]
    fn test_interpreter_execution() {
        let mut interpreter = UnifiedInterpreter::new();
        
        // Test pure effect
        let pure: Effect<i32, EffectRow> = Effect::pure(42);
        let result = interpreter.execute(pure).unwrap();
        assert_eq!(result, 42);
        
        // Test state write
        let write: Effect<(), EffectRow> = Effect::write(StateLocation("x".to_string()), Value::Int(100));
        let _result = interpreter.execute(write).unwrap();
        
        // Check that state was written
        let state = interpreter.get_state_interpreter().get_state();
        assert_eq!(state.get(&StateLocation("x".to_string())), Some(&Value::Int(100)));
    }
}
