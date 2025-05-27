//! Pattern System
//!
//! Defines patterns for capability-based interactions and message handling within the system.
//! This module consolidates standardized message formats and communication patterns for 
//! interactions between system components.

use std::collections::BTreeMap;
use crate::primitive::ids::{DomainId, MessageId, ResourceId};
use crate::primitive::string::Str;
use crate::expression::r#type::TypeExpr;
use crate::expression::value::ValueExpr;

//-----------------------------------------------------------------------------
// Message Type Definition
//-----------------------------------------------------------------------------

/// Represents a generic message within the system.
/// Messages enable communication between different domains in the system.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Unique identifier for the message.
    pub id: MessageId,
    /// Identifier of the source domain, if applicable.
    pub source_domain_id: Option<DomainId>,
    /// Identifier of the target domain.
    pub target_domain_id: DomainId,
    /// Identifier of the resource this message pertains to, if any.
    pub target_resource_id: Option<ResourceId>,
    /// The content of the message.
    pub content: Option<ValueExpr>,
}

//-----------------------------------------------------------------------------
// Message Implementation
//-----------------------------------------------------------------------------

impl Message {
    /// Creates a new message with a generated ID.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MessageId,
        target_domain_id: DomainId,
        source_domain_id: Option<DomainId>,
        target_resource_id: Option<ResourceId>,
        content: Option<ValueExpr>,
    ) -> Self {
        Self {
            id,
            target_domain_id,
            source_domain_id,
            target_resource_id,
            content,
        }
    }
}

//-----------------------------------------------------------------------------
// Type Schema
//-----------------------------------------------------------------------------

/// Returns the canonical `TypeExpr` for a "Message"
///
/// A Message has id, target_domain_id, source_domain_id (optional), 
/// target_resource_id (optional), and content (optional).
pub fn message_schema() -> TypeExpr {
    let mut fields = BTreeMap::new();

    // Basic fields that all messages have
    fields.insert(Str::new("id"), TypeExpr::String);
    fields.insert(Str::new("target_domain_id"), TypeExpr::String);

    // Optional fields
    fields.insert(
        Str::new("source_domain_id"),
        TypeExpr::Optional(Box::new(TypeExpr::String).into()),
    );

    fields.insert(
        Str::new("target_resource_id"),
        TypeExpr::Optional(Box::new(TypeExpr::String).into()),
    );

    // Content can be any ValueExpr or null
    fields.insert(
        Str::new("content"),
        TypeExpr::Optional(Box::new(TypeExpr::Any).into()),
    );

    TypeExpr::Record(fields.into())
}

//-----------------------------------------------------------------------------
// Pattern Matching Support
//-----------------------------------------------------------------------------

/// Pattern matching utilities for message handling and routing
pub mod matching {
    use super::*;

    /// Trait for pattern matching against messages
    pub trait MessagePattern {
        /// Check if this pattern matches the given message
        fn matches(&self, message: &Message) -> bool;
    }

    /// Simple domain-based message pattern
    #[derive(Debug, Clone)]
    pub struct DomainPattern {
        pub target_domain_id: Option<DomainId>,
        pub source_domain_id: Option<DomainId>,
    }

    impl MessagePattern for DomainPattern {
        fn matches(&self, message: &Message) -> bool {
            if let Some(target_id) = &self.target_domain_id {
                if message.target_domain_id != *target_id {
                    return false;
                }
            }
            
            if let Some(source_id) = &self.source_domain_id {
                match &message.source_domain_id {
                    Some(msg_source_id) => {
                        if *msg_source_id != *source_id {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            
            true
        }
    }

    /// Resource-based message pattern
    #[derive(Debug, Clone)]
    pub struct ResourcePattern {
        pub target_resource_id: Option<ResourceId>,
    }

    impl MessagePattern for ResourcePattern {
        fn matches(&self, message: &Message) -> bool {
            if let Some(resource_id) = &self.target_resource_id {
                match &message.target_resource_id {
                    Some(msg_resource_id) => *msg_resource_id == *resource_id,
                    None => false,
                }
            } else {
                true
            }
        }
    }
}

//-----------------------------------------------------------------------------
// Capability-based Patterns
//-----------------------------------------------------------------------------

/// Capability patterns for secure message handling
pub mod capability {
    use super::*;

    /// Represents a capability for accessing specific message types or domains
    #[derive(Debug, Clone, PartialEq)]
    pub struct Capability {
        /// The domain this capability grants access to
        pub domain_id: DomainId,
        /// Optional resource-specific access
        pub resource_id: Option<ResourceId>,
        /// Actions permitted by this capability
        pub permissions: Vec<Permission>,
    }

    /// Types of permissions that can be granted
    #[derive(Debug, Clone, PartialEq)]
    pub enum Permission {
        /// Can read messages
        Read,
        /// Can send messages
        Send,
        /// Can modify message routing
        Route,
        /// Can create new capabilities
        Grant,
    }

    impl Capability {
        /// Create a new capability with the specified permissions
        pub fn new(domain_id: DomainId, resource_id: Option<ResourceId>, permissions: Vec<Permission>) -> Self {
            Self {
                domain_id,
                resource_id,
                permissions,
            }
        }

        /// Check if this capability permits the given action on a message
        pub fn permits(&self, message: &Message, permission: &Permission) -> bool {
            // Check if the capability applies to this domain
            if message.target_domain_id != self.domain_id {
                return false;
            }

            // Check resource-specific access if applicable
            if let Some(cap_resource_id) = &self.resource_id {
                match &message.target_resource_id {
                    Some(msg_resource_id) => {
                        if *msg_resource_id != *cap_resource_id {
                            return false;
                        }
                    }
                    None => return false,
                }
            }

            // Check if the capability includes the required permission
            self.permissions.contains(permission)
        }
    }
}

//-----------------------------------------------------------------------------
// Communication Patterns
//-----------------------------------------------------------------------------

/// Common communication patterns for message handling
pub mod communication {
    use super::*;

    /// Request-response pattern for message communication
    #[derive(Debug, Clone)]
    pub struct RequestResponse {
        pub request: Message,
        pub response: Option<Message>,
        pub correlation_id: MessageId,
    }

    impl RequestResponse {
        /// Create a new request-response pair
        pub fn new(request: Message, correlation_id: MessageId) -> Self {
            Self {
                request,
                response: None,
                correlation_id,
            }
        }

        /// Set the response for this request
        pub fn set_response(&mut self, response: Message) {
            self.response = Some(response);
        }

        /// Check if this request-response pair is complete
        pub fn is_complete(&self) -> bool {
            self.response.is_some()
        }
    }

    /// Publish-subscribe pattern for broadcast communication
    #[derive(Debug, Clone)]
    pub struct PubSub {
        pub topic: Str,
        pub subscribers: Vec<DomainId>,
        pub publisher: DomainId,
    }

    impl PubSub {
        /// Create a new publish-subscribe pattern
        pub fn new(topic: Str, publisher: DomainId) -> Self {
            Self {
                topic,
                subscribers: Vec::new(),
                publisher,
            }
        }

        /// Add a subscriber to this topic
        pub fn subscribe(&mut self, domain_id: DomainId) {
            if !self.subscribers.contains(&domain_id) {
                self.subscribers.push(domain_id);
            }
        }

        /// Remove a subscriber from this topic
        pub fn unsubscribe(&mut self, domain_id: &DomainId) {
            self.subscribers.retain(|id| id != domain_id);
        }

        /// Check if a domain is subscribed to this topic
        pub fn is_subscribed(&self, domain_id: &DomainId) -> bool {
            self.subscribers.contains(domain_id)
        }

        /// Create a message for publishing to all subscribers
        pub fn create_broadcast_message(&self, content: ValueExpr, message_id: MessageId) -> Vec<Message> {
            self.subscribers
                .iter()
                .map(|subscriber_id| Message::new(
                    message_id,
                    *subscriber_id,
                    Some(self.publisher),
                    None,
                    Some(content.clone()),
                ))
                .collect()
        }
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::ids::EntityId;

    #[test]
    fn test_message_creation() {
        let msg_id = MessageId::new(EntityId::new([1u8; 32]));
        let target_domain = DomainId::new(EntityId::new([2u8; 32]));
        let source_domain = DomainId::new(EntityId::new([3u8; 32]));
        
        let message = Message::new(
            msg_id,
            target_domain,
            Some(source_domain),
            None,
            None,
        );
        
        assert_eq!(message.id, msg_id);
        assert_eq!(message.target_domain_id, target_domain);
        assert_eq!(message.source_domain_id, Some(source_domain));
        assert!(message.target_resource_id.is_none());
        assert!(message.content.is_none());
    }

    #[test]
    fn test_domain_pattern_matching() {
        use matching::*;
        
        let target_domain = DomainId::new(EntityId::new([1u8; 32]));
        let source_domain = DomainId::new(EntityId::new([2u8; 32]));
        
        let pattern = DomainPattern {
            target_domain_id: Some(target_domain),
            source_domain_id: Some(source_domain),
        };
        
        let message = Message::new(
            MessageId::new(EntityId::new([3u8; 32])),
            target_domain,
            Some(source_domain),
            None,
            None,
        );
        
        assert!(pattern.matches(&message));
        
        // Test non-matching message
        let different_target = DomainId::new(EntityId::new([4u8; 32]));
        let non_matching_message = Message::new(
            MessageId::new(EntityId::new([5u8; 32])),
            different_target,
            Some(source_domain),
            None,
            None,
        );
        
        assert!(!pattern.matches(&non_matching_message));
    }

    #[test]
    fn test_capability_permissions() {
        use capability::*;
        
        let domain_id = DomainId::new(EntityId::new([1u8; 32]));
        let capability = Capability::new(
            domain_id,
            None,
            vec![Permission::Read, Permission::Send],
        );
        
        let message = Message::new(
            MessageId::new(EntityId::new([2u8; 32])),
            domain_id,
            None,
            None,
            None,
        );
        
        assert!(capability.permits(&message, &Permission::Read));
        assert!(capability.permits(&message, &Permission::Send));
        assert!(!capability.permits(&message, &Permission::Route));
        assert!(!capability.permits(&message, &Permission::Grant));
    }

    #[test]
    fn test_pubsub_pattern() {
        use communication::*;
        
        let topic = Str::new("test_topic");
        let publisher = DomainId::new(EntityId::new([1u8; 32]));
        let subscriber1 = DomainId::new(EntityId::new([2u8; 32]));
        let subscriber2 = DomainId::new(EntityId::new([3u8; 32]));
        
        let mut pubsub = PubSub::new(topic, publisher);
        
        pubsub.subscribe(subscriber1);
        pubsub.subscribe(subscriber2);
        
        assert!(pubsub.is_subscribed(&subscriber1));
        assert!(pubsub.is_subscribed(&subscriber2));
        assert_eq!(pubsub.subscribers.len(), 2);
        
        pubsub.unsubscribe(&subscriber1);
        assert!(!pubsub.is_subscribed(&subscriber1));
        assert_eq!(pubsub.subscribers.len(), 1);
    }

    #[test]
    fn test_message_schema() {
        let schema = message_schema();
        // Verify that the schema is a record type
        match schema {
            TypeExpr::Record(_) => {
                // Schema is correctly a record type
            }
            _ => panic!("Message schema should be a Record type"),
        }
    }
} 