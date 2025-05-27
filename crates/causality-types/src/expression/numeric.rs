//! S-expression serialization for numeric types
//!
//! This module provides S-expression serialization for numeric types in the Causality system.

use crate::primitive::number::Number;
use crate::expression::sexpr::{tagged_sexpr, validate_tag, get_list_elements, get_i64_value, get_string_value, FromSexpr, ToSexpr};
use lexpr::Value as SexprValue;
use anyhow::{anyhow, Result};

impl ToSexpr for Number {
    fn to_sexpr(&self) -> SexprValue {
        match self {
            Number::Integer(value) => {
                tagged_sexpr("integer", vec![SexprValue::Number((*value).into())])
            }
            Number::Decimal(value) => {
                // Serialize decimal as string to preserve precision
                tagged_sexpr("decimal", vec![SexprValue::String(value.to_string().into())])
            }
        }
    }
}

impl FromSexpr for Number {
    fn from_sexpr(sexpr: &SexprValue) -> Result<Self> {
        let tag = validate_tag(sexpr, "")?;
        
        let elements = get_list_elements(sexpr)
            .ok_or_else(|| anyhow!("Number S-expression must be a list"))?;
        
        match tag {
            "integer" => {
                if elements.len() != 2 {
                    return Err(anyhow!("Integer number must have exactly one value"));
                }
                
                let value = get_i64_value(&elements[1])
                    .ok_or_else(|| anyhow!("Integer value must be a number"))?;
                
                Ok(Number::Integer(value))
            }
            "decimal" => {
                if elements.len() != 2 {
                    return Err(anyhow!("Decimal number must have exactly one value"));
                }
                
                let value_str = get_string_value(&elements[1])
                    .ok_or_else(|| anyhow!("Decimal value must be a string"))?;
                
                Number::from_decimal_str(value_str)
                    .map_err(|e| anyhow!("Failed to parse decimal: {}", e))
            }
            _ => Err(anyhow!("Unknown number type tag: {}", tag)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_serialization() {
        let number = Number::Integer(42);
        let sexpr = number.to_sexpr();
        let roundtrip = Number::from_sexpr(&sexpr).unwrap();
        assert_eq!(number, roundtrip);
    }

    #[test]
    fn test_decimal_serialization() {
        let number = Number::from_decimal_str("3.14").unwrap();
        let sexpr = number.to_sexpr();
        let roundtrip = Number::from_sexpr(&sexpr).unwrap();
        assert_eq!(number, roundtrip);
    }
} 