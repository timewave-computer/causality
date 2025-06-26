//! Machine-level pattern matching for Layer 0 execution
//!
//! This module provides simplified pattern matching for the register machine,
//! compiled down from the higher-level Layer 1 patterns.

use serde::{Serialize, Deserialize};
use crate::lambda::Symbol;

//-----------------------------------------------------------------------------
// Machine Pattern Types
//-----------------------------------------------------------------------------

/// Simplified pattern for machine-level pattern matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard pattern - matches anything
    Wildcard,
    
    /// Literal value pattern - matches exact values
    Literal(LiteralValue),
    
    /// Constructor pattern with name and sub-patterns
    Constructor {
        /// Constructor name/tag
        name: String,
        /// Sub-patterns for constructor arguments
        patterns: Vec<Pattern>,
    },
    
    /// Product pattern for tuple matching
    Product(Vec<Pattern>),
}

/// Machine-level literal values for pattern matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiteralValue {
    /// Unit value
    Unit,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(u32),
    /// Symbol value
    Symbol(Symbol),
}

//-----------------------------------------------------------------------------
// Pattern Matching Implementation
//-----------------------------------------------------------------------------

impl Pattern {
    /// Create a wildcard pattern
    pub fn wildcard() -> Self {
        Pattern::Wildcard
    }
    
    /// Create a literal pattern
    pub fn literal(value: LiteralValue) -> Self {
        Pattern::Literal(value)
    }
    
    /// Create a constructor pattern
    pub fn constructor(name: String, patterns: Vec<Pattern>) -> Self {
        Pattern::Constructor { name, patterns }
    }
    
    /// Create a product pattern
    pub fn product(patterns: Vec<Pattern>) -> Self {
        Pattern::Product(patterns)
    }
    
    /// Check if this pattern matches a given value
    pub fn matches(&self, value: &crate::machine::MachineValue) -> bool {
        use crate::machine::MachineValue;
        
        match (self, value) {
            // Wildcard matches everything
            (Pattern::Wildcard, _) => true,
            
            // Literal patterns
            (Pattern::Literal(LiteralValue::Unit), MachineValue::Unit) => true,
            (Pattern::Literal(LiteralValue::Bool(p)), MachineValue::Bool(v)) => p == v,
            (Pattern::Literal(LiteralValue::Int(p)), MachineValue::Int(v)) => p == v,
            (Pattern::Literal(LiteralValue::Symbol(p)), MachineValue::Symbol(v)) => p == v,
            
            // Constructor patterns map to Sum values
            (Pattern::Constructor { name, patterns }, MachineValue::Sum { tag, value }) => {
                name == &tag.to_string() && patterns.len() == 1 &&
                patterns[0].matches(value)
            }
            
            // Product patterns map to Product or Tensor values
            (Pattern::Product(patterns), MachineValue::Product(left, right)) => {
                patterns.len() == 2 &&
                patterns[0].matches(left) && patterns[1].matches(right)
            }
            (Pattern::Product(patterns), MachineValue::Tensor(left, right)) => {
                patterns.len() == 2 &&
                patterns[0].matches(left) && patterns[1].matches(right)
            }
            
            // No match
            _ => false,
        }
    }
    
    /// Check if this pattern is exhaustive (always matches)
    pub fn is_exhaustive(&self) -> bool {
        match self {
            Pattern::Wildcard => true,
            Pattern::Literal(_) => false, // Literals are not exhaustive
            Pattern::Constructor { patterns, .. } => {
                // Conservative: only exhaustive if all sub-patterns are exhaustive
                patterns.iter().all(|p| p.is_exhaustive())
            }
            Pattern::Product(patterns) => {
                // Product is exhaustive if all components are exhaustive
                patterns.iter().all(|p| p.is_exhaustive())
            }
        }
    }
    
    /// Get the complexity of this pattern (for optimization)
    pub fn complexity(&self) -> usize {
        match self {
            Pattern::Wildcard => 1,
            Pattern::Literal(_) => 1,
            Pattern::Constructor { patterns, .. } => {
                1 + patterns.iter().map(|p| p.complexity()).sum::<usize>()
            }
            Pattern::Product(patterns) => {
                1 + patterns.iter().map(|p| p.complexity()).sum::<usize>()
            }
        }
    }
}

//-----------------------------------------------------------------------------
// Literal Value Implementations
//-----------------------------------------------------------------------------

impl LiteralValue {
    /// Create a unit literal
    pub fn unit() -> Self {
        LiteralValue::Unit
    }
    
    /// Create a boolean literal
    pub fn bool(value: bool) -> Self {
        LiteralValue::Bool(value)
    }
    
    /// Create an integer literal
    pub fn int(value: u32) -> Self {
        LiteralValue::Int(value)
    }
    
    /// Create a symbol literal
    pub fn symbol(value: Symbol) -> Self {
        LiteralValue::Symbol(value)
    }
}

//-----------------------------------------------------------------------------
// Display and Debug Implementations
//-----------------------------------------------------------------------------

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Literal(lit) => write!(f, "{}", lit),
            Pattern::Constructor { name, patterns } => {
                write!(f, "{}(", name)?;
                for (i, pattern) in patterns.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", pattern)?;
                }
                write!(f, ")")
            }
            Pattern::Product(patterns) => {
                write!(f, "(")?;
                for (i, pattern) in patterns.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", pattern)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl std::fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Unit => write!(f, "()"),
            LiteralValue::Bool(b) => write!(f, "{}", b),
            LiteralValue::Int(i) => write!(f, "{}", i),
            LiteralValue::Symbol(s) => write!(f, ":{}", s),
        }
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::MachineValue;
    
    #[test]
    fn test_wildcard_pattern() {
        let pattern = Pattern::wildcard();
        let value = MachineValue::Int(42);
        
        assert!(pattern.matches(&value));
        assert!(pattern.is_exhaustive());
        assert_eq!(pattern.complexity(), 1);
    }
    
    #[test]
    fn test_literal_pattern() {
        let pattern = Pattern::literal(LiteralValue::int(42));
        let matching_value = MachineValue::Int(42);
        let non_matching_value = MachineValue::Int(24);
        
        assert!(pattern.matches(&matching_value));
        assert!(!pattern.matches(&non_matching_value));
        assert!(!pattern.is_exhaustive());
    }
    
    #[test]
    fn test_constructor_pattern() {
        let pattern = Pattern::constructor("Some".to_string(), vec![
            Pattern::literal(LiteralValue::int(42))
        ]);
        
        let matching_value = MachineValue::Sum {
            tag: crate::lambda::Symbol::from("Some"),
            value: Box::new(MachineValue::Int(42)),
        };
        
        let non_matching_value = MachineValue::Sum {
            tag: crate::lambda::Symbol::from("None"),
            value: Box::new(MachineValue::Unit),
        };
        
        assert!(pattern.matches(&matching_value));
        assert!(!pattern.matches(&non_matching_value));
    }
    
    #[test]
    fn test_product_pattern() {
        let pattern = Pattern::product(vec![
            Pattern::literal(LiteralValue::int(1)),
            Pattern::literal(LiteralValue::bool(true)),
        ]);
        
        let matching_value = MachineValue::Product(
            Box::new(MachineValue::Int(1)),
            Box::new(MachineValue::Bool(true)),
        );
        
        let non_matching_value = MachineValue::Product(
            Box::new(MachineValue::Int(2)),
            Box::new(MachineValue::Bool(true)),
        );
        
        assert!(pattern.matches(&matching_value));
        assert!(!pattern.matches(&non_matching_value));
    }
    
    #[test]
    fn test_pattern_complexity() {
        let simple = Pattern::wildcard();
        let literal = Pattern::literal(LiteralValue::int(42));
        let constructor = Pattern::constructor("Pair".to_string(), vec![
            Pattern::wildcard(),
            Pattern::literal(LiteralValue::bool(true)),
        ]);
        
        assert_eq!(simple.complexity(), 1);
        assert_eq!(literal.complexity(), 1);
        assert_eq!(constructor.complexity(), 3); // 1 + 1 + 1
    }
} 