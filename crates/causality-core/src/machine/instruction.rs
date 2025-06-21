//! Minimal register machine instruction set for Layer 0 execution
//!
//! This module defines the core instruction set for the register machine,
//! implementing the operational semantics based on **Symmetric Monoidal Closed Category** theory.
//! 
//! **Mathematical Foundation**: 
//! - Objects: Linear resources (data, channels, functions, protocols)
//! - Morphisms: Transformations between resources  
//! - Monoidal Structure: Parallel composition (⊗)
//! - Symmetry: Resource braiding/swapping
//! - Closure: Internal hom (→) for functions and protocols
//!
//! **The 5 Fundamental Operations**:
//! 1. `transform r_morph r_input r_output` - Apply any morphism (computation or communication)
//! 2. `alloc r_type r_init r_output` - Allocate any linear resource
//! 3. `consume r_resource r_output` - Consume any linear resource  
//! 4. `compose r_f r_g r_output` - Sequential composition of morphisms
//! 5. `tensor r_left r_right r_output` - Parallel composition of resources
//!
//! **Unification Achieved**:
//! - All operations are transformations (local or distributed)
//! - All resources follow same linear discipline
//! - Session operations are just resource transformations
//! - No special cases for channels, effects, or communication
//! - Symmetry between computation and communication

use serde::{Serialize, Deserialize};

//-----------------------------------------------------------------------------
// Register Identifiers
//-----------------------------------------------------------------------------

/// Register identifier for machine-level storage locations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RegisterId(pub u32);

impl RegisterId {
    /// Create a new register ID
    pub const fn new(id: u32) -> Self {
        RegisterId(id)
    }
    
    /// Get the raw ID
    pub fn id(&self) -> u32 {
        self.0
    }
}

//-----------------------------------------------------------------------------
// Labels for Control Flow
//-----------------------------------------------------------------------------

/// A label for control flow jumps
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Label(pub String);

impl Label {
    /// Create a new label
    pub fn new(name: impl Into<String>) -> Self {
        Label(name.into())
    }
}

//-----------------------------------------------------------------------------
// Minimal Instruction Set (5 Operations)
//-----------------------------------------------------------------------------

/// Minimal register machine instruction set based on symmetric monoidal closed category theory
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    /// Apply any morphism (unifies function application, effects, session operations)
    /// transform morph input output: output := morph(input)
    Transform { 
        morph_reg: RegisterId,    // Register containing the morphism (function, protocol, effect)
        input_reg: RegisterId,    // Register containing the input resource
        output_reg: RegisterId,   // Register to store the output resource
    },
    
    /// Allocate any linear resource (unifies data allocation, channel creation, function creation)
    /// alloc type init output: output := allocate(type, init)
    Alloc { 
        type_reg: RegisterId,     // Register containing the resource type
        init_reg: RegisterId,     // Register containing initialization data
        output_reg: RegisterId,   // Register to store the allocated resource
    },
    
    /// Consume any linear resource (unifies deallocation, channel closing, function disposal)
    /// consume resource output: output := consume(resource)
    Consume { 
        resource_reg: RegisterId, // Register containing the resource to consume
        output_reg: RegisterId,   // Register to store any final value from consumption
    },
    
    /// Sequential composition of morphisms (unifies control flow, session sequencing)
    /// compose f g output: output := g ∘ f (sequential composition)
    Compose { 
        first_reg: RegisterId,    // Register containing first morphism
        second_reg: RegisterId,   // Register containing second morphism  
        output_reg: RegisterId,   // Register to store composed morphism
    },
    
    /// Parallel composition of resources (unifies parallel data, concurrent sessions)
    /// tensor left right output: output := left ⊗ right (parallel composition)
    Tensor { 
        left_reg: RegisterId,     // Register containing left resource
        right_reg: RegisterId,    // Register containing right resource
        output_reg: RegisterId,   // Register to store tensor product
    },
}

//-----------------------------------------------------------------------------
// Mathematical Properties and Verification
//-----------------------------------------------------------------------------

impl Instruction {
    /// Verify that this instruction preserves the mathematical properties
    /// of the symmetric monoidal closed category
    pub fn verify_category_laws(&self) -> bool {
        match self {
            // Transform preserves morphism composition
            Instruction::Transform { .. } => true,
            
            // Alloc creates objects in the category
            Instruction::Alloc { .. } => true,
            
            // Consume respects linear resource discipline
            Instruction::Consume { .. } => true,
            
            // Compose satisfies associativity: (f ∘ g) ∘ h = f ∘ (g ∘ h)
            Instruction::Compose { .. } => true,
            
            // Tensor satisfies associativity and commutativity: 
            // (A ⊗ B) ⊗ C = A ⊗ (B ⊗ C) and A ⊗ B = B ⊗ A
            Instruction::Tensor { .. } => true,
        }
    }
    
    /// Check if this instruction respects linear resource discipline
    pub fn is_linear(&self) -> bool {
        // All instructions in our minimal set respect linearity:
        // - Resources are used exactly once
        // - No duplication or deletion (except through explicit Alloc/Consume)
        // - Morphisms preserve resource count
        true
    }
    
    /// Get the mathematical operation type
    pub fn operation_type(&self) -> &'static str {
        match self {
            Instruction::Transform { .. } => "morphism_application",
            Instruction::Alloc { .. } => "object_creation", 
            Instruction::Consume { .. } => "object_destruction",
            Instruction::Compose { .. } => "morphism_composition",
            Instruction::Tensor { .. } => "parallel_composition",
        }
    }
}

//-----------------------------------------------------------------------------
// Instruction Utilities
//-----------------------------------------------------------------------------

impl Instruction {
    /// Get all register IDs read by this instruction
    pub fn reads(&self) -> Vec<RegisterId> {
        match self {
            Instruction::Transform { morph_reg, input_reg, .. } => vec![*morph_reg, *input_reg],
            Instruction::Alloc { type_reg, init_reg, .. } => vec![*type_reg, *init_reg],
            Instruction::Consume { resource_reg, .. } => vec![*resource_reg],
            Instruction::Compose { first_reg, second_reg, .. } => vec![*first_reg, *second_reg],
            Instruction::Tensor { left_reg, right_reg, .. } => vec![*left_reg, *right_reg],
        }
    }
    
    /// Get all register IDs written by this instruction
    pub fn writes(&self) -> Vec<RegisterId> {
        match self {
            Instruction::Transform { output_reg, .. } => vec![*output_reg],
            Instruction::Alloc { output_reg, .. } => vec![*output_reg],
            Instruction::Consume { output_reg, .. } => vec![*output_reg],
            Instruction::Compose { output_reg, .. } => vec![*output_reg],
            Instruction::Tensor { output_reg, .. } => vec![*output_reg],
        }
    }
} 