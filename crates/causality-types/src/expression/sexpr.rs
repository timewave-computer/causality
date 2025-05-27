//! S-expression serialization for Causality Expression types
//!
//! This module provides S-expression serialization for the core Expression types
//! in the Causality system, enabling interoperability with OCaml as specified
//! in the ml_work/serialization.md document.

use anyhow::{anyhow, Result};
use lexpr::{parse as lexpr_parse, print as lexpr_print, Value as SexprValue};

/// Trait for types that can be converted to S-expressions
pub trait ToSexpr {
    /// Convert to an S-expression
    fn to_sexpr(&self) -> SexprValue;
    
    /// Convert to a canonical S-expression string
    fn to_canonical_sexpr_string(&self) -> String {
        let mut output = Vec::new();
        let mut printer = lexpr_print::Printer::new(&mut output);
        printer.print(&self.to_sexpr()).expect("S-expression printing should not fail");
        String::from_utf8(output).expect("S-expression should be valid UTF-8")
    }
}

/// Trait for types that can be created from S-expressions
pub trait FromSexpr: Sized {
    /// Create from an S-expression
    fn from_sexpr(value: &SexprValue) -> Result<Self>;
    
    /// Create from an S-expression string
    fn from_sexpr_string(s: &str) -> Result<Self> {
        let value = lexpr_parse::from_str(s)
            .map_err(|e| anyhow!("Failed to parse S-expression: {}", e))?;
        Self::from_sexpr(&value)
    }
}

/// Helper functions for accessing S-expression data
pub fn get_list_elements(sexpr: &SexprValue) -> Option<&[SexprValue]> {
    sexpr.as_slice()
}

pub fn get_symbol_value(sexpr: &SexprValue) -> Option<&str> {
    if sexpr.is_symbol() {
        sexpr.as_symbol()
    } else {
        None
    }
}

pub fn get_keyword_value(sexpr: &SexprValue) -> Option<&str> {
    if sexpr.is_keyword() {
        // Get the keyword without the leading ":"
        let full = sexpr.as_keyword()?;
        Some(full.trim_start_matches(':'))
    } else {
        None
    }
}

pub fn get_string_value(sexpr: &SexprValue) -> Option<&str> {
    if sexpr.is_string() {
        sexpr.as_str()
    } else {
        None
    }
}

pub fn get_i64_value(sexpr: &SexprValue) -> Option<i64> {
    if sexpr.is_number() {
        sexpr.as_i64()
    } else {
        None
    }
}

pub fn get_u64_value(sexpr: &SexprValue) -> Option<u64> {
    if sexpr.is_number() {
        sexpr.as_u64()
    } else {
        None
    }
}

pub fn get_f64_value(sexpr: &SexprValue) -> Option<f64> {
    if sexpr.is_number() {
        sexpr.as_f64()
    } else {
        None
    }
}

pub fn get_bool_value(sexpr: &SexprValue) -> Option<bool> {
    sexpr.as_bool()
}

/// Helper function to create a tagged S-expression
pub fn tagged_sexpr(tag: &str, elements: Vec<SexprValue>) -> SexprValue {
    let mut values = Vec::with_capacity(elements.len() + 1);
    values.push(SexprValue::symbol(tag));
    values.extend(elements);
    SexprValue::list(values)
}

/// Helper function to create a map-like S-expression with sorted keys
pub fn map_sexpr(items: Vec<(String, SexprValue)>) -> SexprValue {
    let mut sorted_items = items;
    sorted_items.sort_by(|(a, _), (b, _)| a.cmp(b));
    
    let pairs: Vec<SexprValue> = sorted_items.into_iter()
        .map(|(key, value)| {
            SexprValue::list(vec![
                SexprValue::keyword(&*key),
                value,
            ])
        })
        .collect();
    
    SexprValue::list(pairs)
}

/// Helper to extract a field from a tagged S-expression
pub fn extract_field<'a>(sexpr: &'a SexprValue, field_name: &str) -> Result<&'a SexprValue> {
    // Handle lexpr list iteration
    if sexpr.is_list() {
        let iter = sexpr.list_iter().ok_or_else(|| anyhow!("Expected a list"))?;
        
        for element in iter {
            if element.is_list() {
                let mut pair_iter = element.list_iter().ok_or_else(|| anyhow!("Expected a list"))?;
                if let (Some(key), Some(value)) = (pair_iter.next(), pair_iter.next()) {
                    if let Some(keyword) = get_keyword_value(key) {
                        if keyword == field_name {
                            return Ok(value);
                        }
                    }
                }
            }
        }
    }
    Err(anyhow!("Field '{}' not found in S-expression", field_name))
}

/// Helper to get tag name from a tagged S-expression
pub fn get_tag(sexpr: &SexprValue) -> Result<&str> {
    if sexpr.is_list() {
        let mut iter = sexpr.list_iter().ok_or_else(|| anyhow!("Expected a list"))?;
        if let Some(first) = iter.next() {
            if let Some(tag) = first.as_symbol() {
                return Ok(tag);
            }
        }
    }
    Err(anyhow!("S-expression is not a tagged list"))
}

/// Validate that an S-expression has the expected tag prefix and return the tag
pub fn validate_tag<'a>(sexpr: &'a SexprValue, expected_prefix: &str) -> Result<&'a str> {
    let tag = get_tag(sexpr)?;
    if !expected_prefix.is_empty() && !tag.starts_with(expected_prefix) {
        return Err(anyhow!("Expected tag starting with '{}', found '{}'", expected_prefix, tag));
    }
    Ok(tag)
}

/// Validate that an S-expression has the exact expected tag
pub fn validate_tag_exact(sexpr: &SexprValue, expected: &str) -> Result<()> {
    let tag = get_tag(sexpr)?;
    if tag != expected {
        return Err(anyhow!("Expected tag '{}', found '{}'", expected, tag));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tagged_sexpr() {
        let sexpr = tagged_sexpr("person", vec![
            SexprValue::string("Alice"),
            SexprValue::Number(30.into()),
        ]);
        
        let expected = "(person \"Alice\" 30)";
        let result = format!("{}", sexpr);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_map_sexpr() {
        let map = map_sexpr(vec![
            ("name".to_string(), SexprValue::string("Alice")),
            ("age".to_string(), SexprValue::Number(30.into())),
        ]);
        
        // The map entries should be sorted by key
        let expected = "((#:age 30) (#:name \"Alice\"))";
        let result = format!("{}", map);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_extract_field() {
        let map = map_sexpr(vec![
            ("name".to_string(), SexprValue::string("Alice")),
            ("age".to_string(), SexprValue::Number(30.into())),
        ]);
        
        let name = extract_field(&map, "name").unwrap();
        assert_eq!(name.as_str().unwrap(), "Alice");
        
        let age = extract_field(&map, "age").unwrap();
        assert_eq!(age.as_i64().unwrap(), 30);
        
        let result = extract_field(&map, "not_found");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_tag() {
        let sexpr = tagged_sexpr("person", vec![
            SexprValue::string("Alice"),
            SexprValue::Number(30.into()),
        ]);
        
        let tag = get_tag(&sexpr).unwrap();
        assert_eq!(tag, "person");
        
        let not_a_list_sexpr = SexprValue::string("not a list");
        let result = get_tag(&not_a_list_sexpr);
        assert!(result.is_err());
    }
} 