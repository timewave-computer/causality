//! RISC-V Code Generator for Effect Adapters
//!
//! This module provides functionality for generating RISC-V code
//! from effect adapter schemas, allowing effects to be executed
//! in a zero-knowledge virtual machine.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use crate::error::{Error, Result};
use crate::effect_adapters::codegen::{CodeGenerator, CodegenContext, CodegenTarget, GeneratedCode};
#[cfg(feature = "domain")]
use crate::domain_adapters::schemas::{OperationSchema, TypeSchema};
use crate::effect_adapters::codegen::templates::riscv as external_templates;
use crate::effect_adapters::codegen::templates::apply_template;

/// RISC-V code section type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiscVSectionType {
    /// Code section (.text)
    Text,
    /// Data section (.data)
    Data,
    /// Read-only data section (.rodata)
    RoData,
    /// BSS section (.bss) for uninitialized data
    Bss,
}

/// Represents a RISC-V program section
#[derive(Debug, Clone)]
pub struct RiscVProgramSection {
    /// Section name
    pub name: String,
    /// Section type
    pub section_type: RiscVSectionType,
    /// Section content
    pub content: Vec<u8>,
    /// Section address
    pub address: usize,
    /// Section size
    pub size: usize,
}

/// RISC-V compilation options
#[derive(Debug, Clone)]
pub struct RiscVCompileOptions {
    /// Optimization level
    pub optimization_level: OptimizationLevel,
    /// Generate debug information
    pub debug_info: bool,
    /// Use RISC-V extensions beyond basic RV32I
    pub extensions: RiscVExtensions,
    /// Memory size in bytes
    pub memory_size: usize,
    /// Stack size in bytes
    pub stack_size: usize,
}

/// RISC-V extensions
#[derive(Debug, Clone)]
pub struct RiscVExtensions {
    /// Multiply/Divide (M) extension
    pub m_extension: bool,
    /// Atomic (A) extension
    pub a_extension: bool,
    /// Compressed (C) extension
    pub c_extension: bool,
}

/// Optimization levels
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    /// No optimization (-O0)
    None,
    /// Basic optimization (-O1)
    Basic,
    /// Full optimization (-O2)
    Full,
    /// Size optimization (-Os)
    Size,
}

/// RISC-V Program representation
#[derive(Debug, Clone)]
pub struct RiscVProgram {
    /// Program name
    pub name: Option<String>,
    /// Entry point function name
    pub entry_point: String,
    /// Program sections
    pub sections: Vec<RiscVProgramSection>,
    /// Symbol table
    pub symbols: HashMap<String, usize>,
    /// Memory size in bytes
    pub memory_size: usize,
}

impl Default for RiscVCompileOptions {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::None,
            debug_info: true,
            extensions: RiscVExtensions {
                m_extension: false,
                a_extension: false,
                c_extension: false,
            },
            memory_size: 1024 * 1024, // 1MB
            stack_size: 64 * 1024,    // 64KB
        }
    }
}

/// RISC-V Code Generator
///
/// This code generator produces RISC-V assembly code for the adapter implementation.
#[cfg(feature = "domain")]
pub struct RiscVCodeGenerator {
    pub base_path: PathBuf,
}

#[cfg(feature = "domain")]
impl RiscVCodeGenerator {
    /// Create a new RISC-V code generator with the given base path.
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        RiscVCodeGenerator {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
    
    /// Generate code for a specific operation
    #[cfg(feature = "domain")]
    pub fn generate_operation_code(&self, operation: &OperationSchema) -> String {
        let mut vars = HashMap::new();
        vars.insert("OPERATION_NAME".to_string(), operation.name.clone());
        vars.insert("OPERATION_DESCRIPTION".to_string(), operation.description.clone());
        vars.insert("OPERATION_LOGIC".to_string(), "    # Operation-specific logic\n    li a0, 0\n".to_string());
        
        match apply_template(external_templates::OPERATION_TEMPLATE, &vars) {
            Ok(code) => code,
            Err(_) => "# Error generating operation code".to_string(),
        }
    }
}

#[cfg(feature = "domain")]
impl CodeGenerator for RiscVCodeGenerator {
    fn generate(&self, context: &CodegenContext) -> Result<GeneratedCode> {
        let mut generated = GeneratedCode::new();
        
        let schema = context.schema;
        
        // Setup variables for templates
        let mut vars = HashMap::new();
        vars.insert("ADAPTER_NAME".to_string(), format!("{}Adapter", schema.name));
        vars.insert("DOMAIN_ID".to_string(), schema.domain_id.as_ref().to_string());
        
        // Generate main adapter implementation
        let mut main_code = String::new();
        
        // Add program header
        let mut program_vars = vars.clone();
        program_vars.insert("PROGRAM_NAME".to_string(), schema.name.clone());
        program_vars.insert("ENTRY_POINT".to_string(), "main".to_string());
        program_vars.insert("INSTRUCTIONS".to_string(), "    # Main program logic will be inserted here".to_string());
        program_vars.insert("DATA".to_string(), "    # Data section will be populated at runtime".to_string());
        
        let program_code = apply_template(external_templates::BASIC_PROGRAM_TEMPLATE, &program_vars)?;
        main_code.push_str(&program_code);
        
        // Generate code for each operation
        let mut operations_list = String::new();
        
        for operation in &schema.operations {
            let mut op_vars = vars.clone();
            op_vars.insert("OPERATION_NAME".to_string(), operation.name.clone());
            op_vars.insert("OPERATION_DESCRIPTION".to_string(), operation.description.clone());
            op_vars.insert("OPERATION_LOGIC".to_string(), "    # Operation-specific logic\n    li a0, 0\n".to_string());
            
            let op_code = apply_template(external_templates::OPERATION_TEMPLATE, &op_vars)?;
            main_code.push_str("\n\n");
            main_code.push_str(&op_code);
            
            // Add to operations list for documentation
            operations_list.push_str(&format!("- `{}`: {}\n", operation.name, operation.description));
            
            // Add operation as a support file
            let op_file_name = format!("{}.S", operation.name);
            generated.add_support_file(op_file_name, op_code);
            
            // Generate test for this operation
            let mut test_vars = op_vars.clone();
            test_vars.insert("TEST_VERIFICATION".to_string(), "    # Verify the operation result\n    li t0, 0\n    beq a0, t0, .pass\n    j .fail".to_string());
            
            let test_code = apply_template(external_templates::TEST_TEMPLATE, &test_vars)?;
            let test_file_name = format!("test_{}.S", operation.name);
            generated.add_test_file(test_file_name, test_code);
        }
        
        generated.set_adapter_impl(main_code);
        
        // Generate documentation
        let mut doc_vars = vars.clone();
        doc_vars.insert("OPERATIONS_LIST".to_string(), operations_list);
        
        let doc_content = apply_template(external_templates::DOC_TEMPLATE, &doc_vars)?;
        generated.add_doc_file(format!("{}.md", schema.name), doc_content);
        
        Ok(generated)
    }
    
    fn target(&self) -> CodegenTarget {
        CodegenTarget::RiscV
    }
    
    fn write_to_disk(&self, code: &GeneratedCode, output_dir: &Path) -> Result<()> {
        let dir = PathBuf::from(output_dir);
        
        // Create directories if they don't exist
        fs::create_dir_all(&dir)?;
        
        // Write main implementation file
        let main_file = dir.join("adapter.S");
        fs::write(&main_file, &code.adapter_impl)?;
        
        // Write support files
        let support_dir = dir.join("operations");
        fs::create_dir_all(&support_dir)?;
        
        for (name, content) in &code.support_files {
            let file_path = support_dir.join(name);
            fs::write(&file_path, content)?;
        }
        
        // Write test files
        let test_dir = dir.join("tests");
        fs::create_dir_all(&test_dir)?;
        
        for (name, content) in &code.test_files {
            let file_path = test_dir.join(name);
            fs::write(&file_path, content)?;
        }
        
        // Write documentation files
        let doc_dir = dir.join("docs");
        fs::create_dir_all(&doc_dir)?;
        
        for (name, content) in &code.doc_files {
            let file_path = doc_dir.join(name);
            fs::write(&file_path, content)?;
        }
        
        Ok(())
    }
}

// Provide a no-op implementation when the domain feature is disabled
#[cfg(not(feature = "domain"))]
pub struct RiscVCodeGenerator;

#[cfg(not(feature = "domain"))]
impl CodeGenerator for RiscVCodeGenerator {
    fn generate(&self, _context: &CodegenContext) -> Result<GeneratedCode> {
        Err(Error::FeatureNotEnabled(
            "Domain feature is required for RISC-V code generation".to_string()
        ))
    }
    
    fn target(&self) -> CodegenTarget {
        CodegenTarget::RiscV
    }
    
    fn write_to_disk(&self, _code: &GeneratedCode, _output_dir: &Path) -> Result<()> {
        Err(Error::FeatureNotEnabled(
            "Domain feature is required for RISC-V code generation".to_string()
        ))
    }
}

/// Interface for RISC-V code writers
pub trait RiscVWriter {
    /// Write a label
    fn write_label(&mut self, label: &str) -> Result<()>;
    
    /// Write an instruction
    fn write_instruction(&mut self, instruction: &str) -> Result<()>;
    
    /// Write a comment
    fn write_comment(&mut self, comment: &str) -> Result<()>;
    
    /// Write raw assembly code
    fn write_raw(&mut self, code: &str) -> Result<()>;
    
    /// Write a data directive
    fn write_data_directive(&mut self, directive: &str, value: &str) -> Result<()>;
}

#[cfg(test)]
#[cfg(feature = "domain")]
mod tests {
    use super::*;
    use crate::domain_adapters::schemas::{AdapterSchema, OperationSchema};
    
    #[test]
    fn test_riscv_code_generator() {
        // Create a simple schema for testing
        let mut schema = AdapterSchema::new("TestAdapter", "Test RISC-V adapter");
        
        let op = OperationSchema::new("test_operation", "Test operation");
        schema.add_operation(op);
        
        let context = CodegenContext {
            schema: &schema,
            options: Default::default(),
            variables: HashMap::new(),
        };
        
        // Use a temporary path for testing
        let generator = RiscVCodeGenerator::new(std::env::temp_dir());
        let result = generator.generate(&context);
        
        assert!(result.is_ok());
        let code = result.unwrap();
        
        // Check that we have the expected files
        assert!(!code.adapter_impl.is_empty());
        assert_eq!(code.support_files.len(), 1);
        assert_eq!(code.test_files.len(), 1);
        assert_eq!(code.doc_files.len(), 1);
    }
} 