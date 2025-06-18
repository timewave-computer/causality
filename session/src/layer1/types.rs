// Layer 1 type system - types for messages and sessions

use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Basic types that can be contained in messages
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Unit type (no information)
    Unit,
    
    /// Boolean type
    Bool,
    
    /// Integer type
    Int,
    
    /// Product type (pairs)
    Product(Box<Type>, Box<Type>),
    
    /// Sum type (either/or)
    Sum(Box<Type>, Box<Type>),
    
    /// Record with row type (unified with messages)
    Record(RowType),
    
    /// Session with protocol S
    Session(Box<SessionType>),
}

/// Row types for extensible records
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RowType {
    /// Empty row
    Empty,
    
    /// Row extension: label, type, rest of row
    Extend(String, Box<Type>, Box<RowType>),
    
    /// Row variable (for polymorphism)
    RowVar(String),
}

/// Session types describe communication protocols
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionType {
    /// Send a value of type T, then continue as S
    Send(Box<Type>, Box<SessionType>),
    
    /// Receive a value of type T, then continue as S
    Receive(Box<Type>, Box<SessionType>),
    
    /// Internal choice - we choose which branch to take
    InternalChoice(Vec<(String, SessionType)>),
    
    /// External choice - other party chooses which branch
    ExternalChoice(Vec<(String, SessionType)>),
    
    /// End of communication
    End,
    
    /// Recursive session type
    Recursive(String, Box<SessionType>),
    
    /// Session variable (for recursion)
    Variable(String),
}

impl Type {
    /// Check if two types are equal
    pub fn equals(&self, other: &Type) -> bool {
        self == other
    }
    
    /// Get the size of a type (for allocation)
    pub fn size(&self) -> usize {
        match self {
            Type::Unit => 0,
            Type::Bool => 1,
            Type::Int => 8,
            Type::Product(t1, t2) => t1.size() + t2.size(),
            Type::Sum(t1, t2) => 1 + std::cmp::max(t1.size(), t2.size()),
            Type::Record(_) => 32, // Content-addressed message size (SHA256)
            Type::Session(_) => 8,  // Channel ID size
        }
    }
}

impl SessionType {
    /// Compute the dual of a session type
    pub fn dual(&self) -> SessionType {
        match self {
            SessionType::Send(t, s) => {
                SessionType::Receive(t.clone(), Box::new(s.dual()))
            }
            SessionType::Receive(t, s) => {
                SessionType::Send(t.clone(), Box::new(s.dual()))
            }
            SessionType::InternalChoice(branches) => {
                SessionType::ExternalChoice(
                    branches.iter()
                        .map(|(label, session)| (label.clone(), session.dual()))
                        .collect()
                )
            }
            SessionType::ExternalChoice(branches) => {
                SessionType::InternalChoice(
                    branches.iter()
                        .map(|(label, session)| (label.clone(), session.dual()))
                        .collect()
                )
            }
            SessionType::End => SessionType::End,
            SessionType::Recursive(var, body) => {
                // For simplicity, we'll handle recursion naively
                // In a full implementation, we'd need to substitute properly
                SessionType::Recursive(var.clone(), Box::new(body.dual()))
            }
            SessionType::Variable(var) => SessionType::Variable(var.clone()),
        }
    }
    
    /// Check if this session type is dual to another
    pub fn is_dual_to(&self, other: &SessionType) -> bool {
        self == &other.dual()
    }
}

impl RowType {
    /// Create a row type from a list of fields
    pub fn from_fields(fields: Vec<(String, Type)>) -> Self {
        fields.into_iter()
            .rev()
            .fold(RowType::Empty, |rest, (label, ty)| {
                RowType::Extend(label, Box::new(ty), Box::new(rest))
            })
    }
    
    /// Check if this row type has a field
    pub fn has_field(&self, label: &str) -> bool {
        match self {
            RowType::Empty => false,
            RowType::Extend(l, _, rest) => {
                l == label || rest.has_field(label)
            }
            RowType::RowVar(_) => false, // Conservative: unknown row vars don't have fields
        }
    }
    
    /// Get the type of a field if it exists
    pub fn get_field_type(&self, label: &str) -> Option<&Type> {
        match self {
            RowType::Empty => None,
            RowType::Extend(l, ty, rest) => {
                if l == label {
                    Some(ty)
                } else {
                    rest.get_field_type(label)
                }
            }
            RowType::RowVar(_) => None,
        }
    }
    
    /// Convert to a map for easier manipulation
    pub fn to_field_map(&self) -> HashMap<String, Type> {
        match self {
            RowType::Empty => HashMap::new(),
            RowType::Extend(label, ty, rest) => {
                let mut map = rest.to_field_map();
                map.insert(label.clone(), (**ty).clone());
                map
            }
            RowType::RowVar(_) => HashMap::new(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unit => write!(f, "Unit"),
            Type::Bool => write!(f, "Bool"),
            Type::Int => write!(f, "Int"),
            Type::Product(t1, t2) => write!(f, "({} × {})", t1, t2),
            Type::Sum(t1, t2) => write!(f, "({} + {})", t1, t2),
            Type::Record(row) => write!(f, "Record<{}>", row),
            Type::Session(s) => write!(f, "Session<{}>", s),
        }
    }
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionType::Send(t, s) => write!(f, "!{}.{}", t, s),
            SessionType::Receive(t, s) => write!(f, "?{}.{}", t, s),
            SessionType::InternalChoice(branches) => {
                write!(f, "⊕{{")?;
                for (i, (label, _)) in branches.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", label)?;
                }
                write!(f, "}}")
            }
            SessionType::ExternalChoice(branches) => {
                write!(f, "&{{")?;
                for (i, (label, _)) in branches.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", label)?;
                }
                write!(f, "}}")
            }
            SessionType::End => write!(f, "End"),
            SessionType::Recursive(var, body) => write!(f, "rec {}.{}", var, body),
            SessionType::Variable(var) => write!(f, "{}", var),
        }
    }
}

impl fmt::Display for RowType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RowType::Empty => write!(f, "Empty"),
            RowType::Extend(label, ty, rest) => write!(f, "{}:{} + {}", label, ty, rest),
            RowType::RowVar(var) => write!(f, "{}", var),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_equality() {
        let t1 = Type::Product(Box::new(Type::Int), Box::new(Type::Bool));
        let t2 = Type::Product(Box::new(Type::Int), Box::new(Type::Bool));
        let t3 = Type::Sum(Box::new(Type::Int), Box::new(Type::Bool));
        
        assert!(t1.equals(&t2));
        assert!(!t1.equals(&t3));
    }
    
    #[test]
    fn test_session_duality() {
        // Simple send/receive duality
        let s1 = SessionType::Send(
            Box::new(Type::Int),
            Box::new(SessionType::End)
        );
        let s2 = SessionType::Receive(
            Box::new(Type::Int),
            Box::new(SessionType::End)
        );
        
        assert!(s1.is_dual_to(&s2));
        assert!(s2.is_dual_to(&s1));
        
        // Choice duality
        let choice1 = SessionType::InternalChoice(vec![
            ("left".to_string(), SessionType::End),
            ("right".to_string(), SessionType::End),
        ]);
        let choice2 = SessionType::ExternalChoice(vec![
            ("left".to_string(), SessionType::End),
            ("right".to_string(), SessionType::End),
        ]);
        
        assert!(choice1.is_dual_to(&choice2));
    }
    
    #[test]
    fn test_complex_duality() {
        // Protocol: send int, receive bool, end
        let protocol = SessionType::Send(
            Box::new(Type::Int),
            Box::new(SessionType::Receive(
                Box::new(Type::Bool),
                Box::new(SessionType::End)
            ))
        );
        
        let dual = SessionType::Receive(
            Box::new(Type::Int),
            Box::new(SessionType::Send(
                Box::new(Type::Bool),
                Box::new(SessionType::End)
            ))
        );
        
        assert_eq!(protocol.dual(), dual);
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;
    
    #[test]
    fn test_canonical_serialization_determinism() {
        // Create a complex type structure
        let session_type = SessionType::Send(
            Box::new(Type::Record(RowType::from_fields(vec![
                ("amount".to_string(), Type::Int),
                ("recipient".to_string(), Type::Record(RowType::from_fields(vec![
                    ("address".to_string(), Type::Int),
                ]))),
            ]))),
            Box::new(SessionType::Receive(
                Box::new(Type::Bool),
                Box::new(SessionType::End)
            ))
        );
        
        // Serialize multiple times
        let bytes1 = bincode::serialize(&session_type).unwrap();
        let bytes2 = bincode::serialize(&session_type).unwrap();
        let bytes3 = bincode::serialize(&session_type.clone()).unwrap();
        
        // All should be identical (deterministic serialization)
        assert_eq!(bytes1, bytes2);
        assert_eq!(bytes2, bytes3);
        
        // Deserialize and verify round-trip
        let decoded: SessionType = bincode::deserialize(&bytes1).unwrap();
        assert_eq!(session_type, decoded);
        
        // Re-serialize decoded value
        let bytes4 = bincode::serialize(&decoded).unwrap();
        assert_eq!(bytes1, bytes4);
    }
    
    #[test]
    fn test_canonical_field_ordering() {
        // Create two record types with fields in different orders
        let record1 = Type::Record(RowType::Extend(
            "a".to_string(),
            Box::new(Type::Int),
            Box::new(RowType::Extend(
                "b".to_string(),
                Box::new(Type::Bool),
                Box::new(RowType::Empty)
            ))
        ));
        
        let record2 = Type::Record(RowType::Extend(
            "b".to_string(),
            Box::new(Type::Bool),
            Box::new(RowType::Extend(
                "a".to_string(),
                Box::new(Type::Int),
                Box::new(RowType::Empty)
            ))
        ));
        
        let bytes1 = bincode::serialize(&record1).unwrap();
        let bytes2 = bincode::serialize(&record2).unwrap();
        
        // Different field order should produce different serialization
        // (This ensures we're actually serializing the structure, not just content)
        assert_ne!(bytes1, bytes2);
    }
    
    #[test]
    fn test_type_serialization_coverage() {
        let types = vec![
            Type::Unit,
            Type::Bool,
            Type::Int,
            Type::Product(Box::new(Type::Int), Box::new(Type::Bool)),
            Type::Sum(Box::new(Type::Int), Box::new(Type::Unit)),
            Type::Record(RowType::Empty),
            Type::Session(Box::new(SessionType::End)),
        ];
        
        for ty in types {
            let encoded = bincode::serialize(&ty).unwrap();
            let decoded: Type = bincode::deserialize(&encoded).unwrap();
            assert_eq!(ty, decoded);
        }
    }
}
