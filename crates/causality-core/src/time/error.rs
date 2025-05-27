// Time-related errors
//
// This module defines errors that can occur during time-related operations.

use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Errors that can occur in time-related operations
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeError {
    /// Invalid time format provided
    #[error("Invalid time format: {0}")]
    InvalidFormat(String),
    
    /// Time value out of bounds
    #[error("Time value out of bounds: {0}")]
    OutOfBounds(String),
    
    /// Clock error
    #[error("Clock error: {0}")]
    ClockError(String),
    
    /// Parse error when converting from string
    #[error("Failed to parse time: {0}")]
    ParseError(String),
    
    /// Invalid timestamp
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    
    /// System time error
    #[error("System time error: {0}")]
    SystemTimeError(String),
    
    /// Other general time errors
    #[error("{0}")]
    Other(String),
}

impl TimeError {
    /// Create a new other error from a message
    pub fn new(msg: impl ToString) -> Self {
        TimeError::Other(msg.to_string())
    }
    
    /// Create a parse error
    pub fn parse_error(msg: impl ToString) -> Self {
        TimeError::ParseError(msg.to_string())
    }
    
    /// Create an invalid timestamp error
    pub fn invalid_timestamp(msg: impl ToString) -> Self {
        TimeError::InvalidTimestamp(msg.to_string())
    }
}

/// Convenient type alias for Result with TimeError
pub type TimeResult<T> = std::result::Result<T, TimeError>; 