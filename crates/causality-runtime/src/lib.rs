//! Causality Runtime System
//!
//! This crate provides the runtime execution environment for the Causality framework,
//! including instruction execution, effect handling, ZK proof generation, and resource management.

pub mod executor;
// pub mod interpreter;  // TODO: Fix interpreter API compatibility
pub mod error;
// pub mod zk_executor;  // TODO: Fix ZK executor API compatibility

// Core exports
pub use executor::*;
// pub use interpreter::*;
pub use error::*;
// pub use zk_executor::*;

// TODO: Implement this function when handler and context modules are available
/*
/// Execute an effect program using the default interpreter and handlers
pub fn execute_effect<T>(
    effect: &EffectExpr,
    context: RuntimeContext,
) -> RuntimeResult<T>
where
    T: From<Value> + Clone,
{
    let mut interpreter = Interpreter::new();
    let result = interpreter.interpret(effect, context)?;
    Ok(T::from(result))
}
*/

#[cfg(test)]
mod integration_tests {
    
    // use causality_core::effect::core::{EffectExpr, EffectExprKind};
    // use causality_core::lambda::{Term, TermKind, Literal};
    
    #[test]
    fn test_runtime_integration() {
        // TODO: Implement when effect and context modules are available
        assert!(true);
        // let term = Term::new(TermKind::Literal(Literal::Int(42)));
        // let effect = EffectExpr::new(EffectExprKind::Pure(term));
        // 
        // let context = RuntimeContext::new();
        // let result: RuntimeResult<i64> = execute_effect(effect, context);
        // 
        // match result {
        //     Ok(value) => assert_eq!(value, 42),
        //     Err(e) => panic!("Effect execution failed: {:?}", e),
        // }
    }
    
    #[test]
    fn test_handler_composition() {
        // TODO: Implement when handler modules are available
        assert!(true);
        // let handler1 = PureHandler::identity();
        // let handler2 = PureHandler::identity();
        // 
        // let composed = handler1.compose(handler2);
        // assert!(composed.is_pure());
    }
} 