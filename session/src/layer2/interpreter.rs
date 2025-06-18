// Layer 2 Effect interpreters - execute transformed effects to produce outcomes

use crate::layer2::effect::{Effect, EffectOp, OpResult, EffectRow};
use crate::layer2::outcome::{Outcome, StateTransition, StateLocation, Value};
use std::collections::BTreeMap;

/// Interpreter trait - executes effects to produce outcomes
pub trait Interpreter<R> {
    /// Execute an effect to produce an outcome and result
    fn interpret<T>(&mut self, effect: Effect<T, R>) -> (Outcome, T);
    
    /// Get interpreter name for debugging
    fn name(&self) -> &str;
}

/// Composite interpreter that handles multiple effect types
pub struct CompositeInterpreter {
    /// State storage for state effects
    state: BTreeMap<StateLocation, Value>,
    
    /// Channel buffers for communication effects
    channels: BTreeMap<String, Vec<Value>>,
    
    /// Proof storage (stub implementation)
    proofs: BTreeMap<Value, Value>,
}

impl Default for CompositeInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeInterpreter {
    /// Create a new interpreter with empty state
    pub fn new() -> Self {
        CompositeInterpreter {
            state: BTreeMap::new(),
            channels: BTreeMap::new(),
            proofs: BTreeMap::new(),
        }
    }
    
    /// Handle a state read operation
    fn handle_state_read(&self, loc: &StateLocation) -> (Outcome, Value) {
        let value = self.state.get(loc).cloned().unwrap_or(Value::Unit);
        (Outcome::empty(), value)
    }
    
    /// Handle a state write operation
    fn handle_state_write(&mut self, loc: &StateLocation, val: &Value) -> Outcome {
        let old_value = self.state.insert(loc.clone(), val.clone())
            .unwrap_or(Value::Unit);
        
        Outcome::single(StateTransition::Update {
            location: loc.clone(),
            old_value,
            new_value: val.clone(),
        })
    }
    
    /// Handle a communication send operation
    fn handle_comm_send(&mut self, channel: &str, val: &Value) -> Outcome {
        self.channels
            .entry(channel.to_string())
            .or_default()
            .push(val.clone());
        
        Outcome::empty() // Send doesn't produce state transitions
    }
    
    /// Handle a communication receive operation
    fn handle_comm_receive(&mut self, channel: &str) -> (Outcome, Value) {
        let value = self.channels
            .get_mut(channel)
            .and_then(|buffer| buffer.pop())
            .unwrap_or(Value::Unit);
        
        (Outcome::empty(), value)
    }
    
    /// Handle proof generation (stub implementation)
    fn handle_proof_generate(&mut self, claim: &Value, witness: &Value) -> (Outcome, Value) {
        // Stub: create a simple proof by combining values
        let proof_id = match (claim, witness) {
            (Value::Int(c), Value::Int(w)) => Value::Int(c + w),
            _ => Value::Int(0),
        };
        
        self.proofs.insert(claim.clone(), proof_id.clone());
        (Outcome::empty(), proof_id)
    }
    
    /// Handle proof verification (stub implementation)
    fn handle_proof_verify(&self, proof: &Value, claim: &Value) -> (Outcome, bool) {
        // Stub: check if we have this proof stored for the claim
        let valid = self.proofs.get(claim).map(|p| p == proof).unwrap_or(false);
        (Outcome::empty(), valid)
    }
    
    /// Apply a handler-transformed operation
    fn interpret_transformed<R: 'static>(&mut self, transformed: Effect<OpResult, R>) -> (Outcome, OpResult) {
        // Recursively interpret the transformed effect
        match transformed {
            Effect::Pure(result) => (Outcome::empty(), result),
            Effect::Do { op, cont, .. } => {
                // Execute the transformed operation
                let (outcome, result) = self.execute_op(op);
                // Continue with the result
                let next_effect = cont(result.clone());
                let (next_outcome, final_result) = self.interpret_transformed(next_effect);
                (outcome.compose(next_outcome), final_result)
            }
            Effect::Transform { .. } => {
                panic!("Nested transforms not yet supported")
            }
        }
    }
    
    /// Execute a single operation
    fn execute_op(&mut self, op: EffectOp) -> (Outcome, OpResult) {
        match op {
            EffectOp::StateRead(loc) => {
                let (o, v) = self.handle_state_read(&loc);
                (o, OpResult::Value(v))
            }
            EffectOp::StateWrite(loc, val) => {
                let o = self.handle_state_write(&loc, &val);
                (o, OpResult::Unit)
            }
            EffectOp::CommSend(channel, val) => {
                let o = self.handle_comm_send(&channel, &val);
                (o, OpResult::Unit)
            }
            EffectOp::CommReceive(channel) => {
                let (o, v) = self.handle_comm_receive(&channel);
                (o, OpResult::Value(v))
            }
            EffectOp::ProofGenerate(claim, witness) => {
                let (o, v) = self.handle_proof_generate(&claim, &witness);
                (o, OpResult::Value(v))
            }
            EffectOp::ProofVerify(proof, claim) => {
                let (o, b) = self.handle_proof_verify(&proof, &claim);
                (o, OpResult::Bool(b))
            }
        }
    }
}

impl<R: 'static> Interpreter<R> for CompositeInterpreter {
    fn interpret<T>(&mut self, effect: Effect<T, R>) -> (Outcome, T) {
        match effect {
            Effect::Pure(value) => (Outcome::empty(), value),
            
            Effect::Do { op, cont, .. } => {
                // Execute the operation
                let (outcome, result) = self.execute_op(op);
                
                // Continue with the result
                let next_effect = cont(result);
                let (next_outcome, final_result) = self.interpret(next_effect);
                
                // Compose outcomes
                (outcome.compose(next_outcome), final_result)
            }
            
            Effect::Transform { handler, effect, .. } => {
                // Apply transformation to the inner effect
                self.interpret_transform_helper(*effect, &*handler)
            }
        }
    }
    
    fn name(&self) -> &str {
        "composite"
    }
}

impl CompositeInterpreter {
    /// Helper to interpret transformed effects
    fn interpret_transform_helper<T, R: 'static>(
        &mut self, 
        effect: Effect<T, R>,
        handler: &dyn crate::layer2::effect::Handler<R>
    ) -> (Outcome, T) {
        match effect {
            Effect::Pure(value) => (Outcome::empty(), value),
            
            Effect::Do { op, cont, .. } => {
                // Transform the operation using the handler
                let transformed = handler.transform_op(op);
                
                // Interpret the transformed operation
                let (outcome, result) = self.interpret_transformed(transformed);
                
                // Continue with the original continuation
                let next_effect = cont(result);
                let (next_outcome, final_result) = self.interpret_transform_helper(next_effect, handler);
                
                (outcome.compose(next_outcome), final_result)
            }
            
            Effect::Transform { handler: _inner_handler, effect, .. } => {
                // Compose handlers - for now just use the outer handler
                self.interpret_transform_helper(*effect, handler)
            }
        }
    }
}

/// Pure interpreter that only handles pure effects
pub struct PureInterpreter;

impl<R> Interpreter<R> for PureInterpreter {
    fn interpret<T>(&mut self, effect: Effect<T, R>) -> (Outcome, T) {
        match effect {
            Effect::Pure(value) => (Outcome::empty(), value),
            _ => panic!("PureInterpreter can only handle pure effects"),
        }
    }
    
    fn name(&self) -> &str {
        "pure"
    }
}

/// Helper function to run an effect with the default interpreter
pub fn run_effect<T: 'static>(effect: Effect<T, EffectRow>) -> (Outcome, T) {
    let mut interpreter = CompositeInterpreter::new();
    interpreter.interpret(effect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::effect::{Effect, handle, LoggingStateHandler};
    
    #[test]
    fn test_pure_interpreter() {
        let mut interpreter = PureInterpreter;
        let effect: Effect<i32, EffectRow> = Effect::pure(42);
        let (outcome, result) = interpreter.interpret(effect);
        
        assert!(outcome.is_empty());
        assert_eq!(result, 42);
    }
    
    #[test]
    fn test_state_operations() {
        let mut interpreter = CompositeInterpreter::new();
        let loc = StateLocation("test".to_string());
        
        // Write effect
        let write_effect: Effect<(), EffectRow> = Effect::write(loc.clone(), Value::Int(42));
        let (write_outcome, _) = interpreter.interpret(write_effect);
        
        assert_eq!(write_outcome.declarations.len(), 1);
        match &write_outcome.declarations[0] {
            StateTransition::Update { new_value, .. } => {
                assert_eq!(new_value, &Value::Int(42));
            }
            _ => panic!("Expected Update transition"),
        }
        
        // Read effect
        let read_effect: Effect<Value, EffectRow> = Effect::read(loc);
        let (read_outcome, read_value) = interpreter.interpret(read_effect);
        
        assert!(read_outcome.is_empty());
        assert_eq!(read_value, Value::Int(42));
    }
    
    #[test]
    fn test_communication_operations() {
        let mut interpreter = CompositeInterpreter::new();
        let channel = "test_channel".to_string();
        
        // Send effect
        let send_effect: Effect<(), EffectRow> = Effect::send(channel.clone(), Value::Int(123));
        let (send_outcome, _) = interpreter.interpret(send_effect);
        assert!(send_outcome.is_empty());
        
        // Receive effect
        let receive_effect: Effect<Value, EffectRow> = Effect::receive(channel);
        let (receive_outcome, received) = interpreter.interpret(receive_effect);
        
        assert!(receive_outcome.is_empty());
        assert_eq!(received, Value::Int(123));
    }
    
    #[test]
    fn test_proof_operations() {
        let mut interpreter = CompositeInterpreter::new();
        let claim = Value::Int(42);
        let witness = Value::Int(7);
        
        // Generate proof
        let prove_effect: Effect<Value, EffectRow> = Effect::prove(claim.clone(), witness);
        let (prove_outcome, proof) = interpreter.interpret(prove_effect);
        assert!(prove_outcome.is_empty());
        
        // Verify proof
        let verify_effect: Effect<bool, EffectRow> = Effect::verify(proof, claim.clone());
        let (verify_outcome, valid) = interpreter.interpret(verify_effect);
        
        assert!(verify_outcome.is_empty());
        assert!(valid);
    }
    
    #[test]
    fn test_handler_transform() {
        let mut interpreter = CompositeInterpreter::new();
        let loc = StateLocation("test".to_string());
        
        // Create a read effect with a logging handler
        let read_effect: Effect<Value, EffectRow> = Effect::read(loc.clone());
        let handled_effect = handle(read_effect, LoggingStateHandler);
        
        // Should see the logging output when executed
        let (outcome, _value) = interpreter.interpret(handled_effect);
        assert!(outcome.is_empty());
    }
} 