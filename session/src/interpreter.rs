// End-to-end interpreter using pure effects and natural transformations
// Provides unified execution engine for all layers of the Causality-Valence architecture

use crate::layer2::effect::{Effect, EffectRow, EffectType};
use crate::layer2::handler::{StateInterpreter, CommInterpreter, ProofInterpreter, UnifiedInterpreter};
use crate::layer2::outcome::{Outcome, Value, StateLocation, StateTransition};
use crate::layer3::agent::{Agent, AgentId, AgentRegistry};
use crate::layer3::capability::Capability;
use crate::layer3::choreography::{Choreography, ChoreographyStep, Message};
use crate::layer3::compiler::CompilerError;
use std::collections::HashMap;

/// Unified interpreter for executing choreographies and effects
pub struct Interpreter {
    /// Unified effect interpreter
    unified_interpreter: UnifiedInterpreter,
    
    /// Agent registry
    agent_registry: AgentRegistry,
    
    /// Execution trace for debugging
    trace: Vec<String>,
    
    /// Debug mode flag
    debug_enabled: bool,
    
    /// Channel registry
    channel_registry: ChannelRegistry,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Interpreter {
            unified_interpreter: UnifiedInterpreter::new(),
            agent_registry: AgentRegistry::new(),
            trace: Vec::new(),
            debug_enabled: false,
            channel_registry: ChannelRegistry::new(),
        }
    }

    /// Enable debug mode
    pub fn enable_debug(&mut self) {
        self.debug_enabled = true;
    }

    /// Disable debug mode
    pub fn disable_debug(&mut self) {
        self.debug_enabled = false;
    }

    /// Add an agent to the registry
    pub fn register_agent(&mut self, agent: Agent) -> Result<(), String> {
        self.agent_registry.register(agent)
    }

    /// Execute a single pure effect
    pub fn execute_effect<A>(&mut self, effect: Effect<A, impl Into<EffectRow>>) -> Result<A, String> {
        if self.debug_enabled {
            self.trace.push(format!("Executing effect"));
        }

        let result = self.unified_interpreter.execute(effect);

        if let Err(ref error) = result {
            if self.debug_enabled {
                self.trace.push(format!("Effect execution failed: {}", error));
            }
        }

        result
    }

    /// Execute a choreography by compiling it to effects and running them
    pub fn execute_choreography(&mut self, choreography: &Choreography) -> Result<Outcome, CompilerError> {
        if self.debug_enabled {
            self.trace.push("Starting choreography execution".to_string());
        }

        // Compile choreography to effects
        let effects = crate::layer3::compiler::compile_choreography(choreography, &self.agent_registry)?;

        // Execute each effect
        let mut outcomes = Vec::new();
        for effect in effects {
            // For now, treat all effects as unit effects for choreography execution
            match self.execute_effect(effect) {
                Ok(_) => {
                    // Create a minimal outcome for successful effect execution
                    let outcome = crate::layer2::outcome::Outcome::single(
                        StateTransition::Update {
                            location: StateLocation("choreography_progress".to_string()),
                            old_value: Value::Int(outcomes.len() as i64),
                            new_value: Value::Int((outcomes.len() + 1) as i64),
                        }
                    );
                    outcomes.push(outcome);
                }
                Err(e) => {
                    if self.debug_enabled {
                        self.trace.push(format!("Choreography effect failed: {}", e));
                    }
                    return Err(CompilerError::ExecutionFailed(e));
                }
            }
        }

        // Compose all outcomes
        let final_outcome = outcomes.into_iter().fold(crate::layer2::outcome::Outcome::empty(), |acc, outcome| {
            acc.compose(outcome)
        });

        if self.debug_enabled {
            self.trace.push("Choreography execution completed".to_string());
        }

        Ok(final_outcome)
    }

    /// Get the current state
    pub fn get_state(&self) -> &HashMap<StateLocation, Value> {
        self.unified_interpreter.get_state_interpreter().get_state()
    }

    /// Set a state value
    pub fn set_state(&mut self, location: StateLocation, value: Value) {
        self.unified_interpreter.get_state_interpreter_mut().set_state(location, value);
    }

    /// Get the execution trace
    pub fn get_trace(&self) -> &[String] {
        &self.trace
    }

    /// Clear the execution trace
    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }

    /// Print current state for debugging
    pub fn print_state(&self) {
        println!("=== Interpreter State ===");
        
        let state = self.get_state();
        if state.is_empty() {
            println!("State: (empty)");
        } else {
            println!("State:");
            for (location, value) in state {
                println!("  {:?} = {:?}", location, value);
            }
        }

        let channels = self.unified_interpreter.get_comm_interpreter().get_channels();
        if channels.is_empty() {
            println!("Channels: (empty)");
        } else {
            println!("Channels:");
            for (name, queue) in channels {
                println!("  {} = {:?}", name, queue);
            }
        }

        if self.debug_enabled && !self.trace.is_empty() {
            println!("Execution Trace:");
            for (i, entry) in self.trace.iter().enumerate() {
                println!("  {}: {}", i + 1, entry);
            }
        }
        
        println!("========================");
    }

    /// Get the channel registry
    pub fn get_channel_registry(&mut self) -> &mut ChannelRegistry {
        &mut self.channel_registry
    }

    /// Execute a sequence of effects
    pub fn execute_effects<A>(&mut self, effects: Vec<Effect<A, impl Into<EffectRow>>>) -> Result<Vec<A>, String> {
        let mut results = Vec::new();
        
        for effect in effects {
            let result = self.execute_effect(effect)?;
            results.push(result);
        }
        
        Ok(results)
    }
}

/// Channel registry for managing communication channels
pub struct ChannelRegistry {
    channels: HashMap<String, Channel>,
}

/// A communication channel
pub struct Channel {
    name: String,
    capacity: usize,
    buffer: Vec<Value>,
    participants: Vec<String>,
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelRegistry {
    pub fn new() -> Self {
        ChannelRegistry {
            channels: HashMap::new(),
        }
    }

    /// Create a new channel
    pub fn create_channel(&mut self, name: String, capacity: usize, participants: Vec<String>) {
        let channel = Channel {
            name: name.clone(),
            capacity,
            buffer: Vec::new(),
            participants,
        };
        self.channels.insert(name, channel);
    }

    /// Send a message to a channel
    pub fn send(&mut self, channel_name: &str, value: Value) -> Result<(), String> {
        let channel = self.channels.get_mut(channel_name)
            .ok_or_else(|| format!("Channel '{}' not found", channel_name))?;

        if channel.buffer.len() >= channel.capacity {
            return Err(format!("Channel '{}' is full", channel_name));
        }

        channel.buffer.push(value);
        Ok(())
    }

    /// Receive a message from a channel
    pub fn receive(&mut self, channel_name: &str) -> Result<Option<Value>, String> {
        let channel = self.channels.get_mut(channel_name)
            .ok_or_else(|| format!("Channel '{}' not found", channel_name))?;

        Ok(if channel.buffer.is_empty() {
            None
        } else {
            Some(channel.buffer.remove(0))
        })
    }

    /// Get channel status
    pub fn get_channel_status(&self, channel_name: &str) -> Option<ChannelStatus> {
        self.channels.get(channel_name).map(|channel| {
            ChannelStatus {
                name: channel.name.clone(),
                capacity: channel.capacity,
                current_size: channel.buffer.len(),
                participants: channel.participants.clone(),
            }
        })
    }

    /// List all channels
    pub fn list_channels(&self) -> Vec<String> {
        self.channels.keys().cloned().collect()
    }
}

/// Channel status information
#[derive(Debug, Clone)]
pub struct ChannelStatus {
    pub name: String,
    pub capacity: usize,
    pub current_size: usize,
    pub participants: Vec<String>,
}

/// Helper functions for creating common effects
pub mod effects {
    use super::*;

    /// Create a state read effect
    pub fn read_state<R: Into<EffectRow> + 'static>(location: StateLocation) -> Effect<Value, R> {
        Effect::read(location)
    }

    /// Create a state write effect
    pub fn write_state<R: Into<EffectRow> + 'static>(location: StateLocation, value: Value) -> Effect<(), R> {
        Effect::write(location, value)
    }

    /// Create a communication send effect
    pub fn send_message<R: Into<EffectRow> + 'static>(channel: String, value: Value) -> Effect<(), R> {
        Effect::send(channel, value)
    }

    /// Create a communication receive effect
    pub fn receive_message<R: Into<EffectRow> + 'static>(channel: String) -> Effect<Value, R> {
        Effect::receive(channel)
    }

    /// Create a proof generation effect
    pub fn generate_proof<R: Into<EffectRow> + 'static>(claim: Value, witness: Value) -> Effect<Value, R> {
        Effect::prove(claim, witness)
    }

    /// Create a proof verification effect
    pub fn verify_proof<R: Into<EffectRow> + 'static>(proof: Value, claim: Value) -> Effect<bool, R> {
        Effect::verify(proof, claim)
    }

    /// Sequence two effects
    pub fn sequence<A: 'static, B: 'static, R: Into<EffectRow> + 'static>(
        first: Effect<A, R>, 
        second: Effect<B, R>
    ) -> Effect<B, R> {
        first.then(second)
    }

    /// Create a pure effect from a value
    pub fn pure<A: 'static, R: 'static>(value: A) -> Effect<A, R> {
        Effect::pure(value)
    }
} 