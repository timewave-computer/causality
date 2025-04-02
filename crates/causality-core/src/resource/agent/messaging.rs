// messaging.rs - Agent messaging system
//
// This module defines the messaging system for agent-to-agent communication.
// It provides secure message exchange between agents with support for
// different message types, routing, and delivery guarantees.

use crate::resource_types::ResourceId;
use crate::resource::ResourceType;
use causality_types::{ContentId, ContentHash as TypesContentHash};
use crate::resource::AgentType;
use super::types::{AgentId, AgentError};
use crate::capability::effect::EffectCapability;
use crate::id_utils;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::fmt;
use chrono::{DateTime, Utc};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use blake3;
use hex;
use rand;
use futures::stream::{self, StreamExt};
use rand::Rng;
use causality_types::ContentHash;
use crate::serialization::Serializable;

/// Message hash for calculating content hashes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageHash(String);

impl MessageHash {
    /// Calculate a new message hash from content
    pub fn calculate(content: &[u8]) -> Self {
        // Hash the content using blake3
        let hash_bytes = blake3::hash(content).as_bytes().to_vec();
        let hash = ContentHash::new("blake3", hash_bytes);
        
        // Use the hex representation for display
        Self(format!("message:{}", hash.to_hex()))
    }
    
    /// Get the string representation
    pub fn as_string(&self) -> String {
        self.0.clone()
    }
    
    /// Get the string slice representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for MessageHash {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for MessageHash {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A cryptographic signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(Vec<u8>);

impl Signature {
    /// Create a new signature from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
    
    /// Get the raw bytes of the signature
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    /// Convert to a byte vector
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Errors related to cryptographic signatures
#[derive(Debug, Error, Clone)]
pub enum SignatureError {
    /// Invalid signature format
    #[error("Invalid signature format: {0}")]
    InvalidFormat(String),
    
    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),
    
    /// Invalid key
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    /// Other error
    #[error("Signature error: {0}")]
    Other(String),
}

/// Errors that can occur when working with the messaging system
#[derive(Error, Debug)]
pub enum MessagingError {
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),
    
    /// Message validation error
    #[error("Message validation error: {0}")]
    ValidationError(String),
    
    /// Message routing error
    #[error("Message routing error: {0}")]
    RoutingError(String),
    
    /// Message delivery error
    #[error("Message delivery error: {0}")]
    DeliveryError(String),
    
    /// Message format error
    #[error("Message format error: {0}")]
    FormatError(String),
    
    /// Message encryption error
    #[error("Message encryption error: {0}")]
    EncryptionError(String),
    
    /// Signature error
    #[error("Signature error: {0}")]
    SignatureError(#[from] SignatureError),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// A result type for messaging operations
pub type MessagingResult<T> = Result<T, MessagingError>;

/// Message identifier type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MessageId(ContentId);

impl MessageId {
    /// Create a new message ID
    pub fn new(id: ContentId) -> Self {
        Self(id)
    }
    
    /// Generate a unique message ID
    pub fn generate() -> Self {
        generate_message_id()
    }
    
    /// Get string representation of the message ID
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ContentId> for MessageId {
    fn from(id: ContentId) -> Self {
        Self(id)
    }
}

impl From<&str> for MessageId {
    fn from(s: &str) -> Self {
        let content_input = format!("message:{}", s);
        Self(ContentId::new(content_input))
    }
}

impl From<String> for MessageId {
    fn from(s: String) -> Self {
        let content_input = format!("message:{}", s);
        Self(ContentId::new(content_input))
    }
}

/// Priority level for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Lowest priority
    Low,
    
    /// Normal priority
    Normal,
    
    /// High priority
    High,
    
    /// Urgent priority (will be processed ahead of all others)
    Urgent,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Delivery status for a message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageDeliveryStatus {
    /// Message is pending delivery
    Pending,
    
    /// Message has been sent but not confirmed delivered
    Sent,
    
    /// Message has been delivered to the recipient
    Delivered,
    
    /// Message has been read by the recipient
    Read,
    
    /// Message failed to be delivered
    Failed {
        /// Error reason
        reason: MessageDeliveryFailureReason,
    },
}

/// Reasons for message delivery failure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageDeliveryFailureReason {
    /// Recipient not found
    RecipientNotFound,
    
    /// Recipient is not accepting messages
    RecipientNotAcceptingMessages,
    
    /// Message expired before delivery
    Expired,
    
    /// Network error
    NetworkError,
    
    /// Unknown error
    Unknown,
}

/// Type of message
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// Direct message to a specific agent
    Direct,
    
    /// Broadcast message to multiple agents
    Broadcast,
    
    /// System notification
    SystemNotification,
    
    /// Action request
    ActionRequest,
    
    /// Response to an action request
    ActionResponse {
        /// ID of the request this is responding to
        request_id: MessageId,
    },
    
    /// Capability delegation request
    CapabilityRequest,
    
    /// Capability delegation response
    CapabilityResponse {
        /// ID of the request this is responding to
        request_id: MessageId,
    },
    
    /// Service announcement
    ServiceAnnouncement,
    
    /// Custom message type
    Custom(String),
}

/// Message content formats
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageFormat {
    /// Plain text
    PlainText,
    
    /// JSON content
    Json,
    
    /// Binary data
    Binary,
    
    /// Markdown formatted text
    Markdown,
    
    /// HTML content
    Html,
    
    /// Custom format
    Custom(String),
}

/// Security level for a message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageSecurityLevel {
    /// No special security
    Normal,
    
    /// Message is encrypted
    Encrypted,
    
    /// Message is signed
    Signed,
    
    /// Message is both encrypted and signed
    SignedAndEncrypted,
}

/// Message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique ID for the message
    id: MessageId,
    
    /// Sender agent ID
    sender_id: AgentId,
    
    /// Recipient agent ID
    recipient_id: AgentId,
    
    /// Message subject
    subject: String,
    
    /// Message content
    content: Vec<u8>,
    
    /// Content format
    format: MessageFormat,
    
    /// Message type
    message_type: MessageType,
    
    /// Message priority
    priority: MessagePriority,
    
    /// Security level
    security_level: MessageSecurityLevel,
    
    /// Signature (if signed)
    signature: Option<Signature>,
    
    /// When the message was created
    created_at: DateTime<Utc>,
    
    /// When the message expires (if applicable)
    expires_at: Option<DateTime<Utc>>,
    
    /// Message metadata
    metadata: HashMap<String, String>,
    
    /// Content hash
    content_hash: MessageHash,
}

impl Message {
    /// Create a new message
    pub fn new(
        sender_id: AgentId,
        recipient_id: AgentId,
        subject: impl Into<String>,
        content: impl Into<Vec<u8>>,
        format: MessageFormat,
        message_type: MessageType,
    ) -> Self {
        let subject = subject.into();
        let content = content.into();
        let content_hash = MessageHash::calculate(&content);
        
        Self {
            id: MessageId::generate(),
            sender_id,
            recipient_id,
            subject,
            content,
            format,
            message_type,
            priority: MessagePriority::Normal,
            security_level: MessageSecurityLevel::Normal,
            signature: None,
            created_at: Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
            content_hash,
        }
    }
    
    /// Get the message ID
    pub fn id(&self) -> &MessageId {
        &self.id
    }
    
    /// Get the sender ID
    pub fn sender_id(&self) -> &AgentId {
        &self.sender_id
    }
    
    /// Get the recipient ID
    pub fn recipient_id(&self) -> &AgentId {
        &self.recipient_id
    }
    
    /// Get the message subject
    pub fn subject(&self) -> &str {
        &self.subject
    }
    
    /// Get the message content
    pub fn content(&self) -> &[u8] {
        &self.content
    }
    
    /// Get the content as a string if possible
    pub fn content_as_string(&self) -> Option<String> {
        match self.format {
            MessageFormat::PlainText | MessageFormat::Json | MessageFormat::Markdown | MessageFormat::Html => {
                String::from_utf8(self.content.clone()).ok()
            },
            _ => None,
        }
    }
    
    /// Get the message format
    pub fn format(&self) -> &MessageFormat {
        &self.format
    }
    
    /// Get the message type
    pub fn message_type(&self) -> &MessageType {
        &self.message_type
    }
    
    /// Get the message priority
    pub fn priority(&self) -> MessagePriority {
        self.priority
    }
    
    /// Set the message priority
    pub fn set_priority(&mut self, priority: MessagePriority) {
        self.priority = priority;
    }
    
    /// With priority (builder pattern)
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Get the security level
    pub fn security_level(&self) -> MessageSecurityLevel {
        self.security_level
    }
    
    /// Set the security level
    pub fn set_security_level(&mut self, level: MessageSecurityLevel) {
        self.security_level = level;
    }
    
    /// With security level (builder pattern)
    pub fn with_security_level(mut self, level: MessageSecurityLevel) -> Self {
        self.security_level = level;
        self
    }
    
    /// Get the signature if available
    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }
    
    /// Set the signature
    pub fn set_signature(&mut self, signature: Signature) {
        self.signature = Some(signature);
        self.security_level = match self.security_level {
            MessageSecurityLevel::Normal => MessageSecurityLevel::Signed,
            MessageSecurityLevel::Encrypted => MessageSecurityLevel::SignedAndEncrypted,
            _ => self.security_level,
        };
    }
    
    /// With signature (builder pattern)
    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.set_signature(signature);
        self
    }
    
    /// Get the creation time
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    
    /// Get the expiration time if set
    pub fn expires_at(&self) -> Option<&DateTime<Utc>> {
        self.expires_at.as_ref()
    }
    
    /// Set the expiration time
    pub fn set_expires_at(&mut self, expires_at: DateTime<Utc>) {
        self.expires_at = Some(expires_at);
    }
    
    /// With expiration time (builder pattern)
    pub fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Has the message expired?
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            expires < Utc::now()
        } else {
            false
        }
    }
    
    /// Get all metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    /// Get a specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Set a metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
    
    /// With metadata (builder pattern)
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Get the content hash
    pub fn content_hash(&self) -> &MessageHash {
        &self.content_hash
    }
    
    /// Verify the content hash
    pub fn verify_content_hash(&self) -> bool {
        let calculated_hash = MessageHash::calculate(&self.content);
        calculated_hash == self.content_hash
    }
    
    /// Create a reply to this message
    pub fn create_reply(
        &self,
        content: impl Into<Vec<u8>>,
        format: MessageFormat,
    ) -> Self {
        let content = content.into();
        let content_hash = MessageHash::calculate(&content);
        
        let message_type = match self.message_type {
            MessageType::ActionRequest => MessageType::ActionResponse {
                request_id: self.id.clone(),
            },
            MessageType::CapabilityRequest => MessageType::CapabilityResponse {
                request_id: self.id.clone(),
            },
            _ => MessageType::Direct,
        };
        
        let subject = if self.subject.starts_with("Re:") {
            self.subject.clone()
        } else {
            format!("Re: {}", self.subject)
        };
        
        Self {
            id: MessageId::generate(),
            sender_id: self.recipient_id.clone(),
            recipient_id: self.sender_id.clone(),
            subject,
            content,
            format,
            message_type,
            priority: self.priority,
            security_level: MessageSecurityLevel::Normal,
            signature: None,
            created_at: Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
            content_hash,
        }
    }
}

/// Message delivery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelivery {
    /// Message ID
    message_id: MessageId,
    
    /// Sender agent ID
    sender_id: AgentId,
    
    /// Recipient agent ID
    recipient_id: AgentId,
    
    /// Delivery status
    status: MessageDeliveryStatus,
    
    /// When the message was sent
    sent_at: DateTime<Utc>,
    
    /// When the message was delivered (if applicable)
    delivered_at: Option<DateTime<Utc>>,
    
    /// When the message was read (if applicable)
    read_at: Option<DateTime<Utc>>,
    
    /// Number of delivery attempts
    delivery_attempts: u32,
    
    /// Last error (if any)
    last_error: Option<String>,
    
    /// Delivery metadata
    metadata: HashMap<String, String>,
}

impl MessageDelivery {
    /// Create a new message delivery
    pub fn new(message_id: MessageId, sender_id: AgentId, recipient_id: AgentId) -> Self {
        Self {
            message_id,
            sender_id,
            recipient_id,
            status: MessageDeliveryStatus::Pending,
            sent_at: Utc::now(),
            delivered_at: None,
            read_at: None,
            delivery_attempts: 0,
            last_error: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Get the message ID
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
    
    /// Get the sender ID
    pub fn sender_id(&self) -> &AgentId {
        &self.sender_id
    }
    
    /// Get the recipient ID
    pub fn recipient_id(&self) -> &AgentId {
        &self.recipient_id
    }
    
    /// Get the delivery status
    pub fn status(&self) -> MessageDeliveryStatus {
        self.status
    }
    
    /// Set the delivery status
    pub fn set_status(&mut self, status: MessageDeliveryStatus) {
        self.status = status;
        
        // Update timestamps based on the new status
        match status {
            MessageDeliveryStatus::Delivered => {
                self.delivered_at = Some(Utc::now());
            },
            MessageDeliveryStatus::Read => {
                if self.delivered_at.is_none() {
                    self.delivered_at = Some(Utc::now());
                }
                self.read_at = Some(Utc::now());
            },
            _ => {}
        }
    }
    
    /// Get the sent time
    pub fn sent_at(&self) -> &DateTime<Utc> {
        &self.sent_at
    }
    
    /// Get the delivered time if applicable
    pub fn delivered_at(&self) -> Option<&DateTime<Utc>> {
        self.delivered_at.as_ref()
    }
    
    /// Get the read time if applicable
    pub fn read_at(&self) -> Option<&DateTime<Utc>> {
        self.read_at.as_ref()
    }
    
    /// Get the number of delivery attempts
    pub fn delivery_attempts(&self) -> u32 {
        self.delivery_attempts
    }
    
    /// Increment the delivery attempts
    pub fn increment_attempts(&mut self) {
        self.delivery_attempts += 1;
    }
    
    /// Get the last error if any
    pub fn last_error(&self) -> Option<&String> {
        self.last_error.as_ref()
    }
    
    /// Set the last error
    pub fn set_last_error(&mut self, error: impl Into<String>) {
        self.last_error = Some(error.into());
    }
    
    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    /// Get a specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Set a metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
    
    /// Has the message been delivered?
    pub fn is_delivered(&self) -> bool {
        matches!(self.status, MessageDeliveryStatus::Delivered | MessageDeliveryStatus::Read)
    }
    
    /// Has the message been read?
    pub fn is_read(&self) -> bool {
        matches!(self.status, MessageDeliveryStatus::Read)
    }
    
    /// Has the message failed to deliver?
    pub fn is_failed(&self) -> bool {
        matches!(self.status, MessageDeliveryStatus::Failed { .. })
    }
}

/// Builder for creating messages
pub struct MessageBuilder {
    /// Sender agent ID
    sender_id: Option<AgentId>,
    
    /// Recipient agent ID
    recipient_id: Option<AgentId>,
    
    /// Message subject
    subject: Option<String>,
    
    /// Message content
    content: Option<Vec<u8>>,
    
    /// Content format
    format: MessageFormat,
    
    /// Message type
    message_type: MessageType,
    
    /// Message priority
    priority: MessagePriority,
    
    /// Security level
    security_level: MessageSecurityLevel,
    
    /// Signature
    signature: Option<Signature>,
    
    /// Expiration time
    expires_at: Option<DateTime<Utc>>,
    
    /// Message metadata
    metadata: HashMap<String, String>,
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new() -> Self {
        Self {
            sender_id: None,
            recipient_id: None,
            subject: None,
            content: None,
            format: MessageFormat::PlainText,
            message_type: MessageType::Direct,
            priority: MessagePriority::Normal,
            security_level: MessageSecurityLevel::Normal,
            signature: None,
            expires_at: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the sender ID
    pub fn sender_id(mut self, sender_id: AgentId) -> Self {
        self.sender_id = Some(sender_id);
        self
    }
    
    /// Set the recipient ID
    pub fn recipient_id(mut self, recipient_id: AgentId) -> Self {
        self.recipient_id = Some(recipient_id);
        self
    }
    
    /// Set the subject
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }
    
    /// Set the content
    pub fn content(mut self, content: impl Into<Vec<u8>>) -> Self {
        self.content = Some(content.into());
        self
    }
    
    /// Set the content from a string
    pub fn text_content(mut self, content: impl Into<String>) -> Self {
        let content = content.into();
        self.content = Some(content.into_bytes());
        self
    }
    
    /// Set the format
    pub fn format(mut self, format: MessageFormat) -> Self {
        self.format = format;
        self
    }
    
    /// Set the message type
    pub fn message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = message_type;
        self
    }
    
    /// Set the priority
    pub fn priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set the security level
    pub fn security_level(mut self, level: MessageSecurityLevel) -> Self {
        self.security_level = level;
        self
    }
    
    /// Set the signature
    pub fn signature(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }
    
    /// Set the expiration time
    pub fn expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Add a metadata entry
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Build the message
    pub fn build(self) -> MessagingResult<Message> {
        let sender_id = self.sender_id.ok_or_else(|| {
            MessagingError::ValidationError("Sender ID is required".to_string())
        })?;
        
        let recipient_id = self.recipient_id.ok_or_else(|| {
            MessagingError::ValidationError("Recipient ID is required".to_string())
        })?;
        
        let subject = self.subject.ok_or_else(|| {
            MessagingError::ValidationError("Subject is required".to_string())
        })?;
        
        let content = self.content.ok_or_else(|| {
            MessagingError::ValidationError("Content is required".to_string())
        })?;
        
        let content_hash = MessageHash::calculate(&content);
        
        let mut message = Message {
            id: MessageId::generate(),
            sender_id,
            recipient_id,
            subject,
            content,
            format: self.format,
            message_type: self.message_type,
            priority: self.priority,
            security_level: self.security_level,
            signature: self.signature,
            created_at: Utc::now(),
            expires_at: self.expires_at,
            metadata: self.metadata,
            content_hash,
        };
        
        // Update security level based on signature
        if message.signature.is_some() {
            message.security_level = match message.security_level {
                MessageSecurityLevel::Normal => MessageSecurityLevel::Signed,
                MessageSecurityLevel::Encrypted => MessageSecurityLevel::SignedAndEncrypted,
                _ => message.security_level,
            };
        }
        
        Ok(message)
    }
}

/// Factory for creating messages
pub struct MessageFactory {
    /// Default sender ID for messages created by this factory
    default_sender_id: Option<AgentId>,
    
    /// Default message format
    default_format: MessageFormat,
    
    /// Default message priority
    default_priority: MessagePriority,
    
    /// Default security level
    default_security_level: MessageSecurityLevel,
    
    /// Default message type
    default_message_type: MessageType,
}

impl MessageFactory {
    /// Create a new message factory
    pub fn new() -> Self {
        Self {
            default_sender_id: None,
            default_format: MessageFormat::PlainText,
            default_priority: MessagePriority::Normal,
            default_security_level: MessageSecurityLevel::Normal,
            default_message_type: MessageType::Direct,
        }
    }
    
    /// Set the default sender ID
    pub fn with_default_sender(mut self, sender_id: AgentId) -> Self {
        self.default_sender_id = Some(sender_id);
        self
    }
    
    /// Set the default format
    pub fn with_default_format(mut self, format: MessageFormat) -> Self {
        self.default_format = format;
        self
    }
    
    /// Set the default priority
    pub fn with_default_priority(mut self, priority: MessagePriority) -> Self {
        self.default_priority = priority;
        self
    }
    
    /// Set the default security level
    pub fn with_default_security_level(mut self, level: MessageSecurityLevel) -> Self {
        self.default_security_level = level;
        self
    }
    
    /// Set the default message type
    pub fn with_default_message_type(mut self, message_type: MessageType) -> Self {
        self.default_message_type = message_type;
        self
    }
    
    /// Create a new message
    pub fn create_message(
        &self,
        sender_id: Option<AgentId>,
        recipient_id: AgentId,
        subject: impl Into<String>,
        content: impl Into<Vec<u8>>,
        format: Option<MessageFormat>,
        message_type: Option<MessageType>,
    ) -> MessagingResult<Message> {
        let sender_id = sender_id.or_else(|| self.default_sender_id.clone())
            .ok_or_else(|| MessagingError::ValidationError("Sender ID is required".to_string()))?;
        
        let format = format.unwrap_or(self.default_format.clone());
        let message_type = message_type.unwrap_or(self.default_message_type.clone());
        
        let mut builder = MessageBuilder::new()
            .sender_id(sender_id)
            .recipient_id(recipient_id)
            .subject(subject)
            .content(content)
            .format(format)
            .message_type(message_type)
            .priority(self.default_priority)
            .security_level(self.default_security_level);
        
        builder.build()
    }
    
    /// Create a direct text message
    pub fn create_text_message(
        &self,
        sender_id: Option<AgentId>,
        recipient_id: AgentId,
        subject: impl Into<String>,
        content: impl Into<String>,
    ) -> MessagingResult<Message> {
        let content_string = content.into();
        self.create_message(
            sender_id,
            recipient_id,
            subject,
            content_string.into_bytes(),
            Some(MessageFormat::PlainText),
            Some(MessageType::Direct),
        )
    }
    
    /// Create a system notification
    pub fn create_system_notification(
        &self,
        sender_id: Option<AgentId>,
        recipient_id: AgentId,
        subject: impl Into<String>,
        content: impl Into<String>,
    ) -> MessagingResult<Message> {
        let content_string = content.into();
        self.create_message(
            sender_id,
            recipient_id,
            subject,
            content_string.into_bytes(),
            Some(MessageFormat::PlainText),
            Some(MessageType::SystemNotification),
        )
    }
    
    /// Create a capability request message
    pub fn create_capability_request(
        &self,
        sender_id: Option<AgentId>,
        recipient_id: AgentId,
        capability_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> MessagingResult<Message> {
        let capability_id = capability_id.into();
        let reason = reason.into();
        
        let content = ToString::to_string(&serde_json::json!({
            "capability_id": capability_id,
            "reason": reason,
            "requested_at": Utc::now().to_rfc3339(),
        }));
        
        self.create_message(
            sender_id,
            recipient_id,
            format!("Capability Request: {}", capability_id),
            content.into_bytes(),
            Some(MessageFormat::Json),
            Some(MessageType::CapabilityRequest),
        )
    }
    
    /// Create a service announcement message
    pub fn create_service_announcement(
        &self,
        sender_id: Option<AgentId>,
        service_type: impl Into<String>,
        service_info: impl Into<String>,
    ) -> MessagingResult<Message> {
        let service_type = service_type.into();
        let service_info = service_info.into();
        
        let content = ToString::to_string(&serde_json::json!({
            "service_type": service_type,
            "service_info": service_info,
            "announced_at": Utc::now().to_rfc3339(),
        }));
        
        // Service announcements are broadcast
        let broadcast_id = AgentId::from_content_hash(MessageHash::calculate(b"broadcast").as_str().as_bytes(), AgentType::Operator);
        
        self.create_message(
            sender_id,
            broadcast_id,
            format!("Service Announcement: {}", service_type),
            content.into_bytes(),
            Some(MessageFormat::Json),
            Some(MessageType::ServiceAnnouncement),
        )
    }
}

impl Default for MessageFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a message routing operation
#[derive(Debug)]
pub struct MessageRoutingResult {
    /// Message ID
    pub message_id: MessageId,
    
    /// Number of recipients the message was routed to
    pub recipient_count: usize,
    
    /// Delivery IDs for tracking delivery status
    pub delivery_ids: Vec<MessageId>,
    
    /// Any routing errors that occurred
    pub errors: Vec<MessagingError>,
}

/// Queue of messages for an agent
#[derive(Debug, Clone)]
pub struct MessageQueue {
    /// Agent ID
    agent_id: AgentId,
    
    /// Pending messages
    pending: Vec<Message>,
    
    /// Message history (by conversation)
    history: HashMap<AgentId, Vec<MessageId>>,
}

impl MessageQueue {
    /// Create a new message queue for an agent
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            pending: Vec::new(),
            history: HashMap::new(),
        }
    }
    
    /// Get the agent ID
    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }
    
    /// Queue a message
    pub fn enqueue_message(&mut self, message: Message) {
        // Record in history
        let conversation_partner = if &message.sender_id == &self.agent_id {
            &message.recipient_id
        } else {
            &message.sender_id
        };
        
        self.history
            .entry(conversation_partner.clone())
            .or_insert_with(Vec::new)
            .push(message.id.clone());
        
        // Add to pending queue
        self.pending.push(message);
    }
    
    /// Get pending messages
    pub fn pending_messages(&self) -> &[Message] {
        &self.pending
    }
    
    /// Get pending messages by priority
    pub fn pending_by_priority(&self) -> (Vec<&Message>, Vec<&Message>, Vec<&Message>, Vec<&Message>) {
        let mut urgent = Vec::new();
        let mut high = Vec::new();
        let mut normal = Vec::new();
        let mut low = Vec::new();
        
        for message in &self.pending {
            match message.priority {
                MessagePriority::Urgent => urgent.push(message),
                MessagePriority::High => high.push(message),
                MessagePriority::Normal => normal.push(message),
                MessagePriority::Low => low.push(message),
            }
        }
        
        (urgent, high, normal, low)
    }
    
    /// Remove a message from the queue
    pub fn remove_message(&mut self, message_id: &MessageId) -> Option<Message> {
        if let Some(index) = self.pending.iter().position(|m| m.id == *message_id) {
            Some(self.pending.remove(index))
        } else {
            None
        }
    }
    
    /// Clear all pending messages
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }
    
    /// Get message history with a specific agent
    pub fn get_conversation(&self, agent_id: &AgentId) -> Option<&Vec<MessageId>> {
        self.history.get(agent_id)
    }
    
    /// Number of pending messages
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

/// Manages message routing and delivery
pub struct MessageRouter {
    /// Queues for each agent
    queues: Arc<RwLock<HashMap<AgentId, MessageQueue>>>,
    
    /// Message delivery tracking
    deliveries: Arc<RwLock<HashMap<MessageId, MessageDelivery>>>,
    
    /// Message storage
    messages: Arc<RwLock<HashMap<MessageId, Message>>>,
    
    /// Message factory
    factory: MessageFactory,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            deliveries: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            factory: MessageFactory::new(),
        }
    }
    
    /// Get the message factory
    pub fn factory(&self) -> &MessageFactory {
        &self.factory
    }
    
    /// Register an agent with the messaging system
    pub async fn register_agent(&self, agent_id: AgentId) -> MessagingResult<()> {
        let mut queues = self.queues.write().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        if !queues.contains_key(&agent_id) {
            queues.insert(agent_id.clone(), MessageQueue::new(agent_id));
        }
        
        Ok(())
    }
    
    /// Unregister an agent from the messaging system
    pub async fn unregister_agent(&self, agent_id: &AgentId) -> MessagingResult<()> {
        let mut queues = self.queues.write().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        queues.remove(agent_id);
        
        Ok(())
    }
    
    /// Check if an agent is registered
    pub async fn is_agent_registered(&self, agent_id: &AgentId) -> MessagingResult<bool> {
        let queues = self.queues.read().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        Ok(queues.contains_key(agent_id))
    }
    
    /// Queue a message for delivery
    pub async fn queue_message(&self, message: Message) -> MessagingResult<MessageId> {
        // Store the message
        let message_id = message.id().clone();
        
        {
            let mut messages = self.messages.write().map_err(|e| 
                MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
            )?;
            
            messages.insert(message_id.clone(), message.clone());
        }
        
        // Create delivery tracking
        {
            let mut deliveries = self.deliveries.write().map_err(|e| 
                MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
            )?;
            
            let delivery = MessageDelivery::new(
                message_id.clone(),
                message.sender_id().clone(),
                message.recipient_id().clone(),
            );
            
            deliveries.insert(message_id.clone(), delivery);
        }
        
        // Put in recipient's queue
        {
            let mut queues = self.queues.write().map_err(|e| 
                MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
            )?;
            
            if let Some(queue) = queues.get_mut(message.recipient_id()) {
                queue.enqueue_message(message);
            } else {
                // Recipient not registered
                self.update_delivery_status(
                    &message_id,
                    MessageDeliveryStatus::Failed {
                        reason: MessageDeliveryFailureReason::RecipientNotFound
                    }
                ).await?;
                
                return Err(MessagingError::RoutingError(
                    format!("Recipient {} not registered", message.recipient_id())
                ));
            }
        }
        
        // Update delivery status
        self.update_delivery_status(&message_id, MessageDeliveryStatus::Sent).await?;
        
        Ok(message_id)
    }
    
    /// Route a message to its recipient
    pub async fn route_message(&self, message: Message) -> MessagingResult<MessageRoutingResult> {
        // Check that the message has a valid recipient
        let recipient_id = message.recipient_id().clone();
        
        // Check if recipient ID is empty (using a check for an empty string)
        let recipient_str = format!("{}", recipient_id);
        if recipient_str.is_empty() {
            return Err(MessagingError::ValidationError(
                "Message has no recipient ID".to_string()
            ));
        }
        
        let message_id = message.id().clone();
        
        // Handle broadcast messages
        if recipient_str == "*" || recipient_str == "broadcast" {
            return self.broadcast_message(message).await;
        }
        
        // Check if the recipient is registered
        if !self.is_agent_registered(&recipient_id).await? {
            // Queue message anyway but mark as pending until agent registers
            // Create a clone since queue_message takes ownership
            let message_clone = message.clone();
            self.queue_message(message_clone).await?;
            
            return Ok(MessageRoutingResult {
                message_id,
                recipient_count: 1,
                delivery_ids: Vec::new(),  // No delivery IDs since agent isn't registered
                errors: vec![MessagingError::RoutingError(
                    format!("Recipient not registered: {}", recipient_id)
                )],
            });
        }
        
        // Queue the message for the recipient
        let delivery_id = self.queue_message(message).await?;
        
        Ok(MessageRoutingResult {
            message_id,
            recipient_count: 1,
            delivery_ids: vec![delivery_id],
            errors: Vec::new(),
        })
    }
    
    /// Broadcast a message to all registered agents
    async fn broadcast_message(&self, message: Message) -> MessagingResult<MessageRoutingResult> {
        let message_id = message.id().clone();
        let mut recipient_count = 0;
        let mut delivery_ids = Vec::new();
        let mut errors = Vec::new();
        
        // Get list of all registered agents
        let queues = self.queues.read().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        let agent_ids: Vec<AgentId> = queues.keys().cloned().collect();
        
        // Send to each agent
        for agent_id in agent_ids {
            // Skip sending to the original sender
            if agent_id == *message.sender_id() {
                continue;
            }
            
            // Create a copy of the message for this recipient
            let agent_message = message.clone();
            // We would update recipient ID here if needed
            
            // Try to deliver
            match self.queue_message(agent_message).await {
                Ok(delivery_id) => {
                    recipient_count += 1;
                    delivery_ids.push(delivery_id);
                }
                Err(err) => {
                    errors.push(err);
                }
            }
        }
        
        Ok(MessageRoutingResult {
            message_id,
            recipient_count,
            delivery_ids,
            errors,
        })
    }
    
    /// Get pending messages for an agent
    pub async fn get_pending_messages(&self, agent_id: &AgentId) -> MessagingResult<Vec<Message>> {
        let queues = self.queues.read().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        let queue = queues.get(agent_id).ok_or_else(|| 
            MessagingError::RoutingError(format!("Agent {} not registered", agent_id))
        )?;
        
        // Clone pending messages
        let messages = queue.pending_messages().to_vec();
        
        Ok(messages)
    }
    
    /// Mark a message as delivered
    pub async fn mark_delivered(&self, _agent_id: &AgentId, message_id: &MessageId) -> MessagingResult<()> {
        // Update delivery status
        self.update_delivery_status(message_id, MessageDeliveryStatus::Delivered).await?;
        
        Ok(())
    }
    
    /// Mark a message as read
    pub async fn mark_read(&self, _agent_id: &AgentId, message_id: &MessageId) -> MessagingResult<()> {
        // Update delivery status
        self.update_delivery_status(message_id, MessageDeliveryStatus::Read).await?;
        
        Ok(())
    }
    
    /// Update delivery status for a message
    pub async fn update_delivery_status(
        &self,
        message_id: &MessageId,
        status: MessageDeliveryStatus,
    ) -> MessagingResult<()> {
        let mut deliveries = self.deliveries.write().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        if let Some(delivery) = deliveries.get_mut(message_id) {
            delivery.set_status(status);
            Ok(())
        } else {
            Err(MessagingError::RoutingError(
                format!("No delivery record found for message {}", message_id)
            ))
        }
    }
    
    /// Get a message by ID
    pub async fn get_message(&self, message_id: &MessageId) -> MessagingResult<Message> {
        let messages = self.messages.read().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        messages.get(message_id)
            .cloned()
            .ok_or_else(|| MessagingError::RoutingError(
                format!("Message {} not found", message_id)
            ))
    }
    
    /// Get delivery status for a message
    pub async fn get_delivery_status(&self, message_id: &MessageId) -> MessagingResult<MessageDeliveryStatus> {
        let deliveries = self.deliveries.read().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        deliveries.get(message_id)
            .map(|d| d.status())
            .ok_or_else(|| MessagingError::RoutingError(
                format!("No delivery record found for message {}", message_id)
            ))
    }
    
    /// Clear all pending messages for an agent
    pub async fn clear_agent_queue(&self, agent_id: &AgentId) -> MessagingResult<usize> {
        let mut queues = self.queues.write().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        if let Some(queue) = queues.get_mut(agent_id) {
            let count = queue.pending_count();
            queue.clear_pending();
            Ok(count)
        } else {
            Err(MessagingError::RoutingError(
                format!("Agent {} not registered", agent_id)
            ))
        }
    }
    
    /// Get conversation history between two agents
    pub async fn get_conversation(
        &self,
        agent_id: &AgentId,
        other_agent_id: &AgentId,
    ) -> MessagingResult<Vec<Message>> {
        let queues = self.queues.read().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        let queue = queues.get(agent_id).ok_or_else(|| 
            MessagingError::RoutingError(format!("Agent {} not registered", agent_id))
        )?;
        
        let messages = self.messages.read().map_err(|e| 
            MessagingError::InternalError(format!("Failed to acquire lock: {}", e))
        )?;
        
        let conversation = queue.get_conversation(other_agent_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| messages.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(conversation)
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for agents to send and receive messages
#[async_trait]
pub trait Messaging {
    /// Send a message to another agent
    async fn send_message(&self, message: Message) -> MessagingResult<MessageId>;
    
    /// Get pending messages
    async fn receive_messages(&self) -> MessagingResult<Vec<Message>>;
    
    /// Get messages by priority
    async fn receive_messages_by_priority(&self) -> MessagingResult<(Vec<Message>, Vec<Message>, Vec<Message>, Vec<Message>)>;
    
    /// Mark a message as read
    async fn mark_message_read(&self, message_id: &MessageId) -> MessagingResult<()>;
    
    /// Get conversation history with another agent
    async fn get_conversation_with(&self, agent_id: &AgentId) -> MessagingResult<Vec<Message>>;
    
    /// Create a new message
    async fn create_message(
        &self,
        recipient_id: AgentId,
        subject: impl Into<String> + Send,
        content: impl Into<Vec<u8>> + Send,
        format: MessageFormat,
        message_type: MessageType,
    ) -> MessagingResult<Message>;
    
    /// Create a response to a message
    async fn create_response(
        &self,
        original_message: &Message,
        content: impl Into<Vec<u8>> + Send,
        format: MessageFormat,
    ) -> MessagingResult<Message>;
}

/// Message effects for the effect system
pub struct MessageEffect {
    /// Agent ID
    pub agent_id: AgentId,
    
    /// Effect type
    pub effect_type: MessageEffectType,
    
    /// Message 
    pub message: Option<Message>,
    
    /// Message ID (for status updates)
    pub message_id: Option<MessageId>,
    
    /// Recipient ID (for send operations)
    pub recipient_id: Option<AgentId>,
}

/// Types of message effects
pub enum MessageEffectType {
    /// Send a message
    Send,
    
    /// Mark a message as delivered
    MarkDelivered,
    
    /// Mark a message as read
    MarkRead,
    
    /// Broadcast a message to multiple recipients
    Broadcast,
}

impl MessageEffect {
    /// Create a send message effect
    pub fn send(
        agent_id: AgentId,
        message: Message,
    ) -> Self {
        Self {
            agent_id,
            effect_type: MessageEffectType::Send,
            message: Some(message),
            message_id: None,
            recipient_id: None,
        }
    }
    
    /// Create a mark delivered effect
    pub fn mark_delivered(
        agent_id: AgentId,
        message_id: MessageId,
    ) -> Self {
        Self {
            agent_id,
            effect_type: MessageEffectType::MarkDelivered,
            message: None,
            message_id: Some(message_id),
            recipient_id: None,
        }
    }
    
    /// Create a mark read effect
    pub fn mark_read(
        agent_id: AgentId,
        message_id: MessageId,
    ) -> Self {
        Self {
            agent_id,
            effect_type: MessageEffectType::MarkRead,
            message: None,
            message_id: Some(message_id),
            recipient_id: None,
        }
    }
    
    /// Create a broadcast message effect
    pub fn broadcast(
        agent_id: AgentId,
        message: Message,
    ) -> Self {
        Self {
            agent_id,
            effect_type: MessageEffectType::Broadcast,
            message: Some(message),
            message_id: None,
            recipient_id: None,
        }
    }
}

/// Generate a random message ID based on content addressing
pub fn generate_message_id() -> MessageId {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_be_bytes();
    
    let mut random_bytes = [0u8; 16];
    rand::thread_rng().try_fill(&mut random_bytes).expect("Failed to generate random bytes");
    
    let mut input = Vec::with_capacity(timestamp.len() + random_bytes.len());
    input.extend_from_slice(&timestamp);
    input.extend_from_slice(&random_bytes);
    
    // Create a content hash of the data
    let hash = blake3::hash(&input);
    // Create a content id with a prefixed string containing the message type and hash
    let content_input = format!("message:{}", hex::encode(hash.as_bytes()));
    MessageId::from(ContentId::new(content_input))
}

/// Create a ContentHash from content
fn hash_content(content: &[u8]) -> ContentHash {
    // Hash the content directly
    let hash_bytes = blake3::hash(content).as_bytes().to_vec();
    ContentHash::new("blake3", hash_bytes)
}

/// Agent messaging system tests
#[cfg(test)]
mod tests {
    use super::*;
    use blake3;

    fn create_test_agent_id(name: &str) -> AgentId {
        AgentId::from_content_hash(
            blake3::hash(name.as_bytes()).as_bytes(),
            AgentType::Operator
        )
    }
    
    #[tokio::test]
    async fn test_message_creation() {
        let sender = create_test_agent_id("sender");
        let recipient = create_test_agent_id("recipient");
        
        // Create a simple message
        let message = Message::new(
            sender.clone(),
            recipient.clone(),
            "Test Message",
            "This is a test message",
            MessageFormat::PlainText,
            MessageType::Direct,
        );
        
        assert_eq!(message.sender_id(), &sender);
        assert_eq!(message.recipient_id(), &recipient);
        assert_eq!(message.subject(), "Test Message");
        assert_eq!(message.content_as_string().unwrap(), "This is a test message");
        assert_eq!(message.format(), &MessageFormat::PlainText);
        assert_eq!(message.message_type(), &MessageType::Direct);
        
        // Using the builder
        let builder_message = MessageBuilder::new()
            .sender_id(sender.clone())
            .recipient_id(recipient.clone())
            .subject("Builder Message")
            .text_content("Message from builder")
            .format(MessageFormat::PlainText)
            .message_type(MessageType::Direct)
            .priority(MessagePriority::High)
            .build()
            .unwrap();
        
        assert_eq!(builder_message.sender_id(), &sender);
        assert_eq!(builder_message.recipient_id(), &recipient);
        assert_eq!(builder_message.subject(), "Builder Message");
        assert_eq!(builder_message.content_as_string().unwrap(), "Message from builder");
        assert_eq!(builder_message.priority(), MessagePriority::High);
    }
    
    #[tokio::test]
    async fn test_message_factory() {
        let sender = create_test_agent_id("sender");
        let recipient = create_test_agent_id("recipient");
        
        let factory = MessageFactory::new()
            .with_default_sender(sender.clone())
            .with_default_format(MessageFormat::Json)
            .with_default_priority(MessagePriority::High);
        
        // Create a simple text message
        let text_message = factory.create_text_message(
            None,
            recipient.clone(),
            "Text Message",
            "This is a text message",
        ).unwrap();
        
        assert_eq!(text_message.sender_id(), &sender);
        assert_eq!(text_message.subject(), "Text Message");
        assert_eq!(text_message.format(), &MessageFormat::PlainText);
        assert_eq!(text_message.priority(), MessagePriority::High);
        
        // Create a system notification
        let notification = factory.create_system_notification(
            None,
            recipient.clone(),
            "System Notification",
            "This is a system notification",
        ).unwrap();
        
        assert_eq!(notification.sender_id(), &sender);
        assert_eq!(notification.subject(), "System Notification");
        assert_eq!(notification.message_type(), &MessageType::SystemNotification);
        
        // Create a capability request
        let capability_request = factory.create_capability_request(
            None,
            recipient.clone(),
            "read_document",
            "Need to read this document for analysis",
        ).unwrap();
        
        assert_eq!(capability_request.sender_id(), &sender);
        assert_eq!(capability_request.format(), &MessageFormat::Json);
        assert_eq!(capability_request.message_type(), &MessageType::CapabilityRequest);
        
        // Check the JSON content
        let content_str = capability_request.content_as_string().unwrap();
        let content: serde_json::Value = serde_json::from_str(&content_str).unwrap();
        assert_eq!(content["capability_id"], "read_document");
        assert_eq!(content["reason"], "Need to read this document for analysis");
    }
    
    #[tokio::test]
    async fn test_message_router() {
        let sender = create_test_agent_id("sender");
        let recipient = create_test_agent_id("recipient");
        let router = MessageRouter::new();
        
        // Register agents
        router.register_agent(sender.clone()).await.unwrap();
        router.register_agent(recipient.clone()).await.unwrap();
        
        // Create and route a message
        let message = Message::new(
            sender.clone(),
            recipient.clone(),
            "Routed Message",
            "This is a message via the router",
            MessageFormat::PlainText,
            MessageType::Direct,
        );
        
        let result = router.route_message(message.clone()).await.unwrap();
        assert_eq!(result.recipient_count, 1);
        assert_eq!(result.errors.len(), 0);
        
        // Check that the message is pending
        let pending = router.get_pending_messages(&recipient).await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].subject(), "Routed Message");
        
        // Mark as delivered
        router.mark_delivered(&recipient, &pending[0].id()).await.unwrap();
        
        // Check delivery status
        let status = router.get_delivery_status(&pending[0].id()).await.unwrap();
        assert!(matches!(status, MessageDeliveryStatus::Delivered));
        
        // Mark as read
        router.mark_read(&recipient, &pending[0].id()).await.unwrap();
        
        // Check delivery status again
        let status = router.get_delivery_status(&pending[0].id()).await.unwrap();
        assert!(matches!(status, MessageDeliveryStatus::Read));
    }
    
    #[tokio::test]
    async fn test_broadcast_messaging() {
        let sender = create_test_agent_id("broadcaster");
        let recipient1 = create_test_agent_id("recipient1");
        let recipient2 = create_test_agent_id("recipient2");
        let recipient3 = create_test_agent_id("recipient3");
        
        let router = MessageRouter::new();
        
        // Register all agents
        router.register_agent(sender.clone()).await.unwrap();
        router.register_agent(recipient1.clone()).await.unwrap();
        router.register_agent(recipient2.clone()).await.unwrap();
        router.register_agent(recipient3.clone()).await.unwrap();
        
        // Broadcast agent ID (special agent for broadcasts)
        let broadcast_id = AgentId::from_content_hash(
            blake3::hash(b"broadcast").as_bytes(),
            AgentType::Operator
        );
        
        // Create a broadcast message
        let broadcast_message = Message::new(
            sender.clone(),
            broadcast_id,
            "Broadcast Announcement",
            "This is a broadcast to all agents",
            MessageFormat::PlainText,
            MessageType::Broadcast,
        );
        
        // Route the broadcast message
        let result = router.route_message(broadcast_message).await.unwrap();
        
        // Should be delivered to all agents except sender
        assert_eq!(result.recipient_count, 4); // total agents including sender
        assert_eq!(result.delivery_ids.len(), 3); // delivered to 3 recipients
        
        // Check that all recipients got the message
        let pending1 = router.get_pending_messages(&recipient1).await.unwrap();
        let pending2 = router.get_pending_messages(&recipient2).await.unwrap();
        let pending3 = router.get_pending_messages(&recipient3).await.unwrap();
        
        assert_eq!(pending1.len(), 1);
        assert_eq!(pending2.len(), 1);
        assert_eq!(pending3.len(), 1);
        
        // Sender should not have received their own broadcast
        let sender_pending = router.get_pending_messages(&sender).await.unwrap();
        assert_eq!(sender_pending.len(), 0);
        
        // All should have same subject
        assert_eq!(pending1[0].subject(), "Broadcast Announcement");
        assert_eq!(pending2[0].subject(), "Broadcast Announcement");
        assert_eq!(pending3[0].subject(), "Broadcast Announcement");
    }
    
    #[tokio::test]
    async fn test_message_reply() {
        let sender = create_test_agent_id("original-sender");
        let recipient = create_test_agent_id("original-recipient");
        
        // Create original message
        let original = Message::new(
            sender.clone(),
            recipient.clone(),
            "Original Message",
            "This is the original message",
            MessageFormat::PlainText,
            MessageType::Direct,
        );
        
        // Create a reply
        let reply = original.create_reply(
            "This is a reply to the original message",
            MessageFormat::PlainText,
        );
        
        // Verify reply properties
        assert_eq!(reply.sender_id(), &recipient);
        assert_eq!(reply.recipient_id(), &sender);
        assert_eq!(reply.subject(), "Re: Original Message");
        assert_eq!(reply.content_as_string().unwrap(), "This is a reply to the original message");
        
        // Create an action request
        let action_request = Message::new(
            sender.clone(),
            recipient.clone(),
            "Action Request",
            "Please perform this action",
            MessageFormat::PlainText,
            MessageType::ActionRequest,
        );
        
        // Create a response to the action request
        let action_response = action_request.create_reply(
            "Action completed successfully",
            MessageFormat::PlainText,
        );
        
        // Verify action response properties
        if let MessageType::ActionResponse { request_id } = action_response.message_type() {
            assert_eq!(request_id, action_request.id());
        } else {
            panic!("Expected ActionResponse message type");
        }
    }
} 