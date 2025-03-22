//! Effect Adapter Code Generation
//!
//! This module provides the framework for generating effect adapter code
//! from adapter schemas. It includes code generators for different languages
//! and templating utilities.

use std::collections::HashMap;
use std::path::Path;
use crate::error::Result;
#[cfg(feature = "domain")]
use crate::domain_adapters::schemas::AdapterSchema;

pub mod rust;
pub mod templates;
pub mod javascript;
pub mod riscv;
pub mod converters;
#[cfg(feature = "zk-vm")]
pub mod zk;

/// Target language for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    /// Generate Rust code
    Rust,
    /// Generate TypeScript code
    TypeScript,
    /// Generate RISC-V code
    RiscV,
    /// Generate ZK-compatible code
    ZkRiscV,
}

/// Code generation options
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Target language
    pub target: CodegenTarget,
    /// Output directory
    pub output_dir: String,
    /// Generate tests
    pub generate_tests: bool,
    /// Generate documentation
    pub generate_docs: bool,
    /// Generate examples
    pub generate_examples: bool,
    /// Additional options specific to the target language
    pub additional_options: HashMap<String, String>,
    /// Output path
    pub output_path: Option<String>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        CodegenOptions {
            target: CodegenTarget::Rust,
            output_dir: "src/generated".to_string(),
            generate_tests: true,
            generate_docs: true,
            generate_examples: false,
            additional_options: HashMap::new(),
            output_path: None,
        }
    }
}

/// Code generation context
#[derive(Debug)]
pub struct CodegenContext<'a> {
    /// Adapter schema
    pub schema: &'a AdapterSchema,
    /// Code generation options
    pub options: CodegenOptions,
    /// Template variables
    pub variables: HashMap<String, String>,
}

impl<'a> CodegenContext<'a> {
    /// Create a new code generation context
    pub fn new(schema: &'a AdapterSchema, options: CodegenOptions) -> Self {
        let mut variables = HashMap::new();
        
        // Add basic variables
        variables.insert("DOMAIN_ID".to_string(), schema.domain_id.as_ref().to_string());
        variables.insert("DOMAIN_TYPE".to_string(), schema.domain_type.clone());
        variables.insert("SCHEMA_VERSION".to_string(), schema.version.clone());
        
        CodegenContext {
            schema,
            options,
            variables,
        }
    }
    
    /// Add a variable to the context
    pub fn add_variable(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.variables.insert(name.into(), value.into());
        self
    }
    
    /// Get a variable from the context
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }
}

/// Trait for code generators
pub trait CodeGenerator {
    /// Generate adapter code from a schema
    fn generate(&self, context: &CodegenContext) -> Result<GeneratedCode>;
    
    /// Get the target language for this generator
    fn target(&self) -> CodegenTarget;
    
    /// Write generated code to disk
    fn write_to_disk(&self, code: &GeneratedCode, output_dir: &Path) -> Result<()>;
}

/// Generated adapter code
#[derive(Debug)]
pub struct GeneratedCode {
    /// Main adapter implementation file
    pub adapter_impl: String,
    /// Additional support files
    pub support_files: HashMap<String, String>,
    /// Test files
    pub test_files: HashMap<String, String>,
    /// Documentation files
    pub doc_files: HashMap<String, String>,
    /// Example files
    pub example_files: HashMap<String, String>,
}

impl GeneratedCode {
    /// Create a new empty generated code instance
    pub fn new() -> Self {
        GeneratedCode {
            adapter_impl: String::new(),
            support_files: HashMap::new(),
            test_files: HashMap::new(),
            doc_files: HashMap::new(),
            example_files: HashMap::new(),
        }
    }
    
    /// Set the main adapter implementation
    pub fn set_adapter_impl(&mut self, code: impl Into<String>) -> &mut Self {
        self.adapter_impl = code.into();
        self
    }
    
    /// Add a support file
    pub fn add_support_file(&mut self, name: impl Into<String>, code: impl Into<String>) -> &mut Self {
        self.support_files.insert(name.into(), code.into());
        self
    }
    
    /// Add a test file
    pub fn add_test_file(&mut self, name: impl Into<String>, code: impl Into<String>) -> &mut Self {
        self.test_files.insert(name.into(), code.into());
        self
    }
    
    /// Add a documentation file
    pub fn add_doc_file(&mut self, name: impl Into<String>, content: impl Into<String>) -> &mut Self {
        self.doc_files.insert(name.into(), content.into());
        self
    }
    
    /// Add an example file
    pub fn add_example_file(&mut self, name: impl Into<String>, code: impl Into<String>) -> &mut Self {
        self.example_files.insert(name.into(), code.into());
        self
    }
}

/// Generate code for an adapter schema
pub fn generate_adapter_code(schema: &AdapterSchema, options: CodegenOptions) -> Result<GeneratedCode> {
    let context = CodegenContext::new(schema, options.clone());
    
    match options.target {
        CodegenTarget::Rust => {
            let generator = rust::RustCodeGenerator::new();
            generator.generate(&context)
        },
        CodegenTarget::TypeScript => {
            let generator = javascript::TypeScriptCodeGenerator::new();
            generator.generate(&context)
        },
        CodegenTarget::RiscV => {
            let generator = riscv::RiscVCodeGenerator::new();
            generator.generate(&context)
        },
        CodegenTarget::ZkRiscV => {
            let generator = zk::ZkRiscVCodeGenerator::new();
            generator.generate(&context)
        },
    }
}

/// Write generated code to disk
pub fn write_generated_code(code: &GeneratedCode, output_dir: &str) -> Result<()> {
    let path = Path::new(output_dir);
    
    // Create output directory if it doesn't exist
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    
    // Write adapter implementation
    if !code.adapter_impl.is_empty() {
        let file_path = path.join("adapter_impl.rs");
        std::fs::write(file_path, &code.adapter_impl)?;
    }
    
    // Write support files
    for (name, content) in &code.support_files {
        let file_path = path.join(name);
        std::fs::write(file_path, content)?;
    }
    
    // Write test files
    let test_dir = path.join("tests");
    if !code.test_files.is_empty() && !test_dir.exists() {
        std::fs::create_dir_all(&test_dir)?;
    }
    
    for (name, content) in &code.test_files {
        let file_path = test_dir.join(name);
        std::fs::write(file_path, content)?;
    }
    
    // Write documentation files
    let doc_dir = path.join("docs");
    if !code.doc_files.is_empty() && !doc_dir.exists() {
        std::fs::create_dir_all(&doc_dir)?;
    }
    
    for (name, content) in &code.doc_files {
        let file_path = doc_dir.join(name);
        std::fs::write(file_path, content)?;
    }
    
    // Write example files
    let examples_dir = path.join("examples");
    if !code.example_files.is_empty() && !examples_dir.exists() {
        std::fs::create_dir_all(&examples_dir)?;
    }
    
    for (name, content) in &code.example_files {
        let file_path = examples_dir.join(name);
        std::fs::write(file_path, content)?;
    }
    
    Ok(())
}

/// Create a code generator for the specified target
pub fn create_generator(target: CodegenTarget) -> Result<Box<dyn CodeGenerator>> {
    match target {
        CodegenTarget::Rust => {
            Ok(Box::new(rust::RustCodeGenerator::new()))
        },
        CodegenTarget::TypeScript => {
            Ok(Box::new(javascript::TypeScriptCodeGenerator::new()))
        },
        CodegenTarget::RiscV => {
            Ok(Box::new(riscv::RiscVCodeGenerator::new()))
        },
        #[cfg(feature = "zk-vm")]
        CodegenTarget::ZkRiscV => {
            Ok(Box::new(zk::ZkRiscVCodeGenerator::new()))
        },
        #[cfg(not(feature = "zk-vm"))]
        CodegenTarget::ZkRiscV => {
            Err(crate::error::Error::FeatureNotEnabled(
                "zk-vm feature is required for ZK-RISC-V code generation".to_string()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(feature = "domain")]
    fn create_test_schema() -> AdapterSchema {
        let mut schema = AdapterSchema::default();
        schema.name = "TestAdapter".to_string();
        schema.domain_id = crate::domain_adapters::schemas::DomainId::from_str("test-domain").unwrap();
        schema.domain_type = "TestDomain".to_string();
        schema.version = "1.0.0".to_string();
        schema
    }
    
    #[test]
    #[cfg(feature = "domain")]
    fn test_codegen_context() {
        let schema = create_test_schema();
        let options = CodegenOptions::default();
        let context = CodegenContext::new(&schema, options);
        
        assert_eq!(context.get_variable("DOMAIN_ID"), Some(&"test-domain".to_string()));
        assert_eq!(context.get_variable("DOMAIN_TYPE"), Some(&"TestDomain".to_string()));
    }
    
    #[test]
    #[cfg(feature = "domain")]
    fn test_generated_code() {
        let mut code = GeneratedCode::new();
        code.set_adapter_impl("impl TestAdapter {}");
        code.add_support_file("types.rs", "type TestType = u32;");
        code.add_test_file("basic_test.rs", "#[test] fn test() {}");
        
        assert_eq!(code.adapter_impl, "impl TestAdapter {}");
        assert_eq!(code.support_files.len(), 1);
        assert_eq!(code.test_files.len(), 1);
    }
} 