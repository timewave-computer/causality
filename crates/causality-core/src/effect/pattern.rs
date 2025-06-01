//! Pattern matching support
//!
//! This module implements pattern matching constructs used in
//! match expressions and let bindings.

use super::core::Span;
use crate::lambda::{term::Literal, TypeInner};

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
    pub fn is_exhaustive_for(&self, _type: &TypeInner) -> bool {
        // TODO: Implement exhaustiveness checking
        // This would require type information and constructor analysis
        false
    }
}

//-----------------------------------------------------------------------------
// Conversion to Machine Patterns
//-----------------------------------------------------------------------------

impl From<Pattern> for crate::machine::Pattern {
    fn from(pat: Pattern) -> Self {
        match pat.kind {
            PatternKind::Wildcard => crate::machine::Pattern::Wildcard,
            
            PatternKind::Var(_) => {
                // Variables become register bindings in IR
                // The actual register allocation happens during compilation
                crate::machine::Pattern::Wildcard // Placeholder
            }
            
            PatternKind::Literal(lit) => {
                crate::machine::Pattern::Literal(lit.into())
            }
            
            PatternKind::Constructor { tag, args } => {
                crate::machine::Pattern::Constructor {
                    tag: tag.into(),
                    args: args.into_iter().map(|p| p.into()).collect(),
                }
            }
            
            PatternKind::Product { left, right } => {
                crate::machine::Pattern::Product(
                    Box::new((*left).into()),
                    Box::new((*right).into()),
                )
            }
            
            // Other patterns would need more complex Machine compilation
            _ => crate::machine::Pattern::Wildcard, // Placeholder
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

// Helper to convert lambda Literal to machine LiteralValue
impl From<Literal> for crate::machine::LiteralValue {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Bool(b) => crate::machine::LiteralValue::Bool(b),
            Literal::Int(i) => crate::machine::LiteralValue::Int(i),
            Literal::Symbol(s) => crate::machine::LiteralValue::Symbol(s),
        }
    }
} 