// Integration patterns for system components
// Original file: src/integration.rs

//! Program Integration Module
//!
//! This module provides functionality for integrating the log system with program execution,
//! enabling programs to log effects and reconstruct state from logs.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{Serialize, Deserialize};
use thiserror::Error;

// use crate::log::{LogEntry, LogStorage, EntryType, EntryData}; // Old incorrect path
use causality_engine::log::{LogEntry, LogStorage, EntryType, EntryData};
use causality_engine::log::{FactEntry, EffectEntry, EventEntry};
use causality_engine::log::event_entry::EventSeverity;
use causality_types::{DomainId, Timestamp};
use causality_error::Error as LogError;
use causality_error::CausalityError;
use uuid::Uuid;

/// Result type for integration operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during program integration
#[derive(Error, Debug)]
pub enum Error {
    /// Error with log operations
    #[error("Log error: {0}")]
    Log(#[from] LogError), // Use the aliased LogError
    
    /// Error with program state
    #[error("State error: {0}")]
    State(String),
    
    /// Error with replay
    #[error("Replay error: {0}")]
    Replay(String),
    
    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),
}

impl From<Box<dyn CausalityError>> for Error {
    fn from(err: Box<dyn CausalityError>) -> Self {
        Error::Storage(format!("Storage error: {}", err))
    }
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
    state_handler: Arc<Mutex<S>>,
    /// Current entry position
    current_position: Arc<Mutex<usize>>,
    /// Last checkpoint position
    last_checkpoint: Arc<Mutex<usize>>,
    #[allow(dead_code)]
    start_time: Instant,
}

impl<S: StateHandler> ProgramIntegration<S> {
    /// Create a new program integration system
    pub fn new(config: IntegrationConfig, storage: Arc<dyn LogStorage>, state_handler: S) -> Self {
        ProgramIntegration {
            config,
            storage,
            state_handler: Arc::new(Mutex::new(state_handler)),
            current_position: Arc::new(Mutex::new(0)),
            last_checkpoint: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
        }
    }
    
    /// Log an effect produced by the program
    pub async fn log_effect(&self, effect_type: &str, data: serde_json::Value) -> Result<LogEntry> {
        // Create parameters map
        let mut parameters = HashMap::new();
        parameters.insert("data".to_string(), data);
        
        // Create EffectEntry with the needed fields
        let effect_entry = EffectEntry {
            effect_type: effect_type.to_string(),
            resources: Some(vec![]),
            domains: Some(vec![DomainId::new(&self.config.domain)]),
            code_hash: None,
            parameters: serde_json::Value::Object(serde_json::Map::from_iter(parameters.into_iter().map(|(k, v)| (k, v)))),
            result: None,
            success: true,
            error: None,
        };
        
        // Create EntryData as a tuple variant
        let entry_data = EntryData::Effect(effect_entry);
        
        // Generate a unique ID for the entry
        let entry_id = format!("eff_{}", Uuid::new_v4());
        
        // Create the log entry with the new signature
        let entry = LogEntry::new(entry_id, EntryType::Effect, entry_data);
        
        // Store a copy for return
        let return_entry = entry.clone();
        
        // Update program state
        {
            let mut state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.handle_effect(&entry)?;
        }
        
        // Log the entry
        self.storage.append_entry(entry.clone()).await?;
        
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
        
        Ok(return_entry)
    }
    
    /// Log a fact observed by the program
    pub async fn log_fact(&self, domain: &str, fact_type: &str, data: serde_json::Value) -> Result<LogEntry> {
        // Create FactEntry with the needed fields
        let fact_entry = FactEntry {
            fact_id: format!("fact_{}", Uuid::new_v4()),
            fact_type: fact_type.to_string(),
            domain_id: DomainId::new(domain),
            height: 0,
            hash: "".to_string(),
            timestamp: Timestamp::now(),
            resources: Some(vec![]),
            domains: Some(vec![]),
            data,
        };
        
        // Create EntryData as a tuple variant
        let entry_data = EntryData::Fact(fact_entry);
        
        // Generate a unique ID for the entry
        let entry_id = format!("fact_{}", Uuid::new_v4());
        
        // Create the log entry with the new signature
        let entry = LogEntry::new(entry_id, EntryType::Fact, entry_data);
        
        // Store a copy for return
        let return_entry = entry.clone();
        
        // Update program state
        {
            let mut state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.handle_fact(&entry)?;
        }
        
        // Log the entry
        self.storage.append_entry(entry.clone()).await?;
        
        // Update position
        {
            let mut position = self.current_position.lock().map_err(|_| {
                Error::State("Failed to acquire lock on current position".to_string())
            })?;
            
            *position += 1;
        }
        
        Ok(return_entry)
    }
    
    /// Log an event emitted by the program
    pub async fn log_event(&self, event_type: &str, data: serde_json::Value) -> Result<LogEntry> {
        // Create EventEntry with the needed fields
        let event_entry = EventEntry::new(
            event_type.to_string(),
            EventSeverity::Info,
            "application".to_string(),
            data,
            None, // resources
            Some(vec![DomainId::new(&self.config.domain)])
        );
        
        // Create EntryData as a tuple variant
        let entry_data = EntryData::Event(event_entry);
        
        // Generate a unique ID for the entry
        let entry_id = format!("evt_{}", Uuid::new_v4());
        
        // Create the log entry with the new signature
        let entry = LogEntry::new(entry_id, EntryType::Event, entry_data);
        
        // Store a copy for return
        let return_entry = entry.clone();
        
        // Update program state
        {
            let mut state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.handle_event(&entry)?;
        }
        
        // Log the entry
        self.storage.append_entry(entry.clone()).await?;
        
        // Update position
        {
            let mut position = self.current_position.lock().map_err(|_| {
                Error::State("Failed to acquire lock on current position".to_string())
            })?;
            
            *position += 1;
        }
        
        Ok(return_entry)
    }
    
    /// Get entries from the log
    pub async fn get_entries(&self, start: usize, count: usize) -> Result<Vec<LogEntry>> {
        match self.storage.get_entries(start, count).await {
            Ok(entries) => Ok(entries),
            Err(e) => Err(Error::Storage(format!("Failed to get entries: {}", e))),
        }
    }
    
    /// Replay log entries
    pub async fn replay_log(&self, start_position: usize, end_position: Option<usize>) -> Result<()> {
        let end_pos = match end_position {
            Some(pos) => pos,
            None => {
                match self.storage.get_entry_count().await {
                    Ok(count) => count,
                    Err(_) => 0,
                }
            }
        };
        
        // Skip if end position is less than or equal to start position
        if end_pos <= start_position {
            return Ok(());
        }
        
        // Get the entries to replay
        let entries = self.storage.get_entries(start_position, end_pos - start_position).await?;
        
        // Process each entry
        for entry in entries {
            // Verify integrity if configured
            if self.config.verify_integrity && entry.entry_hash.is_some() {
                // In this implementation we assume entries are valid
                // In a real implementation, you would check the hash
            }
            
            // Process by entry type
            match &entry.data {
                EntryData::Fact(_fact) => {
                    let mut state_handler = self.state_handler.lock().map_err(|_| {
                        Error::State("Failed to acquire lock on state handler".to_string())
                    })?;
                    
                    state_handler.handle_fact(&entry)?;
                },
                EntryData::Effect(_effect) => {
                    let mut state_handler = self.state_handler.lock().map_err(|_| {
                        Error::State("Failed to acquire lock on state handler".to_string())
                    })?;
                    
                    state_handler.handle_effect(&entry)?;
                },
                EntryData::Event(event) => {
                    // Check if this is a checkpoint entry
                    if entry.metadata.get("checkpoint").map_or(false, |v| v == "true") {
                        let mut state_handler = self.state_handler.lock().map_err(|_| {
                            Error::State("Failed to acquire lock on state handler".to_string())
                        })?;
                        
                        state_handler.set_state(event.details.clone())?;
                    } else {
                        let mut state_handler = self.state_handler.lock().map_err(|_| {
                            Error::State("Failed to acquire lock on state handler".to_string())
                        })?;
                        
                        state_handler.handle_event(&entry)?;
                    }
                },
                _ => {
                    // Skip other entry types for now
                }
            }
        }
        
        // Update current position
        {
            let mut position = self.current_position.lock().map_err(|_| {
                Error::State("Failed to acquire lock on current position".to_string())
            })?;
            
            *position = end_pos;
        }
        
        Ok(())
    }
    
    /// Create a checkpoint of the current program state
    pub async fn create_checkpoint(&self) -> Result<()> {
        // Get the current state
        let state = {
            let state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.get_state()?
        };
        
        // Create EventEntry with the needed fields
        let event_entry = EventEntry::new(
            "checkpoint".to_string(),
            EventSeverity::Info,
            "application".to_string(),
            state,
            None, // resources
            Some(vec![DomainId::new(&self.config.domain)])
        );
        
        // Create EntryData as a tuple variant
        let entry_data = EntryData::Event(event_entry);
        
        // Generate a unique ID for the entry
        let entry_id = format!("chk_{}", Uuid::new_v4());
        
        // Create checkpoint entry
        let mut entry = LogEntry::new(entry_id, EntryType::Event, entry_data);
        
        // Add checkpoint metadata
        entry.metadata.insert("checkpoint".to_string(), "true".to_string());
        entry.metadata.insert("position".to_string(), self.get_current_position()?.to_string());
        
        // Add the checkpoint entry
        self.storage.append_entry(entry).await?;
        
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
        let entries = self.storage.get_entries(0, self.storage.get_entry_count().await?).await?;
        
        let mut checkpoint_position = 0;
        let mut checkpoint_entry = None;
        
        for (i, entry) in entries.iter().enumerate().rev() {
            if let Some(is_checkpoint) = entry.metadata.get("checkpoint") {
                if is_checkpoint == "true" {
                    if let EntryData::Event(event) = &entry.data {
                        checkpoint_position = i;
                        checkpoint_entry = Some((entry, event.details.clone()));
                        break;
                    }
                }
            }
        }
        
        if let Some((_entry, state)) = checkpoint_entry {
            // Reset state handler with checkpoint data
            {
                let mut state_handler = self.state_handler.lock().map_err(|_| {
                    Error::State("Failed to acquire lock on state handler".to_string())
                })?;
                
                state_handler.set_state(state)?;
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
        if let EntryData::Effect(effect_data) = &effect.data {
            let effect_type = effect_data.effect_type.to_string();
            let params = match effect_data.parameters.get("data") {
                Some(data) => data.clone(),
                None => serde_json::Value::Null,
            };
            self.apply_effect(&effect_type, &params)
        } else {
            Ok(())
        }
    }
    
    fn handle_fact(&mut self, fact: &LogEntry) -> Result<()> {
        if let EntryData::Fact(fact_data) = &fact.data {
            self.apply_fact(&fact_data.fact_type, &fact_data.data)
        } else {
            Ok(())
        }
    }
    
    fn handle_event(&mut self, event: &LogEntry) -> Result<()> {
        if let EntryData::Event(event_data) = &event.data {
            self.apply_event(&event_data.event_name, &event_data.details)
        } else {
            Ok(())
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