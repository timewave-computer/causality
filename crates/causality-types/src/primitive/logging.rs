//-----------------------------------------------------------------------------
// Logging System
//-----------------------------------------------------------------------------

use async_trait::async_trait;
use crate::serialization::{Decode, DecodeError, Encode, SimpleSerialize};
use std::collections::HashMap;

use crate::primitive::error::BoundedString;
use crate::primitive::string::Str;

/// Maximum number of context entries in a log entry
pub const MAX_CONTEXT_ENTRIES: usize = 16;

/// Log level for message severity
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum LogLevel {
    /// Trace-level messages for detailed debugging
    Trace = 0,

    /// Debug-level messages for development
    Debug = 1,

    /// Informational messages
    Info = 2,

    /// Warning messages
    Warning = 3,

    /// Error messages
    Error = 4,

    /// Critical error messages
    Critical = 5,
}

/// Fixed-size domain identifier for logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogDomainId(pub [u8; 16]);

/// Log entry with bounded size for ZK compatibility
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,

    /// Message content (bounded size)
    pub message: BoundedString,

    /// Timestamp (nanoseconds since UNIX epoch)
    pub timestamp: u64,

    /// Source domain
    pub domain: LogDomainId,

    /// Additional metadata (bounded size)
    pub metadata: HashMap<BoundedString, BoundedString>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        domain: LogDomainId,
        timestamp: u64,
    ) -> Self {
        Self {
            level,
            message: BoundedString::new(message),
            timestamp,
            domain,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the log entry
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        if self.metadata.len() < MAX_CONTEXT_ENTRIES {
            self.metadata
                .insert(BoundedString::new(key), BoundedString::new(value));
        }
        self
    }
}

/// Trait for logging messages with different levels.
/// Implementors handle the actual log output (e.g., to console, file, network).
#[async_trait]
pub trait AsLogger: Send + Sync {
    /// Logs a message at the error level.
    fn error(&self, message: Str);

    /// Log a message at the specified level
    async fn log(&self, entry: LogEntry) -> Result<(), LogError>;

    /// Log a simple message at the specified level
    async fn log_message(
        &self,
        level: LogLevel,
        message: impl Into<String> + Send,
    ) -> Result<(), LogError> {
        let domain = self.default_domain();
        let timestamp = self.current_timestamp();
        self.log(LogEntry::new(level, message, domain, timestamp))
            .await
    }

    /// Log a trace message
    async fn trace(
        &self,
        message: impl Into<String> + Send,
    ) -> Result<(), LogError> {
        self.log_message(LogLevel::Trace, message).await
    }

    /// Log a debug message
    async fn debug(
        &self,
        message: impl Into<String> + Send,
    ) -> Result<(), LogError> {
        self.log_message(LogLevel::Debug, message).await
    }

    /// Log an info message
    async fn info(&self, message: impl Into<String> + Send) -> Result<(), LogError> {
        self.log_message(LogLevel::Info, message).await
    }

    /// Log a warning message
    async fn warning(
        &self,
        message: impl Into<String> + Send,
    ) -> Result<(), LogError> {
        self.log_message(LogLevel::Warning, message).await
    }

    /// Log a critical message
    async fn critical(
        &self,
        message: impl Into<String> + Send,
    ) -> Result<(), LogError> {
        self.log_message(LogLevel::Critical, message).await
    }

    /// Get the default domain for this logger
    fn default_domain(&self) -> LogDomainId;

    /// Get the current timestamp (nanoseconds since UNIX epoch)
    fn current_timestamp(&self) -> u64;

    /// Flush all pending log entries
    async fn flush(&self) -> Result<(), LogError>;
}

/// Error that can occur during logging operations
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Helper macro for logging with contextual information
#[macro_export]
macro_rules! log_with_context {
    ($logger:expr, $level:expr, $message:expr) => {
        $logger.log(
            $crate::core::logger::LogEntry::new(
                $level,
                $message,
                $logger.default_domain(),
                $logger.current_timestamp(),
            )
        ).await
    };
    ($logger:expr, $level:expr, $message:expr, $($key:expr => $value:expr),*) => {{
        let mut entry = $crate::core::logger::LogEntry::new(
            $level,
            $message,
            $logger.default_domain(),
            $logger.current_timestamp(),
        );
        $(
            entry = entry.with_metadata($key, $value);
        )*
        $logger.log(entry).await
    }};
}

// Manually implement Encode for LogLevel
impl Encode for LogLevel {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let value = match self {
            LogLevel::Trace => 0u8,
            LogLevel::Debug => 1u8,
            LogLevel::Info => 2u8,
            LogLevel::Warning => 3u8,
            LogLevel::Error => 4u8,
            LogLevel::Critical => 5u8,
        };
        vec![value]
    }
}

// Manually implement Decode for LogLevel
impl Decode for LogLevel {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Expected at least 1 byte for LogLevel".to_string(),
            });
        }
        
        match bytes[0] {
            0 => Ok(LogLevel::Trace),
            1 => Ok(LogLevel::Debug),
            2 => Ok(LogLevel::Info),
            3 => Ok(LogLevel::Warning),
            4 => Ok(LogLevel::Error),
            5 => Ok(LogLevel::Critical),
            _ => Err(DecodeError {
                message: format!("Invalid LogLevel value: {}", bytes[0]),
            }),
        }
    }
}

// Implement SimpleSerialize for LogLevel
impl SimpleSerialize for LogLevel {}

// Manually implement Encode for LogDomainId
impl Encode for LogDomainId {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

// Manually implement Decode for LogDomainId
impl Decode for LogDomainId {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 16 {
            return Err(DecodeError {
                message: format!("Expected 16 bytes for LogDomainId, got {}", bytes.len()),
            });
        }
        
        let mut array = [0u8; 16];
        array.copy_from_slice(&bytes[0..16]);
        
        Ok(LogDomainId(array))
    }
}

// Implement SimpleSerialize for LogDomainId
impl SimpleSerialize for LogDomainId {}
