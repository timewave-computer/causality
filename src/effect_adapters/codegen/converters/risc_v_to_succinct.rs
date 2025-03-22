//! RISC-V to Succinct Code Converter
//!
//! This module provides functionality to convert RISC-V assembly code
//! to the Succinct language format for zero-knowledge proof generation.

use std::collections::HashMap;
use crate::error::{Error, Result};
use crate::effect_adapters::codegen::riscv::{RiscVProgram, RiscVSectionType};

/// Represents a Succinct program converted from RISC-V
#[derive(Debug, Clone)]
pub struct SuccinctProgram {
    /// Program name
    pub name: String,
    /// Program code in Succinct format
    pub code: String,
    /// Circuit constraints for zero-knowledge proofs
    pub constraints: Vec<String>,
    /// Symbol table mapping names to addresses
    pub symbols: HashMap<String, usize>,
    /// Metadata for the program
    pub metadata: HashMap<String, String>,
}

impl SuccinctProgram {
    /// Create a new empty Succinct program
    pub fn new(name: impl Into<String>) -> Self {
        SuccinctProgram {
            name: name.into(),
            code: String::new(),
            constraints: Vec::new(),
            symbols: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a constraint to the program
    pub fn add_constraint(&mut self, constraint: impl Into<String>) -> &mut Self {
        self.constraints.push(constraint.into());
        self
    }
    
    /// Add a symbol to the program
    pub fn add_symbol(&mut self, name: impl Into<String>, address: usize) -> &mut Self {
        self.symbols.insert(name.into(), address);
        self
    }
    
    /// Add metadata to the program
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Converts RISC-V assembly to Succinct format
pub struct RiscVToSuccinctConverter {
    /// Instruction mapping from RISC-V to Succinct
    instruction_map: HashMap<String, String>,
    /// Include comments in the output
    include_comments: bool,
    /// Generate constraints for zero-knowledge proofs
    generate_constraints: bool,
}

impl RiscVToSuccinctConverter {
    /// Create a new converter with default settings
    pub fn new() -> Self {
        let mut converter = RiscVToSuccinctConverter {
            instruction_map: HashMap::new(),
            include_comments: true,
            generate_constraints: true,
        };
        
        // Initialize instruction mapping
        converter.initialize_mapping();
        
        converter
    }
    
    /// Create a new converter with custom settings
    pub fn with_options(include_comments: bool, generate_constraints: bool) -> Self {
        let mut converter = RiscVToSuccinctConverter {
            instruction_map: HashMap::new(),
            include_comments,
            generate_constraints,
        };
        
        // Initialize instruction mapping
        converter.initialize_mapping();
        
        converter
    }
    
    /// Initialize instruction mapping from RISC-V to Succinct
    fn initialize_mapping(&mut self) {
        // Core arithmetic and logical operations
        self.instruction_map.insert("add".to_string(), "add".to_string());
        self.instruction_map.insert("addi".to_string(), "addi".to_string());
        self.instruction_map.insert("sub".to_string(), "sub".to_string());
        self.instruction_map.insert("mul".to_string(), "mul".to_string());
        self.instruction_map.insert("div".to_string(), "div".to_string());
        self.instruction_map.insert("rem".to_string(), "rem".to_string());
        self.instruction_map.insert("and".to_string(), "and".to_string());
        self.instruction_map.insert("andi".to_string(), "andi".to_string());
        self.instruction_map.insert("or".to_string(), "or".to_string());
        self.instruction_map.insert("ori".to_string(), "ori".to_string());
        self.instruction_map.insert("xor".to_string(), "xor".to_string());
        self.instruction_map.insert("xori".to_string(), "xori".to_string());
        self.instruction_map.insert("sll".to_string(), "sll".to_string());
        self.instruction_map.insert("slli".to_string(), "slli".to_string());
        self.instruction_map.insert("srl".to_string(), "srl".to_string());
        self.instruction_map.insert("srli".to_string(), "srli".to_string());
        self.instruction_map.insert("sra".to_string(), "sra".to_string());
        self.instruction_map.insert("srai".to_string(), "srai".to_string());
        
        // Memory operations
        self.instruction_map.insert("lb".to_string(), "load_byte".to_string());
        self.instruction_map.insert("lh".to_string(), "load_half".to_string());
        self.instruction_map.insert("lw".to_string(), "load_word".to_string());
        self.instruction_map.insert("sb".to_string(), "store_byte".to_string());
        self.instruction_map.insert("sh".to_string(), "store_half".to_string());
        self.instruction_map.insert("sw".to_string(), "store_word".to_string());
        
        // Control flow
        self.instruction_map.insert("beq".to_string(), "branch_eq".to_string());
        self.instruction_map.insert("bne".to_string(), "branch_ne".to_string());
        self.instruction_map.insert("blt".to_string(), "branch_lt".to_string());
        self.instruction_map.insert("bge".to_string(), "branch_ge".to_string());
        self.instruction_map.insert("bltu".to_string(), "branch_ltu".to_string());
        self.instruction_map.insert("bgeu".to_string(), "branch_geu".to_string());
        self.instruction_map.insert("jal".to_string(), "jump_and_link".to_string());
        self.instruction_map.insert("jalr".to_string(), "jump_and_link_reg".to_string());
        
        // Special instructions
        self.instruction_map.insert("li".to_string(), "load_immediate".to_string());
        self.instruction_map.insert("mv".to_string(), "move".to_string());
        self.instruction_map.insert("nop".to_string(), "nop".to_string());
        self.instruction_map.insert("ret".to_string(), "return".to_string());
        self.instruction_map.insert("la".to_string(), "load_address".to_string());
    }
    
    /// Convert a RISC-V program to Succinct format
    pub fn convert(&self, program: &RiscVProgram) -> Result<SuccinctProgram> {
        let program_name = program.name.clone().unwrap_or_else(|| "unnamed".to_string());
        let mut succinct_program = SuccinctProgram::new(&program_name);
        
        // Add metadata
        succinct_program.add_metadata("entry_point", &program.entry_point);
        succinct_program.add_metadata("memory_size", &program.memory_size.to_string());
        
        // Copy symbols
        for (name, address) in &program.symbols {
            succinct_program.add_symbol(name, *address);
        }
        
        // Process each section
        let mut sections_code = String::new();
        
        for section in &program.sections {
            let section_code = match section.section_type {
                RiscVSectionType::Text => self.convert_text_section(section)?,
                RiscVSectionType::Data => self.convert_data_section(section)?,
                RiscVSectionType::RoData => self.convert_rodata_section(section)?,
                RiscVSectionType::Bss => self.convert_bss_section(section)?,
            };
            
            sections_code.push_str(&section_code);
            sections_code.push_str("\n\n");
        }
        
        succinct_program.code = sections_code;
        
        // Generate constraints if enabled
        if self.generate_constraints {
            self.generate_program_constraints(&mut succinct_program)?;
        }
        
        Ok(succinct_program)
    }
    
    /// Convert a text section (code) to Succinct format
    fn convert_text_section(&self, section: &crate::effect_adapters::codegen::riscv::RiscVProgramSection) -> Result<String> {
        let content = String::from_utf8_lossy(&section.content);
        let mut output = String::new();
        
        // Add section header
        output.push_str(&format!("# Section: {} ({})\n", section.name, "text"));
        output.push_str("begin_section text\n");
        
        // Process each line
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                output.push('\n');
                continue;
            }
            
            // Handle comments
            if trimmed.starts_with('#') {
                if self.include_comments {
                    output.push_str(&format!("// {}\n", &trimmed[1..].trim()));
                }
                continue;
            }
            
            // Handle labels
            if trimmed.ends_with(':') {
                let label = &trimmed[0..trimmed.len() - 1];
                output.push_str(&format!("{}:\n", label));
                continue;
            }
            
            // Handle directives
            if trimmed.starts_with('.') {
                // Skip most directives as they're handled differently in Succinct
                if trimmed.starts_with(".globl") {
                    let symbol = trimmed[6..].trim();
                    output.push_str(&format!("export {}\n", symbol));
                }
                continue;
            }
            
            // Handle instructions
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
            if parts.len() >= 1 {
                let instruction = parts[0];
                let operands = if parts.len() > 1 { parts[1] } else { "" };
                
                if let Some(succinct_instruction) = self.instruction_map.get(instruction) {
                    output.push_str(&format!("    {} {}\n", succinct_instruction, operands));
                } else {
                    // Unknown instruction, just copy it directly with a comment
                    output.push_str(&format!("    // Unknown instruction: {} {}\n", instruction, operands));
                }
            }
        }
        
        // Close section
        output.push_str("end_section\n");
        
        Ok(output)
    }
    
    /// Convert a data section to Succinct format
    fn convert_data_section(&self, section: &crate::effect_adapters::codegen::riscv::RiscVProgramSection) -> Result<String> {
        let content = String::from_utf8_lossy(&section.content);
        let mut output = String::new();
        
        // Add section header
        output.push_str(&format!("# Section: {} ({})\n", section.name, "data"));
        output.push_str("begin_section data\n");
        
        // Process each line
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                output.push('\n');
                continue;
            }
            
            // Handle comments
            if trimmed.starts_with('#') {
                if self.include_comments {
                    output.push_str(&format!("// {}\n", &trimmed[1..].trim()));
                }
                continue;
            }
            
            // Handle labels
            if trimmed.ends_with(':') {
                let label = &trimmed[0..trimmed.len() - 1];
                output.push_str(&format!("{}:\n", label));
                continue;
            }
            
            // Handle data directives
            if trimmed.starts_with('.') {
                let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let directive = parts[0];
                    let value = parts[1];
                    
                    match directive {
                        ".byte" => output.push_str(&format!("    db {}\n", value)),
                        ".half" => output.push_str(&format!("    dh {}\n", value)),
                        ".word" => output.push_str(&format!("    dw {}\n", value)),
                        ".string" | ".asciz" => output.push_str(&format!("    string {}\n", value)),
                        ".zero" => output.push_str(&format!("    reserve {}\n", value)),
                        _ => {
                            // Unknown directive, just comment it
                            if self.include_comments {
                                output.push_str(&format!("    // Unknown directive: {}\n", trimmed));
                            }
                        }
                    }
                }
                continue;
            }
        }
        
        // Close section
        output.push_str("end_section\n");
        
        Ok(output)
    }
    
    /// Convert a read-only data section to Succinct format
    fn convert_rodata_section(&self, section: &crate::effect_adapters::codegen::riscv::RiscVProgramSection) -> Result<String> {
        let content = String::from_utf8_lossy(&section.content);
        let mut output = String::new();
        
        // Add section header
        output.push_str(&format!("# Section: {} ({})\n", section.name, "rodata"));
        output.push_str("begin_section rodata\n");
        
        // Process each line similar to data section
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                output.push('\n');
                continue;
            }
            
            // Handle comments
            if trimmed.starts_with('#') {
                if self.include_comments {
                    output.push_str(&format!("// {}\n", &trimmed[1..].trim()));
                }
                continue;
            }
            
            // Handle labels
            if trimmed.ends_with(':') {
                let label = &trimmed[0..trimmed.len() - 1];
                output.push_str(&format!("{}:\n", label));
                continue;
            }
            
            // Handle data directives
            if trimmed.starts_with('.') {
                let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let directive = parts[0];
                    let value = parts[1];
                    
                    match directive {
                        ".byte" => output.push_str(&format!("    db {}\n", value)),
                        ".half" => output.push_str(&format!("    dh {}\n", value)),
                        ".word" => output.push_str(&format!("    dw {}\n", value)),
                        ".string" | ".asciz" => output.push_str(&format!("    string {}\n", value)),
                        _ => {
                            // Unknown directive, just comment it
                            if self.include_comments {
                                output.push_str(&format!("    // Unknown directive: {}\n", trimmed));
                            }
                        }
                    }
                }
                continue;
            }
        }
        
        // Close section
        output.push_str("end_section\n");
        
        Ok(output)
    }
    
    /// Convert a BSS section to Succinct format
    fn convert_bss_section(&self, section: &crate::effect_adapters::codegen::riscv::RiscVProgramSection) -> Result<String> {
        let content = String::from_utf8_lossy(&section.content);
        let mut output = String::new();
        
        // Add section header
        output.push_str(&format!("# Section: {} ({})\n", section.name, "bss"));
        output.push_str("begin_section bss\n");
        
        // Process each line
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                output.push('\n');
                continue;
            }
            
            // Handle comments
            if trimmed.starts_with('#') {
                if self.include_comments {
                    output.push_str(&format!("// {}\n", &trimmed[1..].trim()));
                }
                continue;
            }
            
            // Handle labels
            if trimmed.ends_with(':') {
                let label = &trimmed[0..trimmed.len() - 1];
                output.push_str(&format!("{}:\n", label));
                continue;
            }
            
            // Handle directives
            if trimmed.starts_with('.') {
                let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let directive = parts[0];
                    let value = parts[1];
                    
                    match directive {
                        ".zero" | ".space" => output.push_str(&format!("    reserve {}\n", value)),
                        ".comm" => {
                            let comm_parts: Vec<&str> = value.split(',').collect();
                            if comm_parts.len() >= 2 {
                                let symbol = comm_parts[0].trim();
                                let size = comm_parts[1].trim();
                                output.push_str(&format!("{}:\n", symbol));
                                output.push_str(&format!("    reserve {}\n", size));
                            }
                        },
                        _ => {
                            // Unknown directive, just comment it
                            if self.include_comments {
                                output.push_str(&format!("    // Unknown directive: {}\n", trimmed));
                            }
                        }
                    }
                }
                continue;
            }
        }
        
        // Close section
        output.push_str("end_section\n");
        
        Ok(output)
    }
    
    /// Generate constraints for zero-knowledge proofs
    fn generate_program_constraints(&self, program: &mut SuccinctProgram) -> Result<()> {
        // This is a placeholder for actual constraint generation
        // In a real implementation, this would analyze the code and generate
        // appropriate constraints for the ZK circuit
        
        program.add_constraint("// Automatically generated constraints for zero-knowledge proofs");
        program.add_constraint("constraint integrity(state) { state.valid }");
        program.add_constraint("constraint output_correctness(input, output) { output == expected_result(input) }");
        
        Ok(())
    }
    
    /// Write Succinct program to a file
    pub fn write_to_file(&self, program: &SuccinctProgram, path: &std::path::Path) -> Result<()> {
        use std::fs::File;
        use std::io::Write;
        
        let mut output = String::new();
        
        // Add program header
        output.push_str(&format!("// Succinct Program: {}\n", program.name));
        output.push_str("// Generated by RiscVToSuccinctConverter\n\n");
        
        // Add metadata
        output.push_str("// Metadata\n");
        for (key, value) in &program.metadata {
            output.push_str(&format!("// {}: {}\n", key, value));
        }
        output.push_str("\n");
        
        // Add main code
        output.push_str(&program.code);
        output.push_str("\n");
        
        // Add constraints if present
        if !program.constraints.is_empty() {
            output.push_str("// Circuit constraints\n");
            for constraint in &program.constraints {
                output.push_str(&format!("{}\n", constraint));
            }
        }
        
        // Write to file
        let mut file = File::create(path).map_err(|e| Error::IoError(e.to_string()))?;
        file.write_all(output.as_bytes()).map_err(|e| Error::IoError(e.to_string()))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect_adapters::codegen::riscv::{RiscVProgram, RiscVProgramSection, RiscVSectionType};
    
    fn create_test_program() -> RiscVProgram {
        let text_section = RiscVProgramSection {
            name: ".text".to_string(),
            section_type: RiscVSectionType::Text,
            content: r#"
# Test program
.globl main
main:
    addi sp, sp, -16
    sw ra, 12(sp)
    
    li a0, 10
    li a1, 20
    add a2, a0, a1
    
    lw ra, 12(sp)
    addi sp, sp, 16
    ret
"#.as_bytes().to_vec(),
            address: 0x1000,
            size: 100,
        };
        
        let data_section = RiscVProgramSection {
            name: ".data".to_string(),
            section_type: RiscVSectionType::Data,
            content: r#"
# Data section
test_data:
    .word 0x12345678
    .string "Hello, world!"
"#.as_bytes().to_vec(),
            address: 0x2000,
            size: 50,
        };
        
        let mut symbols = HashMap::new();
        symbols.insert("main".to_string(), 0x1000);
        symbols.insert("test_data".to_string(), 0x2000);
        
        RiscVProgram {
            name: Some("test_program".to_string()),
            entry_point: "main".to_string(),
            sections: vec![text_section, data_section],
            symbols,
            memory_size: 8192,
        }
    }
    
    #[test]
    fn test_converter() {
        let program = create_test_program();
        let converter = RiscVToSuccinctConverter::new();
        
        let result = converter.convert(&program);
        assert!(result.is_ok());
        
        let succinct_program = result.unwrap();
        assert_eq!(succinct_program.name, "test_program");
        assert!(succinct_program.code.contains("begin_section text"));
        assert!(succinct_program.code.contains("export main"));
        assert!(succinct_program.code.contains("add"));
        assert!(succinct_program.code.contains("begin_section data"));
        assert!(succinct_program.code.contains("dw 0x12345678"));
        assert!(!succinct_program.constraints.is_empty());
    }
} 