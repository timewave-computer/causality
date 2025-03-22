//! Zero-Knowledge Effect Adapter Module
//!
//! This module provides functionality for creating and working with
//! zero-knowledge proof effect adapters for the Causality system.
//! It includes error handling, logging, validation, and code generation
//! components for ZK operations.

pub mod error;
pub mod log;
pub mod effects;

use std::sync::Arc;
use error::{Result, ZkError, ValidationErrors};
use log::{ZkLogger, OperationType};
use crate::effect::Effect;
#[cfg(feature = "domain")]
use crate::domain_adapters::succinct::bridge::SuccinctVmBridge;
#[cfg(feature = "domain")]
use crate::effect_adapters::codegen::zk::ZkCodeGenerator;
use crate::zk::{Proof, RiscVProgram, Witness, ZkVirtualMachine};

// Re-export important types from effects
pub use effects::{
    CompileZkProgramEffect, 
    GenerateZkWitnessEffect, 
    GenerateZkProofEffect, 
    VerifyZkProofEffect,
    compile_zk_program,
    generate_zk_witness,
    generate_zk_proof,
    verify_zk_proof
};

// Use the ZkEffectAdapter from the effects module
pub use effects::ZkEffectAdapter;
pub use effects::ZkEffectHandler;

/// Configuration options for ZK operations
#[derive(Debug, Clone)]
pub struct ZkEffectAdapterConfig {
    /// Enable debug mode for more detailed logging
    pub debug_mode: bool,
    /// Enable validation of inputs
    pub validation_enabled: bool,
    /// Code generation output path
    pub code_gen_output_path: Option<String>,
    /// Maximum memory usage in bytes
    pub max_memory_usage: Option<usize>,
}

impl Default for ZkEffectAdapterConfig {
    fn default() -> Self {
        ZkEffectAdapterConfig {
            debug_mode: false,
            validation_enabled: true,
            code_gen_output_path: None,
            max_memory_usage: Some(1024 * 1024 * 128), // 128 MB default
        }
    }
} 