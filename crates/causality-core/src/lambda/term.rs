//! Layer 1 term representation
//!
//! This module defines the term language for the linear lambda calculus.
//! These are the semantic terms that compile down to Layer 0 instructions.

use crate::lambda::base::TypeInner;
use crate::lambda::symbol::Symbol;
use crate::machine::MachineValue;
use serde::{Deserialize, Serialize};

//-----------------------------------------------------------------------------
// Core Term Structure
//-----------------------------------------------------------------------------

/// A term in the linear lambda calculus
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Term {
    /// The term kind
    pub kind: TermKind,

    /// Optional type annotation
    pub ty: Option<TypeInner>,
}

/// Different kinds of terms in Layer 1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    Tensor { left: Box<Term>, right: Box<Term> },

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
    Apply { func: Box<Term>, arg: Box<Term> },

    /// Resource allocation: alloc t
    Alloc { value: Box<Term> },

    /// Resource consumption: consume t
    Consume { resource: Box<Term> },

    /// Let binding: let x = t1 in t2
    Let {
        var: String,
        value: Box<Term>,
        body: Box<Term>,
    },

    // Session type constructors
    /// Create a new session channel: new_channel session_type
    NewChannel {
        session_type: super::base::SessionType,
    },

    /// Send a value on a channel: send channel_term value_term
    Send {
        channel: Box<Term>,
        value: Box<Term>,
    },

    /// Receive a value from a channel: receive channel_term
    Receive { channel: Box<Term> },

    /// Select a choice on an internal choice channel: select channel_term label
    Select { channel: Box<Term>, label: String },

    /// Branch on an external choice channel: case channel_term of { label1 -> t1 | label2 -> t2 | ... }
    Branch {
        channel: Box<Term>,
        branches: Vec<(String, Term)>,
    },

    /// Close a session channel: close channel_term
    Close { channel: Box<Term> },

    /// Fork a session into two endpoints: fork session_type (client_var server_var -> body)
    Fork {
        session_type: super::base::SessionType,
        client_var: String,
        server_var: String,
        body: Box<Term>,
    },

    /// Wait for a session to complete: wait channel_term body
    Wait { channel: Box<Term>, body: Box<Term> },

    // Transform type constructors
    /// Create a transform: transform input_type output_type location body
    Transform {
        input_type: TypeInner,
        output_type: TypeInner,
        location: super::base::Location,
        body: Box<Term>,
    },

    /// Apply a transform: apply_transform transform_term arg_term
    ApplyTransform {
        transform: Box<Term>,
        arg: Box<Term>,
    },

    /// Create a located computation: at location body
    At {
        location: super::base::Location,
        body: Box<Term>,
    },
}

//-----------------------------------------------------------------------------
// Literal Values
//-----------------------------------------------------------------------------

/// Literal values in Layer 1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn lambda_typed(
        param: impl Into<String>,
        param_type: TypeInner,
        body: Term,
    ) -> Self {
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

    // Session type term constructors

    /// Create a new session channel
    pub fn new_channel(session_type: super::base::SessionType) -> Self {
        Self::new(TermKind::NewChannel { session_type })
    }

    /// Send a value on a channel
    pub fn send(channel: Term, value: Term) -> Self {
        Self::new(TermKind::Send {
            channel: Box::new(channel),
            value: Box::new(value),
        })
    }

    /// Receive a value from a channel
    pub fn receive(channel: Term) -> Self {
        Self::new(TermKind::Receive {
            channel: Box::new(channel),
        })
    }

    /// Select a choice on an internal choice channel
    pub fn select(channel: Term, label: impl Into<String>) -> Self {
        Self::new(TermKind::Select {
            channel: Box::new(channel),
            label: label.into(),
        })
    }

    /// Branch on an external choice channel
    pub fn branch(channel: Term, branches: Vec<(String, Term)>) -> Self {
        Self::new(TermKind::Branch {
            channel: Box::new(channel),
            branches,
        })
    }

    /// Close a session channel
    pub fn close(channel: Term) -> Self {
        Self::new(TermKind::Close {
            channel: Box::new(channel),
        })
    }

    /// Fork a session into two endpoints
    pub fn fork(
        session_type: super::base::SessionType,
        client_var: impl Into<String>,
        server_var: impl Into<String>,
        body: Term,
    ) -> Self {
        Self::new(TermKind::Fork {
            session_type,
            client_var: client_var.into(),
            server_var: server_var.into(),
            body: Box::new(body),
        })
    }

    /// Wait for a session to complete
    pub fn wait(channel: Term, body: Term) -> Self {
        Self::new(TermKind::Wait {
            channel: Box::new(channel),
            body: Box::new(body),
        })
    }

    // Transform type term constructors

    /// Create a transform
    pub fn transform(
        input_type: TypeInner,
        output_type: TypeInner,
        location: super::base::Location,
        body: Term,
    ) -> Self {
        Self::new(TermKind::Transform {
            input_type,
            output_type,
            location,
            body: Box::new(body),
        })
    }

    /// Apply a transform
    pub fn apply_transform(transform: Term, arg: Term) -> Self {
        Self::new(TermKind::ApplyTransform {
            transform: Box::new(transform),
            arg: Box::new(arg),
        })
    }

    /// Create a located computation
    pub fn at(location: super::base::Location, body: Term) -> Self {
        Self::new(TermKind::At {
            location,
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
    use super::{Literal, Term, TermKind}; // Items from the parent module (term.rs)
    use crate::lambda::base::{BaseType, TypeInner}; // Types from elsewhere in the crate
    use crate::lambda::symbol::Symbol; // Symbol type

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

        if let TermKind::LetTensor {
            tensor_term,
            left_var,
            right_var,
            body,
        } = term.kind
        {
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
        if let TermKind::Inl {
            value,
            sum_type: ty_ann,
        } = term_inl.kind
        {
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
        if let TermKind::Inr {
            value,
            sum_type: ty_ann,
        } = term_inr.kind
        {
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

        if let TermKind::Case {
            scrutinee,
            left_var,
            left_body,
            right_var,
            right_body,
        } = term.kind
        {
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
        if let TermKind::Lambda {
            param,
            param_type,
            body,
        } = term_lambda_untyped.kind
        {
            assert_eq!(param, "x");
            assert_eq!(param_type, None);
            assert_eq!(*body, body_val);
        } else {
            panic!("Expected TermKind::Lambda (untyped)");
        }

        // Lambda with type annotation
        let term_lambda_typed =
            Term::lambda_typed("y", param_ty_val.clone(), body_val.clone());
        if let TermKind::Lambda {
            param,
            param_type,
            body,
        } = term_lambda_typed.kind
        {
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

    // --- Test Session Type Term Construction

    #[test]
    fn test_term_new_channel() {
        use crate::lambda::base::SessionType;

        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End),
        );
        let term = Term::new_channel(session_type.clone());

        if let TermKind::NewChannel { session_type: st } = term.kind {
            assert_eq!(st, session_type);
        } else {
            panic!("Expected TermKind::NewChannel");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_send() {
        let channel_term = Term::var("ch");
        let value_term = Term::literal(Literal::Int(42));
        let term = Term::send(channel_term.clone(), value_term.clone());

        if let TermKind::Send { channel, value } = term.kind {
            assert_eq!(*channel, channel_term);
            assert_eq!(*value, value_term);
        } else {
            panic!("Expected TermKind::Send");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_receive() {
        let channel_term = Term::var("ch");
        let term = Term::receive(channel_term.clone());

        if let TermKind::Receive { channel } = term.kind {
            assert_eq!(*channel, channel_term);
        } else {
            panic!("Expected TermKind::Receive");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_select() {
        let channel_term = Term::var("ch");
        let label = "option_a";
        let term = Term::select(channel_term.clone(), label);

        if let TermKind::Select {
            channel,
            label: selected_label,
        } = term.kind
        {
            assert_eq!(*channel, channel_term);
            assert_eq!(selected_label, label);
        } else {
            panic!("Expected TermKind::Select");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_branch() {
        let channel_term = Term::var("ch");
        let branches = vec![
            ("option_a".to_string(), Term::var("handle_a")),
            ("option_b".to_string(), Term::var("handle_b")),
        ];
        let term = Term::branch(channel_term.clone(), branches.clone());

        if let TermKind::Branch {
            channel,
            branches: term_branches,
        } = term.kind
        {
            assert_eq!(*channel, channel_term);
            assert_eq!(term_branches, branches);
        } else {
            panic!("Expected TermKind::Branch");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_close() {
        let channel_term = Term::var("ch");
        let term = Term::close(channel_term.clone());

        if let TermKind::Close { channel } = term.kind {
            assert_eq!(*channel, channel_term);
        } else {
            panic!("Expected TermKind::Close");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_fork() {
        use crate::lambda::base::SessionType;

        let session_type = SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(SessionType::End),
        );
        let client_var = "client";
        let server_var = "server";
        let body = Term::var("interaction");

        let term =
            Term::fork(session_type.clone(), client_var, server_var, body.clone());

        if let TermKind::Fork {
            session_type: st,
            client_var: cv,
            server_var: sv,
            body: term_body,
        } = term.kind
        {
            assert_eq!(st, session_type);
            assert_eq!(cv, client_var);
            assert_eq!(sv, server_var);
            assert_eq!(*term_body, body);
        } else {
            panic!("Expected TermKind::Fork");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_term_wait() {
        let channel_term = Term::var("ch");
        let body = Term::var("continuation");
        let term = Term::wait(channel_term.clone(), body.clone());

        if let TermKind::Wait {
            channel,
            body: term_body,
        } = term.kind
        {
            assert_eq!(*channel, channel_term);
            assert_eq!(*term_body, body);
        } else {
            panic!("Expected TermKind::Wait");
        }
        assert_eq!(term.ty, None);
    }

    #[test]
    fn test_session_term_composition() {
        // Test that we can compose session operations
        let session_type = crate::lambda::base::SessionType::Send(
            Box::new(TypeInner::Base(BaseType::Int)),
            Box::new(crate::lambda::base::SessionType::End),
        );

        let fork_term = Term::fork(
            session_type,
            "client",
            "server",
            Term::send(Term::var("client"), Term::literal(Literal::Int(42))),
        );

        // Should compile without issues
        assert!(matches!(fork_term.kind, TermKind::Fork { .. }));
    }

    // --- Test Transform Type Term Construction ---

    #[test]
    fn test_term_transform() {
        use crate::lambda::base::{BaseType, Location, TypeInner};

        let input_type = TypeInner::Base(BaseType::Int);
        let output_type = TypeInner::Base(BaseType::Bool);
        let location = Location::remote("server");
        let body = Term::var("x");

        let transform_term = Term::transform(
            input_type.clone(),
            output_type.clone(),
            location.clone(),
            body.clone(),
        );

        if let TermKind::Transform {
            input_type: it,
            output_type: ot,
            location: loc,
            body: b,
        } = transform_term.kind
        {
            assert_eq!(it, input_type);
            assert_eq!(ot, output_type);
            assert_eq!(loc, location);
            assert_eq!(*b, body);
        } else {
            panic!("Expected TermKind::Transform");
        }
    }

    #[test]
    fn test_term_apply_transform() {
        use crate::lambda::base::{BaseType, Location, TypeInner};

        let transform_term = Term::transform(
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::Bool),
            Location::Local,
            Term::var("x"),
        );
        let arg_term = Term::literal(Literal::Int(42));

        let apply_term =
            Term::apply_transform(transform_term.clone(), arg_term.clone());

        if let TermKind::ApplyTransform { transform, arg } = apply_term.kind {
            assert_eq!(*transform, transform_term);
            assert_eq!(*arg, arg_term);
        } else {
            panic!("Expected TermKind::ApplyTransform");
        }
    }

    #[test]
    fn test_term_at() {
        use crate::lambda::base::Location;

        let location = Location::remote("gpu_cluster");
        let body = Term::apply(Term::var("f"), Term::var("x"));

        let at_term = Term::at(location.clone(), body.clone());

        if let TermKind::At {
            location: loc,
            body: b,
        } = at_term.kind
        {
            assert_eq!(loc, location);
            assert_eq!(*b, body);
        } else {
            panic!("Expected TermKind::At");
        }
    }

    #[test]
    fn test_transform_composition() {
        use crate::lambda::base::{BaseType, Location, TypeInner};

        // Create a transform that doubles an integer
        let double_transform = Term::transform(
            TypeInner::Base(BaseType::Int),
            TypeInner::Base(BaseType::Int),
            Location::Local,
            Term::apply(
                Term::apply(Term::var("mul"), Term::var("x")),
                Term::literal(Literal::Int(2)),
            ),
        );

        // Apply it to an argument
        let application =
            Term::apply_transform(double_transform, Term::literal(Literal::Int(21)));

        // Should compile without issues
        assert!(matches!(application.kind, TermKind::ApplyTransform { .. }));
    }

    #[test]
    fn test_located_computation() {
        use crate::lambda::base::Location;

        // Create a computation that runs on a remote server
        let remote_computation = Term::at(
            Location::remote("gpu_cluster"),
            Term::apply(
                Term::var("expensive_computation"),
                Term::var("large_dataset"),
            ),
        );

        // Should compile without issues
        assert!(matches!(remote_computation.kind, TermKind::At { .. }));
    }
}
