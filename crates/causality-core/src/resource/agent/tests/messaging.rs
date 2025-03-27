// messaging.rs - Tests for the agent messaging system
//
// This file contains tests for the messaging system functionality,
// including message creation, routing, delivery and agent communication.

use crate::resource_types::ResourceId;
use crate::resource::agent::types::{AgentId, AgentType, AgentState};
use crate::resource::agent::agent::{Agent, AgentImpl, AgentBuilder};
use crate::resource::agent::messaging::{
    Message, MessageId, MessageType, MessageFormat, MessagePriority, 
    MessageSecurityLevel, MessageRouter, MessageFactory, MessageBuilder,
    MessageDeliveryStatus, Messaging
};

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::crypto::{ContentHash, Signature, KeyPair};
use tokio;

// Helper function to create a test agent ID
fn create_test_agent_id(name: &str) -> AgentId {
    AgentId::from_content_hash(ContentHash::calculate(name.as_bytes()))
}

// A simple mock agent implementation supporting messaging
struct MockAgent {
    agent_id: AgentId,
    router: MessageRouter,
}

impl MockAgent {
    fn new(agent_id: AgentId, router: MessageRouter) -> Self {
        Self { agent_id, router }
    }
    
    fn id(&self) -> &AgentId {
        &self.agent_id
    }
}

#[async_trait::async_trait]
impl Messaging for MockAgent {
    async fn send_message(&self, message: Message) -> Result<MessageId, crate::resource::agent::messaging::MessagingError> {
        self.router.queue_message(message).await
    }
    
    async fn receive_messages(&self) -> Result<Vec<Message>, crate::resource::agent::messaging::MessagingError> {
        self.router.get_pending_messages(&self.agent_id).await
    }
    
    async fn receive_messages_by_priority(&self) -> Result<(Vec<Message>, Vec<Message>, Vec<Message>, Vec<Message>), crate::resource::agent::messaging::MessagingError> {
        let messages = self.router.get_pending_messages(&self.agent_id).await?;
        
        let mut urgent = Vec::new();
        let mut high = Vec::new();
        let mut normal = Vec::new();
        let mut low = Vec::new();
        
        for message in messages {
            match message.priority() {
                MessagePriority::Urgent => urgent.push(message),
                MessagePriority::High => high.push(message),
                MessagePriority::Normal => normal.push(message),
                MessagePriority::Low => low.push(message),
            }
        }
        
        Ok((urgent, high, normal, low))
    }
    
    async fn mark_message_read(&self, message_id: &MessageId) -> Result<(), crate::resource::agent::messaging::MessagingError> {
        self.router.mark_read(&self.agent_id, message_id).await
    }
    
    async fn get_conversation_with(&self, agent_id: &AgentId) -> Result<Vec<Message>, crate::resource::agent::messaging::MessagingError> {
        self.router.get_conversation(&self.agent_id, agent_id).await
    }
    
    async fn create_message(
        &self,
        recipient_id: AgentId,
        subject: impl Into<String> + Send,
        content: impl Into<Vec<u8>> + Send,
        format: MessageFormat,
        message_type: MessageType,
    ) -> Result<Message, crate::resource::agent::messaging::MessagingError> {
        let message = Message::new(
            self.agent_id.clone(),
            recipient_id,
            subject,
            content,
            format,
            message_type,
        );
        
        Ok(message)
    }
    
    async fn create_response(
        &self,
        original_message: &Message,
        content: impl Into<Vec<u8>> + Send,
        format: MessageFormat,
    ) -> Result<Message, crate::resource::agent::messaging::MessagingError> {
        Ok(original_message.create_reply(content, format))
    }
}

#[tokio::test]
async fn test_messaging_system_basic() {
    let router = MessageRouter::new();
    
    // Create test agents
    let alice_id = create_test_agent_id("alice");
    let bob_id = create_test_agent_id("bob");
    
    // Register agents
    router.register_agent(alice_id.clone()).await.unwrap();
    router.register_agent(bob_id.clone()).await.unwrap();
    
    // Create mock agents
    let alice = MockAgent::new(alice_id.clone(), router.clone());
    let bob = MockAgent::new(bob_id.clone(), router.clone());
    
    // Alice sends a message to Bob
    let message = alice.create_message(
        bob_id.clone(),
        "Hello from Alice",
        "Hi Bob, how are you?",
        MessageFormat::PlainText,
        MessageType::Direct,
    ).await.unwrap();
    
    alice.send_message(message).await.unwrap();
    
    // Bob receives messages
    let bob_messages = bob.receive_messages().await.unwrap();
    assert_eq!(bob_messages.len(), 1);
    
    let received = &bob_messages[0];
    assert_eq!(received.sender_id(), &alice_id);
    assert_eq!(received.subject(), "Hello from Alice");
    assert_eq!(received.content_as_string().unwrap(), "Hi Bob, how are you?");
    
    // Bob marks message as read
    bob.mark_message_read(received.id()).await.unwrap();
    
    // Get delivery status
    let status = router.get_delivery_status(received.id()).await.unwrap();
    assert!(matches!(status, MessageDeliveryStatus::Read));
    
    // Bob replies to Alice
    let reply = bob.create_response(
        received,
        "I'm doing great! How about you?",
        MessageFormat::PlainText,
    ).await.unwrap();
    
    bob.send_message(reply).await.unwrap();
    
    // Alice receives reply
    let alice_messages = alice.receive_messages().await.unwrap();
    assert_eq!(alice_messages.len(), 1);
    
    let alice_received = &alice_messages[0];
    assert_eq!(alice_received.sender_id(), &bob_id);
    assert_eq!(alice_received.subject(), "Re: Hello from Alice");
    assert_eq!(alice_received.content_as_string().unwrap(), "I'm doing great! How about you?");
}

#[tokio::test]
async fn test_message_priorities() {
    let router = MessageRouter::new();
    
    // Create test agents
    let alice_id = create_test_agent_id("alice-priority");
    let bob_id = create_test_agent_id("bob-priority");
    
    // Register agents
    router.register_agent(alice_id.clone()).await.unwrap();
    router.register_agent(bob_id.clone()).await.unwrap();
    
    // Create mock agents
    let alice = MockAgent::new(alice_id.clone(), router.clone());
    let bob = MockAgent::new(bob_id.clone(), router.clone());
    
    // Alice sends messages with different priorities
    let low_priority = alice.create_message(
        bob_id.clone(),
        "Low Priority",
        "This is a low priority message",
        MessageFormat::PlainText,
        MessageType::Direct,
    ).await.unwrap()
    .with_priority(MessagePriority::Low);
    
    let normal_priority = alice.create_message(
        bob_id.clone(),
        "Normal Priority",
        "This is a normal priority message",
        MessageFormat::PlainText,
        MessageType::Direct,
    ).await.unwrap();
    
    let high_priority = alice.create_message(
        bob_id.clone(),
        "High Priority",
        "This is a high priority message",
        MessageFormat::PlainText,
        MessageType::Direct,
    ).await.unwrap()
    .with_priority(MessagePriority::High);
    
    let urgent_priority = alice.create_message(
        bob_id.clone(),
        "Urgent Priority",
        "This is an urgent priority message",
        MessageFormat::PlainText,
        MessageType::Direct,
    ).await.unwrap()
    .with_priority(MessagePriority::Urgent);
    
    // Send all messages
    alice.send_message(low_priority).await.unwrap();
    alice.send_message(normal_priority).await.unwrap();
    alice.send_message(high_priority).await.unwrap();
    alice.send_message(urgent_priority).await.unwrap();
    
    // Bob receives messages by priority
    let (urgent, high, normal, low) = bob.receive_messages_by_priority().await.unwrap();
    
    assert_eq!(urgent.len(), 1);
    assert_eq!(high.len(), 1);
    assert_eq!(normal.len(), 1);
    assert_eq!(low.len(), 1);
    
    assert_eq!(urgent[0].subject(), "Urgent Priority");
    assert_eq!(high[0].subject(), "High Priority");
    assert_eq!(normal[0].subject(), "Normal Priority");
    assert_eq!(low[0].subject(), "Low Priority");
}

#[tokio::test]
async fn test_message_expiration() {
    let router = MessageRouter::new();
    
    // Create test agents
    let alice_id = create_test_agent_id("alice-expiry");
    let bob_id = create_test_agent_id("bob-expiry");
    
    // Register agents
    router.register_agent(alice_id.clone()).await.unwrap();
    router.register_agent(bob_id.clone()).await.unwrap();
    
    // Create a message that expires in 50ms
    let message = MessageBuilder::new()
        .sender_id(alice_id.clone())
        .recipient_id(bob_id.clone())
        .subject("Expiring Message")
        .text_content("This message will expire soon")
        .format(MessageFormat::PlainText)
        .message_type(MessageType::Direct)
        .expires_at(Utc::now() + Duration::milliseconds(50))
        .build()
        .unwrap();
    
    // Queue the message
    router.queue_message(message).await.unwrap();
    
    // Wait a bit to allow the message to expire
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // Get bob's messages
    let bob_messages = router.get_pending_messages(&bob_id).await.unwrap();
    assert_eq!(bob_messages.len(), 1);
    
    // Check if expired
    assert!(bob_messages[0].is_expired());
}

#[tokio::test]
async fn test_message_formats() {
    let router = MessageRouter::new();
    
    // Create test agents
    let alice_id = create_test_agent_id("alice-formats");
    let bob_id = create_test_agent_id("bob-formats");
    
    // Register agents
    router.register_agent(alice_id.clone()).await.unwrap();
    router.register_agent(bob_id.clone()).await.unwrap();
    
    // Create mock agents
    let alice = MockAgent::new(alice_id.clone(), router.clone());
    
    // Create different format messages
    let plain_text = alice.create_message(
        bob_id.clone(),
        "Plain Text Message",
        "This is a plain text message",
        MessageFormat::PlainText,
        MessageType::Direct,
    ).await.unwrap();
    
    let json_message = alice.create_message(
        bob_id.clone(),
        "JSON Message",
        r#"{"key": "value", "number": 42}"#,
        MessageFormat::Json,
        MessageType::Direct,
    ).await.unwrap();
    
    let markdown_message = alice.create_message(
        bob_id.clone(),
        "Markdown Message",
        "# Header\n**Bold text**\n- List item",
        MessageFormat::Markdown,
        MessageType::Direct,
    ).await.unwrap();
    
    let binary_message = alice.create_message(
        bob_id.clone(),
        "Binary Message",
        vec![0x01, 0x02, 0x03, 0x04],
        MessageFormat::Binary,
        MessageType::Direct,
    ).await.unwrap();
    
    // Check content string conversion works correctly
    assert_eq!(plain_text.content_as_string().unwrap(), "This is a plain text message");
    assert_eq!(json_message.content_as_string().unwrap(), r#"{"key": "value", "number": 42}"#);
    assert_eq!(markdown_message.content_as_string().unwrap(), "# Header\n**Bold text**\n- List item");
    
    // Binary shouldn't convert to string
    assert!(binary_message.content_as_string().is_none());
}

#[tokio::test]
async fn test_message_factory_and_custom_types() {
    let router = MessageRouter::new();
    
    // Create test agents
    let system_id = create_test_agent_id("system");
    let user_id = create_test_agent_id("user");
    
    // Register agents
    router.register_agent(system_id.clone()).await.unwrap();
    router.register_agent(user_id.clone()).await.unwrap();
    
    // Create a message factory
    let factory = MessageFactory::new()
        .with_default_sender(system_id.clone())
        .with_default_format(MessageFormat::Json)
        .with_default_message_type(MessageType::SystemNotification);
    
    // Create a system notification
    let notification = factory.create_system_notification(
        None,
        user_id.clone(),
        "System Update",
        "The system will be down for maintenance at 2:00 PM",
    ).await.unwrap();
    
    assert_eq!(notification.sender_id(), &system_id);
    assert_eq!(notification.recipient_id(), &user_id);
    assert_eq!(notification.subject(), "System Update");
    assert_eq!(notification.message_type(), &MessageType::SystemNotification);
    
    // Create a custom message type
    let custom_message = MessageBuilder::new()
        .sender_id(system_id.clone())
        .recipient_id(user_id.clone())
        .subject("Custom Message")
        .text_content("This is a custom message")
        .format(MessageFormat::PlainText)
        .message_type(MessageType::Custom("audit-log".to_string()))
        .build()
        .unwrap();
    
    if let MessageType::Custom(custom_type) = custom_message.message_type() {
        assert_eq!(custom_type, "audit-log");
    } else {
        panic!("Expected custom message type");
    }
}

#[tokio::test]
async fn test_message_security_and_signing() {
    // Create a key pair for signing
    let key_pair = KeyPair::generate();
    
    let router = MessageRouter::new();
    
    // Create test agents
    let alice_id = create_test_agent_id("alice-security");
    let bob_id = create_test_agent_id("bob-security");
    
    // Register agents
    router.register_agent(alice_id.clone()).await.unwrap();
    router.register_agent(bob_id.clone()).await.unwrap();
    
    // Create a message
    let mut message = Message::new(
        alice_id.clone(),
        bob_id.clone(),
        "Signed Message",
        "This is a signed message",
        MessageFormat::PlainText,
        MessageType::Direct,
    );
    
    // Sign the message
    let signature = key_pair.sign(&message.content()).unwrap();
    message.set_signature(signature.clone());
    
    // Check security level was updated
    assert_eq!(message.security_level(), MessageSecurityLevel::Signed);
    
    // Queue the message
    router.queue_message(message.clone()).await.unwrap();
    
    // Get bob's messages
    let bob_messages = router.get_pending_messages(&bob_id).await.unwrap();
    assert_eq!(bob_messages.len(), 1);
    
    let received = &bob_messages[0];
    
    // Verify signature
    let signature = received.signature().unwrap();
    assert!(key_pair.verify(&received.content(), signature).is_ok());
    
    // Create encrypted message
    let mut encrypted_message = Message::new(
        alice_id.clone(),
        bob_id.clone(),
        "Encrypted Message",
        "This message is encrypted",
        MessageFormat::PlainText,
        MessageType::Direct,
    );
    
    encrypted_message.set_security_level(MessageSecurityLevel::Encrypted);
    
    // Sign and encrypt
    let signature = key_pair.sign(&encrypted_message.content()).unwrap();
    encrypted_message.set_signature(signature);
    
    // Check security level
    assert_eq!(encrypted_message.security_level(), MessageSecurityLevel::SignedAndEncrypted);
}

#[tokio::test]
async fn test_conversation_tracking() {
    let router = MessageRouter::new();
    
    // Create test agents
    let alice_id = create_test_agent_id("alice-convo");
    let bob_id = create_test_agent_id("bob-convo");
    
    // Register agents
    router.register_agent(alice_id.clone()).await.unwrap();
    router.register_agent(bob_id.clone()).await.unwrap();
    
    // Create mock agents
    let alice = MockAgent::new(alice_id.clone(), router.clone());
    let bob = MockAgent::new(bob_id.clone(), router.clone());
    
    // Send a few messages back and forth
    for i in 1..=3 {
        // Alice to Bob
        let msg = alice.create_message(
            bob_id.clone(),
            format!("Message {} from Alice", i),
            format!("This is message {} content", i),
            MessageFormat::PlainText,
            MessageType::Direct,
        ).await.unwrap();
        
        alice.send_message(msg).await.unwrap();
        
        // Bob's replies
        let bob_messages = bob.receive_messages().await.unwrap();
        let reply = bob.create_response(
            &bob_messages.last().unwrap(),
            format!("Reply to message {}", i),
            MessageFormat::PlainText,
        ).await.unwrap();
        
        bob.send_message(reply).await.unwrap();
    }
    
    // Get conversation history
    let alice_view = alice.get_conversation_with(&bob_id).await.unwrap();
    let bob_view = bob.get_conversation_with(&alice_id).await.unwrap();
    
    // Each agent should see messages they sent and received
    assert_eq!(alice_view.len(), 6); // 3 sent + 3 received
    assert_eq!(bob_view.len(), 6);   // 3 received + 3 sent
    
    // First message should be from Alice
    assert_eq!(alice_view[0].sender_id(), &alice_id);
    assert_eq!(bob_view[0].sender_id(), &alice_id);
    
    // Verify conversation flow
    for i in 0..3 {
        let msg_idx = i * 2;
        let reply_idx = msg_idx + 1;
        
        // Original from Alice
        assert_eq!(alice_view[msg_idx].sender_id(), &alice_id);
        assert_eq!(alice_view[msg_idx].recipient_id(), &bob_id);
        
        // Reply from Bob
        assert_eq!(alice_view[reply_idx].sender_id(), &bob_id);
        assert_eq!(alice_view[reply_idx].recipient_id(), &alice_id);
    }
} 