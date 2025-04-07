// Purpose: Extends ObserverRegistry with log storage functionality

use std::sync::Arc;
use crate::observer::ObserverRegistry;
use crate::replay::AsyncLogStorageAdapter;

/// Extension traits for ObserverRegistry to provide log storage access
pub trait LogStorageExt {
    /// Get the log storage adapter from the registry
    fn get_log_storage(&self) -> Arc<AsyncLogStorageAdapter>;
}

impl LogStorageExt for Arc<ObserverRegistry> {
    fn get_log_storage(&self) -> Arc<AsyncLogStorageAdapter> {
        // This method would normally retrieve a log storage observer from the registry
        // For now, create a temporary one - this will need to be improved in a real implementation
        // to properly retrieve the registered log storage
        match AsyncLogStorageAdapter::new_temp() {
            Ok(adapter) => Arc::new(adapter),
            Err(_) => panic!("Failed to create temporary log storage adapter"),
        }
    }
} 