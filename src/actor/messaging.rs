//! Actor messaging system
//!
//! This module provides the messaging infrastructure for actors to communicate
//! with each other using various patterns like request-response and publish-subscribe.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::types::Timestamp;
use super::ActorId;

mod mailbox;
mod patterns;

pub use mailbox::MailboxSystem;
pub use patterns::{RequestResponse, PubSubSystem};

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Low priority message
    Low = 0,
    /// Normal priority message (default)
    Normal = 1,
    /// High priority message
    High = 2,
    /// Critical priority message (processed before all others)
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

/// Message identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(String);

impl MessageId {
    /// Create a new random message ID
    pub fn new() -> Self {
        MessageId(Uuid::new_v4().to_string())
    }
    
    /// Create a message ID from a string
    pub fn from_string(id: String) -> Self {
        MessageId(id)
    }
    
    /// Get the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message categories for routing and processing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageCategory {
    /// Command messages that trigger actions
    Command,
    /// Query messages that request information
    Query,
    /// Event messages that notify about state changes
    Event,
    /// System messages for internal actor lifecycle management
    System,
    /// Custom message category
    Custom(String),
}

/// Message envelope containing a message and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: MessageId,
    /// Sender actor ID
    pub sender: Option<ActorId>,
    /// Recipient actor ID
    pub recipient: ActorId,
    /// Related message ID (for replies)
    pub in_reply_to: Option<MessageId>,
    /// Message category
    pub category: MessageCategory,
    /// Message priority
    pub priority: MessagePriority,
    /// When this message was created
    pub created_at: Timestamp,
    /// When this message expires (if applicable)
    pub expires_at: Option<Timestamp>,
    /// Message payload
    pub payload: MessagePayload,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Message {
    /// Create a new message
    pub fn new(
        sender: Option<ActorId>,
        recipient: ActorId,
        category: MessageCategory,
        payload: MessagePayload,
    ) -> Self {
        Message {
            id: MessageId::new(),
            sender,
            recipient,
            in_reply_to: None,
            category,
            priority: MessagePriority::Normal,
            created_at: Timestamp::now(),
            expires_at: None,
            payload,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a reply to another message
    pub fn reply_to(
        original: &Message,
        sender: ActorId,
        payload: MessagePayload,
    ) -> Self {
        let recipient = match &original.sender {
            Some(sender) => sender.clone(),
            None => return Self::new(Some(sender), ActorId::new("unknown"), original.category.clone(), payload),
        };
        
        let mut reply = Self::new(
            Some(sender),
            recipient,
            original.category.clone(),
            payload,
        );
        
        reply.in_reply_to = Some(original.id.clone());
        reply
    }
    
    /// Set the message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set the message expiration time
    pub fn with_expiration(mut self, expires_in_seconds: u64) -> Self {
        self.expires_at = Some(self.created_at + expires_in_seconds);
        self
    }
    
    /// Add metadata to the message
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Check if this message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Timestamp::now().value() > expires_at.value()
        } else {
            false
        }
    }
}

/// Message payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Text message
    Text(String),
    /// JSON-encoded data
    Json(String),
    /// Binary data
    Binary(Vec<u8>),
    /// Command message
    Command {
        /// Command name
        name: String,
        /// Command arguments as JSON
        args: String,
    },
    /// Query message
    Query {
        /// Query name
        name: String,
        /// Query parameters as JSON
        params: String,
    },
    /// Event message
    Event {
        /// Event name
        name: String,
        /// Event data as JSON
        data: String,
    },
    /// System message
    System {
        /// System message type
        message_type: String,
        /// System message data as JSON
        data: String,
    },
}

/// Message handler trait
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    async fn handle_message(&self, message: Message) -> Result<Option<Message>>;
    
    /// Get the message types this handler can process
    fn supported_categories(&self) -> Vec<MessageCategory>;
    
    /// Check if this handler can process a message
    fn can_handle(&self, message: &Message) -> bool {
        self.supported_categories().contains(&message.category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let sender = ActorId::new("sender");
        let recipient = ActorId::new("recipient");
        
        let message = Message::new(
            Some(sender.clone()),
            recipient.clone(),
            MessageCategory::Command,
            MessagePayload::Text("Hello, world!".to_string()),
        );
        
        assert_eq!(message.sender.unwrap(), sender);
        assert_eq!(message.recipient, recipient);
        assert_eq!(message.category, MessageCategory::Command);
        assert_eq!(message.priority, MessagePriority::Normal);
        
        // Test priority setting
        let high_priority = message.clone().with_priority(MessagePriority::High);
        assert_eq!(high_priority.priority, MessagePriority::High);
        
        // Test expiration
        let expiring = message.clone().with_expiration(3600);
        assert!(expiring.expires_at.is_some());
        assert!(!expiring.is_expired());
    }
    
    #[test]
    fn test_message_reply() {
        let sender = ActorId::new("sender");
        let recipient = ActorId::new("recipient");
        
        let original = Message::new(
            Some(sender.clone()),
            recipient.clone(),
            MessageCategory::Query,
            MessagePayload::Text("Query?".to_string()),
        );
        
        let reply = Message::reply_to(
            &original,
            recipient.clone(),
            MessagePayload::Text("Answer!".to_string()),
        );
        
        assert_eq!(reply.sender.unwrap(), recipient);
        assert_eq!(reply.recipient, sender);
        assert_eq!(reply.in_reply_to.unwrap(), original.id);
        assert_eq!(reply.category, MessageCategory::Query);
    }
} 