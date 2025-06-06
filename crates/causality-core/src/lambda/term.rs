//! Layer 1 term representation
//!
//! This module defines the term language for the linear lambda calculus.
//! These are the semantic terms that compile down to Layer 0 instructions.

use super::{TypeInner, Symbol};
use crate::MachineValue;

//-----------------------------------------------------------------------------
// Core Term Structure
//-----------------------------------------------------------------------------

/// A term in the linear lambda calculus
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    /// The term kind
    pub kind: TermKind,
    
    /// Optional type annotation
    pub ty: Option<TypeInner>,
}

/// Different kinds of terms in Layer 1
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermKind {
    /// Variable reference
    Var(String),
    
    /// Literal value
    Literal(Literal),
    
    /// Unit value introduction
    Unit,
    
    /// Unit elimination: letunit t1 t2
    LetUnit {
        unit_term: Box<Term>,
        body: Box<Term>,
    },
    
    /// Tensor introduction: tensor t1 t2
    Tensor {
        left: Box<Term>,
        right: Box<Term>,
    },
    
    /// Tensor elimination: lettensor t1 (x y -> t2)
    LetTensor {
        tensor_term: Box<Term>,
        left_var: String,
        right_var: String,
        body: Box<Term>,
    },
    
    /// Left injection: inl t
    Inl {
        value: Box<Term>,
        sum_type: TypeInner,
    },
    
    /// Right injection: inr t
    Inr {
        value: Box<Term>,
        sum_type: TypeInner,
    },
    
    /// Case analysis: case t of { inl x -> t1 | inr y -> t2 }
    Case {
        scrutinee: Box<Term>,
        left_var: String,
        left_body: Box<Term>,
        right_var: String,
        right_body: Box<Term>,
    },
    
    /// Lambda abstraction: λx. t
    Lambda {
        param: String,
        param_type: Option<TypeInner>,
        body: Box<Term>,
    },
    
    /// Function application: t1 t2
    Apply {
        func: Box<Term>,
        arg: Box<Term>,
    },
    
    /// Resource allocation: alloc t
    Alloc {
        value: Box<Term>,
    },
    
    /// Resource consumption: consume t
    Consume {
        resource: Box<Term>,
    },
    
    /// Let binding: let x = t1 in t2
    Let {
        var: String,
        value: Box<Term>,
        body: Box<Term>,
    },
}

//-----------------------------------------------------------------------------
// Literal Values
//-----------------------------------------------------------------------------

/// Literal values in Layer 1
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    /// Unit literal
    Unit,
    
    /// Boolean literal
    Bool(bool),
    
    /// Integer literal
    Int(u32),
    
    /// Symbol literal
    Symbol(Symbol),
}

//-----------------------------------------------------------------------------
// Term Construction Helpers
//-----------------------------------------------------------------------------

impl Term {
    /// Create a new term with the given kind
    pub fn new(kind: TermKind) -> Self {
        Self { kind, ty: None }
    }
    
    /// Create a term with a type annotation
    pub fn with_type(mut self, ty: TypeInner) -> Self {
        self.ty = Some(ty);
        self
    }
    
    /// Create a variable term
    pub fn var(name: impl Into<String>) -> Self {
        Self::new(TermKind::Var(name.into()))
    }
    
    /// Create a literal term
    pub fn literal(lit: Literal) -> Self {
        Self::new(TermKind::Literal(lit))
    }
    
    /// Create a unit term
    pub fn unit() -> Self {
        Self::new(TermKind::Unit)
    }
    
    /// Create a tensor term
    pub fn tensor(left: Term, right: Term) -> Self {
        Self::new(TermKind::Tensor {
            left: Box::new(left),
            right: Box::new(right),
        })
    }
    
    /// Create a lambda term
    pub fn lambda(param: impl Into<String>, body: Term) -> Self {
        Self::new(TermKind::Lambda {
            param: param.into(),
            param_type: None,
            body: Box::new(body),
        })
    }
    
    /// Create a lambda term with parameter type
    pub fn lambda_typed(param: impl Into<String>, param_type: TypeInner, body: Term) -> Self {
        Self::new(TermKind::Lambda {
            param: param.into(),
            param_type: Some(param_type),
            body: Box::new(body),
        })
    }
    
    /// Create an application term
    pub fn apply(func: Term, arg: Term) -> Self {
        Self::new(TermKind::Apply {
            func: Box::new(func),
            arg: Box::new(arg),
        })
    }
    
    /// Create an alloc term
    pub fn alloc(value: Term) -> Self {
        Self::new(TermKind::Alloc {
            value: Box::new(value),
        })
    }
    
    /// Create a consume term
    pub fn consume(resource: Term) -> Self {
        Self::new(TermKind::Consume {
            resource: Box::new(resource),
        })
    }
    
    /// Create a let binding
    pub fn let_bind(var: impl Into<String>, value: Term, body: Term) -> Self {
        Self::new(TermKind::Let {
            var: var.into(),
            value: Box::new(value),
            body: Box::new(body),
        })
    }
}

//-----------------------------------------------------------------------------
// Conversion to Machine Values
//-----------------------------------------------------------------------------

impl From<Literal> for MachineValue {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Unit => MachineValue::Unit,
            Literal::Bool(b) => MachineValue::Bool(b),
            Literal::Int(i) => MachineValue::Int(i),
            Literal::Symbol(s) => MachineValue::Symbol(s),
        }
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{Term, TermKind, Literal}; // Items from the parent module (term.rs)
    use crate::lambda::base::{BaseType, TypeInner}; // Types from elsewhere in the crate
    use crate::lambda::symbol::Symbol;      // Symbol type

    // --- Test Term Construction: Variables and Literals

    #[test]
    fn test_term_var() {
        let term = Term::var("x");
        assert_eq!(term.kind, TermKind::Var("x".to_string()));
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_literal() {
        // --- Bool literal
        let term_bool = Term::literal(Literal::Bool(true));
        assert_eq!(term_bool.kind, TermKind::Literal(Literal::Bool(true)));
        assert_eq!(term_bool.ty, None);

        // --- Int literal
        let term_int = Term::literal(Literal::Int(123));
        assert_eq!(term_int.kind, TermKind::Literal(Literal::Int(123)));
        assert_eq!(term_int.ty, None);

        // --- Symbol literal
        let sym = Symbol::new("my_symbol");
        let term_sym = Term::literal(Literal::Symbol(sym.clone()));
        assert_eq!(term_sym.kind, TermKind::Literal(Literal::Symbol(sym)));
        assert_eq!(term_sym.ty, None);
    }

    // --- Test Term Construction: Unit Type Primitives

    #[test]
    fn test_term_unit() {
        let term = Term::unit();
        assert_eq!(term.kind, TermKind::Unit);
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_let_unit() {
        let unit_val = Term::unit();
        let body_val = Term::var("x");
        let term = Term::new(TermKind::LetUnit {
            unit_term: Box::new(unit_val.clone()),
            body: Box::new(body_val.clone()),
        });

        if let TermKind::LetUnit { unit_term, body } = term.kind {
            assert_eq!(*unit_term, unit_val);
            assert_eq!(*body, body_val);
        } else {
            panic!("Expected TermKind::LetUnit");
        }
        assert_eq!(term.ty, None);
    }

    // --- Test Term Construction: Tensor Product Primitives

    #[test]
    fn test_term_tensor() {
        let left_val = Term::var("a");
        let right_val = Term::var("b");
        let term = Term::tensor(left_val.clone(), right_val.clone());

        if let TermKind::Tensor { left, right } = term.kind {
            assert_eq!(*left, left_val);
            assert_eq!(*right, right_val);
        } else {
            panic!("Expected TermKind::Tensor");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_let_tensor() {
        let tensor_val = Term::var("pair");
        let body_val = Term::var("l"); // Using one of the bound vars
        let term = Term::new(TermKind::LetTensor {
            tensor_term: Box::new(tensor_val.clone()),
            left_var: "l".to_string(),
            right_var: "r".to_string(),
            body: Box::new(body_val.clone()),
        });

        if let TermKind::LetTensor { tensor_term, left_var, right_var, body } = term.kind {
            assert_eq!(*tensor_term, tensor_val);
            assert_eq!(left_var, "l");
            assert_eq!(right_var, "r");
            assert_eq!(*body, body_val);
        } else {
            panic!("Expected TermKind::LetTensor");
        }
        assert_eq!(term.ty, None);
    }

    // --- Test Term Construction: Sum Type Primitives

    #[test]
    fn test_term_inl_inr() {
        let val_int = Term::literal(Literal::Int(1));
        let val_bool = Term::literal(Literal::Bool(true));
        let sum_ty = TypeInner::Sum(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(TypeInner::Base(BaseType::Bool)),
        );

        // Inl
        let term_inl = Term::new(TermKind::Inl {
            value: Box::new(val_int.clone()),
            sum_type: sum_ty.clone(),
        });
        if let TermKind::Inl { value, sum_type: ty_ann } = term_inl.kind {
            assert_eq!(*value, val_int);
            assert_eq!(ty_ann, sum_ty);
        } else {
            panic!("Expected TermKind::Inl");
        }

        // Inr
        let term_inr = Term::new(TermKind::Inr {
            value: Box::new(val_bool.clone()),
            sum_type: sum_ty.clone(),
        });
        if let TermKind::Inr { value, sum_type: ty_ann } = term_inr.kind {
            assert_eq!(*value, val_bool);
            assert_eq!(ty_ann, sum_ty);
        } else {
            panic!("Expected TermKind::Inr");
        }
    }

    #[test]
    fn test_term_case() {
        let scrutinee_val = Term::var("s");
        let left_body_val = Term::var("x_val");
        let right_body_val = Term::var("y_val");

        let term = Term::new(TermKind::Case {
            scrutinee: Box::new(scrutinee_val.clone()),
            left_var: "x".to_string(),
            left_body: Box::new(left_body_val.clone()),
            right_var: "y".to_string(),
            right_body: Box::new(right_body_val.clone()),
        });

        if let TermKind::Case { scrutinee, left_var, left_body, right_var, right_body } = term.kind {
            assert_eq!(*scrutinee, scrutinee_val);
            assert_eq!(left_var, "x");
            assert_eq!(*left_body, left_body_val);
            assert_eq!(right_var, "y");
            assert_eq!(*right_body, right_body_val);
        } else {
            panic!("Expected TermKind::Case");
        }
        assert_eq!(term.ty, None);
    }

    // --- Test Term Construction: Function Type Primitives

    #[test]
    fn test_term_lambda() {
        let body_val = Term::var("x");
        let param_ty_val = TypeInner::Base(BaseType::Int);

        // Lambda without type annotation
        let term_lambda_untyped = Term::lambda("x", body_val.clone());
        if let TermKind::Lambda { param, param_type, body } = term_lambda_untyped.kind {
            assert_eq!(param, "x");
            assert_eq!(param_type, None);
            assert_eq!(*body, body_val);
        } else {
            panic!("Expected TermKind::Lambda (untyped)");
        }

        // Lambda with type annotation
        let term_lambda_typed = Term::lambda_typed("y", param_ty_val.clone(), body_val.clone());
        if let TermKind::Lambda { param, param_type, body } = term_lambda_typed.kind {
            assert_eq!(param, "y");
            assert_eq!(param_type, Some(param_ty_val));
            assert_eq!(*body, body_val);
        } else {
            panic!("Expected TermKind::Lambda (typed)");
        }
    }

    #[test]
    fn test_term_apply() {
        let func_val = Term::var("f");
        let arg_val = Term::var("arg");
        let term = Term::apply(func_val.clone(), arg_val.clone());

        if let TermKind::Apply { func, arg } = term.kind {
            assert_eq!(*func, func_val);
            assert_eq!(*arg, arg_val);
        } else {
            panic!("Expected TermKind::Apply");
        }
        assert_eq!(term.ty, None);
    }

    // --- Test Term Construction: Resource Primitives

    #[test]
    fn test_term_alloc() {
        let value_to_alloc = Term::literal(Literal::Int(10));
        let term = Term::alloc(value_to_alloc.clone());

        if let TermKind::Alloc { value } = term.kind {
            assert_eq!(*value, value_to_alloc);
        } else {
            panic!("Expected TermKind::Alloc");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_consume() {
        let resource_to_consume = Term::var("resource_id");
        let term = Term::consume(resource_to_consume.clone());

        if let TermKind::Consume { resource } = term.kind {
            assert_eq!(*resource, resource_to_consume);
        } else {
            panic!("Expected TermKind::Consume");
        }
        assert_eq!(term.ty, None);
    }

    // --- Test Term Construction: Let Binding

    #[test]
    fn test_term_let() {
        let val_to_bind = Term::literal(Literal::Int(5));
        let body_val = Term::var("x");
        let term = Term::let_bind("x", val_to_bind.clone(), body_val.clone());

        if let TermKind::Let { var, value, body } = term.kind {
            assert_eq!(var, "x");
            assert_eq!(*value, val_to_bind);
            assert_eq!(*body, body_val);
        } else {
            panic!("Expected TermKind::Let");
        }
        assert_eq!(term.ty, None);
    }

    // --- Test Term Construction: Type Annotation

    #[test]
    fn test_term_with_type() {
        let mut term = Term::var("z");
        let ty_annotation = TypeInner::Base(BaseType::Bool);
        term = term.with_type(ty_annotation.clone());

        assert_eq!(term.ty, Some(ty_annotation));
    }
}