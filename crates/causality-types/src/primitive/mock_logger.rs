//! Mock logger implementation for testing

use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::logging::{AsLogger, LogDomainId, LogEntry, LogError, LogLevel};
use crate::primitive::string::Str;

//-----------------------------------------------------------------------------
// Mock Logger
//-----------------------------------------------------------------------------

/// A mock logger that stores log entries in memory for testing.
#[derive(Debug)]
pub struct MockLogger {
    /// Domain ID for this logger
    domain_id: LogDomainId,
    /// Log entries stored in memory
    entries: Mutex<VecDeque<LogEntry>>,
    /// Maximum number of entries to store
    capacity: usize,
    /// Current timestamp (can be set manually for tests)
    timestamp: Mutex<u64>,
}

impl MockLogger {
    /// Create a new mock logger with a random domain ID
    pub fn new(capacity: usize) -> Self {
        // Create a domain ID with bytes derived from the current timestamp
        let mut domain_bytes = [0u8; 16];
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;

        // Fill in bytes from timestamp
        for (i, byte) in domain_bytes.iter_mut().enumerate().take(8) {
            *byte = ((timestamp >> (i * 8)) & 0xFF) as u8;
        }
        // Fill remaining bytes with more timestamp-derived data to avoid collisions
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

    /// Set the current timestamp (for testing)
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

    /// For testing: Get entries filtered by log level
    pub fn entries_at_level(&self, level: LogLevel) -> Vec<LogEntry> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .filter(|entry| entry.level == level)
            .cloned()
            .collect()
    }

    /// For testing: Get all entries
    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries()
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

    /// Print all log entries to the console
    pub fn print_entries(&self) {
        for entry in self.entries.lock().unwrap().iter() {
            println!(
                "[{:?}] {:?} ({}): {}",
                entry.level,
                entry.domain.0, // Assuming LogDomainId.0 is the relevant part for display
                entry.timestamp,
                entry.message.as_str() // Assuming BoundedString has as_str()
            );
            for (key, value) in &entry.metadata {
                println!("    {}: {}", key.as_str(), value.as_str());
            }
        }
    }
}

// Manual Clone implementation since Mutex doesn't implement Clone
impl Clone for MockLogger {
    fn clone(&self) -> Self {
        // Create a new logger with the same domain ID and capacity
        let mut new_logger = Self::new(self.capacity);
        new_logger.domain_id = self.domain_id;

        // Copy the current timestamp
        if let (Ok(source), Ok(mut target)) =
            (self.timestamp.lock(), new_logger.timestamp.lock())
        {
            *target = *source;
        }

        // Copy the entries if possible
        if let (Ok(source), Ok(mut target)) =
            (self.entries.lock(), new_logger.entries.lock())
        {
            target.clear();
            target.extend(source.iter().cloned());
        }

        new_logger
    }
}

// Special implementation
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
        // Store the entry directly
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
        // No actual flushing needed for the mock
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;

    // Simple non-tokio async executor for tests
    fn block_on<F: Future>(future: F) -> F::Output {
        // Create a new single threaded executor without tokio
        use std::sync::{Arc, Mutex};
        use std::task::{Context, Poll, Wake, Waker};
        
        struct Woke(Arc<Mutex<bool>>);
        
        impl Wake for Woke {
            fn wake(self: Arc<Self>) {
                *self.0.lock().unwrap() = true;
            }
        }
        
        // Create waker using Wake implementation
        let woke = Arc::new(Mutex::new(false));
        let waker = Waker::from(Arc::new(Woke(woke.clone())));
        let mut cx = Context::from_waker(&waker);
        
        // Use Box::pin instead of Pin::new
        let mut future = Box::pin(future);
        
        loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(val) => break val,
                Poll::Pending => {
                    if *woke.lock().unwrap() {
                        *woke.lock().unwrap() = false;
                        continue;
                    }
                    std::thread::yield_now();
                }
            }
        }
    }

    #[test]
    fn test_mock_logger() {
        let logger = MockLogger::new(10);

        // Test basic logging
        block_on(async {
            logger.debug("Test debug message").await.unwrap();
            logger.info("Test info message").await.unwrap();
            logger.warning("Test warn message").await.unwrap();
            logger.error("Test error message".into());
        });

        // Verify entries were stored
        let entries = logger.get_entries();
        assert_eq!(entries.len(), 4);

        // Test filtering by level
        let error_entries = logger.entries_at_level(LogLevel::Error);
        assert_eq!(error_entries.len(), 1);
        assert_eq!(error_entries[0].message.0, "Test error message");

        // Test clearing entries
        logger.clear();
        assert_eq!(logger.get_entries().len(), 0);
    }
}
