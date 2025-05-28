//! Mock implementations for system-level testing

use async_trait::async_trait;
use causality_types::{
    primitive::{
        logging::{AsLogger, LogDomainId, LogEntry, LogError, LogLevel},
        string::Str,
    },
};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

//-----------------------------------------------------------------------------
// Mock Logger
//-----------------------------------------------------------------------------

/// Mock logger that stores log entries in memory for testing
#[derive(Debug)]
pub struct MockLogger {
    domain_id: LogDomainId,
    entries: Mutex<VecDeque<LogEntry>>,
    capacity: usize,
    timestamp: Mutex<u64>,
}

impl MockLogger {
    /// Create a new mock logger with specified capacity
    pub fn new(capacity: usize) -> Self {
        let mut domain_bytes = [0u8; 16];
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;

        // Fill domain bytes with timestamp-derived data
        for (i, byte) in domain_bytes.iter_mut().enumerate().take(8) {
            *byte = ((timestamp >> (i * 8)) & 0xFF) as u8;
        }
        for (i, byte) in domain_bytes.iter_mut().enumerate().skip(8).take(8) {
            *byte = ((timestamp >> ((i - 8) * 8 + 4)) & 0xFF) as u8;
        }

        Self {
            domain_id: LogDomainId(domain_bytes),
            entries: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            timestamp: Mutex::new(timestamp),
        }
    }

    /// Set the current timestamp for testing
    pub fn set_timestamp(&self, timestamp: u64) {
        *self.timestamp.lock().unwrap() = timestamp;
    }

    /// Get all log entries
    pub fn entries(&self) -> Vec<LogEntry> {
        self.entries.lock().unwrap().iter().cloned().collect()
    }

    /// Clear all log entries
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    /// Get entries filtered by log level
    pub fn entries_at_level(&self, level: LogLevel) -> Vec<LogEntry> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .filter(|entry| entry.level == level)
            .cloned()
            .collect()
    }

    /// Check if any error messages have been logged
    pub fn has_errors(&self) -> bool {
        self.entries.lock().unwrap().iter().any(|entry| {
            entry.level == LogLevel::Error || entry.level == LogLevel::Critical
        })
    }

    /// Check if any warning messages have been logged
    pub fn has_warnings(&self) -> bool {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .any(|entry| entry.level == LogLevel::Warning)
    }
}

impl Clone for MockLogger {
    fn clone(&self) -> Self {
        let mut new_logger = Self::new(self.capacity);
        new_logger.domain_id = self.domain_id;

        if let (Ok(source), Ok(mut target)) =
            (self.timestamp.lock(), new_logger.timestamp.lock())
        {
            *target = *source;
        }

        if let (Ok(source), Ok(mut target)) =
            (self.entries.lock(), new_logger.entries.lock())
        {
            target.clear();
            target.extend(source.iter().cloned());
        }

        new_logger
    }
}

#[async_trait]
impl AsLogger for MockLogger {
    fn error(&self, message: Str) {
        let entry = LogEntry::new(
            LogLevel::Error,
            message.as_str(),
            self.default_domain(),
            self.current_timestamp(),
        );
        let mut entries = self.entries.lock().unwrap();
        entries.push_back(entry);
        if entries.len() > self.capacity {
            entries.pop_front();
        }
    }

    async fn log(&self, entry: LogEntry) -> Result<(), LogError> {
        let mut entries = self.entries.lock().unwrap();
        entries.push_back(entry);
        if entries.len() > self.capacity {
            entries.pop_front();
        }
        Ok(())
    }

    fn default_domain(&self) -> LogDomainId {
        self.domain_id
    }

    fn current_timestamp(&self) -> u64 {
        *self.timestamp.lock().unwrap()
    }

    async fn flush(&self) -> Result<(), LogError> {
        Ok(())
    }
}

//-----------------------------------------------------------------------------
// Mock Provider
//-----------------------------------------------------------------------------

/// Generic mock provider for testing
#[derive(Debug, Clone)]
pub struct MockProvider {
    data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    #[allow(dead_code)]
    fail_on_missing: bool,
}

impl MockProvider {
    /// Create a new mock provider
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            fail_on_missing: false,
        }
    }

    /// Create a mock provider that fails when data is missing
    pub fn new_strict() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            fail_on_missing: true,
        }
    }

    /// Insert test data
    pub fn insert(&self, key: String, value: Vec<u8>) {
        self.data.lock().unwrap().insert(key, value);
    }

    /// Get test data
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.lock().unwrap().get(key).cloned()
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.lock().unwrap().contains_key(key)
    }

    /// Clear all data
    pub fn clear(&self) {
        self.data.lock().unwrap().clear();
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.data.lock().unwrap().keys().cloned().collect()
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

//-----------------------------------------------------------------------------
// Mock Store
//-----------------------------------------------------------------------------

/// Mock storage implementation for testing
#[derive(Debug, Clone)]
pub struct MockStore {
    storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    read_only: bool,
}

impl MockStore {
    /// Create a new mock store
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            read_only: false,
        }
    }

    /// Create a read-only mock store
    pub fn new_read_only() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            read_only: true,
        }
    }

    /// Write data to the store
    pub fn write(&self, key: &str, data: &[u8]) -> Result<(), String> {
        if self.read_only {
            return Err("Store is read-only".to_string());
        }
        self.storage
            .lock()
            .unwrap()
            .insert(key.to_string(), data.to_vec());
        Ok(())
    }

    /// Read data from the store
    pub fn read(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        Ok(self.storage.lock().unwrap().get(key).cloned())
    }

    /// Delete data from the store
    pub fn delete(&self, key: &str) -> Result<(), String> {
        if self.read_only {
            return Err("Store is read-only".to_string());
        }
        self.storage.lock().unwrap().remove(key);
        Ok(())
    }

    /// Check if key exists
    pub fn exists(&self, key: &str) -> bool {
        self.storage.lock().unwrap().contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.storage.lock().unwrap().keys().cloned().collect()
    }

    /// Clear all data
    pub fn clear(&self) -> Result<(), String> {
        if self.read_only {
            return Err("Store is read-only".to_string());
        }
        self.storage.lock().unwrap().clear();
        Ok(())
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.storage.lock().unwrap().len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.storage.lock().unwrap().is_empty()
    }
}

impl Default for MockStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_logger() {
        let logger = MockLogger::new(10);
        
        // Test basic logging
        logger.error(Str::from("Test error"));
        
        let entries = logger.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, LogLevel::Error);
        
        // Test capacity limit
        for i in 0..15 {
            let entry = LogEntry::new(
                LogLevel::Info,
                format!("Message {}", i),
                logger.default_domain(),
                logger.current_timestamp(),
            );
            logger.log(entry).await.unwrap();
        }
        
        let entries = logger.entries();
        assert!(entries.len() <= 10);
        
        logger.clear();
        assert_eq!(logger.entries().len(), 0);
    }

    #[test]
    fn test_mock_provider() {
        let provider = MockProvider::new();
        
        provider.insert("key1".to_string(), b"value1".to_vec());
        provider.insert("key2".to_string(), b"value2".to_vec());
        
        assert_eq!(provider.get("key1"), Some(b"value1".to_vec()));
        assert_eq!(provider.get("key2"), Some(b"value2".to_vec()));
        assert_eq!(provider.get("key3"), None);
        
        assert!(provider.contains_key("key1"));
        assert!(!provider.contains_key("key3"));
        
        let keys = provider.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        
        provider.clear();
        assert_eq!(provider.keys().len(), 0);
    }

    #[test]
    fn test_mock_store() {
        let store = MockStore::new();
        
        // Test write and read
        store.write("key1", b"value1").unwrap();
        assert_eq!(store.read("key1").unwrap(), Some(b"value1".to_vec()));
        
        // Test exists
        assert!(store.exists("key1"));
        assert!(!store.exists("key2"));
        
        // Test delete
        store.delete("key1").unwrap();
        assert!(!store.exists("key1"));
        
        // Test read-only store
        let ro_store = MockStore::new_read_only();
        assert!(ro_store.write("key", b"value").is_err());
        assert!(ro_store.delete("key").is_err());
        assert!(ro_store.clear().is_err());
    }
} 