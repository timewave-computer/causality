// ID Utilities
//
// Provides utilities for generating content-addressed IDs

use causality_types::ContentId;
use rand::random;
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a unique content-addressed identifier with the given prefix
pub fn generate_unique_id(prefix: &str) -> String {
    // Get current timestamp in seconds
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Create a combined vector with the prefix, timestamp, and random bytes
    let mut combined = Vec::with_capacity(prefix.len() + 16);
    combined.extend_from_slice(prefix.as_bytes());
    combined.extend_from_slice(&timestamp.to_be_bytes());
    
    // Add some randomness
    let rand_bytes: [u8; 8] = random();
    combined.extend_from_slice(&rand_bytes);
    
    // Create a ContentId from the combined data
    let content_id = ContentId::from_bytes(&combined);
    
    // Format with the specified prefix
    format!("{}-{}", prefix, content_id.to_string())
}

/// Generate a unique content-addressed identifier for a decision
pub fn generate_decision_id() -> String {
    generate_unique_id("decision")
}

/// Generate a unique content-addressed identifier for a system operation
pub fn generate_system_operation_id() -> String {
    generate_unique_id("operation")
}

/// Generate a unique content-addressed identifier for a maintenance window
pub fn generate_maintenance_window_id() -> String {
    generate_unique_id("window")
}

/// Generate a unique content-addressed identifier for a transfer
pub fn generate_transfer_id() -> String {
    generate_unique_id("transfer")
} 