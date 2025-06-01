//! Register machine instructions
//!
//! This module defines the core instruction set for the register machine,
//! implementing the operational semantics of the language.
//! 
//! The machine operates with 9 core instructions:
//! - move r₁ r₂: Move value between registers
//! - apply r_fn r_arg r_out: Function application
//! - match r_sum r_left r_right label_l label_r: Pattern matching on sums
//! - alloc r_type r_val r_out: Allocate resource
//! - consume r_resource r_out: Consume resource
//! - check constraint: Verify constraint
//! - perform effect r_out: Execute effect
//! - select r_cond r_true r_false r_out: Conditional value selection
//! - witness r_out: Read from untrusted witness

use crate::lambda::{TypeInner, Symbol};

//-----------------------------------------------------------------------------
// Register Identifiers
//-----------------------------------------------------------------------------

/// Unique identifier for a register in the machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Label for control flow jumps
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(pub String);

impl Label {
    /// Create a new label
    pub fn new(name: impl Into<String>) -> Self {
        Label(name.into())
    }
}

//-----------------------------------------------------------------------------
// Core Instruction Set (9 Instructions)
//-----------------------------------------------------------------------------

/// Core instruction set for the register machine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// Move value from one register to another (consuming source)
    /// move r₁ r₂
    Move {
        src: RegisterId,
        dst: RegisterId,
    },
    
    /// Apply a function to an argument, storing result in destination
    /// apply r_fn r_arg r_out
    Apply {
        fn_reg: RegisterId,
        arg_reg: RegisterId,
        out_reg: RegisterId,
    },
    
    /// Pattern match on a sum type value
    /// match r_sum r_left r_right label_l label_r
    Match {
        sum_reg: RegisterId,
        left_reg: RegisterId,
        right_reg: RegisterId,
        left_label: Label,
        right_label: Label,
    },
    
    /// Allocate a new resource on the heap
    /// alloc r_type r_val r_out
    Alloc {
        type_reg: RegisterId,
        val_reg: RegisterId,
        out_reg: RegisterId,
    },
    
    /// Consume a resource (marking it as used in heap)
    /// consume r_resource r_out
    Consume {
        resource_reg: RegisterId,
        out_reg: RegisterId,
    },
    
    /// Check a constraint/precondition
    /// check constraint
    Check {
        constraint: ConstraintExpr,
    },
    
    /// Perform an effect
    /// perform effect r_out
    Perform {
        effect: Effect,
        out_reg: RegisterId,
    },
    
    /// Conditional value selection
    /// select r_cond r_true r_false r_out
    Select {
        cond_reg: RegisterId,
        true_reg: RegisterId,
        false_reg: RegisterId,
        out_reg: RegisterId,
    },
    
    /// Read from untrusted witness
    /// witness r_out
    Witness {
        out_reg: RegisterId,
    },
    
    /// Defines a label at the current position in the program sequence.
    /// This instruction does nothing during execution but is used by the
    /// ReductionEngine to build its label map for jumps.
    LabelMarker(Label),
}

//-----------------------------------------------------------------------------
// Constraint Expressions
//-----------------------------------------------------------------------------

/// Constraint expressions for runtime checks
#[derive(Debug, Clone, PartialEq, Eq)]
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
    
    /// Type check
    HasType(RegisterId, TypeInner),
    
    /// Linear resource consumption check
    IsConsumed(RegisterId),
    
    /// Capability check
    HasCapability(RegisterId, Symbol),
    
    /// Ownership check
    IsOwner(RegisterId, RegisterId), // resource, owner
    
    /// Custom predicate
    Predicate {
        name: Symbol,
        args: Vec<RegisterId>,
    },
}

//-----------------------------------------------------------------------------
// Effects
//-----------------------------------------------------------------------------

/// Effect to be performed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    /// Name/tag of the effect
    pub tag: Symbol,
    
    /// Parameters for the effect
    pub params: Vec<RegisterId>,
    
    /// Pre-condition constraint
    pub pre: ConstraintExpr,
    
    /// Post-condition constraint
    pub post: ConstraintExpr,
    
    /// Optimization hints
    pub hints: Vec<Hint>,
}

/// Optimization hints for effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hint {
    /// Batch with effects matching selector
    BatchWith(Selector),
    
    /// Minimize metric
    Minimize(Metric),
    
    /// Maximize metric
    Maximize(Metric),
    
    /// Prefer specific domain
    PreferDomain(String),
    
    /// Require specific domain
    RequireDomain(String),
    
    /// Deadline for completion
    Deadline(u64), // timestamp
    
    /// Conjunction of hints
    HintAll(Vec<Hint>),
    
    /// Disjunction of hints
    HintAny(Vec<Hint>),
}

/// Selector for batching effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// Effects with same type tag
    SameType,
    
    /// Effects with same target
    SameTarget,
    
    /// Custom selection predicate
    Custom(String),
}

/// Metric to optimize
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Metric {
    Price,
    Latency,
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
    /// Name/tag of the effect
    pub tag: Symbol,
    
    /// Arguments to the effect
    pub args: Vec<RegisterId>,
    
    /// Expected return type
    pub return_type: Option<TypeInner>,
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
            Instruction::Perform { effect, .. } => effect.params.clone(),
            Instruction::Select { cond_reg, true_reg, false_reg, .. } => {
                vec![*cond_reg, *true_reg, *false_reg]
            }
            Instruction::Witness { .. } => vec![],
            Instruction::LabelMarker(_) => vec![],
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