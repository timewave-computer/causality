//! Actor messaging system
//!
//! This module provides the messaging infrastructure for actors to communicate
//! with each other using various patterns like request-response and publish-subscribe.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::error::{Error, Result};
use crate::types::{Timestamp, TraceId};
use crate::actor::{ActorIdBox, ActorRole};
use crate::actor::role::ActorCapability;
use crate::crypto::hash::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};

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

/// Message content for ID generation
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct MessageIdContent {
    /// Sender (if any)
    sender: Option<String>,
    /// Recipient
    recipient: String,
    /// Message timestamp
    timestamp: i64,
    /// Random component for uniqueness
    nonce: [u8; 8],
}

impl ContentAddressed for MessageIdContent {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl MessageId {
    /// Create a new content-derived message ID
    pub fn new() -> Self {
        // Create default content for ID generation
        let content = MessageIdContent {
            sender: None,
            recipient: "unknown".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Generate content-derived ID
        let content_id = content.content_id();
        MessageId(format!("msg:{}", content_id))
    }
    
    /// Create a message ID from specific information
    pub fn from_message_info(
        sender: Option<&ActorIdBox>,
        recipient: &ActorIdBox,
    ) -> Self {
        // Create content with message information
        let content = MessageIdContent {
            sender: sender.map(|s| s.to_string()),
            recipient: recipient.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            nonce: rand::random::<[u8; 8]>(),
        };
        
        // Generate content-derived ID
        let content_id = content.content_id();
        MessageId(format!("msg:{}", content_id))
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

impl From<ContentId> for MessageId {
    fn from(content_id: ContentId) -> Self {
        Self(format!("msg:{}", content_id))
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

/// Message used for communication between actors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique ID for this message
    pub id: MessageId,
    
    /// Actor that sent this message (if any)
    pub sender: Option<ActorIdBox>,
    
    /// Actor that should receive this message
    pub recipient: ActorIdBox,
    
    /// Message category
    pub category: MessageCategory,
    
    /// Message payload
    pub payload: MessagePayload,
    
    /// Message priority
    pub priority: MessagePriority,
    
    /// When the message was created
    pub created_at: Timestamp,
    
    /// When the message expires (if applicable)
    pub expires_at: Option<Timestamp>,
    
    /// Trace ID for tracking related messages
    pub trace_id: Option<TraceId>,
    
    /// Reference to another message (for replies)
    pub in_reply_to: Option<MessageId>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Message {
    /// Create a new message
    pub fn new(
        sender: Option<ActorIdBox>,
        recipient: ActorIdBox,
        category: MessageCategory,
        payload: MessagePayload,
    ) -> Self {
        Self {
            id: MessageId::new(),
            sender,
            recipient,
            category,
            payload,
            priority: MessagePriority::default(),
            created_at: Timestamp::now(),
            expires_at: None,
            trace_id: Some(TraceId::new()),
            in_reply_to: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a reply to this message
    pub fn reply_to(
        original: &Message,
        sender: ActorIdBox,
        payload: MessagePayload,
    ) -> Self {
        let recipient = match &original.sender {
            Some(sender) => sender.clone(),
            None => ActorIdBox::new(),
        };
        
        let mut reply = Self::new(
            Some(sender),
            recipient,
            original.category.clone(),
            payload,
        );
        
        reply.in_reply_to = Some(original.id.clone());
        reply.trace_id = original.trace_id.clone();
        
        reply
    }
    
    /// Set the message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set the message expiration time
    pub fn with_expiration(mut self, expires_in_seconds: u64) -> Self {
        let expires_at = Timestamp::from_seconds(self.created_at.to_seconds() + expires_in_seconds);
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Add metadata to the message
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Check if this message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = &self.expires_at {
            Timestamp::now() > *expires_at
        } else {
            false
        }
    }
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
        let sender = ActorIdBox::from("sender");
        let recipient = ActorIdBox::from("recipient");
        
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
    }
    
    #[test]
    fn test_message_reply() {
        let original_sender = ActorIdBox::from("original_sender");
        let original_recipient = ActorIdBox::from("original_recipient");
        
        let original = Message::new(
            Some(original_sender.clone()),
            original_recipient.clone(),
            MessageCategory::Query,
            MessagePayload::Text("Query data?".to_string()),
        );
        
        let reply = Message::reply_to(
            &original,
            original_recipient.clone(),
            MessagePayload::Text("Response data!".to_string()),
        );
        
        assert_eq!(reply.sender.unwrap(), original_recipient);
        assert_eq!(reply.recipient, original_sender);
        assert_eq!(reply.category, MessageCategory::Query);
    }
} 