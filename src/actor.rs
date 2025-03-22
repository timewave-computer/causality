// Actor system for Causality
//
// This module provides actor framework functionality for managing and
// communicating between different participants in the system.

// Module declarations
pub mod identity;
pub mod role;
pub mod user;
pub mod operator;
pub mod committee;
pub mod registry;
pub mod communication;
pub mod messaging;

// Re-exports of core types
pub use identity::{Identity, IdentityId, IdentityType, IdentityVerifier};
pub use role::{Role, RoleId, RoleType, RoleCapability};
pub use user::{User, UserId, UserType, UserStatus};
pub use operator::{Operator, OperatorId, OperatorType, OperatorStatus};
pub use committee::{Committee, CommitteeId, CommitteeType, CommitteeStatus};
pub use registry::{ActorRegistry, IdentityRegistry, RoleRegistry};
pub use communication::{Message, MessageId, MessageStatus, MessageType};

use crate::types::{DomainId, ResourceId, Timestamp};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

/// Type for actor IDs
///
/// This is a base trait for all actor ID types in the system.
pub trait ActorId: Debug + Display + Clone + PartialEq + Eq + Hash + Send + Sync {}

/// Base trait for all actors in the system
///
/// An actor is an entity that can perform actions in the system.
pub trait Actor: Debug + Send + Sync {
    /// The type of ID for this actor
    type Id: ActorId;
    
    /// Get the ID of this actor
    fn id(&self) -> &Self::Id;
    
    /// Get the display name of this actor
    fn name(&self) -> &str;
    
    /// Get the domains this actor is associated with
    fn domains(&self) -> &[DomainId];
    
    /// Get the resources this actor has access to
    fn resources(&self) -> &[ResourceId];
    
    /// Get when this actor was created
    fn created_at(&self) -> Timestamp;
    
    /// Get when this actor was last active
    fn last_active(&self) -> Timestamp;
    
    /// Check if this actor is currently active
    fn is_active(&self) -> bool;
    
    /// Mark this actor as active
    fn set_active(&mut self, active: bool);
    
    /// Update the last active timestamp
    fn update_last_active(&mut self, timestamp: Timestamp);
}

/// A trait for entities that can send messages
///
/// A sender is able to create and dispatch messages to recipients.
pub trait MessageSender: Debug + Send + Sync {
    /// Send a message to a recipient
    fn send_message<R: MessageRecipient>(
        &self,
        recipient: &R,
        message_type: MessageType,
        content: &[u8],
    ) -> Result<MessageId, MessageError>;
    
    /// Check if a message was delivered
    fn is_delivered(&self, message_id: &MessageId) -> bool;
    
    /// Get the status of a message
    fn message_status(&self, message_id: &MessageId) -> Option<MessageStatus>;
}

/// A trait for entities that can receive messages
///
/// A recipient is able to receive and process messages from senders.
pub trait MessageRecipient: Debug + Send + Sync {
    /// Receive a message from a sender
    fn receive_message<S: MessageSender>(
        &mut self,
        sender: &S,
        message: Message,
    ) -> Result<(), MessageError>;
    
    /// Process received messages
    fn process_messages(&mut self) -> usize;
    
    /// Check if there are any unprocessed messages
    fn has_messages(&self) -> bool;
    
    /// Get the number of unprocessed messages
    fn message_count(&self) -> usize;
}

/// Error type for messaging operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageError {
    /// The sender is not authorized to send this message
    Unauthorized,
    /// The recipient is not available
    RecipientUnavailable,
    /// The message is too large
    MessageTooLarge,
    /// The message is invalid
    InvalidMessage,
    /// An internal error occurred
    InternalError(String),
}

/// Shared actor reference
///
/// This is a shared reference to an actor that can be cloned
/// and passed around without copying the actor itself.
pub struct SharedActor<A: Actor + ?Sized> {
    /// The inner actor
    inner: Arc<A>,
}

impl<A: Actor + ?Sized> SharedActor<A> {
    /// Create a new shared actor
    pub fn new(actor: A) -> Self
    where
        A: Sized,
    {
        SharedActor {
            inner: Arc::new(actor),
        }
    }
    
    /// Get a reference to the inner actor
    pub fn inner(&self) -> &A {
        &self.inner
    }
}

impl<A: Actor + ?Sized> Clone for SharedActor<A> {
    fn clone(&self) -> Self {
        SharedActor {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<A: Actor + ?Sized> Debug for SharedActor<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedActor({:?})", self.inner)
    }
}

/// Create a shared actor from an actor
pub fn shared<A: Actor + Sized>(actor: A) -> SharedActor<A> {
    SharedActor::new(actor)
} 