//! Circuit Definition
//!
//! This module defines the Circuit structure that represents a compiled unit
//! containing a subgraph and its associated expressions.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

// Standard library import
use std::collections::HashSet;

// External dependencies
use causality_types::serialization::{SimpleSerialize, Encode, Decode, DecodeError};

// Internal import
use crate::ids::CircuitId;
use causality_types::primitive::ids::ExprId;

//-----------------------------------------------------------------------------
// Circuit Structure
//-----------------------------------------------------------------------------

/// Represents a compiled unit containing a subgraph and its associated expressions.
///
/// A Circuit is the fundamental compilation unit in the Causality system, containing
/// a serialized subgraph and the expressions used within it. Each Circuit has a
/// deterministic ID derived from its content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Circuit {
    /// Deterministic ID based on subgraph and expression content.
    pub id: CircuitId,

    /// Serialized bytes of the subgraph structure.
    pub subgraph_bytes: Vec<u8>,

    /// Set of expression IDs used within this circuit's subgraph.
    /// Using HashSet for quick lookups, but must be sorted for ID generation.
    pub expression_ids: HashSet<ExprId>,

    /// Placeholder: Assume a single main constraint expression ID for the circuit for now.
    pub main_constraint_expr_id: Option<ExprId>,
}

//-----------------------------------------------------------------------------
// Circuit Implementation
//-----------------------------------------------------------------------------

impl SimpleSerialize for Circuit {}

impl Encode for Circuit {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.as_ssz_bytes());
        bytes.extend(self.subgraph_bytes.as_ssz_bytes());
        // Convert HashSet to Vec for deterministic serialization
        let mut expr_ids: Vec<_> = self.expression_ids.iter().collect();
        expr_ids.sort();
        bytes.extend((expr_ids.len() as u64).as_ssz_bytes());
        for expr_id in expr_ids {
            bytes.extend(expr_id.as_ssz_bytes());
        }
        if let Some(main_expr) = &self.main_constraint_expr_id {
            bytes.push(1); // Some flag
            bytes.extend(main_expr.as_ssz_bytes());
        } else {
            bytes.push(0); // None flag
        }
        bytes
    }
}

impl Decode for Circuit {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode CircuitId
        let id_size = std::mem::size_of::<CircuitId>();
        if bytes.len() < offset + id_size {
            return Err(DecodeError::new(&format!("Invalid byte length: got {}, expected at least {}", bytes.len(), offset + id_size)));
        }
        let id = CircuitId::from_ssz_bytes(&bytes[offset..offset + id_size])?;
        offset += id_size;
        
        // Decode subgraph_bytes
        let subgraph_bytes = Vec::<u8>::from_ssz_bytes(&bytes[offset..])?;
        // Calculate how many bytes were consumed by the Vec<u8> decoding
        let subgraph_bytes_encoded = subgraph_bytes.as_ssz_bytes();
        offset += subgraph_bytes_encoded.len();
        
        // Decode expression_ids length
        let len_size = std::mem::size_of::<u64>();
        if bytes.len() < offset + len_size {
            return Err(DecodeError::new(&format!("Invalid byte length: got {}, expected at least {}", bytes.len(), offset + len_size)));
        }
        let expr_ids_len = u64::from_ssz_bytes(&bytes[offset..offset + len_size])?;
        offset += len_size;
        
        // Decode expression_ids
        let mut expression_ids = HashSet::new();
        let expr_id_size = std::mem::size_of::<ExprId>();
        for _ in 0..expr_ids_len {
            if bytes.len() < offset + expr_id_size {
                return Err(DecodeError::new(&format!("Invalid byte length: got {}, expected at least {}", bytes.len(), offset + expr_id_size)));
            }
            let expr_id = ExprId::from_ssz_bytes(&bytes[offset..offset + expr_id_size])?;
            expression_ids.insert(expr_id);
            offset += expr_id_size;
        }
        
        // Decode main_constraint_expr_id
        if bytes.len() < offset + 1 {
            return Err(DecodeError::new(&format!("Invalid byte length: got {}, expected at least {}", bytes.len(), offset + 1)));
        }
        let has_main_expr = bytes[offset] == 1;
        offset += 1;
        
        let main_constraint_expr_id = if has_main_expr {
            if bytes.len() < offset + expr_id_size {
                return Err(DecodeError::new(&format!("Invalid byte length: got {}, expected at least {}", bytes.len(), offset + expr_id_size)));
            }
            let main_expr = ExprId::from_ssz_bytes(&bytes[offset..offset + expr_id_size])?;
            Some(main_expr)
        } else {
            None
        };
        
        Ok(Circuit {
            id,
            subgraph_bytes,
            expression_ids,
            main_constraint_expr_id,
        })
    }
}

impl Circuit {
    /// Create a new Circuit with the provided components
    pub fn new(
        id: CircuitId,
        subgraph_bytes: Vec<u8>,
        expression_ids: HashSet<ExprId>,
        main_constraint_expr_id: Option<ExprId>,
    ) -> Self {
        Self {
            id,
            subgraph_bytes,
            expression_ids,
            main_constraint_expr_id,
        }
    }
}
