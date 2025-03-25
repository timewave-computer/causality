// Actor mailbox implementation
// Original file: src/actor/messaging/mailbox.rs

//! Actor mailbox system
//!
//! This module provides a mailbox system for actors to receive and process messages
//! asynchronously, handling priorities, delivery guarantees, and dead-letter queues.

use std::collections::{HashMap, BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};
use async_trait::async_trait;

use causality_types::{Error, Result};
use causality_types::Timestamp;
use super::{Message, MessageId, MessagePriority, MessageHandler};
use causality_core::ActorId;

/// Message delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    /// Message was delivered successfully
    Delivered,
    /// Delivery was attempted but failed
    Failed,
    /// Message was queued for delivery
    Queued,
    /// Message was rejected by the recipient
    Rejected,
    /// Message timed out waiting for delivery
    TimedOut,
}

/// Mailbox configuration options
#[derive(Debug, Clone)]
pub struct MailboxConfig {
    /// Maximum number of messages in the mailbox
    pub capacity: usize,
    /// Whether to drop older messages when mailbox is full
    pub drop_oldest_when_full: bool,
    /// Whether to sort messages by priority
    pub priority_enabled: bool,
    /// Whether to persist messages
    pub persistence_enabled: bool,
    /// Maximum time to keep a message in the mailbox
    pub message_ttl_seconds: u64,
    /// Whether to enable dead letter handling
    pub dead_letter_enabled: bool,
}

impl Default for MailboxConfig {
    fn default() -> Self {
        MailboxConfig {
            capacity: 1000,
            drop_oldest_when_full: true,
            priority_enabled: true,
            persistence_enabled: false,
            message_ttl_seconds: 3600, // 1 hour
            dead_letter_enabled: true,
        }
    }
}

/// Actor mailbox for receiving and processing messages
pub struct ActorMailbox {
    /// Actor ID this mailbox belongs to
    actor_id: ActorId,
    /// Priority queues for messages
    priority_queues: Arc<Mutex<BTreeMap<MessagePriority, VecDeque<Message>>>>,
    /// Mailbox configuration
    config: MailboxConfig,
    /// Message handler
    handler: Arc<dyn MessageHandler>,
    /// Channel sender for submitting messages
    tx: mpsc::Sender<Message>,
    /// Channel receiver for processing messages
    rx: Arc<Mutex<Option<mpsc::Receiver<Message>>>>,
    /// Dead letter queue for failed messages
    dead_letters: Arc<Mutex<VecDeque<(Message, String)>>>,
}

impl ActorMailbox {
    /// Create a new actor mailbox
    pub fn new(
        actor_id: ActorId,
        handler: Arc<dyn MessageHandler>,
        config: MailboxConfig,
    ) -> Self {
        let (tx, rx) = mpsc::channel(config.capacity);
        
        let mut priority_queues = BTreeMap::new();
        priority_queues.insert(MessagePriority::Low, VecDeque::new());
        priority_queues.insert(MessagePriority::Normal, VecDeque::new());
        priority_queues.insert(MessagePriority::High, VecDeque::new());
        priority_queues.insert(MessagePriority::Critical, VecDeque::new());
        
        ActorMailbox {
            actor_id,
            priority_queues: Arc::new(Mutex::new(priority_queues)),
            config,
            handler,
            tx,
            rx: Arc::new(Mutex::new(Some(rx))),
            dead_letters: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    /// Get the actor ID
    pub fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }
    
    /// Get a sender for this mailbox
    pub fn sender(&self) -> mpsc::Sender<Message> {
        self.tx.clone()
    }
    
    /// Send a message to this mailbox
    pub async fn send(&self, message: Message) -> Result<DeliveryStatus> {
        if message.recipient != self.actor_id {
            return Err(Error::InvalidInput(
                format!("Message recipient ({}) does not match mailbox actor ({})",
                    message.recipient, self.actor_id)
            ));
        }
        
        // Check if message has expired
        if message.is_expired() {
            return Ok(DeliveryStatus::Rejected);
        }
        
        // Try to send via channel first for direct delivery
        match self.tx.try_send(message.clone()) {
            Ok(_) => return Ok(DeliveryStatus::Delivered),
            Err(mpsc::error::TrySendError::Full(_)) => {
                // Channel is full, use priority queue
            },
            Err(mpsc::error::TrySendError::Closed(_)) => {
                return Err(Error::SendError("Mailbox channel is closed".to_string()));
            }
        }
        
        // Use priority queue as fallback
        if self.config.priority_enabled {
            let mut queues = self.priority_queues.lock().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            
            let queue = queues.entry(message.priority).or_insert_with(VecDeque::new);
            
            if queue.len() >= self.config.capacity {
                if self.config.drop_oldest_when_full {
                    queue.pop_front(); // Remove oldest message
                } else {
                    return Ok(DeliveryStatus::Rejected);
                }
            }
            
            queue.push_back(message);
            Ok(DeliveryStatus::Queued)
        } else {
            // Try async send with timeout if not using priority queue
            match timeout(Duration::from_millis(100), self.tx.send(message.clone())).await {
                Ok(Ok(_)) => Ok(DeliveryStatus::Delivered),
                Ok(Err(_)) => Err(Error::SendError("Failed to send message".to_string())),
                Err(_) => {
                    // Send timed out, add to dead letter queue if enabled
                    if self.config.dead_letter_enabled {
                        let mut dead_letters = self.dead_letters.lock().map_err(|_| 
                            Error::InternalError("Failed to acquire lock".to_string()))?;
                        dead_letters.push_back((message, "Send timed out".to_string()));
                    }
                    Ok(DeliveryStatus::TimedOut)
                }
            }
        }
    }
    
    /// Process a message with the handler
    async fn process_message(&self, message: Message) -> Result<Option<Message>> {
        // Check if message has expired
        if message.is_expired() {
            if self.config.dead_letter_enabled {
                let mut dead_letters = self.dead_letters.lock().map_err(|_| 
                    Error::InternalError("Failed to acquire lock".to_string()))?;
                dead_letters.push_back((message, "Message expired".to_string()));
            }
            return Ok(None);
        }
        
        // Process with handler
        match self.handler.handle_message(message.clone()).await {
            Ok(response) => Ok(response),
            Err(e) => {
                // Add to dead letter queue if enabled
                if self.config.dead_letter_enabled {
                    let mut dead_letters = self.dead_letters.lock().map_err(|_| 
                        Error::InternalError("Failed to acquire lock".to_string()))?;
                    dead_letters.push_back((message, format!("Handler error: {}", e)));
                }
                Err(e)
            }
        }
    }
    
    /// Start processing messages
    pub async fn start_processing(&self) -> Result<()> {
        let rx_option = {
            let mut rx_guard = self.rx.lock().map_err(|_| 
                Error::InternalError("Failed to acquire lock".to_string()))?;
            rx_guard.take()
        };
        
        let rx = match rx_option {
            Some(rx) => rx,
            None => return Err(Error::InvalidState("Receiver already taken".to_string())),
        };
        
        let priority_queues = self.priority_queues.clone();
        let handler = self.handler.clone();
        let actor_id = self.actor_id.clone();
        let dead_letters = self.dead_letters.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut rx = rx;
            
            loop {
                // First check priority queues
                let message = {
                    let mut queues = priority_queues.lock().unwrap();
                    
                    // Check critical messages first
                    let mut message = None;
                    for priority in [
                        MessagePriority::Critical,
                        MessagePriority::High,
                        MessagePriority::Normal,
                        MessagePriority::Low,
                    ].iter() {
                        if let Some(queue) = queues.get_mut(priority) {
                            if let Some(msg) = queue.pop_front() {
                                message = Some(msg);
                                break;
                            }
                        }
                    }
                    message
                };
                
                // Process priority message if found
                if let Some(message) = message {
                    if let Err(e) = Self::handle_message(
                        &handler, message.clone(), &dead_letters, config.dead_letter_enabled
                    ).await {
                        eprintln!("Error handling message for {}: {}", actor_id, e);
                    }
                    continue;
                }
                
                // Then check channel for new messages
                match rx.recv().await {
                    Some(message) => {
                        if let Err(e) = Self::handle_message(
                            &handler, message.clone(), &dead_letters, config.dead_letter_enabled
                        ).await {
                            eprintln!("Error handling message for {}: {}", actor_id, e);
                        }
                    },
                    None => {
                        // Channel closed, exit loop
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Helper to handle a message and add to dead letter queue on error
    async fn handle_message(
        handler: &Arc<dyn MessageHandler>,
        message: Message,
        dead_letters: &Arc<Mutex<VecDeque<(Message, String)>>>,
        dead_letter_enabled: bool,
    ) -> Result<()> {
        // Check if message has expired
        if message.is_expired() {
            if dead_letter_enabled {
                let mut dl = dead_letters.lock().map_err(|_| 
                    Error::InternalError("Failed to acquire lock".to_string()))?;
                dl.push_back((message, "Message expired".to_string()));
            }
            return Ok(());
        }
        
        // Process with handler
        match handler.handle_message(message.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Add to dead letter queue if enabled
                if dead_letter_enabled {
                    let mut dl = dead_letters.lock().map_err(|_| 
                        Error::InternalError("Failed to acquire lock".to_string()))?;
                    dl.push_back((message, format!("Handler error: {}", e)));
                }
                Err(e)
            }
        }
    }
    
    /// Get messages from the dead letter queue
    pub fn get_dead_letters(&self) -> Result<Vec<(Message, String)>> {
        let dead_letters = self.dead_letters.lock().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        Ok(dead_letters.iter().cloned().collect())
    }
    
    /// Clear the dead letter queue
    pub fn clear_dead_letters(&self) -> Result<()> {
        let mut dead_letters = self.dead_letters.lock().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        dead_letters.clear();
        Ok(())
    }
}

/// Mailbox system that manages multiple actor mailboxes
pub struct MailboxSystem {
    /// Mailboxes by actor ID
    mailboxes: Arc<RwLock<HashMap<ActorId, Arc<ActorMailbox>>>>,
    /// Default mailbox configuration
    default_config: MailboxConfig,
}

impl MailboxSystem {
    /// Create a new mailbox system
    pub fn new() -> Self {
        MailboxSystem {
            mailboxes: Arc::new(RwLock::new(HashMap::new())),
            default_config: MailboxConfig::default(),
        }
    }
    
    /// Set the default mailbox configuration
    pub fn with_default_config(mut self, config: MailboxConfig) -> Self {
        self.default_config = config;
        self
    }
    
    /// Register a mailbox for an actor
    pub fn register_mailbox(
        &self,
        actor_id: ActorId,
        handler: Arc<dyn MessageHandler>,
        config: Option<MailboxConfig>,
    ) -> Result<Arc<ActorMailbox>> {
        let config = config.unwrap_or_else(|| self.default_config.clone());
        let mailbox = Arc::new(ActorMailbox::new(actor_id.clone(), handler, config));
        
        let mut mailboxes = self.mailboxes.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        mailboxes.insert(actor_id.clone(), mailbox.clone());
        Ok(mailbox)
    }
    
    /// Unregister a mailbox
    pub fn unregister_mailbox(&self, actor_id: &ActorId) -> Result<()> {
        let mut mailboxes = self.mailboxes.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        mailboxes.remove(actor_id);
        Ok(())
    }
    
    /// Get a mailbox for an actor
    pub fn get_mailbox(&self, actor_id: &ActorId) -> Result<Arc<ActorMailbox>> {
        let mailboxes = self.mailboxes.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        match mailboxes.get(actor_id) {
            Some(mailbox) => Ok(mailbox.clone()),
            None => Err(Error::NotFound(format!("Mailbox not found for actor {}", actor_id))),
        }
    }
    
    /// Send a message to an actor
    pub async fn send_message(&self, message: Message) -> Result<DeliveryStatus> {
        let mailboxes = self.mailboxes.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        match mailboxes.get(&message.recipient) {
            Some(mailbox) => mailbox.send(message).await,
            None => {
                // TODO: Consider a global dead letter queue for undeliverable messages
                Ok(DeliveryStatus::Failed)
            },
        }
    }
    
    /// Start processing messages for all mailboxes
    pub async fn start_all(&self) -> Result<()> {
        let mailboxes = self.mailboxes.read().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        for mailbox in mailboxes.values() {
            mailbox.start_processing().await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_core::messaging::{MessageCategory, MessagePayload};
    
    struct TestHandler;
    
    #[async_trait]
    impl MessageHandler for TestHandler {
        async fn handle_message(&self, message: Message) -> Result<Option<Message>> {
            // Echo the message back
            if let Some(sender) = &message.sender {
                let response = Message::reply_to(
                    &message,
                    message.recipient.clone(),
                    MessagePayload::Text("Echo: Test handled".to_string()),
                );
                Ok(Some(response))
            } else {
                Ok(None)
            }
        }
        
        fn supported_categories(&self) -> Vec<MessageCategory> {
            vec![
                MessageCategory::Command,
                MessageCategory::Query,
                MessageCategory::Event,
            ]
        }
    }
    
    #[tokio::test]
    async fn test_mailbox_send_receive() {
        let actor_id = ActorId::new("test-actor");
        let handler = Arc::new(TestHandler);
        
        let config = MailboxConfig {
            capacity: 10,
            ..MailboxConfig::default()
        };
        
        let mailbox = ActorMailbox::new(actor_id.clone(), handler, config);
        
        // Send a message
        let sender_id = ActorId::new("sender");
        let message = Message::new(
            Some(sender_id.clone()),
            actor_id.clone(),
            MessageCategory::Command,
            MessagePayload::Text("Test message".to_string()),
        );
        
        let status = mailbox.send(message).await.unwrap();
        assert!(matches!(status, DeliveryStatus::Delivered | DeliveryStatus::Queued));
    }
    
    #[tokio::test]
    async fn test_mailbox_system() {
        let system = MailboxSystem::new();
        
        let actor1_id = ActorId::new("actor1");
        let actor2_id = ActorId::new("actor2");
        
        let handler1 = Arc::new(TestHandler);
        let handler2 = Arc::new(TestHandler);
        
        // Register mailboxes
        let mailbox1 = system.register_mailbox(actor1_id.clone(), handler1, None).unwrap();
        let mailbox2 = system.register_mailbox(actor2_id.clone(), handler2, None).unwrap();
        
        // Start processing
        mailbox1.start_processing().await.unwrap();
        mailbox2.start_processing().await.unwrap();
        
        // Send a message
        let message = Message::new(
            Some(actor1_id.clone()),
            actor2_id.clone(),
            MessageCategory::Query,
            MessagePayload::Text("Test query".to_string()),
        );
        
        let status = system.send_message(message).await.unwrap();
        assert!(matches!(status, DeliveryStatus::Delivered | DeliveryStatus::Queued));
        
        // Should be able to get the mailbox
        let mailbox = system.get_mailbox(&actor2_id).unwrap();
        assert_eq!(mailbox.actor_id(), &actor2_id);
        
        // Unregister a mailbox
        system.unregister_mailbox(&actor1_id).unwrap();
        assert!(system.get_mailbox(&actor1_id).is_err());
    }
} 