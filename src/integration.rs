//! Program Integration Module
//!
//! This module provides functionality for integrating the log system with program execution,
//! enabling programs to log effects and reconstruct state from logs.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::log::{LogEntry, LogStorage, EntryType, EntryData};

/// Result type for integration operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during program integration
#[derive(Error, Debug)]
pub enum Error {
    /// Error with log operations
    #[error("Log error: {0}")]
    Log(#[from] crate::log::Error),
    
    /// Error with program state
    #[error("State error: {0}")]
    State(String),
    
    /// Error with replay
    #[error("Replay error: {0}")]
    Replay(String),
    
    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// A handler for program state updates during replay
pub trait StateHandler: Send + Sync {
    /// Handle an effect entry
    fn handle_effect(&mut self, effect: &LogEntry) -> Result<()>;
    
    /// Handle a fact entry
    fn handle_fact(&mut self, fact: &LogEntry) -> Result<()>;
    
    /// Handle an event entry
    fn handle_event(&mut self, event: &LogEntry) -> Result<()>;
    
    /// Get the current state
    fn get_state(&self) -> Result<serde_json::Value>;
    
    /// Set the state directly (e.g., for initialization)
    fn set_state(&mut self, state: serde_json::Value) -> Result<()>;
}

/// Configuration for program integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// The domain for the program
    pub domain: String,
    /// Whether to verify log integrity during replay
    pub verify_integrity: bool,
    /// Maximum batch size for log operations
    pub max_batch_size: usize,
    /// Checkpoint interval (number of entries)
    pub checkpoint_interval: usize,
    /// Whether to enable automatic checkpoints
    pub auto_checkpoint: bool,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        IntegrationConfig {
            domain: "default".to_string(),
            verify_integrity: true,
            max_batch_size: 100,
            checkpoint_interval: 1000,
            auto_checkpoint: true,
        }
    }
}

/// A system for integrating program execution with logging
pub struct ProgramIntegration<S: StateHandler> {
    /// Configuration
    config: IntegrationConfig,
    /// Log storage
    storage: Arc<dyn LogStorage>,
    /// Program state handler
    state_handler: Mutex<S>,
    /// Current entry position
    current_position: Mutex<usize>,
    /// Last checkpoint position
    last_checkpoint: Mutex<usize>,
    /// Start time for tracking performance
    start_time: Instant,
}

impl<S: StateHandler> ProgramIntegration<S> {
    /// Create a new program integration system
    pub fn new(config: IntegrationConfig, storage: Arc<dyn LogStorage>, state_handler: S) -> Self {
        ProgramIntegration {
            config,
            storage,
            state_handler: Mutex::new(state_handler),
            current_position: Mutex::new(0),
            last_checkpoint: Mutex::new(0),
            start_time: Instant::now(),
        }
    }
    
    /// Log an effect produced by the program
    pub async fn log_effect(&self, effect_type: &str, data: serde_json::Value) -> Result<LogEntry> {
        let entry_data = EntryData::Effect {
            domain: self.config.domain.clone(),
            effect_type: effect_type.to_string(),
            data,
        };
        
        let entry = LogEntry::new(EntryType::Effect, entry_data);
        self.storage.add_entry(&entry).await?;
        
        // Update program state
        {
            let mut state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.handle_effect(&entry)?;
        }
        
        // Update position
        {
            let mut position = self.current_position.lock().map_err(|_| {
                Error::State("Failed to acquire lock on current position".to_string())
            })?;
            
            *position += 1;
            
            // Check if we need to create a checkpoint
            if self.config.auto_checkpoint && *position % self.config.checkpoint_interval == 0 {
                self.create_checkpoint().await?;
            }
        }
        
        Ok(entry)
    }
    
    /// Log a fact observed by the program
    pub async fn log_fact(&self, domain: &str, fact_type: &str, data: serde_json::Value) -> Result<LogEntry> {
        let entry_data = EntryData::Fact {
            domain: domain.to_string(),
            fact_type: fact_type.to_string(),
            data,
            source: None,
        };
        
        let entry = LogEntry::new(EntryType::Fact, entry_data);
        self.storage.add_entry(&entry).await?;
        
        // Update program state
        {
            let mut state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.handle_fact(&entry)?;
        }
        
        // Update position
        {
            let mut position = self.current_position.lock().map_err(|_| {
                Error::State("Failed to acquire lock on current position".to_string())
            })?;
            
            *position += 1;
        }
        
        Ok(entry)
    }
    
    /// Log an event emitted by the program
    pub async fn log_event(&self, event_type: &str, data: serde_json::Value) -> Result<LogEntry> {
        let entry_data = EntryData::Event {
            domain: self.config.domain.clone(),
            event_type: event_type.to_string(),
            data,
        };
        
        let entry = LogEntry::new(EntryType::Event, entry_data);
        self.storage.add_entry(&entry).await?;
        
        // Update program state
        {
            let mut state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.handle_event(&entry)?;
        }
        
        // Update position
        {
            let mut position = self.current_position.lock().map_err(|_| {
                Error::State("Failed to acquire lock on current position".to_string())
            })?;
            
            *position += 1;
        }
        
        Ok(entry)
    }
    
    /// Replay log entries to reconstruct state
    pub async fn replay_log(&self, start_position: usize, end_position: Option<usize>) -> Result<()> {
        // Get entries to replay
        let end = end_position.unwrap_or_else(|| {
            match self.storage.get_entry_count() {
                Ok(count) => count,
                Err(_) => 0,
            }
        });
        
        if start_position >= end {
            return Ok(());
        }
        
        // Process entries in batches
        let mut current = start_position;
        while current < end {
            let batch_end = (current + self.config.max_batch_size).min(end);
            let entries = self.storage.get_entries(current, batch_end).await?;
            
            for entry in &entries {
                // Verify integrity if enabled
                if self.config.verify_integrity && !entry.verify_hash() {
                    return Err(Error::Replay(format!(
                        "Hash verification failed for entry at position {}", current
                    )));
                }
                
                // Apply entry to state
                let mut state_handler = self.state_handler.lock().map_err(|_| {
                    Error::State("Failed to acquire lock on state handler".to_string())
                })?;
                
                match entry.entry_type {
                    EntryType::Effect => state_handler.handle_effect(entry)?,
                    EntryType::Fact => state_handler.handle_fact(entry)?,
                    EntryType::Event => state_handler.handle_event(entry)?,
                }
                
                current += 1;
            }
            
            // Update current position
            {
                let mut position = self.current_position.lock().map_err(|_| {
                    Error::State("Failed to acquire lock on current position".to_string())
                })?;
                
                *position = current;
            }
        }
        
        Ok(())
    }
    
    /// Create a checkpoint of the current state
    pub async fn create_checkpoint(&self) -> Result<()> {
        // Get current state
        let state = {
            let state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.get_state()?
        };
        
        // Create checkpoint entry
        let metadata = {
            let mut metadata = HashMap::new();
            metadata.insert("checkpoint".to_string(), "true".to_string());
            metadata.insert("position".to_string(), self.get_current_position()?.to_string());
            metadata
        };
        
        let entry_data = EntryData::Event {
            domain: self.config.domain.clone(),
            event_type: "checkpoint".to_string(),
            data: state,
        };
        
        let mut entry = LogEntry::new(EntryType::Event, entry_data);
        entry.metadata = metadata;
        
        // Add checkpoint to log
        self.storage.add_entry(&entry).await?;
        
        // Update last checkpoint position
        {
            let mut last_checkpoint = self.last_checkpoint.lock().map_err(|_| {
                Error::State("Failed to acquire lock on last checkpoint".to_string())
            })?;
            
            *last_checkpoint = self.get_current_position()?;
        }
        
        Ok(())
    }
    
    /// Restore from the latest checkpoint
    pub async fn restore_from_checkpoint(&self) -> Result<usize> {
        // Find the latest checkpoint
        let entries = self.storage.get_entries(0, self.storage.get_entry_count()?).await?;
        
        let mut checkpoint_position = 0;
        let mut checkpoint_entry = None;
        
        for (i, entry) in entries.iter().enumerate().rev() {
            if entry.entry_type == EntryType::Event {
                if let EntryData::Event { event_type, .. } = &entry.entry_data {
                    if event_type == "checkpoint" {
                        checkpoint_position = i;
                        checkpoint_entry = Some(entry);
                        break;
                    }
                }
            }
        }
        
        if let Some(entry) = checkpoint_entry {
            // Extract state from checkpoint
            if let EntryData::Event { data, .. } = &entry.entry_data {
                // Reset state handler with checkpoint data
                {
                    let mut state_handler = self.state_handler.lock().map_err(|_| {
                        Error::State("Failed to acquire lock on state handler".to_string())
                    })?;
                    
                    state_handler.set_state(data.clone())?;
                }
                
                // Update position
                {
                    let mut position = self.current_position.lock().map_err(|_| {
                        Error::State("Failed to acquire lock on current position".to_string())
                    })?;
                    
                    *position = checkpoint_position;
                }
                
                // Update last checkpoint
                {
                    let mut last_checkpoint = self.last_checkpoint.lock().map_err(|_| {
                        Error::State("Failed to acquire lock on last checkpoint".to_string())
                    })?;
                    
                    *last_checkpoint = checkpoint_position;
                }
                
                return Ok(checkpoint_position);
            }
        }
        
        Err(Error::Replay("No checkpoint found".to_string()))
    }
    
    /// Recover program state from log
    pub async fn recover_state(&self) -> Result<()> {
        // First try to recover from checkpoint
        let checkpoint_position = match self.restore_from_checkpoint().await {
            Ok(pos) => pos + 1, // Start after the checkpoint
            Err(_) => 0,        // No checkpoint, start from the beginning
        };
        
        // Replay log from checkpoint to latest entry
        self.replay_log(checkpoint_position, None).await?;
        
        Ok(())
    }
    
    /// Get the current position in the log
    pub fn get_current_position(&self) -> Result<usize> {
        let position = self.current_position.lock().map_err(|_| {
            Error::State("Failed to acquire lock on current position".to_string())
        })?;
        
        Ok(*position)
    }
    
    /// Get the current program state
    pub fn get_state(&self) -> Result<serde_json::Value> {
        let state_handler = self.state_handler.lock().map_err(|_| {
            Error::State("Failed to acquire lock on state handler".to_string())
        })?;
        
        state_handler.get_state()
    }
}

/// A simple state handler that uses a JSON value as state
pub struct JsonStateHandler {
    /// The current state
    state: serde_json::Value,
    /// Custom effect handlers
    effect_handlers: HashMap<String, Box<dyn Fn(&mut serde_json::Value, &serde_json::Value) -> Result<()> + Send + Sync>>,
    /// Custom fact handlers
    fact_handlers: HashMap<String, Box<dyn Fn(&mut serde_json::Value, &serde_json::Value) -> Result<()> + Send + Sync>>,
    /// Custom event handlers
    event_handlers: HashMap<String, Box<dyn Fn(&mut serde_json::Value, &serde_json::Value) -> Result<()> + Send + Sync>>,
}

impl JsonStateHandler {
    /// Create a new JSON state handler
    pub fn new() -> Self {
        JsonStateHandler {
            state: serde_json::Value::Object(serde_json::Map::new()),
            effect_handlers: HashMap::new(),
            fact_handlers: HashMap::new(),
            event_handlers: HashMap::new(),
        }
    }
    
    /// Create a new JSON state handler with initial state
    pub fn with_state(state: serde_json::Value) -> Self {
        JsonStateHandler {
            state,
            effect_handlers: HashMap::new(),
            fact_handlers: HashMap::new(),
            event_handlers: HashMap::new(),
        }
    }
    
    /// Register a custom effect handler
    pub fn register_effect_handler<F>(&mut self, effect_type: &str, handler: F)
    where
        F: Fn(&mut serde_json::Value, &serde_json::Value) -> Result<()> + Send + Sync + 'static,
    {
        self.effect_handlers.insert(effect_type.to_string(), Box::new(handler));
    }
    
    /// Register a custom fact handler
    pub fn register_fact_handler<F>(&mut self, fact_type: &str, handler: F)
    where
        F: Fn(&mut serde_json::Value, &serde_json::Value) -> Result<()> + Send + Sync + 'static,
    {
        self.fact_handlers.insert(fact_type.to_string(), Box::new(handler));
    }
    
    /// Register a custom event handler
    pub fn register_event_handler<F>(&mut self, event_type: &str, handler: F)
    where
        F: Fn(&mut serde_json::Value, &serde_json::Value) -> Result<()> + Send + Sync + 'static,
    {
        self.event_handlers.insert(event_type.to_string(), Box::new(handler));
    }
    
    /// Apply an effect to the state
    fn apply_effect(&mut self, effect_type: &str, data: &serde_json::Value) -> Result<()> {
        if let Some(handler) = self.effect_handlers.get(effect_type) {
            handler(&mut self.state, data)
        } else {
            // Default behavior: merge data into state
            if let serde_json::Value::Object(state_obj) = &mut self.state {
                if let serde_json::Value::Object(data_obj) = data {
                    for (key, value) in data_obj {
                        state_obj.insert(key.clone(), value.clone());
                    }
                    Ok(())
                } else {
                    // Not an object, store under effect type key
                    state_obj.insert(effect_type.to_string(), data.clone());
                    Ok(())
                }
            } else {
                Err(Error::State("State is not an object".to_string()))
            }
        }
    }
    
    /// Apply a fact to the state
    fn apply_fact(&mut self, fact_type: &str, data: &serde_json::Value) -> Result<()> {
        if let Some(handler) = self.fact_handlers.get(fact_type) {
            handler(&mut self.state, data)
        } else {
            // Default behavior: store facts in a "facts" object
            if let serde_json::Value::Object(state_obj) = &mut self.state {
                let facts = state_obj.entry("facts").or_insert_with(|| {
                    serde_json::Value::Object(serde_json::Map::new())
                });
                
                if let serde_json::Value::Object(facts_obj) = facts {
                    let fact_list = facts_obj.entry(fact_type).or_insert_with(|| {
                        serde_json::Value::Array(Vec::new())
                    });
                    
                    if let serde_json::Value::Array(list) = fact_list {
                        list.push(data.clone());
                    } else {
                        *fact_list = serde_json::Value::Array(vec![data.clone()]);
                    }
                    
                    Ok(())
                } else {
                    Err(Error::State("Facts is not an object".to_string()))
                }
            } else {
                Err(Error::State("State is not an object".to_string()))
            }
        }
    }
    
    /// Apply an event to the state
    fn apply_event(&mut self, event_type: &str, data: &serde_json::Value) -> Result<()> {
        if let Some(handler) = self.event_handlers.get(event_type) {
            handler(&mut self.state, data)
        } else {
            // Default behavior: store events in an "events" object
            if let serde_json::Value::Object(state_obj) = &mut self.state {
                let events = state_obj.entry("events").or_insert_with(|| {
                    serde_json::Value::Object(serde_json::Map::new())
                });
                
                if let serde_json::Value::Object(events_obj) = events {
                    let event_list = events_obj.entry(event_type).or_insert_with(|| {
                        serde_json::Value::Array(Vec::new())
                    });
                    
                    if let serde_json::Value::Array(list) = event_list {
                        list.push(data.clone());
                    } else {
                        *event_list = serde_json::Value::Array(vec![data.clone()]);
                    }
                    
                    Ok(())
                } else {
                    Err(Error::State("Events is not an object".to_string()))
                }
            } else {
                Err(Error::State("State is not an object".to_string()))
            }
        }
    }
}

impl StateHandler for JsonStateHandler {
    fn handle_effect(&mut self, effect: &LogEntry) -> Result<()> {
        if let EntryData::Effect { effect_type, data, .. } = &effect.entry_data {
            self.apply_effect(effect_type, data)
        } else {
            Err(Error::InvalidOperation("Not an effect entry".to_string()))
        }
    }
    
    fn handle_fact(&mut self, fact: &LogEntry) -> Result<()> {
        if let EntryData::Fact { fact_type, data, .. } = &fact.entry_data {
            self.apply_fact(fact_type, data)
        } else {
            Err(Error::InvalidOperation("Not a fact entry".to_string()))
        }
    }
    
    fn handle_event(&mut self, event: &LogEntry) -> Result<()> {
        if let EntryData::Event { event_type, data, .. } = &event.entry_data {
            self.apply_event(event_type, data)
        } else {
            Err(Error::InvalidOperation("Not an event entry".to_string()))
        }
    }
    
    fn get_state(&self) -> Result<serde_json::Value> {
        Ok(self.state.clone())
    }
    
    fn set_state(&mut self, state: serde_json::Value) -> Result<()> {
        self.state = state;
        Ok(())
    }
}

impl Default for JsonStateHandler {
    fn default() -> Self {
        Self::new()
    }
} 