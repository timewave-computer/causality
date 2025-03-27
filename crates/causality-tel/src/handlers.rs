// TEL handler system
// Original file: src/tel/handlers.rs

//! TEL Handler interfaces for the three-layer effect architecture
//!
//! This module defines the handler interfaces for translating TEL operations into
//! concrete effects in the three-layer architecture.

use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use causality_types::Address;
use crate::resource::{ResourceId, Quantity};
use crate::domain::{DomainId, DomainRegistry};
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult,
    TransferEffect, QueryEffect, StorageEffect
};
use crate::script::{TelScript, TelOperation, TelOperationType};

/// Base trait for all TEL handlers
#[async_trait]
pub trait TelHandler: Send + Sync + Debug {
    /// Get the effect type this handler creates
    fn effect_type(&self) -> &'static str;
    
    /// Get the TEL function name this handler processes
    fn tel_function_name(&self) -> &'static str;
    
    /// Get the domain type this handler supports
    fn domain_type(&self) -> &'static str;
    
    /// Parse TEL parameters and create an effect
    async fn create_effect(&self, params: Value, context: &EffectContext) -> Result<Arc<dyn Effect>, anyhow::Error>;
    
    /// Check if this handler can handle the given TEL function
    fn can_handle(&self, function_name: &str, domain_type: &str) -> bool {
        self.tel_function_name() == function_name && self.domain_type() == domain_type
    }
}

/// A constraint-specific TEL handler for a particular effect type
#[async_trait]
pub trait ConstraintTelHandler<C: Effect + ?Sized>: TelHandler {
    /// Create a specific constrained effect
    async fn create_constrained_effect(&self, params: Value, context: &EffectContext) -> Result<Arc<C>, anyhow::Error>;
}

/// Handler for transfer effects
#[async_trait]
pub trait TransferTelHandler: ConstraintTelHandler<dyn TransferEffect> {
    /// Get the supported token types
    fn supported_tokens(&self) -> Vec<String>;
    
    /// Check if a token type is supported
    fn supports_token(&self, token_type: &str) -> bool {
        self.supported_tokens().iter().any(|t| t == token_type)
    }
}

/// Handler for storage effects
#[async_trait]
pub trait StorageTelHandler: ConstraintTelHandler<dyn StorageEffect> {
    /// Get the supported storage strategies
    fn supported_storage_strategies(&self) -> Vec<String>;
    
    /// Check if a storage strategy is supported
    fn supports_storage_strategy(&self, strategy: &str) -> bool {
        self.supported_storage_strategies().iter().any(|s| s == strategy)
    }
}

/// Handler for query effects
#[async_trait]
pub trait QueryTelHandler: ConstraintTelHandler<dyn QueryEffect> {
    /// Get the supported query types
    fn supported_query_types(&self) -> Vec<String>;
    
    /// Check if a query type is supported
    fn supports_query_type(&self, query_type: &str) -> bool {
        self.supported_query_types().iter().any(|q| q == query_type)
    }
}

/// Registry for TEL handlers
pub struct TelHandlerRegistry {
    /// Handlers indexed by (function_name, domain_type)
    handlers: HashMap<(String, String), Arc<dyn TelHandler>>,
    
    /// Domain registry for domain information
    domain_registry: Arc<DomainRegistry>,
}

impl TelHandlerRegistry {
    /// Create a new TEL handler registry
    pub fn new(domain_registry: Arc<DomainRegistry>) -> Self {
        Self {
            handlers: HashMap::new(),
            domain_registry,
        }
    }
    
    /// Register a TEL handler
    pub fn register_handler(&mut self, handler: Arc<dyn TelHandler>) {
        let key = (
            handler.tel_function_name().to_string(),
            handler.domain_type().to_string(),
        );
        self.handlers.insert(key, handler);
    }
    
    /// Get a handler for a specific function and domain
    pub fn get_handler(&self, function_name: &str, domain_type: &str) -> Option<Arc<dyn TelHandler>> {
        let key = (function_name.to_string(), domain_type.to_string());
        self.handlers.get(&key).cloned()
    }
    
    /// Find an appropriate handler for a function and domain
    pub fn find_handler_for_domain(&self, function_name: &str, domain_id: &DomainId) -> Option<Arc<dyn TelHandler>> {
        // Get domain info
        let domain_info = self.domain_registry.get_domain_info(domain_id)?;
        
        // Try to get a handler for this domain type
        let domain_type = domain_info.domain_type.to_string();
        self.get_handler(function_name, &domain_type)
    }
    
    /// Create an effect for a TEL function call
    pub async fn create_effect(
        &self,
        function_name: &str,
        params: Value,
        domain_id: &DomainId,
        context: &EffectContext,
    ) -> Result<Arc<dyn Effect>, anyhow::Error> {
        // Find a handler
        let handler = self.find_handler_for_domain(function_name, domain_id)
            .ok_or_else(|| anyhow::anyhow!(
                "No handler found for function '{}' on domain '{}'",
                function_name, domain_id
            ))?;
        
        // Create the effect
        handler.create_effect(params, context).await
    }
}

/// Base implementation for TEL handlers
#[derive(Debug)]
pub struct BaseTelHandler<C: Effect + ?Sized> {
    /// The effect type this handler creates
    effect_type: &'static str,
    
    /// The TEL function name this handler processes
    tel_function_name: &'static str,
    
    /// The domain type this handler supports
    domain_type: &'static str,
    
    /// Marker for the constraint type
    _constraint: PhantomData<C>,
}

impl<C: Effect + ?Sized> BaseTelHandler<C> {
    /// Create a new base TEL handler
    pub fn new(
        effect_type: &'static str,
        tel_function_name: &'static str,
        domain_type: &'static str,
    ) -> Self {
        Self {
            effect_type,
            tel_function_name,
            domain_type,
            _constraint: PhantomData,
        }
    }
}

#[async_trait]
impl<C: Effect + ?Sized> TelHandler for BaseTelHandler<C> {
    fn effect_type(&self) -> &'static str {
        self.effect_type
    }
    
    fn tel_function_name(&self) -> &'static str {
        self.tel_function_name
    }
    
    fn domain_type(&self) -> &'static str {
        self.domain_type
    }
    
    async fn create_effect(&self, _params: Value, _context: &EffectContext) -> Result<Arc<dyn Effect>, anyhow::Error> {
        Err(anyhow::anyhow!("BaseTelHandler cannot create effects directly - override this method"))
    }
}

/// Standard parameters for transfer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferParams {
    /// Source address
    pub from: String,
    /// Destination address
    pub to: String,
    /// Asset identifier
    pub asset: String,
    /// Transfer amount
    pub amount: String,
}

/// Standard parameters for storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageParams {
    /// Register ID to store data in
    pub register_id: String,
    /// Fields to store
    pub fields: Vec<String>,
    /// Storage strategy
    pub strategy: String,
}

/// Standard parameters for query operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    /// Function to call
    pub function: String,
    /// Arguments to the function
    pub args: Value,
}

/// TEL compiler interface
#[async_trait]
pub trait TelCompiler {
    /// Compile a TEL script into effects
    async fn compile(&self, script: &TelScript, context: &EffectContext) -> Result<Vec<Arc<dyn Effect>>, anyhow::Error>;
    
    /// Execute a TEL script
    async fn execute(&self, script: &TelScript, context: EffectContext) -> Result<Vec<EffectOutcome>, anyhow::Error>;
}

/// Standard implementation of TEL compiler
pub struct StandardTelCompiler {
    /// Registry of handlers
    handler_registry: Arc<TelHandlerRegistry>,
}

impl StandardTelCompiler {
    /// Create a new standard TEL compiler
    pub fn new(handler_registry: Arc<TelHandlerRegistry>) -> Self {
        Self {
            handler_registry,
        }
    }
    
    /// Convert a single TEL operation to effects
    async fn operation_to_effects(
        &self,
        operation: &TelOperation,
        context: &EffectContext,
    ) -> Result<Vec<Arc<dyn Effect>>, anyhow::Error> {
        match operation.operation_type {
            TelOperationType::Transfer | TelOperationType::Store | TelOperationType::Query => {
                // For atomic operations, find a handler and create the effect
                let domain_id = operation.domain_id.clone().ok_or_else(|| 
                    anyhow::anyhow!("Domain ID is required for operation: {}", operation.function_name))?;
                    
                let handler = self.handler_registry.find_handler_for_domain(
                    &operation.function_name, 
                    &domain_id
                ).ok_or_else(|| 
                    anyhow::anyhow!("No handler found for: {} on domain {}", 
                        operation.function_name, domain_id))?;
                
                let effect = handler.create_effect(operation.parameters.clone(), context).await?;
                Ok(vec![effect])
            },
            TelOperationType::Sequence => {
                // For sequence operations, process each child in order
                let mut effects = Vec::new();
                for child in &operation.children {
                    let child_effects = self.operation_to_effects(child, context).await?;
                    effects.extend(child_effects);
                }
                Ok(effects)
            },
            TelOperationType::Parallel => {
                // For parallel operations, process all children (in real implementation would be concurrent)
                let mut effects = Vec::new();
                for child in &operation.children {
                    let child_effects = self.operation_to_effects(child, context).await?;
                    effects.extend(child_effects);
                }
                Ok(effects)
            },
            TelOperationType::Conditional => {
                // For conditional operations, we need special handling
                // In a real implementation, this would create a conditional effect
                // For now, we'll just process the first child (condition) and return an error
                if operation.children.is_empty() {
                    return Err(anyhow::anyhow!("Conditional operation requires at least one child"));
                }
                
                Err(anyhow::anyhow!("Conditional operations not fully implemented yet"))
            },
            TelOperationType::Custom(_) => {
                // Custom operations would require special handling
                Err(anyhow::anyhow!("Custom operations not implemented yet"))
            },
        }
    }
}

#[async_trait]
impl TelCompiler for StandardTelCompiler {
    async fn compile(&self, script: &TelScript, context: &EffectContext) -> Result<Vec<Arc<dyn Effect>>, anyhow::Error> {
        // Basic implementation that processes each operation in the script
        
        // Check if the script is empty
        if script.operations().is_empty() {
            return Ok(Vec::new());
        }
        
        // Process each top-level operation
        let mut all_effects = Vec::new();
        for operation in script.operations() {
            let effects = self.operation_to_effects(operation, context).await?;
            all_effects.extend(effects);
        }
        
        Ok(all_effects)
    }
    
    async fn execute(&self, script: &TelScript, context: EffectContext) -> Result<Vec<EffectOutcome>, anyhow::Error> {
        // Compile the script into effects
        let effects = self.compile(script, &context).await?;
        
        // If no effects, return empty result
        if effects.is_empty() {
            return Ok(Vec::new());
        }
        
        // Create orchestrator for execution
        let validator = crate::effect::EffectValidator::new(
            // These would be provided in a real implementation
            Arc::new(crate::domain::DomainRegistry::new()),
            Arc::new(crate::resource::MockCapabilityRepository::new()),
            Arc::new(crate::resource::MockResourceAPI::new()),
        );
        let orchestrator = crate::effect::EffectOrchestrator::new(validator);
        
        // Execute the effects in sequence
        orchestrator.execute_sequence(effects, context).await
            .map_err(|e| anyhow::anyhow!("Effect execution error: {}", e))
    }
} 
