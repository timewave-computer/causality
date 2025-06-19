// Virtual blockchain types that maintain linear semantics

pub mod mailbox;
pub mod token;
pub mod state_diff;

// Re-export key types
pub use mailbox::{Mailbox, MailboxId, TokenId, MailboxConstraint, MailboxEffect, MailboxError};
pub use token::{Token, TokenSend, TokenReceive, TokenEffect, TokenError};
pub use state_diff::{StateDiff, StateDiffId, StateConstraint, StateDiffError};

// Common traits for virtual types
pub trait VirtualType {
    /// Convert to a session type representation
    fn to_session_type(&self) -> crate::layer1::SessionType;
    
    /// Check if operations maintain linear semantics
    fn check_linearity(&self) -> bool;
}

// Blockchain-specific effects that extend the base effect system
pub enum BlockchainEffect<A, R> {
    /// Mailbox operations
    Mailbox(MailboxEffect<A, R>),
    
    /// Token operations
    Token(TokenEffect<A, R>),
} 