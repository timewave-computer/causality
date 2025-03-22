// Content-addressed code system for Causality
//
// This module provides infrastructure for content-addressed code storage,
// loading, and execution.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::error::{Error, Result};
use crate::execution::context::{ExecutionContext, Value};

// Legacy hash implementation (re-exported from effect_adapters)
pub mod hash;

// Repository implementation (only when code-repo is enabled)
pub mod repository;
pub mod executor;
pub mod compatibility;
pub mod riscv_metadata;
pub mod definition;
pub mod registry;

// Re-export core types
pub use crate::effect_adapters::hash::Hash as ContentHash;
pub use crate::effect_adapters::hash::{HashAlgorithm, ContentHasher};
pub use crate::effect_adapters::repository::{CodeRepository, CodeEntry, CodeMetadata};
pub use executor::{ContentAddressableExecutor, ExecutionContext, Value};
pub use compatibility::CompatibilityChecker;
pub use riscv_metadata::{RiscVMetadata, RiscVCompatibilityChecker, RiscVMetadataExporter};
pub use definition::{CodeDefinition, CodeContent};
pub use registry::{NameRegistry, NameRecord};

// Temporal Effect Language (TEL) Integration

/// Represents a TEL program
#[derive(Debug, Clone)]
pub struct TelProgram {
    /// The content hash of the program
    pub hash: ContentHash,
    /// Program name
    pub name: Option<String>,
    /// The TEL source code
    pub source: String,
    /// Program AST (abstract syntax tree)
    pub ast: TelAst,
    /// Resource requirements
    pub resources: TelResourceRequirements,
    /// Effect dependencies
    pub effect_dependencies: Vec<String>,
}

/// TEL abstract syntax tree
#[derive(Debug, Clone)]
pub struct TelAst {
    /// Root node of the AST
    pub root: TelNode,
    /// All nodes in the program
    pub nodes: Vec<TelNode>,
}

/// TEL AST node
#[derive(Debug, Clone)]
pub struct TelNode {
    /// Node ID
    pub id: usize,
    /// Node type
    pub node_type: TelNodeType,
    /// Child node IDs
    pub children: Vec<usize>,
    /// Node attributes
    pub attributes: HashMap<String, Value>,
}

/// TEL node types
#[derive(Debug, Clone, PartialEq)]
pub enum TelNodeType {
    /// Program root
    Program,
    /// Effect application
    Effect,
    /// Control flow: Sequence
    Sequence,
    /// Control flow: Parallel
    Parallel,
    /// Control flow: Conditional
    Conditional,
    /// Control flow: Iteration
    Iteration,
    /// Variable definition
    VariableDefinition,
    /// Variable reference
    VariableReference,
    /// Literal value
    Literal,
    /// Binary operation
    BinaryOperation,
    /// Function call
    FunctionCall,
    /// Domain-specific extension
    DomainExtension(String),
}

/// TEL resource requirements
#[derive(Debug, Clone)]
pub struct TelResourceRequirements {
    /// Memory usage in bytes
    pub memory_bytes: usize,
    /// CPU usage in milliseconds
    pub cpu_millis: usize,
    /// I/O operations count
    pub io_operations: usize,
    /// Effect count
    pub effect_count: usize,
}

/// Interface for TEL integration
pub trait TelIntegrator: Send + Sync {
    /// Parse TEL source into an AST
    fn parse_tel(
        &self,
        source: &str,
    ) -> Result<TelAst>;
    
    /// Compile TEL source to content-addressed representation
    fn compile_tel(
        &self,
        source: &str,
        name: Option<String>,
    ) -> Result<TelProgram>;
    
    /// Validate TEL program against effects
    fn validate_tel(
        &self,
        program: &TelProgram,
    ) -> Result<bool>;
    
    /// Execute TEL program
    fn execute_tel(
        &self,
        program: &TelProgram,
        context: &mut ExecutionContext,
    ) -> Result<Value>;
    
    /// Get TEL program by hash
    fn get_tel_program(
        &self,
        hash: &ContentHash,
    ) -> Result<TelProgram>;
    
    /// List all registered TEL programs
    fn list_tel_programs(&self) -> Result<Vec<TelProgram>>;
}

/// Compiler for content-addressed TEL programs
pub struct ContentAddressedTelCompiler {
    /// The code repository
    repository: Arc<crate::effect_adapters::repository::CodeRepository>,
    /// Effect integrator
    effect_integrator: Arc<dyn crate::effect::EffectIntegrator>,
    /// Compiled programs
    programs: RwLock<HashMap<ContentHash, TelProgram>>,
}

impl ContentAddressedTelCompiler {
    /// Create a new content-addressed TEL compiler
    pub fn new(
        repository: Arc<crate::effect_adapters::repository::CodeRepository>,
        effect_integrator: Arc<dyn crate::effect::EffectIntegrator>,
    ) -> Self {
        ContentAddressedTelCompiler {
            repository,
            effect_integrator,
            programs: RwLock::new(HashMap::new()),
        }
    }
}

impl TelIntegrator for ContentAddressedTelCompiler {
    fn parse_tel(
        &self,
        source: &str,
    ) -> Result<TelAst> {
        // In a real implementation, this would actually parse the TEL source
        // Here we just return a placeholder empty AST
        Ok(TelAst {
            root: TelNode {
                id: 0,
                node_type: TelNodeType::Program,
                children: vec![],
                attributes: HashMap::new(),
            },
            nodes: vec![],
        })
    }
    
    fn compile_tel(
        &self,
        source: &str,
        name: Option<String>,
    ) -> Result<TelProgram> {
        // Parse the source
        let ast = self.parse_tel(source)?;
        
        // Calculate a hash of the source
        let hash = ContentHash::new(source.as_bytes());
        
        // Create a program
        let program = TelProgram {
            hash,
            name,
            source: source.to_string(),
            ast,
            resources: TelResourceRequirements {
                memory_bytes: 0,
                cpu_millis: 0,
                io_operations: 0,
                effect_count: 0,
            },
            effect_dependencies: vec![],
        };
        
        // Store the program
        let mut programs = self.programs.write().map_err(|_| 
            Error::LockError("Failed to acquire programs write lock".to_string()))?;
        programs.insert(program.hash.clone(), program.clone());
        
        Ok(program)
    }
    
    fn validate_tel(
        &self,
        program: &TelProgram,
    ) -> Result<bool> {
        // In a real implementation, this would validate the program against available effects
        // Here we just return true as a placeholder
        Ok(true)
    }
    
    fn execute_tel(
        &self,
        program: &TelProgram,
        context: &mut ExecutionContext,
    ) -> Result<Value> {
        // In a real implementation, this would execute the program
        // Here we just return a null value as a placeholder
        Ok(Value::Null)
    }
    
    fn get_tel_program(
        &self,
        hash: &ContentHash,
    ) -> Result<TelProgram> {
        let programs = self.programs.read().map_err(|_| 
            Error::LockError("Failed to acquire programs read lock".to_string()))?;
        
        programs.get(hash).cloned().ok_or_else(|| 
            Error::NotFound(format!("TEL program not found: {}", hash)))
    }
    
    fn list_tel_programs(&self) -> Result<Vec<TelProgram>> {
        let programs = self.programs.read().map_err(|_| 
            Error::LockError("Failed to acquire programs read lock".to_string()))?;
        
        Ok(programs.values().cloned().collect())
    }
} 