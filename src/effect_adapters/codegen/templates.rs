//! Template utilities for code generation
//!
//! This module provides template handling utilities and language-specific
//! template modules for code generation.

use std::collections::HashMap;
use crate::error::{Error, Result};

pub mod rust;
pub mod javascript;
pub mod riscv;

/// Apply a template with variables
///
/// Replaces `{{VAR_NAME}}` with the corresponding value in the variables map.
pub fn apply_template(template: &str, variables: &HashMap<String, String>) -> Result<String> {
    let mut result = template.to_string();
    
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_apply_template() {
        let template = "Hello {{NAME}}! Today is {{DAY}}.";
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "World".to_string());
        vars.insert("DAY".to_string(), "Monday".to_string());
        
        let result = apply_template(template, &vars).unwrap();
        assert_eq!(result, "Hello World! Today is Monday.");
    }
    
    #[test]
    fn test_apply_template_with_missing_vars() {
        let template = "Hello {{NAME}}! Today is {{DAY}}.";
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "World".to_string());
        
        let result = apply_template(template, &vars).unwrap();
        assert_eq!(result, "Hello World! Today is {{DAY}}.");
    }
} 