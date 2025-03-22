// Effect interpreter for Causality
//
// This module provides the infrastructure for interpreting Causality effects
// by executing them in a Rust environment.

use std::sync::Arc;
use std::marker::PhantomData;

use crate::effect::Effect;
use crate::error::{Error, Result};
use crate::handler::{EffectHandler, SharedHandler};
use crate::continuation::Continuation;
use crate::types::{Amount, Timestamp, Balance, Account};

/// An interpreter for executing Causality effects
///
/// The Interpreter is responsible for executing effects by resolving them
/// with appropriate handlers and managing continuations.
#[derive(Debug)]
pub struct Interpreter {
    /// The handler used to resolve effects
    handler: SharedHandler,
}

impl Interpreter {
    /// Create a new interpreter with the given handler
    pub fn new(handler: SharedHandler) -> Self {
        Interpreter { handler }
    }
    
    /// Execute an effect and return its result
    ///
    /// This method resolves the effect using the configured handler and
    /// executes any continuations attached to the effect.
    pub fn execute<T>(&self, effect: &dyn Effect<T>) -> Result<T> 
    where
        T: 'static + Send + Sync,
    {
        match effect.kind() {
            crate::effect::EffectKind::Deposit => {
                let account = effect.account();
                let amount = effect.amount().ok_or(Error::MissingAmount)?;
                let timestamp = effect.timestamp().ok_or(Error::MissingTimestamp)?;
                
                match self.handler.handle_deposit(account, amount, timestamp) {
                    Ok(()) => effect.continue_with(()),
                    Err(e) => Err(e),
                }
            },
            crate::effect::EffectKind::Withdrawal => {
                let account = effect.account();
                let amount = effect.amount().ok_or(Error::MissingAmount)?;
                let timestamp = effect.timestamp().ok_or(Error::MissingTimestamp)?;
                
                match self.handler.handle_withdrawal(account, amount, timestamp) {
                    Ok(()) => effect.continue_with(()),
                    Err(e) => Err(e),
                }
            },
            crate::effect::EffectKind::Observation => {
                let account = effect.account();
                let timestamp = effect.timestamp().ok_or(Error::MissingTimestamp)?;
                
                match self.handler.handle_observation(account, timestamp) {
                    Ok(balance) => effect.continue_with(balance),
                    Err(e) => Err(e),
                }
            },
            crate::effect::EffectKind::Pure => {
                // Pure effects don't need handler interaction
                effect.continue_with(())
            },
            crate::effect::EffectKind::Sequence => {
                // Sequence effects need to execute their inner effects in order
                let inner_effects = effect.inner_effects()
                    .ok_or(Error::MissingInnerEffects)?;
                
                let mut result = Ok(());
                for inner_effect in inner_effects {
                    match self.execute(&**inner_effect) {
                        Ok(_) => continue,
                        Err(e) => {
                            result = Err(e);
                            break;
                        }
                    }
                }
                
                match result {
                    Ok(()) => effect.continue_with(()),
                    Err(e) => Err(e),
                }
            },
        }
    }
    
    /// Execute multiple effects in sequence
    ///
    /// This method executes a sequence of effects, stopping on the first error.
    pub fn execute_sequence<T>(&self, effects: &[Box<dyn Effect<T>>]) -> Result<Vec<T>> 
    where
        T: 'static + Send + Sync,
    {
        let mut results = Vec::with_capacity(effects.len());
        
        for effect in effects {
            match self.execute(&**effect) {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
    
    /// Change the handler used by this interpreter
    pub fn with_handler(self, handler: SharedHandler) -> Self {
        Interpreter { handler }
    }
}

/// Trait for types that can be interpreted
///
/// This trait provides a convenient way to execute effects.
pub trait Interpret<T> {
    /// Interpret this effect with the given interpreter
    fn interpret(&self, interpreter: &Interpreter) -> Result<T>;
}

impl<T> Interpret<T> for dyn Effect<T> 
where
    T: 'static + Send + Sync,
{
    fn interpret(&self, interpreter: &Interpreter) -> Result<T> {
        interpreter.execute(self)
    }
}

/// A tracing interpreter that records executed effects
///
/// This interpreter wraps another interpreter and records all effects
/// that are executed, allowing them to be replayed later.
#[derive(Debug)]
pub struct TracingInterpreter<T> {
    /// The underlying interpreter
    inner: Interpreter,
    /// The trace of executed effects
    trace: Vec<Box<dyn Effect<T>>>,
    /// Type marker
    _marker: PhantomData<T>,
}

impl<T> TracingInterpreter<T>
where
    T: 'static + Send + Sync + Clone,
{
    /// Create a new tracing interpreter
    pub fn new(handler: SharedHandler) -> Self {
        TracingInterpreter {
            inner: Interpreter::new(handler),
            trace: Vec::new(),
            _marker: PhantomData,
        }
    }
    
    /// Execute an effect and record it in the trace
    pub fn execute(&mut self, effect: &dyn Effect<T>) -> Result<T> {
        let result = self.inner.execute(effect);
        
        // Only record the effect if execution was successful
        if result.is_ok() {
            self.trace.push(effect.box_clone());
        }
        
        result
    }
    
    /// Get the trace of executed effects
    pub fn trace(&self) -> &[Box<dyn Effect<T>>] {
        &self.trace
    }
    
    /// Clear the trace
    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }
    
    /// Change the handler used by this interpreter
    pub fn with_handler(self, handler: SharedHandler) -> Self {
        TracingInterpreter {
            inner: self.inner.with_handler(handler),
            trace: self.trace,
            _marker: PhantomData,
        }
    }
}

/// A mock interpreter for testing
///
/// This interpreter doesn't actually execute effects, but instead
/// returns mock responses based on configured behavior.
#[derive(Debug)]
pub struct MockInterpreter<T> {
    /// The mock responses to return for each effect kind
    responses: std::collections::HashMap<crate::effect::EffectKind, Result<T>>,
    /// Type marker
    _marker: PhantomData<T>,
}

impl<T> MockInterpreter<T>
where
    T: 'static + Send + Sync + Clone,
{
    /// Create a new mock interpreter
    pub fn new() -> Self {
        MockInterpreter {
            responses: std::collections::HashMap::new(),
            _marker: PhantomData,
        }
    }
    
    /// Set the response for a specific effect kind
    pub fn set_response(&mut self, kind: crate::effect::EffectKind, response: Result<T>) {
        self.responses.insert(kind, response);
    }
    
    /// Execute an effect and return the mock response
    pub fn execute(&self, effect: &dyn Effect<T>) -> Result<T> {
        let kind = effect.kind();
        
        match self.responses.get(&kind) {
            Some(result) => result.clone(),
            None => Err(Error::UnhandledEffect(format!("No mock response configured for effect kind: {:?}", kind))),
        }
    }
}

// Interpreter for executing effects
//
// This module provides an interpreter for executing effects using a handler.

use std::sync::Arc;

use crate::effect::{CoreEffect, Effect};
use crate::handler::EffectHandler;

/// An interpreter for executing effects
///
/// This interpreter executes effects using a handler.
#[derive(Debug)]
pub struct Interpreter<H> {
    /// The handler to use for executing effects
    handler: H,
}

impl<H> Interpreter<H> 
where
    H: EffectHandler,
{
    /// Create a new interpreter with the given handler
    pub fn new(handler: H) -> Self {
        Interpreter { handler }
    }
    
    /// Execute an effect with the interpreter's handler
    pub fn execute<R>(&self, effect: &Box<dyn Effect<Output = R>>) -> R {
        // Create a clone of the effect so we can execute it
        // This is needed because `execute` consumes `self`
        let effect_clone = unsafe { 
            // This is safe because we're just cloning the effect
            // for execution purposes
            std::ptr::read(effect as *const _)
        };
        
        effect_clone.execute(&self.handler)
    }
}

/// A tracing interpreter that records executed effects
///
/// This interpreter wraps another handler and records the effects that are executed.
#[derive(Debug)]
pub struct TracingInterpreter<H> {
    /// The inner handler
    handler: H,
    /// The trace of executed effects
    trace: Vec<TraceEntry>,
}

/// A trace entry recording an executed effect
#[derive(Debug)]
pub struct TraceEntry {
    /// The type of effect that was executed
    effect_type: String,
    /// The timestamp when the effect was executed
    timestamp: std::time::SystemTime,
}

impl TraceEntry {
    /// Create a new trace entry
    fn new<R>(effect: &Box<dyn Effect<Output = R>>) -> Self {
        TraceEntry {
            effect_type: format!("{:?}", effect),
            timestamp: std::time::SystemTime::now(),
        }
    }
    
    /// Get the effect type
    pub fn effect_type(&self) -> &str {
        &self.effect_type
    }
    
    /// Get the timestamp
    pub fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }
}

impl<H> TracingInterpreter<H>
where
    H: EffectHandler,
{
    /// Create a new tracing interpreter
    pub fn new(handler: H) -> Self {
        TracingInterpreter {
            handler,
            trace: Vec::new(),
        }
    }
    
    /// Execute an effect and record it in the trace
    pub fn execute<R>(&mut self, effect: &Box<dyn Effect<Output = R>>) -> R {
        // Add the effect to the trace
        self.trace.push(TraceEntry::new(effect));
        
        // Create a clone of the effect so we can execute it
        let effect_clone = unsafe { 
            // This is safe because we're just cloning the effect
            // for execution purposes
            std::ptr::read(effect as *const _)
        };
        
        effect_clone.execute(&self.handler)
    }
    
    /// Get the trace of executed effects
    pub fn trace(&self) -> &[TraceEntry] {
        &self.trace
    }
    
    /// Clear the trace
    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }
    
    /// Get the inner handler
    pub fn handler(&self) -> &H {
        &self.handler
    }
} 