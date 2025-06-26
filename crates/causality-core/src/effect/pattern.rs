//! Pattern matching for Layer 1 terms and types
//!
//! This module provides pattern matching capabilities for the lambda calculus
//! with linear types, supporting destructuring of data types and exhaustiveness checking.

use crate::lambda::{TypeInner, Literal};
use crate::lambda::base::BaseType;
use crate::Span;

//-----------------------------------------------------------------------------
// Pattern Definitions
//-----------------------------------------------------------------------------

/// Pattern for matching and destructuring values
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    /// The pattern kind
    pub kind: PatternKind,
    
    /// Type annotation on the pattern
    pub type_annotation: Option<TypeInner>,
    
    /// Source location
    pub span: Option<Span>,
}

/// Different kinds of patterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternKind {
    /// Wildcard pattern - matches anything, binds nothing
    Wildcard,
    
    /// Variable pattern - matches anything, binds to variable
    Var(String),
    
    /// Literal pattern - matches exact literal values
    Literal(Literal),
    
    /// Constructor pattern for sum types
    Constructor {
        /// Constructor tag/name
        tag: String,
        
        /// Patterns for constructor arguments
        args: Vec<Pattern>,
    },
    
    /// Product pattern (tuple destructuring)
    Product {
        /// Pattern for first element
        left: Box<Pattern>,
        
        /// Pattern for second element
        right: Box<Pattern>,
    },
    
    /// Record pattern for destructuring records
    Record {
        /// Field patterns
        fields: Vec<FieldPattern>,
        
        /// Whether this is an open pattern (allows extra fields)
        open: bool,
    },
    
    /// As pattern (pattern alias) - matches pattern and binds to variable
    As {
        /// Inner pattern to match
        pattern: Box<Pattern>,
        
        /// Variable to bind the matched value to
        var: String,
    },
    
    /// Or pattern - matches if any sub-pattern matches
    Or {
        /// Alternative patterns
        patterns: Vec<Pattern>,
    },
}

/// Field pattern for record destructuring
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldPattern {
    /// Field name
    pub field: String,
    
    /// Pattern for the field value
    pub pattern: Pattern,
}

//-----------------------------------------------------------------------------
// Pattern Construction Helpers
//-----------------------------------------------------------------------------

impl Pattern {
    /// Create a new pattern with a kind
    pub fn new(kind: PatternKind) -> Self {
        Self {
            kind,
            type_annotation: None,
            span: None,
        }
    }
    
    /// Create pattern with type annotation
    pub fn with_type(mut self, type_annotation: TypeInner) -> Self {
        self.type_annotation = Some(type_annotation);
        self
    }
    
    /// Create pattern with span
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
    
    /// Create a wildcard pattern
    pub fn wildcard() -> Self {
        Self::new(PatternKind::Wildcard)
    }
    
    /// Create a variable pattern
    pub fn var(name: String) -> Self {
        Self::new(PatternKind::Var(name))
    }
    
    /// Create a literal pattern
    pub fn literal(lit: Literal) -> Self {
        Self::new(PatternKind::Literal(lit))
    }
    
    /// Create a constructor pattern
    pub fn constructor(tag: String, args: Vec<Pattern>) -> Self {
        Self::new(PatternKind::Constructor { tag, args })
    }
    
    /// Create a product pattern
    pub fn product(left: Pattern, right: Pattern) -> Self {
        Self::new(PatternKind::Product {
            left: Box::new(left),
            right: Box::new(right),
        })
    }
    
    /// Create a record pattern
    pub fn record(fields: Vec<FieldPattern>, open: bool) -> Self {
        Self::new(PatternKind::Record { fields, open })
    }
    
    /// Create an as pattern
    pub fn as_pattern(pattern: Pattern, var: String) -> Self {
        Self::new(PatternKind::As {
            pattern: Box::new(pattern),
            var,
        })
    }
    
    /// Create an or pattern
    pub fn or(patterns: Vec<Pattern>) -> Self {
        Self::new(PatternKind::Or { patterns })
    }
}

//-----------------------------------------------------------------------------
// Pattern Analysis
//-----------------------------------------------------------------------------

impl Pattern {
    /// Get all variables bound by this pattern
    pub fn bound_vars(&self) -> Vec<String> {
        match &self.kind {
            PatternKind::Wildcard => vec![],
            PatternKind::Var(name) => vec![name.clone()],
            PatternKind::Literal(_) => vec![],
            
            PatternKind::Constructor { args, .. } => {
                args.iter().flat_map(|p| p.bound_vars()).collect()
            }
            
            PatternKind::Product { left, right } => {
                let mut vars = left.bound_vars();
                vars.extend(right.bound_vars());
                vars
            }
            
            PatternKind::Record { fields, .. } => {
                fields.iter().flat_map(|f| f.pattern.bound_vars()).collect()
            }
            
            PatternKind::As { pattern, var } => {
                let mut vars = pattern.bound_vars();
                vars.push(var.clone());
                vars
            }
            
            PatternKind::Or { patterns } => {
                // For or patterns, all alternatives must bind the same variables
                patterns.first()
                    .map(|p| p.bound_vars())
                    .unwrap_or_default()
            }
        }
    }
    
    /// Check if this pattern is refutable (can fail to match)
    pub fn is_refutable(&self) -> bool {
        match &self.kind {
            PatternKind::Wildcard => false,
            PatternKind::Var(_) => false,
            PatternKind::Literal(_) => true, // Literals can fail to match
            
            PatternKind::Constructor {  .. } => {
                // Constructor patterns are refutable unless they're the only constructor
                // We assume they're refutable for safety
                true
            }
            
            PatternKind::Product { left, right } => {
                left.is_refutable() || right.is_refutable()
            }
            
            PatternKind::Record { fields, open } => {
                if *open {
                    // Open records are less refutable
                    fields.iter().any(|f| f.pattern.is_refutable())
                } else {
                    // Closed records must match exactly
                    true
                }
            }
            
            PatternKind::As { pattern, .. } => pattern.is_refutable(),
            
            PatternKind::Or { patterns } => {
                // Or patterns are refutable if all alternatives are refutable
                patterns.iter().all(|p| p.is_refutable())
            }
        }
    }
    
    /// Check if this pattern is exhaustive for a given type
    pub fn is_exhaustive_for(&self, type_inner: &TypeInner) -> bool {
        match (type_inner, &self.kind) {
            // Unit type is exhaustive with wildcard or unit constructor
            (TypeInner::Base(BaseType::Unit), PatternKind::Wildcard) => true,
            (TypeInner::Base(BaseType::Unit), PatternKind::Constructor { tag, .. }) => tag == "unit",
            
            // Bool type requires both true and false cases, or wildcard
            (TypeInner::Base(BaseType::Bool), PatternKind::Wildcard) => true,
            (TypeInner::Base(BaseType::Bool), PatternKind::Or { patterns }) => {
                let has_true = patterns.iter().any(|p| matches!(p.kind, PatternKind::Literal(Literal::Bool(true))));
                let has_false = patterns.iter().any(|p| matches!(p.kind, PatternKind::Literal(Literal::Bool(false))));
                has_true && has_false
            }
            
            // Int and Symbol types are infinite, so only wildcard is exhaustive
            (TypeInner::Base(BaseType::Int), PatternKind::Wildcard) => true,
            (TypeInner::Base(BaseType::Symbol), PatternKind::Wildcard) => true,
            
            // Product types require exhaustive patterns for both components
            (TypeInner::Product(left_type, right_type), PatternKind::Product { left, right }) => {
                left.is_exhaustive_for(left_type) && right.is_exhaustive_for(right_type)
            }
            (TypeInner::Product(_, _), PatternKind::Wildcard) => true,
            
            // Linear function types - only wildcard is exhaustive (functions are opaque)
            (TypeInner::LinearFunction(_, _), PatternKind::Wildcard) => true,
            
            // Session types - only wildcard is exhaustive (sessions are protocol-dependent)
            (TypeInner::Session(_), PatternKind::Wildcard) => true,
            
            // Variable patterns are exhaustive for any type
            (_, PatternKind::Var(_)) => true,
            
            // Wildcard patterns are always exhaustive
            (_, PatternKind::Wildcard) => true,
            
            // As patterns delegate to their inner pattern
            (t, PatternKind::As { pattern, .. }) => pattern.is_exhaustive_for(t),
            
            // Or patterns are exhaustive if they cover all cases
            (t, PatternKind::Or { patterns }) => {
                // For simplicity, we check if any single pattern is exhaustive
                // A more sophisticated analysis would check if the union covers all cases
                patterns.iter().any(|p| p.is_exhaustive_for(t))
            }
            
            // Constructor patterns require knowledge of all constructors for the type
            // For now, we conservatively return false unless it's a known exhaustive case
            (_, PatternKind::Constructor { .. }) => false,
            
            // Record patterns require matching all required fields
            (_, PatternKind::Record { open: true, .. }) => true, // Open records are exhaustive
            (_, PatternKind::Record { open: false, .. }) => false, // Closed records need field analysis
            
            // Literal patterns are only exhaustive for singleton types
            (_, PatternKind::Literal(_)) => false,
            
            // Catch-all for any remaining patterns
            _ => false,
        }
    }
}

//-----------------------------------------------------------------------------
// Conversion to Machine Patterns
//-----------------------------------------------------------------------------

// Machine pattern conversion
impl From<Pattern> for crate::machine::Pattern {
    fn from(pattern: Pattern) -> Self {
        match pattern.kind {
            PatternKind::Wildcard => crate::machine::Pattern::Wildcard,
            PatternKind::Var(_) => crate::machine::Pattern::Wildcard, // Variables become wildcards in machine layer
            PatternKind::Literal(lit) => crate::machine::Pattern::Literal(lit.into()),
            PatternKind::Constructor { tag, args } => {
                crate::machine::Pattern::Constructor {
                    name: tag,
                    patterns: args.into_iter().map(|p| p.into()).collect(),
                }
            }
            PatternKind::Product { left, right } => {
                crate::machine::Pattern::Product(
                    vec![(*left).into(), (*right).into()]
                )
            }
            PatternKind::Record { fields, .. } => {
                // Convert record patterns to product patterns for machine layer
                // Records are represented as nested products in the machine layer
                if fields.is_empty() {
                    crate::machine::Pattern::Wildcard
                } else if fields.len() == 1 {
                    // Single field record becomes a simple pattern
                    fields[0].pattern.clone().into()
                } else {
                    // Multiple fields become nested products: {a, b, c} → (a, (b, c))
                    let field_patterns: Vec<crate::machine::Pattern> = fields
                        .into_iter()
                        .map(|field| field.pattern.into())
                        .collect();
                    
                    // Create a product pattern from all fields
                    crate::machine::Pattern::Product(field_patterns)
                }
            }
            PatternKind::As { pattern, .. } => (*pattern).into(), // Strip the alias
            PatternKind::Or { patterns } => {
                // Take the first pattern as a simplification
                patterns.into_iter().next()
                    .map(|p| p.into())
                    .unwrap_or(crate::machine::Pattern::Wildcard)
            }
        }
    }
}

impl From<Literal> for crate::machine::LiteralValue {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Bool(b) => crate::machine::LiteralValue::Bool(b),
            Literal::Int(i) => crate::machine::LiteralValue::Int(i),
            Literal::Symbol(s) => crate::machine::LiteralValue::Symbol(s),
            Literal::Unit => crate::machine::LiteralValue::Unit,
        }
    }
}

//-----------------------------------------------------------------------------
// Helper Implementations
//-----------------------------------------------------------------------------

impl FieldPattern {
    /// Create a new field pattern
    pub fn new(field: String, pattern: Pattern) -> Self {
        Self { field, pattern }
    }
}

/// Convenience constructors
impl PatternKind {
    /// Create a unit literal pattern
    pub fn unit() -> Self {
        // Unit is not a literal in Layer 1, it's a separate term kind
        // For patterns, we treat it as a constructor with no arguments
        PatternKind::Constructor {
            tag: "unit".to_string(),
            args: vec![],
        }
    }
    
    /// Create a boolean literal pattern
    pub fn bool(value: bool) -> Self {
        PatternKind::Literal(Literal::Bool(value))
    }
    
    /// Create an integer literal pattern
    pub fn int(value: u32) -> Self {
        PatternKind::Literal(Literal::Int(value))
    }
    
    /// Create a symbol literal pattern
    pub fn symbol(value: String) -> Self {
        PatternKind::Literal(Literal::Symbol(value.into()))
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine;
    
    #[test]
    fn test_pattern_conversion_to_machine() {
        // Test wildcard conversion
        let wildcard = Pattern::wildcard();
        let machine_wildcard: machine::Pattern = wildcard.into();
        assert_eq!(machine_wildcard, machine::Pattern::Wildcard);
        
        // Test literal conversion
        let literal = Pattern::literal(Literal::Int(42));
        let machine_literal: machine::Pattern = literal.into();
        assert_eq!(machine_literal, machine::Pattern::Literal(machine::LiteralValue::Int(42)));
        
        // Test constructor conversion
        let constructor = Pattern::constructor("Some".to_string(), vec![
            Pattern::literal(Literal::Bool(true))
        ]);
        let machine_constructor: machine::Pattern = constructor.into();
        match machine_constructor {
            machine::Pattern::Constructor { name, patterns } => {
                assert_eq!(name, "Some");
                assert_eq!(patterns.len(), 1);
                assert_eq!(patterns[0], machine::Pattern::Literal(machine::LiteralValue::Bool(true)));
            }
            _ => panic!("Expected Constructor pattern"),
        }
        
        // Test product conversion
        let product = Pattern::product(
            Pattern::literal(Literal::Int(1)),
            Pattern::literal(Literal::Symbol("test".into()))
        );
        let machine_product: machine::Pattern = product.into();
        match machine_product {
            machine::Pattern::Product(patterns) => {
                assert_eq!(patterns.len(), 2);
                assert_eq!(patterns[0], machine::Pattern::Literal(machine::LiteralValue::Int(1)));
                assert_eq!(patterns[1], machine::Pattern::Literal(machine::LiteralValue::Symbol("test".into())));
            }
            _ => panic!("Expected Product pattern"),
        }
    }
    
    #[test]
    fn test_literal_conversion() {
        assert_eq!(
            machine::LiteralValue::from(Literal::Unit),
            machine::LiteralValue::Unit
        );
        assert_eq!(
            machine::LiteralValue::from(Literal::Bool(true)),
            machine::LiteralValue::Bool(true)
        );
        assert_eq!(
            machine::LiteralValue::from(Literal::Int(42)),
            machine::LiteralValue::Int(42)
        );
        assert_eq!(
            machine::LiteralValue::from(Literal::Symbol("test".into())),
            machine::LiteralValue::Symbol("test".into())
        );
    }
} 