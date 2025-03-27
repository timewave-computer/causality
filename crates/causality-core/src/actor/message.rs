// Message handling for the actor system
//
// This module provides abstractions for defining, sending, and handling messages
// in the actor system.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use std::error::Error;

use crate::time::Timestamp;

/// A trait for messages that can be sent to actors
pub trait Message: Send + 'static {
    /// The type of response this message expects
    type Response: Send + 'static;
}

/// The result of handling a message
pub enum HandleResult<T> {
    /// The message was handled successfully
    Success(T),
    
    /// The message handling was deferred
    Deferred,
    
    /// The message handling failed
    Failure(Box<dyn Error + Send + Sync>),
    
    /// The message was not handled
    Unhandled,
}

impl<T> HandleResult<T> {
    /// Create a success result
    pub fn success(value: T) -> Self {
        Self::Success(value)
    }
    
    /// Create a deferred result
    pub fn deferred() -> Self {
        Self::Deferred
    }
    
    /// Create a failure result
    pub fn failure(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Failure(Box::new(error))
    }
    
    /// Create an unhandled result
    pub fn unhandled() -> Self {
        Self::Unhandled
    }
    
    /// Check if the result is a success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }
    
    /// Check if the result is deferred
    pub fn is_deferred(&self) -> bool {
        matches!(self, Self::Deferred)
    }
    
    /// Check if the result is a failure
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failure(_))
    }
    
    /// Check if the result is unhandled
    pub fn is_unhandled(&self) -> bool {
        matches!(self, Self::Unhandled)
    }
    
    /// Get the success value
    pub fn success_value(&self) -> Option<&T> {
        match self {
            Self::Success(value) => Some(value),
            _ => None,
        }
    }
    
    /// Get the failure error
    pub fn failure_error(&self) -> Option<&(dyn Error + Send + Sync)> {
        match self {
            Self::Failure(error) => Some(&**error),
            _ => None,
        }
    }
}

/// A trait for handling messages of a specific type
pub trait MessageHandler<M: Message> {
    /// Handle a message
    fn handle(&mut self, message: M) -> HandleResult<M::Response>;
}

/// Options for sending messages
#[derive(Debug, Clone)]
pub struct SendOptions {
    /// The timeout for the message
    pub timeout: Option<Duration>,
    
    /// The priority of the message
    pub priority: MessagePriority,
    
    /// Whether the message is critical
    pub critical: bool,
}

impl Default for SendOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            priority: MessagePriority::Normal,
            critical: false,
        }
    }
}

impl SendOptions {
    /// Create a new set of send options
    pub fn new() -> Self {
        Default::default()
    }
    
    /// Set the timeout for the message
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Set the priority of the message
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set whether the message is critical
    pub fn with_critical(mut self, critical: bool) -> Self {
        self.critical = critical;
        self
    }
}

/// Priority of a message
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Low priority, handled after normal messages
    Low = 0,
    
    /// Normal priority, handled in order
    Normal = 1,
    
    /// High priority, handled before normal messages
    High = 2,
    
    /// System priority, handled before all other messages
    System = 3,
}

/// A trait for objects that can send messages
pub trait MessageSender<M: Message> {
    /// The error type returned when sending fails
    type Error: Error;
    
    /// Send a message
    fn send(&self, msg: M) -> Result<(), Self::Error>;
    
    /// Send a message with options
    fn send_with(&self, msg: M, options: SendOptions) -> Result<(), Self::Error>;
    
    /// Send a message and wait for a response
    fn ask(&self, msg: M) -> Result<M::Response, Self::Error>;
    
    /// Send a message with options and wait for a response
    fn ask_with(&self, msg: M, options: SendOptions) -> Result<M::Response, Self::Error>;
}

/// A trait for objects that can receive messages
pub trait MessageReceiver<M: Message> {
    /// The error type returned when receiving fails
    type Error: Error;
    
    /// Receive a message
    fn receive(&self) -> Result<M, Self::Error>;
    
    /// Receive a message with a timeout
    fn receive_timeout(&self, timeout: Duration) -> Result<M, Self::Error>;
    
    /// Try to receive a message without blocking
    fn try_receive(&self) -> Result<Option<M>, Self::Error>;
}

/// A message envelope that wraps a message with metadata
#[derive(Debug)]
pub struct MessageEnvelope<M: Message> {
    /// The message
    pub message: M,
    
    /// The options for sending the message
    pub options: SendOptions,
    
    /// The creation time of the envelope
    pub created_at: Timestamp,
}

impl<M: Message> MessageEnvelope<M> {
    /// Create a new message envelope
    pub fn new(message: M) -> Self {
        Self {
            message,
            options: SendOptions::default(),
            created_at: crate::time::now(),
        }
    }
    
    /// Create a new message envelope with options
    pub fn with_options(message: M, options: SendOptions) -> Self {
        Self {
            message,
            options,
            created_at: crate::time::now(),
        }
    }
    
    /// Check if the message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(timeout) = self.options.timeout {
            let elapsed = crate::time::now() - self.created_at;
            elapsed > crate::time::Duration::from_millis(timeout.as_millis() as u64)
        } else {
            false
        }
    }
    
    /// Get the priority of the message
    pub fn priority(&self) -> MessagePriority {
        self.options.priority
    }
    
    /// Check if the message is critical
    pub fn is_critical(&self) -> bool {
        self.options.critical
    }
}

/// Helper functions for working with messages
pub mod helpers {
    use super::*;
    
    /// Create a message handler from a function
    pub fn handler_fn<M, F>(f: F) -> impl MessageHandler<M>
    where
        M: Message,
        F: FnMut(M) -> HandleResult<M::Response> + 'static,
    {
        struct FnHandler<M, F> {
            f: F,
            _marker: std::marker::PhantomData<M>,
        }
        
        impl<M, F> MessageHandler<M> for FnHandler<M, F>
        where
            M: Message,
            F: FnMut(M) -> HandleResult<M::Response>,
        {
            fn handle(&mut self, message: M) -> HandleResult<M::Response> {
                (self.f)(message)
            }
        }
        
        FnHandler {
            f,
            _marker: std::marker::PhantomData,
        }
    }
    
    /// Create a successful result
    pub fn success<T>(value: T) -> HandleResult<T> {
        HandleResult::success(value)
    }
    
    /// Create a deferred result
    pub fn deferred<T>() -> HandleResult<T> {
        HandleResult::deferred()
    }
    
    /// Create a failure result
    pub fn failure<T>(error: impl Error + Send + Sync + 'static) -> HandleResult<T> {
        HandleResult::failure(error)
    }
    
    /// Create an unhandled result
    pub fn unhandled<T>() -> HandleResult<T> {
        HandleResult::unhandled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // A test message
    struct TestMessage {
        value: i32,
    }
    
    impl Message for TestMessage {
        type Response = i32;
    }
    
    #[test]
    fn test_message_envelope() {
        let message = TestMessage { value: 42 };
        let envelope = MessageEnvelope::new(message);
        
        assert_eq!(envelope.message.value, 42);
        assert_eq!(envelope.priority(), MessagePriority::Normal);
        assert!(!envelope.is_critical());
        assert!(!envelope.is_expired());
        
        let options = SendOptions::new()
            .with_priority(MessagePriority::High)
            .with_critical(true);
        
        let message = TestMessage { value: 43 };
        let envelope = MessageEnvelope::with_options(message, options);
        
        assert_eq!(envelope.message.value, 43);
        assert_eq!(envelope.priority(), MessagePriority::High);
        assert!(envelope.is_critical());
    }
    
    #[test]
    fn test_handle_result() {
        let success = HandleResult::success(42);
        assert!(success.is_success());
        assert_eq!(success.success_value(), Some(&42));
        
        let deferred = HandleResult::<i32>::deferred();
        assert!(deferred.is_deferred());
        
        struct TestError;
        
        impl std::fmt::Debug for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "TestError")
            }
        }
        
        impl std::fmt::Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "TestError")
            }
        }
        
        impl std::error::Error for TestError {}
        
        let failure = HandleResult::<i32>::failure(TestError);
        assert!(failure.is_failure());
        assert!(failure.failure_error().is_some());
        
        let unhandled = HandleResult::<i32>::unhandled();
        assert!(unhandled.is_unhandled());
    }
    
    #[test]
    fn test_message_handler() {
        let mut handler = helpers::handler_fn(|message: TestMessage| {
            HandleResult::success(message.value * 2)
        });
        
        let message = TestMessage { value: 42 };
        let result = handler.handle(message);
        
        assert!(result.is_success());
        assert_eq!(result.success_value(), Some(&84));
    }
} 