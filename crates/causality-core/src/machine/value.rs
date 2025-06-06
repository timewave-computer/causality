//! Machine values and register values
//!
//! This module defines the value types that can be stored in registers
//! and manipulated by the register machine.

use super::instruction::{RegisterId, LiteralValue, Pattern, Label};
use crate::system::content_addressing::ResourceId;
use crate::lambda::{TypeInner, Symbol, BaseType};

/// Values that can be stored in registers
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterValue {
    /// The actual value
    pub value: MachineValue,
    
    /// Type information
    pub value_type: Option<TypeInner>,
    
    /// Whether this register has been consumed (for linearity)
    pub consumed: bool,
}

/// Runtime values in the register machine
#[derive(Debug, Clone, PartialEq)]
pub enum MachineValue {
    /// Unit value
    Unit,
    
    /// Boolean value
    Bool(bool),
    
    /// Integer value
    Int(u32),
    
    /// Symbol value
    Symbol(Symbol),
    
    /// Product value (pair)
    Product(Box<MachineValue>, Box<MachineValue>),
    
    /// Sum value (tagged union)
    Sum {
        tag: Symbol,
        value: Box<MachineValue>,
    },
    
    /// Function value (closure stored in register)
    Function {
        /// Parameter registers to bind
        params: Vec<RegisterId>,
        /// Label pointing to the function's code in the main program
        body_label: Label,
        /// Holds a reference to the captured lexical environment for closures
        /// None if the function is not a closure or captures no variables
        capture_env_reg: Option<RegisterId>,
    },
    
    /// Resource reference (points to resource heap)
    ResourceRef(ResourceId),
    
    /// Builtin function
    BuiltinFunction(Symbol),
    
    /// Type value (for alloc instruction)
    Type(TypeInner),
    
    /// Effect result placeholder
    EffectResult(String),
    
    /// Partially applied function (for currying)
    PartiallyApplied {
        name: Symbol,
        args: Vec<MachineValue>,
    },
}

impl MachineValue {
    /// Convert a literal value to a machine value
    pub fn from_literal(literal: LiteralValue) -> Self {
        match literal {
            LiteralValue::Unit => MachineValue::Unit,
            LiteralValue::Bool(b) => MachineValue::Bool(b),
            LiteralValue::Int(i) => MachineValue::Int(i),
            LiteralValue::Symbol(s) => MachineValue::Symbol(s),
        }
    }
    
    /// Get the type of this value
    pub fn get_type(&self) -> TypeInner {
        match self {
            MachineValue::Unit => TypeInner::Base(BaseType::Unit),
            MachineValue::Bool(_) => TypeInner::Base(BaseType::Bool),
            MachineValue::Int(_) => TypeInner::Base(BaseType::Int),
            MachineValue::Symbol(_) => TypeInner::Base(BaseType::Symbol),
            
            MachineValue::Product(l, r) => {
                TypeInner::Product(
                    Box::new(l.get_type()),
                    Box::new(r.get_type())
                )
            }
            
            MachineValue::Sum { .. } => {
                // For sum types, we'd need more context
                // This is a placeholder - in a full implementation
                // we'd track the complete sum type
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::Function { .. } => {
                // Function types would need parameter and return type info
                // This is a placeholder - in a full implementation
                // we'd track the complete function type
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::ResourceRef(_) => {
                // Resource references would need type lookup
                // This is a placeholder - in a full implementation
                // we'd track the resource type
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::BuiltinFunction(_) => {
                // Builtin functions would need type lookup
                // This is a placeholder - in a full implementation
                // we'd track the function type
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::Type(_) => {
                // Type values would need type lookup
                // This is a placeholder - in a full implementation
                // we'd track the type
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::EffectResult(_) => {
                // Effect results would need type lookup
                // This is a placeholder - in a full implementation
                // we'd track the effect type
                TypeInner::Base(BaseType::Symbol)
            }
            
            MachineValue::PartiallyApplied { .. } => {
                // Partially applied functions would need type lookup
                // This is a placeholder - in a full implementation
                // we'd track the function type
                TypeInner::Base(BaseType::Symbol)
            }
        }
    }
    
    /// Check if this value matches a pattern
    pub fn matches_pattern(&self, pattern: &Pattern) -> bool {
        match (self, pattern) {
            (_, Pattern::Wildcard) => true,
            (_, Pattern::Var(_)) => true, // Variables always match
            
            (MachineValue::Sum { tag, value }, Pattern::Constructor { tag: pat_tag, args }) => {
                tag == pat_tag && args.len() == 1 && value.matches_pattern(&args[0])
            }
            
            (value, Pattern::Literal(lit)) => {
                &MachineValue::from_literal(lit.clone()) == value
            }
            
            (MachineValue::Product(l, r), Pattern::Product(pat_l, pat_r)) => {
                l.matches_pattern(pat_l) && r.matches_pattern(pat_r)
            }
            
            _ => false,
        }
    }
} 