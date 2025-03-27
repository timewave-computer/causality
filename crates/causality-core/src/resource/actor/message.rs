// Resource actor message system
//
// This module provides the message passing system for resource actors,
// defining message types, routing, and delivery patterns.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::crypto::{ContentId, ContentAddressed};

use super::{ActorError, ActorResult};

/// Actor message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: ContentId,
    
    /// Sender ID
    pub sender: ContentId,
    
    /// Receiver ID
    pub receiver: ContentId,
    
    /// Message type
    pub message_type: String,
    
    /// Message payload
    pub payload: serde_json::Value,
    
    /// Message metadata
    pub metadata: HashMap<String, String>,
    
    /// Message timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Message expiration
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Message {
    /// Create a new message
    pub fn new(
        sender: ContentId,
        receiver: ContentId,
        message_type: &str,
        payload: serde_json::Value,
        metadata: HashMap<String, String>,
    ) -> Self {
        let id = ContentId::new(&format!(
            "{}_{}_{}_{}", 
            sender, 
            receiver, 
            message_type, 
            chrono::Utc::now().timestamp_millis()
        ));
        
        Self {
            id,
            sender,
            receiver,
            message_type: message_type.to_string(),
            payload,
            metadata,
            timestamp: chrono::Utc::now(),
            expires_at: None,
        }
    }
    
    /// Add metadata to the message
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Set expiration on the message
    pub fn with_expiration(mut self, expiration: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expiration);
        self
    }
    
    /// Check if the message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < chrono::Utc::now()
        } else {
            false
        }
    }
}

/// Message handler
#[async_trait]
pub trait MessageHandler: Send + Sync + Debug {
    /// Handle a message
    async fn handle(&self, message: Message) -> ActorResult<()>;
}

/// Message router
#[async_trait]
pub trait MessageRouter: Send + Sync + Debug {
    /// Route a message to its destination
    async fn route(&self, message: Message) -> ActorResult<()>;
    
    /// Register a handler for a specific message type
    async fn register_handler(
        &self,
        message_type: &str,
        handler: Arc<dyn MessageHandler>,
    ) -> ActorResult<()>;
    
    /// Unregister a handler for a specific message type
    async fn unregister_handler(&self, message_type: &str) -> ActorResult<()>;
}

/// Message queue
#[async_trait]
pub trait MessageQueue: Send + Sync + Debug {
    /// Enqueue a message
    async fn enqueue(&self, message: Message) -> ActorResult<()>;
    
    /// Dequeue a message
    async fn dequeue(&self) -> ActorResult<Option<Message>>;
    
    /// Get the number of messages in the queue
    async fn len(&self) -> ActorResult<usize>;
    
    /// Check if the queue is empty
    async fn is_empty(&self) -> ActorResult<bool>;
}

/// Message channel
#[async_trait]
pub trait MessageChannel: Send + Sync + Debug {
    /// Send a message through the channel
    async fn send(&self, message: Message) -> ActorResult<()>;
    
    /// Receive a message from the channel
    async fn receive(&self) -> ActorResult<Message>;
    
    /// Check if the channel is closed
    async fn is_closed(&self) -> ActorResult<bool>;
    
    /// Close the channel
    async fn close(&self) -> ActorResult<()>;
}

/// Message broker
#[async_trait]
pub trait MessageBroker: Send + Sync + Debug {
    /// Publish a message to a topic
    async fn publish(&self, topic: &str, message: Message) -> ActorResult<()>;
    
    /// Subscribe to a topic
    async fn subscribe(&self, topic: &str) -> ActorResult<()>;
    
    /// Unsubscribe from a topic
    async fn unsubscribe(&self, topic: &str) -> ActorResult<()>;
    
    /// Receive a message from a subscribed topic
    async fn receive(&self, topic: &str) -> ActorResult<Message>;
} 