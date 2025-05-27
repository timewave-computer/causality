//-----------------------------------------------------------------------------
// Purpose: Utility functions and implementations for the algebraic effects system
//-----------------------------------------------------------------------------

use causality_types::{
    expr::{expr_type::TypeExpr, value::ValueExpr},
    effects_core::ConversionError,
};
use crate::extension_traits::{PrimitiveEffectInput, PrimitiveEffectOutput};

//-----------------------------------------------------------------------------
// Effect Name Utilities
//-----------------------------------------------------------------------------

/// Generates a Lisp-compatible function name from an effect type string.
/// Replaces dots and colons with hyphens, and converts to lowercase.
/// Example: "my.domain::MyEffect" -> "my-domain-myeffect"
pub fn generate_lisp_function_name_for_effect_type(effect_type_str: &str) -> String {
    effect_type_str
        .replace(".", "-")
        .replace("::", "-")
        .to_lowercase()
}

//-----------------------------------------------------------------------------
// Primitive Type Helper Functions
//-----------------------------------------------------------------------------

/// Convert a ValueExpr to unit type
pub fn unit_from_value_expr(value: ValueExpr) -> Result<(), ConversionError> {
    match value {
        ValueExpr::Nil => Ok(()),
        other => Err(ConversionError::TypeMismatch {
            expected: "Nil".to_string(),
            found: format!("{:?}", other),
        }),
    }
}

/// Convert a unit to ValueExpr
pub fn unit_to_value_expr(_: &()) -> Result<ValueExpr, ConversionError> {
    Ok(ValueExpr::Nil)
}

/// Get the schema for unit type
pub fn unit_schema() -> TypeExpr {
    TypeExpr::Unit
}

/// Convert a ValueExpr to String using extension trait
pub fn string_from_value_expr(value: ValueExpr) -> Result<String, ConversionError> {
    String::from_value_expr_ext(value)
}

/// Convert a String to ValueExpr using extension trait
pub fn string_to_value_expr(value: &String) -> Result<ValueExpr, ConversionError> {
    value.to_value_expr_ext()
}

/// Get the schema for String type using extension trait
pub fn string_schema() -> TypeExpr {
    <String as PrimitiveEffectInput>::schema_ext()
}

/// Convert a ValueExpr to i64 using extension trait
pub fn i64_from_value_expr(value: ValueExpr) -> Result<i64, ConversionError> {
    i64::from_value_expr_ext(value)
}

/// Convert a i64 to ValueExpr using extension trait
pub fn i64_to_value_expr(value: &i64) -> Result<ValueExpr, ConversionError> {
    value.to_value_expr_ext()
}

/// Get the schema for i64 type using extension trait
pub fn i64_schema() -> TypeExpr {
    <i64 as PrimitiveEffectInput>::schema_ext()
}

/// Convert a ValueExpr to bool using extension trait
pub fn bool_from_value_expr(value: ValueExpr) -> Result<bool, ConversionError> {
    bool::from_value_expr_ext(value)
}

/// Convert a bool to ValueExpr using extension trait
pub fn bool_to_value_expr(value: &bool) -> Result<ValueExpr, ConversionError> {
    value.to_value_expr_ext()
}

/// Get the schema for bool type using extension trait
pub fn bool_schema() -> TypeExpr {
    <bool as PrimitiveEffectInput>::schema_ext()
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::{
        expr::value::{Number, ValueExpr},
        // EffectInput, // Removed
    };

    #[test]
    fn test_generate_lisp_name() {
        assert_eq!(
            generate_lisp_function_name_for_effect_type("com.example::MyEffect"),
            "com-example-myeffect"
        );
        assert_eq!(
            generate_lisp_function_name_for_effect_type("another.TestEffect"),
            "another-testeffect"
        );
        assert_eq!(
            generate_lisp_function_name_for_effect_type("Simple"),
            "simple"
        );
    }

    #[test]
    fn test_effect_input_output_unit() {
        // Test using extension trait directly
        let unit_value = ValueExpr::Nil;
        let _: () = unit_from_value_expr(unit_value).unwrap();
        let serialized = unit_to_value_expr(&()).unwrap();
        assert_eq!(serialized, ValueExpr::Nil);
    }

    #[test]
    fn test_effect_input_output_string() {
        // Test String implementation with extension trait
        let test_str = "hello world";
        let string_value = ValueExpr::String(test_str.into());
        let parsed: String = string_from_value_expr(string_value).unwrap();
        assert_eq!(parsed, test_str);
        let serialized = string_to_value_expr(&parsed).unwrap();
        if let ValueExpr::String(s) = serialized {
            assert_eq!(s.to_string(), test_str);
        } else {
            panic!("Expected String ValueExpr");
        }
    }

    #[test]
    fn test_effect_input_output_i64() {
        // Test i64 implementation with direct conversion
        let test_num = 42i64;
        let num_value = ValueExpr::Number(Number::Integer(test_num));
        let parsed: i64 = i64_from_value_expr(num_value).unwrap();
        assert_eq!(parsed, test_num);
        let serialized = i64_to_value_expr(&parsed).unwrap();
        if let ValueExpr::Number(Number::Integer(n)) = serialized {
            assert_eq!(n, test_num);
        } else {
            panic!("Expected Int ValueExpr");
        }
    }

    #[test]
    fn test_effect_input_output_bool() {
        // Test bool implementation with extension trait
        let test_bool = true;
        let bool_value = ValueExpr::Bool(test_bool);
        let parsed: bool = bool_from_value_expr(bool_value).unwrap();
        assert_eq!(parsed, test_bool);
        let serialized = bool_to_value_expr(&parsed).unwrap();
        if let ValueExpr::Bool(b) = serialized {
            assert_eq!(b, test_bool);
        } else {
            panic!("Expected Bool ValueExpr");
        }
    }
}
