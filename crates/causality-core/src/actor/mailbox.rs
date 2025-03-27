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
use std::fmt::{self, Debug};
use std::error::Error;

use causality_types::{Error, Result};
use causality_types::Timestamp;
use super::messaging::{Message, MessageId, MessagePriority, MessageCategory, MessagePayload, TraceId};
use super::MessageHandler;
use super::types::{ActorIdBox, ContentAddressedActorId};

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

/// Capacity of a mailbox
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailboxCapacity {
    /// Unbounded mailbox
    Unbounded,
    
    /// Bounded mailbox with maximum capacity
    Bounded(usize),
}

impl MailboxCapacity {
    /// Check if the mailbox is bounded
    pub fn is_bounded(&self) -> bool {
        matches!(self, MailboxCapacity::Bounded(_))
    }
    
    /// Get the capacity if bounded
    pub fn capacity(&self) -> Option<usize> {
        match self {
            MailboxCapacity::Bounded(capacity) => Some(*capacity),
            MailboxCapacity::Unbounded => None,
        }
    }
}

/// Error type for mailbox operations
#[derive(Debug, thiserror::Error)]
pub enum MailboxError {
    /// The mailbox is full
    #[error("mailbox is full (capacity: {capacity})")]
    Full {
        /// The capacity of the mailbox
        capacity: usize,
    },
    
    /// The mailbox is closed
    #[error("mailbox is closed")]
    Closed,
    
    /// No message available
    #[error("no message available")]
    Empty,
    
    /// Timeout occurred
    #[error("timeout occurred")]
    Timeout,
    
    /// Message expired
    #[error("message expired")]
    Expired,
    
    /// Internal error
    #[error("internal error: {0}")]
    Internal(#[from] Box<dyn Error + Send + Sync>),
}

/// Statistics about a mailbox
#[derive(Debug, Clone, Copy)]
pub struct MailboxStats {
    /// The number of messages in the mailbox
    pub message_count: usize,
    
    /// The capacity of the mailbox
    pub capacity: Option<usize>,
    
    /// The number of messages processed
    pub processed_count: usize,
    
    /// The number of dropped messages
    pub dropped_count: usize,
    
    /// The number of expired messages
    pub expired_count: usize,
}

/// A trait for actor mailboxes
pub trait Mailbox<M: Message>: Send + Sync + 'static {
    /// Send a message to the mailbox
    fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), MailboxError>;
    
    /// Receive a message from the mailbox
    fn receive(&self) -> Result<MessageEnvelope<M>, MailboxError>;
    
    /// Try to receive a message without blocking
    fn try_receive(&self) -> Result<Option<MessageEnvelope<M>>, MailboxError>;
    
    /// Receive a message with a timeout
    fn receive_timeout(&self, timeout: Duration) -> Result<MessageEnvelope<M>, MailboxError>;
    
    /// Check if the mailbox is empty
    fn is_empty(&self) -> bool;
    
    /// Check if the mailbox is full
    fn is_full(&self) -> bool;
    
    /// Get the number of messages in the mailbox
    fn len(&self) -> usize;
    
    /// Get statistics about the mailbox
    fn stats(&self) -> MailboxStats;
    
    /// Close the mailbox
    fn close(&self);
    
    /// Check if the mailbox is closed
    fn is_closed(&self) -> bool;
    
    /// Clear the mailbox
    fn clear(&self);
}

/// A simple mailbox implementation using a mutex and VecDeque
pub struct SimpleMailbox<M: Message> {
    /// The inner state of the mailbox
    inner: Arc<Mutex<SimpleMailboxState<M>>>,
}

/// The inner state of a simple mailbox
struct SimpleMailboxState<M: Message> {
    /// The queue of messages
    queue: VecDeque<MessageEnvelope<M>>,
    
    /// The capacity of the mailbox
    capacity: MailboxCapacity,
    
    /// Whether the mailbox is closed
    closed: bool,
    
    /// Statistics about the mailbox
    stats: MailboxStats,
}

impl<M: Message> SimpleMailbox<M> {
    /// Create a new simple mailbox
    pub fn new(capacity: MailboxCapacity) -> Self {
        let cap_option = match capacity {
            MailboxCapacity::Bounded(cap) => Some(cap),
            MailboxCapacity::Unbounded => None,
        };
        
        Self {
            inner: Arc::new(Mutex::new(SimpleMailboxState {
                queue: VecDeque::new(),
                capacity,
                closed: false,
                stats: MailboxStats {
                    message_count: 0,
                    capacity: cap_option,
                    processed_count: 0,
                    dropped_count: 0,
                    expired_count: 0,
                },
            })),
        }
    }
}

impl<M: Message> Mailbox<M> for SimpleMailbox<M> {
    fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), MailboxError> {
        let mut inner = self.inner.lock().unwrap();
        
        if inner.closed {
            return Err(MailboxError::Closed);
        }
        
        if envelope.is_expired() {
            inner.stats.expired_count += 1;
            return Err(MailboxError::Expired);
        }
        
        match inner.capacity {
            MailboxCapacity::Bounded(cap) if inner.queue.len() >= cap => {
                inner.stats.dropped_count += 1;
                Err(MailboxError::Full { capacity: cap })
            }
            _ => {
                inner.queue.push_back(envelope);
                inner.stats.message_count = inner.queue.len();
                Ok(())
            }
        }
    }
    
    fn receive(&self) -> Result<MessageEnvelope<M>, MailboxError> {
        loop {
            let mut inner = self.inner.lock().unwrap();
            
            if inner.closed {
                return Err(MailboxError::Closed);
            }
            
            if let Some(envelope) = inner.queue.pop_front() {
                inner.stats.message_count = inner.queue.len();
                inner.stats.processed_count += 1;
                
                if envelope.is_expired() {
                    inner.stats.expired_count += 1;
                    continue;
                }
                
                return Ok(envelope);
            }
            
            return Err(MailboxError::Empty);
        }
    }
    
    fn try_receive(&self) -> Result<Option<MessageEnvelope<M>>, MailboxError> {
        let mut inner = self.inner.lock().unwrap();
        
        if inner.closed {
            return Err(MailboxError::Closed);
        }
        
        loop {
            if let Some(envelope) = inner.queue.pop_front() {
                inner.stats.message_count = inner.queue.len();
                inner.stats.processed_count += 1;
                
                if envelope.is_expired() {
                    inner.stats.expired_count += 1;
                    continue;
                }
                
                return Ok(Some(envelope));
            }
            
            return Ok(None);
        }
    }
    
    fn receive_timeout(&self, timeout: Duration) -> Result<MessageEnvelope<M>, MailboxError> {
        // This is a simple implementation that just polls
        // A more efficient implementation would use a condition variable
        let start = std::time::Instant::now();
        
        loop {
            if let Ok(Some(envelope)) = self.try_receive() {
                return Ok(envelope);
            }
            
            if start.elapsed() >= timeout {
                return Err(MailboxError::Timeout);
            }
            
            // Sleep a bit to avoid spinning
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    
    fn is_empty(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.queue.is_empty()
    }
    
    fn is_full(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        
        match inner.capacity {
            MailboxCapacity::Bounded(cap) => inner.queue.len() >= cap,
            MailboxCapacity::Unbounded => false,
        }
    }
    
    fn len(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.queue.len()
    }
    
    fn stats(&self) -> MailboxStats {
        let inner = self.inner.lock().unwrap();
        inner.stats
    }
    
    fn close(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.closed = true;
    }
    
    fn is_closed(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.closed
    }
    
    fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.queue.clear();
        inner.stats.message_count = 0;
    }
}

impl<M: Message> Debug for SimpleMailbox<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner.lock().unwrap();
        
        f.debug_struct("SimpleMailbox")
            .field("capacity", &inner.capacity)
            .field("closed", &inner.closed)
            .field("stats", &inner.stats)
            .finish()
    }
}

impl<M: Message> Clone for SimpleMailbox<M> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// A priority mailbox that processes messages according to their priority
pub struct PriorityMailbox<M: Message> {
    /// High priority mailbox
    high: SimpleMailbox<M>,
    
    /// Normal priority mailbox
    normal: SimpleMailbox<M>,
    
    /// Low priority mailbox
    low: SimpleMailbox<M>,
    
    /// Whether the mailbox is closed
    closed: Arc<Mutex<bool>>,
}

impl<M: Message> PriorityMailbox<M> {
    /// Create a new priority mailbox
    pub fn new(capacity: MailboxCapacity) -> Self {
        Self {
            high: SimpleMailbox::new(capacity),
            normal: SimpleMailbox::new(capacity),
            low: SimpleMailbox::new(capacity),
            closed: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Get the appropriate mailbox for a priority
    fn mailbox_for_priority(&self, priority: MessagePriority) -> &SimpleMailbox<M> {
        match priority {
            MessagePriority::High | MessagePriority::System => &self.high,
            MessagePriority::Normal => &self.normal,
            MessagePriority::Low => &self.low,
        }
    }
}

impl<M: Message> Mailbox<M> for PriorityMailbox<M> {
    fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), MailboxError> {
        if *self.closed.lock().unwrap() {
            return Err(MailboxError::Closed);
        }
        
        let priority = envelope.priority();
        self.mailbox_for_priority(priority).send(envelope)
    }
    
    fn receive(&self) -> Result<MessageEnvelope<M>, MailboxError> {
        if *self.closed.lock().unwrap() {
            return Err(MailboxError::Closed);
        }
        
        // Try to receive from high priority first, then normal, then low
        if let Ok(envelope) = self.high.try_receive() {
            if let Some(envelope) = envelope {
                return Ok(envelope);
            }
        }
        
        if let Ok(envelope) = self.normal.try_receive() {
            if let Some(envelope) = envelope {
                return Ok(envelope);
            }
        }
        
        if let Ok(envelope) = self.low.try_receive() {
            if let Some(envelope) = envelope {
                return Ok(envelope);
            }
        }
        
        // All mailboxes are empty, so just wait on the high priority one
        self.high.receive()
            .or_else(|_| self.normal.receive())
            .or_else(|_| self.low.receive())
    }
    
    fn try_receive(&self) -> Result<Option<MessageEnvelope<M>>, MailboxError> {
        if *self.closed.lock().unwrap() {
            return Err(MailboxError::Closed);
        }
        
        // Try to receive from high priority first, then normal, then low
        if let Ok(Some(envelope)) = self.high.try_receive() {
            return Ok(Some(envelope));
        }
        
        if let Ok(Some(envelope)) = self.normal.try_receive() {
            return Ok(Some(envelope));
        }
        
        if let Ok(Some(envelope)) = self.low.try_receive() {
            return Ok(Some(envelope));
        }
        
        Ok(None)
    }
    
    fn receive_timeout(&self, timeout: Duration) -> Result<MessageEnvelope<M>, MailboxError> {
        if *self.closed.lock().unwrap() {
            return Err(MailboxError::Closed);
        }
        
        let start = std::time::Instant::now();
        
        loop {
            // Try to receive from high priority first, then normal, then low
            if let Ok(Some(envelope)) = self.high.try_receive() {
                return Ok(envelope);
            }
            
            if let Ok(Some(envelope)) = self.normal.try_receive() {
                return Ok(envelope);
            }
            
            if let Ok(Some(envelope)) = self.low.try_receive() {
                return Ok(envelope);
            }
            
            if start.elapsed() >= timeout {
                return Err(MailboxError::Timeout);
            }
            
            // Sleep a bit to avoid spinning
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    
    fn is_empty(&self) -> bool {
        self.high.is_empty() && self.normal.is_empty() && self.low.is_empty()
    }
    
    fn is_full(&self) -> bool {
        self.high.is_full() && self.normal.is_full() && self.low.is_full()
    }
    
    fn len(&self) -> usize {
        self.high.len() + self.normal.len() + self.low.len()
    }
    
    fn stats(&self) -> MailboxStats {
        let high_stats = self.high.stats();
        let normal_stats = self.normal.stats();
        let low_stats = self.low.stats();
        
        MailboxStats {
            message_count: high_stats.message_count + normal_stats.message_count + low_stats.message_count,
            capacity: high_stats.capacity, // All mailboxes have the same capacity
            processed_count: high_stats.processed_count + normal_stats.processed_count + low_stats.processed_count,
            dropped_count: high_stats.dropped_count + normal_stats.dropped_count + low_stats.dropped_count,
            expired_count: high_stats.expired_count + normal_stats.expired_count + low_stats.expired_count,
        }
    }
    
    fn close(&self) {
        let mut closed = self.closed.lock().unwrap();
        *closed = true;
        
        self.high.close();
        self.normal.close();
        self.low.close();
    }
    
    fn is_closed(&self) -> bool {
        *self.closed.lock().unwrap()
    }
    
    fn clear(&self) {
        self.high.clear();
        self.normal.clear();
        self.low.clear();
    }
}

impl<M: Message> Debug for PriorityMailbox<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PriorityMailbox")
            .field("high", &self.high)
            .field("normal", &self.normal)
            .field("low", &self.low)
            .field("closed", &self.is_closed())
            .finish()
    }
}

impl<M: Message> Clone for PriorityMailbox<M> {
    fn clone(&self) -> Self {
        Self {
            high: self.high.clone(),
            normal: self.normal.clone(),
            low: self.low.clone(),
            closed: self.closed.clone(),
        }
    }
}

/// Helper functions for working with mailboxes
pub mod helpers {
    use super::*;
    
    /// Create a simple mailbox
    pub fn simple_mailbox<M: Message>(capacity: impl Into<usize>) -> impl Mailbox<M> {
        SimpleMailbox::new(MailboxCapacity::Bounded(capacity.into()))
    }
    
    /// Create an unbounded mailbox
    pub fn unbounded_mailbox<M: Message>() -> impl Mailbox<M> {
        SimpleMailbox::new(MailboxCapacity::Unbounded)
    }
    
    /// Create a priority mailbox
    pub fn priority_mailbox<M: Message>(capacity: impl Into<usize>) -> impl Mailbox<M> {
        PriorityMailbox::new(MailboxCapacity::Bounded(capacity.into()))
    }
    
    /// Create an unbounded priority mailbox
    pub fn unbounded_priority_mailbox<M: Message>() -> impl Mailbox<M> {
        PriorityMailbox::new(MailboxCapacity::Unbounded)
    }
}

/// Actor mailbox for receiving and processing messages
pub struct ActorMailbox {
    /// Actor ID this mailbox belongs to
    actor_id: ActorIdBox,
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
        actor_id: ActorIdBox,
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
    
    /// Get the actor ID for this mailbox
    pub fn actor_id(&self) -> &ActorIdBox {
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
    mailboxes: Arc<RwLock<HashMap<ActorIdBox, Arc<ActorMailbox>>>>,
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
        actor_id: ActorIdBox,
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
    
    /// Unregister a mailbox for an actor
    pub fn unregister_mailbox(&self, actor_id: &ActorIdBox) -> Result<()> {
        let mut mailboxes = self.mailboxes.write().map_err(|_| 
            Error::InternalError("Failed to acquire lock".to_string()))?;
        
        mailboxes.remove(actor_id);
        Ok(())
    }
    
    /// Get a mailbox for an actor
    pub fn get_mailbox(&self, actor_id: &ActorIdBox) -> Result<Arc<ActorMailbox>> {
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
    use async_trait::async_trait;
    
    struct TestHandler;
    
    #[async_trait]
    impl MessageHandler for TestHandler {
        async fn handle_message(&self, message: Message) -> Result<Option<Message>> {
            // Just echo back the message content
            Ok(Some(message))
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
        let actor_id = ActorIdBox::from(ContentAddressedActorId::with_name("test-actor"));
        let handler = Arc::new(TestHandler);
        
        let config = MailboxConfig {
            capacity: 10,
            ..Default::default()
        };
        
        let mailbox = ActorMailbox::new(actor_id, handler, config);
        
        // Send a message
        let sender_id = ActorIdBox::from(ContentAddressedActorId::with_name("sender"));
        let message = Message::new(
            Some(sender_id.clone()),
            mailbox.actor_id().clone(),
            MessageCategory::Command,
            MessagePayload::Text("test message".to_string()),
        );
        
        mailbox.send(message.clone()).await.expect("Failed to send message");
        
        // Start processing
        mailbox.start_processing().await.expect("Failed to start processing");
        
        // Sleep a bit to allow message to be processed
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check dead letters (should be empty)
        let dead_letters = mailbox.get_dead_letters().expect("Failed to get dead letters");
        assert!(dead_letters.is_empty());
    }
    
    #[tokio::test]
    async fn test_mailbox_system() -> Result<()> {
        let system = MailboxSystem::new();
        
        let actor1_id = ActorIdBox::from(ContentAddressedActorId::with_name("actor1"));
        let actor2_id = ActorIdBox::from(ContentAddressedActorId::with_name("actor2"));
        
        let handler1 = Arc::new(TestHandler);
        let handler2 = Arc::new(TestHandler);
        
        let config = MailboxConfig {
            capacity: 10,
            ..Default::default()
        };
        
        // Register mailboxes
        let mailbox1 = system.register_mailbox(actor1_id.clone(), handler1, Some(config.clone()))?;
        let mailbox2 = system.register_mailbox(actor2_id.clone(), handler2, Some(config.clone()))?;
        
        // Start all mailboxes
        system.start_all().await?;
        
        // Send a message to actor1
        let message = Message::new(
            Some(actor2_id.clone()),
            actor1_id.clone(),
            MessageCategory::Command,
            MessagePayload::Text("hello from actor2".to_string()),
        );
        
        system.send_message(message).await?;
        
        // Sleep a bit to allow message to be processed
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Unregister mailboxes
        system.unregister_mailbox(&actor1_id)?;
        system.unregister_mailbox(&actor2_id)?;
        
        Ok(())
    }
} 