//! Register machine instruction set for Layer 0 execution
//!
//! This module defines the core instruction set for the register machine,
//! implementing the operational semantics of the language.
//! 
//! The machine operates with 11 core instructions:
//! - move r₁ r₂: Move value between registers
//! - apply r_fn r_arg r_out: Function application
//! - match r_sum r_left r_right label_l label_r: Pattern matching on sums
//! - alloc r_type r_val r_out: Allocate resource
//! - consume r_resource r_out: Consume resource
//! - check constraint: Verify constraint
//! - perform effect r_out: Execute effect
//! - select r_cond r_true r_false r_out: Conditional value selection
//! - witness r_out: Read from untrusted witness

use crate::lambda::Symbol;
use serde::{Serialize, Deserialize};

//-----------------------------------------------------------------------------
// Register Identifiers
//-----------------------------------------------------------------------------

/// Register identifier
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Label(pub String);

impl Label {
    /// Create a new label
    pub fn new(name: impl Into<String>) -> Self {
        Label(name.into())
    }
}

//-----------------------------------------------------------------------------
// Core Instruction Set (11 Instructions)
//-----------------------------------------------------------------------------

/// Register machine instruction set
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    /// Move value between registers: dst := src
    Move { src: RegisterId, dst: RegisterId },
    
    /// Function application: out := fn(arg)
    Apply { fn_reg: RegisterId, arg_reg: RegisterId, out_reg: RegisterId },
    
    /// Pattern matching: match sum with left => l | right => r
    Match { 
        sum_reg: RegisterId, 
        left_reg: RegisterId, 
        right_reg: RegisterId,
        left_label: String,
        right_label: String,
    },
    
    /// Allocate resource: out := alloc(type, value)
    Alloc { type_reg: RegisterId, val_reg: RegisterId, out_reg: RegisterId },
    
    /// Consume resource: out := consume(resource)
    Consume { resource_reg: RegisterId, out_reg: RegisterId },
    
    /// Check constraint: assert(constraint)
    Check { constraint: ConstraintExpr },
    
    /// Perform effect: out := perform(effect)
    Perform { effect: Effect, out_reg: RegisterId },
    
    /// Conditional selection: out := cond ? true_val : false_val
    Select { cond_reg: RegisterId, true_reg: RegisterId, false_reg: RegisterId, out_reg: RegisterId },
    
    /// Witness value: out := witness()
    Witness { out_reg: RegisterId },
    
    /// Label marker for control flow
    LabelMarker(String),
    
    /// Return from function call
    Return { 
        /// Register containing the result value (if Some)
        result_reg: Option<RegisterId> 
    },
}

//-----------------------------------------------------------------------------
// Constraint Expressions
//-----------------------------------------------------------------------------

/// Constraint expressions for runtime checks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintExpr {
    /// Always true
    True,
    
    /// Always false
    False,
    
    /// Logical AND
    And(Box<ConstraintExpr>, Box<ConstraintExpr>),
    
    /// Logical OR
    Or(Box<ConstraintExpr>, Box<ConstraintExpr>),
    
    /// Logical NOT
    Not(Box<ConstraintExpr>),
    
    /// Register equality check
    Equal(RegisterId, RegisterId),
    
    /// Numeric comparisons
    LessThan(RegisterId, RegisterId),
    GreaterThan(RegisterId, RegisterId),
    LessEqual(RegisterId, RegisterId),
    GreaterEqual(RegisterId, RegisterId),
    
    /// Type check (simplified with string type name)
    HasType(RegisterId, String),
    
    /// Linear resource consumption check
    IsConsumed(RegisterId),
    
    /// Capability check (simplified with string capability name)
    HasCapability(RegisterId, String),
    
    /// Ownership check
    IsOwner(RegisterId, RegisterId), // resource, owner
    
    /// Custom predicate (simplified with string name)
    Predicate {
        name: String,
        args: Vec<RegisterId>,
    },
}

//-----------------------------------------------------------------------------
// Effects
//-----------------------------------------------------------------------------

/// Effect to be performed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    /// Name/tag of the effect (simplified as string)
    pub tag: String,
    
    /// Pre-condition constraint
    pub pre: ConstraintExpr,
    
    /// Post-condition constraint
    pub post: ConstraintExpr,
    
    /// Optimization hints
    pub hints: Vec<Hint>,
}

/// Optimization hints for effects
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hint {
    /// Prefer parallel execution
    Parallel,
    
    /// Prefer sequential execution
    Sequential,
    
    /// Target execution domain (simplified as string)
    Domain(String),
    
    /// Optimization priority
    Priority(u32),
    
    /// Resource usage estimate
    ResourceUsage(u32),
    
    /// Apply all hints in the list (from MachineHint)
    HintAll(Vec<MachineHint>),
    
    /// Batch operations with given selector (from MachineHint)
    BatchWith(Selector),
    
    /// Minimize the given metric (from MachineHint)
    Minimize(Metric),
    
    /// Prefer execution in specific domain (from MachineHint)
    PreferDomain(String),
    
    /// Deadline for completion (milliseconds)
    Deadline(u64),
    
    /// Machine-level hint integration
    Custom(MachineHint),
}

//-----------------------------------------------------------------------------
// Pattern Matching
//-----------------------------------------------------------------------------

/// A pattern matching arm with pattern and instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    /// Pattern to match against
    pub pattern: Pattern,
    
    /// Instructions to execute if pattern matches
    pub instructions: Vec<Instruction>,
}

/// Patterns for matching values
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// Wildcard pattern - matches anything
    Wildcard,
    
    /// Variable binding pattern
    Var(RegisterId),
    
    /// Constructor pattern for sum types
    Constructor {
        tag: Symbol,
        args: Vec<Pattern>,
    },
    
    /// Literal value pattern
    Literal(LiteralValue),
    
    /// Product pattern (destructuring)
    Product(Box<Pattern>, Box<Pattern>),
}

//-----------------------------------------------------------------------------
// Literal Values
//-----------------------------------------------------------------------------

/// Literal values that can appear in patterns and instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralValue {
    /// Unit value
    Unit,
    
    /// Boolean value
    Bool(bool),
    
    /// Integer value
    Int(u32),
    
    /// Symbol value (ZK-compatible)
    Symbol(Symbol),
}

//-----------------------------------------------------------------------------
// Effect Calls
//-----------------------------------------------------------------------------

/// Represents a call to an effect
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectCall {
    /// Name/tag of the effect (simplified as string)
    pub tag: String,
    
    /// Arguments to the effect
    pub args: Vec<RegisterId>,
    
    /// Expected return type (simplified as string description)
    pub return_type: Option<String>,
}

//-----------------------------------------------------------------------------
// Instruction Utilities
//-----------------------------------------------------------------------------

impl Instruction {
    /// Get all register IDs read by this instruction
    pub fn reads(&self) -> Vec<RegisterId> {
        match self {
            Instruction::Move { src, .. } => vec![*src],
            Instruction::Apply { fn_reg, arg_reg, .. } => vec![*fn_reg, *arg_reg],
            Instruction::Match { sum_reg, .. } => vec![*sum_reg],
            Instruction::Alloc { type_reg, val_reg, .. } => vec![*type_reg, *val_reg],
            Instruction::Consume { resource_reg, .. } => vec![*resource_reg],
            Instruction::Check { constraint } => constraint.reads(),
            Instruction::Perform { effect, .. } => effect.pre.reads(),
            Instruction::Select { cond_reg, true_reg, false_reg, .. } => {
                vec![*cond_reg, *true_reg, *false_reg]
            }
            Instruction::Witness { .. } => vec![],
            Instruction::LabelMarker(_) => vec![],
            Instruction::Return { result_reg } => {
                result_reg.iter().cloned().collect()
            }
        }
    }
    
    /// Get all register IDs written by this instruction
    pub fn writes(&self) -> Vec<RegisterId> {
        match self {
            Instruction::Move { dst, .. } => vec![*dst],
            Instruction::Apply { out_reg, .. } => vec![*out_reg],
            Instruction::Match { left_reg, right_reg, .. } => vec![*left_reg, *right_reg],
            Instruction::Alloc { out_reg, .. } => vec![*out_reg],
            Instruction::Consume { out_reg, .. } => vec![*out_reg],
            Instruction::Check { .. } => vec![],
            Instruction::Perform { out_reg, .. } => vec![*out_reg],
            Instruction::Select { out_reg, .. } => vec![*out_reg],
            Instruction::Witness { out_reg } => vec![*out_reg],
            Instruction::LabelMarker(_) => vec![],
            Instruction::Return { result_reg } => {
                result_reg.iter().cloned().collect()
            }
        }
    }
}

impl ConstraintExpr {
    /// Get all register IDs read by this constraint
    pub fn reads(&self) -> Vec<RegisterId> {
        match self {
            ConstraintExpr::True | ConstraintExpr::False => vec![],
            ConstraintExpr::And(l, r) | ConstraintExpr::Or(l, r) => {
                let mut reads = l.reads();
                reads.extend(r.reads());
                reads
            }
            ConstraintExpr::Not(expr) => expr.reads(),
            ConstraintExpr::Equal(a, b) 
            | ConstraintExpr::LessThan(a, b)
            | ConstraintExpr::GreaterThan(a, b)
            | ConstraintExpr::LessEqual(a, b)
            | ConstraintExpr::GreaterEqual(a, b)
            | ConstraintExpr::IsOwner(a, b) => vec![*a, *b],
            ConstraintExpr::HasType(reg, _) 
            | ConstraintExpr::IsConsumed(reg)
            | ConstraintExpr::HasCapability(reg, _) => vec![*reg],
            ConstraintExpr::Predicate { args, .. } => args.clone(),
        }
    }
}

//-----------------------------------------------------------------------------
// Metrics and Selectors for Optimization
//-----------------------------------------------------------------------------

/// Metrics for optimization hints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Metric {
    /// Optimize for low latency
    Latency,
    
    /// Optimize for low cost
    Cost,
    
    /// Optimize for high throughput
    Throughput,
    
    /// Optimize for low memory usage
    Memory,
    
    /// Custom metric
    Custom(String),
}

/// Selectors for batching strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selector {
    /// Batch operations of the same type
    SameType,
    
    /// Batch operations on same domain
    SameDomain,
    
    /// Batch operations with similar cost
    SimilarCost,
    
    /// Custom selector
    Custom(String),
}

/// Machine-level optimization hints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineHint {
    /// Apply all hints in the list
    HintAll(Vec<MachineHint>),
    
    /// Batch operations with given selector
    BatchWith(Selector),
    
    /// Minimize the given metric
    Minimize(Metric),
    
    /// Prefer execution in specific domain (simplified as string)
    PreferDomain(String),
    
    /// Deadline for completion (milliseconds)
    Deadline(u64),
    
    /// Custom hint
    Custom(String),
} 