//! Value system for Causality Lisp runtime
//!
//! This module provides the runtime value representation for Lisp expressions,
//! including support for linear types and resource management.

use crate::{ast::{Expr, Param}, error::{LispError, EvalError}};
use causality_core::{
    system::content_addressing::{EntityId, Str},
    lambda::Symbol,
};
use std::collections::HashMap;
use std::rc::Rc;

/// Runtime value in Causality Lisp
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub kind: ValueKind,
    pub type_info: TypeInfo,
    pub linearity: LinearityInfo,
}

/// Value kinds supported in Causality Lisp
#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    /// Nil value
    Nil,
    
    /// Boolean values
    Bool(bool),
    
    /// Integer values
    Int(i64),
    
    /// Floating point values
    Float(f64),
    
    /// String values (ZK-compatible bounded strings)
    String(Str),
    
    /// Symbol values (identifiers) (ZK-compatible symbols)
    Symbol(Symbol),
    
    /// List values
    List(Vec<Value>),
    
    /// Function values
    Function {
        params: Vec<Symbol>,
        body: crate::ast::Expr,
        closure: Environment,
    },
    
    /// Built-in function
    Builtin {
        name: Symbol,
        arity: Arity,
        func: BuiltinFunc,
    },
    
    /// Resource values with linear tracking
    Resource {
        id: Str,
        resource_type: Symbol,
        consumed: bool,
    },
    
    /// Effect values
    Effect {
        effect_type: Symbol,
        data: Box<Value>,
    },
    
    /// Lambda/function closure
    Lambda {
        params: Vec<Param>,
        body: Expr,
    },
    
    /// Quoted expression
    Quoted(Expr),
    
    /// Tensor (pair) value
    Tensor(Box<Value>, Box<Value>),
    
    /// Sum value with tag
    Sum {
        tag: u8,
        value: Box<Value>,
    },
    
    /// Record value
    Record(HashMap<Symbol, Value>),
}

/// Function arity specification
#[derive(Debug, Clone, PartialEq)]
pub enum Arity {
    /// Exact number of arguments
    Exact(usize),
    /// Minimum number of arguments
    AtLeast(usize),
    /// Variable arguments
    Variadic,
}

/// Built-in function type
#[derive(Clone)]
pub struct BuiltinFunc {
    pub func: BuiltinFunction,
}

impl std::fmt::Debug for BuiltinFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BuiltinFunc {{ func: <function> }}")
    }
}

impl PartialEq for BuiltinFunc {
    fn eq(&self, other: &Self) -> bool {
        // Compare by pointer address for built-ins
        Rc::ptr_eq(&self.func, &other.func)
    }
}

/// Type information for values
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub type_name: String,
    pub constraints: Vec<String>,
}

/// Linear type tracking information
#[derive(Debug, Clone, PartialEq)]
pub struct LinearityInfo {
    pub is_linear: bool,
    pub is_consumed: bool,
    pub ownership: Ownership,
}

/// Ownership tracking for linear types
#[derive(Debug, Clone, PartialEq)]
pub enum Ownership {
    Owned,
    Borrowed,
    Moved,
}

/// Environment for variable bindings
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub bindings: HashMap<Symbol, Value>,
    pub parent: Option<Box<Environment>>,
}

impl Default for LinearityInfo {
    fn default() -> Self {
        Self {
            is_linear: false,
            is_consumed: false,
            ownership: Ownership::Owned,
        }
    }
}

impl Value {
    /// Create a nil value
    pub fn nil() -> Self {
        Self {
            kind: ValueKind::Nil,
            type_info: TypeInfo {
                type_name: "Nil".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a unit value (alias for nil)
    pub fn unit() -> Self {
        Self::nil()
    }
    
    /// Create a boolean value
    pub fn bool(b: bool) -> Self {
        Self {
            kind: ValueKind::Bool(b),
            type_info: TypeInfo {
                type_name: "Bool".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create an integer value
    pub fn int(i: i64) -> Self {
        Self {
            kind: ValueKind::Int(i),
            type_info: TypeInfo {
                type_name: "Int".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a floating-point value
    pub fn float(f: f64) -> Self {
        Self {
            kind: ValueKind::Float(f),
            type_info: TypeInfo {
                type_name: "Float".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a string value
    pub fn string(s: impl Into<Str>) -> Self {
        Self {
            kind: ValueKind::String(s.into()),
            type_info: TypeInfo {
                type_name: "String".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a symbol value
    pub fn symbol(s: impl Into<Symbol>) -> Self {
        Self {
            kind: ValueKind::Symbol(s.into()),
            type_info: TypeInfo {
                type_name: "Symbol".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a list value
    pub fn list(items: Vec<Value>) -> Self {
        Self {
            kind: ValueKind::List(items),
            type_info: TypeInfo {
                type_name: "List".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a resource value
    pub fn resource(id: impl Into<Str>, resource_type: impl Into<Symbol>) -> Self {
        Self {
            kind: ValueKind::Resource {
                id: id.into(),
                resource_type: resource_type.into(),
                consumed: false,
            },
            type_info: TypeInfo {
                type_name: "Resource".to_string(),
                constraints: vec!["Linear".to_string()],
            },
            linearity: LinearityInfo {
                is_linear: true,
                is_consumed: false,
                ownership: Ownership::Owned,
            },
        }
    }
    
    /// Create a quoted expression value
    pub fn quoted(expr: Expr) -> Self {
        Self {
            kind: ValueKind::Quoted(expr),
            type_info: TypeInfo {
                type_name: "Quoted".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a tensor (pair) value
    pub fn tensor(left: Value, right: Value) -> Self {
        Self {
            kind: ValueKind::Tensor(Box::new(left), Box::new(right)),
            type_info: TypeInfo {
                type_name: "Tensor".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a left sum value
    pub fn sum_left(value: Value) -> Self {
        Self {
            kind: ValueKind::Sum {
                tag: 0,
                value: Box::new(value),
            },
            type_info: TypeInfo {
                type_name: "Sum".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a right sum value
    pub fn sum_right(value: Value) -> Self {
        Self {
            kind: ValueKind::Sum {
                tag: 1,
                value: Box::new(value),
            },
            type_info: TypeInfo {
                type_name: "Sum".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a record value
    pub fn record(fields: HashMap<Symbol, Value>) -> Self {
        Self {
            kind: ValueKind::Record(fields),
            type_info: TypeInfo {
                type_name: "Record".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Destructure a tensor into its left and right components
    pub fn destructure_tensor(&self) -> Result<(Value, Value), LispError> {
        match &self.kind {
            ValueKind::Tensor(left, right) => Ok((left.as_ref().clone(), right.as_ref().clone())),
            _ => Err(LispError::Eval(EvalError::TypeMismatch {
                expected: "Tensor".to_string(),
                found: self.type_name().to_string(),
            })),
        }
    }
    
    /// Destructure a sum value
    pub fn destructure_sum(&self) -> Result<(u8, Value), LispError> {
        match &self.kind {
            ValueKind::Sum { tag, value } => Ok((*tag, value.as_ref().clone())),
            _ => Err(LispError::Eval(EvalError::TypeMismatch {
                expected: "Sum".to_string(),
                found: self.type_name().to_string(),
            })),
        }
    }
    
    /// Extract resource ID if this is a resource value
    pub fn as_resource_id(&self) -> Option<EntityId> {
        match &self.kind {
            ValueKind::Resource { id, .. } => {
                // Try to parse the string ID as hex
                EntityId::from_hex(id.as_str()).ok()
            }
            _ => None,
        }
    }
    
    /// Check if this value is truthy
    pub fn is_truthy(&self) -> bool {
        match &self.kind {
            ValueKind::Nil => false,
            ValueKind::Bool(b) => *b,
            ValueKind::Int(i) => *i != 0,
            ValueKind::Float(f) => *f != 0.0,
            ValueKind::String(s) => !s.as_str().is_empty(),
            ValueKind::Symbol(_) => true,
            ValueKind::List(items) => !items.is_empty(),
            ValueKind::Function { .. } => true,
            ValueKind::Builtin { .. } => true,
            ValueKind::Resource { .. } => true,
            ValueKind::Effect { .. } => true,
            ValueKind::Lambda { .. } => true,
            ValueKind::Quoted(_) => true,
            ValueKind::Tensor(_, _) => true,
            ValueKind::Sum { .. } => true,
            ValueKind::Record(fields) => !fields.is_empty(),
        }
    }
    
    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match &self.kind {
            ValueKind::Nil => "Nil",
            ValueKind::Bool(_) => "Bool",
            ValueKind::Int(_) => "Int",
            ValueKind::Float(_) => "Float",
            ValueKind::String(_) => "String",
            ValueKind::Symbol(_) => "Symbol",
            ValueKind::List(_) => "List",
            ValueKind::Function { .. } => "Function",
            ValueKind::Builtin { .. } => "Builtin",
            ValueKind::Resource { .. } => "Resource",
            ValueKind::Effect { .. } => "Effect",
            ValueKind::Lambda { .. } => "Lambda",
            ValueKind::Quoted(_) => "Quoted",
            ValueKind::Tensor(_, _) => "Tensor",
            ValueKind::Sum { .. } => "Sum",
            ValueKind::Record(_) => "Record",
        }
    }
    
    /// Mark this value as consumed (for linear types)
    pub fn consume(&mut self) -> Result<(), crate::error::EvalError> {
        if self.linearity.is_linear {
            if self.linearity.is_consumed {
                return Err(crate::error::EvalError::LinearityViolation(
                    "Value already consumed".to_string()
                ));
            }
            self.linearity.is_consumed = true;
            
            // Mark resource as consumed
            if let ValueKind::Resource { consumed, .. } = &mut self.kind {
                *consumed = true;
            }
        }
        Ok(())
    }
    
    /// Create a lambda value
    pub fn lambda(params: Vec<Param>, body: Expr) -> Self {
        Self {
            kind: ValueKind::Lambda { params, body },
            type_info: TypeInfo {
                type_name: "Lambda".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a built-in function value
    pub fn builtin(name: impl Into<Symbol>, arity: i32) -> Self {
        let name_symbol = name.into();
        let name_str = name_symbol.to_string();
        let func = create_builtin_function(&name_str);
        
        Self {
            kind: ValueKind::Builtin {
                name: name_symbol,
                arity: if arity < 0 { Arity::Variadic } else { Arity::Exact(arity as usize) },
                func,
            },
            type_info: TypeInfo {
                type_name: "Builtin".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
    
    /// Create a sum value with tag and value
    pub fn sum(tag: u8, value: Value) -> Self {
        Self {
            kind: ValueKind::Sum {
                tag,
                value: Box::new(value),
            },
            type_info: TypeInfo {
                type_name: "Sum".to_string(),
                constraints: vec![],
            },
            linearity: LinearityInfo::default(),
        }
    }
}

/// Create a builtin function implementation
fn create_builtin_function(name: &str) -> BuiltinFunc {
    let func: BuiltinFunction = match name {
        "add" => Rc::new(|args| {
            if args.is_empty() {
                return Ok(Value::int(0));
            }
            
            let mut result = match &args[0].kind {
                ValueKind::Int(i) => *i as f64,
                ValueKind::Float(f) => *f,
                _ => return Err(crate::error::EvalError::TypeMismatch {
                    expected: "number".to_string(),
                    found: "other".to_string(),
                }),
            };
            
            let mut is_float = matches!(&args[0].kind, ValueKind::Float(_));
            
            for arg in &args[1..] {
                match &arg.kind {
                    ValueKind::Int(i) => result += *i as f64,
                    ValueKind::Float(f) => {
                        result += f;
                        is_float = true;
                    }
                    _ => return Err(crate::error::EvalError::TypeMismatch {
                        expected: "number".to_string(),
                        found: "other".to_string(),
                    }),
                }
            }
            
            if is_float {
                Ok(Value::float(result))
            } else {
                Ok(Value::int(result as i64))
            }
        }),
        "subtract" => Rc::new(|args| {
            if args.len() < 2 {
                return Err(crate::error::EvalError::InvalidCall("subtract requires at least 2 arguments".to_string()));
            }
            
            let mut result = match &args[0].kind {
                ValueKind::Int(i) => *i as f64,
                ValueKind::Float(f) => *f,
                _ => return Err(crate::error::EvalError::TypeMismatch {
                    expected: "number".to_string(),
                    found: "other".to_string(),
                }),
            };
            
            let mut is_float = matches!(&args[0].kind, ValueKind::Float(_));
            
            for arg in &args[1..] {
                match &arg.kind {
                    ValueKind::Int(i) => result -= *i as f64,
                    ValueKind::Float(f) => {
                        result -= f;
                        is_float = true;
                    }
                    _ => return Err(crate::error::EvalError::TypeMismatch {
                        expected: "number".to_string(),
                        found: "other".to_string(),
                    }),
                }
            }
            
            if is_float {
                Ok(Value::float(result))
            } else {
                Ok(Value::int(result as i64))
            }
        }),
        "multiply" => Rc::new(|args| {
            if args.is_empty() {
                return Ok(Value::int(1));
            }
            
            let mut result = match &args[0].kind {
                ValueKind::Int(i) => *i as f64,
                ValueKind::Float(f) => *f,
                _ => return Err(crate::error::EvalError::TypeMismatch {
                    expected: "number".to_string(),
                    found: "other".to_string(),
                }),
            };
            
            let mut is_float = matches!(&args[0].kind, ValueKind::Float(_));
            
            for arg in &args[1..] {
                match &arg.kind {
                    ValueKind::Int(i) => result *= *i as f64,
                    ValueKind::Float(f) => {
                        result *= f;
                        is_float = true;
                    }
                    _ => return Err(crate::error::EvalError::TypeMismatch {
                        expected: "number".to_string(),
                        found: "other".to_string(),
                    }),
                }
            }
            
            if is_float {
                Ok(Value::float(result))
            } else {
                Ok(Value::int(result as i64))
            }
        }),
        "divide" => Rc::new(|args| {
            if args.len() != 2 {
                return Err(crate::error::EvalError::InvalidCall("divide requires 2 arguments".to_string()));
            }
            match (&args[0].kind, &args[1].kind) {
                (ValueKind::Int(a), ValueKind::Int(b)) => {
                    if *b == 0 {
                        Err(crate::error::EvalError::DivisionByZero)
                    } else {
                        Ok(Value::int(a / b))
                    }
                }
                (ValueKind::Float(a), ValueKind::Float(b)) => {
                    if *b == 0.0 {
                        Err(crate::error::EvalError::DivisionByZero)
                    } else {
                        Ok(Value::float(a / b))
                    }
                }
                (ValueKind::Int(a), ValueKind::Float(b)) => {
                    if *b == 0.0 {
                        Err(crate::error::EvalError::DivisionByZero)
                    } else {
                        Ok(Value::float(*a as f64 / b))
                    }
                }
                (ValueKind::Float(a), ValueKind::Int(b)) => {
                    if *b == 0 {
                        Err(crate::error::EvalError::DivisionByZero)
                    } else {
                        Ok(Value::float(a / *b as f64))
                    }
                }
                _ => Err(crate::error::EvalError::TypeMismatch {
                    expected: "number".to_string(),
                    found: "other".to_string(),
                }),
            }
        }),
        "equals" => Rc::new(|args| {
            if args.len() != 2 {
                return Err(crate::error::EvalError::InvalidCall("equals requires 2 arguments".to_string()));
            }
            Ok(Value::bool(args[0] == args[1]))
        }),
        "car" => Rc::new(|args| {
            if args.len() != 1 {
                return Err(crate::error::EvalError::InvalidCall("car requires 1 argument".to_string()));
            }
            match &args[0].kind {
                ValueKind::List(items) => {
                    if items.is_empty() {
                        Ok(Value::nil())
                    } else {
                        Ok(items[0].clone())
                    }
                }
                _ => Err(crate::error::EvalError::TypeMismatch {
                    expected: "list".to_string(),
                    found: "other".to_string(),
                }),
            }
        }),
        "cdr" => Rc::new(|args| {
            if args.len() != 1 {
                return Err(crate::error::EvalError::InvalidCall("cdr requires 1 argument".to_string()));
            }
            match &args[0].kind {
                ValueKind::List(items) => {
                    if items.is_empty() {
                        Ok(Value::nil())
                    } else {
                        Ok(Value::list(items[1..].to_vec()))
                    }
                }
                _ => Err(crate::error::EvalError::TypeMismatch {
                    expected: "list".to_string(),
                    found: "other".to_string(),
                }),
            }
        }),
        "list" => Rc::new(|args| {
            Ok(Value::list(args.to_vec()))
        }),
        _ => Rc::new(|_| {
            Err(crate::error::EvalError::InvalidCall("Unknown builtin function".to_string()))
        }),
    };
    
    BuiltinFunc { func }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
    
    /// Create a new environment with a parent
    pub fn with_parent(parent: Environment) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    /// Bind a variable to a value
    pub fn bind(&mut self, name: Symbol, value: Value) {
        self.bindings.insert(name, value);
    }
    
    /// Look up a variable
    pub fn lookup(&self, name: &Symbol) -> Option<&Value> {
        self.bindings.get(name).or_else(|| {
            self.parent.as_ref().and_then(|parent| parent.lookup(name))
        })
    }
    
    /// Look up a variable mutably
    pub fn lookup_mut(&mut self, name: &Symbol) -> Option<&mut Value> {
        if self.bindings.contains_key(name) {
            self.bindings.get_mut(name)
        } else {
            self.parent.as_mut().and_then(|parent| parent.lookup_mut(name))
        }
    }
}

/// Type alias for built-in function signatures to reduce complexity
pub type BuiltinFunction = Rc<dyn Fn(&[Value]) -> Result<Value, EvalError>>;

/// A callable value with parameter info
pub struct CallableValue {
    /// Function implementation
    pub func: BuiltinFunction,
} 