//! Causality Compiler: Parse → Check → Compile
//!
//! Minimal compiler implementation following the three-layer architecture.
//! Compiles Lisp source code through S-expressions and lambda calculus
//! to verifiable register machine instructions using the new 5-instruction API.

#![allow(clippy::result_large_err)]

pub mod almanac_runtime;
pub mod almanac_schema;
pub mod artifact;
pub mod checker;
pub mod error;
pub mod error_handling;
pub mod event_storage;
pub mod observability;
pub mod pipeline;
pub mod proof_primitives;
pub mod query_primitives;
pub mod state_analysis;
pub mod storage_backend;
pub mod storage_layout;
pub mod traverse_almanac_integration;
pub mod traverse_integration;
pub mod types;
pub mod valence_analysis;
pub mod valence_coprocessor_integration;
pub mod valence_state_persistence;

#[cfg(test)]
pub mod benchmarks;
#[cfg(test)]
pub mod sexpr_spec_tests;
#[cfg(test)]
pub mod storage_integration_tests;

// Re-export key types for convenience
pub use artifact::{
    build_artifact, verify_artifact, ArtifactCache, ContentAddressedArtifact,
    ContentHash,
};
pub use checker::{check_linearity, check_sexpr, TypeEnvironment};
pub use error::{CompileError, CompileResult};
pub use pipeline::{compile, compile_expression, CompiledArtifact, SExpression};
// pub use enhanced_pipeline::{
//     EnhancedCompilerPipeline, CompiledProgram, CompilationMetadata,
//     CodeGenerator, InstructionOptimizer, OptimizationPass
// };
pub use types::CompileResult as CompileResultEnum;

/// Minimal test function for E2E validation
/// Compiles a simple expression and returns the instructions
pub fn minimal_test(
) -> Result<Vec<causality_core::machine::instruction::Instruction>, CompileError> {
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
use causality_core::system::content_addressing::EntityId;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Compiled TEG artifact for bridge workflows
#[derive(Debug, Clone)]
pub struct CompiledTeg {
    pub id: EntityId,
    pub name: String,
    pub base_dir: PathBuf,
    pub expressions: BTreeMap<String, String>,
    pub handlers: BTreeMap<String, String>,
    pub subgraphs: BTreeMap<String, String>,
}

/// Compile a TEG definition from file
pub fn compile_teg_definition(
    path: &Path,
    name: Option<String>,
) -> Result<CompiledTeg, CompileError> {
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
        expressions: BTreeMap::new(),
        handlers: BTreeMap::new(),
        subgraphs: BTreeMap::new(),
    })
}
