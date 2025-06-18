// Layer 2 Outcome structure - declarative state changes with proofs

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::fmt;

/// A declarative outcome representing intended state changes
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// List of state transitions to apply
    pub declarations: Vec<StateTransition>,
    
    /// Proof of validity (stub for now)
    pub proof: Proof,
    
    /// Content-addressed commitment
    pub commitment: [u8; 32],
}

/// Types of state transitions
#[derive(Debug, Clone, PartialEq)]
pub enum StateTransition {
    /// Transfer resources between addresses
    Transfer {
        from: Address,
        to: Address,
        amount: u64,
        resource_type: ResourceType,
    },
    
    /// Update a state location
    Update {
        location: StateLocation,
        old_value: Value,
        new_value: Value,
    },
    
    /// Create new state
    Create {
        location: StateLocation,
        value: Value,
    },
    
    /// Delete existing state
    Delete {
        location: StateLocation,
    },
}

/// Address identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address(pub String);

/// Resource type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceType(pub String);

/// State location identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct StateLocation(pub String);

/// Generic value type
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Value {
    Unit,
    Bool(bool),
    Int(i64),
    String(String),
    Bytes(Vec<u8>),
    Struct(Vec<(String, Value)>),
    Address(Address),
}

/// Proof type (stub for now)
#[derive(Debug, Clone, PartialEq)]
pub struct Proof {
    /// Proof data (would be ZK proof in real implementation)
    pub data: Vec<u8>,
}

impl Outcome {
    /// Create an empty outcome
    pub fn empty() -> Self {
        let empty_proof = Proof {
            data: vec![],
        };
        
        let mut outcome = Outcome {
            declarations: vec![],
            proof: empty_proof,
            commitment: [0; 32],
        };
        
        outcome.commitment = outcome.compute_commitment();
        outcome
    }
    
    /// Create an outcome with a single transition
    pub fn single(transition: StateTransition) -> Self {
        let mut outcome = Outcome {
            declarations: vec![transition],
            proof: Proof {
                data: vec![],
            },
            commitment: [0; 32],
        };
        
        outcome.commitment = outcome.compute_commitment();
        outcome
    }
    
    /// Check if outcome is empty
    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }
    
    /// Compose two outcomes
    pub fn compose(self, other: Outcome) -> Self {
        let mut declarations = self.declarations;
        declarations.extend(other.declarations);
        
        // Combine proofs (simplified)
        let mut proof_data = self.proof.data;
        proof_data.extend(other.proof.data);
        
        let mut result = Outcome {
            declarations,
            proof: Proof { data: proof_data },
            commitment: [0; 32],
        };
        
        result.commitment = result.compute_commitment();
        result
    }
    
    /// Compute content-addressed commitment
    fn compute_commitment(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        // Hash each declaration
        for decl in &self.declarations {
            hasher.update(format!("{:?}", decl).as_bytes());
        }
        
        // Hash the proof
        hasher.update(&self.proof.data);
        
        hasher.finalize().into()
    }
    
    /// Verify the outcome's proof
    pub fn verify(&self) -> bool {
        // Stub verification - in real implementation would verify ZK proof
        !self.proof.data.is_empty() || self.declarations.is_empty()
    }
}

impl Value {
    // Note: to_string is provided by Display trait implementation
}

/// Implement the outcome algebra operations
impl std::ops::Add for Outcome {
    type Output = Outcome;
    
    fn add(self, rhs: Outcome) -> Outcome {
        self.compose(rhs)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bytes(b) => write!(f, "{:?}", b),
            Value::Struct(fields) => {
                write!(f, "{{")?;
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", name, value)?;
                }
                write!(f, "}}")
            }
            Value::Address(a) => write!(f, "{}", a.0),
        }
    }
}

impl fmt::Display for StateTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateTransition::Transfer { from, to, amount, resource_type } => {
                write!(f, "Transfer {} {} from {} to {}", amount, resource_type.0, from.0, to.0)
            }
            StateTransition::Update { location, .. } => {
                write!(f, "Update {}", location.0)
            }
            StateTransition::Create { location, .. } => {
                write!(f, "Create {}", location.0)
            }
            StateTransition::Delete { location } => {
                write!(f, "Delete {}", location.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_outcome() {
        let outcome = Outcome::empty();
        assert!(outcome.declarations.is_empty());
        assert!(outcome.verify());
    }
    
    #[test]
    fn test_single_transition() {
        let transition = StateTransition::Transfer {
            from: Address("Alice".to_string()),
            to: Address("Bob".to_string()),
            amount: 100,
            resource_type: ResourceType("Token".to_string()),
        };
        
        let outcome = Outcome::single(transition.clone());
        assert_eq!(outcome.declarations.len(), 1);
        assert_eq!(outcome.declarations[0], transition);
    }
    
    #[test]
    fn test_outcome_composition() {
        let t1 = StateTransition::Create {
            location: StateLocation("account/alice".to_string()),
            value: Value::Int(1000),
        };
        
        let t2 = StateTransition::Transfer {
            from: Address("Alice".to_string()),
            to: Address("Bob".to_string()),
            amount: 100,
            resource_type: ResourceType("Token".to_string()),
        };
        
        let o1 = Outcome::single(t1);
        let o2 = Outcome::single(t2);
        
        let composed = o1 + o2;
        assert_eq!(composed.declarations.len(), 2);
    }
    
    #[test]
    fn test_commitment_changes() {
        let mut outcome = Outcome::empty();
        let initial_commitment = outcome.commitment;
        
        outcome.declarations.push(StateTransition::Create {
            location: StateLocation("test".to_string()),
            value: Value::Bool(true),
        });
        outcome.commitment = outcome.compute_commitment();
        
        assert_ne!(outcome.commitment, initial_commitment);
    }
}
