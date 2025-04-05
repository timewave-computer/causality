// Time Observer Module
//
// This module provides abstractions for observing time-related events and changes.

use std::sync::Mutex;
use async_trait::async_trait;

use super::timestamp::Timestamp;
use super::clock::ClockTime;

/// A trait for observing time-related events
#[async_trait]
pub trait TimeObserver: Send + Sync {
    /// Called when time advances in a domain
    async fn on_time_advance(&self, domain_id: &str, causal_time: Timestamp, clock_time: Option<ClockTime>);
    
    /// Called when a time domain is created
    async fn on_domain_create(&self, domain_id: &str);
    
    /// Called when time domains are merged
    async fn on_domain_merge(&self, source_id: &str, target_id: &str);
    
    /// Called when time domains are synchronized
    async fn on_domain_sync(&self, domain_id: &str, other_id: &str);
}

/// A simple time observer that logs events
#[derive(Debug)]
pub struct LoggingTimeObserver {
    /// The name of this observer
    name: String,
}

impl LoggingTimeObserver {
    /// Create a new logging time observer
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl TimeObserver for LoggingTimeObserver {
    async fn on_time_advance(&self, domain_id: &str, causal_time: Timestamp, clock_time: Option<ClockTime>) {
        let clock_time_str = clock_time.map(|t| format!(", clock time: {}", t)).unwrap_or_default();
        println!("[{}] Time advanced in domain {}: causal time: {}{}", 
            self.name, domain_id, causal_time, clock_time_str);
    }
    
    async fn on_domain_create(&self, domain_id: &str) {
        println!("[{}] Time domain created: {}", self.name, domain_id);
    }
    
    async fn on_domain_merge(&self, source_id: &str, target_id: &str) {
        println!("[{}] Time domains merged: {} -> {}", 
            self.name, source_id, target_id);
    }
    
    async fn on_domain_sync(&self, domain_id: &str, other_id: &str) {
        println!("[{}] Time domains synchronized: {} <-> {}", 
            self.name, domain_id, other_id);
    }
}

/// A time observer that collects events
#[derive(Debug)]
pub struct CollectingTimeObserver {
    /// The name of this observer
    name: String,
    
    /// The collected time advance events
    time_advances: Mutex<Vec<(String, Timestamp, Option<ClockTime>)>>,
    
    /// The collected domain create events
    domain_creates: Mutex<Vec<String>>,
    
    /// The collected domain merge events
    domain_merges: Mutex<Vec<(String, String)>>,
    
    /// The collected domain sync events
    domain_syncs: Mutex<Vec<(String, String)>>,
}

impl CollectingTimeObserver {
    /// Create a new collecting time observer
    pub fn new(name: impl Into<String>) -> Self {
        Self { 
            name: name.into(),
            time_advances: Mutex::new(Vec::new()),
            domain_creates: Mutex::new(Vec::new()),
            domain_merges: Mutex::new(Vec::new()),
            domain_syncs: Mutex::new(Vec::new()),
        }
    }
    
    /// Get the collected time advance events
    pub fn time_advances(&self) -> Vec<(String, Timestamp, Option<ClockTime>)> {
        self.time_advances.lock().unwrap().clone()
    }
    
    /// Get the collected domain create events
    pub fn domain_creates(&self) -> Vec<String> {
        self.domain_creates.lock().unwrap().clone()
    }
    
    /// Get the collected domain merge events
    pub fn domain_merges(&self) -> Vec<(String, String)> {
        self.domain_merges.lock().unwrap().clone()
    }
    
    /// Get the collected domain sync events
    pub fn domain_syncs(&self) -> Vec<(String, String)> {
        self.domain_syncs.lock().unwrap().clone()
    }
    
    /// Clear all collected events
    pub fn clear(&self) {
        self.time_advances.lock().unwrap().clear();
        self.domain_creates.lock().unwrap().clear();
        self.domain_merges.lock().unwrap().clear();
        self.domain_syncs.lock().unwrap().clear();
    }
}

impl Default for CollectingTimeObserver {
    fn default() -> Self {
        Self::new("")
    }
}

#[async_trait]
impl TimeObserver for CollectingTimeObserver {
    async fn on_time_advance(&self, domain_id: &str, causal_time: Timestamp, clock_time: Option<ClockTime>) {
        self.time_advances.lock().unwrap().push((domain_id.to_string(), causal_time, clock_time));
    }
    
    async fn on_domain_create(&self, domain_id: &str) {
        self.domain_creates.lock().unwrap().push(domain_id.to_string());
    }
    
    async fn on_domain_merge(&self, source_id: &str, target_id: &str) {
        self.domain_merges.lock().unwrap().push((source_id.to_string(), target_id.to_string()));
    }
    
    async fn on_domain_sync(&self, domain_id: &str, other_id: &str) {
        self.domain_syncs.lock().unwrap().push((domain_id.to_string(), other_id.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::timestamp::Timestamp;

    #[tokio::test]
    async fn test_logging_observer() {
        let observer = Arc::new(LoggingTimeObserver::new("test"));
        
        observer.on_time_advance("test_domain", Timestamp::from_nanos(1000), None).await;
        observer.on_domain_create("test_domain").await;
        observer.on_domain_merge("source", "target").await;
        observer.on_domain_sync("domain1", "domain2").await;
    }

    #[tokio::test]
    async fn test_collecting_observer() {
        let observer = Arc::new(CollectingTimeObserver::new("test"));
        
        observer.on_time_advance("test_domain", Timestamp::from_nanos(1000), None).await;
        observer.on_domain_create("test_domain").await;
        observer.on_domain_merge("source", "target").await;
        observer.on_domain_sync("domain1", "domain2").await;
        
        let time_advances = observer.time_advances();
        let domain_creates = observer.domain_creates();
        let domain_merges = observer.domain_merges();
        let domain_syncs = observer.domain_syncs();
        
        assert_eq!(time_advances.len(), 1);
        assert_eq!(domain_creates.len(), 1);
        assert_eq!(domain_merges.len(), 1);
        assert_eq!(domain_syncs.len(), 1);
        
        assert_eq!(time_advances[0].0, "test_domain");
        assert_eq!(time_advances[0].1, Timestamp::from_nanos(1000));
        assert!(time_advances[0].2.is_none());
        
        assert_eq!(domain_creates[0], "test_domain");
        
        assert_eq!(domain_merges[0].0, "source");
        assert_eq!(domain_merges[0].1, "target");
        
        assert_eq!(domain_syncs[0].0, "domain1");
        assert_eq!(domain_syncs[0].1, "domain2");
    }
} 