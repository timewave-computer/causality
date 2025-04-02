// Placeholder for time view types

use serde::{Serialize, Deserialize};
use crate::time::map::TimeMapSnapshot; // Assuming TimeMapSnapshot is in map.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeView {
    // Placeholder fields
    pub domain_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeViewSnapshot {
    pub snapshot: TimeMapSnapshot,
    // Add view-specific fields
} 