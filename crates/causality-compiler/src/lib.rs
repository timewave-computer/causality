//! Causality Compiler: Parse → Check → Compile
//!
//! Minimal compiler implementation following the three-layer architecture.
//! Compiles Lisp source code through S-expressions and lambda calculus
//! to verifiable register machine instructions.

#![allow(clippy::result_large_err)]

pub mod error;
pub mod pipeline;
pub mod checker;
pub mod artifact;
pub mod enhanced_pipeline;
pub mod types;

#[cfg(test)]
pub mod benchmarks;

// Re-export key types for convenience
pub use pipeline::{
    compile, compile_expression, compile_program,
    CompiledArtifact, SExpression
};
pub use error::{CompileError, CompileResult};
pub use checker::{check_sexpr, check_linearity, TypeEnvironment};
pub use artifact::{
    ContentAddressedArtifact, ContentHash, ArtifactCache,
    build_artifact, verify_artifact
};
pub use enhanced_pipeline::{
    EnhancedCompilerPipeline, CompiledProgram, CompilationMetadata,
    CodeGenerator, InstructionOptimizer, OptimizationPass
};
pub use types::CompileResult as CompileResultEnum;

// Export TEG compilation functions
//pub use {compile_teg_definition, CompiledTeg};

/// Minimal test function for E2E validation
/// Compiles a simple expression and returns the instructions
pub fn minimal_test() -> Result<Vec<causality_core::machine::instruction::Instruction>, CompileError> {
    let source = "(pure 42)";
    let artifact = compile(source)?;
    Ok(artifact.instructions)
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_full_pipeline() {
        // Test the complete Parse → Check → Compile flow
        let source = "(pure 42)";
        
        // Parse
        let artifact = compile(source).unwrap();
        assert_eq!(artifact.source, source);
        
        // Check (currently placeholder)
        assert!(check_sexpr(&artifact.sexpr).is_ok());
        
        // Verify compilation produced instructions
        assert!(!artifact.instructions.is_empty());
    }
    
    #[test]
    fn test_content_addressing() {
        let source = "(pure 123)"; // Use a simpler expression without arithmetic
        
        // Build content-addressed artifact
        let artifact = build_artifact(source).unwrap();
        
        // Verify hash consistency
        assert!(verify_artifact(&artifact));
        
        // Test caching
        let mut cache = ArtifactCache::new();
        let hash = artifact.hash().clone();
        cache.insert(artifact);
        
        assert!(cache.contains(&hash));
        let retrieved = cache.get(&hash).unwrap();
        assert_eq!(retrieved.source(), source);
    }
    
    #[test]
    fn test_error_handling() {
        // Test parse error
        let result = compile("(unclosed list");
        assert!(result.is_err());
        
        // Test invalid syntax  
        let result = compile("(pure)"); // missing argument
        assert!(result.is_err());
    }
}

// TEG compilation support for bridge tests
use std::path::PathBuf;
use std::collections::HashMap;
use causality_core::system::content_addressing::EntityId;

/// Compiled TEG artifact for bridge workflows
#[derive(Debug, Clone)]
pub struct CompiledTeg {
    pub id: EntityId,
    pub name: String,
    pub base_dir: PathBuf,
    pub expressions: HashMap<String, String>,
    pub handlers: HashMap<String, String>,
    pub subgraphs: HashMap<String, String>,
}

/// Compile a TEG definition from file
pub fn compile_teg_definition(path: &PathBuf, name: Option<String>) -> Result<CompiledTeg, CompileError> {
    // For now, return a mock implementation since TEG compilation isn't fully implemented
    let teg_name = name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });
    
    Ok(CompiledTeg {
        id: EntityId::new([1u8; 32]),
        name: teg_name,
        base_dir: path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf(),
        expressions: HashMap::new(),
        handlers: HashMap::new(),
        subgraphs: HashMap::new(),
    })
} 