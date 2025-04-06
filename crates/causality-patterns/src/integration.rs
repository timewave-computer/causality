// Integration patterns for system components
// Original file: src/integration.rs

//! Program Integration Module
//!
//! This module provides functionality for integrating the log system with program execution,
//! enabling programs to log effects and reconstruct state from logs.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};
use thiserror::Error;

// use crate::log::{LogEntry, LogStorage, EntryType, EntryData}; // Old incorrect path
use causality_engine::log::{LogEntry, LogStorage, EntryType, EntryData};
use causality_engine::log::{FactEntry, EffectEntry, EventEntry};
use causality_engine::log::event_entry::EventSeverity;
use causality_types::{DomainId, Timestamp};
use causality_error::Error as LogError;
use causality_error::CausalityError;
use rand::RngCore; // Import RngCore for random bytes

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

impl From<causality_types::ContentAddressingError> for Error {
    fn from(err: causality_types::ContentAddressingError) -> Self {
        Error::Storage(format!("Content addressing error: {}", err))
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
        // Temporary ID generation using timestamp + nonce (replace Uuid)
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let nonce = rand::thread_rng().next_u64();
        let entry_id = format!("eff_{}_{}", timestamp, nonce);
        
        // Create parameters map
        let mut parameters = HashMap::new();
        parameters.insert("data".to_string(), causality_engine::log::types::BorshJsonValue(data));
        
        // Create EffectEntry with the needed fields
        let effect_entry = EffectEntry {
            effect_type: causality_engine::log::types::SerializableEffectType(effect_type.to_string()),
            resources: Vec::new(),
            domains: vec![DomainId::new(&self.config.domain)],
            code_hash: None,
            parameters,
            result: None,
            success: true,
            error: None,
            domain_id: DomainId::new(&self.config.domain),
            effect_id: effect_type.to_string(),
            status: "success".to_string(),
        };
        
        // Create EntryData as a tuple variant
        let entry_data = EntryData::Effect(effect_entry);
        
        // Create the log entry with the new signature
        let entry = LogEntry::new(
            EntryType::Effect,
            entry_data,
            None, // trace_id
            None, // parent_id
            HashMap::new(), // metadata
        )?;
        
        // Store a copy for return
        let return_entry = entry.clone();
        
        // Update program state
        {
            let mut state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.handle_effect(&return_entry)?;
        }
        
        // Log the entry
        self.storage.append_entry(return_entry.clone()).await?;
        
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
        // Temporary ID generation using timestamp + nonce (replace Uuid)
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let nonce = rand::thread_rng().next_u64();
        let entry_id = format!("fact_{}_{}", timestamp, nonce);
        
        // Create FactEntry with the needed fields
        let fact_entry = FactEntry {
            domain: DomainId::new(domain),
            block_height: 0,
            block_hash: None,
            observed_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
            fact_type: fact_type.to_string(),
            resources: Vec::new(),
            data: causality_engine::log::types::BorshJsonValue(data),
            verified: false,
            domain_id: DomainId::new(domain),
            fact_id: entry_id.clone(),
        };
        
        // Create EntryData as a tuple variant
        let entry_data = EntryData::Fact(fact_entry);
        
        // Create the log entry with the new signature
        let entry = LogEntry::new(
            EntryType::Fact,
            entry_data,
            None, // trace_id
            None, // parent_id
            HashMap::new(), // metadata
        )?;
        
        // Store a copy for return
        let return_entry = entry.clone();
        
        // Update program state
        {
            let mut state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.handle_fact(&return_entry)?;
        }
        
        // Log the entry
        self.storage.append_entry(return_entry.clone()).await?;
        
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
        // Temporary ID generation using timestamp + nonce (replace Uuid)
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let nonce = rand::thread_rng().next_u64();
        let entry_id = format!("evt_{}_{}", timestamp, nonce);
        
        // Create event entry
        let event_entry = EventEntry::new(
            event_type.to_string(),
            EventSeverity::Info,
            "system".to_string(),
            causality_engine::log::types::BorshJsonValue(data),
            None,
            Some(vec![DomainId::new(&self.config.domain)]),
        );
        
        // Create EntryData as a tuple variant
        let entry_data = EntryData::Event(event_entry);
        
        // Create the log entry with the new signature
        let entry = LogEntry::new(
            EntryType::Event,
            entry_data,
            None, // trace_id
            None, // parent_id
            HashMap::new(), // metadata
        )?;
        
        // Store a copy for return
        let return_entry = entry.clone();
        
        // Update program state
        {
            let mut state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.handle_event(&return_entry)?;
        }
        
        // Log the entry
        self.storage.append_entry(return_entry.clone()).await?;
        
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
        for i in 0..entries.len() {
            // Process the entry
            let entry = entries[i].clone();
            
            // Verify entry integrity if configured
            if self.config.verify_integrity {
                self.verify_entry_integrity(&entry)?;
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
                    if event.event_name == "state" {
                        let mut state_handler = self.state_handler.lock().map_err(|_| {
                            Error::State("Failed to acquire lock on state handler".to_string())
                        })?;
                        
                        // Extract and convert BorshJsonValue to serde_json::Value for set_state
                        let json_value = match &event.details {
                            causality_engine::log::types::BorshJsonValue(val) => val.clone(),
                        };
                        
                        state_handler.set_state(json_value)?;
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
    
    /// Create a checkpoint with the current state
    pub async fn create_checkpoint(&self) -> Result<()> {
        // Get the current state
        let state = {
            let state_handler = self.state_handler.lock().map_err(|_| {
                Error::State("Failed to acquire lock on state handler".to_string())
            })?;
            
            state_handler.get_state()?
        };
        
        // Get the current position
        let position = self.get_current_position()?;
        
        // Temporary ID generation
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let nonce = rand::thread_rng().next_u64();
        let entry_id = format!("chk_{}_{}", timestamp, nonce);
        
        // Create event entry
        let event_entry = EventEntry::new(
            "checkpoint".to_string(),
            EventSeverity::Info,
            "system".to_string(),
            causality_engine::log::types::BorshJsonValue(state),
            None,
            Some(vec![DomainId::new(&self.config.domain)]),
        );
        
        // Create EntryData
        let entry_data = EntryData::Event(event_entry);
        
        // Create the log entry
        let mut entry = LogEntry::new(
            EntryType::Event,
            entry_data,
            None, // trace_id
            None, // parent_id
            HashMap::new(), // metadata
        )?;
        
        // Add metadata
        entry.metadata.insert("checkpoint".to_string(), "true".to_string());
        entry.metadata.insert("position".to_string(), self.get_current_position()?.to_string());
        
        // Log the entry
        self.storage.append_entry(entry).await?;
        
        // Update last checkpoint position
        {
            let mut last_checkpoint = self.last_checkpoint.lock().map_err(|_| {
                Error::State("Failed to acquire lock on last checkpoint".to_string())
            })?;
            
            *last_checkpoint = position;
        }
        
        Ok(())
    }
    
    /// Find the last checkpoint and restore program state from it
    pub async fn restore_from_checkpoint(&self) -> Result<usize> {
        // Get the entry count
        let entry_count = self.storage.entry_count()?;
        
        // Read all entries (would be more efficient to have a reverse iteration)
        let entries = self.storage.read(0, entry_count)?;
        
        // Find the latest checkpoint
        let mut checkpoint_entry = None;
        let mut checkpoint_position = 0;
        
        for entry in entries.iter().rev() {
            if entry.entry_type == EntryType::Event {
                if let Some(true) = entry.metadata.get("checkpoint").map(|v| v == "true") {
                    checkpoint_entry = Some(entry.clone());
                    checkpoint_position = entry.metadata.get("position")
                        .and_then(|p| p.parse::<usize>().ok())
                        .unwrap_or(0);
                    break;
                }
            }
        }
        
        // If we found a checkpoint, restore from it
        if let Some(entry) = checkpoint_entry {
            if let EntryData::Event(event_data) = &entry.data {
                let mut state_handler = self.state_handler.lock().map_err(|_| {
                    Error::State("Failed to acquire lock on state handler".to_string())
                })?;
                
                // Convert BorshJsonValue to serde_json::Value for set_state
                let json_value = match &event_data.details {
                    causality_engine::log::types::BorshJsonValue(val) => val.clone(),
                };
                
                state_handler.set_state(json_value)?;
                
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

    /// Verify the integrity of a log entry (if supported by the entry)
    fn verify_entry_integrity(&self, entry: &LogEntry) -> Result<bool> {
        // Skip verification if disabled
        if !self.config.verify_integrity {
            return Ok(true);
        }
        
        // No direct integrity checking yet, as ID is now based on content hash
        // Future versions would recalculate the hash and compare
        Ok(true)
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
        // Check if we have a handler for this effect type
        if let Some(handler) = self.effect_handlers.get(effect_type) {
            // Use the registered handler
            handler(&mut self.state, data)?;
            Ok(())
        } else {
            // Default handling based on effect type
            match effect_type {
                // Transfer effect moves value from one account to another
                "transfer" => {
                    if let Some(from) = data.get("from").and_then(|v| v.as_str()) {
                        if let Some(to) = data.get("to").and_then(|v| v.as_str()) {
                            if let Some(amount) = data.get("amount").and_then(|v| v.as_u64()) {
                                // Update from account
                                if let Some(accounts) = self.state.get_mut("accounts") {
                                    if let Some(account) = accounts.get_mut(from) {
                                        if let Some(balance) = account.get_mut("balance") {
                                            if let Some(current) = balance.as_u64() {
                                                if current >= amount {
                                                    *balance = serde_json::json!(current - amount);
                                                    
                                                    // Update to account
                                                    if let Some(to_account) = accounts.get_mut(to) {
                                                        if let Some(to_balance) = to_account.get_mut("balance") {
                                                            if let Some(to_current) = to_balance.as_u64() {
                                                                *to_balance = serde_json::json!(to_current + amount);
                                                                return Ok(());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(Error::State(format!("Failed to apply transfer effect: {:?}", data)))
                },
                // Update effect sets a value
                "update" => {
                    if let Some(path) = data.get("path").and_then(|v| v.as_str()) {
                        if let Some(value) = data.get("value") {
                            self.set_value_at_path(path, value.clone())?;
                            return Ok(());
                        }
                    }
                    Err(Error::State(format!("Failed to apply update effect: {:?}", data)))
                },
                // Default case for unknown effects
                _ => {
                    // Just record the effect in the effects history
                    if let Some(history) = self.state.get_mut("effects") {
                        if let Some(effects_array) = history.as_array_mut() {
                            let effect_record = serde_json::json!({
                                "type": effect_type,
                                "data": data,
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            });
                            effects_array.push(effect_record);
                            return Ok(());
                        }
                    }
                    // If no history array, create one
                    let effects = vec![serde_json::json!({
                        "type": effect_type,
                        "data": data,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    })];
                    self.state["effects"] = serde_json::json!(effects);
                    Ok(())
                }
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

    /// Set a value at a specific path in the state
    fn set_value_at_path(&mut self, path: &str, value: serde_json::Value) -> Result<()> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &mut self.state;
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, set the value
                if let serde_json::Value::Object(obj) = current {
                    obj.insert(part.to_string(), value);
                    return Ok(());
                } else {
                    return Err(Error::State(format!("Cannot set value at path {}: parent is not an object", path)));
                }
            } else {
                // Navigate to the next part
                if let serde_json::Value::Object(obj) = current {
                    if !obj.contains_key(*part) {
                        obj.insert(part.to_string(), serde_json::json!({}));
                    }
                    
                    current = obj.get_mut(*part).unwrap();
                } else {
                    return Err(Error::State(format!("Cannot navigate path {}: part {} is not an object", path, part)));
                }
            }
        }
        
        Ok(())
    }
}

impl StateHandler for JsonStateHandler {
    fn handle_effect(&mut self, effect: &LogEntry) -> Result<()> {
        if let EntryData::Effect(effect_data) = &effect.data {
            // Extract the data parameter from the effect
            let params = match effect_data.parameters.get("data") {
                Some(data) => {
                    if let causality_engine::log::types::BorshJsonValue(value) = data {
                        value.clone()
                    } else {
                        serde_json::Value::Null
                    }
                },
                None => serde_json::Value::Null,
            };
            
            self.apply_effect(&effect_data.effect_type, &params)
        } else {
            Err(Error::InvalidOperation("Not an effect entry".to_string()))
        }
    }
    
    fn handle_fact(&mut self, fact: &LogEntry) -> Result<()> {
        if let EntryData::Fact(fact_data) = &fact.data {
            // Extract the data from BorshJsonValue
            let json_value = match &fact_data.data {
                causality_engine::log::types::BorshJsonValue(val) => val.clone(),
            };
            
            self.apply_fact(&fact_data.fact_type, &json_value)
        } else {
            Err(Error::InvalidOperation("Not a fact entry".to_string()))
        }
    }
    
    fn handle_event(&mut self, event: &LogEntry) -> Result<()> {
        if let EntryData::Event(event_data) = &event.data {
            // Extract the details from BorshJsonValue
            let json_value = match &event_data.details {
                causality_engine::log::types::BorshJsonValue(val) => val.clone(),
            };
            
            self.apply_event(&event_data.event_name, &json_value)
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