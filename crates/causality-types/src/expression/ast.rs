//! Expression AST (Abstract Syntax Tree) for the Causality framework.
//!
//! This module defines the execution layer of the Causality type system, implementing
//! a pure, content-addressed Merkle-serialized AST for constraint expressions and
//! computations. It provides operations that manipulate values according to type schemas,
//! maintaining strict determinism for ZK compatibility.
//!
//! The AST is based on a combinator approach, providing a minimal but powerful set of
//! operations suitable for ZK circuit evaluation.

use crate::primitive::string::Str;
use crate::expression::value::ValueExpr;
use crate::serialization::{Decode, DecodeError, DecodeWithLength, Encode, SimpleSerialize};
use crate::primitive::ids::{DomainId, NodeId, ExprId};
use anyhow::Result;
use sha2::Digest;
use std::collections::HashMap;

//-----------------------------------------------------------------------------
// Wrapper Types for Breaking Recursion
//-----------------------------------------------------------------------------

// These wrapper types are used throughout the codebase to break recursive type
// definitions for serialization. Without them, the compiler couldn't determine
// the size of recursive types, and serialization would fail with stack overflows.

/// Box wrapper for Expr
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprBox(pub Box<Expr>);

impl Encode for ExprBox {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}
impl Decode for ExprBox {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(ExprBox(Box::new(Expr::from_ssz_bytes(bytes)?)))
    }
}
impl SimpleSerialize for ExprBox {}

impl DecodeWithLength for ExprBox {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (expr, consumed) = Expr::from_ssz_bytes_with_length(bytes)?;
        Ok((ExprBox(Box::new(expr)), consumed))
    }
}

/// Vector wrapper for Expr collections
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprVec(pub Vec<Expr>);

impl Encode for ExprVec {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}
impl Decode for ExprVec {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(ExprVec(Vec::<Expr>::from_ssz_bytes(bytes)?))
    }
}
impl SimpleSerialize for ExprVec {}

impl DecodeWithLength for ExprVec {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (vec, consumed) = Vec::<Expr>::from_ssz_bytes_with_length(bytes)?;
        Ok((ExprVec(vec), consumed))
    }
}

/// Vector wrapper for pattern matching key-value pairs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprPairs(pub Vec<(Expr, Expr)>);

impl Encode for ExprPairs {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}
impl Decode for ExprPairs {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(ExprPairs(Vec::<(Expr, Expr)>::from_ssz_bytes(bytes)?))
    }
}
impl SimpleSerialize for ExprPairs {}

//-----------------------------------------------------------------------------
// Combinator Definition
//-----------------------------------------------------------------------------

/// Atomic combinators that form the foundation of the combinator Lisp
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
#[repr(align(4))]
pub enum AtomicCombinator {
    //--- Core Combinators
    /// S combinator: (S f g x) = (f x (g x))
    S,
    /// K combinator: (K x y) = x
    K,
    /// I combinator: (I x) = x
    I,
    /// C combinator: (C p t f) = if p then t else f
    C,

    //--- Control Flow
    /// (if cond then else)
    If,
    /// (let [bindings...] body)
    Let,

    //--- Logical Operations
    /// (and expr1 expr2 ... exprN)
    And,
    /// (or expr1 expr2 ... exprN)
    Or,
    /// (not expr)
    Not,

    //--- Equality
    /// (eq? expr1 expr2)
    Eq,

    //--- Arithmetic
    /// (+ expr1 expr2)
    Add,
    /// (- expr1 expr2)
    Sub,
    /// (* expr1 expr2)
    Mul,
    /// (/ expr1 expr2)
    Div,

    //--- Comparison
    /// (> expr1 expr2)
    Gt,
    /// (< expr1 expr2)
    Lt,
    /// (>= expr1 expr2)
    Gte,
    /// (<= expr1 expr2)
    Lte,

    //--- Data Access & Construction
    /// (get-context-value key_str)
    GetContextValue,
    /// (get-field target field_name)
    GetField,
    /// (completed effect_ref)
    Completed,

    //--- List Operations
    /// (list elem1 elem2 ... elemN)
    List,

    //--- Extended List/Map Operations ---
    /// (nth index list)
    Nth,
    /// (length list) - Gets the length of a list
    Length,
    /// (cons item list)
    Cons,
    /// (car list) - Gets the first element of a list
    Car,
    /// (cdr list) - Gets the rest of a list (all but the first element)
    Cdr,
    /// (make-map (key1 val1) (key2 val2) ...)
    MakeMap,
    /// (map-get key map) - Gets a value from a map by key
    MapGet,
    /// (map-has-key? key map) - Checks if a map contains a key
    MapHasKey,

    //--- Definitions ---
    /// (define symbol value-expr) - Defines a symbol in the current scope.
    Define,
    /// (defun name params-list body-expr) - Defines a function.
    Defun,
    
    //--- Special Forms ---
    /// (quote expr) - Returns the unevaluated expression.
    Quote,
}

impl Encode for AtomicCombinator {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        (*self as u8).as_ssz_bytes()
    }
}

impl Decode for AtomicCombinator {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let val = u8::from_ssz_bytes(bytes)?;
        match val {
            0 => Ok(AtomicCombinator::S),
            1 => Ok(AtomicCombinator::K),
            2 => Ok(AtomicCombinator::I),
            3 => Ok(AtomicCombinator::C),
            4 => Ok(AtomicCombinator::If),
            5 => Ok(AtomicCombinator::Let),
            6 => Ok(AtomicCombinator::And),
            7 => Ok(AtomicCombinator::Or),
            8 => Ok(AtomicCombinator::Not),
            9 => Ok(AtomicCombinator::Eq),
            10 => Ok(AtomicCombinator::Add),
            11 => Ok(AtomicCombinator::Sub),
            12 => Ok(AtomicCombinator::Mul),
            13 => Ok(AtomicCombinator::Div),
            14 => Ok(AtomicCombinator::Gt),
            15 => Ok(AtomicCombinator::Lt),
            16 => Ok(AtomicCombinator::Gte),
            17 => Ok(AtomicCombinator::Lte),
            18 => Ok(AtomicCombinator::GetContextValue),
            19 => Ok(AtomicCombinator::GetField),
            20 => Ok(AtomicCombinator::Completed),
            21 => Ok(AtomicCombinator::List),
            22 => Ok(AtomicCombinator::Nth),
            23 => Ok(AtomicCombinator::Length),
            24 => Ok(AtomicCombinator::Cons),
            25 => Ok(AtomicCombinator::Car),
            26 => Ok(AtomicCombinator::Cdr),
            27 => Ok(AtomicCombinator::MakeMap),
            28 => Ok(AtomicCombinator::MapGet),
            29 => Ok(AtomicCombinator::MapHasKey),
            30 => Ok(AtomicCombinator::Define),
            31 => Ok(AtomicCombinator::Defun),
            32 => Ok(AtomicCombinator::Quote),
            _ => Err(DecodeError {
                message: format!("Invalid AtomicCombinator value: {}", val),
            }),
        }
    }
}

impl SimpleSerialize for AtomicCombinator {}

impl DecodeWithLength for AtomicCombinator {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        let combinator = Self::from_ssz_bytes(bytes)?;
        Ok((combinator, 1)) // AtomicCombinator is always 1 byte
    }
}

//-----------------------------------------------------------------------------
// Atom Definition
//-----------------------------------------------------------------------------

/// Atomic values in expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    /// Integer value
    Integer(i64),
    /// String value
    String(Str),
    /// Boolean value
    Boolean(bool),
    /// Nil/null value
    Nil,
}

impl Encode for Atom {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Atom::Integer(i) => {
                bytes.push(0u8); // tag
                bytes.extend_from_slice(&i.to_le_bytes());
            }
            Atom::String(s) => {
                bytes.push(1u8); // tag
                bytes.extend_from_slice(&s.as_ssz_bytes());
            }
            Atom::Boolean(b) => {
                bytes.push(2u8); // tag
                bytes.push(if *b { 1u8 } else { 0u8 });
            }
            Atom::Nil => {
                bytes.push(3u8); // tag
            }
        }
        bytes
    }
}

impl Decode for Atom {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Empty bytes for Atom".to_string(),
            });
        }
        
        match bytes[0] {
            0 => {
                if bytes.len() < 9 {
                    return Err(DecodeError {
                        message: "Insufficient bytes for Integer".to_string(),
                    });
                }
                let mut int_bytes = [0u8; 8];
                int_bytes.copy_from_slice(&bytes[1..9]);
                Ok(Atom::Integer(i64::from_le_bytes(int_bytes)))
            }
            1 => {
                let s = Str::from_ssz_bytes(&bytes[1..])?;
                Ok(Atom::String(s))
            }
            2 => {
                if bytes.len() < 2 {
                    return Err(DecodeError {
                        message: "Insufficient bytes for Boolean".to_string(),
                    });
                }
                Ok(Atom::Boolean(bytes[1] != 0))
            }
            3 => Ok(Atom::Nil),
            _ => Err(DecodeError {
                message: format!("Invalid Atom tag: {}", bytes[0]),
            }),
        }
    }
}

impl SimpleSerialize for Atom {}

impl DecodeWithLength for Atom {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Empty bytes for Atom".to_string(),
            });
        }
        
        match bytes[0] {
            0 => {
                if bytes.len() < 9 {
                    return Err(DecodeError {
                        message: "Insufficient bytes for Integer".to_string(),
                    });
                }
                let mut int_bytes = [0u8; 8];
                int_bytes.copy_from_slice(&bytes[1..9]);
                Ok((Atom::Integer(i64::from_le_bytes(int_bytes)), 9))
            }
            1 => {
                let (s, consumed) = Str::from_ssz_bytes_with_length(&bytes[1..])?;
                Ok((Atom::String(s), consumed + 1))
            }
            2 => {
                if bytes.len() < 2 {
                    return Err(DecodeError {
                        message: "Insufficient bytes for Boolean".to_string(),
                    });
                }
                Ok((Atom::Boolean(bytes[1] != 0), 2))
            }
            3 => Ok((Atom::Nil, 1)),
            _ => Err(DecodeError {
                message: format!("Invalid Atom tag: {}", bytes[0]),
            }),
        }
    }
}

//-----------------------------------------------------------------------------
// Expression Definition
//-----------------------------------------------------------------------------

/// Expression AST node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// Atomic value (number, string, boolean, nil)
    Atom(Atom),

    /// Constant value (for backward compatibility)
    Const(ValueExpr),

    /// Variable reference
    Var(Str),

    /// Lambda abstraction (anonymous function)
    /// Parameters are a list of identifiers, body is a boxed expression
    Lambda(Vec<Str>, ExprBox),

    /// Function application
    /// Function is a boxed expression, arguments are a list of expressions
    Apply(ExprBox, ExprVec),

    /// Atomic combinator (includes all predefined combinators)
    Combinator(AtomicCombinator),

    /// Dynamic expression (step-bounded evaluation for ZK coprocessor)
    /// Parameters: N (step bound), expr (expression to evaluate)
    Dynamic(u32, ExprBox),
}

impl Encode for Expr {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Expr::Atom(atom) => {
                bytes.push(0u8); // tag
                bytes.extend_from_slice(&atom.as_ssz_bytes());
            }
            Expr::Const(value) => {
                bytes.push(1u8); // tag
                bytes.extend_from_slice(&value.as_ssz_bytes());
            }
            Expr::Var(name) => {
                bytes.push(2u8); // tag
                bytes.extend_from_slice(&name.as_ssz_bytes());
            }
            Expr::Lambda(params, body) => {
                bytes.push(3u8); // tag
                bytes.extend_from_slice(&params.as_ssz_bytes());
                bytes.extend_from_slice(&body.as_ssz_bytes());
            }
            Expr::Apply(func, args) => {
                bytes.push(4u8); // tag
                bytes.extend_from_slice(&func.as_ssz_bytes());
                bytes.extend_from_slice(&args.as_ssz_bytes());
            }
            Expr::Combinator(comb) => {
                bytes.push(5u8); // tag
                bytes.extend_from_slice(&comb.as_ssz_bytes());
            }
            Expr::Dynamic(steps, expr) => {
                bytes.push(6u8); // tag
                bytes.extend_from_slice(&steps.to_le_bytes());
                bytes.extend_from_slice(&expr.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl DecodeWithLength for Expr {
    fn from_ssz_bytes_with_length(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "Empty bytes for Expr".to_string(),
            });
        }
        
        let mut offset = 1; // Skip tag byte
        
        match bytes[0] {
            0 => {
                let (atom, consumed) = Atom::from_ssz_bytes_with_length(&bytes[offset..])?;
                Ok((Expr::Atom(atom), offset + consumed))
            }
            1 => {
                let (value, consumed) = ValueExpr::from_ssz_bytes_with_length(&bytes[offset..])?;
                Ok((Expr::Const(value), offset + consumed))
            }
            2 => {
                let (name, consumed) = Str::from_ssz_bytes_with_length(&bytes[offset..])?;
                Ok((Expr::Var(name), offset + consumed))
            }
            3 => {
                let (params, consumed1) = Vec::<Str>::from_ssz_bytes_with_length(&bytes[offset..])?;
                offset += consumed1;
                let (body, consumed2) = ExprBox::from_ssz_bytes_with_length(&bytes[offset..])?;
                Ok((Expr::Lambda(params, body), offset + consumed2))
            }
            4 => {
                let (func, consumed1) = ExprBox::from_ssz_bytes_with_length(&bytes[offset..])?;
                offset += consumed1;
                let (args, consumed2) = ExprVec::from_ssz_bytes_with_length(&bytes[offset..])?;
                Ok((Expr::Apply(func, args), offset + consumed2))
            }
            5 => {
                let (comb, consumed) = AtomicCombinator::from_ssz_bytes_with_length(&bytes[offset..])?;
                Ok((Expr::Combinator(comb), offset + consumed))
            }
            6 => {
                if bytes.len() < offset + 4 {
                    return Err(DecodeError {
                        message: "Insufficient bytes for Dynamic steps".to_string(),
                    });
                }
                let mut steps_bytes = [0u8; 4];
                steps_bytes.copy_from_slice(&bytes[offset..offset + 4]);
                let steps = u32::from_le_bytes(steps_bytes);
                offset += 4;
                let (expr, consumed) = ExprBox::from_ssz_bytes_with_length(&bytes[offset..])?;
                Ok((Expr::Dynamic(steps, expr), offset + consumed))
            }
            _ => Err(DecodeError {
                message: format!("Invalid Expr tag: {}", bytes[0]),
            }),
        }
    }
}

impl Decode for Expr {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (expr, _) = Self::from_ssz_bytes_with_length(bytes)?;
        Ok(expr)
    }
}

impl SimpleSerialize for Expr {}

//-----------------------------------------------------------------------------
// Domain-Aware AST Nodes (from ast_nodes.rs)
//-----------------------------------------------------------------------------

/// Domain-aware expression node that is content-addressable
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAwareExprNode {
    pub domain_id: DomainId,
    pub expr_id: ExprId,
    pub expression: ValueExpr,
    pub child_refs: Vec<NodeId>,
    pub metadata: HashMap<String, String>,
}

impl Encode for DomainAwareExprNode {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Domain ID (32 bytes)
        bytes.extend(self.domain_id.as_ssz_bytes());
        
        // Expression ID (32 bytes)
        bytes.extend(self.expr_id.as_ssz_bytes());
        
        // Expression data
        bytes.extend(self.expression.as_ssz_bytes());
        
        // Child references count + data
        let child_count = self.child_refs.len() as u32;
        bytes.extend(child_count.to_le_bytes());
        for child in &self.child_refs {
            bytes.extend(child.as_ssz_bytes());
        }
        
        // Metadata count + data
        let metadata_count = self.metadata.len() as u32;
        bytes.extend(metadata_count.to_le_bytes());
        for (key, value) in &self.metadata {
            let key_bytes = key.as_bytes();
            let value_bytes = value.as_bytes();
            bytes.extend((key_bytes.len() as u32).to_le_bytes());
            bytes.extend(key_bytes);
            bytes.extend((value_bytes.len() as u32).to_le_bytes());
            bytes.extend(value_bytes);
        }
        
        bytes
    }
}

impl Decode for DomainAwareExprNode {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 64 {
            return Err(DecodeError {
                message: "DomainAwareExprNode: insufficient bytes".to_string(),
            });
        }
        
        let mut offset = 0;
        
        // Domain ID
        let domain_id = DomainId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;
        
        // Expression ID
        let expr_id = ExprId::from_ssz_bytes(&bytes[offset..offset + 32])?;
        offset += 32;
        
        // Expression data (variable length)
        let expr_start = offset;
        let _expr_len = bytes.len() - offset; // Simplified for now
        
        // For now, use a simplified approach
        let expression = ValueExpr::from_ssz_bytes(&bytes[expr_start..])?;
        
        Ok(DomainAwareExprNode {
            domain_id,
            expr_id,
            expression,
            child_refs: Vec::new(), // Simplified for now
            metadata: HashMap::new(), // Simplified for now
        })
    }
}

/// Builder pattern for constructing domain-aware expression nodes
#[derive(Debug, Clone, Default)]
pub struct DomainAwareExprBuilder {
    domain_id: Option<DomainId>,
    expr_id: Option<ExprId>,
    expression: Option<ValueExpr>,
    child_refs: Vec<NodeId>,
    metadata: HashMap<String, String>,
}

impl DomainAwareExprBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn domain_id(mut self, domain_id: DomainId) -> Self {
        self.domain_id = Some(domain_id);
        self
    }
    
    pub fn expr_id(mut self, expr_id: ExprId) -> Self {
        self.expr_id = Some(expr_id);
        self
    }
    
    pub fn expression(mut self, expression: ValueExpr) -> Self {
        self.expression = Some(expression);
        self
    }
    
    pub fn add_child_ref(mut self, child_ref: NodeId) -> Self {
        self.child_refs.push(child_ref);
        self
    }
    
    pub fn add_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    pub fn build(self) -> Result<DomainAwareExprNode> {
        let domain_id = self.domain_id.ok_or_else(|| anyhow::anyhow!("Missing domain_id"))?;
        let expr_id = self.expr_id.ok_or_else(|| anyhow::anyhow!("Missing expr_id"))?;
        let expression = self.expression.ok_or_else(|| anyhow::anyhow!("Missing expression"))?;
        
        Ok(DomainAwareExprNode {
            domain_id,
            expr_id,
            expression,
            child_refs: self.child_refs,
            metadata: self.metadata,
        })
    }
}

/// Reference to a domain-aware expression node
#[derive(Debug, Clone, PartialEq)]
pub struct DomainAwareExprRef {
    pub domain_id: DomainId,
    pub content_id: NodeId,
    pub ssz_root: [u8; 32],
}

impl DomainAwareExprRef {
    pub fn new(node: &DomainAwareExprNode) -> Self {
        let ssz_bytes = node.as_ssz_bytes();
        let mut hasher = sha2::Sha256::new();
        hasher.update(&ssz_bytes);
        let ssz_root: [u8; 32] = hasher.finalize().into();
        
        // Create content ID from the hash
        let content_id = NodeId::new(ssz_root);
        
        Self {
            domain_id: node.domain_id,
            content_id,
            ssz_root,
        }
    }
    
    pub fn verify(&self, node: &DomainAwareExprNode) -> bool {
        if self.domain_id != node.domain_id {
            return false;
        }
        
        let ssz_bytes = node.as_ssz_bytes();
        let mut hasher = sha2::Sha256::new();
        hasher.update(&ssz_bytes);
        let computed_root: [u8; 32] = hasher.finalize().into();
        
        computed_root == self.ssz_root
    }
}

impl Encode for DomainAwareExprRef {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.domain_id.as_ssz_bytes());
        bytes.extend(self.content_id.as_ssz_bytes());
        bytes.extend(&self.ssz_root);
        bytes
    }
}

impl Decode for DomainAwareExprRef {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 96 {
            return Err(DecodeError {
                message: "DomainAwareExprRef: insufficient bytes".to_string(),
            });
        }
        
        let domain_id = DomainId::from_ssz_bytes(&bytes[0..32])?;
        let content_id = NodeId::from_ssz_bytes(&bytes[32..64])?;
        let mut ssz_root = [0u8; 32];
        ssz_root.copy_from_slice(&bytes[64..96]);
        
        Ok(Self {
            domain_id,
            content_id,
            ssz_root,
        })
    }
}

//-----------------------------------------------------------------------------
// Traits and Utility Implementations
//-----------------------------------------------------------------------------

/// Trait for converting types to expressions
pub trait AsExpr {
    /// Convert to an Expr
    fn to_expr(&self) -> Expr;
}

// Utility implementations for wrapper types
impl std::iter::FromIterator<Expr> for ExprVec {
    fn from_iter<I: IntoIterator<Item = Expr>>(iter: I) -> Self {
        ExprVec(iter.into_iter().collect())
    }
}

impl std::ops::Deref for ExprVec {
    type Target = Vec<Expr>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ExprVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ExprVec> for Vec<Expr> {
    fn from(vec: ExprVec) -> Self {
        vec.0
    }
}

impl From<Vec<Expr>> for ExprVec {
    fn from(vec: Vec<Expr>) -> Self {
        ExprVec(vec)
    }
}

impl std::ops::Deref for ExprBox {
    type Target = Box<Expr>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ExprBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ExprBox> for Box<Expr> {
    fn from(boxed: ExprBox) -> Self {
        boxed.0
    }
}

impl From<Box<Expr>> for ExprBox {
    fn from(boxed: Box<Expr>) -> Self {
        ExprBox(boxed)
    }
}

impl std::ops::Deref for ExprPairs {
    type Target = Vec<(Expr, Expr)>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ExprPairs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ExprPairs> for Vec<(Expr, Expr)> {
    fn from(pairs: ExprPairs) -> Self {
        pairs.0
    }
}

impl From<Vec<(Expr, Expr)>> for ExprPairs {
    fn from(vec: Vec<(Expr, Expr)>) -> Self {
        ExprPairs(vec)
    }
} 