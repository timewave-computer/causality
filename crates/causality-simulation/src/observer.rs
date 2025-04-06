// Purpose: Defines the Observer trait for subscribing to log events from running scenarios
//
// This module provides functionality for observing log entries in real-time during simulation runs

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use rand;

// Use the LogEntry and related types directly from replay.rs
use crate::replay::{LogEntry, LogEntryType};
use crate::agent::agent_id;
use causality_core::resource::ResourceId;

/// Filter to select specific log entries
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    /// Filter by entry types
    pub entry_types: Option<Vec<LogEntryType>>,
    /// Filter by agent IDs
    pub agent_ids: Option<Vec<ResourceId>>,
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
    pub fn with_agent(mut self, agent_id: ResourceId) -> Self {
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
    
    /// Merge another filter with this one
    pub fn merge(mut self, other: LogFilter) -> Self {
        // Merge entry types
        if let Some(other_entry_types) = other.entry_types {
            let entry_types = self.entry_types.get_or_insert_with(Vec::new);
            entry_types.extend(other_entry_types);
        }

        // Merge agent IDs
        if let Some(other_agent_ids) = other.agent_ids {
            let agent_ids = self.agent_ids.get_or_insert_with(Vec::new);
            agent_ids.extend(other_agent_ids);
        }

        // Merge domains
        if let Some(other_domains) = other.domains {
            let domains = self.domains.get_or_insert_with(Vec::new);
            domains.extend(other_domains);
        }

        // Use the last payload pattern (if any)
        if other.payload_pattern.is_some() {
            self.payload_pattern = other.payload_pattern;
        }

        // Merge metadata
        if let Some(other_metadata) = other.metadata {
            let metadata = self.metadata.get_or_insert_with(HashMap::new);
            metadata.extend(other_metadata);
        }

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

/// Trait for objects that observe the log
pub trait Observer: Send + Sync + std::fmt::Debug {
    /// Handle a log entry
    fn on_log_entry(&self, entry: LogEntry);

    /// Handle simulation start
    fn on_simulation_start(&self, run_id: &str) {
        // Default implementation does nothing
    }

    /// Handle simulation end
    fn on_simulation_end(&self, run_id: &str) {
        // Default implementation does nothing
    }
    
    /// Apply a filter to this observer
    fn apply_filter(&mut self, filter: LogFilter);
    
    /// Get the name of this observer
    fn name(&self) -> &str {
        // Default implementation returns empty string
        ""
    }
}

/// Callback type for handling log entries
#[allow(missing_debug_implementations)]
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

impl std::fmt::Debug for CallbackObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackObserver")
            .field("name", &self.name)
            .field("callback", &format_args!("<function>"))
            .field("filter", &self.filter)
            .finish()
    }
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
    
    fn apply_filter(&mut self, filter: LogFilter) {
        self.filter = Some(filter);
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// An observer that stores log entries in memory
#[derive(Debug)]
pub struct MemoryObserver {
    /// Observer name for identification
    name: String,
    /// Stored log entries
    entries: Arc<Mutex<Vec<LogEntry>>>,
    /// Optional filter to select entries
    filter: Option<LogFilter>,
}

impl MemoryObserver {
    /// Create a new memory observer
    pub fn new() -> Self {
        Self {
            name: "MemoryObserver".to_string(),
            entries: Arc::new(Mutex::new(Vec::new())),
            filter: None,
        }
    }
    
    /// Create a new memory observer with a specific name
    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: Arc::new(Mutex::new(Vec::new())),
            filter: None,
        }
    }
    
    /// Set a filter for this observer
    pub fn with_filter(&mut self, filter: LogFilter) -> &mut Self {
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
    
    /// Get the name of this observer
    pub fn name(&self) -> &str {
        &self.name
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
    
    fn apply_filter(&mut self, filter: LogFilter) {
        self.filter = Some(filter);
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Observer that pushes log entries to a channel
#[derive(Debug, Clone)]
pub struct ChannelObserver {
    /// Observer name
    name: String,
    /// The sender for log entries
    tx: UnboundedSender<LogEntry>,
    /// Filter to select specific entries
    filter: Option<LogFilter>,
}

impl ChannelObserver {
    /// Create a new channel observer
    pub fn new() -> (Self, UnboundedReceiver<LogEntry>) {
        let (sender, receiver) = unbounded_channel();
        
        let observer = Self {
            name: "ChannelObserver".to_string(),
            tx: sender,
            filter: None,
        };
        
        (observer, receiver)
    }
    
    /// Set a filter for this observer
    pub fn with_filter(&mut self, filter: LogFilter) -> &mut Self {
        self.filter = Some(filter);
        self
    }
    
    /// Get the name of this observer
    pub fn name(&self) -> &str {
        &self.name
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
        
        // Send the entry down the channel
        if let Err(e) = self.tx.send(entry) {
            eprintln!("Failed to send log entry: {}", e);
        }
    }
    
    fn on_simulation_start(&self, run_id: &str) {
        // Send a start notification as a special entry
        if let Ok(entry) = crate::replay::log_helpers::create_simulation_event(
            "simulation_start",
            serde_json::json!({"run_id": run_id}),
            Some(run_id.to_string()),
        ) {
            if let Err(e) = self.tx.send(entry) {
                eprintln!("Failed to send simulation start entry: {}", e);
            }
        }
    }
    
    fn on_simulation_end(&self, run_id: &str) {
        // Send an end notification as a special entry
        if let Ok(entry) = crate::replay::log_helpers::create_simulation_event(
            "simulation_end",
            serde_json::json!({"run_id": run_id}),
            Some(run_id.to_string()),
        ) {
            if let Err(e) = self.tx.send(entry) {
                eprintln!("Failed to send simulation end entry: {}", e);
            }
        }
    }
    
    fn apply_filter(&mut self, filter: LogFilter) {
        self.filter = Some(filter);
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Registry for simulation observers that maintains a list of active observers.
#[derive(Debug, Clone)]
pub struct ObserverRegistry {
    /// Observers registered for simulation events.
    observers: Arc<RwLock<HashMap<String, Arc<dyn Observer>>>>,
    /// Filter to apply to log entries before dispatching to observers.
    filter: Arc<RwLock<Option<LogFilter>>>,
}

impl ObserverRegistry {
    /// Create a new observer registry
    pub fn new() -> Self {
        Self {
            observers: Arc::new(RwLock::new(HashMap::new())),
            filter: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Register a new observer
    pub fn register(&self, observer: Arc<dyn Observer>) {
        let mut observers = self.observers.write().unwrap();
        observers.insert(observer.name().to_string(), observer);
    }
    
    /// Notify all observers about a log entry
    pub fn notify_log_entry(&self, entry: LogEntry) {
        let observers = self.observers.read().unwrap();
        for observer in observers.values() {
            observer.on_log_entry(entry.clone());
        }
    }
    
    /// Notify all observers about simulation start
    pub fn notify_simulation_start(&self, run_id: &str) {
        let observers = self.observers.read().unwrap();
        for observer in observers.values() {
            observer.on_simulation_start(run_id);
        }
    }
    
    /// Notify all observers about simulation end
    pub fn notify_simulation_end(&self, run_id: &str) {
        let observers = self.observers.read().unwrap();
        for observer in observers.values() {
            observer.on_simulation_end(run_id);
        }
    }
    
    /// Get the number of registered observers
    pub fn observer_count(&self) -> usize {
        let observers = self.observers.read().unwrap();
        observers.len()
    }
}

impl Observer for ObserverRegistry {
    fn on_log_entry(&self, entry: LogEntry) {
        self.notify_log_entry(entry);
    }
    
    fn on_simulation_start(&self, run_id: &str) {
        self.notify_simulation_start(run_id);
    }
    
    fn on_simulation_end(&self, run_id: &str) {
        self.notify_simulation_end(run_id);
    }
    
    fn apply_filter(&mut self, new_filter: LogFilter) {
        let mut observers = self.observers.write().unwrap();
        let mut current_filter = self.filter.write().unwrap();
        *current_filter = Some(current_filter.take().unwrap_or_default().merge(new_filter));
    }
    
    fn name(&self) -> &str {
        "ObserverRegistry"
    }
}

/// MultiObserver that combines multiple observers
#[derive(Debug)]
pub struct MultiObserver {
    /// Observer name
    name: String,
    /// List of observers
    observers: Vec<Box<dyn Observer>>,
}

impl MultiObserver {
    /// Create a new multi observer
    pub fn new() -> Self {
        Self {
            name: "MultiObserver".to_string(),
            observers: Vec::new(),
        }
    }

    /// Add an observer to the multi observer
    pub fn add_observer<T: Observer + 'static>(&mut self, observer: T) -> &mut Self {
        self.observers.push(Box::new(observer));
        self
    }

    /// Set a filter for all observers
    pub fn with_filter(&mut self, filter: LogFilter) -> &mut Self {
        for observer in &mut self.observers {
            observer.apply_filter(filter.clone());
        }
        self
    }
}

impl Observer for MultiObserver {
    fn on_log_entry(&self, entry: LogEntry) {
        for observer in &self.observers {
            observer.on_log_entry(entry.clone());
        }
    }

    fn apply_filter(&mut self, filter: LogFilter) {
        for observer in &mut self.observers {
            observer.apply_filter(filter.clone());
        }
    }

    fn on_simulation_start(&self, run_id: &str) {
        for observer in &self.observers {
            observer.on_simulation_start(run_id);
        }
    }

    fn on_simulation_end(&self, run_id: &str) {
        for observer in &self.observers {
            observer.on_simulation_end(run_id);
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    // Helper function to create a test log entry
    fn create_test_entry(
        entry_type: LogEntryType,
        agent_id_str: Option<&str>,
        domain: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> LogEntry {
        let agent_id = agent_id_str.map(|id| agent_id::from_string(id));
        let payload = serde_json::json!({});
        let run_id = Some("test-run".to_string());
        
        // Create a basic entry directly with the specified entry_type instead of using helpers
        LogEntry {
            id: format!("test-id-{}", uuid::Uuid::new_v4()),
            timestamp: chrono::Utc::now(),
            entry_type,
            agent_id,
            domain: domain.map(|d| d.to_string()),
            payload,
            parent_id: None,
            run_id,
            metadata,
            content_hash: "test-hash".to_string(),
        }
    }
    
    #[test]
    fn test_log_filter() {
        // Test empty filter (matches all)
        let empty_filter = LogFilter::new();
        let entry = create_test_entry(
            LogEntryType::AgentAction,
            Some("agent1"),
            Some("domain1"),
            HashMap::new(),
        );
        assert!(empty_filter.matches(&entry));
        
        // Test entry type filter
        let type_filter = LogFilter::new().with_entry_type(LogEntryType::AgentAction);
        assert!(type_filter.matches(&entry));
        
        let type_filter2 = LogFilter::new().with_entry_type(LogEntryType::DomainEvent);
        assert!(!type_filter2.matches(&entry));
        
        // Test agent filter
        let agent_filter = LogFilter::new().with_agent(agent_id::from_string("agent1"));
        assert!(agent_filter.matches(&entry));
        
        let agent_filter2 = LogFilter::new().with_agent(agent_id::from_string("agent2"));
        assert!(!agent_filter2.matches(&entry));
        
        // Test domain filter
        let domain_filter = LogFilter::new().with_domain("domain1".to_string());
        assert!(domain_filter.matches(&entry));
        
        let domain_filter2 = LogFilter::new().with_domain("domain2".to_string());
        assert!(!domain_filter2.matches(&entry));
        
        // Test combined filters
        let combined_filter = LogFilter::new()
            .with_entry_type(LogEntryType::AgentAction)
            .with_agent(agent_id::from_string("agent1"))
            .with_domain("domain1".to_string());
        assert!(combined_filter.matches(&entry));
        
        let combined_filter2 = LogFilter::new()
            .with_entry_type(LogEntryType::AgentAction)
            .with_agent(agent_id::from_string("agent2"));
        assert!(!combined_filter2.matches(&entry));
    }
    
    #[test]
    fn test_memory_observer() {
        // Create a test entry
        let entry = create_test_entry(
            LogEntryType::AgentAction,
            Some("agent1"),
            Some("domain1"),
            HashMap::new(),
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
        let mut filtered_observer = MemoryObserver::new();
        filtered_observer.with_filter(LogFilter::new().with_agent(agent_id::from_string("agent2")));
        
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
            HashMap::new(),
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
        let (mut observer, mut receiver) = ChannelObserver::new();
        
        // Create test entries using only valid LogEntryType variants
        let entry1 = create_test_entry(
            LogEntryType::SimulationEvent,
            Some("agent1"),
            Some("domain1"),
            HashMap::new(),
        );
        
        let entry2 = create_test_entry(
            LogEntryType::AgentState,
            Some("agent2"),
            Some("domain1"),
            HashMap::new(),
        );
        
        // Create a separate filtered observer
        let (mut filtered_observer, mut filtered_receiver) = ChannelObserver::new();
        
        // Create the agent ID before using it in with_filter to avoid temporary value issues
        let agent_id = agent_id::from_string("agent2");
        filtered_observer.with_filter(LogFilter::new().with_agent(agent_id));
        
        // Send entries to both observers
        observer.on_log_entry(entry1.clone());
        observer.on_log_entry(entry2.clone());
        
        filtered_observer.on_log_entry(entry1.clone());
        filtered_observer.on_log_entry(entry2.clone());
        
        // The regular observer should receive both entries
        assert_eq!(receiver.try_recv().unwrap().entry_type, LogEntryType::SimulationEvent);
        assert_eq!(receiver.try_recv().unwrap().entry_type, LogEntryType::AgentState);
        assert!(receiver.try_recv().is_err());
        
        // The filtered observer should only receive the entry for agent2
        assert_eq!(filtered_receiver.try_recv().unwrap().entry_type, LogEntryType::AgentState);
        assert!(filtered_receiver.try_recv().is_err());
    }
    
    #[test]
    fn test_observer_registry() {
        // Create a registry
        let registry = ObserverRegistry::new();
        
        // Create memory observers with different names
        let observer1 = Arc::new(MemoryObserver::with_name("observer1"));
        
        // Register the first observer
        registry.register(observer1.clone());
        
        // Check observer count after registering one observer
        let count1 = registry.observer_count();
        assert_eq!(count1, 1);
        
        // Create and register the second observer with a different name
        let observer2 = Arc::new(MemoryObserver::with_name("observer2"));
        registry.register(observer2.clone());
        
        // Check observer count - this should match the number of observers registered
        let count2 = registry.observer_count();
        assert_eq!(count2, 2);
        
        // Create a test entry
        let entry = create_test_entry(
            LogEntryType::AgentAction,
            Some("agent1"),
            Some("domain1"),
            HashMap::new(),
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
