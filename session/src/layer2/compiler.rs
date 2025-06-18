// Layer 2 to Layer 1 compiler - translates declarative outcomes to session protocols

use crate::layer1::types::{Type, SessionType, RowType};
use crate::layer1::session::Term;
use crate::layer1::linear::Variable;
use crate::layer2::outcome::{Outcome, StateTransition, Value};
use crate::layer2::effect::{Effect, EffectOp};
use std::collections::BTreeMap;

/// Compile an outcome to a sequence of Layer 1 terms
pub fn compile_outcome(outcome: &Outcome) -> Result<Term, CompileError> {
    // For each state transition, generate appropriate Layer 1 operations
    let mut terms = Vec::new();
    
    for transition in &outcome.declarations {
        let term = compile_transition(transition)?;
        terms.push(term);
    }
    
    // Combine terms into a sequence
    if terms.is_empty() {
        Ok(Term::Unit)
    } else {
        Ok(sequence_terms(terms))
    }
}

/// Compile a single state transition to Layer 1 term
fn compile_transition(transition: &StateTransition) -> Result<Term, CompileError> {
    match transition {
        StateTransition::Transfer { from, to, amount, resource_type } => {
            // Create a transfer message as a record
            let mut fields = BTreeMap::new();
            fields.insert("from".to_string(), Box::new(Term::Var(Variable(from.0.clone()))));
            fields.insert("to".to_string(), Box::new(Term::Var(Variable(to.0.clone()))));
            fields.insert("amount".to_string(), Box::new(Term::Int(*amount as i64)));
            fields.insert("resource".to_string(), Box::new(Term::Var(Variable(resource_type.0.clone()))));
            
            Ok(Term::Record(fields))
        }
        
        StateTransition::Update { location, old_value, new_value } => {
            // Create an update message
            let mut fields = BTreeMap::new();
            fields.insert("location".to_string(), Box::new(Term::Var(Variable(location.0.clone()))));
            fields.insert("old".to_string(), Box::new(value_to_term(old_value)));
            fields.insert("new".to_string(), Box::new(value_to_term(new_value)));
            
            Ok(Term::Record(fields))
        }
        
        StateTransition::Create { location, value } => {
            // Create a creation message
            let mut fields = BTreeMap::new();
            fields.insert("location".to_string(), Box::new(Term::Var(Variable(location.0.clone()))));
            fields.insert("value".to_string(), Box::new(value_to_term(value)));
            
            Ok(Term::Record(fields))
        }
        
        StateTransition::Delete { location } => {
            // Create a deletion message
            let mut fields = BTreeMap::new();
            fields.insert("location".to_string(), Box::new(Term::Var(Variable(location.0.clone()))));
            
            Ok(Term::Record(fields))
        }
    }
}

/// Convert a Layer 2 value to a Layer 1 term
fn value_to_term(value: &Value) -> Term {
    match value {
        Value::Unit => Term::Unit,
        Value::Bool(b) => Term::Bool(*b),
        Value::Int(n) => Term::Int(*n),
        Value::String(s) => Term::Var(Variable(s.clone())), // Simplified
        Value::Bytes(_) => Term::Unit, // Not directly supported
        Value::Struct(fields) => {
            let mut term_fields = BTreeMap::new();
            for (name, val) in fields {
                term_fields.insert(name.clone(), Box::new(value_to_term(val)));
            }
            Term::Record(term_fields)
        }
        Value::Address(addr) => Term::Var(Variable(addr.0.clone())),
    }
}

/// Sequence multiple terms
fn sequence_terms(terms: Vec<Term>) -> Term {
    terms.into_iter()
        .rev()
        .fold(Term::Unit, |acc, term| {
            Term::Let {
                var: Variable("_".to_string()),
                value: Box::new(term),
                body: Box::new(acc),
            }
        })
}

/// Compile an effect to a session type
pub fn compile_effect_to_session<T, R>(effect: &Effect<T, R>) -> Result<SessionType, CompileError> {
    match effect {
        Effect::Pure(_) => Ok(SessionType::End),
        
        Effect::Do { op, .. } => {
            match op {
                EffectOp::StateRead(_) => {
                    // Reading state is like receiving a message
                    Ok(SessionType::Receive(
                        Box::new(Type::Record(RowType::Empty)),
                        Box::new(SessionType::End)
                    ))
                }
                
                EffectOp::StateWrite(_, _) => {
                    // Writing state is like sending a message
                    Ok(SessionType::Send(
                        Box::new(Type::Record(RowType::Empty)),
                        Box::new(SessionType::End)
                    ))
                }
                
                EffectOp::CommSend(_, _) => {
                    // Direct send operation
                    Ok(SessionType::Send(
                        Box::new(Type::Record(RowType::Empty)),
                        Box::new(SessionType::End)
                    ))
                }
                
                EffectOp::CommReceive(_) => {
                    // Direct receive operation
                    Ok(SessionType::Receive(
                        Box::new(Type::Record(RowType::Empty)),
                        Box::new(SessionType::End)
                    ))
                }
                
                EffectOp::ProofGenerate(_, _) => {
                    // Proof generation is like internal computation
                    Ok(SessionType::End)
                }
                
                EffectOp::ProofVerify(_, _) => {
                    // Proof verification is like internal computation
                    Ok(SessionType::End)
                }
            }
        }
        
        Effect::Transform { effect, .. } => {
            // Recursively compile the inner effect
            compile_effect_to_session(&**effect)
        }
    }
}

/// Compile error types
#[derive(Debug)]
pub enum CompileError {
    UnsupportedFeature(String),
    TypeMismatch(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UnsupportedFeature(msg) => write!(f, "Unsupported feature: {}", msg),
            CompileError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::outcome::{Address, ResourceType, StateLocation};
    
    #[test]
    fn test_compile_empty_outcome() {
        let outcome = Outcome::empty();
        let term = compile_outcome(&outcome).unwrap();
        
        match term {
            Term::Unit => (),
            _ => panic!("Expected Unit term"),
        }
    }
    
    #[test]
    fn test_compile_transfer() {
        let transition = StateTransition::Transfer {
            from: Address("Alice".to_string()),
            to: Address("Bob".to_string()),
            amount: 100,
            resource_type: ResourceType("Token".to_string()),
        };
        
        let outcome = Outcome::single(transition);
        let term = compile_outcome(&outcome).unwrap();
        
        // The term is wrapped in a Let, so unwrap it
        match term {
            Term::Let { value, .. } => {
                // Check that the value is a Record
                match value.as_ref() {
                    Term::Record(_) => (),
                    _ => panic!("Expected Record term inside Let"),
                }
            }
            _ => panic!("Expected Let term"),
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
