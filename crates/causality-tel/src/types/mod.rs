//! The TEL Type System
//!
//! This module implements the static type system for the Temporal Effect Language (TEL).
//! The type system is based on a combination of:
//!
//! 1. **Simple Types**: Basic types like integers, strings, booleans, etc.
//! 2. **Row Types**: A system for extensible records with structural typing
//! 3. **Effect Types**: A system for tracking and handling effects in computations
//!
//! ## Type System Architecture
//!
//! The TEL type system is built around the concept of row polymorphism, which allows for:
//!
//! - **Extensible Records**: Records with a known set of fields and potentially unknown extensions
//! - **Effect Tracking**: Effects with a known set of operations and potentially unknown extensions
//!
//! The combination of these features enables powerful static typing while maintaining flexibility.
//!
//! ### Row Types
//!
//! Row types represent a collection of labeled fields with associated types. They support:
//!
//! - **Extension Variables**: Unknown parts of rows represented by extension variables
//! - **Structural Subtyping**: Based on field presence and compatibility
//! - **Operations**: Field access, field update, field removal, and row merging
//!
//! The row type system uses the concept of "lack" constraints to ensure type safety:
//! a field can only be added to a row if that field is not already present.
//!
//! ### Effect Types
//!
//! Effect types are built on top of row types, where each "field" represents an effect operation.
//! They support:
//!
//! - **Effect Operations**: Named operations with function types (parameter and result)
//! - **Effect Handlers**: Functions that handle specific effects
//! - **Effect Composition**: Combining multiple effect handlers
//! - **Effect Subtyping**: Based on effect operation presence and compatibility
//!
//! The effect system allows for tracking effects through the type system, ensuring that
//! all effects used in a computation are properly handled.
//!
//! ## Implementation Details
//!
//! The implementation follows these principles:
//!
//! 1. **Declarative Type System**: Types describe the shape and behavior of values
//! 2. **Structural Typing**: Types are compared based on their structure rather than names
//! 3. **Type Safety**: The type system prevents type errors at compile time
//! 4. **Type Inference**: Types can be inferred from context (in progress)
//!
//! This module provides the core types, type checking, and type operations needed
//! for the TEL language.

pub mod row;
pub mod effect;

use std::collections::{HashMap, BTreeMap};
use std::fmt;

use serde::{Serialize, Deserialize};
use causality_types::content_addressing;
use causality_types::crypto_primitives::ContentId;

// Re-export effect row and remove duplicate
pub use effect::EffectRow;

/// Represents a TEL type definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TelType {
    /// Base types
    Base(BaseType),
    
    /// Content-addressed type (ContentId<T>)
    ContentId(Box<TelType>),
    
    /// List type (List<T>)
    List(Box<TelType>),
    
    /// Map type (Map<K, V>)
    Map(Box<TelType>, Box<TelType>),
    
    /// Record type with fields
    Record(RecordType),
    
    /// Function type (T1 -> T2)
    Function(Box<TelType>, Box<TelType>),
    
    /// Effect type (Effect<E, R>)
    Effect(Box<TelType>, Box<TelType>),
    
    /// Resource type (Resource<R>)
    Resource(Box<TelType>),
    
    /// Domain type (Domain<D>)
    Domain(Box<TelType>),
    
    /// Handler type (Handler<E, R>)
    Handler(Box<TelType>, Box<TelType>),
    
    /// Type variable for type inference
    TypeVar(String),
    
    /// Row variable for row polymorphism
    RowVar(String),
    
    /// Unknown type (used during type inference)
    Unknown,
}

/// Base types for TEL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseType {
    /// Unit type (representing null/void)
    Unit,
    
    /// Boolean type
    Bool,
    
    /// 64-bit integer
    Int,
    
    /// 64-bit floating point
    Float,
    
    /// UTF-8 string
    String,
    
    /// Timestamp from causality-core
    Timestamp,
    
    /// Domain-specific amount type
    Amount,
    
    /// Dynamic/any type
    Any,
}

/// Record type with fields
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordType {
    /// Fields in the record, using BTreeMap for deterministic ordering and Hash implementation
    pub fields: BTreeMap<String, TelType>,
    
    /// Row variable (extension) for row polymorphism
    pub extension: Option<String>,
}

/// Represents a typed TEL value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TelValue {
    /// Unit value
    Unit,
    
    /// Boolean value
    Bool(bool),
    
    /// Integer value
    Int(i64),
    
    /// Float value
    Float(f64),
    
    /// String value
    String(String),
    
    /// List value
    List(Vec<TelValue>),
    
    /// Map value
    Map(HashMap<String, TelValue>),
    
    /// Record value
    Record(HashMap<String, TelValue>),
    
    /// Content ID value
    ContentId(ContentId),
    
    /// Resource value
    Resource {
        resource_type: String,
        quantity: i64,
        data: Box<TelValue>,
    },
    
    /// Effect value
    Effect {
        effect_name: String,
        args: Vec<TelValue>,
    },
}

impl TelValue {
    /// Convert a serde_json::Value to a TelValue
    pub fn from_json(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => TelValue::Unit,
            serde_json::Value::Bool(b) => TelValue::Bool(b),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    TelValue::Int(n.as_i64().unwrap())
                } else {
                    TelValue::Float(n.as_f64().unwrap_or(0.0))
                }
            },
            serde_json::Value::String(s) => TelValue::String(s),
            serde_json::Value::Array(arr) => {
                let items = arr.into_iter()
                    .map(|item| TelValue::from_json(item))
                    .collect();
                TelValue::List(items)
            },
            serde_json::Value::Object(obj) => {
                let map = obj.into_iter()
                    .map(|(k, v)| (k, TelValue::from_json(v)))
                    .collect();
                TelValue::Map(map)
            }
        }
    }
    
    /// Convert a TelValue to a serde_json::Value
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            TelValue::Unit => serde_json::Value::Null,
            TelValue::Bool(b) => serde_json::Value::Bool(*b),
            TelValue::Int(n) => serde_json::json!(*n),
            TelValue::Float(f) => serde_json::json!(*f),
            TelValue::String(s) => serde_json::Value::String(s.clone()),
            TelValue::List(items) => {
                serde_json::Value::Array(items.iter().map(|item| item.to_json()).collect())
            },
            TelValue::Map(map) => {
                let json_map = serde_json::Map::from_iter(
                    map.iter().map(|(k, v)| (k.clone(), v.to_json()))
                );
                serde_json::Value::Object(json_map)
            },
            TelValue::Record(fields) => {
                let json_map = serde_json::Map::from_iter(
                    fields.iter().map(|(k, v)| (k.clone(), v.to_json()))
                );
                serde_json::Value::Object(json_map)
            },
            TelValue::ContentId(cid) => serde_json::Value::String(cid.to_string()),
            TelValue::Resource { resource_type, quantity, data } => {
                let mut map = serde_json::Map::new();
                map.insert("resource_type".to_string(), serde_json::Value::String(resource_type.clone()));
                map.insert("quantity".to_string(), serde_json::json!(*quantity));
                map.insert("data".to_string(), data.to_json());
                serde_json::Value::Object(map)
            },
            TelValue::Effect { effect_name, args } => {
                let mut map = serde_json::Map::new();
                map.insert("effect_name".to_string(), serde_json::Value::String(effect_name.clone()));
                map.insert("args".to_string(), serde_json::Value::Array(
                    args.iter().map(|arg| arg.to_json()).collect()
                ));
                serde_json::Value::Object(map)
            },
        }
    }

    /// Convert a Literal to a TelValue
    pub fn from_literal(lit: &crate::combinators::Literal) -> Self {
        use crate::combinators::Literal;
        match lit {
            Literal::Null => TelValue::Unit,
            Literal::Bool(b) => TelValue::Bool(*b),
            Literal::Int(n) => TelValue::Int(*n),
            Literal::Float(f) => TelValue::Float(*f),
            Literal::String(s) => TelValue::String(s.clone()),
            Literal::List(items) => {
                let tel_items = items.iter()
                    .map(|item| TelValue::from_literal(item))
                    .collect();
                TelValue::List(tel_items)
            },
            Literal::Map(map) => {
                let tel_map = map.iter()
                    .map(|(k, v)| (k.clone(), TelValue::from_literal(v)))
                    .collect();
                TelValue::Map(tel_map)
            },
        }
    }
    
    /// Convert a TelValue to a Literal
    pub fn to_literal(&self) -> crate::combinators::Literal {
        use crate::combinators::Literal;
        match self {
            TelValue::Unit => Literal::Null,
            TelValue::Bool(b) => Literal::Bool(*b),
            TelValue::Int(n) => Literal::Int(*n),
            TelValue::Float(f) => Literal::Float(*f),
            TelValue::String(s) => Literal::String(s.clone()),
            TelValue::List(items) => {
                let lit_items = items.iter()
                    .map(|item| item.to_literal())
                    .collect();
                Literal::List(lit_items)
            },
            TelValue::Map(map) => {
                let lit_map = map.iter()
                    .map(|(k, v)| (k.clone(), v.to_literal()))
                    .collect();
                Literal::Map(lit_map)
            },
            TelValue::Record(fields) => {
                // Convert record to map for literal representation
                let lit_map = fields.iter()
                    .map(|(k, v)| (k.clone(), v.to_literal()))
                    .collect();
                Literal::Map(lit_map)
            },
            TelValue::ContentId(cid) => Literal::String(cid.to_string()),
            TelValue::Resource { resource_type, quantity, data } => {
                // Convert resource to map representation
                let mut map = HashMap::new();
                map.insert("resource_type".to_string(), Literal::String(resource_type.clone()));
                map.insert("quantity".to_string(), Literal::Int(*quantity));
                map.insert("data".to_string(), data.to_literal());
                Literal::Map(map)
            },
            TelValue::Effect { effect_name, args } => {
                // Convert effect to map representation
                let mut map = HashMap::new();
                map.insert("effect_name".to_string(), Literal::String(effect_name.clone()));
                let args_lit = args.iter().map(|arg| arg.to_literal()).collect();
                map.insert("args".to_string(), Literal::List(args_lit));
                Literal::Map(map)
            },
        }
    }
}

/// Type environment for type checking
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// Type bindings for variables
    pub bindings: HashMap<String, TelType>,
    
    /// Constraints on type variables
    pub constraints: Vec<TypeConstraint>,
}

/// Type constraint for type checking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeConstraint {
    /// Type equality constraint (T1 = T2)
    Equals(TelType, TelType),
    
    /// Subtype constraint (T1 <: T2)
    Subtype(TelType, TelType),
    
    /// Lacks constraint (r lacks field) for row polymorphism
    Lacks(String, String),
    
    /// Row disjointness constraint (r1 # r2) for row merging
    Disjoint(String, String),
}

impl TelType {
    /// Get the base type for a simple type name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Unit" => Some(TelType::Base(BaseType::Unit)),
            "Bool" => Some(TelType::Base(BaseType::Bool)),
            "Int" => Some(TelType::Base(BaseType::Int)),
            "Float" => Some(TelType::Base(BaseType::Float)),
            "String" => Some(TelType::Base(BaseType::String)),
            "Timestamp" => Some(TelType::Base(BaseType::Timestamp)),
            "Amount" => Some(TelType::Base(BaseType::Amount)),
            "Any" => Some(TelType::Base(BaseType::Any)),
            _ => None,
        }
    }
    
    /// Create a content ID type
    pub fn content_id(inner: TelType) -> Self {
        TelType::ContentId(Box::new(inner))
    }
    
    /// Create a list type
    pub fn list(inner: TelType) -> Self {
        TelType::List(Box::new(inner))
    }
    
    /// Create a map type
    pub fn map(key: TelType, value: TelType) -> Self {
        TelType::Map(Box::new(key), Box::new(value))
    }
    
    /// Create a function type
    pub fn function(param: TelType, result: TelType) -> Self {
        TelType::Function(Box::new(param), Box::new(result))
    }
    
    /// Create an effect type
    pub fn effect(effect: TelType, result: TelType) -> Self {
        TelType::Effect(Box::new(effect), Box::new(result))
    }
    
    /// Create a resource type
    pub fn resource(inner: TelType) -> Self {
        TelType::Resource(Box::new(inner))
    }
    
    /// Create a domain type
    pub fn domain(inner: TelType) -> Self {
        TelType::Domain(Box::new(inner))
    }
    
    /// Create a handler type
    pub fn handler(effect: TelType, result: TelType) -> Self {
        TelType::Handler(Box::new(effect), Box::new(result))
    }
    
    /// Create a type variable
    pub fn type_var(name: impl Into<String>) -> Self {
        TelType::TypeVar(name.into())
    }
    
    /// Create a row variable
    pub fn row_var(name: impl Into<String>) -> Self {
        TelType::RowVar(name.into())
    }
    
    /// Create a record type
    pub fn record(fields: BTreeMap<String, TelType>, extension: Option<String>) -> Self {
        TelType::Record(RecordType { fields, extension })
    }
    
    /// Check if this type is a subtype of another
    pub fn is_subtype(&self, other: &TelType) -> bool {
        match (self, other) {
            // Same types are subtypes
            (a, b) if a == b => true,
            
            // Record subtyping (structural)
            (TelType::Record(a), TelType::Record(b)) => {
                // All fields in b must be in a with compatible types
                for (field, b_type) in &b.fields {
                    if let Some(a_type) = a.fields.get(field) {
                        if !a_type.is_subtype(b_type) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                
                // Check row extension
                match (&a.extension, &b.extension) {
                    (Some(_), Some(_)) => true, // Both have extensions, assume compatible
                    (Some(_), None) => true,    // a has extension, b doesn't
                    (None, Some(_)) => false,   // b has extension, a doesn't
                    (None, None) => true,       // Neither has extension
                }
            },
            
            // Function subtyping is contravariant in params, covariant in result
            (TelType::Function(a_param, a_result), TelType::Function(b_param, b_result)) => {
                b_param.is_subtype(a_param) && a_result.is_subtype(b_result)
            },
            
            // Effect subtyping is invariant in effect row, covariant in result
            (TelType::Effect(a_effect, a_result), TelType::Effect(b_effect, b_result)) => {
                a_effect == b_effect && a_result.is_subtype(b_result)
            },
            
            // ContentId subtyping follows inner type
            (TelType::ContentId(a_inner), TelType::ContentId(b_inner)) => {
                a_inner.is_subtype(b_inner)
            },
            
            // List subtyping follows element type
            (TelType::List(a_elem), TelType::List(b_elem)) => {
                a_elem.is_subtype(b_elem)
            },
            
            // Map subtyping is invariant in key, covariant in value
            (TelType::Map(a_key, a_val), TelType::Map(b_key, b_val)) => {
                a_key == b_key && a_val.is_subtype(b_val)
            },
            
            // Resource subtyping follows inner type
            (TelType::Resource(a_inner), TelType::Resource(b_inner)) => {
                a_inner.is_subtype(b_inner)
            },
            
            // Unknown is a subtype of everything (for type inference)
            (TelType::Unknown, _) => true,
            
            // Type variables subtyping would depend on constraints
            (TelType::TypeVar(_), _) => false,
            
            // Everything else is not a subtype
            _ => false,
        }
    }
}

impl fmt::Display for TelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelType::Base(base) => write!(f, "{}", base),
            TelType::ContentId(inner) => write!(f, "ContentId<{}>", inner),
            TelType::List(inner) => write!(f, "List<{}>", inner),
            TelType::Map(key, val) => write!(f, "Map<{}, {}>", key, val),
            TelType::Record(record) => {
                write!(f, "{{ ")?;
                let mut first = true;
                for (field, ty) in &record.fields {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field, ty)?;
                    first = false;
                }
                if let Some(ext) = &record.extension {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "| {}", ext)?;
                }
                write!(f, " }}")
            },
            TelType::Function(param, result) => write!(f, "{} -> {}", param, result),
            TelType::Effect(effect, result) => write!(f, "Effect<{}, {}>", effect, result),
            TelType::Resource(inner) => write!(f, "Resource<{}>", inner),
            TelType::Domain(inner) => write!(f, "Domain<{}>", inner),
            TelType::Handler(effect, result) => write!(f, "Handler<{}, {}>", effect, result),
            TelType::TypeVar(name) => write!(f, "{}", name),
            TelType::RowVar(name) => write!(f, "{}", name),
            TelType::Unknown => write!(f, "?"),
        }
    }
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseType::Unit => write!(f, "Unit"),
            BaseType::Bool => write!(f, "Bool"),
            BaseType::Int => write!(f, "Int"),
            BaseType::Float => write!(f, "Float"),
            BaseType::String => write!(f, "String"),
            BaseType::Timestamp => write!(f, "Timestamp"),
            BaseType::Amount => write!(f, "Amount"),
            BaseType::Any => write!(f, "Any"),
        }
    }
}

impl TypeEnvironment {
    /// Create a new empty type environment
    pub fn new() -> Self {
        TypeEnvironment {
            bindings: HashMap::new(),
            constraints: Vec::new(),
        }
    }
    
    /// Add a type binding
    pub fn add_binding(&mut self, name: impl Into<String>, ty: TelType) {
        self.bindings.insert(name.into(), ty);
    }
    
    /// Get the type of a variable
    pub fn get_type(&self, name: &str) -> Option<&TelType> {
        self.bindings.get(name)
    }
    
    /// Add a type constraint
    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }
    
    /// Check if a type satisfies the constraints
    pub fn satisfies_constraints(&self, ty: &TelType) -> bool {
        for constraint in &self.constraints {
            match constraint {
                TypeConstraint::Equals(a, b) => {
                    if a == ty && *b != *ty {
                        return false;
                    }
                    if b == ty && *a != *ty {
                        return false;
                    }
                },
                TypeConstraint::Subtype(a, b) => {
                    if a == ty && !ty.is_subtype(b) {
                        return false;
                    }
                    if b == ty && !a.is_subtype(ty) {
                        return false;
                    }
                },
                // Lacks and Disjoint constraints apply to row variables
                _ => {}
            }
        }
        true
    }
}

/// Type rules for the core combinators
pub mod type_rules {
    use super::*;
    use crate::combinators::Combinator;
    
    /// Initialize the type environment with combinator types
    pub fn init_combinator_types() -> TypeEnvironment {
        let mut env = TypeEnvironment::new();
        
        // I : a -> a
        let a = TelType::type_var("a");
        env.add_binding("I", TelType::function(a.clone(), a));
        
        // K : a -> b -> a
        let a = TelType::type_var("a");
        let b = TelType::type_var("b");
        env.add_binding("K", TelType::function(
            a.clone(),
            TelType::function(b, a.clone())
        ));
        
        // S : (a -> b -> c) -> (a -> b) -> a -> c
        let a = TelType::type_var("a");
        let b = TelType::type_var("b");
        let c = TelType::type_var("c");
        env.add_binding("S", TelType::function(
            TelType::function(a.clone(), TelType::function(b.clone(), c.clone())),
            TelType::function(
                TelType::function(a.clone(), b.clone()),
                TelType::function(a.clone(), c.clone())
            )
        ));
        
        // B : (b -> c) -> (a -> b) -> a -> c
        let a = TelType::type_var("a");
        let b = TelType::type_var("b");
        let c = TelType::type_var("c");
        env.add_binding("B", TelType::function(
            TelType::function(b.clone(), c.clone()),
            TelType::function(
                TelType::function(a.clone(), b.clone()),
                TelType::function(a.clone(), c.clone())
            )
        ));
        
        // C : (a -> b -> c) -> b -> a -> c
        let a = TelType::type_var("a");
        let b = TelType::type_var("b");
        let c = TelType::type_var("c");
        env.add_binding("C", TelType::function(
            TelType::function(a.clone(), TelType::function(b.clone(), c.clone())),
            TelType::function(
                b.clone(),
                TelType::function(a.clone(), c.clone())
            )
        ));
        
        env
    }
    
    /// Infer the type of a combinator expression
    pub fn infer_type(expr: &Combinator, env: &TypeEnvironment) -> Result<TelType, String> {
        match expr {
            // Base combinators have fixed types
            Combinator::I => Ok(env.get_type("I").unwrap().clone()),
            Combinator::K => Ok(env.get_type("K").unwrap().clone()),
            Combinator::S => Ok(env.get_type("S").unwrap().clone()),
            Combinator::B => Ok(env.get_type("B").unwrap().clone()),
            Combinator::C => Ok(env.get_type("C").unwrap().clone()),
            
            // Application type is the result type of the function
            Combinator::App { function, argument } => {
                let f_type = infer_type(function, env)?;
                let x_type = infer_type(argument, env)?;
                
                match f_type {
                    TelType::Function(param_type, result_type) => {
                        // Check if argument type is compatible with parameter type
                        if x_type.is_subtype(&param_type) {
                            Ok(*result_type)
                        } else {
                            Err(format!("Type mismatch: expected {}, got {}", param_type, x_type))
                        }
                    },
                    _ => Err(format!("Expected function type, got {}", f_type)),
                }
            },
            
            // Literal types
            Combinator::Literal(literal) => {
                match literal {
                    crate::combinators::Literal::Int(_) => Ok(TelType::Base(BaseType::Int)),
                    crate::combinators::Literal::Float(_) => Ok(TelType::Base(BaseType::Float)),
                    crate::combinators::Literal::String(_) => Ok(TelType::Base(BaseType::String)),
                    crate::combinators::Literal::Bool(_) => Ok(TelType::Base(BaseType::Bool)),
                    crate::combinators::Literal::Null => Ok(TelType::Base(BaseType::Unit)),
                    crate::combinators::Literal::List(items) => {
                        // Infer element type from the first item
                        if let Some(first) = items.first() {
                            let first_type = infer_literal_type(first)?;
                            Ok(TelType::list(first_type))
                        } else {
                            // Empty list - use unknown element type
                            Ok(TelType::list(TelType::Unknown))
                        }
                    },
                    crate::combinators::Literal::Map(entries) => {
                        // Assume string keys for maps
                        if let Some((_, first_val)) = entries.iter().next() {
                            let val_type = infer_literal_type(first_val)?;
                            Ok(TelType::map(TelType::Base(BaseType::String), val_type))
                        } else {
                            // Empty map - use unknown value type
                            Ok(TelType::map(TelType::Base(BaseType::String), TelType::Unknown))
                        }
                    },
                }
            },
            
            // Variable reference type comes from environment
            Combinator::Ref(name) => {
                if let Some(ty) = env.get_type(name) {
                    Ok(ty.clone())
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            },
            
            // Effect combinator type
            Combinator::Effect { effect_name, args, core_effect } => {
                // Each effect would have a specific type signature
                // For now, we'll use a generic effect type
                Ok(TelType::effect(
                    TelType::type_var("e"),
                    TelType::type_var("r")
                ))
            },
            
            // State transition type
            Combinator::StateTransition { target_state, fields, resource_id } => {
                // State transitions return a ContentId for the new state
                Ok(TelType::content_id(TelType::Record(RecordType {
                    fields: BTreeMap::new(),
                    extension: None,
                })))
            },
            
            // Content addressing operations
            Combinator::ContentId(expr) => {
                let inner_type = infer_type(expr, env)?;
                Ok(TelType::content_id(inner_type))
            },
            
            Combinator::Store(expr) => {
                let inner_type = infer_type(expr, env)?;
                Ok(TelType::content_id(inner_type))
            },
            
            Combinator::Load(expr) => {
                match infer_type(expr, env)? {
                    TelType::ContentId(inner_type) => Ok(*inner_type),
                    _ => Err("Load requires a ContentId argument".to_string()),
                }
            },
            
            // Don't know how to infer type for other combinators yet
            _ => Err(format!("Cannot infer type for combinator: {:?}", expr)),
        }
    }
    
    /// Infer the type of a literal value
    fn infer_literal_type(literal: &crate::combinators::Literal) -> Result<TelType, String> {
        match literal {
            crate::combinators::Literal::Int(_) => Ok(TelType::Base(BaseType::Int)),
            crate::combinators::Literal::Float(_) => Ok(TelType::Base(BaseType::Float)),
            crate::combinators::Literal::String(_) => Ok(TelType::Base(BaseType::String)),
            crate::combinators::Literal::Bool(_) => Ok(TelType::Base(BaseType::Bool)),
            crate::combinators::Literal::Null => Ok(TelType::Base(BaseType::Unit)),
            crate::combinators::Literal::List(items) => {
                // Infer element type from the first item
                if let Some(first) = items.first() {
                    let first_type = infer_literal_type(first)?;
                    Ok(TelType::list(first_type))
                } else {
                    // Empty list - use unknown element type
                    Ok(TelType::list(TelType::Unknown))
                }
            },
            crate::combinators::Literal::Map(entries) => {
                // Assume string keys for maps
                if let Some((_, first_val)) = entries.iter().next() {
                    let val_type = infer_literal_type(first_val)?;
                    Ok(TelType::map(TelType::Base(BaseType::String), val_type))
                } else {
                    // Empty map - use unknown value type
                    Ok(TelType::map(TelType::Base(BaseType::String), TelType::Unknown))
                }
            },
        }
    }
}

impl From<BaseType> for TelType {
    fn from(base_type: BaseType) -> Self {
        TelType::Base(base_type)
    }
}

#[cfg(test)]
pub mod tests; 