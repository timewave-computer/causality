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
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::types::{ContentId, ContentHash, TraceId};
use crate::actor::{ActorId, ActorCapability};

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

/// Message envelope containing a message and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Unique message ID
    pub message_id: String,
    /// Message content
    pub content: Vec<u8>,
    /// Content type
    pub content_type: String,
    /// Sender actor ID
    pub sender: ActorId,
    /// Recipient actor ID(s)
    pub recipients: Vec<ActorId>,
    /// Topic (if applicable)
    pub topic: Option<String>,
    /// Trace ID for correlation
    pub trace_id: TraceId,
    /// Timestamp when the message was sent
    pub sent_at: u64,
    /// Timestamp when the message expires
    pub expires_at: Option<u64>,
    /// Delivery options
    pub delivery_options: DeliveryOptions,
    /// Headers for additional metadata
    pub headers: HashMap<String, String>,
}

impl MessageEnvelope {
    /// Create a new message envelope
    pub fn new(
        content: impl AsRef<[u8]>,
        content_type: impl Into<String>,
        sender: ActorId,
        recipients: Vec<ActorId>,
        delivery_options: DeliveryOptions,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let message_id = Uuid::new_v4().to_string();
        let trace_id = TraceId::new();
        
        // Calculate expiration if timeout is set
        let expires_at = delivery_options.timeout_seconds
            .map(|timeout| now + timeout);
            
        MessageEnvelope {
            message_id,
            content: content.as_ref().to_vec(),
            content_type: content_type.into(),
            sender,
            recipients,
            topic: None,
            trace_id,
            sent_at: now,
            expires_at,
            delivery_options,
            headers: HashMap::new(),
        }
    }
    
    /// Create a new message envelope with a specific message ID
    pub fn with_id(
        message_id: impl Into<String>,
        content: impl AsRef<[u8]>,
        content_type: impl Into<String>,
        sender: ActorId,
        recipients: Vec<ActorId>,
        delivery_options: DeliveryOptions,
    ) -> Self {
        let mut envelope = Self::new(content, content_type, sender, recipients, delivery_options);
        envelope.message_id = message_id.into();
        envelope
    }
    
    /// Set the topic for this message
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }
    
    /// Set the trace ID for this message
    pub fn with_trace(mut self, trace_id: TraceId) -> Self {
        self.trace_id = trace_id;
        self
    }
    
    /// Add a header to this message
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    /// Check if this message has expired
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
    
    /// Get a content ID for this message
    pub fn content_id(&self) -> ContentId {
        let hash_str = format!(
            "{}:{}:{}",
            self.message_id,
            self.sent_at,
            ContentHash::new(&self.content),
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
    pub subscriber_id: ActorId,
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
        subscriber_id: ActorId,
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
    
    /// Send a message to this subscriber
    pub async fn send(&self, message: MessageEnvelope) -> Result<()> {
        if self.is_expired() {
            return Err(Error::Expired("Subscription has expired".to_string()));
        }
        
        // Apply timeout if configured in the message
        if let Some(timeout_seconds) = message.delivery_options.timeout_seconds {
            match timeout(
                Duration::from_secs(timeout_seconds),
                self.sender.send(message.clone()),
            ).await {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(_)) => Err(Error::MessageDeliveryFailed(
                    "Failed to send message".to_string(),
                )),
                Err(_) => Err(Error::Timeout(
                    format!("Message delivery timed out after {} seconds", timeout_seconds),
                )),
            }
        } else {
            // No timeout
            self.sender.send(message).await.map_err(|_| {
                Error::MessageDeliveryFailed("Failed to send message".to_string())
            })
        }
    }
}

/// Communication system for actor messaging
#[derive(Debug)]
pub struct CommunicationSystem {
    /// Message handlers by actor ID
    handlers: RwLock<HashMap<ActorId, Arc<dyn MessageHandler>>>,
    /// Subscriptions by topic
    subscriptions: RwLock<HashMap<String, Vec<Subscription>>>,
    /// Delivery status by message ID
    delivery_status: RwLock<HashMap<String, DeliveryStatus>>,
    /// Message channels by actor ID
    channels: RwLock<HashMap<ActorId, mpsc::Sender<MessageEnvelope>>>,
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
    pub fn register_handler(&self, actor_id: ActorId, handler: Arc<dyn MessageHandler>) -> Result<()> {
        let mut handlers = self.handlers.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on handlers".to_string())
        })?;
        
        handlers.insert(actor_id, handler);
        Ok(())
    }
    
    /// Create a channel for an actor
    pub fn create_channel(&self, actor_id: ActorId, buffer_size: usize) -> Result<mpsc::Receiver<MessageEnvelope>> {
        let (tx, rx) = mpsc::channel(buffer_size);
        
        let mut channels = self.channels.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on channels".to_string())
        })?;
        
        channels.insert(actor_id, tx);
        
        Ok(rx)
    }
    
    /// Get a channel for an actor
    pub fn get_channel(&self, actor_id: &ActorId) -> Result<Option<mpsc::Sender<MessageEnvelope>>> {
        let channels = self.channels.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on channels".to_string())
        })?;
        
        Ok(channels.get(actor_id).cloned())
    }
    
    /// Subscribe to a topic
    pub fn subscribe(
        &self, 
        subscriber_id: ActorId, 
        topic: impl Into<String>,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<MessageEnvelope>> {
        let topic_str = topic.into();
        let (tx, rx) = mpsc::channel(buffer_size);
        
        let subscription = Subscription::new(subscriber_id.clone(), topic_str.clone(), tx);
        
        let mut subscriptions = self.subscriptions.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on subscriptions".to_string())
        })?;
        
        let topic_subscriptions = subscriptions.entry(topic_str).or_insert_with(Vec::new);
        
        // Remove any existing subscriptions for this actor to this topic
        topic_subscriptions.retain(|sub| sub.subscriber_id != subscriber_id);
        
        // Add the new subscription
        topic_subscriptions.push(subscription);
        
        Ok(rx)
    }
    
    /// Unsubscribe from a topic
    pub fn unsubscribe(&self, subscriber_id: &ActorId, topic: &str) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on subscriptions".to_string())
        })?;
        
        if let Some(topic_subscriptions) = subscriptions.get_mut(topic) {
            topic_subscriptions.retain(|sub| &sub.subscriber_id != subscriber_id);
            
            // Remove the topic entirely if there are no more subscriptions
            if topic_subscriptions.is_empty() {
                subscriptions.remove(topic);
            }
        }
        
        Ok(())
    }
    
    /// Publish a message to a topic
    pub async fn publish(&self, topic: &str, message: MessageEnvelope) -> Result<()> {
        let subscriptions = self.subscriptions.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on subscriptions".to_string())
        })?;
        
        if let Some(topic_subscriptions) = subscriptions.get(topic) {
            // Filter out expired subscriptions
            let active_subscriptions: Vec<&Subscription> = topic_subscriptions
                .iter()
                .filter(|sub| !sub.is_expired())
                .collect();
                
            if active_subscriptions.is_empty() {
                return Err(Error::NoSubscribers(format!("No active subscribers for topic: {}", topic)));
            }
            
            let mut successfully_delivered = false;
            
            for subscription in active_subscriptions {
                let result = subscription.send(message.clone()).await;
                if result.is_ok() {
                    successfully_delivered = true;
                }
            }
            
            if successfully_delivered {
                self.update_delivery_status(&message.message_id, DeliveryStatus::Sent)?;
                Ok(())
            } else {
                let error_msg = format!("Failed to deliver message to any subscribers for topic: {}", topic);
                self.update_delivery_status(
                    &message.message_id, 
                    DeliveryStatus::Failed(error_msg.clone())
                )?;
                Err(Error::MessageDeliveryFailed(error_msg))
            }
        } else {
            let error_msg = format!("No subscribers for topic: {}", topic);
            self.update_delivery_status(
                &message.message_id, 
                DeliveryStatus::Failed(error_msg.clone())
            )?;
            Err(Error::NoSubscribers(error_msg))
        }
    }
    
    /// Send a message to specific recipients
    pub async fn send(&self, message: MessageEnvelope) -> Result<()> {
        if message.recipients.is_empty() {
            return Err(Error::InvalidArgument("No recipients specified".to_string()));
        }
        
        // Check if the message has expired
        if message.is_expired() {
            let error_msg = format!("Message {} has expired", message.message_id);
            self.update_delivery_status(
                &message.message_id, 
                DeliveryStatus::Failed(error_msg.clone())
            )?;
            return Err(Error::Expired(error_msg));
        }
        
        let channels = self.channels.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on channels".to_string())
        })?;
        
        // Check if we're broadcasting or sending to specific recipients
        if message.delivery_options.broadcast {
            let mut successfully_delivered = false;
            
            for recipient in &message.recipients {
                if let Some(channel) = channels.get(recipient) {
                    let result = channel.send(message.clone()).await;
                    if result.is_ok() {
                        successfully_delivered = true;
                    }
                }
            }
            
            if successfully_delivered {
                self.update_delivery_status(&message.message_id, DeliveryStatus::Sent)?;
                Ok(())
            } else {
                let error_msg = "Failed to deliver message to any recipients".to_string();
                self.update_delivery_status(
                    &message.message_id, 
                    DeliveryStatus::Failed(error_msg.clone())
                )?;
                Err(Error::MessageDeliveryFailed(error_msg))
            }
        } else {
            // We need to deliver to all recipients or fail
            for recipient in &message.recipients {
                if let Some(channel) = channels.get(recipient) {
                    if let Err(e) = channel.send(message.clone()).await {
                        let error_msg = format!(
                            "Failed to deliver message to recipient {}: {}",
                            recipient.as_str(),
                            e
                        );
                        self.update_delivery_status(
                            &message.message_id, 
                            DeliveryStatus::Failed(error_msg.clone())
                        )?;
                        return Err(Error::MessageDeliveryFailed(error_msg));
                    }
                } else {
                    let error_msg = format!("No channel found for recipient: {}", recipient.as_str());
                    self.update_delivery_status(
                        &message.message_id, 
                        DeliveryStatus::Failed(error_msg.clone())
                    )?;
                    return Err(Error::MessageDeliveryFailed(error_msg));
                }
            }
            
            self.update_delivery_status(&message.message_id, DeliveryStatus::Sent)?;
            Ok(())
        }
    }
    
    /// Handle a message
    pub async fn handle_message(&self, message: MessageEnvelope) -> Result<()> {
        let handlers = self.handlers.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on handlers".to_string())
        })?;
        
        for recipient in &message.recipients {
            if let Some(handler) = handlers.get(recipient) {
                if handler.can_handle(&message) {
                    handler.handle(message.clone()).await?;
                    self.update_delivery_status(&message.message_id, DeliveryStatus::Delivered)?;
                } else {
                    let error_msg = format!(
                        "Handler for {} cannot handle message of type {}",
                        recipient.as_str(),
                        message.content_type
                    );
                    self.update_delivery_status(
                        &message.message_id, 
                        DeliveryStatus::Failed(error_msg.clone())
                    )?;
                    return Err(Error::UnsupportedMessageType(error_msg));
                }
            } else {
                let error_msg = format!("No handler found for recipient: {}", recipient.as_str());
                self.update_delivery_status(
                    &message.message_id, 
                    DeliveryStatus::Failed(error_msg.clone())
                )?;
                return Err(Error::MessageDeliveryFailed(error_msg));
            }
        }
        
        Ok(())
    }
    
    /// Acknowledge a message
    pub fn acknowledge_message(&self, message_id: &str) -> Result<()> {
        self.update_delivery_status(message_id, DeliveryStatus::Acknowledged)
    }
    
    /// Reject a message
    pub fn reject_message(&self, message_id: &str, reason: impl Into<String>) -> Result<()> {
        self.update_delivery_status(message_id, DeliveryStatus::Rejected(reason.into()))
    }
    
    /// Get the delivery status of a message
    pub fn get_delivery_status(&self, message_id: &str) -> Result<Option<DeliveryStatus>> {
        let status = self.delivery_status.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on delivery status".to_string())
        })?;
        
        Ok(status.get(message_id).cloned())
    }
    
    /// Update the delivery status of a message
    fn update_delivery_status(&self, message_id: &str, status: DeliveryStatus) -> Result<()> {
        let mut delivery_status = self.delivery_status.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on delivery status".to_string())
        })?;
        
        delivery_status.insert(message_id.to_string(), status);
        
        Ok(())
    }
    
    /// Clean up expired subscriptions
    pub fn cleanup_expired_subscriptions(&self) -> Result<usize> {
        let mut subscriptions = self.subscriptions.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on subscriptions".to_string())
        })?;
        
        let mut cleaned_count = 0;
        let mut topics_to_remove = Vec::new();
        
        for (topic, subs) in subscriptions.iter_mut() {
            let original_count = subs.len();
            subs.retain(|sub| !sub.is_expired());
            
            cleaned_count += original_count - subs.len();
            
            if subs.is_empty() {
                topics_to_remove.push(topic.clone());
            }
        }
        
        // Remove empty topics
        for topic in topics_to_remove {
            subscriptions.remove(&topic);
        }
        
        Ok(cleaned_count)
    }
}

impl Default for CommunicationSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    
    #[tokio::test]
    async fn test_message_envelope() {
        let sender = ActorId::new("sender");
        let recipient = ActorId::new("recipient");
        
        let options = DeliveryOptions {
            priority: MessagePriority::High,
            timeout_seconds: Some(60),
            require_ack: true,
            ..Default::default()
        };
        
        let message = MessageEnvelope::new(
            b"Hello, World!",
            "text/plain",
            sender.clone(),
            vec![recipient.clone()],
            options.clone(),
        );
        
        assert_eq!(message.content, b"Hello, World!");
        assert_eq!(message.content_type, "text/plain");
        assert_eq!(message.sender, sender);
        assert_eq!(message.recipients, vec![recipient]);
        assert_eq!(message.delivery_options.priority, MessagePriority::High);
        assert_eq!(message.delivery_options.timeout_seconds, Some(60));
        assert_eq!(message.delivery_options.require_ack, true);
        
        // Test with topic
        let message_with_topic = message.clone().with_topic("test-topic");
        assert_eq!(message_with_topic.topic, Some("test-topic".to_string()));
        
        // Test with header
        let message_with_header = message.clone().with_header("key", "value");
        assert_eq!(message_with_header.headers.get("key"), Some(&"value".to_string()));
        
        // Test content ID
        let content_id = message.content_id();
        assert_eq!(content_id.content_type, "actor-message");
    }
    
    #[derive(Debug)]
    struct TestMessageHandler {
        actor_id: ActorId,
        received_messages: Mutex<Vec<MessageEnvelope>>,
    }
    
    impl TestMessageHandler {
        fn new(actor_id: impl Into<String>) -> Self {
            TestMessageHandler {
                actor_id: ActorId::new(actor_id),
                received_messages: Mutex::new(Vec::new()),
            }
        }
    }
    
    #[async_trait]
    impl MessageHandler for TestMessageHandler {
        async fn handle(&self, message: MessageEnvelope) -> Result<()> {
            let mut received = self.received_messages.lock().unwrap();
            received.push(message);
            Ok(())
        }
        
        fn can_handle(&self, message: &MessageEnvelope) -> bool {
            message.content_type == "text/plain" || message.content_type == "application/json"
        }
    }
    
    #[tokio::test]
    async fn test_communication_system() -> Result<()> {
        let system = CommunicationSystem::new();
        
        // Create actors
        let sender_id = ActorId::new("sender");
        let recipient_id = ActorId::new("recipient");
        
        // Register handlers
        let recipient_handler = Arc::new(TestMessageHandler::new("recipient"));
        system.register_handler(recipient_id.clone(), recipient_handler.clone())?;
        
        // Create channels
        let _sender_rx = system.create_channel(sender_id.clone(), 10)?;
        let _recipient_rx = system.create_channel(recipient_id.clone(), 10)?;
        
        // Create a message
        let options = DeliveryOptions::default();
        let message = MessageEnvelope::new(
            b"Hello, Recipient!",
            "text/plain",
            sender_id.clone(),
            vec![recipient_id.clone()],
            options,
        );
        
        // Send the message
        system.send(message.clone()).await?;
        
        // Handle the message
        system.handle_message(message.clone()).await?;
        
        // Check if the handler received the message
        let received = recipient_handler.received_messages.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].content, b"Hello, Recipient!");
        
        // Check delivery status
        let status = system.get_delivery_status(&message.message_id)?;
        assert_eq!(status, Some(DeliveryStatus::Delivered));
        
        // Acknowledge the message
        system.acknowledge_message(&message.message_id)?;
        
        let updated_status = system.get_delivery_status(&message.message_id)?;
        assert_eq!(updated_status, Some(DeliveryStatus::Acknowledged));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_pub_sub() -> Result<()> {
        let system = CommunicationSystem::new();
        
        // Create actors
        let publisher_id = ActorId::new("publisher");
        let subscriber1_id = ActorId::new("subscriber1");
        let subscriber2_id = ActorId::new("subscriber2");
        
        // Subscribe to topics
        let mut sub1_rx = system.subscribe(subscriber1_id.clone(), "topic1", 10)?;
        let mut sub2_rx = system.subscribe(subscriber2_id.clone(), "topic1", 10)?;
        
        // Create a message
        let options = DeliveryOptions::default();
        let message = MessageEnvelope::new(
            b"Hello, Subscribers!",
            "text/plain",
            publisher_id,
            vec![],  // Not needed for publish
            options,
        ).with_topic("topic1");
        
        // Publish the message
        system.publish("topic1", message.clone()).await?;
        
        // Check if subscribers received the message
        let received1 = sub1_rx.try_recv().unwrap();
        let received2 = sub2_rx.try_recv().unwrap();
        
        assert_eq!(received1.content, b"Hello, Subscribers!");
        assert_eq!(received2.content, b"Hello, Subscribers!");
        
        // Unsubscribe one subscriber
        system.unsubscribe(&subscriber1_id, "topic1")?;
        
        // Create another message
        let message2 = MessageEnvelope::new(
            b"Hello again!",
            "text/plain",
            publisher_id,
            vec![],
            options,
        ).with_topic("topic1");
        
        // Publish again
        system.publish("topic1", message2.clone()).await?;
        
        // Check that only subscriber2 received it
        assert!(sub1_rx.try_recv().is_err()); // Should be empty
        let received2_again = sub2_rx.try_recv().unwrap();
        assert_eq!(received2_again.content, b"Hello again!");
        
        Ok(())
    }
} 