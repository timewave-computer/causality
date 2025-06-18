// Unified interpreter for the Causality-Valence architecture
// Enhanced with improved channel management and synchronization

use crate::layer2::effect::{Effect, EffectRow, EffectOp, OpResult};
use crate::layer2::outcome::{Value, StateLocation};
use crate::layer3::agent::AgentRegistry;
use crate::layer3::choreography::Choreography;
use crate::layer3::compiler::compile_choreography;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Condvar};

/// State snapshot for debugging
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// Step number
    pub step: usize,
    /// Operation executed
    pub operation: String,
    /// State after operation
    pub state: BTreeMap<StateLocation, Value>,
    /// Channels after operation
    pub channels: BTreeMap<String, Vec<Value>>,
}

/// Debug mode options
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct DebugOptions {
    /// Enable debug logging
    pub log_enabled: bool,
    /// Take state snapshots
    pub snapshots_enabled: bool,
    /// Step-by-step execution
    pub step_mode: bool,
    /// Callback for step-by-step execution
    pub step_callback: Option<fn(&StateSnapshot) -> bool>,
}

/// Global state for the session system
#[derive(Debug, Clone)]
pub struct GlobalState {
    /// Global state variables
    pub state: BTreeMap<StateLocation, Value>,
    /// Communication channels
    pub channels: BTreeMap<String, Vec<Value>>,
}

/// Channel management for multi-party communication
#[derive(Debug)]
pub struct ChannelManager {
    /// Channel message queues - deterministic ordering with BTreeMap
    channels: BTreeMap<String, Vec<Value>>,
    /// Synchronization primitives for channels
    sync_channels: BTreeMap<String, (Arc<Mutex<bool>>, Arc<Condvar>)>,
    /// Channel capacity limits
    capacities: BTreeMap<String, usize>,
    /// Channel participants - deterministic ordering
    participants: BTreeMap<String, Vec<String>>,
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelManager {
    pub fn new() -> Self {
        ChannelManager {
            channels: BTreeMap::new(),
            sync_channels: BTreeMap::new(),
            capacities: BTreeMap::new(),
            participants: BTreeMap::new(),
        }
    }
    
    /// Create a new channel with deterministic participant ordering
    pub fn create_channel(&mut self, name: String, capacity: usize, mut participants: Vec<String>) {
        participants.sort(); // Ensure deterministic ordering
        self.channels.insert(name.clone(), Vec::new());
        self.capacities.insert(name.clone(), capacity);
        self.participants.insert(name.clone(), participants);
        
        let sync_data = (Arc::new(Mutex::new(false)), Arc::new(Condvar::new()));
        self.sync_channels.insert(name, sync_data);
    }
    
    /// Send a message to a channel (with capacity checking)
    pub fn send(&mut self, channel: &str, value: Value) -> Result<(), String> {
        // Auto-create channel if it doesn't exist
        if !self.channels.contains_key(channel) {
            self.create_channel(channel.to_string(), 100, vec![]);
        }
        
        let channel_queue = self.channels.get_mut(channel)
            .ok_or_else(|| format!("Channel '{}' does not exist", channel))?;
            
        let capacity = self.capacities.get(channel).unwrap_or(&100);
        
        if channel_queue.len() >= *capacity {
            return Err(format!("Channel '{}' is at capacity ({})", channel, capacity));
        }
        
        channel_queue.push(value);
        
        // Signal waiting receivers
        if let Some((mutex, condvar)) = self.sync_channels.get(channel) {
            let _guard = mutex.lock().unwrap();
            condvar.notify_all();
        }
        
        Ok(())
    }
    
    /// Receive a message from a channel
    pub fn receive(&mut self, channel: &str) -> Result<Option<Value>, String> {
        // Auto-create channel if it doesn't exist
        if !self.channels.contains_key(channel) {
            self.create_channel(channel.to_string(), 100, vec![]);
        }
        
        let channel_queue = self.channels.get_mut(channel)
            .ok_or_else(|| format!("Channel '{}' does not exist", channel))?;
        
        Ok(channel_queue.pop())
    }
    
    /// Blocking receive - waits for a message
    pub fn receive_blocking(&mut self, channel: &str) -> Result<Value, String> {
        loop {
            if let Some(value) = self.receive(channel)? {
                return Ok(value);
            }
            
            // Wait for notification
            if let Some((mutex, condvar)) = self.sync_channels.get(channel) {
                let guard = mutex.lock().unwrap();
                let _result = condvar.wait(guard).unwrap();
            }
        }
    }
    
    /// Get channel status
    pub fn get_channel_status(&self, channel: &str) -> Option<(usize, usize, &Vec<String>)> {
        let queue_len = self.channels.get(channel)?.len();
        let capacity = *self.capacities.get(channel)?;
        let participants = self.participants.get(channel)?;
        
        Some((queue_len, capacity, participants))
    }
    
    /// List all channels
    pub fn list_channels(&self) -> Vec<String> {
        self.channels.keys().cloned().collect()
    }
}

/// The unified interpreter state
pub struct Interpreter {
    /// Layer 2: State and effects
    state: BTreeMap<StateLocation, Value>,
    channel_registry: ChannelManager,
    effect_log: Vec<String>,
    
    /// Layer 3: Agent registry
    agent_registry: AgentRegistry,
    
    /// Debug options
    debug_options: DebugOptions,
    
    /// State snapshots
    snapshots: Vec<StateSnapshot>,
    
    /// Current step counter
    step_counter: usize,
    
    /// Error context for better diagnostics
    error_context: ErrorContext,
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
            state: BTreeMap::new(),
            channel_registry: ChannelManager::new(),
            effect_log: Vec::new(),
            agent_registry: AgentRegistry::new(),
            debug_options: DebugOptions::default(),
            snapshots: Vec::new(),
            step_counter: 0,
            error_context: ErrorContext::new(),
        }
    }
    
    /// Enable debug mode (simple)
    pub fn enable_debug(&mut self) {
        self.debug_options.log_enabled = true;
    }
    
    /// Configure debug options
    pub fn set_debug_options(&mut self, options: DebugOptions) {
        self.debug_options = options;
    }
    
    /// Get state snapshots
    pub fn get_snapshots(&self) -> &[StateSnapshot] {
        &self.snapshots
    }
    
    /// Take a snapshot
    fn take_snapshot(&mut self, operation: String) {
        if self.debug_options.snapshots_enabled {
            let snapshot = StateSnapshot {
                step: self.step_counter,
                operation,
                state: self.state.clone(),
                channels: self.channel_registry.channels.clone(),
            };
            
            // If in step mode, call the callback
            if self.debug_options.step_mode {
                if let Some(callback) = self.debug_options.step_callback {
                    if !callback(&snapshot) {
                        // Callback returned false, stop execution
                        panic!("Execution stopped by debug callback");
                    }
                }
            }
            
            self.snapshots.push(snapshot);
            self.step_counter += 1;
        }
    }
    
    /// Execute a choreography (Layer 3)
    #[allow(clippy::result_large_err)]
    pub fn execute_choreography(&mut self, choreo: &Choreography) -> Result<(), InterpreterError> {
        if self.debug_options.log_enabled {
            self.effect_log.push(format!("=== Executing choreography: {:?} ===", choreo));
        }
        
        // Compile choreography to effects
        let effects = compile_choreography(choreo, &self.agent_registry)
            .map_err(|e| InterpreterError::compilation_error(format!("{:?}", e)))?;
        
        if self.debug_options.log_enabled {
            self.effect_log.push(format!("Compiled to {} effects", effects.len()));
        }
        
        // Execute each effect
        for (i, effect) in effects.into_iter().enumerate() {
            if self.debug_options.log_enabled {
                self.effect_log.push(format!("--- Effect {} ---", i + 1));
            }
            self.execute_effect(effect)?;
        }
        
        Ok(())
    }
    
    /// Execute an effect (Layer 2)
    #[allow(clippy::result_large_err)]
    pub fn execute_effect<A>(&mut self, effect: Effect<A, EffectRow>) -> Result<A, InterpreterError> {
        match effect {
            Effect::Pure(value) => {
                if self.debug_options.log_enabled {
                    self.effect_log.push("Pure(<value>)".to_string());
                }
                Ok(value)
            }
            
            Effect::Do { op, cont, .. } => {
                let result = self.execute_operation(&op)?;
                let next_effect = cont(result);
                self.execute_effect(next_effect)
            }
            
            Effect::Transform { handler, effect, .. } => {
                // For now, just execute the inner effect
                // In a full implementation, we'd apply the handler transformation
                if self.debug_options.log_enabled {
                    self.effect_log.push(format!("Applying handler: {}", handler.name()));
                }
                self.execute_effect(*effect)
            }
        }
    }
    
    /// Execute a single operation
    #[allow(clippy::result_large_err)]
    fn execute_operation(&mut self, op: &EffectOp) -> Result<OpResult, InterpreterError> {
        let op_string = format!("{:?}", op);
        
        let result = match op {
            EffectOp::StateRead(loc) => {
                let value = self.state.get(loc)
                    .cloned()
                    .unwrap_or(Value::Unit);
                
                if self.debug_options.log_enabled {
                    self.effect_log.push(format!("StateRead({:?}) = {:?}", loc, value));
                }
                
                Ok(OpResult::Value(value))
            }
            
            EffectOp::StateWrite(loc, value) => {
                self.state.insert(loc.clone(), value.clone());
                
                if self.debug_options.log_enabled {
                    self.effect_log.push(format!("StateWrite({:?}, {:?})", loc, value));
                }
                
                Ok(OpResult::Unit)
            }
            
            EffectOp::CommSend(channel, value) => {
                match self.channel_registry.send(channel, value.clone()) {
                    Ok(()) => {
                        if self.debug_options.log_enabled {
                            self.effect_log.push(format!("CommSend({}, {:?})", channel, value));
                        }
                        Ok(OpResult::Unit)
                    }
                    Err(e) => {
                        if self.debug_options.log_enabled {
                            self.effect_log.push(format!("CommSend({}, {:?}) FAILED: {}", channel, value, e));
                        }
                        Err(InterpreterError::runtime_error(e, op_string.clone()))
                    }
                }
            }
            
            EffectOp::CommReceive(channel) => {
                match self.channel_registry.receive(channel) {
                    Ok(Some(value)) => {
                        if self.debug_options.log_enabled {
                            self.effect_log.push(format!("CommReceive({}) = {:?}", channel, value));
                        }
                        Ok(OpResult::Value(value))
                    }
                    Ok(None) => {
                        if self.debug_options.log_enabled {
                            self.effect_log.push(format!("CommReceive({}) = <empty>", channel));
                        }
                        Ok(OpResult::Value(Value::Unit))
                    }
                    Err(e) => {
                        if self.debug_options.log_enabled {
                            self.effect_log.push(format!("CommReceive({}) FAILED: {}", channel, e));
                        }
                        Err(InterpreterError::runtime_error(e, op_string.clone()))
                    }
                }
            }
            
            EffectOp::ProofGenerate(claim, witness) => {
                // Simplified proof generation
                let proof = Value::String(format!("proof({:?}, {:?})", claim, witness));
                
                if self.debug_options.log_enabled {
                    self.effect_log.push(format!("ProofGenerate({:?}, {:?}) = {:?}", claim, witness, proof));
                }
                
                Ok(OpResult::Value(proof))
            }
            
            EffectOp::ProofVerify(proof, claim) => {
                // Simplified proof verification
                let valid = match proof {
                    Value::String(s) => s.contains(&format!("{:?}", claim)),
                    _ => false,
                };
                
                if self.debug_options.log_enabled {
                    self.effect_log.push(format!("ProofVerify({:?}, {:?}) = {}", proof, claim, valid));
                }
                
                Ok(OpResult::Bool(valid))
            }
        };
        
        // Take snapshot after operation
        self.take_snapshot(op_string);
        
        result
    }
    
    /// Get the current state
    pub fn get_state(&self) -> &BTreeMap<StateLocation, Value> {
        &self.state
    }
    
    /// Set a state value (useful for initialization)
    pub fn set_state(&mut self, location: StateLocation, value: Value) {
        self.state.insert(location, value);
    }
    
    /// Get the effect log
    pub fn get_effect_log(&self) -> &[String] {
        &self.effect_log
    }
    
    /// Get agent registry
    pub fn get_agent_registry(&mut self) -> &mut AgentRegistry {
        &mut self.agent_registry
    }
    
    /// Get channel registry
    pub fn get_channel_registry(&mut self) -> &mut ChannelManager {
        &mut self.channel_registry
    }
    
    /// Clear the interpreter state (useful for testing)
    pub fn clear(&mut self) {
        self.state.clear();
        self.channel_registry = ChannelManager::new();
        self.effect_log.clear();
        self.snapshots.clear();
        self.step_counter = 0;
        self.error_context = ErrorContext::new();
    }
    
    /// Pretty print the current state
    pub fn print_state(&self) {
        println!("=== Interpreter State ===");
        println!("Step: {}", self.step_counter);
        
        println!("\nState:");
        for (loc, val) in &self.state {
            println!("  {:?} = {:?}", loc, val);
        }
        
        println!("\nChannels:");
        for channel in self.channel_registry.list_channels() {
            if let Some((queue_len, capacity, participants)) = self.channel_registry.get_channel_status(&channel) {
                println!("  {} = {}/{} messages, participants: {:?}", channel, queue_len, capacity, participants);
            }
        }
        
        if !self.effect_log.is_empty() {
            println!("\nRecent Effects:");
            let start = self.effect_log.len().saturating_sub(5);
            for log in &self.effect_log[start..] {
                println!("  {}", log);
            }
        }
    }
}

/// Interpreter error types with enhanced diagnostics
#[derive(Debug)]
pub enum InterpreterError {
    /// Compilation error with source location
    CompilationError {
        message: String,
        source_location: Option<String>,
        context: Vec<String>,
    },
    
    /// Runtime error with call stack
    RuntimeError {
        message: String,
        operation: String,
        state_snapshot: Option<StateSnapshot>,
        call_stack: Vec<String>,
    },
    
    /// Invalid operation with suggestions
    InvalidOperation {
        message: String,
        attempted_operation: String,
        suggestions: Vec<String>,
    },
    
    /// Channel error with detailed information
    ChannelError {
        message: String,
        channel_name: String,
        channel_status: Option<(usize, usize, Vec<String>)>, // (current, capacity, participants)
    },
    
    /// Agent error with registry state
    AgentError {
        message: String,
        agent_id: Option<String>,
        available_agents: Vec<String>,
    },
    
    /// Type error with type information
    TypeError {
        message: String,
        expected_type: String,
        actual_type: String,
        location: Option<String>,
    },
}

impl InterpreterError {
    /// Create a compilation error with context
    pub fn compilation_error(message: impl Into<String>) -> Self {
        InterpreterError::CompilationError {
            message: message.into(),
            source_location: None,
            context: Vec::new(),
        }
    }
    
    /// Create a runtime error with operation context
    pub fn runtime_error(message: impl Into<String>, operation: impl Into<String>) -> Self {
        InterpreterError::RuntimeError {
            message: message.into(),
            operation: operation.into(),
            state_snapshot: None,
            call_stack: Vec::new(),
        }
    }
    
    /// Create a channel error with status information
    pub fn channel_error(message: impl Into<String>, channel_name: impl Into<String>) -> Self {
        InterpreterError::ChannelError {
            message: message.into(),
            channel_name: channel_name.into(),
            channel_status: None,
        }
    }
    
    /// Add context to an error
    pub fn with_context(mut self, context: String) -> Self {
        match &mut self {
            InterpreterError::CompilationError { context: ctx, .. } => ctx.push(context),
            InterpreterError::RuntimeError { call_stack, .. } => call_stack.push(context),
            _ => {}
        }
        self
    }
    
    /// Add state snapshot to runtime error
    pub fn with_snapshot(mut self, snapshot: StateSnapshot) -> Self {
        if let InterpreterError::RuntimeError { state_snapshot, .. } = &mut self {
            *state_snapshot = Some(snapshot);
        }
        self
    }
    
    /// Get detailed diagnostic information
    pub fn get_diagnostic(&self) -> String {
        match self {
            InterpreterError::CompilationError { message, source_location, context } => {
                let mut diag = format!("Compilation Error: {}", message);
                if let Some(loc) = source_location {
                    diag.push_str(&format!("\n  Location: {}", loc));
                }
                if !context.is_empty() {
                    diag.push_str("\n  Context:");
                    for ctx in context {
                        diag.push_str(&format!("\n    - {}", ctx));
                    }
                }
                diag
            }
            
            InterpreterError::RuntimeError { message, operation, state_snapshot, call_stack } => {
                let mut diag = format!("Runtime Error: {}", message);
                diag.push_str(&format!("\n  During operation: {}", operation));
                
                if !call_stack.is_empty() {
                    diag.push_str("\n  Call stack:");
                    for (i, frame) in call_stack.iter().enumerate() {
                        diag.push_str(&format!("\n    {}: {}", i, frame));
                    }
                }
                
                if let Some(snapshot) = state_snapshot {
                    diag.push_str(&format!("\n  State at error (step {}):", snapshot.step));
                    diag.push_str(&format!("\n    State entries: {}", snapshot.state.len()));
                    diag.push_str(&format!("\n    Active channels: {}", snapshot.channels.len()));
                }
                
                diag
            }
            
            InterpreterError::InvalidOperation { message, attempted_operation, suggestions } => {
                let mut diag = format!("Invalid Operation: {}", message);
                diag.push_str(&format!("\n  Attempted: {}", attempted_operation));
                if !suggestions.is_empty() {
                    diag.push_str("\n  Suggestions:");
                    for suggestion in suggestions {
                        diag.push_str(&format!("\n    - {}", suggestion));
                    }
                }
                diag
            }
            
            InterpreterError::ChannelError { message, channel_name, channel_status } => {
                let mut diag = format!("Channel Error: {}", message);
                diag.push_str(&format!("\n  Channel: {}", channel_name));
                if let Some((current, capacity, participants)) = channel_status {
                    diag.push_str(&format!("\n  Status: {}/{} messages", current, capacity));
                    diag.push_str(&format!("\n  Participants: {:?}", participants));
                }
                diag
            }
            
            InterpreterError::AgentError { message, agent_id, available_agents } => {
                let mut diag = format!("Agent Error: {}", message);
                if let Some(id) = agent_id {
                    diag.push_str(&format!("\n  Agent: {}", id));
                }
                if !available_agents.is_empty() {
                    diag.push_str("\n  Available agents:");
                    for agent in available_agents {
                        diag.push_str(&format!("\n    - {}", agent));
                    }
                }
                diag
            }
            
            InterpreterError::TypeError { message, expected_type, actual_type, location } => {
                let mut diag = format!("Type Error: {}", message);
                diag.push_str(&format!("\n  Expected: {}", expected_type));
                diag.push_str(&format!("\n  Actual: {}", actual_type));
                if let Some(loc) = location {
                    diag.push_str(&format!("\n  Location: {}", loc));
                }
                diag
            }
        }
    }
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_diagnostic())
    }
}

impl std::error::Error for InterpreterError {}

/// Enhanced error context for better diagnostics
#[derive(Debug, Clone)]
pub struct ErrorContext {
    operation_stack: Vec<String>,
    current_choreography: Option<String>,
    current_agent: Option<String>,
    execution_step: usize,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorContext {
    pub fn new() -> Self {
        ErrorContext {
            operation_stack: Vec::new(),
            current_choreography: None,
            current_agent: None,
            execution_step: 0,
        }
    }
    
    pub fn push_operation(&mut self, op: String) {
        self.operation_stack.push(op);
    }
    
    pub fn pop_operation(&mut self) {
        self.operation_stack.pop();
    }
    
    pub fn set_choreography(&mut self, choreo: String) {
        self.current_choreography = Some(choreo);
    }
    
    pub fn set_agent(&mut self, agent: String) {
        self.current_agent = Some(agent);
    }
    
    pub fn increment_step(&mut self) {
        self.execution_step += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::choreography::{ChoreographyStep};
    use crate::layer3::agent::{Agent, AgentId};
    use crate::layer3::capability::Capability;
    use crate::layer2::effect::EffectType;
    
    #[test]
    fn test_interpreter_creation() {
        let interp = Interpreter::new();
        assert!(interp.state.is_empty());
        assert!(interp.effect_log.is_empty());
    }
    
    #[test]
    fn test_execute_state_operations() {
        let mut interp = Interpreter::new();
        interp.enable_debug();
        
        // Write to state
        let write_effect = Effect::<(), EffectRow>::write(
            StateLocation("test".to_string()),
            Value::String("hello".to_string())
        );
        
        interp.execute_effect(write_effect).unwrap();
        
        // Read from state
        let read_effect = Effect::<Value, EffectRow>::read(StateLocation("test".to_string()));
        
        let value = interp.execute_effect(read_effect).unwrap();
        assert_eq!(value, Value::String("hello".to_string()));
        
        // Check log - should have Pure, StateWrite, Pure, StateRead
        assert!(interp.get_effect_log().len() >= 2);
        
        // Check that we have both operations logged
        let log_str = interp.get_effect_log().join(" ");
        assert!(log_str.contains("StateWrite"));
        assert!(log_str.contains("StateRead"));
    }
    
    #[test]
    fn test_execute_choreography() {
        let mut interp = Interpreter::new();
        
        // Register agents with capabilities
        let mut alice = Agent::new("Alice");
        let comm_cap = Capability::new(
            "Communication".to_string(),
            EffectRow::from_effects(vec![
                ("comm_send".to_string(), EffectType::Comm),
            ])
        );
        alice.add_capability(comm_cap);
        
        let bob = Agent::new("Bob");
        
        interp.get_agent_registry().register(alice).unwrap();
        interp.get_agent_registry().register(bob).unwrap();
        
        // Create a simple choreography
        use crate::layer3::choreography::Message;
        let step = ChoreographyStep::Send {
            from: AgentId::new("Alice"),
            to: AgentId::new("Bob"),
            message: Message::Text("Hello".to_string()),
        };
        
        let choreo = Choreography::Step(step);
        
        // Execute
        interp.enable_debug();
        interp.execute_choreography(&choreo).unwrap();
        
        // Check that message was sent
        assert!(!interp.get_effect_log().is_empty());
        assert!(interp.get_effect_log()[0].contains("Executing choreography"));
    }
    
    #[test]
    fn test_communication() {
        let mut interp = Interpreter::new();
        interp.enable_debug();
        
        // Send a message
        let send_effect = Effect::<(), EffectRow>::send(
            "test-channel".to_string(),
            Value::String("test message".to_string())
        );
        
        interp.execute_effect(send_effect).unwrap();
        
        // Receive the message
        let recv_effect = Effect::<Value, EffectRow>::receive("test-channel".to_string());
        
        let value = interp.execute_effect(recv_effect).unwrap();
        assert_eq!(value, Value::String("test message".to_string()));
        
        // Second receive should get Unit
        let recv2_effect = Effect::<Value, EffectRow>::receive("test-channel".to_string());
        let value2 = interp.execute_effect(recv2_effect).unwrap();
        assert_eq!(value2, Value::Unit);
    }
    
    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_debug_snapshots() {
        let mut interp = Interpreter::new();
        
        // Enable snapshots
        let mut debug_opts = DebugOptions::default();
        debug_opts.snapshots_enabled = true;
        interp.set_debug_options(debug_opts);
        
        // Execute some operations
        let write1 = Effect::<(), EffectRow>::write(
            StateLocation("x".to_string()),
            Value::Int(42)
        );
        let write2 = Effect::<(), EffectRow>::write(
            StateLocation("y".to_string()),
            Value::String("hello".to_string())
        );
        
        interp.execute_effect(write1).unwrap();
        interp.execute_effect(write2).unwrap();
        
        // Check snapshots
        let snapshots = interp.get_snapshots();
        assert_eq!(snapshots.len(), 2);
        
        // First snapshot should have just x
        assert_eq!(snapshots[0].step, 0);
        assert_eq!(snapshots[0].state.get(&StateLocation("x".to_string())), Some(&Value::Int(42)));
        assert!(!snapshots[0].state.contains_key(&StateLocation("y".to_string())));
        
        // Second snapshot should have both
        assert_eq!(snapshots[1].step, 1);
        assert_eq!(snapshots[1].state.get(&StateLocation("x".to_string())), Some(&Value::Int(42)));
        assert_eq!(snapshots[1].state.get(&StateLocation("y".to_string())), Some(&Value::String("hello".to_string())));
    }
} 