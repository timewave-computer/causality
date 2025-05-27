//! S-expression serialization for numeric types
//!
//! This module provides S-expression serialization for numeric types in the Causality system.

use crate::primitive::number::Number;
use crate::expression::sexpr::{tagged_sexpr, FromSexpr, ToSexpr};
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
        // Parse as list manually - lexpr uses Cons structures
        if sexpr.is_list() {
            // Use the cons iterator to get elements
            let mut iter = sexpr.list_iter().ok_or_else(|| anyhow!("Expected a list"))?;
            
            // Get the tag (first element)
            let tag_value = iter.next().ok_or_else(|| anyhow!("Empty list"))?;
            let tag = tag_value.as_symbol().ok_or_else(|| anyhow!("First element must be a symbol"))?;
            
            // Get the value (second element)
            let value_element = iter.next().ok_or_else(|| anyhow!("Missing value element"))?;
            
            match tag {
                "integer" => {
                    let value = value_element.as_i64()
                        .ok_or_else(|| anyhow!("Integer value must be a number"))?;
                    Ok(Number::Integer(value))
                }
                "decimal" => {
                    let value_str = value_element.as_str()
                        .ok_or_else(|| anyhow!("Decimal value must be a string"))?;
                    Number::from_decimal_str(value_str)
                        .map_err(|e| anyhow!("Failed to parse decimal: {}", e))
                }
                _ => Err(anyhow!("Unknown number type tag: {}", tag)),
            }
        } else {
            Err(anyhow!("Number S-expression must be a list"))
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