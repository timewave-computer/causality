// sexpr_utils.rs
//
// Utilities for S-expression serialization and deserialization for Rust/OCaml interoperability
// as described in ml_work/serialization.md.
//
// This implements a canonical S-expression format for debugging and development-time data exchange.
// Content addressing is handled exclusively by SSZ serialization.

use anyhow::{Result, anyhow};
use lexpr::{Value as SexprValue, print as lexpr_print, parse as lexpr_parse};

/// Trait for types that can be converted to and from S-expressions (for debugging only)
pub trait SexprSerializable: Sized {
    /// Convert this value to a canonical S-expression
    fn to_sexpr(&self) -> SexprValue;
    
    /// Convert this value to a canonical S-expression string
    fn to_canonical_sexpr_string(&self) -> String {
        let mut output = Vec::new();
        let mut printer = lexpr_print::Printer::new(&mut output);
        printer.print(&self.to_sexpr()).expect("S-expression printing should not fail");
        String::from_utf8(output).expect("S-expression should be valid UTF-8")
    }
    
    /// Parse an S-expression into this type
    fn from_sexpr(value: &SexprValue) -> Result<Self>;
    
    /// Parse an S-expression string into this type
    fn from_sexpr_string(s: &str) -> Result<Self> {
        let value = lexpr_parse::from_str(s)
            .map_err(|e| anyhow!("Failed to parse S-expression: {}", e))?;
        Self::from_sexpr(&value)
    }
}

/// Helper functions for working with S-expressions
/// Create an S-expression list with a type tag as the first element
pub fn tagged_sexpr(tag: &str, elements: Vec<SexprValue>) -> SexprValue {
    let mut list = Vec::with_capacity(elements.len() + 1);
    list.push(SexprValue::symbol(tag));
    list.extend(elements);
    SexprValue::list(list)
}

/// Create an S-expression map with key-value pairs, sorted by key
pub fn map_sexpr(pairs: Vec<(String, SexprValue)>) -> SexprValue {
    // Sort pairs by key for canonicalization
    let mut sorted_pairs = pairs;
    sorted_pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
    
    let list: Vec<SexprValue> = sorted_pairs.into_iter()
        .map(|(key, value)| {
            SexprValue::cons(
                SexprValue::keyword(&*key),
                value
            )
        })
        .collect();
    
    SexprValue::list(list)
}

/// Get a field from a map-like S-expression by manually walking the structure
pub fn extract_field<'a>(sexpr: &'a SexprValue, field_name: &str) -> Result<&'a SexprValue> {
    if !sexpr.is_list() {
        return Err(anyhow!("S-expression is not a list"));
    }
    
    // Walk the Cons structure manually
    let mut current = sexpr;
    while let Some((item, rest)) = current.as_pair() {
        if !item.is_cons() {
            current = rest;
            continue;
        }
        
        if let Some((key, value)) = item.as_pair() {
            if !key.is_keyword() {
                current = rest;
                continue;
            }
            
            if let Some(key_str) = key.as_keyword() {
                if key_str == field_name {
                    return Ok(value);
                }
            }
        }
        
        current = rest;
    }
    
    Err(anyhow!("Field '{}' not found in S-expression", field_name))
}

/// Get the tag of an S-expression (first symbol in the list)
pub fn get_tag(sexpr: &SexprValue) -> Result<&str> {
    if !sexpr.is_list() {
        return Err(anyhow!("S-expression is not a list"));
    }
    
    // Handle Cons structure directly instead of using as_slice()
    if let Some((first, _rest)) = sexpr.as_pair() {
        if first.is_symbol() {
            return first.as_symbol().ok_or_else(|| anyhow!("Failed to extract symbol"));
        }
    }
    
    Err(anyhow!("First element is not a symbol"))
}

/// Validate that an S-expression has the expected tag
pub fn validate_tag(sexpr: &SexprValue, expected: &str) -> Result<()> {
    let tag = get_tag(sexpr)?;
    if tag != expected {
        return Err(anyhow!("Expected tag '{}', found '{}'", expected, tag));
    }
    Ok(())
}

// Helper functions for basic value extraction
pub fn get_string_value(sexpr: &SexprValue) -> Option<String> {
    match sexpr {
        SexprValue::String(s) => Some(s.to_string()),
        _ => None,
    }
}

pub fn get_i64_value(sexpr: &SexprValue) -> Option<i64> {
    sexpr.as_i64()
}

pub fn get_u64_value(sexpr: &SexprValue) -> Option<u64> {
    sexpr.as_u64()
}

pub fn get_f64_value(sexpr: &SexprValue) -> Option<f64> {
    sexpr.as_f64()
}

pub fn get_bool_value(sexpr: &SexprValue) -> Option<bool> {
    sexpr.as_bool()
}

/// Get the elements of a list if it is a list
pub fn get_list_elements(sexpr: &SexprValue) -> Option<&[SexprValue]> {
    if sexpr.is_list() {
        sexpr.as_slice()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Example struct for testing
    #[derive(Debug, Clone, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }
    
    impl SexprSerializable for Person {
        fn to_sexpr(&self) -> SexprValue {
            tagged_sexpr("person", vec![
                map_sexpr(vec![
                    ("name".to_string(), SexprValue::string(self.name.as_str())),
                    ("age".to_string(), SexprValue::from(self.age)),
                ])
            ])
        }
        
        fn from_sexpr(value: &SexprValue) -> Result<Self> {
            validate_tag(value, "person")?;
            
            // Get the second element (the map) from the Cons structure
            if let Some((_tag, rest)) = value.as_pair() {
                if let Some((map, _)) = rest.as_pair() {
                    let name = extract_field(map, "name")
                        .and_then(|v| get_string_value(v).ok_or_else(|| anyhow!("name is not a string")))?;
                        
                    let age = extract_field(map, "age")
                        .and_then(|v| get_u64_value(v).ok_or_else(|| anyhow!("age is not an integer")))
                        .map(|age| age as u32)?;
                        
                    return Ok(Person { name, age });
                }
            }
            
            Err(anyhow!("Invalid S-expression structure for Person"))
        }
    }
    
    #[test]
    fn test_sexpr_serialization() {
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };
        
        let sexpr = person.to_sexpr();
        let sexpr_str = person.to_canonical_sexpr_string();
        
        println!("S-expression: {}", sexpr_str);
        
        let roundtrip = Person::from_sexpr(&sexpr).unwrap();
        assert_eq!(person, roundtrip);
        
        let roundtrip_str = Person::from_sexpr_string(&sexpr_str).unwrap();
        assert_eq!(person, roundtrip_str);
    }
} 