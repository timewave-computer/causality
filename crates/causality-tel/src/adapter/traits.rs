// TEL adapter trait definitions
// Original file: src/tel/adapter/traits.rs

// Core traits for domain adapters
use std::fmt::Debug;
use std::sync::Arc;
use std::collections::HashMap;
use std::any::Any;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use causality_tel::{Effect, DomainId, ResourceId};
use causality_tel::{TelError, TelResult};
use causality_tel::ValidationResult;
use causality_tel::AdapterMetadata;

/// Context information for effect compilation
#[derive(Debug, Clone)]
pub struct CompilerContext {
    /// Domain-specific parameters
    pub parameters: HashMap<String, Value>,
    /// Resource identifiers referenced in the effect
    pub resource_ids: Vec<ResourceId>,
    /// Chain-specific context (e.g., current block number)
    pub chain_context: HashMap<String, Value>,
    /// Compilation options
    pub options: CompilationOptions,
}

impl Default for CompilerContext {
    fn default() -> Self {
        Self {
            parameters: HashMap::new(),
            resource_ids: Vec::new(),
            chain_context: HashMap::new(),
            options: CompilationOptions::default(),
        }
    }
}

/// Options for effect compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationOptions {
    /// Whether to optimize the compilation
    pub optimize: bool,
    /// Whether to validate the effect before compilation
    pub validate: bool,
    /// Gas limit for the transaction (if applicable)
    pub gas_limit: Option<u64>,
    /// Whether to simulate the transaction before sending
    pub dry_run: bool,
    /// Maximum transaction size (in bytes)
    pub max_tx_size: Option<usize>,
}

impl Default for CompilationOptions {
    fn default() -> Self {
        Self {
            optimize: true,
            validate: true,
            gas_limit: None,
            dry_run: false,
            max_tx_size: None,
        }
    }
}

/// Result of compiling an effect
#[derive(Debug, Clone)]
pub struct CompilationResult<T> {
    /// The compiled output
    pub output: T,
    /// The domain this compilation is for
    pub domain: DomainId,
    /// Estimated cost (e.g., gas, compute units)
    pub estimated_cost: Option<u64>,
    /// Size of the compiled output (in bytes)
    pub size: usize,
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// Trait for compiling effects to domain-specific formats
pub trait EffectCompiler: Send + Sync + Debug + Any {
    /// The output type produced by this compiler
    type Output: Clone + Debug;
    
    /// Compile an effect into a domain-specific format
    fn compile(
        &self, 
        effect: &Effect, 
        context: &CompilerContext
    ) -> TelResult<CompilationResult<Self::Output>>;
    
    /// Validate whether an effect can be compiled for this domain
    fn validate(&self, effect: &Effect, context: &CompilerContext) -> TelResult<()>;
    
    /// Get the domain ID this compiler handles
    fn domain_id(&self) -> &DomainId;
    
    /// Check if this compiler supports the given effect type
    fn supports_effect(&self, effect: &Effect) -> bool;
    
    /// Estimate the cost of an effect without full compilation
    fn estimate_cost(&self, effect: &Effect, context: &CompilerContext) -> TelResult<u64> {
        // Default implementation does full compilation and returns the estimate
        match self.compile(effect, context) {
            Ok(result) => match result.estimated_cost {
                Some(cost) => Ok(cost),
                None => Err(TelError::UnsupportedOperation(
                    "Cost estimation not supported by this adapter".to_string(),
                )),
            },
            Err(err) => Err(err),
        }
    }
    
    /// Split an effect into multiple domain-specific effects if needed
    fn split_effect(&self, effect: &Effect) -> TelResult<Vec<Effect>> {
        // Default implementation returns the effect as is
        Ok(vec![effect.clone()])
    }
    
    /// Get metadata about this compiler
    fn metadata(&self) -> HashMap<String, Value> {
        // Default implementation returns minimal metadata
        let mut metadata = HashMap::new();
        metadata.insert("domain".to_string(), Value::String(self.domain_id().clone()));
        metadata
    }
    
    /// Get a reference to self as Any for downcasting
    fn as_any(&self) -> &dyn Any;
} 
