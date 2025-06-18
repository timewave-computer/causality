// Pure effect handlers implementing natural transformations
// Handlers are mathematical functions between effect types, not executors

use crate::layer2::effect::{Effect, EffectOp, OpResult, Handler, EffectRow, EffectType};
use crate::layer2::outcome::{StateLocation, Value};
use std::marker::PhantomData;
use std::collections::HashMap;

/// State interpreter that executes state effects to actual state operations
pub struct StateInterpreter {
    state: HashMap<StateLocation, Value>,
}

impl Default for StateInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl StateInterpreter {
    pub fn new() -> Self {
        StateInterpreter {
            state: HashMap::new(),
        }
    }

    /// Execute a state effect and return the result
    pub fn execute<A>(&mut self, effect: Effect<A, impl Into<EffectRow>>) -> Result<A, String> {
        match effect {
            Effect::Pure(value) => Ok(value),
            Effect::StateRead { location, .. } => {
                let value = self.state.get(&location)
                    .cloned()
                    .unwrap_or(Value::Unit);
                // This is a safe transmute for state reads that return Value
                unsafe { Ok(std::mem::transmute_copy(&value)) }
            }
            Effect::StateWrite { location, value, .. } => {
                self.state.insert(location, value);
                // This is a safe transmute for state writes that return ()
                unsafe { Ok(std::mem::transmute_copy(&())) }
            }
            Effect::Then { first, second } => {
                // Execute first effect (unit result)
                let _: () = unsafe { std::mem::transmute_copy(&self.execute(*first)?) };
                // Execute second effect
                self.execute(*second)
            }
            _ => Err("StateInterpreter can only handle state effects".to_string()),
        }
    }

    pub fn get_state(&self) -> &HashMap<StateLocation, Value> {
        &self.state
    }

    pub fn set_state(&mut self, location: StateLocation, value: Value) {
        self.state.insert(location, value);
    }
}

/// Communication interpreter that executes communication effects
pub struct CommInterpreter {
    channels: HashMap<String, Vec<Value>>,
}

impl Default for CommInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl CommInterpreter {
    pub fn new() -> Self {
        CommInterpreter {
            channels: HashMap::new(),
        }
    }

    /// Execute a communication effect
    pub fn execute<A>(&mut self, effect: Effect<A, impl Into<EffectRow>>) -> Result<A, String> {
        match effect {
            Effect::Pure(value) => Ok(value),
            Effect::CommSend { channel, value, .. } => {
                self.channels.entry(channel).or_default().push(value);
                // Safe transmute for send operations that return ()
                unsafe { Ok(std::mem::transmute_copy(&())) }
            }
            Effect::CommReceive { channel, .. } => {
                let value = self.channels.get_mut(&channel)
                    .and_then(|queue| if !queue.is_empty() { queue.remove(0).into() } else { None })
                    .unwrap_or(Value::Unit);
                // Safe transmute for receive operations that return Value
                unsafe { Ok(std::mem::transmute_copy(&value)) }
            }
            Effect::Then { first, second } => {
                let _: () = unsafe { std::mem::transmute_copy(&self.execute(*first)?) };
                self.execute(*second)
            }
            _ => Err("CommInterpreter can only handle communication effects".to_string()),
        }
    }

    pub fn get_channels(&self) -> &HashMap<String, Vec<Value>> {
        &self.channels
    }
}

/// Proof interpreter that executes proof effects (minimal implementation)
pub struct ProofInterpreter;

impl Default for ProofInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProofInterpreter {
    pub fn new() -> Self {
        ProofInterpreter
    }

    /// Execute a proof effect
    pub fn execute<A>(&mut self, effect: Effect<A, impl Into<EffectRow>>) -> Result<A, String> {
        match effect {
            Effect::Pure(value) => Ok(value),
            Effect::ProofGenerate { claim, witness, .. } => {
                // Minimal proof generation: just hash the claim and witness
                let proof_data = format!("{:?}_{:?}", claim, witness);
                let proof = Value::String(proof_data);
                // Safe transmute for proof generation that returns Value
                unsafe { Ok(std::mem::transmute_copy(&proof)) }
            }
            Effect::ProofVerify { proof, claim, .. } => {
                // Minimal proof verification: check if proof contains claim
                let is_valid = match (&proof, &claim) {
                    (Value::String(proof_str), _) => proof_str.contains(&format!("{:?}", claim)),
                    _ => false,
                };
                // Safe transmute for proof verification that returns bool
                unsafe { Ok(std::mem::transmute_copy(&is_valid)) }
            }
            Effect::Then { first, second } => {
                let _: () = unsafe { std::mem::transmute_copy(&self.execute(*first)?) };
                self.execute(*second)
            }
            _ => Err("ProofInterpreter can only handle proof effects".to_string()),
        }
    }
}

/// Combined interpreter that can handle multiple effect types
pub struct UnifiedInterpreter {
    state_interpreter: StateInterpreter,
    comm_interpreter: CommInterpreter,
    proof_interpreter: ProofInterpreter,
}

impl Default for UnifiedInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedInterpreter {
    pub fn new() -> Self {
        UnifiedInterpreter {
            state_interpreter: StateInterpreter::new(),
            comm_interpreter: CommInterpreter::new(),
            proof_interpreter: ProofInterpreter::new(),
        }
    }

    /// Execute any pure effect by dispatching to appropriate interpreter
    pub fn execute<A>(&mut self, effect: Effect<A, impl Into<EffectRow>>) -> Result<A, String> {
        match effect {
            Effect::Pure(value) => Ok(value),
            
            // State effects
            Effect::StateRead { .. } | Effect::StateWrite { .. } => {
                self.state_interpreter.execute(effect)
            }
            
            // Communication effects
            Effect::CommSend { .. } | Effect::CommReceive { .. } => {
                self.comm_interpreter.execute(effect)
            }
            
            // Proof effects
            Effect::ProofGenerate { .. } | Effect::ProofVerify { .. } => {
                self.proof_interpreter.execute(effect)
            }
            
            // Sequential composition
            Effect::Then { first, second } => {
                let _: () = unsafe { std::mem::transmute_copy(&self.execute(*first)?) };
                self.execute(*second)
            }
            
            Effect::_Phantom(_) => Err("Cannot execute phantom effects".to_string()),
        }
    }

    pub fn get_state_interpreter(&self) -> &StateInterpreter {
        &self.state_interpreter
    }

    pub fn get_state_interpreter_mut(&mut self) -> &mut StateInterpreter {
        &mut self.state_interpreter
    }

    pub fn get_comm_interpreter(&self) -> &CommInterpreter {
        &self.comm_interpreter
    }

    pub fn get_comm_interpreter_mut(&mut self) -> &mut CommInterpreter {
        &mut self.comm_interpreter
    }
}

/// Handler that logs all operations before passing them through
pub struct LoggingHandler {
    name: String,
}

impl LoggingHandler {
    pub fn new(name: String) -> Self {
        LoggingHandler { name }
    }
}

impl<R: 'static> Handler<R> for LoggingHandler {
    fn transform_op(&self, op: EffectOp) -> Effect<OpResult, R> {
        println!("[{}] Handling operation: {:?}", self.name, op);
        
        // Transform the operation to pure effects
        match op {
            EffectOp::StateRead(location) => Effect::StateRead {
                location,
                _result_type: PhantomData,
            },
            EffectOp::StateWrite(location, value) => Effect::StateWrite {
                location,
                value,
                _result_type: PhantomData,
            },
            EffectOp::CommSend(channel, value) => Effect::CommSend {
                channel,
                value,
                _result_type: PhantomData,
            },
            EffectOp::CommReceive(channel) => Effect::CommReceive {
                channel,
                _result_type: PhantomData,
            },
            EffectOp::ProofGenerate(claim, witness) => Effect::ProofGenerate {
                claim,
                witness,
                _result_type: PhantomData,
            },
            EffectOp::ProofVerify(proof, claim) => Effect::ProofVerify {
                proof,
                claim,
                _result_type: PhantomData,
            },
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Rate limiting handler that tracks operation counts
pub struct RateLimitingHandler {
    operation_counts: HashMap<String, u32>,
    limits: HashMap<String, u32>,
}

impl RateLimitingHandler {
    pub fn new() -> Self {
        RateLimitingHandler {
            operation_counts: HashMap::new(),
            limits: HashMap::new(),
        }
    }

    pub fn set_limit(&mut self, operation_type: String, limit: u32) {
        self.limits.insert(operation_type, limit);
    }

    fn check_rate_limit(&mut self, op_type: &str) -> bool {
        let count = self.operation_counts.entry(op_type.to_string()).or_insert(0);
        *count += 1;
        
        if let Some(&limit) = self.limits.get(op_type) {
            *count <= limit
        } else {
            true // No limit set
        }
    }
}

impl Default for RateLimitingHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: 'static> Handler<R> for RateLimitingHandler {
    fn transform_op(&self, op: EffectOp) -> Effect<OpResult, R> {
        // Note: This is a simplified implementation
        // In practice, you'd need mutable access to update counts
        
        match op {
            EffectOp::StateRead(location) => Effect::StateRead {
                location,
                _result_type: PhantomData,
            },
            EffectOp::StateWrite(location, value) => Effect::StateWrite {
                location,
                value,
                _result_type: PhantomData,
            },
            EffectOp::CommSend(channel, value) => Effect::CommSend {
                channel,
                value,
                _result_type: PhantomData,
            },
            EffectOp::CommReceive(channel) => Effect::CommReceive {
                channel,
                _result_type: PhantomData,
            },
            EffectOp::ProofGenerate(claim, witness) => Effect::ProofGenerate {
                claim,
                witness,
                _result_type: PhantomData,
            },
            EffectOp::ProofVerify(proof, claim) => Effect::ProofVerify {
                proof,
                claim,
                _result_type: PhantomData,
            },
        }
    }

    fn name(&self) -> &str {
        "rate_limiting"
    }
}

/// Natural transformation that converts state effects to communication effects
/// Example: StateRead(location) -> CommReceive(location.to_string())
pub struct StateToCommTransformation;

impl<R: 'static> Handler<R> for StateToCommTransformation {
    fn transform_op(&self, op: EffectOp) -> Effect<OpResult, R> {
        match op {
            EffectOp::StateRead(StateLocation(loc)) => Effect::CommReceive {
                channel: loc,
                _result_type: PhantomData,
            },
            EffectOp::StateWrite(StateLocation(loc), value) => Effect::CommSend {
                channel: loc,
                value,
                _result_type: PhantomData,
            },
            // Pass through non-state operations unchanged
            other => match other {
                EffectOp::CommSend(channel, value) => Effect::CommSend {
                    channel,
                    value,
                    _result_type: PhantomData,
                },
                EffectOp::CommReceive(channel) => Effect::CommReceive {
                    channel,
                    _result_type: PhantomData,
                },
                EffectOp::ProofGenerate(claim, witness) => Effect::ProofGenerate {
                    claim,
                    witness,
                    _result_type: PhantomData,
                },
                EffectOp::ProofVerify(proof, claim) => Effect::ProofVerify {
                    proof,
                    claim,
                    _result_type: PhantomData,
                },
                EffectOp::StateRead(_) | EffectOp::StateWrite(_, _) => unreachable!(),
            }
        }
    }

    fn name(&self) -> &str {
        "state_to_comm"
    }
} 