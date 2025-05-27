//! Program Implementation
//!
//! Contains the implementation for registered programs that track
//! program versions and compilation state.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

// Standard library import
use std::collections::{HashSet, VecDeque};

// External dependencies
use causality_types::serialization::{SimpleSerialize, Encode, Decode, DecodeError};

// Internal import
use crate::ids::{CircuitId, ProgramId};

/// Program compilation stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationStage {
    /// Initial program registration
    Registered,

    /// Program successfully compiled
    Compiled,

    /// Program optimization completed
    Optimized,

    /// Program has been deployed
    Deployed,
}

//-----------------------------------------------------------------------------
// RegisteredProgram
//-----------------------------------------------------------------------------

/// A program registered in the compiler registry

#[derive(Debug, Clone)]
pub struct RegisteredProgram {
    /// The current version of the program
    current: Program,

    /// Previous versions of the program (most recent first)
    history: VecDeque<Program>,

    /// Current compilation stage
    compilation_stage: CompilationStage,

    /// Maximum number of versions to keep in history
    max_history: usize,
}

impl RegisteredProgram {
    /// Creates a new registered program
    pub fn new(program: Program) -> Self {
        Self {
            current: program,
            history: VecDeque::new(),
            compilation_stage: CompilationStage::Registered,
            max_history: 10, // Default history size
        }
    }

    /// Gets the current version of the program
    pub fn current_version(&self) -> &Program {
        &self.current
    }

    /// Gets the program history
    pub fn history(&self) -> &VecDeque<Program> {
        &self.history
    }

    /// Gets the current compilation stage
    pub fn compilation_stage(&self) -> CompilationStage {
        self.compilation_stage
    }

    /// Updates the program with a new version
    pub fn update_program(&mut self, new_program: Program) {
        // Move current to history
        self.history.push_front(self.current.clone());

        // Trim history if needed
        while self.history.len() > self.max_history {
            self.history.pop_back();
        }

        // Set the new current version
        self.current = new_program;
    }

    /// Sets the compilation stage
    pub fn set_compilation_stage(&mut self, stage: CompilationStage) {
        self.compilation_stage = stage;
    }

    /// Sets the maximum history size
    pub fn set_max_history(&mut self, max_history: usize) {
        self.max_history = max_history;

        // Trim history if needed
        while self.history.len() > self.max_history {
            self.history.pop_back();
        }
    }

    /// Reverts to a previous version
    pub fn revert_to_previous(&mut self) -> Option<Program> {
        if let Some(previous) = self.history.pop_front() {
            let old_current = std::mem::replace(&mut self.current, previous);
            Some(old_current)
        } else {
            None
        }
    }
}

//-----------------------------------------------------------------------------
// Program Definition
//-----------------------------------------------------------------------------

/// Represents a complete, compiled program consisting of multiple circuits.
/// Minimal version for initial end-to-end flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    /// Deterministic ID based on the included circuits.
    pub id: ProgramId,
    /// Set of Circuit IDs included in this program.
    /// Using HashSet for quick lookups, but must be sorted for ID generation.
    pub circuit_ids: HashSet<CircuitId>,
    // Note: Removed graph_data, resources, expressions fields for minimal plan.
}

//-----------------------------------------------------------------------------
// Program Implementation
//-----------------------------------------------------------------------------

impl SimpleSerialize for Program {}

impl Encode for Program {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.as_ssz_bytes());
        // Convert HashSet to Vec for deterministic serialization
        let mut circuit_ids: Vec<_> = self.circuit_ids.iter().collect();
        circuit_ids.sort();
        bytes.extend((circuit_ids.len() as u64).as_ssz_bytes());
        for circuit_id in circuit_ids {
            bytes.extend(circuit_id.as_ssz_bytes());
        }
        bytes
    }
}

impl Decode for Program {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode ProgramId
        let id_size = std::mem::size_of::<ProgramId>();
        if bytes.len() < offset + id_size {
            return Err(DecodeError::new(&format!("Invalid byte length: got {}, expected at least {}", bytes.len(), offset + id_size)));
        }
        let id = ProgramId::from_ssz_bytes(&bytes[offset..offset + id_size])?;
        offset += id_size;
        
        // Decode circuit_ids length
        let len_size = std::mem::size_of::<u64>();
        if bytes.len() < offset + len_size {
            return Err(DecodeError::new(&format!("Invalid byte length: got {}, expected at least {}", bytes.len(), offset + len_size)));
        }
        let circuit_ids_len = u64::from_ssz_bytes(&bytes[offset..offset + len_size])?;
        offset += len_size;
        
        // Decode circuit_ids
        let mut circuit_ids = HashSet::new();
        let circuit_id_size = std::mem::size_of::<CircuitId>();
        for _ in 0..circuit_ids_len {
            if bytes.len() < offset + circuit_id_size {
                return Err(DecodeError::new(&format!("Invalid byte length: got {}, expected at least {}", bytes.len(), offset + circuit_id_size)));
            }
            let circuit_id = CircuitId::from_ssz_bytes(&bytes[offset..offset + circuit_id_size])?;
            circuit_ids.insert(circuit_id);
            offset += circuit_id_size;
        }
        
        Ok(Program { id, circuit_ids })
    }
}

impl Program {
    /// Creates a new minimal Program with the given ID and circuit IDs
    pub fn new(id: ProgramId, circuit_ids: HashSet<CircuitId>) -> Self {
        Self { id, circuit_ids }
    }
    // Note: Removed add_resource, add_expression methods for minimal plan.
}
