//! Session types for typed communication protocols
//!
//! This module implements session types as a foundational abstraction for Layer 2,
//! providing type-safe communication protocols with automatic duality checking.
//! Session types form the third pillar of Layer 2 alongside effects and intents.

use std::fmt;
use serde::{Deserialize, Serialize};

use crate::lambda::base::{Value, TypeInner};
use super::core::EffectExpr;

/// Unique identifier for session declarations
pub type SessionId = String;

/// Unique identifier for session roles
pub type RoleId = String;

/// Session type syntax following standard session type notation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionType {
    /// Send value of type T, continue with S (!T.S)
    Send(TypeInner, Box<SessionType>),
    
    /// Receive value of type T, continue with S (?T.S)
    Receive(TypeInner, Box<SessionType>),
    
    /// Internal choice - offer one of the branches (⊕{...})
    InternalChoice(Vec<SessionType>),
    
    /// External choice - accept one of the branches (&{...})
    ExternalChoice(Vec<SessionType>),
    
    /// Protocol termination (End)
    End,
    
    /// Recursive session type (rec X.S)
    Recursive(String, Box<SessionType>),
    
    /// Session type variable (X)
    Variable(String),
}

/// Session role specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRole {
    pub name: RoleId,
    pub protocol: SessionType,
}

/// Complete session declaration with all roles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDeclaration {
    pub name: SessionId,
    pub roles: Vec<SessionRole>,
    pub verified_duality: bool,
}

/// Current state of a session protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Ready to begin protocol
    Ready,
    
    /// Waiting to send value of specified type
    WaitingSend(TypeInner),
    
    /// Waiting to receive value of specified type
    WaitingReceive(TypeInner),
    
    /// Waiting to make a choice from available options
    WaitingChoice(Vec<String>),
    
    /// Waiting for external choice
    WaitingBranch(Vec<String>),
    
    /// Protocol completed
    Terminated,
    
    /// Error state
    Error(String),
}

/// Session channel representing one endpoint of a session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionChannel {
    pub id: String,
    pub session_type: SessionType,
    pub role: RoleId,
    pub state: SessionState,
    pub message_history: Vec<SessionMessage>,
}

/// Message in a session communication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMessage {
    pub value: Value,
    pub message_type: TypeInner,
    pub timestamp: u64,
}

/// Session branch for case operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionBranch {
    pub label: String,
    pub body: EffectExpr,
}

/// Session operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionOperation {
    Send(Value),
    Receive,
    Select(String),
    Branch(Vec<String>),
}

/// Session type errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionError {
    /// Protocol violation - operation doesn't match expected protocol
    ProtocolViolation {
        expected: SessionType,
        actual: SessionOperation,
    },
    
    /// Duality mismatch between roles
    DualityMismatch {
        session_name: String,
        role1: String,
        role2: String,
    },
    
    /// Session channel closed unexpectedly
    ChannelClosed {
        session_id: String,
    },
    
    /// Invalid choice selection
    InvalidChoice {
        available_choices: Vec<String>,
        selected_choice: String,
    },
    
    /// Recursive type depth exceeded
    RecursionDepthExceeded {
        max_depth: usize,
    },
    
    /// Session not found in registry
    SessionNotFound {
        session_name: String,
    },
    
    /// Malformed session type
    MalformedSessionType {
        reason: String,
    },
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionError::ProtocolViolation { expected, actual } => {
                write!(f, "Protocol violation: expected {:?}, got {:?}", expected, actual)
            }
            SessionError::DualityMismatch { session_name, role1, role2 } => {
                write!(f, "Duality mismatch in session '{}' between roles '{}' and '{}'", 
                       session_name, role1, role2)
            }
            SessionError::ChannelClosed { session_id } => {
                write!(f, "Session channel '{}' is closed", session_id)
            }
            SessionError::InvalidChoice { available_choices, selected_choice } => {
                write!(f, "Invalid choice '{}', available: {:?}", selected_choice, available_choices)
            }
            SessionError::RecursionDepthExceeded { max_depth } => {
                write!(f, "Recursion depth exceeded: {}", max_depth)
            }
            SessionError::SessionNotFound { session_name } => {
                write!(f, "Session '{}' not found", session_name)
            }
            SessionError::MalformedSessionType { reason } => {
                write!(f, "Malformed session type: {}", reason)
            }
        }
    }
}

impl std::error::Error for SessionError {}

/// Duality computation for session types
pub fn compute_dual(session_type: &SessionType) -> SessionType {
    match session_type {
        SessionType::Send(t, s) => SessionType::Receive(t.clone(), Box::new(compute_dual(s))),
        SessionType::Receive(t, s) => SessionType::Send(t.clone(), Box::new(compute_dual(s))),
        SessionType::InternalChoice(branches) => {
            SessionType::ExternalChoice(branches.iter().map(compute_dual).collect())
        }
        SessionType::ExternalChoice(branches) => {
            SessionType::InternalChoice(branches.iter().map(compute_dual).collect())
        }
        SessionType::End => SessionType::End,
        SessionType::Recursive(var, s) => {
            SessionType::Recursive(var.clone(), Box::new(compute_dual(s)))
        }
        SessionType::Variable(var) => SessionType::Variable(var.clone()),
    }
}

/// Verify that two session types are duals
pub fn verify_duality(s1: &SessionType, s2: &SessionType) -> bool {
    let dual_s1 = compute_dual(s1);
    &dual_s1 == s2
}

/// Check if a session type is well-formed
pub fn is_well_formed(session_type: &SessionType) -> Result<(), SessionError> {
    is_well_formed_with_depth(session_type, 0, 100) // Max recursion depth of 100
}

fn is_well_formed_with_depth(
    session_type: &SessionType, 
    depth: usize, 
    max_depth: usize
) -> Result<(), SessionError> {
    if depth > max_depth {
        return Err(SessionError::RecursionDepthExceeded { max_depth });
    }
    
    match session_type {
        SessionType::Send(_, s) | SessionType::Receive(_, s) => {
            is_well_formed_with_depth(s, depth, max_depth)
        }
        SessionType::InternalChoice(branches) | SessionType::ExternalChoice(branches) => {
            if branches.is_empty() {
                return Err(SessionError::MalformedSessionType {
                    reason: "Choice must have at least one branch".to_string(),
                });
            }
            for branch in branches {
                is_well_formed_with_depth(branch, depth, max_depth)?;
            }
            Ok(())
        }
        SessionType::End => Ok(()),
        SessionType::Recursive(_, s) => {
            is_well_formed_with_depth(s, depth + 1, max_depth)
        }
        SessionType::Variable(_) => Ok(()),
    }
}

/// Substitute a session type variable with another session type
pub fn substitute(
    var: &str, 
    replacement: &SessionType, 
    target: &SessionType
) -> SessionType {
    match target {
        SessionType::Send(t, s) => {
            SessionType::Send(t.clone(), Box::new(substitute(var, replacement, s)))
        }
        SessionType::Receive(t, s) => {
            SessionType::Receive(t.clone(), Box::new(substitute(var, replacement, s)))
        }
        SessionType::InternalChoice(branches) => {
            SessionType::InternalChoice(
                branches.iter().map(|b| substitute(var, replacement, b)).collect()
            )
        }
        SessionType::ExternalChoice(branches) => {
            SessionType::ExternalChoice(
                branches.iter().map(|b| substitute(var, replacement, b)).collect()
            )
        }
        SessionType::End => SessionType::End,
        SessionType::Recursive(v, s) => {
            if v == var {
                // Variable is bound by this recursive type, don't substitute
                SessionType::Recursive(v.clone(), s.clone())
            } else {
                SessionType::Recursive(v.clone(), Box::new(substitute(var, replacement, s)))
            }
        }
        SessionType::Variable(v) => {
            if v == var {
                replacement.clone()
            } else {
                SessionType::Variable(v.clone())
            }
        }
    }
}

/// Progress a session type through an operation
pub fn progress_session(
    session_type: &SessionType,
    operation: &SessionOperation,
) -> Result<SessionType, SessionError> {
    match (session_type, operation) {
        (SessionType::Send(_expected_type, continuation), SessionOperation::Send(_value)) => {
            // In a real implementation, we'd verify that value has expected_type
            Ok((**continuation).clone())
        }
        (SessionType::Receive(_expected_type, continuation), SessionOperation::Receive) => {
            Ok((**continuation).clone())
        }
        (SessionType::InternalChoice(branches), SessionOperation::Select(choice)) => {
            if let Some(index) = choice.parse::<usize>().ok() {
                if index < branches.len() {
                    Ok(branches[index].clone())
                } else {
                    Err(SessionError::InvalidChoice {
                        available_choices: (0..branches.len()).map(|i| i.to_string()).collect(),
                        selected_choice: choice.clone(),
                    })
                }
            } else {
                Err(SessionError::InvalidChoice {
                    available_choices: (0..branches.len()).map(|i| i.to_string()).collect(),
                    selected_choice: choice.clone(),
                })
            }
        }
        _ => Err(SessionError::ProtocolViolation {
            expected: session_type.clone(),
            actual: operation.clone(),
        }),
    }
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionType::Send(t, s) => write!(f, "!{}.{}", format_type(t), s),
            SessionType::Receive(t, s) => write!(f, "?{}.{}", format_type(t), s),
            SessionType::InternalChoice(branches) => {
                write!(f, "⊕{{")?;
                for (i, branch) in branches.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", branch)?;
                }
                write!(f, "}}")
            }
            SessionType::ExternalChoice(branches) => {
                write!(f, "&{{")?;
                for (i, branch) in branches.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", branch)?;
                }
                write!(f, "}}")
            }
            SessionType::End => write!(f, "End"),
            SessionType::Recursive(var, s) => write!(f, "rec {}.{}", var, s),
            SessionType::Variable(var) => write!(f, "{}", var),
        }
    }
}

fn format_type(t: &TypeInner) -> String {
    // Simple type formatting - in a real implementation this would be more sophisticated
    format!("{:?}", t)
}

impl SessionChannel {
    pub fn new(id: String, session_type: SessionType, role: RoleId) -> Self {
        Self {
            id,
            session_type: session_type.clone(),
            role,
            state: SessionState::Ready,
            message_history: Vec::new(),
        }
    }
    
    pub fn is_terminated(&self) -> bool {
        matches!(self.state, SessionState::Terminated)
    }
    
    pub fn is_ready(&self) -> bool {
        matches!(self.state, SessionState::Ready)
    }
    
    pub fn add_message(&mut self, message: SessionMessage) {
        self.message_history.push(message);
    }
}

impl SessionDeclaration {
    pub fn new(name: SessionId, roles: Vec<SessionRole>) -> Self {
        Self {
            name,
            roles,
            verified_duality: false,
        }
    }
    
    pub fn verify_duality(&mut self) -> Result<(), SessionError> {
        // For simplicity, we only verify binary sessions for now
        if self.roles.len() == 2 {
            let role1 = &self.roles[0];
            let role2 = &self.roles[1];
            
            if verify_duality(&role1.protocol, &role2.protocol) {
                self.verified_duality = true;
                Ok(())
            } else {
                Err(SessionError::DualityMismatch {
                    session_name: self.name.clone(),
                    role1: role1.name.clone(),
                    role2: role2.name.clone(),
                })
            }
        } else {
            // For multi-party sessions, we'd need more sophisticated checking
            self.verified_duality = true; // Assume valid for now
            Ok(())
        }
    }
    
    pub fn get_role_protocol(&self, role_name: &str) -> Option<&SessionType> {
        self.roles.iter()
            .find(|role| role.name == role_name)
            .map(|role| &role.protocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::BaseType;
    
    #[test]
    fn test_duality_computation() {
        let send_int = SessionType::Send(
            TypeInner::Base(BaseType::Int), 
            Box::new(SessionType::End)
        );
        let recv_int = SessionType::Receive(
            TypeInner::Base(BaseType::Int),
            Box::new(SessionType::End)
        );
        
        assert_eq!(compute_dual(&send_int), recv_int);
        assert_eq!(compute_dual(&recv_int), send_int);
    }
    
    #[test]
    fn test_duality_involution() {
        let original = SessionType::Send(
            TypeInner::Base(BaseType::Int),
            Box::new(SessionType::Receive(
                TypeInner::Base(BaseType::Symbol),
                Box::new(SessionType::End)
            ))
        );
        
        let dual_of_dual = compute_dual(&compute_dual(&original));
        assert_eq!(original, dual_of_dual);
    }
    
    #[test]
    fn test_session_declaration_duality() {
        let client_role = SessionRole {
            name: "client".to_string(),
            protocol: SessionType::Send(
                TypeInner::Base(BaseType::Int),
                Box::new(SessionType::Receive(
                    TypeInner::Base(BaseType::Symbol),
                    Box::new(SessionType::End)
                ))
            ),
        };
        
        let server_role = SessionRole {
            name: "server".to_string(),
            protocol: SessionType::Receive(
                TypeInner::Base(BaseType::Int),
                Box::new(SessionType::Send(
                    TypeInner::Base(BaseType::Symbol),
                    Box::new(SessionType::End)
                ))
            ),
        };
        
        let mut session = SessionDeclaration::new(
            "PaymentProtocol".to_string(),
            vec![client_role, server_role]
        );
        
        assert!(session.verify_duality().is_ok());
        assert!(session.verified_duality);
    }
    
    #[test]
    fn test_well_formed_session_types() {
        let valid_session = SessionType::Send(
            TypeInner::Base(BaseType::Int),
            Box::new(SessionType::End)
        );
        
        assert!(is_well_formed(&valid_session).is_ok());
        
        let empty_choice = SessionType::InternalChoice(vec![]);
        assert!(is_well_formed(&empty_choice).is_err());
    }
    
    #[test]
    fn test_session_progress() {
        let session_type = SessionType::Send(
            TypeInner::Base(BaseType::Int),
            Box::new(SessionType::End)
        );
        
        let operation = SessionOperation::Send(Value::Int(42));
        let result = progress_session(&session_type, &operation);
        
        assert_eq!(result.unwrap(), SessionType::End);
    }
} 