// Log system integration tests
//
// This module contains integration tests for the log system.

#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use std::sync::Arc;

    #[test]
    fn test_memory_storage() {
        let storage = MemoryLogStorage::new();
        assert_eq!(storage.entry_count().unwrap(), 0);
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = entry::LogEntry::new_event(
            "test_event",
            entry::EventSeverity::Info,
            "Test event message",
            None,  // resources
            None,  // domains
            None   // metadata
        );
        
        assert_eq!(entry.entry_type, entry::EntryType::Event);
    }
} 