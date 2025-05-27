// Purpose: Utility functions for time-related operations.

use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current time in milliseconds since Unix epoch.
/// This function is intended for use by clock implementations that need
/// to access the real system time.
pub fn get_current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
