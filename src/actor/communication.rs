// Actor Communication Module
//
// This module provides communication mechanisms for actors in the Causality system.
// It includes message passing, pub/sub, and reliable delivery patterns.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::time::timeout;
use borsh::{BorshSerialize, BorshDeserialize};
use getrandom;

use crate::error::{Error, Result};
use crate::types::{ContentId, ContentHash, TraceId};
use crate::actor::{ActorIdBox, ActorCapability, ActorRole};
use crate::crypto::content_addressed::{ContentAddressed, ContentId as CryptoContentId};

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Low priority message
    Low = 0,
    /// Normal priority message
    Normal = 1,
    /// High priority message
    High = 2,
    /// Critical priority message
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

/// Timestamp for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Create a new timestamp with the current time
    pub fn now() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Timestamp(now)
    }

    /// Create a timestamp from seconds since the epoch
    pub fn from_seconds(seconds: u64) -> Self {
        Timestamp(seconds)
    }

    /// Get the timestamp value in seconds
    pub fn to_seconds(&self) -> u64 {
        self.0
    }
}

/// Message category for routing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageCategory {
    /// Normal direct message
    Normal,
    /// Message sent to a topic
    Topic(String),
    /// System message
    System,
    /// Custom category
    Custom(String),
}

/// Message payload with content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    /// Content of the message
    pub content: Vec<u8>,
    /// Content type (MIME type)
    pub content_type: String,
    /// Additional headers
    pub headers: HashMap<String, String>,
}

impl MessagePayload {
    /// Create a new message payload
    pub fn new(content: impl AsRef<[u8]>, content_type: String) -> Self {
        MessagePayload {
            content: content.as_ref().to_vec(),
            content_type,
            headers: HashMap::new(),
        }
    }

    /// Get the content as a UTF-8 string if possible
    pub fn as_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.content)
            .map_err(|e| Error::DecodingFailed(e.to_string()))
    }

    /// Add a header to the payload
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// Message delivery options
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeliveryOptions {
    /// Message priority
    pub priority: MessagePriority,
    /// Timeout for delivery (in seconds)
    pub timeout_seconds: Option<u64>,
    /// Whether acknowledgment is required
    pub require_ack: bool,
    /// Maximum retry count
    pub max_retries: Option<u32>,
    /// Retry delay (in seconds)
    pub retry_delay_seconds: Option<u64>,
    /// Whether to deliver to multiple recipients
    pub broadcast: bool,
    /// Whether to persist the message
    pub persistent: bool,
}

impl Default for DeliveryOptions {
    fn default() -> Self {
        DeliveryOptions {
            priority: MessagePriority::Normal,
            timeout_seconds: Some(30),
            require_ack: true,
            max_retries: Some(3),
            retry_delay_seconds: Some(5),
            broadcast: false,
            persistent: false,
        }
    }
}

/// Content data for message ID generation
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MessageContentData {
    /// Sender ID
    pub sender: String,
    
    /// Recipients (hashed together)
    pub recipients_hash: String,
    
    /// Content hash of the message payload
    pub payload_hash: String,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Random nonce for uniqueness
    pub nonce: [u8; 8],
}

impl ContentAddressed for MessageContentData {
    fn content_hash(&self) -> Result<CryptoContentId> {
        let bytes = self.to_bytes()?;
        Ok(CryptoContentId::from_bytes(&bytes)?)
    }
    
    fn verify(&self, content_id: &CryptoContentId) -> Result<bool> {
        let calculated_id = self.content_hash()?;
        Ok(calculated_id == *content_id)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let bytes = borsh::to_vec(self)
            .map_err(|e| Error::Serialization(format!("Failed to serialize MessageContentData: {}", e)))?;
        Ok(bytes)
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        borsh::from_slice(bytes)
            .map_err(|e| Error::Deserialization(format!("Failed to deserialize MessageContentData: {}", e)))
    }
}

/// Message envelope containing a message and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Unique message ID
    pub id: String,
    /// The actor that sent this message
    pub sender: ActorIdBox,
    /// The recipients of this message
    pub recipients: Vec<ActorIdBox>,
    /// Message category
    pub category: MessageCategory,
    /// Message payload
    pub payload: MessagePayload,
    /// When the message was created
    pub timestamp: Timestamp,
    /// Related trace ID for tracking
    pub trace_id: Option<TraceId>,
}

impl MessageEnvelope {
    /// Create a new message envelope
    pub fn new(
        content: impl AsRef<[u8]>,
        content_type: impl Into<String>,
        sender: ActorIdBox,
        recipients: Vec<ActorIdBox>,
        delivery_options: DeliveryOptions,
    ) -> Self {
        let now = Timestamp::now();
        let content_bytes = content.as_ref().to_vec();
        
        // Generate a content-based message ID
        let recipients_str = recipients.iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(",");
            
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
        
        let content_data = MessageContentData {
            sender: sender.to_string(),
            recipients_hash: format!("recipients:{}", ContentHash::new(recipients_str.as_bytes())),
            payload_hash: format!("payload:{}", ContentHash::new(&content_bytes)),
            timestamp: now.to_seconds(),
            nonce,
        };
        
        let message_id = content_data.content_hash()
            .map(|id| id.to_string())
            .unwrap_or_else(|_| format!("msg-error-{}", now.to_seconds()));
        
        let trace_id = TraceId::new();
            
        MessageEnvelope {
            id: message_id,
            sender,
            recipients,
            category: MessageCategory::Normal,
            payload: MessagePayload::new(content_bytes, content_type.into()),
            timestamp: now,
            trace_id: Some(trace_id),
        }
    }
    
    /// Create a new message envelope with a specific message ID
    pub fn with_id(
        message_id: impl Into<String>,
        content: impl AsRef<[u8]>,
        content_type: impl Into<String>,
        sender: ActorIdBox,
        recipients: Vec<ActorIdBox>,
        delivery_options: DeliveryOptions,
    ) -> Self {
        let mut envelope = Self::new(content, content_type, sender, recipients, delivery_options);
        envelope.id = message_id.into();
        envelope
    }
    
    /// Set the topic for this message
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.category = MessageCategory::Topic(topic.into());
        self
    }
    
    /// Set the trace ID for this message
    pub fn with_trace(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
    
    /// Add a header to this message
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.payload.headers.insert(key.into(), value.into());
        self
    }
    
    /// Check if this message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.timestamp.to_seconds() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
                
            now > expires_at
        } else {
            false
        }
    }
    
    /// Get a content ID for this message
    pub fn content_id(&self) -> ContentId {
        let hash_str = format!(
            "{}:{}:{}",
            self.id,
            self.timestamp.to_seconds(),
            ContentHash::new(&self.payload.content),
        );
        
        let hash = ContentHash::new(&hash_str);
        ContentId::new(hash, "actor-message")
    }
}

/// Message delivery status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeliveryStatus {
    /// Message has been sent
    Sent,
    /// Message has been delivered
    Delivered,
    /// Message delivery failed
    Failed(String),
    /// Message timed out
    TimedOut,
    /// Message was acknowledged
    Acknowledged,
    /// Message was rejected
    Rejected(String),
}

/// Message handler trait for processing messages
#[async_trait]
pub trait MessageHandler: Send + Sync + Debug {
    /// Handle a message
    async fn handle(&self, message: MessageEnvelope) -> Result<()>;
    
    /// Check if this handler can handle a message
    fn can_handle(&self, message: &MessageEnvelope) -> bool;
}

/// A subscription to a topic
#[derive(Debug)]
pub struct Subscription {
    /// Subscriber actor ID
    pub subscriber_id: ActorIdBox,
    /// Topic being subscribed to
    pub topic: String,
    /// When this subscription was created
    pub created_at: u64,
    /// When this subscription expires (if applicable)
    pub expires_at: Option<u64>,
    /// Message sender for delivering messages
    pub sender: mpsc::Sender<MessageEnvelope>,
}

impl Subscription {
    /// Create a new subscription
    pub fn new(
        subscriber_id: ActorIdBox,
        topic: impl Into<String>,
        sender: mpsc::Sender<MessageEnvelope>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Subscription {
            subscriber_id,
            topic: topic.into(),
            created_at: now,
            expires_at: None,
            sender,
        }
    }
    
    /// Set the expiration for this subscription
    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Check if this subscription has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
                
            now > expires_at
        } else {
            false
        }
    }
    
    /// Send a message through this subscription
    pub async fn send(&self, message: MessageEnvelope) -> Result<()> {
        // Skip if subscription has expired
        if self.is_expired() {
            return Err(Error::SubscriptionExpired);
        }
        
        // Try to send the message with a timeout
        let send_result = timeout(
            Duration::from_secs(5),
            self.sender.send(message),
        ).await;
        
        match send_result {
                Ok(Ok(_)) => Ok(()),
            Ok(Err(_)) => Err(Error::ChannelClosed),
            Err(_) => Err(Error::Timeout("message delivery".into())),
        }
    }
}

/// Communication system for managing actor messaging
#[derive(Debug)]
pub struct CommunicationSystem {
    /// Message handlers by actor ID
    handlers: RwLock<HashMap<ActorIdBox, Arc<dyn MessageHandler>>>,
    /// Subscriptions by topic
    subscriptions: RwLock<HashMap<String, Vec<Subscription>>>,
    /// Delivery status by message ID
    delivery_status: RwLock<HashMap<String, DeliveryStatus>>,
    /// Message channels by actor ID
    channels: RwLock<HashMap<ActorIdBox, mpsc::Sender<MessageEnvelope>>>,
}

impl CommunicationSystem {
    /// Create a new communication system
    pub fn new() -> Self {
        CommunicationSystem {
            handlers: RwLock::new(HashMap::new()),
            subscriptions: RwLock::new(HashMap::new()),
            delivery_status: RwLock::new(HashMap::new()),
            channels: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a message handler for an actor
    pub fn register_handler(&self, actor_id: ActorIdBox, handler: Arc<dyn MessageHandler>) -> Result<()> {
        let mut handlers = self.handlers.write().map_err(|_| Error::LockPoisoned)?;
        handlers.insert(actor_id, handler);
        Ok(())
    }
    
    /// Create a message channel for an actor
    pub fn create_channel(&self, actor_id: ActorIdBox, buffer_size: usize) -> Result<mpsc::Receiver<MessageEnvelope>> {
        let (tx, rx) = mpsc::channel(buffer_size);
        
        let mut channels = self.channels.write().map_err(|_| Error::LockPoisoned)?;
        channels.insert(actor_id, tx);
        
        Ok(rx)
    }
    
    /// Get the message channel for an actor
    pub fn get_channel(&self, actor_id: &ActorIdBox) -> Result<Option<mpsc::Sender<MessageEnvelope>>> {
        let channels = self.channels.read().map_err(|_| Error::LockPoisoned)?;
        Ok(channels.get(actor_id).cloned())
    }
    
    /// Subscribe an actor to a topic
    pub fn subscribe(
        &self, 
        subscriber_id: ActorIdBox, 
        topic: impl Into<String>,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<MessageEnvelope>> {
        let topic = topic.into();
        let (tx, rx) = mpsc::channel(buffer_size);
        
        let subscription = Subscription::new(subscriber_id, topic.clone(), tx);
        
        let mut subscriptions = self.subscriptions.write().map_err(|_| Error::LockPoisoned)?;
        
        if let Some(subs) = subscriptions.get_mut(&topic) {
            // Add to existing topic
            subs.push(subscription);
        } else {
            // Create new topic
            subscriptions.insert(topic, vec![subscription]);
        }
        
        Ok(rx)
    }
    
    /// Unsubscribe an actor from a topic
    pub fn unsubscribe(&self, subscriber_id: &ActorIdBox, topic: &str) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().map_err(|_| Error::LockPoisoned)?;
        
        if let Some(subs) = subscriptions.get_mut(topic) {
            // Remove all subscriptions for this actor to this topic
            subs.retain(|sub| &sub.subscriber_id != subscriber_id);
            
            // If no more subscriptions, remove the topic
            if subs.is_empty() {
                subscriptions.remove(topic);
            }
            
                Ok(())
        } else {
            Err(Error::TopicNotFound(topic.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_message_envelope() {
        let sender = ActorIdBox::from("sender");
        let recipient = ActorIdBox::from("recipient");
        
        let message = MessageEnvelope::new(
            "Test content",
            "text/plain",
            sender.clone(),
            vec![recipient.clone()],
            DeliveryOptions::default(),
        );
        
        assert_eq!(message.sender, sender);
        assert_eq!(message.recipients.len(), 1);
        assert_eq!(message.recipients[0], recipient);
        assert_eq!(message.category, MessageCategory::Normal);
        assert_eq!(message.payload.content_type, "text/plain");
        assert!(message.trace_id.is_some());
        
        // Test with topic
        let topic_message = message.clone().with_topic("test-topic");
        assert!(matches!(topic_message.category, MessageCategory::Topic(ref t) if t == "test-topic"));
        
        // Test content ID
        let content_id = message.content_id();
        assert!(content_id.to_string().contains("actor-message"));
    }
} 