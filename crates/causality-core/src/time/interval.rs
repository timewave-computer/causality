// Placeholder for interval types

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntervalBound {
    Inclusive(u64),
    Exclusive(u64),
    Unbounded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeInterval {
    pub start: IntervalBound,
    pub end: IntervalBound,
} 