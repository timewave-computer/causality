//! Core type bindings for OCaml FFI
//!
//! This module provides OCaml-compatible wrappers for core Causality types
//! with proper serialization and memory management.

#[cfg(feature = "ocaml-ffi")]
use ocaml::{FromValue, ToValue, Value};
#[cfg(feature = "ocaml-ffi")]
use ocaml_derive::{FromValue, ToValue};
#[cfg(feature = "ocaml-ffi")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "ocaml-ffi")]
use causality_core::{
    lambda::base::Value as CoreLispValue,
    resource::ResourceId as CoreResourceId,
    effect::EffectId as CoreEffectId,
};

/// OCaml-compatible wrapper for ResourceId
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromValue, ToValue)]
pub struct ResourceId {
    /// Internal resource ID (as a string for OCaml compatibility)
    pub id: String,
}

#[cfg(feature = "ocaml-ffi")]
impl ResourceId {
    pub fn new(id: String) -> Self {
        Self { id }
    }
    
    pub fn from_core(core_id: &CoreResourceId) -> Self {
        Self {
            id: core_id.to_string(),
        }
    }
    
    pub fn to_core(&self) -> CoreResourceId {
        CoreResourceId::new(self.id.clone())
    }
}

/// OCaml-compatible wrapper for EffectId
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromValue, ToValue)]
pub struct EffectId {
    /// Internal effect ID (as a string for OCaml compatibility)
    pub id: String,
}

#[cfg(feature = "ocaml-ffi")]
impl EffectId {
    pub fn new(id: String) -> Self {
        Self { id }
    }
    
    pub fn from_core(core_id: &CoreEffectId) -> Self {
        Self {
            id: core_id.to_string(),
        }
    }
    
    pub fn to_core(&self) -> CoreEffectId {
        CoreEffectId::new(self.id.clone())
    }
}

/// OCaml-compatible wrapper for ExprId (expression identifier)
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromValue, ToValue)]
pub struct ExprId {
    /// Internal expression ID (as integer for simplicity)
    pub id: u64,
}

#[cfg(feature = "ocaml-ffi")]
impl ExprId {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// OCaml-compatible wrapper for LispValue
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromValue, ToValue)]
pub enum LispValue {
    /// Unit value ()
    Unit,
    
    /// Boolean value
    Bool(bool),
    
    /// Integer value
    Int(i64),
    
    /// String value
    String(String),
    
    /// Symbol value
    Symbol(String),
    
    /// List of values
    List(Vec<LispValue>),
    
    /// Resource reference
    ResourceRef(ResourceId),
}

#[cfg(feature = "ocaml-ffi")]
impl LispValue {
    /// Convert from core LispValue to FFI LispValue
    pub fn from_core(core_value: &CoreLispValue) -> Result<Self, String> {
        match core_value {
            CoreLispValue::Unit => Ok(LispValue::Unit),
            CoreLispValue::Bool(b) => Ok(LispValue::Bool(*b)),
            CoreLispValue::Int(i) => Ok(LispValue::Int(*i as i64)),
            CoreLispValue::String(s) => Ok(LispValue::String(s.as_str().to_string())),
            CoreLispValue::Symbol(sym) => Ok(LispValue::Symbol(sym.as_str().to_string())),
            _ => Err(format!("Unsupported LispValue variant: {:?}", core_value)),
        }
    }
    
    /// Convert to core LispValue from FFI LispValue
    pub fn to_core(&self) -> Result<CoreLispValue, String> {
        match self {
            LispValue::Unit => Ok(CoreLispValue::Unit),
            LispValue::Bool(b) => Ok(CoreLispValue::Bool(*b)),
            LispValue::Int(i) => Ok(CoreLispValue::Int(*i as u32)),
            LispValue::String(s) => Ok(CoreLispValue::String(s.as_str().into())),
            LispValue::Symbol(s) => {
                Ok(CoreLispValue::Symbol(s.as_str().into()))
            }
            LispValue::List(_) => Err("List conversion not yet supported".to_string()),
            LispValue::ResourceRef(_) => Err("ResourceRef conversion not yet supported".to_string()),
        }
    }
}

/// FFI functions for LispValue manipulation
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_unit() -> LispValue {
    LispValue::Unit
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_bool(value: bool) -> LispValue {
    LispValue::Bool(value)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_int(value: i64) -> LispValue {
    LispValue::Int(value)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_string(value: String) -> LispValue {
    LispValue::String(value)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_symbol(value: String) -> LispValue {
    LispValue::Symbol(value)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_list(values: Vec<LispValue>) -> LispValue {
    LispValue::List(values)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_resource_ref(res_id: ResourceId) -> LispValue {
    LispValue::ResourceRef(res_id)
}

/// Convert LispValue to string representation for debugging
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_to_string_debug(value: LispValue) -> String {
    format!("{:?}", value)
}

/// Check if two LispValues are equal
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_equal(a: LispValue, b: LispValue) -> bool {
    a == b
}

/// Get the type tag of a LispValue
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn lisp_value_type_tag(value: LispValue) -> String {
    match value {
        LispValue::Unit => "unit".to_string(),
        LispValue::Bool(_) => "bool".to_string(),
        LispValue::Int(_) => "int".to_string(),
        LispValue::String(_) => "string".to_string(),
        LispValue::Symbol(_) => "symbol".to_string(),
        LispValue::List(_) => "list".to_string(),
        LispValue::ResourceRef(_) => "resource_ref".to_string(),
    }
}

/// ResourceId operations
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn resource_id_new(id: String) -> ResourceId {
    ResourceId::new(id)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn resource_id_to_string(res_id: ResourceId) -> String {
    res_id.id
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn resource_id_equal(a: ResourceId, b: ResourceId) -> bool {
    a == b
}

/// EffectId operations
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn effect_id_new(id: String) -> EffectId {
    EffectId::new(id)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn effect_id_to_string(effect_id: EffectId) -> String {
    effect_id.id
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn effect_id_equal(a: EffectId, b: EffectId) -> bool {
    a == b
}

/// ExprId operations
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_id_new(id: u64) -> ExprId {
    ExprId::new(id)
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_id_to_int(expr_id: ExprId) -> u64 {
    expr_id.id
}

#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_id_equal(a: ExprId, b: ExprId) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ocaml-ffi")]
    use super::*;
    
    #[cfg(feature = "ocaml-ffi")]
    #[test]
    fn test_lisp_value_conversions() {
        // Test all basic LispValue variants
        let unit = LispValue::Unit;
        let bool_val = LispValue::Bool(true);
        let int_val = LispValue::Int(42);
        let string_val = LispValue::String("test".to_string());
        let symbol_val = LispValue::Symbol("symbol".to_string());
        
        // Test type tags
        assert_eq!(lisp_value_type_tag(unit), "unit");
        assert_eq!(lisp_value_type_tag(bool_val), "bool");
        assert_eq!(lisp_value_type_tag(int_val), "int");
        assert_eq!(lisp_value_type_tag(string_val), "string");
        assert_eq!(lisp_value_type_tag(symbol_val), "symbol");
        
        // Test equality
        assert!(lisp_value_equal(LispValue::Unit, LispValue::Unit));
        assert!(lisp_value_equal(LispValue::Bool(true), LispValue::Bool(true)));
        assert!(!lisp_value_equal(LispValue::Bool(true), LispValue::Bool(false)));
    }
    
    #[cfg(feature = "ocaml-ffi")]
    #[test]
    fn test_round_trip_conversion() {
        // Test round-trip conversion for supported types
        let test_values = vec![
            CoreLispValue::Unit,
            CoreLispValue::Bool(true),
            CoreLispValue::Bool(false),
            CoreLispValue::Int(42),
            CoreLispValue::String("test".into()),
            CoreLispValue::Symbol("symbol".into()),
        ];
        
        for core_value in test_values {
            let ffi_value = LispValue::from_core(&core_value).unwrap();
            let converted_back = ffi_value.to_core().unwrap();
            assert_eq!(core_value, converted_back);
        }
    }
    
    #[cfg(feature = "ocaml-ffi")]
    #[test]
    fn test_resource_id_operations() {
        let res_id = resource_id_new("test_resource".to_string());
        assert_eq!(resource_id_to_string(res_id.clone()), "test_resource");
        
        let res_id2 = resource_id_new("test_resource".to_string());
        assert!(resource_id_equal(res_id, res_id2));
        
        let res_id3 = resource_id_new("different_resource".to_string());
        assert!(!resource_id_equal(res_id, res_id3));
    }
} 