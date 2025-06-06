//! Configuration for the Causality API
//!
//! This module provides configuration types for the API server
//! and execution environment.

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Server host
    pub host: String,
    
    /// Server port
    pub port: u16,
    
    /// Enable CORS
    pub enable_cors: bool,
    
    /// Maximum request size in bytes
    pub max_request_size: usize,
    
    /// Session timeout in seconds
    pub session_timeout: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            enable_cors: true,
            max_request_size: 1024 * 1024, // 1MB
            session_timeout: 3600, // 1 hour
        }
    }
} 