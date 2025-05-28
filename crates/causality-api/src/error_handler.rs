//! API Error Handler
//!
//! This module implements specialized error handling for API interactions, including
//! contextual error creation, error logging, and statistics tracking. All error handling
//! components maintain ZK compatibility with bounded sizes.

use async_trait::async_trait;
use crate::serialization::{Encode, Decode, SimpleSerialize, DecodeError};
use causality_types::primitive::error::ContextualError;
use causality_types::primitive::error::ErrorCategory;
use causality_types::primitive::error::{AsErrorContext, ErrorMetadata};
use causality_types::primitive::logging::LogLevel;
use causality_types::primitive::mock_logger::MockLogger;
use causality_types::primitive::logging::AsLogger;
use std::sync::Arc;
use tokio::sync::Mutex;

//-----------------------------------------------------------------------------
// Core Error Type
//-----------------------------------------------------------------------------

/// API-specific error handler with enhanced context
pub struct ApiErrorHandler {
    /// Logger for recording errors
    logger: Arc<MockLogger>,

    /// Error statistics
    stats: Mutex<ErrorStats>,
}

/// Statistics for error tracking

#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    /// Total number of errors handled
    pub total_errors: u64,

    /// Number of errors by category
    pub errors_by_category: Vec<(ErrorCategory, u64)>,

    /// Number of retried operations
    pub retries: u64,
}


//-----------------------------------------------------------------------------
// Error Handler Implementation
//-----------------------------------------------------------------------------

impl ApiErrorHandler {
    /// Create a new API error handler
    pub fn new(logger: Arc<MockLogger>) -> Self {
        Self {
            logger,
            stats: Mutex::new(ErrorStats::default()),
        }
    }

    /// Create a new API error handler with custom context
    pub fn with_context(
        _context: Arc<dyn AsErrorContext>,
        logger: Arc<MockLogger>,
    ) -> Self {
        Self {
            logger,
            stats: Mutex::new(ErrorStats::default()),
        }
    }

    /// Create an error with API context
    pub fn create_error(
        &self,
        message: impl Into<String>,
        category: ErrorCategory,
    ) -> ContextualError {
        // Create the error directly rather than through the trait object
        ContextualError::new(message.into(), ErrorMetadata::new(category))
    }

    /// Handle an error asynchronously (log it and update stats)
    pub async fn handle_error(
        &self,
        error: &ContextualError,
    ) -> Result<(), ContextualError> {
        /// Update error statistics with this error
        async fn update_stats(
            stats_mutex: &Mutex<ErrorStats>,
            error: &ContextualError,
        ) {
            let mut stats_guard = stats_mutex.lock().await;
            stats_guard.total_errors += 1;

            let category = error.category();

            match stats_guard
                .errors_by_category
                .iter_mut()
                .find(|(cat, _)| *cat == category)
            {
                Some((_, count)) => *count += 1,
                None => stats_guard.errors_by_category.push((category, 1)),
            }
        }

        // Update statistics
        update_stats(&self.stats, error).await;

        // Log the error
        let log_result = (*self.logger)
            .log_message(LogLevel::Error, format!("API Error: {}", error))
            .await;

        if log_result.is_err() {
            // If logging fails, just continue - we don't want to compound the error
            // Optionally, print to stderr here if logging is critical
            // eprintln!("Failed to log error: {:?}", log_result.unwrap_err());
        }

        Ok(())
    }

    /// Get current error statistics
    pub async fn get_stats(&self) -> ErrorStats {
        self.stats.lock().await.clone()
    }
}

//-----------------------------------------------------------------------------
// Error Response Building
//-----------------------------------------------------------------------------

/// Trait for contextual error response building
#[async_trait]
pub trait ErrorResponseBuilder: Send + Sync {
    /// Build an error response from a contextual error
    async fn build_error_response(&self, error: &ContextualError) -> Vec<u8>;

    /// Extract error details for client consumption
    fn extract_error_details(&self, error: &ContextualError) -> ErrorDetails;
}

/// Client-facing error details
#[derive(Debug, Clone)]
pub struct ErrorDetails {
    /// Unique error identifier
    pub error_id: String,

    /// Error category
    pub category: ErrorCategory,

    /// Human-readable error message
    pub message: String,

    /// Additional context information
    pub context: String,

    /// Error timestamp
    pub timestamp: u64,

    /// Additional details (if available)
    pub details: Option<String>,
}

//-----------------------------------------------------------------------------
// SSZ Serialization Implementations
//-----------------------------------------------------------------------------

// ErrorStats
impl Encode for ErrorStats {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.total_errors.as_ssz_bytes());
        
        // Serialize errors_by_category vector
        bytes.extend((self.errors_by_category.len() as u64).as_ssz_bytes());
        for (category, count) in &self.errors_by_category {
            bytes.extend(category.as_ssz_bytes());
            bytes.extend(count.as_ssz_bytes());
        }
        
        bytes.extend(self.retries.as_ssz_bytes());
        
        bytes
    }
}

impl Decode for ErrorStats {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let total_errors = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode total_errors: {}", e) })?;
        offset += 8;
        
        // Decode errors_by_category vector
        let category_count = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode category count: {}", e) })? as usize;
        offset += 8;
        
        let mut errors_by_category = Vec::with_capacity(category_count);
        for i in 0..category_count {
            let category = ErrorCategory::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode error category {}: {}", i, e) })?;
            let category_size = category.as_ssz_bytes().len();
            offset += category_size;
            
            let count = u64::from_ssz_bytes(&bytes[offset..offset + 8])
                .map_err(|e| DecodeError { message: format!("Failed to decode error count {}: {}", i, e) })?;
            offset += 8;
            
            errors_by_category.push((category, count));
        }
        
        let retries = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode retries: {}", e) })?;
        
        Ok(ErrorStats {
            total_errors,
            errors_by_category,
            retries,
        })
    }
}

impl SimpleSerialize for ErrorStats {}

// ErrorDetails
impl Encode for ErrorDetails {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend(self.error_id.as_ssz_bytes());
        bytes.extend(self.category.as_ssz_bytes());
        bytes.extend(self.message.as_ssz_bytes());
        bytes.extend(self.context.as_ssz_bytes());
        bytes.extend(self.timestamp.as_ssz_bytes());
        
        // Serialize details as Option
        match &self.details {
            Some(details) => {
                bytes.push(1u8);
                bytes.extend(details.as_ssz_bytes());
            }
            None => {
                bytes.push(0u8);
            }
        }
        
        bytes
    }
}

impl Decode for ErrorDetails {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        let error_id = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode error_id: {}", e) })?;
        let error_id_size = error_id.as_ssz_bytes().len();
        offset += error_id_size;
        
        let category = ErrorCategory::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode category: {}", e) })?;
        let category_size = category.as_ssz_bytes().len();
        offset += category_size;
        
        let message = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode message: {}", e) })?;
        let message_size = message.as_ssz_bytes().len();
        offset += message_size;
        
        let context = String::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode context: {}", e) })?;
        let context_size = context.as_ssz_bytes().len();
        offset += context_size;
        
        let timestamp = u64::from_ssz_bytes(&bytes[offset..offset + 8])
            .map_err(|e| DecodeError { message: format!("Failed to decode timestamp: {}", e) })?;
        offset += 8;
        
        // Decode details option
        let has_details = bytes[offset];
        offset += 1;
        
        let details = if has_details == 1 {
            let details_str = String::from_ssz_bytes(&bytes[offset..])
                .map_err(|e| DecodeError { message: format!("Failed to decode details: {}", e) })?;
            Some(details_str)
        } else {
            None
        };
        
        Ok(ErrorDetails {
            error_id,
            category,
            message,
            context,
            timestamp,
            details,
        })
    }
}

impl SimpleSerialize for ErrorDetails {}
