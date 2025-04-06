// Purpose: Provides a unified log format and replay capabilities for the simulation system
//
// This module implements a content-addressed, standardized log format for recording
// all agent actions, events, and states during simulation, enabling deterministic replay.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use sha2::Digest;

use crate::agent::AgentId;
use crate::scenario::Scenario;

/// Errors that can occur during log operations
#[derive(Error, Debug)]
pub enum LogError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Invalid log entry: {0}")]
    InvalidEntry(String),
    
    #[error("Hash verification failed")]
    HashVerificationFailed,
    
    #[error("Log segment not found: {0}")]
    SegmentNotFound(String),
    
    #[error("Invalid log format")]
    InvalidFormat,
}

/// Result type for log operations
pub type Result<T> = std::result::Result<T, LogError>;

/// Severity level for log entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Types of log entries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogEntryType {
    /// Agent action (e.g., transaction submission, message sending)
    AgentAction,
    /// Agent state change
    AgentState,
    /// Simulation event (e.g., agent start/stop, network partition)
    SimulationEvent,
    /// Domain event (e.g., block produced, transaction confirmed)
    DomainEvent,
    /// Fact observation
    FactObservation,
    /// System event
    SystemEvent,
    /// Custom entry type
    Custom(String),
}

/// A content-addressed log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier (hash of the entry content)
    pub id: String,
    /// Timestamp when the entry was created
    pub timestamp: DateTime<Utc>,
    /// Type of the log entry
    pub entry_type: LogEntryType,
    /// Source agent that created this entry
    pub agent_id: Option<AgentId>,
    /// Associated domain (if applicable)
    pub domain: Option<String>,
    /// Specific data payload
    pub payload: serde_json::Value,
    /// Optional reference to parent entry ID
    pub parent_id: Option<String>,
    /// Optional run identifier for the simulation
    pub run_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Content hash for verification
    pub content_hash: String,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(
        entry_type: LogEntryType,
        agent_id: Option<AgentId>,
        domain: Option<String>,
        payload: serde_json::Value,
        parent_id: Option<String>,
        run_id: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<Self> {
        let timestamp = Utc::now();
        
        // Create the entry without hash first
        let mut entry = Self {
            id: String::new(), // Will be set after hashing
            timestamp,
            entry_type,
            agent_id,
            domain,
            payload,
            parent_id,
            run_id,
            metadata,
            content_hash: String::new(), // Will be computed below
        };
        
        // Compute the content hash 
        let content_to_hash = serde_json::to_string(&entry)
            .map_err(|e| LogError::Serialization(e.to_string()))?;
        let hash = format!("{:x}", sha2::Sha256::digest(content_to_hash.as_bytes()));
        
        // Set the hash values
        entry.content_hash = hash.clone();
        entry.id = hash;
        
        Ok(entry)
    }
    
    /// Verify the content hash of this entry
    pub fn verify(&self) -> Result<bool> {
        // Create a copy without the hash for verification
        let mut verification_copy = self.clone();
        verification_copy.content_hash = String::new();
        
        // Compute the hash of the copy
        let content_to_hash = serde_json::to_string(&verification_copy)
            .map_err(|e| LogError::Serialization(e.to_string()))?;
        let computed_hash = format!("{:x}", sha2::Sha256::digest(content_to_hash.as_bytes()));
        
        Ok(computed_hash == self.content_hash)
    }
}

/// Storage for simulation logs
pub struct LogStorage {
    /// Base directory for storing logs
    base_dir: PathBuf,
    /// Current run identifier
    run_id: String,
    /// In-memory entries for the current session (for quick access)
    entries: Arc<Mutex<Vec<LogEntry>>>,
}

impl LogStorage {
    /// Create a new log storage
    pub fn new(base_dir: impl AsRef<Path>, run_id: Option<String>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        // Create the base directory if it doesn't exist
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)?;
        }
        
        // Generate a run ID if not provided
        let run_id = match run_id {
            Some(id) => id,
            None => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                format!("run-{}", now)
            }
        };
        
        // Create run directory
        let run_dir = base_dir.join(&run_id);
        if !run_dir.exists() {
            fs::create_dir_all(&run_dir)?;
        }
        
        Ok(Self {
            base_dir,
            run_id,
            entries: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// Get the run directory
    pub fn run_dir(&self) -> PathBuf {
        self.base_dir.join(&self.run_id)
    }
    
    /// Get the run ID
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
    
    /// Append an entry to the log
    pub fn append(&self, entry: LogEntry) -> Result<()> {
        // Verify the entry hash before appending
        if !entry.verify()? {
            return Err(LogError::HashVerificationFailed);
        }
        
        // Append to in-memory storage
        {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry.clone());
        }
        
        // Append to disk storage
        let file_path = self.run_dir().join(format!("{}.json", entry.id));
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)?;
        
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &entry)
            .map_err(|e| LogError::Serialization(e.to_string()))?;
        
        // Update the index
        self.update_index()?;
        
        Ok(())
    }
    
    /// Get all entries
    pub fn get_all_entries(&self) -> Result<Vec<LogEntry>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries.clone())
    }
    
    /// Read entries from disk for a specific run
    pub fn read_run(&self, run_id: &str) -> Result<Vec<LogEntry>> {
        let run_dir = self.base_dir.join(run_id);
        if !run_dir.exists() {
            return Err(LogError::SegmentNotFound(run_id.to_string()));
        }
        
        let index_path = run_dir.join("index.json");
        if !index_path.exists() {
            return Err(LogError::InvalidFormat);
        }
        
        // Read the index file
        let file = File::open(index_path)?;
        let reader = BufReader::new(file);
        
        let index: Vec<String> = serde_json::from_reader(reader)
            .map_err(|e| LogError::Deserialization(e.to_string()))?;
        
        let mut entries = Vec::new();
        
        // Read each entry
        for entry_id in index {
            let entry_path = run_dir.join(format!("{}.json", entry_id));
            let file = File::open(entry_path)?;
            let reader = BufReader::new(file);
            
            let entry: LogEntry = serde_json::from_reader(reader)
                .map_err(|e| LogError::Deserialization(e.to_string()))?;
            
            // Verify the entry hash
            if !entry.verify()? {
                return Err(LogError::HashVerificationFailed);
            }
            
            entries.push(entry);
        }
        
        // Sort entries by timestamp
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        Ok(entries)
    }
    
    /// Find entries by type
    pub fn find_by_type(&self, entry_type: LogEntryType) -> Result<Vec<LogEntry>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries
            .iter()
            .filter(|e| e.entry_type == entry_type)
            .cloned()
            .collect())
    }
    
    /// Find entries by agent ID
    pub fn find_by_agent(&self, agent_id: &AgentId) -> Result<Vec<LogEntry>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries
            .iter()
            .filter(|e| e.agent_id.as_ref() == Some(agent_id))
            .cloned()
            .collect())
    }
    
    /// Find entries by parent ID
    pub fn find_by_parent(&self, parent_id: &str) -> Result<Vec<LogEntry>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries
            .iter()
            .filter(|e| e.parent_id.as_ref() == Some(&parent_id.to_string()))
            .cloned()
            .collect())
    }
    
    /// Update the index file
    fn update_index(&self) -> Result<()> {
        let entries = self.entries.lock().unwrap();
        let entry_ids: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
        
        let index_path = self.run_dir().join("index.json");
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(index_path)?;
            
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &entry_ids)
            .map_err(|e| LogError::Serialization(e.to_string()))?;
            
        Ok(())
    }
}

/// Manager for replaying simulation logs
pub struct ReplayManager {
    /// Storage for accessing logs
    storage: LogStorage,
    /// Scenario that was originally executed
    scenario: Scenario,
    /// Current position in the replay
    current_position: usize,
    /// Total entries to replay
    total_entries: usize,
    /// Entries loaded for replay
    entries: Vec<LogEntry>,
}

impl ReplayManager {
    /// Create a new replay manager
    pub fn new(log_dir: impl AsRef<Path>, run_id: &str, scenario: Scenario) -> Result<Self> {
        let storage = LogStorage::new(log_dir, Some(run_id.to_string()))?;
        let entries = storage.read_run(run_id)?;
        let total_entries = entries.len();
        
        Ok(Self {
            storage,
            scenario,
            current_position: 0,
            total_entries,
            entries,
        })
    }
    
    /// Get the total number of entries
    pub fn total_entries(&self) -> usize {
        self.total_entries
    }
    
    /// Get the current position
    pub fn current_position(&self) -> usize {
        self.current_position
    }
    
    /// Get the original scenario
    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }
    
    /// Start replay from the beginning
    pub fn start(&mut self) -> Result<()> {
        self.current_position = 0;
        Ok(())
    }
    
    /// Get the next entry in the replay
    pub fn next(&mut self) -> Option<&LogEntry> {
        if self.current_position >= self.total_entries {
            return None;
        }
        
        let entry = &self.entries[self.current_position];
        self.current_position += 1;
        
        Some(entry)
    }
    
    /// Reset the replay to the beginning
    pub fn reset(&mut self) {
        self.current_position = 0;
    }
    
    /// Skip to a specific position
    pub fn skip_to(&mut self, position: usize) -> Result<()> {
        if position >= self.total_entries {
            return Err(LogError::InvalidEntry(format!(
                "Position {} exceeds total entries {}",
                position, self.total_entries
            )));
        }
        
        self.current_position = position;
        Ok(())
    }
    
    /// Get entries for a specific agent
    pub fn get_agent_entries(&self, agent_id: &AgentId) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.agent_id.as_ref() == Some(agent_id))
            .collect()
    }
    
    /// Get entries of a specific type
    pub fn get_entries_by_type(&self, entry_type: LogEntryType) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.entry_type == entry_type)
            .collect()
    }
    
    /// Verify all entries in the replay
    pub fn verify_all_entries(&self) -> Result<bool> {
        for entry in &self.entries {
            if !entry.verify()? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

/// Helper functions for creating log entries
pub mod log_helpers {
    use super::*;
    
    /// Create an agent action log entry
    pub fn create_agent_action(
        agent_id: AgentId,
        action_type: &str,
        payload: serde_json::Value,
        run_id: Option<String>,
    ) -> Result<LogEntry> {
        let mut metadata = HashMap::new();
        metadata.insert("action_type".to_string(), action_type.to_string());
        
        LogEntry::new(
            LogEntryType::AgentAction,
            Some(agent_id),
            None,
            payload,
            None,
            run_id,
            metadata,
        )
    }
    
    /// Create an agent state log entry
    pub fn create_agent_state(
        agent_id: AgentId,
        state: serde_json::Value,
        run_id: Option<String>,
    ) -> Result<LogEntry> {
        let mut metadata = HashMap::new();
        metadata.insert("state_type".to_string(), "agent_state".to_string());
        
        LogEntry::new(
            LogEntryType::AgentState,
            Some(agent_id),
            None,
            state,
            None,
            run_id,
            metadata,
        )
    }
    
    /// Create a simulation event log entry
    pub fn create_simulation_event(
        event_type: &str,
        payload: serde_json::Value,
        run_id: Option<String>,
    ) -> Result<LogEntry> {
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), event_type.to_string());
        
        LogEntry::new(
            LogEntryType::SimulationEvent,
            None,
            None,
            payload,
            None,
            run_id,
            metadata,
        )
    }
    
    /// Create a domain event log entry
    pub fn create_domain_event(
        domain: &str,
        event_type: &str,
        payload: serde_json::Value,
        run_id: Option<String>,
    ) -> Result<LogEntry> {
        let mut metadata = HashMap::new();
        metadata.insert("event_type".to_string(), event_type.to_string());
        
        LogEntry::new(
            LogEntryType::DomainEvent,
            None,
            Some(domain.to_string()),
            payload,
            None,
            run_id,
            metadata,
        )
    }
    
    /// Create a fact observation log entry
    pub fn create_fact_observation(
        agent_id: Option<AgentId>,
        domain: &str,
        fact_type: &str,
        fact_data: serde_json::Value,
        run_id: Option<String>,
    ) -> Result<LogEntry> {
        let mut metadata = HashMap::new();
        metadata.insert("fact_type".to_string(), fact_type.to_string());
        
        LogEntry::new(
            LogEntryType::FactObservation,
            agent_id,
            Some(domain.to_string()),
            fact_data,
            None,
            run_id,
            metadata,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_log_entry_creation_and_verification() {
        let mut metadata = HashMap::new();
        metadata.insert("test".to_string(), "value".to_string());
        
        let entry = LogEntry::new(
            LogEntryType::AgentAction,
            Some(AgentId::new("agent1")),
            Some("ethereum".to_string()),
            serde_json::json!({"action": "send_transaction"}),
            None,
            Some("test-run".to_string()),
            metadata,
        )
        .unwrap();
        
        assert!(entry.verify().unwrap());
        
        // Verify content hash matches ID
        assert_eq!(entry.id, entry.content_hash);
        
        // Modify the entry and verify hash fails
        let mut modified = entry.clone();
        modified.payload = serde_json::json!({"action": "different_action"});
        assert!(!modified.verify().unwrap());
    }
    
    #[test]
    fn test_log_storage() {
        let temp_dir = tempdir().unwrap();
        let storage = LogStorage::new(temp_dir.path(), Some("test-run".to_string())).unwrap();
        
        // Create and append an entry
        let entry = log_helpers::create_agent_action(
            AgentId::new("agent1"),
            "send_transaction",
            serde_json::json!({"tx_hash": "0x123"}),
            Some("test-run".to_string()),
        )
        .unwrap();
        
        storage.append(entry).unwrap();
        
        // Retrieve all entries
        let entries = storage.get_all_entries().unwrap();
        assert_eq!(entries.len(), 1);
        
        // Verify entry was stored correctly
        assert_eq!(entries[0].agent_id, Some(AgentId::new("agent1")));
        assert_eq!(entries[0].entry_type, LogEntryType::AgentAction);
        
        // Test reading from disk
        let read_entries = storage.read_run("test-run").unwrap();
        assert_eq!(read_entries.len(), 1);
        assert_eq!(read_entries[0].id, entries[0].id);
    }
    
    #[test]
    fn test_replay_manager() {
        let temp_dir = tempdir().unwrap();
        let storage = LogStorage::new(temp_dir.path(), Some("test-run".to_string())).unwrap();
        
        // Create test entries
        for i in 0..5 {
            let entry = log_helpers::create_agent_action(
                AgentId::new(format!("agent{}", i % 2 + 1)),
                "action",
                serde_json::json!({"index": i}),
                Some("test-run".to_string()),
            )
            .unwrap();
            
            storage.append(entry).unwrap();
        }
        
        // Create a test scenario
        let scenario = Scenario {
            name: "Test Scenario".to_string(),
            agents: Vec::new(),
            domains: Vec::new(),
            initial_facts: Vec::new(),
            invariants: Vec::new(),
        };
        
        // Create replay manager
        let mut replay = ReplayManager::new(temp_dir.path(), "test-run", scenario).unwrap();
        
        // Verify total entries
        assert_eq!(replay.total_entries(), 5);
        
        // Test iteration
        let mut count = 0;
        while let Some(_) = replay.next() {
            count += 1;
        }
        assert_eq!(count, 5);
        
        // Test reset
        replay.reset();
        assert_eq!(replay.current_position(), 0);
        
        // Test filtering by agent
        let agent1_entries = replay.get_agent_entries(&AgentId::new("agent1"));
        let agent2_entries = replay.get_agent_entries(&AgentId::new("agent2"));
        
        assert_eq!(agent1_entries.len(), 3); // Entries for agent1 (0, 2, 4)
        assert_eq!(agent2_entries.len(), 2); // Entries for agent2 (1, 3)
        
        // Verify all entries
        assert!(replay.verify_all_entries().unwrap());
    }
}
