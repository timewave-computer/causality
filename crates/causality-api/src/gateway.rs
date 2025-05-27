//! API Gateway Traits
//!
//! This module defines the core traits and types that all API gateways must implement.
//! These interfaces provide a standardized way for external systems to communicate
//! with the Causality framework's components.

use async_trait::async_trait;
// Removed unused import: std::sync::Arc

//-----------------------------------------------------------------------------
// Core Gateway Interface
//-----------------------------------------------------------------------------

/// Core gateway trait that all external API endpoints must implement.
/// Provides a standardized way to handle communication with external systems.
#[async_trait]
pub trait ApiGateway: Send + Sync {
    /// The request type handled by this gateway
    type Request: Send + Sync;

    /// The response type returned by this gateway
    type Response: Send + Sync;

    /// Process an incoming request and return a response
    async fn handle_request(
        &self,
        request: Self::Request,
    ) -> Result<Self::Response, ApiError>;

    /// Check if the gateway is healthy and ready to process requests
    async fn health_check(&self) -> bool;
}

//-----------------------------------------------------------------------------
// Factory Interface
//-----------------------------------------------------------------------------

/// Factory trait for creating API gateways
#[async_trait]
pub trait ApiGatewayFactory: Send + Sync {
    /// The type of gateway this factory creates
    type Gateway: ApiGateway;

    /// Create a new gateway instance
    async fn create_gateway(&self) -> Result<Self::Gateway, ApiError>;
}

//-----------------------------------------------------------------------------
// Error Handling
//-----------------------------------------------------------------------------

/// Error type for API operations
#[derive(Debug, Clone)]
pub enum ApiError {
    /// The request was malformed or invalid
    InvalidRequest(String),

    /// Authentication or authorization failed
    AuthError(String),

    /// An internal error occurred while processing the request
    InternalError(String),

    /// The requested resource was not found
    NotFound(String),

    /// The service is temporarily unavailable
    ServiceUnavailable(String),

    /// Serialization or deserialization error
    SerializationError(String),
}

impl ApiError {
    /// Constructor for serialization errors
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        Self::SerializationError(msg.into())
    }

    /// Constructor for transaction-related errors
    pub fn transaction<S: Into<String>>(msg: S) -> Self {
        Self::InternalError(msg.into())
    }

    /// Constructor for query-related errors
    pub fn query<S: Into<String>>(msg: S) -> Self {
        Self::InternalError(msg.into())
    }

    /// Constructor for invalid request errors
    pub fn invalid_request<S: Into<String>>(msg: S) -> Self {
        Self::InvalidRequest(msg.into())
    }
}
