//! ZK Virtual Machine Compiler
//!
//! This module implements the translation of Temporal Effect Graphs (TEGs)
//! to RISC-V or other VM code for zero-knowledge proof generation.
//! It provides interfaces for compiling TEGs and interacting with ZK provers.

// TODO generation.md Step 16: Implement ZK compilation features
// - Implement TEG -> RISC-V (or other VM) compilation
// - Implement prover system interaction
// - Potentially incorporate valence-coprocessor prover interaction

use std::path::PathBuf;
use std::collections::HashMap;
use thiserror::Error;

use causality_ir::graph::TemporalEffectGraph;

/// Errors related to ZK compilation and proving
#[derive(Error, Debug)]
pub enum ZkError {
    #[error("Compilation error: {0}")]
    CompilationError(String),
    
    #[error("Prover error: {0}")]
    ProverError(String),
    
    #[error("Invalid TEG: {0}")]
    InvalidTeg(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("System error: {0}")]
    SystemError(String),
}

/// Result type for ZK operations
pub type ZkResult<T> = Result<T, ZkError>;

/// Compilation options for the ZkVM compiler
#[derive(Debug, Clone)]
pub struct CompilationOptions {
    /// Output directory for compiled artifacts
    pub output_dir: Option<PathBuf>,
    
    /// Optimization level (0-3)
    pub optimization_level: u8,
    
    /// Whether to generate debug information
    pub debug_info: bool,
}

impl Default for CompilationOptions {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimization_level: 1,
            debug_info: false,
        }
    }
}

/// Trait for compiling a TEG to VM code
pub trait TegVmCompiler {
    /// Compile a TEG to VM code
    fn compile(&self, teg: &TemporalEffectGraph, options: Option<CompilationOptions>) -> ZkResult<CompiledProgram>;
}

/// Compiled program output
#[derive(Debug)]
pub struct CompiledProgram {
    /// Compiled binary
    pub binary: Vec<u8>,
    
    /// Program metadata
    pub metadata: HashMap<String, String>,
    
    /// Program inputs (public and private)
    pub inputs: ProgramInputs,
}

/// Program inputs definition
#[derive(Debug)]
pub struct ProgramInputs {
    /// Public inputs
    pub public: Vec<ProgramInput>,
    
    /// Private inputs
    pub private: Vec<ProgramInput>,
}

/// Program input definition
#[derive(Debug)]
pub struct ProgramInput {
    /// Input name
    pub name: String,
    
    /// Input type
    pub input_type: String,
    
    /// Input description
    pub description: String,
}

/// Trait for interacting with a ZK prover
pub trait ZkProver {
    /// Generate a proof for a compiled program with the given inputs
    fn prove(&self, program: &CompiledProgram, inputs: HashMap<String, Vec<u8>>) -> ZkResult<ZkProof>;
    
    /// Verify a proof
    fn verify(&self, proof: &ZkProof, public_inputs: HashMap<String, Vec<u8>>) -> ZkResult<bool>;
}

/// Zero-knowledge proof output
#[derive(Debug)]
pub struct ZkProof {
    /// The proof data
    pub proof_data: Vec<u8>,
    
    /// Public inputs used in the proof
    pub public_inputs: HashMap<String, Vec<u8>>,
    
    /// Proof metadata
    pub metadata: HashMap<String, String>,
}

// TODO: Add concrete implementations of TegVmCompiler and ZkProver 