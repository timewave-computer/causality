// Code generation module for effect adapters
// This module handles generating code from adapter schemas

use std::path::Path;
use std::fs;
use anyhow::{Result, Context};

use crate::schemas::{AdapterSchema, load_schema, validate_schema};

/// Compile a schema into adapter code
pub fn compile_schema(schema_path: &Path, output_dir: &Path, language: &str, verbose: bool) -> Result<()> {
    // Load and validate the schema
    let schema = load_schema(schema_path)?;
    validate_schema(&schema)?;
    
    // Check if language is supported
    if !is_language_supported(language) {
        anyhow::bail!("Unsupported language: {}", language);
    }
    
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)
        .context(format!("Failed to create output directory: {}", output_dir.display()))?;
    
    // Generate code based on language
    match language {
        "rust" => generate_rust_adapter(&schema, output_dir, verbose)?,
        "typescript" => generate_typescript_adapter(&schema, output_dir, verbose)?,
        _ => anyhow::bail!("Unsupported language: {}", language),
    }
    
    Ok(())
}

/// Check if a language is supported
fn is_language_supported(language: &str) -> bool {
    matches!(language, "rust" | "typescript")
}

/// Generate Rust adapter code
fn generate_rust_adapter(schema: &AdapterSchema, output_dir: &Path, verbose: bool) -> Result<()> {
    if verbose {
        println!("Generating Rust adapter: {}", schema.name);
    }
    
    // Determine the output file path
    let mut file_name = schema.name.to_lowercase();
    file_name = file_name.replace(' ', "_");
    let output_file = output_dir.join(format!("{}.rs", file_name));
    
    // Generate the adapter code
    let mut code = String::new();
    
    // Add header
    code.push_str(&format!("// Adapter: {}\n", schema.name));
    code.push_str(&format!("// Version: {}\n", schema.version));
    code.push_str("// Generated code - do not modify\n\n");
    
    // Add imports
    code.push_str("use anyhow::Result;\n");
    code.push_str("use serde::{Serialize, Deserialize};\n");
    code.push_str("use std::sync::Arc;\n\n");
    
    // Generate effect structs
    for effect in &schema.effects {
        generate_rust_effect(effect, &mut code)?;
    }
    
    // Write the code to the output file
    fs::write(&output_file, code)
        .context(format!("Failed to write output file: {}", output_file.display()))?;
    
    if verbose {
        println!("Generated adapter code: {}", output_file.display());
    }
    
    Ok(())
}

/// Generate a Rust effect struct
fn generate_rust_effect(effect: &crate::schemas::EffectDefinition, code: &mut String) -> Result<()> {
    // Add documentation
    if !effect.documentation.is_empty() {
        code.push_str(&format!("/// {}\n", effect.documentation));
    }
    
    // Add struct definition
    code.push_str(&format!("#[derive(Debug, Clone, Serialize, Deserialize)]\n"));
    code.push_str(&format!("pub struct {}Effect {{\n", effect.name));
    
    // Add fields
    for param in &effect.parameters {
        if !param.documentation.is_empty() {
            code.push_str(&format!("    /// {}\n", param.documentation));
        }
        code.push_str(&format!("    pub {}: {},\n", param.name, rust_type_for(&param.type_name)));
    }
    
    // Close struct
    code.push_str("}\n\n");
    
    // Add implementation
    code.push_str(&format!("impl {}Effect {{\n", effect.name));
    
    // Add constructor
    code.push_str("    /// Create a new effect\n");
    code.push_str("    pub fn new(\n");
    for param in &effect.parameters {
        code.push_str(&format!("        {}: {},\n", param.name, rust_type_for(&param.type_name)));
    }
    code.push_str("    ) -> Self {\n");
    code.push_str("        Self {\n");
    for param in &effect.parameters {
        code.push_str(&format!("            {},\n", param.name));
    }
    code.push_str("        }\n");
    code.push_str("    }\n");
    
    // Close implementation
    code.push_str("}\n\n");
    
    Ok(())
}

/// Map schema type to Rust type
fn rust_type_for(type_name: &str) -> &str {
    match type_name {
        "String" => "String",
        "Number" => "f64",
        "Integer" => "i64",
        "Boolean" => "bool",
        "Array" => "Vec<serde_json::Value>",
        "Object" => "std::collections::HashMap<String, serde_json::Value>",
        _ => "String",  // Default to String for unknown types
    }
}

/// Generate TypeScript adapter code
fn generate_typescript_adapter(schema: &AdapterSchema, output_dir: &Path, verbose: bool) -> Result<()> {
    if verbose {
        println!("Generating TypeScript adapter: {}", schema.name);
    }
    
    // Stub implementation - to be expanded later
    let mut file_name = schema.name.to_lowercase();
    file_name = file_name.replace(' ', "_");
    let output_file = output_dir.join(format!("{}.ts", file_name));
    
    let code = format!("// TypeScript adapter for {}\n// Version: {}\n// This is a stub implementation\n",
        schema.name, schema.version);
    
    fs::write(&output_file, code)
        .context(format!("Failed to write output file: {}", output_file.display()))?;
    
    if verbose {
        println!("Generated TypeScript adapter: {}", output_file.display());
    }
    
    Ok(())
}

/// Create template directories for templates and sample schemas
#[allow(dead_code)]
pub fn create_template_structure(base_dir: &Path) -> Result<()> {
    // Create required directories
    let template_dir = base_dir.join("templates");
    let rust_dir = template_dir.join("rust");
    let ts_dir = template_dir.join("typescript");
    let schema_dir = base_dir.join("schemas");
    
    fs::create_dir_all(&rust_dir)?;
    fs::create_dir_all(&ts_dir)?;
    fs::create_dir_all(&schema_dir)?;
    
    // Create sample files
    create_sample_rust_template(&rust_dir)?;
    create_sample_schema(&schema_dir)?;
    
    Ok(())
}

/// Create a sample Rust template file
#[allow(dead_code)]
fn create_sample_rust_template(dir: &Path) -> Result<()> {
    let template = r#"// Adapter: {{name}}
// Version: {{version}}
// Generated code - do not modify

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

{{#each effects}}
/// {{documentation}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{name}}Effect {
    {{#each parameters}}
    /// {{documentation}}
    pub {{name}}: {{type}},
    {{/each}}
}

impl {{name}}Effect {
    /// Create a new effect
    pub fn new(
        {{#each parameters}}
        {{name}}: {{type}},
        {{/each}}
    ) -> Self {
        Self {
            {{#each parameters}}
            {{name}},
            {{/each}}
        }
    }
}
{{/each}}
"#;
    
    let output_file = dir.join("adapter.rs.tmpl");
    fs::write(output_file, template)?;
    
    Ok(())
}

/// Create a sample schema file
#[allow(dead_code)]
fn create_sample_schema(dir: &Path) -> Result<()> {
    let schema = r#"{
  "name": "EthereumAdapter",
  "version": "1.0.0",
  "language": "rust",
  "effects": [
    {
      "name": "Transfer",
      "function": "transfer",
      "documentation": "Transfer ETH or tokens between accounts",
      "parameters": [
        {
          "name": "from",
          "type_name": "String",
          "required": true,
          "documentation": "Source address"
        },
        {
          "name": "to",
          "type_name": "String",
          "required": true,
          "documentation": "Destination address"
        },
        {
          "name": "amount",
          "type_name": "String",
          "required": true,
          "documentation": "Amount to transfer"
        },
        {
          "name": "token",
          "type_name": "String",
          "required": false,
          "default_value": "ETH",
          "documentation": "Token to transfer (default: ETH)"
        }
      ]
    }
  ]
}"#;
    
    let output_file = dir.join("sample.json");
    fs::write(output_file, schema)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[test]
    fn test_is_language_supported() {
        assert!(is_language_supported("rust"));
        assert!(is_language_supported("typescript"));
        assert!(!is_language_supported("python"));
    }
    
    #[test]
    fn test_rust_type_mapping() {
        assert_eq!(rust_type_for("String"), "String");
        assert_eq!(rust_type_for("Number"), "f64");
        assert_eq!(rust_type_for("Integer"), "i64");
        assert_eq!(rust_type_for("Boolean"), "bool");
        assert_eq!(rust_type_for("Unknown"), "String");
    }
} 