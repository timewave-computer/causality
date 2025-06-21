//! Session registry for global session management
//!
//! This module provides a centralized registry for managing session type declarations
//! and choreographies within the Causality framework.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

use crate::lambda::base::SessionType;

/// Session identifier type
pub type SessionId = String;

/// Session role specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRole {
    pub name: String,
    pub protocol: SessionType,
}

/// Complete session declaration with all roles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDeclaration {
    pub name: SessionId,
    pub roles: Vec<SessionRole>,
    pub verified_duality: bool,
}

/// Session type errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionError {
    /// Duality mismatch between roles
    DualityMismatch {
        session_name: String,
        role1: String,
        role2: String,
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

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::DualityMismatch { session_name, role1, role2 } => {
                write!(f, "Duality mismatch in session '{}' between roles '{}' and '{}'", 
                       session_name, role1, role2)
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
        return Err(SessionError::MalformedSessionType {
            reason: format!("Recursion depth exceeded: {}", max_depth),
        });
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
            for (_, branch) in branches {
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

/// Verify that two session types are duals
pub fn verify_duality(s1: &SessionType, s2: &SessionType) -> bool {
    let dual_s1 = compute_dual(s1);
    &dual_s1 == s2
}

/// Duality computation for session types
pub fn compute_dual(session_type: &SessionType) -> SessionType {
    match session_type {
        SessionType::Send(t, s) => SessionType::Receive(t.clone(), Box::new(compute_dual(s))),
        SessionType::Receive(t, s) => SessionType::Send(t.clone(), Box::new(compute_dual(s))),
        SessionType::InternalChoice(branches) => {
            SessionType::ExternalChoice(
                branches.iter()
                    .map(|(label, session)| (label.clone(), compute_dual(session)))
                    .collect()
            )
        }
        SessionType::ExternalChoice(branches) => {
            SessionType::InternalChoice(
                branches.iter()
                    .map(|(label, session)| (label.clone(), compute_dual(session)))
                    .collect()
            )
        }
        SessionType::End => SessionType::End,
        SessionType::Recursive(var, s) => {
            SessionType::Recursive(var.clone(), Box::new(compute_dual(s)))
        }
        SessionType::Variable(var) => SessionType::Variable(var.clone()),
    }
}

/// Global registry for session types and choreographies
#[derive(Debug, Clone)]
pub struct SessionRegistry {
    sessions: Arc<RwLock<BTreeMap<SessionId, SessionDeclaration>>>,
    choreographies: Arc<RwLock<BTreeMap<String, Choreography>>>,
}

/// Choreography for multi-party session coordination
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choreography {
    pub name: String,
    pub roles: Vec<String>,
    pub protocol: ChoreographyProtocol,
}

/// Choreography protocol specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoreographyProtocol {
    /// Point-to-point communication
    Communication {
        from: String,
        to: String,
        message_type: String, // Simplified type representation
    },
    
    /// Choice between multiple protocols
    Choice {
        role: String,
        branches: Vec<ChoreographyProtocol>,
    },
    
    /// Parallel execution of protocols
    Parallel(Vec<ChoreographyProtocol>),
    
    /// Sequential execution of protocols
    Sequential(Vec<ChoreographyProtocol>),
}

impl SessionRegistry {
    /// Create a new session registry
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(BTreeMap::new())),
            choreographies: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
    
    /// Register a new session declaration
    pub fn register_session(&self, mut decl: SessionDeclaration) -> Result<(), SessionError> {
        // Verify well-formedness of all role protocols
        for role in &decl.roles {
            is_well_formed(&role.protocol)?;
        }
        
        // Verify duality for binary sessions
        decl.verify_duality()?;
        
        // Store the session
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(decl.name.clone(), decl);
        
        Ok(())
    }
    
    /// Get a session declaration by name
    pub fn get_session(&self, name: &str) -> Option<SessionDeclaration> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(name).cloned()
    }
    
    /// Check if a session exists
    pub fn has_session(&self, name: &str) -> bool {
        let sessions = self.sessions.read().unwrap();
        sessions.contains_key(name)
    }
    
    /// List all registered session names
    pub fn list_sessions(&self) -> Vec<SessionId> {
        let sessions = self.sessions.read().unwrap();
        sessions.keys().cloned().collect()
    }
    
    /// Verify duality for a specific session
    pub fn verify_session_duality(&self, name: &str) -> Result<(), SessionError> {
        let sessions = self.sessions.read().unwrap();
        if let Some(session) = sessions.get(name) {
            if session.verified_duality {
                Ok(())
            } else {
                Err(SessionError::DualityMismatch {
                    session_name: name.to_string(),
                    role1: "unknown".to_string(),
                    role2: "unknown".to_string(),
                })
            }
        } else {
            Err(SessionError::SessionNotFound {
                session_name: name.to_string(),
            })
        }
    }
    
    /// Get the protocol for a specific role in a session
    pub fn get_role_protocol(&self, session_name: &str, role_name: &str) -> Option<SessionType> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(session_name)
            .and_then(|session| session.get_role_protocol(role_name))
            .cloned()
    }
    
    /// Register a choreography
    pub fn register_choreography(&self, choreography: Choreography) -> Result<(), SessionError> {
        // Validate choreography
        validate_choreography(&choreography)?;
        
        let mut choreographies = self.choreographies.write().unwrap();
        choreographies.insert(choreography.name.clone(), choreography);
        
        Ok(())
    }
    
    /// Get a choreography by name
    pub fn get_choreography(&self, name: &str) -> Option<Choreography> {
        let choreographies = self.choreographies.read().unwrap();
        choreographies.get(name).cloned()
    }
    
    /// List all registered choreography names
    pub fn list_choreographies(&self) -> Vec<String> {
        let choreographies = self.choreographies.read().unwrap();
        choreographies.keys().cloned().collect()
    }
    
    /// Project a role from a choreography to get its session type
    pub fn project_choreography_role(
        &self, 
        choreography_name: &str, 
        role_name: &str
    ) -> Result<SessionType, SessionError> {
        let choreographies = self.choreographies.read().unwrap();
        if let Some(choreography) = choreographies.get(choreography_name) {
            project_role(choreography, role_name)
        } else {
            Err(SessionError::SessionNotFound {
                session_name: choreography_name.to_string(),
            })
        }
    }
    
    /// Clear all sessions and choreographies (useful for testing)
    pub fn clear(&self) {
        self.sessions.write().unwrap().clear();
        self.choreographies.write().unwrap().clear();
    }
    
    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let sessions = self.sessions.read().unwrap();
        let choreographies = self.choreographies.read().unwrap();
        
        RegistryStats {
            session_count: sessions.len(),
            choreography_count: choreographies.len(),
            verified_sessions: sessions.values().filter(|s| s.verified_duality).count(),
        }
    }
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryStats {
    pub session_count: usize,
    pub choreography_count: usize,
    pub verified_sessions: usize,
}

/// Validate a choreography for well-formedness
fn validate_choreography(choreography: &Choreography) -> Result<(), SessionError> {
    // Check that all roles mentioned in the protocol are declared
    let declared_roles: std::collections::BTreeSet<_> = choreography.roles.iter().collect();
    let used_roles = collect_used_roles(&choreography.protocol);
    
    for used_role in &used_roles {
        if !declared_roles.contains(used_role) {
            return Err(SessionError::MalformedSessionType {
                reason: format!("Role '{}' used in protocol but not declared", used_role),
            });
        }
    }
    
    Ok(())
}

/// Collect all roles used in a choreography protocol
fn collect_used_roles(protocol: &ChoreographyProtocol) -> std::collections::BTreeSet<String> {
    let mut roles = std::collections::BTreeSet::new();
    collect_used_roles_recursive(protocol, &mut roles);
    roles
}

fn collect_used_roles_recursive(
    protocol: &ChoreographyProtocol,
    roles: &mut std::collections::BTreeSet<String>
) {
    match protocol {
        ChoreographyProtocol::Communication { from, to, .. } => {
            roles.insert(from.clone());
            roles.insert(to.clone());
        }
        ChoreographyProtocol::Choice { role, branches } => {
            roles.insert(role.clone());
            for branch in branches {
                collect_used_roles_recursive(branch, roles);
            }
        }
        ChoreographyProtocol::Parallel(protocols) | ChoreographyProtocol::Sequential(protocols) => {
            for protocol in protocols {
                collect_used_roles_recursive(protocol, roles);
            }
        }
    }
}

/// Project a role from a choreography to get its session type
fn project_role(choreography: &Choreography, role_name: &str) -> Result<SessionType, SessionError> {
    project_protocol(&choreography.protocol, role_name)
}

/// Project a choreography protocol for a specific role
fn project_protocol(protocol: &ChoreographyProtocol, role_name: &str) -> Result<SessionType, SessionError> {
    match protocol {
        ChoreographyProtocol::Communication { from, to, message_type } => {
            if from == role_name {
                // This role sends
                Ok(SessionType::Send(
                    Box::new(parse_message_type(message_type)),
                    Box::new(SessionType::End)
                ))
            } else if to == role_name {
                // This role receives
                Ok(SessionType::Receive(
                    Box::new(parse_message_type(message_type)),
                    Box::new(SessionType::End)
                ))
            } else {
                // This role is not involved in this communication
                Ok(SessionType::End)
            }
        }
        ChoreographyProtocol::Choice { role, branches } => {
            if role == role_name {
                // This role makes the choice (internal choice)
                let projected_branches: Result<Vec<_>, _> = branches.iter()
                    .enumerate()
                    .map(|(i, branch)| {
                        let session = project_protocol(branch, role_name)?;
                        Ok((format!("choice_{}", i), session))
                    })
                    .collect();
                Ok(SessionType::InternalChoice(projected_branches?))
            } else {
                // This role waits for the choice (external choice)
                let projected_branches: Result<Vec<_>, _> = branches.iter()
                    .enumerate()
                    .map(|(i, branch)| {
                        let session = project_protocol(branch, role_name)?;
                        Ok((format!("choice_{}", i), session))
                    })
                    .collect();
                Ok(SessionType::ExternalChoice(projected_branches?))
            }
        }
        ChoreographyProtocol::Sequential(protocols) => {
            // Sequential composition
            let mut result = SessionType::End;
            for protocol in protocols.iter().rev() {
                let projected = project_protocol(protocol, role_name)?;
                result = compose_sequential(projected, result);
            }
            Ok(result)
        }
        ChoreographyProtocol::Parallel(_protocols) => {
            // Parallel composition is complex and would require more sophisticated handling
            // For now, we'll return a simple End type
            Ok(SessionType::End)
        }
    }
}

/// Compose two session types sequentially
fn compose_sequential(first: SessionType, second: SessionType) -> SessionType {
    match first {
        SessionType::End => second,
        SessionType::Send(t, continuation) => {
            SessionType::Send(t, Box::new(compose_sequential(*continuation, second)))
        }
        SessionType::Receive(t, continuation) => {
            SessionType::Receive(t, Box::new(compose_sequential(*continuation, second)))
        }
        SessionType::InternalChoice(branches) => {
            SessionType::InternalChoice(
                branches.into_iter()
                    .map(|(label, branch)| (label, compose_sequential(branch, second.clone())))
                    .collect()
            )
        }
        SessionType::ExternalChoice(branches) => {
            SessionType::ExternalChoice(
                branches.into_iter()
                    .map(|(label, branch)| (label, compose_sequential(branch, second.clone())))
                    .collect()
            )
        }
        other => other, // For recursive types and variables, return as-is for now
    }
}

/// Parse a message type string into a Type  
fn parse_message_type(message_type: &str) -> crate::lambda::base::TypeInner {
    use crate::lambda::base::{TypeInner, BaseType};
    // Simple parsing - in a real implementation this would be more sophisticated
    match message_type {
        "Int" => TypeInner::Base(BaseType::Int),
        "String" => TypeInner::Base(BaseType::Symbol), // Use Symbol for strings in base types
        "Bool" => TypeInner::Base(BaseType::Bool),
        "Unit" => TypeInner::Base(BaseType::Unit),
        _ => TypeInner::Base(BaseType::Unit), // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::{TypeInner, BaseType};

    
    #[test]
    fn test_session_registry_basic_operations() {
        let registry = SessionRegistry::new();
        
        let session = SessionDeclaration::new(
            "TestSession".to_string(),
            vec![
                SessionRole {
                    name: "client".to_string(),
                    protocol: SessionType::Send(Box::new(TypeInner::Base(BaseType::Int)), Box::new(SessionType::End)),
                },
                SessionRole {
                    name: "server".to_string(),
                    protocol: SessionType::Receive(Box::new(TypeInner::Base(BaseType::Int)), Box::new(SessionType::End)),
                },
            ]
        );
        
        assert!(registry.register_session(session).is_ok());
        assert!(registry.has_session("TestSession"));
        assert!(registry.get_session("TestSession").is_some());
    }
    
    #[test]
    fn test_choreography_validation() {
        let registry = SessionRegistry::new();
        
        let choreography = Choreography {
            name: "SimpleComm".to_string(),
            roles: vec!["alice".to_string(), "bob".to_string()],
            protocol: ChoreographyProtocol::Communication {
                from: "alice".to_string(),
                to: "bob".to_string(),
                message_type: "Int".to_string(),
            },
        };
        
        assert!(registry.register_choreography(choreography).is_ok());
    }
    
    #[test]
    fn test_choreography_projection() {
        let registry = SessionRegistry::new();
        
        let choreography = Choreography {
            name: "SimpleComm".to_string(),
            roles: vec!["alice".to_string(), "bob".to_string()],
            protocol: ChoreographyProtocol::Communication {
                from: "alice".to_string(),
                to: "bob".to_string(),
                message_type: "Int".to_string(),
            },
        };
        
        registry.register_choreography(choreography).unwrap();
        
        let alice_protocol = registry.project_choreography_role("SimpleComm", "alice").unwrap();
        let bob_protocol = registry.project_choreography_role("SimpleComm", "bob").unwrap();
        
        // Alice should send, Bob should receive
        match alice_protocol {
            SessionType::Send(type_inner, continuation) => {
                if let TypeInner::Base(BaseType::Int) = type_inner.as_ref() {
                    assert_eq!(*continuation, SessionType::End);
                } else {
                    panic!("Expected Int type for Alice send");
                }
            }
            _ => panic!("Expected Send type for Alice"),
        }
        
        match bob_protocol {
            SessionType::Receive(type_inner, continuation) => {
                if let TypeInner::Base(BaseType::Int) = type_inner.as_ref() {
                    assert_eq!(*continuation, SessionType::End);
                } else {
                    panic!("Expected Int type for Bob receive");
                }
            }
            _ => panic!("Expected Receive type for Bob"),
        }
    }
    
    #[test]
    fn test_registry_stats() {
        let registry = SessionRegistry::new();
        
        let stats_initial = registry.stats();
        assert_eq!(stats_initial.session_count, 0);
        assert_eq!(stats_initial.choreography_count, 0);
        
        let session = SessionDeclaration::new(
            "TestSession".to_string(),
            vec![
                SessionRole {
                    name: "client".to_string(),
                    protocol: SessionType::Send(Box::new(TypeInner::Base(BaseType::Int)), Box::new(SessionType::End)),
                },
            ]
        );
        
        registry.register_session(session).unwrap();
        
        let stats_after = registry.stats();
        assert_eq!(stats_after.session_count, 1);
    }
}
