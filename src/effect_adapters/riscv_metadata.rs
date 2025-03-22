// RISC-V compatibility metadata for content-addressed code
//
// This module provides metadata for RISC-V compatibility checks,
// ensuring that code can be executed in a ZK virtual machine.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Metadata for RISC-V compatibility checks
///
/// This struct contains metadata about the RISC-V features and
/// constraints used by a piece of code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiscVMetadata {
    /// The RISC-V ISA extension used
    pub isa_extension: String,
    /// The RISC-V ISA version
    pub isa_version: String,
    /// The maximum memory usage in bytes
    pub max_memory: u64,
    /// The maximum stack depth
    pub max_stack_depth: u32,
    /// The maximum number of instructions
    pub max_instructions: u64,
    /// Whether floating point operations are used
    pub uses_floating_point: bool,
    /// Whether atomics are used
    pub uses_atomics: bool,
    /// Whether the code uses multiplication/division instructions
    pub uses_mul_div: bool,
    /// Custom properties for extensibility
    pub properties: HashMap<String, String>,
}

impl Default for RiscVMetadata {
    fn default() -> Self {
        Self {
            isa_extension: "RV32I".to_string(),
            isa_version: "2.1".to_string(),
            max_memory: 16 * 1024 * 1024, // 16 MB
            max_stack_depth: 1024,
            max_instructions: 1_000_000,
            uses_floating_point: false,
            uses_atomics: false,
            uses_mul_div: false,
            properties: HashMap::new(),
        }
    }
}

impl RiscVMetadata {
    /// Create new RISC-V metadata with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the RISC-V ISA extension
    pub fn with_isa_extension(mut self, extension: String) -> Self {
        self.isa_extension = extension;
        self
    }

    /// Set the RISC-V ISA version
    pub fn with_isa_version(mut self, version: String) -> Self {
        self.isa_version = version;
        self
    }

    /// Set the maximum memory usage
    pub fn with_max_memory(mut self, max_memory: u64) -> Self {
        self.max_memory = max_memory;
        self
    }

    /// Set the maximum stack depth
    pub fn with_max_stack_depth(mut self, max_stack_depth: u32) -> Self {
        self.max_stack_depth = max_stack_depth;
        self
    }

    /// Set the maximum number of instructions
    pub fn with_max_instructions(mut self, max_instructions: u64) -> Self {
        self.max_instructions = max_instructions;
        self
    }

    /// Set whether floating point operations are used
    pub fn with_floating_point(mut self, uses_floating_point: bool) -> Self {
        self.uses_floating_point = uses_floating_point;
        self
    }

    /// Set whether atomics are used
    pub fn with_atomics(mut self, uses_atomics: bool) -> Self {
        self.uses_atomics = uses_atomics;
        self
    }

    /// Set whether multiplication/division instructions are used
    pub fn with_mul_div(mut self, uses_mul_div: bool) -> Self {
        self.uses_mul_div = uses_mul_div;
        self
    }

    /// Add a custom property
    pub fn with_property(mut self, key: String, value: String) -> Self {
        self.properties.insert(key, value);
        self
    }
}

/// Compatibility checker for RISC-V metadata
///
/// This struct checks if RISC-V code is compatible with a
/// ZK virtual machine.
#[derive(Debug, Clone)]
pub struct RiscVCompatibilityChecker {
    /// The supported RISC-V ISA extensions
    supported_extensions: Vec<String>,
    /// The supported RISC-V ISA versions
    supported_versions: Vec<String>,
    /// The maximum memory allowed
    max_memory: u64,
    /// The maximum stack depth allowed
    max_stack_depth: u32,
    /// The maximum instructions allowed
    max_instructions: u64,
    /// Whether floating point operations are allowed
    allow_floating_point: bool,
    /// Whether atomics are allowed
    allow_atomics: bool,
    /// Whether multiplication/division instructions are allowed
    allow_mul_div: bool,
}

impl Default for RiscVCompatibilityChecker {
    fn default() -> Self {
        Self {
            supported_extensions: vec!["RV32I".to_string(), "RV32IM".to_string()],
            supported_versions: vec!["2.0".to_string(), "2.1".to_string()],
            max_memory: 32 * 1024 * 1024, // 32 MB
            max_stack_depth: 2048,
            max_instructions: 10_000_000,
            allow_floating_point: false,
            allow_atomics: false,
            allow_mul_div: true,
        }
    }
}

impl RiscVCompatibilityChecker {
    /// Create a new RISC-V compatibility checker
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a supported ISA extension
    pub fn add_supported_extension(mut self, extension: String) -> Self {
        self.supported_extensions.push(extension);
        self
    }

    /// Add a supported ISA version
    pub fn add_supported_version(mut self, version: String) -> Self {
        self.supported_versions.push(version);
        self
    }

    /// Set the maximum memory allowed
    pub fn with_max_memory(mut self, max_memory: u64) -> Self {
        self.max_memory = max_memory;
        self
    }

    /// Set the maximum stack depth allowed
    pub fn with_max_stack_depth(mut self, max_stack_depth: u32) -> Self {
        self.max_stack_depth = max_stack_depth;
        self
    }

    /// Set the maximum instructions allowed
    pub fn with_max_instructions(mut self, max_instructions: u64) -> Self {
        self.max_instructions = max_instructions;
        self
    }

    /// Set whether floating point operations are allowed
    pub fn allow_floating_point(mut self, allow: bool) -> Self {
        self.allow_floating_point = allow;
        self
    }

    /// Set whether atomics are allowed
    pub fn allow_atomics(mut self, allow: bool) -> Self {
        self.allow_atomics = allow;
        self
    }

    /// Set whether multiplication/division instructions are allowed
    pub fn allow_mul_div(mut self, allow: bool) -> Self {
        self.allow_mul_div = allow;
        self
    }

    /// Check if RISC-V metadata is compatible with this checker
    pub fn check_compatibility(&self, metadata: &RiscVMetadata) -> Result<(), String> {
        // Check ISA extension
        if !self.supported_extensions.contains(&metadata.isa_extension) {
            return Err(format!(
                "Unsupported ISA extension: {} (supported: {:?})",
                metadata.isa_extension, self.supported_extensions
            ));
        }

        // Check ISA version
        if !self.supported_versions.contains(&metadata.isa_version) {
            return Err(format!(
                "Unsupported ISA version: {} (supported: {:?})",
                metadata.isa_version, self.supported_versions
            ));
        }

        // Check memory usage
        if metadata.max_memory > self.max_memory {
            return Err(format!(
                "Memory usage exceeds limit: {} > {}",
                metadata.max_memory, self.max_memory
            ));
        }

        // Check stack depth
        if metadata.max_stack_depth > self.max_stack_depth {
            return Err(format!(
                "Stack depth exceeds limit: {} > {}",
                metadata.max_stack_depth, self.max_stack_depth
            ));
        }

        // Check instruction count
        if metadata.max_instructions > self.max_instructions {
            return Err(format!(
                "Instruction count exceeds limit: {} > {}",
                metadata.max_instructions, self.max_instructions
            ));
        }

        // Check floating point usage
        if metadata.uses_floating_point && !self.allow_floating_point {
            return Err(format!("Floating point operations are not allowed"));
        }

        // Check atomics usage
        if metadata.uses_atomics && !self.allow_atomics {
            return Err(format!("Atomic operations are not allowed"));
        }

        // Check mul/div usage
        if metadata.uses_mul_div && !self.allow_mul_div {
            return Err(format!("Multiplication/division operations are not allowed"));
        }

        Ok(())
    }
}

/// Trait for exporting RISC-V metadata
pub trait RiscVMetadataExporter {
    /// Export RISC-V metadata
    fn export_riscv_metadata(&self) -> RiscVMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riscv_metadata_default() {
        let metadata = RiscVMetadata::default();
        assert_eq!(metadata.isa_extension, "RV32I");
        assert_eq!(metadata.isa_version, "2.1");
        assert_eq!(metadata.max_memory, 16 * 1024 * 1024);
        assert_eq!(metadata.max_stack_depth, 1024);
        assert_eq!(metadata.max_instructions, 1_000_000);
        assert!(!metadata.uses_floating_point);
        assert!(!metadata.uses_atomics);
        assert!(!metadata.uses_mul_div);
        assert!(metadata.properties.is_empty());
    }

    #[test]
    fn test_riscv_metadata_builder() {
        let metadata = RiscVMetadata::new()
            .with_isa_extension("RV32IM".to_string())
            .with_isa_version("2.1".to_string())
            .with_max_memory(8 * 1024 * 1024)
            .with_max_stack_depth(512)
            .with_max_instructions(500_000)
            .with_floating_point(false)
            .with_atomics(false)
            .with_mul_div(true)
            .with_property("custom_property".to_string(), "value".to_string());

        assert_eq!(metadata.isa_extension, "RV32IM");
        assert_eq!(metadata.isa_version, "2.1");
        assert_eq!(metadata.max_memory, 8 * 1024 * 1024);
        assert_eq!(metadata.max_stack_depth, 512);
        assert_eq!(metadata.max_instructions, 500_000);
        assert!(!metadata.uses_floating_point);
        assert!(!metadata.uses_atomics);
        assert!(metadata.uses_mul_div);
        assert_eq!(
            metadata.properties.get("custom_property"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_compatibility_checker() {
        let checker = RiscVCompatibilityChecker::new();
        
        // Should be compatible with default metadata
        let metadata = RiscVMetadata::default();
        assert!(checker.check_compatibility(&metadata).is_ok());
        
        // Should reject incompatible metadata
        let incompatible = RiscVMetadata::new()
            .with_isa_extension("RV64I".to_string())
            .with_isa_version("2.1".to_string());
        assert!(checker.check_compatibility(&incompatible).is_err());
        
        // Should reject excessive memory usage
        let high_memory = RiscVMetadata::new()
            .with_max_memory(64 * 1024 * 1024);
        assert!(checker.check_compatibility(&high_memory).is_err());
        
        // Should reject floating point operations if not allowed
        let uses_fp = RiscVMetadata::new()
            .with_floating_point(true);
        assert!(checker.check_compatibility(&uses_fp).is_err());
    }
} 