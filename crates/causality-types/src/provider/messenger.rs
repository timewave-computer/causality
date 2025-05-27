//! Messenger Service Provider Interface
//!
//! Defines the Messenger SPI for message delivery across the system.
//! This trait allows sending messages, encompassing both general
//! publishing and targeted sends.

//-----------------------------------------------------------------------------
// Messaging Provider Trait
//-----------------------------------------------------------------------------

use crate::{anyhow::Result, system::pattern::Message};

/// Trait for entities capable of sending or publishing messages.
/// Implementors should handle the specifics of message transport and persistence if needed.
#[async_trait::async_trait]
pub trait AsMessenger: Send + Sync {
    /// Sends a message.
    ///
    /// This method should ensure the message is processed according to the
    /// system's messaging semantics, which might involve invoking Lisp behaviors
    /// associated with the message's target resource type.
    ///
    /// # Arguments
    /// * `message` - A reference to the `Message` to be sent.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    async fn send_message(&self, message: &Message) -> Result<()>;
}
