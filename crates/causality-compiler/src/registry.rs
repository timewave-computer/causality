//! Program Registry
//!
//! Provides a registry for tracking compiled programs and their constituent parts.
//! The registry maintains information about program versions, compilation status,
//! and dependencies.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

// Import circuit and program related type
use crate::circuit::Circuit;
use crate::ids::{CircuitId, ProgramId};
use crate::program::Program;

use causality_types::{
    core::id::DomainId,
    serialization::{Decode, Encode},
};

//-----------------------------------------------------------------------------
// Registry Error Types
//-----------------------------------------------------------------------------

/// Errors that can occur when interacting with a registry
#[derive(Debug, Error)]
pub enum RegistryError {
    /// The requested item was not found
    #[error("Item not found: {0}")]
    NotFound(String),

    /// An item with this ID already exists
    #[error("Item already exists: {0}")]
    Conflict(String),

    /// An error occurred while accessing storage
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

//-----------------------------------------------------------------------------
// Error Codes and Messages
//-----------------------------------------------------------------------------

/// Error codes and messages for registry operations
pub mod registry_errors {
    pub const PROGRAM_NOT_FOUND: u32 = 5101;
    pub const CIRCUIT_NOT_FOUND: u32 = 5102;
    pub const OPERATION_FAILED: u32 = 5103;

    pub fn program_not_found(id: &str) -> String {
        format!("Program not found: {}", id)
    }

    pub fn circuit_not_found(id: &str) -> String {
        format!("Circuit not found: {}", id)
    }

    pub fn operation_failed(msg: &str) -> String {
        format!("Operation failed: {}", msg)
    }
}

//-----------------------------------------------------------------------------
// Storage Backend
//-----------------------------------------------------------------------------

/// Storage backend for ProgramRegistry
#[derive(Debug)]
pub enum RegistryStorageBackend {
    /// Traditional HashMap storage
    HashMap {
        programs: Arc<RwLock<HashMap<ProgramId, Program>>>,
        circuits: Arc<RwLock<HashMap<CircuitId, Circuit>>>,
    },
    /// SMT-backed storage with domain awareness
    Smt {
        smt: Arc<parking_lot::Mutex<causality_core::smt::TegMultiDomainSmt<causality_core::smt::MemoryBackend>>>,
        domain_id: DomainId,
    },
}

impl Default for RegistryStorageBackend {
    fn default() -> Self {
        Self::HashMap {
            programs: Arc::new(RwLock::new(HashMap::new())),
            circuits: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Clone for RegistryStorageBackend {
    fn clone(&self) -> Self {
        match self {
            Self::HashMap { programs, circuits } => Self::HashMap {
                programs: Arc::clone(programs),
                circuits: Arc::clone(circuits),
            },
            Self::Smt { smt, domain_id } => Self::Smt {
                smt: Arc::clone(smt),
                domain_id: *domain_id,
            },
        }
    }
}

//-----------------------------------------------------------------------------
// ProgramRegistry
//-----------------------------------------------------------------------------

/// A simple, in-memory registry for compiled Programs and Circuits.
#[derive(Debug, Clone, Default)]
pub struct ProgramRegistry {
    storage: RegistryStorageBackend,
}

impl ProgramRegistry {
    /// Create a new registry with HashMap storage
    pub fn new() -> Self {
        Self {
            storage: RegistryStorageBackend::default(),
        }
    }

    /// Create a new registry with SMT storage
    pub fn new_with_smt(domain_id: DomainId) -> Self {
        let backend = causality_core::smt::MemoryBackend::new();
        let smt = causality_core::smt::TegMultiDomainSmt::new(backend);
        Self {
            storage: RegistryStorageBackend::Smt {
                smt: Arc::new(parking_lot::Mutex::new(smt)),
                domain_id,
            },
        }
    }

    /// Insert a new program, fails if ID already exists
    pub fn insert(&mut self, id: ProgramId, program: Program) -> Result<()> {
        match &mut self.storage {
            RegistryStorageBackend::HashMap { programs, .. } => {
                let mut programs = programs
                    .write()
                    .map_err(|_| anyhow!("Failed to acquire write lock"))?;

                if programs.contains_key(&id) {
                    return Err(anyhow!("Program with ID {:?} already exists", id));
                }

                programs.insert(id, program);
            }
            RegistryStorageBackend::Smt { smt, domain_id } => {
                let program_key = format!("{}-program-{}", domain_id.namespace_prefix(), id);
                
                // Check if program already exists
                if smt.lock().has_data(&program_key) {
                    return Err(anyhow!("Program with ID {:?} already exists", id));
                }
                
                // Store program data in SMT
                let program_data = program.as_ssz_bytes();
                smt.lock().store_data(&program_key, &program_data)
                    .map_err(|e| anyhow!("Failed to store program: {}", e))?;
            }
        }
        Ok(())
    }

    /// Get a program by its ID
    pub fn get(&self, id: &ProgramId) -> Result<Program> {
        match &self.storage {
            RegistryStorageBackend::HashMap { programs, .. } => {
                let programs = programs
                    .read()
                    .map_err(|_| anyhow!("Failed to acquire read lock"))?;

                programs
                    .get(id)
                    .cloned()
                    .ok_or_else(|| anyhow!("Program with ID {:?} not found", id))
            }
            RegistryStorageBackend::Smt { smt, domain_id } => {
                let program_key = format!("{}-program-{}", domain_id.namespace_prefix(), id);
                if let Ok(Some(program_data)) = smt.lock().get_data(&program_key) {
                    // Deserialize program from SSZ bytes
                    match <Program as Decode>::from_ssz_bytes(&program_data) {
                        Ok(program) => Ok(program),
                        Err(e) => Err(anyhow!("Failed to deserialize program: {}", e)),
                    }
                } else {
                    Err(anyhow!("Program with ID {:?} not found", id))
                }
            }
        }
    }

    /// Remove a program by ID
    pub fn remove(&mut self, id: &ProgramId) -> Result<Program> {
        match &mut self.storage {
            RegistryStorageBackend::HashMap { programs, .. } => {
                let mut programs = programs
                    .write()
                    .map_err(|_| anyhow!("Failed to acquire write lock"))?;

                programs
                    .remove(id)
                    .ok_or_else(|| anyhow!("Program with ID {:?} not found for removal", id))
            }
            RegistryStorageBackend::Smt { smt, domain_id } => {
                // Implement SMT program removal
                let program_key = format!("{}-program-{}", domain_id.namespace_prefix(), id);
                
                // First, get the program to return it
                if let Ok(Some(program_data)) = smt.lock().get_data(&program_key) {
                    match <Program as Decode>::from_ssz_bytes(&program_data) {
                        Ok(program) => {
                            // Remove the program by storing empty data
                            smt.lock().store_data(&program_key, &[])
                                .map_err(|e| anyhow!("Failed to remove program: {}", e))?;
                            Ok(program)
                        }
                        Err(e) => Err(anyhow!("Failed to deserialize program for removal: {}", e)),
                    }
                } else {
                    Err(anyhow!("Program with ID {:?} not found for removal", id))
                }
            }
        }
    }

    /// Check if a program exists
    pub fn contains(&self, id: &ProgramId) -> bool {
        match &self.storage {
            RegistryStorageBackend::HashMap { programs, .. } => {
                if let Ok(programs) = programs.read() {
                    programs.contains_key(id)
                } else {
                    false
                }
            }
            RegistryStorageBackend::Smt { smt, domain_id } => {
                let program_key = format!("{}-program-{}", domain_id.namespace_prefix(), id);
                smt.lock().has_data(&program_key)
            }
        }
    }

    /// Count of programs in the registry
    pub fn count(&self) -> usize {
        match &self.storage {
            RegistryStorageBackend::HashMap { programs, .. } => {
                if let Ok(programs) = programs.read() {
                    programs.len()
                } else {
                    0
                }
            }
            RegistryStorageBackend::Smt { smt: _, domain_id: _ } => {
                // Count programs by checking keys with the domain prefix
                // For now, return 0 as SMT doesn't expose key iteration
                // In a full implementation, we'd maintain a separate counter
                0
            }
        }
    }

    /// Clear all programs
    pub fn clear(&mut self) -> Result<()> {
        match &mut self.storage {
            RegistryStorageBackend::HashMap { programs, circuits } => {
                let mut programs = programs
                    .write()
                    .map_err(|_| anyhow!("Failed to acquire write lock"))?;
                let mut circuits = circuits
                    .write()
                    .map_err(|_| anyhow!("Failed to acquire write lock"))?;
                programs.clear();
                circuits.clear();
            }
            RegistryStorageBackend::Smt { smt: _, domain_id: _ } => {
                // Clear all programs and circuits by clearing all keys with the domain prefix
                // For now, we'll just return success as SMT doesn't expose bulk operations
                // In a full implementation, we'd iterate through all keys and remove them
                
            }
        }
        Ok(())
    }

    /// Insert a circuit into the registry
    pub fn insert_circuit(
        &mut self,
        circuit: Circuit,
    ) -> Result<CircuitId, RegistryError> {
        let id = circuit.id;
        
        match &mut self.storage {
            RegistryStorageBackend::HashMap { circuits, .. } => {
                let mut circuits = circuits.write().map_err(|_| {
                    RegistryError::StorageError(
                        "Failed to acquire circuit write lock".to_string(),
                    )
                })?;

                if circuits.contains_key(&id) {
                    return Err(RegistryError::Conflict(format!(
                        "Circuit with ID {:?} already exists",
                        id
                    )));
                }
                circuits.insert(id, circuit);
            }
            RegistryStorageBackend::Smt { smt, domain_id } => {
                let circuit_key = format!("{}-circuit-{}", domain_id.namespace_prefix(), id);
                
                // Check if circuit already exists
                if smt.lock().has_data(&circuit_key) {
                    return Err(RegistryError::Conflict(format!(
                        "Circuit with ID {:?} already exists",
                        id
                    )));
                }
                
                // Store circuit data in SMT
                let circuit_data = circuit.as_ssz_bytes();
                smt.lock().store_data(&circuit_key, &circuit_data)
                    .map_err(|e| RegistryError::StorageError(format!("Failed to store circuit: {}", e)))?;
            }
        }
        Ok(id)
    }

    /// Get a circuit by its ID
    pub fn get_circuit(&self, id: &CircuitId) -> Result<Circuit, RegistryError> {
        match &self.storage {
            RegistryStorageBackend::HashMap { circuits, .. } => {
                let circuits = circuits.read().map_err(|_| {
                    RegistryError::StorageError(
                        "Failed to acquire circuit read lock".to_string(),
                    )
                })?;
                circuits.get(id).cloned().ok_or_else(|| {
                    RegistryError::NotFound(format!("Circuit with ID {:?} not found", id))
                })
            }
            RegistryStorageBackend::Smt { smt, domain_id } => {
                let circuit_key = format!("{}-circuit-{}", domain_id.namespace_prefix(), id);
                if let Ok(Some(circuit_data)) = smt.lock().get_data(&circuit_key) {
                    // Deserialize circuit from SSZ bytes
                    match <Circuit as Decode>::from_ssz_bytes(&circuit_data) {
                        Ok(circuit) => Ok(circuit),
                        Err(e) => Err(RegistryError::StorageError(format!("Failed to deserialize circuit: {}", e))),
                    }
                } else {
                    Err(RegistryError::NotFound(format!("Circuit with ID {:?} not found", id)))
                }
            }
        }
    }

    /// List all program IDs
    pub fn list_program_ids(&self) -> Vec<ProgramId> {
        match &self.storage {
            RegistryStorageBackend::HashMap { programs, .. } => {
                if let Ok(programs) = programs.read() {
                    programs.keys().cloned().collect()
                } else {
                    Vec::new()
                }
            }
            RegistryStorageBackend::Smt { .. } => {
                // TODO: Implement SMT iteration for program IDs
                Vec::new()
            }
        }
    }

    /// List all programs
    pub fn list_programs(&self) -> Vec<Program> {
        match &self.storage {
            RegistryStorageBackend::HashMap { programs, .. } => {
                if let Ok(programs) = programs.read() {
                    programs.values().cloned().collect()
                } else {
                    Vec::new()
                }
            }
            RegistryStorageBackend::Smt { .. } => {
                // TODO: Implement SMT iteration for programs
                Vec::new()
            }
        }
    }

    /// Get all programs matching a domain
    pub fn get_programs_by_domain(&self, _domain: &DomainId) -> Vec<Program> {
        // TODO: Implement domain filtering
        self.list_programs()
    }
}
