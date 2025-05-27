//! Request Dispatcher Provider Interface
//!
//! Defines the AsRequestDispatcher Service Provider Interface (SPI).
//! This trait allows dispatching requests and receiving responses in a
//! consistent way across the system.

//-----------------------------------------------------------------------------
// Request Dispatcher Trait
//-----------------------------------------------------------------------------

use anyhow::Result;
use async_trait::async_trait;

/// Trait for dispatching requests and receiving responses.
/// Implementors of this trait can handle a request and return a corresponding response.
#[async_trait]
pub trait AsRequestDispatcher {
    /// The type of the request content.
    type RequestContent;

    /// The type of the response content.
    type ResponseContent;

    /// Dispatches a request and returns a response.
    async fn dispatch_request(
        &self,
        request: Self::RequestContent,
    ) -> Result<Self::ResponseContent>;
}
