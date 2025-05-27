// Purpose: Expression utility functions for creating and manipulating Expr, ValueExpr, and TypeExpr instances.

use crate::extension_traits::{ExprExt, TypeExprExt, ValueExprExt};
use crate::utils::serialization::{
    deserialize_map, deserialize_vector, serialize_map, serialize_vector,
};
use causality_types::{
    core::{
        id::{ExprId, TypeExprId, ValueExprId},
        numeric::Number,
        str::Str,
    },
    expr::{
        ast::{Atom, AtomicCombinator, Expr, ExprBox, ExprVec},
        expr_type::{TypeExpr, TypeExprBox, TypeExprMap, TypeExprVec},
        value::{ValueExpr, ValueExprMap, ValueExprRef, ValueExprVec},
    },
    serialization::{Decode, Encode},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Expr (AST) Utilities
//-----------------------------------------------------------------------------

/// Creates an Expr from a ValueExpr.
pub fn create_expr_from_value_expr(value: ValueExpr) -> Expr {
    match value {
        ValueExpr::String(s) => Expr::Atom(Atom::String(s)),
        ValueExpr::Number(Number::Integer(i)) => Expr::Atom(Atom::Integer(i)),
        ValueExpr::Bool(b) => Expr::Atom(Atom::Boolean(b)),
        ValueExpr::Unit | ValueExpr::Nil => Expr::Atom(Atom::Nil),
        _ => Expr::Const(value),
    }
}

pub fn create_expr_atom_integer(value: i64) -> Expr {
    Expr::Atom(Atom::Integer(value))
}

pub fn create_expr_atom_string<S: AsRef<str>>(value: S) -> Expr {
    Expr::Atom(Atom::String(Str::new(value.as_ref())))
}

pub fn create_expr_atom_boolean(value: bool) -> Expr {
    Expr::Atom(Atom::Boolean(value))
}

pub fn create_expr_atom_nil() -> Expr {
    Expr::Atom(Atom::Nil)
}

pub fn create_expr_var<S: AsRef<str>>(name: S) -> Expr {
    Expr::Var(Str::new(name.as_ref()))
}

pub fn create_expr_lambda<T: AsRef<str>>(params: Vec<T>, body: Expr) -> Expr {
    Expr::Lambda(
        params.into_iter().map(|s| Str::new(s.as_ref())).collect(),
        ExprBox(Box::new(body)),
    )
}

pub fn create_expr_apply(func: Expr, args: Vec<Expr>) -> Expr {
    Expr::Apply(ExprBox(Box::new(func)), ExprVec(args))
}

pub fn create_expr_combinator(comb: AtomicCombinator) -> Expr {
    Expr::Combinator(comb)
}

pub fn create_expr_dynamic(steps: u32, expr: Expr) -> Expr {
    Expr::Dynamic(steps, ExprBox(Box::new(expr)))
}

/// Specific combinator constructors
pub fn create_expr_if(cond: Expr, then_expr: Expr, else_expr: Expr) -> Expr {
    create_expr_apply(
        create_expr_combinator(AtomicCombinator::If),
        vec![cond, then_expr, else_expr],
    )
}

pub fn create_expr_s_combinator() -> Expr {
    create_expr_combinator(AtomicCombinator::S)
}

pub fn create_expr_k_combinator() -> Expr {
    create_expr_combinator(AtomicCombinator::K)
}

pub fn create_expr_i_combinator() -> Expr {
    create_expr_combinator(AtomicCombinator::I)
}

pub fn create_expr_add(lhs: Expr, rhs: Expr) -> Expr {
    create_expr_apply(
        create_expr_combinator(AtomicCombinator::Add),
        vec![lhs, rhs],
    )
}

pub fn create_expr_eq(lhs: Expr, rhs: Expr) -> Expr {
    create_expr_apply(create_expr_combinator(AtomicCombinator::Eq), vec![lhs, rhs])
}

/// Compute the ID for an Expr
pub fn compute_expr_id(expr: &Expr) -> ExprId {
    let hash = compute_expr_hash(expr);
    ExprId(hash)
}

/// Compute the hash for an Expr
pub fn compute_expr_hash(expr: &Expr) -> [u8; 32] {
    let bytes = expr.to_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

/// Extract a ValueExpr from this Expr, if it represents a Const value.
pub fn expr_as_value(expr: &Expr) -> Option<&ValueExpr> {
    match expr {
        Expr::Const(value) => Some(value),
        _ => None,
    }
}

/// Check if this Expr is a Const ValueExpr of a type that satisfies the predicate.
pub fn expr_is_value_of_type<F>(expr: &Expr, type_check: F) -> bool
where
    F: Fn(&ValueExpr) -> bool,
{
    match expr_as_value(expr) {
        Some(value) => type_check(value),
        None => false,
    }
}

/// Serialize an AST expression using SSZ.
pub fn serialize_expr(expr: &Expr) -> Result<Vec<u8>, String> {
    Ok(expr.as_ssz_bytes())
}

/// Deserialize an AST expression from SSZ.
pub fn deserialize_expr(bytes: &[u8]) -> Result<Expr, String> {
    Expr::from_ssz_bytes(bytes).map_err(|e| e.message)
}

//-----------------------------------------------------------------------------
// ValueExpr Utilities
//-----------------------------------------------------------------------------

/// ValueExpr constructors
pub fn create_value_expr_unit() -> ValueExpr {
    ValueExpr::Unit
}

pub fn create_value_expr_bool(b: bool) -> ValueExpr {
    ValueExpr::Bool(b)
}

pub fn create_value_expr_int(i: i64) -> ValueExpr {
    ValueExpr::Number(Number::Integer(i))
}

pub fn create_value_expr_string<S: AsRef<str>>(s: S) -> ValueExpr {
    ValueExpr::String(Str::new(s.as_ref()))
}

pub fn create_value_expr_nil() -> ValueExpr {
    ValueExpr::Nil
}

pub fn create_value_expr_number(n: Number) -> ValueExpr {
    ValueExpr::Number(n)
}

pub fn create_value_expr_list(items: Vec<ValueExpr>) -> ValueExpr {
    ValueExpr::List(ValueExprVec(items))
}

pub fn create_value_expr_map(map_items: BTreeMap<Str, ValueExpr>) -> ValueExpr {
    ValueExpr::Map(ValueExprMap(map_items))
}

pub fn create_value_expr_record(record_items: BTreeMap<Str, ValueExpr>) -> ValueExpr {
    ValueExpr::Record(ValueExprMap(record_items))
}

pub fn create_value_expr_ref(reference: ValueExprRef) -> ValueExpr {
    ValueExpr::Ref(reference)
}

pub fn create_value_expr_lambda(
    params: Vec<Str>,
    body_expr_id: ExprId,
    captured_env: ValueExprMap,
) -> ValueExpr {
    ValueExpr::Lambda {
        params,
        body_expr_id,
        captured_env,
    }
}

pub fn create_value_expr_empty_list() -> ValueExpr {
    ValueExpr::List(ValueExprVec(Vec::new()))
}

pub fn create_value_expr_empty_map() -> ValueExpr {
    ValueExpr::Map(ValueExprMap(BTreeMap::new()))
}

pub fn create_value_expr_empty_record() -> ValueExpr {
    ValueExpr::Record(ValueExprMap(BTreeMap::new()))
}

/// Compute the ID of a ValueExpr
pub fn compute_value_expr_id(value_expr: &ValueExpr) -> ValueExprId {
    value_expr.id()
}

/// Compute the hash of a ValueExpr
pub fn compute_value_expr_hash(value_expr: &ValueExpr) -> [u8; 32] {
    let bytes = value_expr.to_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

/// Accessor functions for ValueExpr
pub fn value_expr_as_bool(value: &ValueExpr) -> Option<bool> {
    if let ValueExpr::Bool(b) = value {
        Some(*b)
    } else {
        None
    }
}

pub fn value_expr_as_int(value: &ValueExpr) -> Option<i64> {
    if let ValueExpr::Number(Number::Integer(i)) = value {
        Some(*i)
    } else {
        None
    }
}

pub fn value_expr_as_number(value: &ValueExpr) -> Option<&Number> {
    if let ValueExpr::Number(n) = value {
        Some(n)
    } else {
        None
    }
}

pub fn value_expr_as_string(value: &ValueExpr) -> Option<&Str> {
    if let ValueExpr::String(s) = value {
        Some(s)
    } else {
        None
    }
}

pub fn value_expr_as_list(value: &ValueExpr) -> Option<&[ValueExpr]> {
    if let ValueExpr::List(ValueExprVec(items)) = value {
        Some(items)
    } else {
        None
    }
}

pub fn value_expr_as_list_mut(value: &mut ValueExpr) -> Option<&mut Vec<ValueExpr>> {
    if let ValueExpr::List(ValueExprVec(items)) = value {
        Some(items)
    } else {
        None
    }
}

pub fn value_expr_as_map(value: &ValueExpr) -> Option<&BTreeMap<Str, ValueExpr>> {
    if let ValueExpr::Map(ValueExprMap(map_items)) = value {
        Some(map_items)
    } else {
        None
    }
}

pub fn value_expr_as_map_mut(
    value: &mut ValueExpr,
) -> Option<&mut BTreeMap<Str, ValueExpr>> {
    if let ValueExpr::Map(ValueExprMap(map_items)) = value {
        Some(map_items)
    } else {
        None
    }
}

pub fn value_expr_as_record(value: &ValueExpr) -> Option<&BTreeMap<Str, ValueExpr>> {
    if let ValueExpr::Record(ValueExprMap(record_items)) = value {
        Some(record_items)
    } else {
        None
    }
}

pub fn value_expr_as_record_mut(
    value: &mut ValueExpr,
) -> Option<&mut BTreeMap<Str, ValueExpr>> {
    if let ValueExpr::Record(ValueExprMap(record_items)) = value {
        Some(record_items)
    } else {
        None
    }
}

pub fn value_expr_as_ref(value: &ValueExpr) -> Option<&ValueExprRef> {
    if let ValueExpr::Ref(r) = value {
        Some(r)
    } else {
        None
    }
}

/// Predicate functions for ValueExpr
pub fn value_expr_is_unit(value: &ValueExpr) -> bool {
    matches!(value, ValueExpr::Unit)
}

pub fn value_expr_is_bool(value: &ValueExpr) -> bool {
    matches!(value, ValueExpr::Bool(_))
}

pub fn value_expr_is_int(value: &ValueExpr) -> bool {
    matches!(value, ValueExpr::Number(Number::Integer(_)))
}

pub fn value_expr_is_string(value: &ValueExpr) -> bool {
    matches!(value, ValueExpr::String(_))
}

pub fn value_expr_is_list(value: &ValueExpr) -> bool {
    matches!(value, ValueExpr::List(_))
}

pub fn value_expr_is_map(value: &ValueExpr) -> bool {
    matches!(value, ValueExpr::Map(_))
}

pub fn value_expr_is_record(value: &ValueExpr) -> bool {
    matches!(value, ValueExpr::Record(_))
}

pub fn value_expr_is_ref(value: &ValueExpr) -> bool {
    matches!(value, ValueExpr::Ref(_))
}

pub fn value_expr_is_numeric(value: &ValueExpr) -> bool {
    matches!(value, ValueExpr::Number(_))
}

pub fn value_expr_as_bytes(value: &ValueExpr) -> Option<Vec<u8>> {
    match value {
        ValueExpr::String(s) => Some(s.as_str().as_bytes().to_vec()),
        _ => None,
    }
}

pub fn value_expr_get_variant_name(value: &ValueExpr) -> &'static str {
    match value {
        ValueExpr::Unit => "Unit",
        ValueExpr::Bool(_) => "Bool",
        ValueExpr::Number(_) => "Number",
        ValueExpr::String(_) => "String",
        ValueExpr::List(_) => "List",
        ValueExpr::Map(_) => "Map",
        ValueExpr::Record(_) => "Record",
        ValueExpr::Ref(_) => "Ref",
        ValueExpr::Lambda { .. } => "Lambda",
        ValueExpr::Nil => "Nil",
    }
}

/// Serialization utilities for ValueExpr
pub fn serialize_value_expr_vec(vec: &[ValueExpr]) -> Result<Vec<u8>, std::io::Error> {
    serialize_vector(vec)
}

pub fn deserialize_value_expr_vec(bytes: &[u8]) -> Result<Vec<ValueExpr>, std::io::Error> {
    deserialize_vector(bytes)
}

pub fn serialize_str_value_expr_map(
    map: &BTreeMap<Str, ValueExpr>,
) -> Result<Vec<u8>, std::io::Error> {
    serialize_map(map)
}

pub fn deserialize_str_value_expr_map(
    bytes: &[u8],
) -> Result<BTreeMap<Str, ValueExpr>, std::io::Error> {
    deserialize_map(bytes)
}

//-----------------------------------------------------------------------------
// TypeExpr Utilities
//-----------------------------------------------------------------------------

/// Creates a new `List` type expression.
pub fn create_type_expr_list(element_type: TypeExpr) -> TypeExpr {
    TypeExpr::List(TypeExprBox(Box::new(element_type)))
}

/// Creates a new `Map` type expression.
pub fn create_type_expr_map(key_type: TypeExpr, value_type: TypeExpr) -> TypeExpr {
    TypeExpr::Map(
        TypeExprBox(Box::new(key_type)),
        TypeExprBox(Box::new(value_type)),
    )
}

/// Creates a new `Record` type expression.
pub fn create_type_expr_record(fields: BTreeMap<Str, TypeExpr>) -> TypeExpr {
    TypeExpr::Record(TypeExprMap(fields))
}

/// Creates a new `Union` type expression.
pub fn create_type_expr_union(types: Vec<TypeExpr>) -> TypeExpr {
    TypeExpr::Union(TypeExprVec(types))
}

/// Creates a new `Intersection` type expression.
pub fn create_type_expr_intersection(types: Vec<TypeExpr>) -> TypeExpr {
    TypeExpr::Intersection(TypeExprVec(types))
}

/// Creates a new `Optional` type expression.
pub fn create_type_expr_optional(inner_type: TypeExpr) -> TypeExpr {
    TypeExpr::Optional(TypeExprBox(Box::new(inner_type)))
}

/// Creates a new `Tuple` type expression.
pub fn create_type_expr_tuple(types: Vec<TypeExpr>) -> TypeExpr {
    TypeExpr::Tuple(TypeExprVec(types))
}

/// Creates a new `Function` type expression.
pub fn create_type_expr_function(param_type: TypeExpr, return_type: TypeExpr) -> TypeExpr {
    TypeExpr::Function(
        TypeExprBox(Box::new(param_type)),
        TypeExprBox(Box::new(return_type)),
    )
}

/// Checks if the type expression represents a primitive type.
pub fn type_expr_is_primitive(type_expr: &TypeExpr) -> bool {
    matches!(
        type_expr,
        TypeExpr::Any
            | TypeExpr::Unit
            | TypeExpr::Bool
            | TypeExpr::String
            | TypeExpr::Integer
            | TypeExpr::Fixed
            | TypeExpr::Ratio
            | TypeExpr::Number
    )
}

/// Checks if the type expression can be considered a schema type.
pub fn type_expr_is_schema_type(type_expr: &TypeExpr) -> bool {
    matches!(
        type_expr,
        TypeExpr::List(_)
            | TypeExpr::Map(_, _)
            | TypeExpr::Record(_)
            | TypeExpr::Union(_)
            | TypeExpr::Intersection(_)
            | TypeExpr::Optional(_)
            | TypeExpr::Tuple(_)
            | TypeExpr::Enum(_)
            | TypeExpr::Function(_, _)
    )
}

/// Compute the ID of a TypeExpr
pub fn compute_type_expr_id(type_expr: &TypeExpr) -> TypeExprId {
    type_expr.id()
}

/// Compute the hash of a TypeExpr
pub fn compute_type_expr_hash(type_expr: &TypeExpr) -> [u8; 32] {
    let bytes = type_expr.to_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

/// Derives a schema ID for a given `TypeExpr` by computing its content hash.
pub fn derive_schema_id(type_expr: &TypeExpr) -> TypeExprId {
    type_expr.id()
} 