// Purpose: Defines the Observer trait for subscribing to log events from running scenarios
//
// This module provides functionality for observing log entries in real-time during simulation runs

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

// Use the LogEntry and related types directly from replay.rs
use crate::replay::{LogEntry, LogEntryType};
use crate::agent::AgentId;

/// Filter to select specific log entries
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    /// Filter by entry types
    pub entry_types: Option<Vec<LogEntryType>>,
    /// Filter by agent IDs
    pub agent_ids: Option<Vec<AgentId>>,
    /// Filter by domains
    pub domains: Option<Vec<String>>,
    /// Filter by regex pattern on payload
    pub payload_pattern: Option<String>,
    /// Filter by metadata keys and values
    pub metadata: Option<HashMap<String, String>>,
}

impl LogFilter {
    /// Create a new empty filter (matches all entries)
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add entry type filter
    pub fn with_entry_type(mut self, entry_type: LogEntryType) -> Self {
        let entry_types = self.entry_types.get_or_insert_with(Vec::new);
        entry_types.push(entry_type);
        self
    }
    
    /// Add agent ID filter
    pub fn with_agent(mut self, agent_id: AgentId) -> Self {
        let agent_ids = self.agent_ids.get_or_insert_with(Vec::new);
        agent_ids.push(agent_id);
        self
    }
    
    /// Add domain filter
    pub fn with_domain(mut self, domain: String) -> Self {
        let domains = self.domains.get_or_insert_with(Vec::new);
        domains.push(domain);
        self
    }
    
    /// Add payload pattern filter
    pub fn with_payload_pattern(mut self, pattern: String) -> Self {
        self.payload_pattern = Some(pattern);
        self
    }
    
    /// Add metadata filter
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key, value);
        self
    }
    
    /// Check if an entry matches this filter
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Check entry type filter
        if let Some(entry_types) = &self.entry_types {
            if !entry_types.iter().any(|t| *t == entry.entry_type) {
                return false;
            }
        }
        
        // Check agent ID filter
        if let Some(agent_ids) = &self.agent_ids {
            if let Some(entry_agent_id) = &entry.agent_id {
                if !agent_ids.iter().any(|a| a == entry_agent_id) {
                    return false;
                }
            } else {
                // Entry has no agent ID but filter requires one
                return false;
            }
        }
        
        // Check domain filter
        if let Some(domains) = &self.domains {
            if let Some(entry_domain) = &entry.domain {
                if !domains.iter().any(|d| d == entry_domain) {
                    return false;
                }
            } else {
                // Entry has no domain but filter requires one
                return false;
            }
        }
        
        // Check payload pattern filter
        if let Some(pattern) = &self.payload_pattern {
            let payload_str = serde_json::to_string(&entry.payload).unwrap_or_default();
            if !payload_str.contains(pattern) {
                return false;
            }
        }
        
        // Check metadata filter
        if let Some(metadata) = &self.metadata {
            for (key, value) in metadata {
                if entry.metadata.get(key) != Some(value) {
                    return false;
                }
            }
        }
        
        // All filters passed
        true
    }
}

/// Trait for observing simulation log events
pub trait Observer: Send + Sync {
    /// Called when a new log entry is produced
    fn on_log_entry(&self, entry: LogEntry);
    
    /// Called when the simulation starts
    fn on_simulation_start(&self, run_id: &str);
    
    /// Called when the simulation ends
    fn on_simulation_end(&self, run_id: &str);
}

/// Callback type for handling log entries
pub type LogEntryCallback = Box<dyn Fn(LogEntry) + Send + Sync>;

/// An observer that forwards log entries to a callback function
pub struct CallbackObserver {
    /// Observer name for identification
    name: String,
    /// The callback to invoke for each log entry
    callback: LogEntryCallback,
    /// Optional filter to select entries
    filter: Option<LogFilter>,
}

impl CallbackObserver {
    /// Create a new callback observer
    pub fn new(
        name: impl Into<String>, 
        filter: Option<LogFilter>,
        callback: LogEntryCallback
    ) -> Self {
        Self {
            name: name.into(),
            callback,
            filter,
        }
    }
    
    /// Set a filter for this observer
    pub fn with_filter(mut self, filter: LogFilter) -> Self {
        self.filter = Some(filter);
        self
    }
    
    /// Get the name of this observer
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Observer for CallbackObserver {
    fn on_log_entry(&self, entry: LogEntry) {
        // Check if the entry matches our filter
        if let Some(filter) = &self.filter {
            if !filter.matches(&entry) {
                return;
            }
        }
        
        // Call the callback
        (self.callback)(entry);
    }
    
    fn on_simulation_start(&self, _run_id: &str) {
        // No default behavior
    }
    
    fn on_simulation_end(&self, _run_id: &str) {
        // No default behavior
    }
}

/// An observer that stores log entries in memory
pub struct MemoryObserver {
    /// Stored log entries
    entries: Arc<Mutex<Vec<LogEntry>>>,
    /// Optional filter to select entries
    filter: Option<LogFilter>,
}

impl MemoryObserver {
    /// Create a new memory observer
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            filter: None,
        }
    }
    
    /// Set a filter for this observer
    pub fn with_filter(mut self, filter: LogFilter) -> Self {
        self.filter = Some(filter);
        self
    }
    
    /// Get all stored entries
    pub fn get_entries(&self) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.clone()
    }
    
    /// Clear all stored entries
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }
}

impl Observer for MemoryObserver {
    fn on_log_entry(&self, entry: LogEntry) {
        // Check if the entry matches our filter
        if let Some(filter) = &self.filter {
            if !filter.matches(&entry) {
                return;
            }
        }
        
        // Store the entry
        let mut entries = self.entries.lock().unwrap();
        entries.push(entry);
    }
    
    fn on_simulation_start(&self, _run_id: &str) {
        self.clear();
    }
    
    fn on_simulation_end(&self, _run_id: &str) {
        // No default behavior
    }
}

/// An observer that sends log entries to a channel
pub struct ChannelObserver {
    /// Sender for the channel
    sender: UnboundedSender<LogEntry>,
    /// Optional filter to select entries
    filter: Option<LogFilter>,
}

impl ChannelObserver {
    /// Create a new channel observer
    pub fn new() -> (Self, UnboundedReceiver<LogEntry>) {
        let (sender, receiver) = unbounded_channel::<LogEntry>();
        
        let observer = Self {
            sender,
            filter: None,
        };
        
        (observer, receiver)
    }
    
    /// Set a filter for this observer
    pub fn with_filter(mut self, filter: LogFilter) -> Self {
        self.filter = Some(filter);
        self
    }
}

impl Observer for ChannelObserver {
    fn on_log_entry(&self, entry: LogEntry) {
        // Check if the entry matches our filter
        if let Some(filter) = &self.filter {
            if !filter.matches(&entry) {
                return;
            }
        }
        
        // Send the entry to the channel, ignoring errors if receiver is dropped
        let _ = self.sender.send(entry);
    }
    
    fn on_simulation_start(&self, run_id: &str) {
        // Create a special start event
        let payload = serde_json::json!({
            "event": "simulation_start",
            "run_id": run_id,
        });
        
        if let Ok(entry) = crate::replay::log_helpers::create_simulation_event(
            "start",
            payload,
            Some(run_id.to_string()),
        ) {
            let _ = self.sender.send(entry);
        }
    }
    
    fn on_simulation_end(&self, run_id: &str) {
        // Create a special end event
        let payload = serde_json::json!({
            "event": "simulation_end",
            "run_id": run_id,
        });
        
        if let Ok(entry) = crate::replay::log_helpers::create_simulation_event(
            "end",
            payload,
            Some(run_id.to_string()),
        ) {
            let _ = self.sender.send(entry);
        }
    }
}

/// Registry for managing multiple observers
pub struct ObserverRegistry {
    /// List of registered observers
    observers: Arc<Mutex<Vec<Arc<dyn Observer>>>>,
}

impl ObserverRegistry {
    /// Create a new observer registry
    pub fn new() -> Self {
        Self {
            observers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Register a new observer
    pub fn register(&self, observer: Arc<dyn Observer>) {
        let mut observers = self.observers.lock().unwrap();
        observers.push(observer);
    }
    
    /// Notify all observers about a log entry
    pub fn notify_log_entry(&self, entry: LogEntry) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.on_log_entry(entry.clone());
        }
    }
    
    /// Notify all observers about simulation start
    pub fn notify_simulation_start(&self, run_id: &str) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.on_simulation_start(run_id);
        }
    }
    
    /// Notify all observers about simulation end
    pub fn notify_simulation_end(&self, run_id: &str) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.on_simulation_end(run_id);
        }
    }
    
    /// Get the number of registered observers
    pub fn observer_count(&self) -> usize {
        let observers = self.observers.lock().unwrap();
        observers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    // Helper function to create a test log entry
    fn create_test_entry(
        entry_type: LogEntryType,
        agent_id: Option<&str>,
        domain: Option<&str>,
        payload: serde_json::Value,
    ) -> LogEntry {
        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("test".to_string(), "value".to_string());
        
        let agent_id = agent_id.map(|id| AgentId::new(id));
        let domain = domain.map(|d| d.to_string());
        let run_id = Some("test-run".to_string());
        
        crate::replay::LogEntry::new(
            entry_type,
            agent_id,
            domain,
            payload,
            None, // parent_id
            run_id,
            metadata,
        ).unwrap()
    }
    
    #[test]
    fn test_filter_matching() {
        // Create a test entry
        let entry = create_test_entry(
            LogEntryType::AgentAction,
            Some("agent1"),
            Some("domain1"),
            serde_json::json!({"action": "test_action"}),
        );
        
        // Test entry type filter
        let filter = LogFilter::new().with_entry_type(LogEntryType::AgentAction);
        assert!(filter.matches(&entry));
        
        let filter = LogFilter::new().with_entry_type(LogEntryType::AgentState);
        assert!(!filter.matches(&entry));
        
        // Test agent ID filter
        let filter = LogFilter::new().with_agent(AgentId::new("agent1"));
        assert!(filter.matches(&entry));
        
        let filter = LogFilter::new().with_agent(AgentId::new("agent2"));
        assert!(!filter.matches(&entry));
        
        // Test domain filter
        let filter = LogFilter::new().with_domain("domain1".to_string());
        assert!(filter.matches(&entry));
        
        let filter = LogFilter::new().with_domain("domain2".to_string());
        assert!(!filter.matches(&entry));
        
        // Test payload pattern filter
        let filter = LogFilter::new().with_payload_pattern("test_action".to_string());
        assert!(filter.matches(&entry));
        
        let filter = LogFilter::new().with_payload_pattern("other_action".to_string());
        assert!(!filter.matches(&entry));
        
        // Test metadata filter
        let filter = LogFilter::new().with_metadata("test".to_string(), "value".to_string());
        assert!(filter.matches(&entry));
        
        let filter = LogFilter::new().with_metadata("test".to_string(), "wrong".to_string());
        assert!(!filter.matches(&entry));
        
        // Test combined filters
        let filter = LogFilter::new()
            .with_entry_type(LogEntryType::AgentAction)
            .with_agent(AgentId::new("agent1"))
            .with_domain("domain1".to_string());
        assert!(filter.matches(&entry));
        
        let filter = LogFilter::new()
            .with_entry_type(LogEntryType::AgentAction)
            .with_agent(AgentId::new("agent2"));
        assert!(!filter.matches(&entry));
    }
    
    #[test]
    fn test_memory_observer() {
        // Create a test entry
        let entry = create_test_entry(
            LogEntryType::AgentAction,
            Some("agent1"),
            Some("domain1"),
            serde_json::json!({"action": "test_action"}),
        );
        
        // Create a memory observer
        let observer = MemoryObserver::new();
        
        // Notify the observer
        observer.on_log_entry(entry.clone());
        
        // Check that the entry was stored
        let entries = observer.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, entry.id);
        
        // Test filtering
        let filtered_observer = MemoryObserver::new()
            .with_filter(LogFilter::new().with_agent(AgentId::new("agent2")));
        
        // This should be filtered out
        filtered_observer.on_log_entry(entry.clone());
        
        let entries = filtered_observer.get_entries();
        assert_eq!(entries.len(), 0);
        
        // Test clearing
        observer.clear();
        assert_eq!(observer.get_entries().len(), 0);
    }
    
    #[test]
    fn test_callback_observer() {
        // Create a counter
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // Create a callback observer
        let callback = Box::new(move |_entry: LogEntry| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        let observer = CallbackObserver::new("test", None, callback);
        
        // Create a test entry
        let entry = create_test_entry(
            LogEntryType::AgentAction,
            Some("agent1"),
            Some("domain1"),
            serde_json::json!({"action": "test_action"}),
        );
        
        // Notify the observer multiple times
        for _ in 0..3 {
            observer.on_log_entry(entry.clone());
        }
        
        // Check the counter
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
    
    #[test]
    fn test_channel_observer() {
        // Create a channel observer
        let (observer, mut receiver) = ChannelObserver::new();
        
        // Create a test entry
        let entry = create_test_entry(
            LogEntryType::AgentAction,
            Some("agent1"),
            Some("domain1"),
            serde_json::json!({"action": "test_action"}),
        );
        
        // Notify the observer
        observer.on_log_entry(entry.clone());
        
        // Receive the entry from the channel
        let received = receiver.try_recv().unwrap();
        assert_eq!(received.id, entry.id);
        
        // Test filtering
        let (filtered_observer, mut filtered_receiver) = ChannelObserver::new();
        filtered_observer.with_filter(LogFilter::new().with_agent(AgentId::new("agent2")));
        
        // This should be filtered out
        filtered_observer.on_log_entry(entry.clone());
        
        // The channel should be empty
        assert!(filtered_receiver.try_recv().is_err());
    }
    
    #[test]
    fn test_observer_registry() {
        // Create a registry
        let registry = ObserverRegistry::new();
        
        // Create memory observers
        let observer1 = Arc::new(MemoryObserver::new());
        let observer2 = Arc::new(MemoryObserver::new());
        
        // Register the observers
        registry.register(observer1.clone());
        registry.register(observer2.clone());
        
        // Check observer count
        assert_eq!(registry.observer_count(), 2);
        
        // Create a test entry
        let entry = create_test_entry(
            LogEntryType::AgentAction,
            Some("agent1"),
            Some("domain1"),
            serde_json::json!({"action": "test_action"}),
        );
        
        // Notify all observers
        registry.notify_log_entry(entry.clone());
        
        // Check that both observers received the entry
        assert_eq!(observer1.get_entries().len(), 1);
        assert_eq!(observer2.get_entries().len(), 1);
        
        // Notify about simulation events
        registry.notify_simulation_start("test-run");
        registry.notify_simulation_end("test-run");
    }
}
