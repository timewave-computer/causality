// Common messaging patterns for actors
// Original file: src/actor/messaging/patterns.rs

//! Actor messaging patterns
//!
//! This module implements common messaging patterns for actor communication:
//! - Request-Response: Send a request and wait for a response
//! - Publish-Subscribe: Publish messages to topics and subscribe to them

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use async_trait::async_trait;
use borsh::{BorshSerialize, BorshDeserialize};
use std::time::SystemTime;

use causality_types::{Error, Result};
use causality_types::Timestamp;
use super::{Message, MessageId, MessageCategory, MessagePayload, MessageHandler};
use causality_core::ActorId;
use crate::crypto::content_addressed::{ContentAddressed, ContentId};

/// Timeout error for request-response pattern
#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    /// Request timed out
    #[error("Request timed out after {0:?}")]
    Timeout(Duration),
    /// Failed to send request
    #[error("Failed to send request: {0}")]
    SendError(String),
    /// Failed to receive response
    #[error("Failed to receive response: {0}")]
    ReceiveError(String),
    /// Response was rejected
    #[error("Response was rejected: {0}")]
    Rejected(String),
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Request-response pattern implementation
pub struct RequestResponse {
    /// Sender actor ID
    sender: ActorId,
    /// In-flight requests waiting for responses
    pending_requests: Arc<RwLock<HashMap<MessageId, oneshot::Sender<Result<Message>>>>>,
}

impl RequestResponse {
    /// Create a new request-response handler
    pub fn new(sender: ActorId) -> Self {
        RequestResponse {
            sender,
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Send a request and wait for a response
    pub async fn request(
        &self,
        recipient: ActorId,
        category: MessageCategory,
        payload: MessagePayload,
        timeout_duration: Option<Duration>,
    ) -> Result<Message, RequestError> {
        // Create a channel for the response
        let (tx, rx) = oneshot::channel();
        
        // Create the request message
        let message = Message::new(
            Some(self.sender.clone()),
            recipient,
            category,
            payload,
        );
        
        // Store the response channel
        {
            let mut pending = self.pending_requests.write().map_err(|_| 
                RequestError::Internal("Failed to acquire lock".to_string()))?;
            pending.insert(message.id.clone(), tx);
        }
        
        // Send the message
        // Note: In a real implementation, this would use the MailboxSystem to send
        // the message. For simplicity, we'll just assume it's sent.
        
        // Wait for the response with an optional timeout
        match timeout_duration {
            Some(duration) => {
                match timeout(duration, rx).await {
                    Ok(Ok(Ok(response))) => Ok(response),
                    Ok(Ok(Err(e))) => Err(RequestError::Rejected(e.to_string())),
                    Ok(Err(_)) => Err(RequestError::ReceiveError("Response channel closed".to_string())),
                    Err(_) => {
                        // Remove the pending request on timeout
                        let mut pending = self.pending_requests.write().map_err(|_| 
                            RequestError::Internal("Failed to acquire lock".to_string()))?;
                        pending.remove(&message.id);
                        
                        Err(RequestError::Timeout(duration))
                    }
                }
            },
            None => {
                match rx.await {
                    Ok(Ok(response)) => Ok(response),
                    Ok(Err(e)) => Err(RequestError::Rejected(e.to_string())),
                    Err(_) => Err(RequestError::ReceiveError("Response channel closed".to_string())),
                }
            }
        }
    }
    
    /// Handle a response for a pending request
    pub fn handle_response(&self, response: Message) -> Result<bool> {
        // Check if this is a response to a pending request
        if let Some(in_reply_to) = &response.in_reply_to {
            let mut pending = self.pending_requests.write().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            
            if let Some(tx) = pending.remove(in_reply_to) {
                // Send the response to the waiting caller
                let _ = tx.send(Ok(response));
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Clean up expired pending requests
    pub fn cleanup_expired_requests(&self) -> Result<usize> {
        let mut pending = self.pending_requests.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        let count_before = pending.len();
        
        // TODO: In a real implementation, we would track request timestamps
        // and clean up expired ones. For now, this is a placeholder.
        
        Ok(count_before - pending.len())
    }
}

/// Topic for publish-subscribe pattern
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Topic(String);

impl Topic {
    /// Create a new topic
    pub fn new(name: &str) -> Self {
        Topic(name.to_string())
    }
    
    /// Get the topic name
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Content data for subscription ID generation
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SubscriptionContentData {
    /// Topic name
    pub topic_name: String,
    
    /// Creation timestamp
    pub timestamp: u64,
    
    /// Random nonce for uniqueness
    pub nonce: [u8; 8],
}

impl ContentAddressed for SubscriptionContentData {
    fn content_hash(&self) -> Result<ContentId> {
        let bytes = self.to_bytes()?;
        Ok(ContentId::from_bytes(&bytes)?)
    }
    
    fn verify(&self, content_id: &ContentId) -> Result<bool> {
        let calculated_id = self.content_hash()?;
        Ok(calculated_id == *content_id)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let bytes = borsh::to_vec(self)
            .map_err(|e| Error::Serialization(format!("Failed to serialize SubscriptionContentData: {}", e)))?;
        Ok(bytes)
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        borsh::from_slice(bytes)
            .map_err(|e| Error::Deserialization(format!("Failed to deserialize SubscriptionContentData: {}", e)))
    }
}

/// Subscription ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionId(String);

impl SubscriptionId {
    /// Create a new random subscription ID
    pub fn new() -> Self {
        SubscriptionId(self::random_content_id().to_string())
    }
    
    /// Create a new subscription ID for a topic
    pub fn for_topic(topic: &Topic) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
        
        let content_data = SubscriptionContentData {
            topic_name: topic.name().to_string(),
            timestamp: now,
            nonce,
        };
        
        let id = content_data.content_hash()
            .map(|id| id.to_string())
            .unwrap_or_else(|_| format!("error-generating-id-{}", now));
            
        SubscriptionId(id)
    }
}

/// Helper function to generate a random content ID
fn random_content_id() -> ContentId {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
        
    let mut nonce = [0u8; 8];
    getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
    
    let content_data = SubscriptionContentData {
        topic_name: format!("random-{}", now),
        timestamp: now,
        nonce,
    };
    
    content_data.content_hash()
        .unwrap_or_else(|_| ContentId::from_bytes(&nonce.to_vec()).unwrap())
}

/// Subscription information
struct Subscription {
    /// Subscriber actor ID
    subscriber: ActorId,
    /// Topic
    topic: Topic,
    /// Channel to send messages to
    tx: mpsc::Sender<Message>,
    /// When this subscription was created
    created_at: Timestamp,
}

/// Publish-subscribe system implementation
pub struct PubSubSystem {
    /// Topics and their subscribers
    topics: Arc<RwLock<HashMap<Topic, HashSet<SubscriptionId>>>>,
    /// Subscriptions by ID
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Subscription>>>,
}

impl PubSubSystem {
    /// Create a new publish-subscribe system
    pub fn new() -> Self {
        PubSubSystem {
            topics: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Subscribe to a topic
    pub fn subscribe(
        &self,
        subscriber: ActorId,
        topic: Topic,
        buffer_size: usize,
    ) -> Result<(SubscriptionId, mpsc::Receiver<Message>)> {
        let subscription_id = SubscriptionId::for_topic(&topic);
        let (tx, rx) = mpsc::channel(buffer_size);
        
        let subscription = Subscription {
            subscriber: subscriber.clone(),
            topic: topic.clone(),
            tx,
            created_at: Timestamp::now(),
        };
        
        // Add to subscriptions
        {
            let mut subscriptions = self.subscriptions.write().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            subscriptions.insert(subscription_id.clone(), subscription);
        }
        
        // Add to topics
        {
            let mut topics = self.topics.write().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            
            let subscribers = topics.entry(topic).or_insert_with(HashSet::new);
            subscribers.insert(subscription_id.clone());
        }
        
        Ok((subscription_id, rx))
    }
    
    /// Unsubscribe from a topic
    pub fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<bool> {
        // Get the subscription
        let topic = {
            let subscriptions = self.subscriptions.read().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            
            match subscriptions.get(&subscription_id) {
                Some(subscription) => subscription.topic.clone(),
                None => return Ok(false),
            }
        };
        
        // Remove from topics
        {
            let mut topics = self.topics.write().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            
            if let Some(subscribers) = topics.get_mut(&topic) {
                subscribers.remove(&subscription_id);
                
                // Remove topic if no subscribers
                if subscribers.is_empty() {
                    topics.remove(&topic);
                }
            }
        }
        
        // Remove from subscriptions
        {
            let mut subscriptions = self.subscriptions.write().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            subscriptions.remove(&subscription_id);
        }
        
        Ok(true)
    }
    
    /// Publish a message to a topic
    pub async fn publish(
        &self,
        publisher: Option<ActorId>,
        topic: Topic,
        payload: MessagePayload,
    ) -> Result<usize> {
        // Get subscribers
        let subscriber_ids = {
            let topics = self.topics.read().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            
            match topics.get(&topic) {
                Some(subscribers) => subscribers.iter().cloned().collect::<Vec<_>>(),
                None => return Ok(0),
            }
        };
        
        if subscriber_ids.is_empty() {
            return Ok(0);
        }
        
        // Get subscription channels
        let subscriptions = {
            let subscriptions = self.subscriptions.read().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            
            subscriber_ids.iter()
                .filter_map(|id| subscriptions.get(id).map(|s| (id.clone(), s.subscriber.clone(), s.tx.clone())))
                .collect::<Vec<_>>()
        };
        
        let mut sent_count = 0;
        
        // Send the message to each subscriber
        for (subscription_id, subscriber, tx) in subscriptions {
            let message = Message::new(
                publisher.clone(),
                subscriber,
                MessageCategory::Event,
                payload.clone(),
            )
            .with_metadata("topic", topic.name());
            
            // Try to send, ignoring errors (subscribers might be offline)
            if tx.try_send(message).is_ok() {
                sent_count += 1;
            } else {
                // Consider unsubscribing if the channel is full or closed
                let _ = self.unsubscribe(subscription_id);
            }
        }
        
        Ok(sent_count)
    }
    
    /// Get all topics
    pub fn get_topics(&self) -> Result<Vec<Topic>> {
        let topics = self.topics.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        Ok(topics.keys().cloned().collect())
    }
    
    /// Get subscribers for a topic
    pub fn get_subscribers(&self, topic: &Topic) -> Result<Vec<ActorId>> {
        let topics = self.topics.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        let subscriptions = self.subscriptions.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        match topics.get(topic) {
            Some(subscriber_ids) => {
                Ok(subscriber_ids.iter()
                    .filter_map(|id| subscriptions.get(id).map(|s| s.subscriber.clone()))
                    .collect())
            },
            None => Ok(Vec::new()),
        }
    }
    
    /// Clean up dead subscriptions
    pub async fn cleanup_dead_subscriptions(&self) -> Result<usize> {
        // Get subscriptions to check
        let subscriptions_to_check = {
            let subscriptions = self.subscriptions.read().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            
            subscriptions.iter()
                .map(|(id, sub)| (id.clone(), sub.tx.clone()))
                .collect::<Vec<_>>()
        };
        
        let mut removed_count = 0;
        
        // Check each subscription
        for (id, tx) in subscriptions_to_check {
            if tx.is_closed() {
                if self.unsubscribe(id)? {
                    removed_count += 1;
                }
            }
        }
        
        Ok(removed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_request_response() {
        let sender = ActorId::new("sender");
        let recipient = ActorId::new("recipient");
        
        let rr = RequestResponse::new(sender.clone());
        
        // We can't fully test this without integrating with the MailboxSystem,
        // but we can test the response handling
        
        // Simulate a pending request
        let message = Message::new(
            Some(sender.clone()),
            recipient.clone(),
            MessageCategory::Query,
            MessagePayload::Text("Test query".to_string()),
        );
        
        let (tx, rx) = oneshot::channel();
        
        {
            let mut pending = rr.pending_requests.write().unwrap();
            pending.insert(message.id.clone(), tx);
        }
        
        // Simulate a response
        let response = Message::reply_to(
            &message,
            recipient.clone(),
            MessagePayload::Text("Test response".to_string()),
        );
        
        assert!(rr.handle_response(response).unwrap());
        
        // The pending request should now have received a response
        let result = rx.await.unwrap();
        assert!(result.is_ok());
        
        // The message ID should no longer be in pending requests
        let pending = rr.pending_requests.read().unwrap();
        assert!(!pending.contains_key(&message.id));
    }
    
    #[tokio::test]
    async fn test_publish_subscribe() {
        let pubsub = PubSubSystem::new();
        
        let subscriber1 = ActorId::new("subscriber1");
        let subscriber2 = ActorId::new("subscriber2");
        let publisher = ActorId::new("publisher");
        
        let topic = Topic::new("test-topic");
        
        // Subscribe
        let (sub_id1, mut rx1) = pubsub.subscribe(subscriber1, topic.clone(), 10).unwrap();
        let (sub_id2, mut rx2) = pubsub.subscribe(subscriber2, topic.clone(), 10).unwrap();
        
        // Publish
        let count = pubsub.publish(
            Some(publisher),
            topic.clone(),
            MessagePayload::Text("Test message".to_string()),
        ).await.unwrap();
        
        assert_eq!(count, 2);
        
        // Both subscribers should receive the message
        let msg1 = rx1.recv().await.unwrap();
        let msg2 = rx2.recv().await.unwrap();
        
        if let MessagePayload::Text(text1) = &msg1.payload {
            assert_eq!(text1, "Test message");
        } else {
            panic!("Expected Text payload");
        }
        
        if let MessagePayload::Text(text2) = &msg2.payload {
            assert_eq!(text2, "Test message");
        } else {
            panic!("Expected Text payload");
        }
        
        // Unsubscribe one
        assert!(pubsub.unsubscribe(sub_id1).unwrap());
        
        // Publish again
        let count = pubsub.publish(
            Some(publisher),
            topic.clone(),
            MessagePayload::Text("Second message".to_string()),
        ).await.unwrap();
        
        assert_eq!(count, 1);
        
        // Only the second subscriber should receive it
        let msg2 = rx2.recv().await.unwrap();
        
        if let MessagePayload::Text(text2) = &msg2.payload {
            assert_eq!(text2, "Second message");
        } else {
            panic!("Expected Text payload");
        }
        
        // Get topics and subscribers
        let topics = pubsub.get_topics().unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0], topic);
        
        let subscribers = pubsub.get_subscribers(&topic).unwrap();
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], subscriber2);
    }
} 