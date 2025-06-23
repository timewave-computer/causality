//! Fixed register file for ZK-VM compatible execution
//!
//! This module implements a fixed-size register file using array-based storage
//! for predictable memory access patterns required by zero-knowledge virtual machines.
//!
//! **Design Principles**:
//! - Fixed register count (no dynamic allocation)
//! - Array-based storage for predictable access patterns
//! - Deterministic register allocation and recycling
//! - Bounded resource usage for ZK proof generation
//! - Linear resource discipline enforcement

use crate::{
    machine::instruction::RegisterId,
    machine::resource::ResourceId,
    system::deterministic::DeterministicSystem,
};
use serde::{Serialize, Deserialize};
use std::collections::{BTreeSet, BTreeMap};

//-----------------------------------------------------------------------------
// Configuration Constants
//-----------------------------------------------------------------------------

/// Maximum number of registers in the register file
/// This is a compile-time constant for ZK-VM compatibility
pub const MAX_REGISTERS: usize = 1024;

/// Register allocation pool size
/// Tracks which registers are available for allocation
pub const ALLOCATION_POOL_SIZE: usize = MAX_REGISTERS;

//-----------------------------------------------------------------------------
// Register File Implementation
//-----------------------------------------------------------------------------

/// Fixed-size register file for deterministic execution
#[derive(Debug, Clone)]
pub struct RegisterFile {
    /// Fixed-size array of register slots
    registers: [Option<ResourceId>; MAX_REGISTERS],
    
    /// Set of available register IDs for allocation
    /// Uses BTreeSet for deterministic ordering
    available_registers: BTreeSet<u32>,
    
    /// Set of allocated register IDs
    /// Tracks which registers are currently in use
    allocated_registers: BTreeSet<u32>,
    
    /// Next register ID for deterministic allocation
    /// Monotonically increasing counter
    next_register_id: u32,
}

impl RegisterFile {
    /// Create a new empty register file
    pub fn new() -> Self {
        let mut available_registers = BTreeSet::new();
        
        // Initialize all registers as available
        for i in 0..MAX_REGISTERS as u32 {
            available_registers.insert(i);
        }
        
        Self {
            registers: [None; MAX_REGISTERS],
            available_registers,
            allocated_registers: BTreeSet::new(),
            next_register_id: 0,
        }
    }
    
    /// Allocate a new register deterministically
    /// Returns None if no registers are available
    pub fn allocate_register(&mut self, _det_sys: &mut DeterministicSystem) -> Option<RegisterId> {
        // Get the smallest available register ID for deterministic allocation
        if let Some(&register_id) = self.available_registers.iter().next() {
            self.available_registers.remove(&register_id);
            self.allocated_registers.insert(register_id);
            Some(RegisterId::new(register_id))
        } else {
            None
        }
    }
    
    /// Free a register and make it available for reuse
    pub fn free_register(&mut self, reg_id: RegisterId) -> Result<(), RegisterFileError> {
        let id = reg_id.id();
        
        // Validate register ID bounds
        if id >= MAX_REGISTERS as u32 {
            return Err(RegisterFileError::InvalidRegisterId(id));
        }
        
        // Check if register is actually allocated
        if !self.allocated_registers.contains(&id) {
            return Err(RegisterFileError::RegisterNotAllocated(id));
        }
        
        // Clear the register contents
        self.registers[id as usize] = None;
        
        // Move from allocated to available
        self.allocated_registers.remove(&id);
        self.available_registers.insert(id);
        
        Ok(())
    }
    
    /// Read a register value
    pub fn read_register(&self, register: RegisterId) -> Result<Option<ResourceId>, RegisterFileError> {
        if register.id() as usize >= MAX_REGISTERS {
            return Err(RegisterFileError::InvalidRegister(register.id()));
        }
        
        // Check if register is allocated
        if !self.allocated_registers.contains(&register.id()) {
            return Err(RegisterFileError::RegisterNotAllocated(register.id()));
        }
        
        Ok(self.registers[register.id() as usize])
    }
    
    /// Write a resource ID to a register
    pub fn write_register(&mut self, reg_id: RegisterId, resource_id: Option<ResourceId>) -> Result<(), RegisterFileError> {
        let id = reg_id.id();
        
        // Validate register ID bounds
        if id >= MAX_REGISTERS as u32 {
            return Err(RegisterFileError::InvalidRegisterId(id));
        }
        
        // Check if register is allocated
        if !self.allocated_registers.contains(&id) {
            return Err(RegisterFileError::RegisterNotAllocated(id));
        }
        
        self.registers[id as usize] = resource_id;
        Ok(())
    }
    
    /// Get the number of allocated registers
    pub fn allocated_count(&self) -> usize {
        self.allocated_registers.len()
    }
    
    /// Get the number of available registers
    pub fn available_count(&self) -> usize {
        self.available_registers.len()
    }
    
    /// Check if the register file is full
    pub fn is_full(&self) -> bool {
        self.allocated_registers.len() >= MAX_REGISTERS
    }
    
    /// Check if a register is allocated
    pub fn is_allocated(&self, reg_id: RegisterId) -> bool {
        let id = reg_id.id();
        id < MAX_REGISTERS as u32 && self.allocated_registers.contains(&id)
    }
    
    /// Get all allocated register IDs
    pub fn allocated_registers(&self) -> Vec<RegisterId> {
        self.allocated_registers
            .iter()
            .map(|&id| RegisterId::new(id))
            .collect()
    }
    
    /// Create a snapshot of the current register file state
    pub fn snapshot(&self) -> RegisterFileSnapshot {
        RegisterFileSnapshot {
            register_contents: self.registers,
            allocated_registers: self.allocated_registers.clone(),
            next_register_id: self.next_register_id,
        }
    }
    
    /// Restore register file from a snapshot
    pub fn restore_from_snapshot(&mut self, snapshot: RegisterFileSnapshot) {
        self.registers = snapshot.register_contents;
        self.allocated_registers = snapshot.allocated_registers;
        self.next_register_id = snapshot.next_register_id;
        
        // Rebuild available registers set
        self.available_registers.clear();
        for i in 0..MAX_REGISTERS as u32 {
            if !self.allocated_registers.contains(&i) {
                self.available_registers.insert(i);
            }
        }
    }
    
    /// Get usage statistics for optimization
    pub fn get_usage_stats(&self) -> BTreeMap<RegisterId, RegisterUsageStats> {
        // For now, return basic stats
        // In a full implementation, this would track actual usage patterns
        let mut stats = BTreeMap::new();
        
        for &reg_id in &self.allocated_registers {
            stats.insert(RegisterId::new(reg_id), RegisterUsageStats {
                allocation_count: 1,
                read_count: 0,
                write_count: 0,
                last_used: 0,
                coalescable: true,
            });
        }
        
        stats
    }
    
    /// Optimize register allocation to minimize pressure
    pub fn optimize_allocation(&mut self, _det_sys: &mut DeterministicSystem) -> Result<Vec<RegisterId>, RegisterFileError> {
        let mut optimized_registers = Vec::new();
        
        // Find registers that can be freed based on usage patterns
        let stats = self.get_usage_stats();
        let mut candidates_for_freeing = Vec::new();
        
        for (reg_id, stat) in stats {
            // If register hasn't been used recently and has low usage, consider freeing
            if stat.read_count == 0 && stat.write_count == 0 && stat.coalescable {
                candidates_for_freeing.push(reg_id);
            }
        }
        
        // Free unused registers to reduce pressure
        for reg_id in candidates_for_freeing {
            if self.is_allocated(reg_id) {
                // Only free if the register is actually empty
                if let Ok(resource_opt) = self.read_register(reg_id) {
                    if resource_opt.is_none() {
                        self.free_register(reg_id)?;
                        optimized_registers.push(reg_id);
                    }
                }
            }
        }
        
        Ok(optimized_registers)
    }
    
    /// Find register coalescing opportunities
    pub fn find_coalescing_candidates(&self) -> Vec<CoalescingCandidate> {
        let mut candidates = Vec::new();
        let stats = self.get_usage_stats();
        
        // Look for registers that could be merged
        let allocated_regs: Vec<_> = self.allocated_registers.iter().collect();
        
        for (i, &&reg1_id) in allocated_regs.iter().enumerate() {
            for &&reg2_id in allocated_regs.iter().skip(i + 1) {
                let reg1 = RegisterId::new(reg1_id);
                let reg2 = RegisterId::new(reg2_id);
                
                // Check if these registers are candidates for coalescing
                if let (Some(stats1), Some(stats2)) = (stats.get(&reg1), stats.get(&reg2)) {
                    if stats1.coalescable && stats2.coalescable {
                        // Calculate benefit score based on usage patterns
                        let benefit = self.calculate_coalescing_benefit(&reg1, &reg2, stats1, stats2);
                        
                        if benefit > 0 {
                            candidates.push(CoalescingCandidate {
                                register: reg1,
                                merge_target: reg2,
                                benefit_score: benefit,
                            });
                        }
                    }
                }
            }
        }
        
        // Sort by benefit score (highest first)
        candidates.sort_by(|a, b| b.benefit_score.cmp(&a.benefit_score));
        candidates
    }
    
    /// Calculate the benefit of coalescing two registers
    fn calculate_coalescing_benefit(&self, reg1: &RegisterId, reg2: &RegisterId, stats1: &RegisterUsageStats, stats2: &RegisterUsageStats) -> u64 {
        // Simple heuristic: benefit is higher for registers with similar usage patterns
        let usage_similarity = if stats1.read_count == stats2.read_count && stats1.write_count == stats2.write_count {
            10
        } else {
            0
        };
        
        // Benefit is higher for less frequently used registers
        let frequency_factor = 20 - (stats1.read_count + stats1.write_count + stats2.read_count + stats2.write_count).min(20);
        
        // Check if both registers contain compatible values
        let compatibility_bonus = if self.are_registers_compatible(reg1, reg2) {
            15
        } else {
            0
        };
        
        usage_similarity + frequency_factor + compatibility_bonus
    }
    
    /// Check if two registers contain compatible values for coalescing
    fn are_registers_compatible(&self, reg1: &RegisterId, reg2: &RegisterId) -> bool {
        // For now, assume compatibility if both are empty or both contain resources
        match (self.read_register(*reg1), self.read_register(*reg2)) {
            (Ok(None), Ok(None)) => true, // Both empty
            (Ok(Some(_)), Ok(Some(_))) => true, // Both have resources (could check type compatibility)
            _ => false,
        }
    }
    
    /// Perform register coalescing optimization
    pub fn coalesce_registers(&mut self, candidates: &[CoalescingCandidate]) -> Result<usize, RegisterFileError> {
        let mut coalesced_count = 0;
        
        for candidate in candidates {
            if self.is_allocated(candidate.register) && self.is_allocated(candidate.merge_target) && self.are_registers_compatible(&candidate.register, &candidate.merge_target) {
                // Perform the coalescing by moving content from register to merge_target
                if let Ok(Some(resource)) = self.read_register(candidate.register) {
                    // If merge_target is empty, move the resource there
                    if let Ok(None) = self.read_register(candidate.merge_target) {
                        self.write_register(candidate.merge_target, Some(resource))?;
                        self.write_register(candidate.register, None)?;
                        self.free_register(candidate.register)?;
                        coalesced_count += 1;
                    }
                } else if let Ok(None) = self.read_register(candidate.register) {
                    // Source register is empty, just free it
                    self.free_register(candidate.register)?;
                    coalesced_count += 1;
                }
            }
        }
        
        Ok(coalesced_count)
    }
    
    /// Get register pressure (percentage of registers in use)
    pub fn register_pressure(&self) -> f64 {
        self.allocated_count() as f64 / MAX_REGISTERS as f64
    }
    
    /// Check if register pressure is high and optimization is needed
    pub fn needs_optimization(&self) -> bool {
        self.register_pressure() > 0.8 // 80% threshold
    }
    
    /// Perform comprehensive register optimization
    pub fn optimize(&mut self, _det_sys: &mut DeterministicSystem) -> Result<OptimizationResult, RegisterFileError> {
        let initial_allocated = self.allocated_count();
        let initial_pressure = self.register_pressure();
        
        // Step 1: Basic allocation optimization
        let freed_registers = self.optimize_allocation(_det_sys)?;
        
        // Step 2: Register coalescing
        let candidates = self.find_coalescing_candidates();
        let coalesced_count = self.coalesce_registers(&candidates)?;
        
        let final_allocated = self.allocated_count();
        let final_pressure = self.register_pressure();
        
        Ok(OptimizationResult {
            initial_allocated_count: initial_allocated,
            final_allocated_count: final_allocated,
            freed_count: freed_registers.len(),
            coalesced_count,
            initial_pressure,
            final_pressure,
            pressure_reduction: initial_pressure - final_pressure,
        })
    }
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self::new()
    }
}

//-----------------------------------------------------------------------------
// Register File Snapshot
//-----------------------------------------------------------------------------

/// Snapshot of register file state for execution tracing
#[derive(Debug, Clone)]
pub struct RegisterFileSnapshot {
    /// Contents of all registers at snapshot time
    pub register_contents: [Option<ResourceId>; MAX_REGISTERS],
    
    /// Set of allocated register IDs
    pub allocated_registers: BTreeSet<u32>,
    
    /// Next register ID counter
    pub next_register_id: u32,
}

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

/// Errors that can occur during register file operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegisterFileError {
    /// Register ID is out of bounds
    InvalidRegisterId(u32),
    
    /// Attempted to access unallocated register
    RegisterNotAllocated(u32),
    
    /// No registers available for allocation
    NoRegistersAvailable,
    
    /// Register file is full
    RegisterFileFull,
    
    /// Invalid register access
    InvalidRegister(u32),
}

impl std::fmt::Display for RegisterFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterFileError::InvalidRegisterId(id) => {
                write!(f, "Invalid register ID: {} (max: {})", id, MAX_REGISTERS - 1)
            }
            RegisterFileError::RegisterNotAllocated(id) => {
                write!(f, "Register {} is not allocated", id)
            }
            RegisterFileError::NoRegistersAvailable => {
                write!(f, "No registers available for allocation")
            }
            RegisterFileError::RegisterFileFull => {
                write!(f, "Register file is full ({} registers)", MAX_REGISTERS)
            }
            RegisterFileError::InvalidRegister(id) => {
                write!(f, "Invalid register access: {}", id)
            }
        }
    }
}

impl std::error::Error for RegisterFileError {}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_allocation() {
        let mut register_file = RegisterFile::new();
        let mut det_sys = DeterministicSystem::new();
        
        // Should be able to allocate registers
        let reg1 = register_file.allocate_register(&mut det_sys).unwrap();
        let reg2 = register_file.allocate_register(&mut det_sys).unwrap();
        
        assert_ne!(reg1, reg2);
        assert_eq!(register_file.allocated_count(), 2);
        assert_eq!(register_file.available_count(), MAX_REGISTERS - 2);
    }
    
    #[test]
    fn test_register_read_write() {
        let mut register_file = RegisterFile::new();
        let mut det_sys = DeterministicSystem::new();
        
        let reg = register_file.allocate_register(&mut det_sys).unwrap();
        let resource_id = ResourceId::new(42);
        
        // Write and read back
        register_file.write_register(reg, Some(resource_id)).unwrap();
        let read_value = register_file.read_register(reg).unwrap();
        
        assert_eq!(read_value, Some(resource_id));
    }
    
    #[test]
    fn test_register_free() {
        let mut register_file = RegisterFile::new();
        let mut det_sys = DeterministicSystem::new();
        
        let reg = register_file.allocate_register(&mut det_sys).unwrap();
        assert_eq!(register_file.allocated_count(), 1);
        
        // Free the register
        register_file.free_register(reg).unwrap();
        assert_eq!(register_file.allocated_count(), 0);
        assert_eq!(register_file.available_count(), MAX_REGISTERS);
    }
    
    #[test]
    fn test_invalid_register_access() {
        let register_file = RegisterFile::new();
        let invalid_reg = RegisterId::new(MAX_REGISTERS as u32);
        
        // Should fail with invalid register ID
        assert!(matches!(
            register_file.read_register(invalid_reg),
            Err(RegisterFileError::InvalidRegister(_))
        ));
    }
    
    #[test]
    fn test_unallocated_register_access() {
        let register_file = RegisterFile::new();
        let reg = RegisterId::new(0); // Valid ID but not allocated
        
        // Should fail with register not allocated
        assert!(matches!(
            register_file.read_register(reg),
            Err(RegisterFileError::RegisterNotAllocated(_))
        ));
    }
}

/// Register usage statistics for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUsageStats {
    /// How many times this register has been allocated
    pub allocation_count: u64,
    
    /// How many times this register has been read
    pub read_count: u64,
    
    /// How many times this register has been written
    pub write_count: u64,
    
    /// Last time this register was used
    pub last_used: u64,
    
    /// Whether this register is a candidate for coalescing
    pub coalescable: bool,
}

/// Register coalescing candidate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoalescingCandidate {
    /// The register that could be coalesced
    pub register: RegisterId,
    
    /// The register it could be merged with
    pub merge_target: RegisterId,
    
    /// Estimated benefit of coalescing (higher is better)
    pub benefit_score: u64,
}

/// Result of register optimization
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub initial_allocated_count: usize,
    pub final_allocated_count: usize,
    pub freed_count: usize,
    pub coalesced_count: usize,
    pub initial_pressure: f64,
    pub final_pressure: f64,
    pub pressure_reduction: f64,
} 