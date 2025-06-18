// Layer 2 to Layer 1 compiler - translates effects and outcomes to sessions

use crate::layer1::types::{Type, SessionType, RowType};
use crate::layer2::outcome::{Outcome, StateTransition, Value};
use crate::layer2::effect::Effect;
use thiserror::Error;

/// Compilation errors from Layer 2 to Layer 1
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Unsupported effect type: {0}")]
    UnsupportedEffect(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Missing handler for effect: {0}")]
    MissingHandler(String),
    
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
    
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),
}

/// Compile an outcome to a session type (stub implementation)
pub fn compile_outcome_to_session(outcome: &Outcome) -> Result<SessionType, CompileError> {
    if outcome.is_empty() {
        Ok(SessionType::End)
    } else {
        // For now, just return a simple send session for any non-empty outcome
        Ok(SessionType::Send(
            Box::new(Type::Record(RowType::Empty)),
            Box::new(SessionType::End)
        ))
    }
}

/// Compile an effect to a session type
pub fn compile_effect_to_session<T, R>(effect: &Effect<T, R>) -> Result<SessionType, CompileError> {
    match effect {
        Effect::Pure(_) => Ok(SessionType::End),
        
        Effect::StateRead { .. } => {
            // State read becomes a receive operation
            Ok(SessionType::Receive(
                Box::new(Type::Record(RowType::Empty)),
                Box::new(SessionType::End)
            ))
        }
        
        Effect::StateWrite { .. } => {
            // State write becomes a send operation
            Ok(SessionType::Send(
                Box::new(Type::Record(RowType::Empty)),
                Box::new(SessionType::End)
            ))
        }
        
        Effect::CommSend { .. } => {
            // Communication send
            Ok(SessionType::Send(
                Box::new(Type::Record(RowType::Empty)),
                Box::new(SessionType::End)
            ))
        }
        
        Effect::CommReceive { .. } => {
            // Communication receive
            Ok(SessionType::Receive(
                Box::new(Type::Record(RowType::Empty)),
                Box::new(SessionType::End)
            ))
        }
        
        Effect::ProofGenerate { .. } => {
            // Proof generation is internal computation
            Ok(SessionType::End)
        }
        
        Effect::ProofVerify { .. } => {
            // Proof verification is internal computation
            Ok(SessionType::End)
        }
        
        Effect::Then { first, second } => {
            // Sequential composition
            let first_session = compile_effect_to_session(first)?;
            let second_session = compile_effect_to_session(second)?;
            
            // Compose sessions sequentially
            Ok(compose_sessions(first_session, second_session))
        }
        
        Effect::_Phantom(_) => {
            Ok(SessionType::End)
        }
    }
}

/// Convert a value to a type
fn value_to_type(value: &Value) -> Type {
    match value {
        Value::Unit => Type::Unit,
        Value::Bool(_) => Type::Bool,
        Value::Int(_) => Type::Int,
        Value::String(_) => Type::Record(RowType::Empty), // Simplified - would need string type
        Value::Bytes(_) => Type::Record(RowType::Empty), // Simplified
        Value::Struct(fields) => {
            // Convert struct to record type
            let row_fields: Vec<(String, Type)> = fields.iter()
                .map(|(name, value)| (name.clone(), value_to_type(value)))
                .collect();
            Type::Record(RowType::from_fields(row_fields))
        },
        Value::Address(_) => Type::Record(RowType::Empty), // Simplified
    }
}

/// Compose two session types sequentially
fn compose_sessions(first: SessionType, second: SessionType) -> SessionType {
    match first {
        SessionType::End => second,
        SessionType::Send(msg_type, cont) => {
            SessionType::Send(msg_type, Box::new(compose_sessions(*cont, second)))
        }
        SessionType::Receive(msg_type, cont) => {
            SessionType::Receive(msg_type, Box::new(compose_sessions(*cont, second)))
        }
        _ => second, // Simplified composition
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::outcome::{Address, ResourceType, StateLocation};
    
    #[test]
    fn test_compile_empty_outcome() {
        let outcome = Outcome::empty();
        let session = compile_outcome_to_session(&outcome).unwrap();
        
        match session {
            SessionType::End => (),
            _ => panic!("Expected End session type"),
        }
    }
    
    #[test]
    fn test_compile_effect_to_session() {
        use crate::layer2::effect::Effect;
        use crate::layer2::EffectRow;
        
        // Pure effect compiles to End
        let pure: Effect<i32, EffectRow> = Effect::pure(42);
        let session = compile_effect_to_session(&pure).unwrap();
        assert_eq!(session, SessionType::End);
        
        // State read compiles to Receive
        let read: Effect<Value, EffectRow> = Effect::read(StateLocation("test".to_string()));
        let session = compile_effect_to_session(&read).unwrap();
        match session {
            SessionType::Receive(_, _) => (),
            _ => panic!("Expected Receive session type"),
        }
    }
}
