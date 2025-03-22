// Three-Layer Effect Architecture
//
// This module implements the three-layer effect architecture as described in ADR-023,
// consisting of Algebraic Effect Layer, Effect Constraints Layer, and Domain Implementation Layer.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::effect::{EffectResult, EffectError, EffectOutcome, EffectContext};
use crate::resource::{ResourceId, ResourceCapability, Right};
use crate::address::Address;
use crate::tel::TelScript;
use crate::domain::DomainId;

//
// Layer 1: Algebraic Effect Layer
//

/// Core trait representing an algebraic effect in the system
/// This is the base trait for all effects in the architecture
#[async_trait]
pub trait AlgebraicEffect: Send + Sync {
    /// Get a unique identifier for this effect type
    fn effect_type(&self) -> &'static str;
    
    /// Get a human-readable name for this effect
    fn name(&self) -> &str;
    
    /// Get a description of what this effect does
    fn description(&self) -> &str;
    
    /// Get the resource IDs this effect operates on
    fn resource_ids(&self) -> Vec<ResourceId>;
    
    /// Get the primary domain this effect targets
    fn primary_domain(&self) -> Option<DomainId>;
    
    /// Get the parameters for this effect
    fn parameters(&self) -> HashMap<String, serde_json::Value>;
    
    /// Get the required capabilities to execute this effect
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)>;
    
    /// Execute the effect
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Validate that the effect can be executed with the given context
    async fn validate(&self, context: &EffectContext) -> EffectResult<()>;
    
    /// Get the relevant constraint traits this effect implements
    fn constraint_traits(&self) -> Vec<&'static str>;
    
    /// Check if this effect satisfies a particular constraint
    fn satisfies_constraint(&self, constraint: &str) -> bool;
    
    /// Get TEL implementation hints for this effect
    fn tel_hints(&self) -> Option<TelHints>;
}

/// Hints for TEL code generation/execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelHints {
    /// Target domain type
    pub domain_type: String,
    
    /// Function name pattern in TEL
    pub function_pattern: String,
    
    /// Parameter mapping hints
    pub parameter_mappings: HashMap<String, String>,
    
    /// Required imports/includes for TEL
    pub required_imports: Vec<String>,
    
    /// Additional metadata for code generation
    pub metadata: HashMap<String, String>,
}

//
// Layer 2: Effect Constraints Layer
//

/// A trait for effects that transfer resources between addresses
#[async_trait]
pub trait TransferEffect: AlgebraicEffect {
    /// Get the source address for the transfer
    fn source(&self) -> &Address;
    
    /// Get the destination address for the transfer
    fn destination(&self) -> &Address;
    
    /// Get the resource being transferred
    fn resource_id(&self) -> &ResourceId;
    
    /// Get the quantity being transferred (for fungible resources)
    fn quantity(&self) -> Option<u128>;
    
    /// Validate the transfer is possible
    async fn validate_transfer(&self, context: &EffectContext) -> EffectResult<()>;
}

/// A trait for effects that deposit resources to an address
#[async_trait]
pub trait DepositEffect: AlgebraicEffect {
    /// Get the destination address for the deposit
    fn destination(&self) -> &Address;
    
    /// Get the resource being deposited
    fn resource_id(&self) -> &ResourceId;
    
    /// Get the quantity being deposited (for fungible resources)
    fn quantity(&self) -> Option<u128>;
    
    /// Get the source of the deposit (if applicable)
    fn source(&self) -> Option<&Address>;
    
    /// Validate the deposit is possible
    async fn validate_deposit(&self, context: &EffectContext) -> EffectResult<()>;
}

/// A trait for effects that withdraw resources from an address
#[async_trait]
pub trait WithdrawEffect: AlgebraicEffect {
    /// Get the source address for the withdrawal
    fn source(&self) -> &Address;
    
    /// Get the resource being withdrawn
    fn resource_id(&self) -> &ResourceId;
    
    /// Get the quantity being withdrawn (for fungible resources)
    fn quantity(&self) -> Option<u128>;
    
    /// Get the destination of the withdrawal (if applicable)
    fn destination(&self) -> Option<&Address>;
    
    /// Validate the withdrawal is possible
    async fn validate_withdrawal(&self, context: &EffectContext) -> EffectResult<()>;
}

/// A trait for effects that store data on-chain
#[async_trait]
pub trait StorageEffect: AlgebraicEffect {
    /// Get the resource ID being stored
    fn resource_id(&self) -> &ResourceId;
    
    /// Get the domain where data is being stored
    fn storage_domain(&self) -> &DomainId;
    
    /// Get the fields being stored
    fn fields(&self) -> &HashSet<String>;
    
    /// Get the visibility of the stored data
    fn visibility(&self) -> StorageVisibility;
    
    /// Validate the storage operation is possible
    async fn validate_storage(&self, context: &EffectContext) -> EffectResult<()>;
}

/// A trait for effects that query on-chain data
#[async_trait]
pub trait QueryEffect: AlgebraicEffect {
    /// Get the resource ID being queried
    fn resource_id(&self) -> &ResourceId;
    
    /// Get the domain where data is being queried
    fn query_domain(&self) -> &DomainId;
    
    /// Get the query parameters
    fn query_parameters(&self) -> &HashMap<String, serde_json::Value>;
    
    /// Validate the query operation is possible
    async fn validate_query(&self, context: &EffectContext) -> EffectResult<()>;
}

/// Visibility options for storage effects
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageVisibility {
    /// Data is publicly visible
    Public,
    
    /// Data is private but can be selectively revealed
    Private,
    
    /// Data is encrypted
    Encrypted,
    
    /// Only commitments to the data are stored
    CommitmentOnly,
}

//
// Layer 3: Domain Implementation Layer
//

/// A runtime for executing effects with domain-specific implementations
pub struct EffectRuntime {
    /// Domain adapters for executing effects on specific domains
    domain_adapters: HashMap<DomainId, Arc<dyn DomainAdapter>>,
    
    /// TEL compiler for generating domain-specific code
    tel_compiler: Arc<dyn TelCompiler>,
    
    /// Fallback handlers for effects without domain implementations
    fallback_handlers: HashMap<&'static str, Arc<dyn FallbackHandler>>,
}

impl EffectRuntime {
    /// Create a new effect runtime
    pub fn new(
        tel_compiler: Arc<dyn TelCompiler>,
    ) -> Self {
        Self {
            domain_adapters: HashMap::new(),
            tel_compiler,
            fallback_handlers: HashMap::new(),
        }
    }
    
    /// Register a domain adapter
    pub fn register_domain_adapter(&mut self, domain_id: DomainId, adapter: Arc<dyn DomainAdapter>) {
        self.domain_adapters.insert(domain_id, adapter);
    }
    
    /// Register a fallback handler for an effect type
    pub fn register_fallback_handler(&mut self, effect_type: &'static str, handler: Arc<dyn FallbackHandler>) {
        self.fallback_handlers.insert(effect_type, handler);
    }
    
    /// Execute an effect
    pub async fn execute_effect(&self, effect: Arc<dyn AlgebraicEffect>, context: EffectContext) 
        -> EffectResult<EffectOutcome> 
    {
        // Validate effect
        effect.validate(&context).await?;
        
        // Check if there's a domain-specific implementation
        if let Some(domain_id) = effect.primary_domain() {
            if let Some(adapter) = self.domain_adapters.get(&domain_id) {
                if adapter.supports_effect_type(effect.effect_type()) {
                    return adapter.execute_effect(effect.clone(), context).await;
                }
            }
        }
        
        // Try using TEL for execution
        if let Some(tel_hints) = effect.tel_hints() {
            return self.execute_with_tel(effect.clone(), context, tel_hints).await;
        }
        
        // Try fallback handler
        if let Some(handler) = self.fallback_handlers.get(effect.effect_type()) {
            return handler.handle_effect(effect.clone(), context).await;
        }
        
        // No implementation found
        Err(EffectError::NotImplemented)
    }
    
    /// Execute an effect using TEL
    async fn execute_with_tel(
        &self,
        effect: Arc<dyn AlgebraicEffect>,
        context: EffectContext,
        hints: TelHints,
    ) -> EffectResult<EffectOutcome> {
        // Generate TEL code
        let tel_code = self.tel_compiler.generate_code(effect.clone(), &hints)
            .map_err(|e| EffectError::ExecutionError(format!("TEL code generation failed: {}", e)))?;
        
        // Get domain adapter
        let domain_id = effect.primary_domain()
            .ok_or_else(|| EffectError::ExecutionError("No primary domain specified".to_string()))?;
            
        let adapter = self.domain_adapters.get(&domain_id)
            .ok_or_else(|| EffectError::ExecutionError(format!("No domain adapter for domain: {:?}", domain_id)))?;
        
        // Execute the TEL code on the domain
        adapter.execute_tel(tel_code, context).await
    }
}

/// Adapter for executing effects on a specific domain
#[async_trait]
pub trait DomainAdapter: Send + Sync {
    /// Get the domain ID this adapter handles
    fn domain_id(&self) -> &DomainId;
    
    /// Check if this adapter supports a specific effect type
    fn supports_effect_type(&self, effect_type: &str) -> bool;
    
    /// Execute an effect directly
    async fn execute_effect(&self, effect: Arc<dyn AlgebraicEffect>, context: EffectContext) 
        -> EffectResult<EffectOutcome>;
    
    /// Execute TEL code
    async fn execute_tel(&self, tel_code: TelScript, context: EffectContext) 
        -> EffectResult<EffectOutcome>;
    
    /// Check if an effect can be executed on this domain
    async fn can_execute(&self, effect: Arc<dyn AlgebraicEffect>) -> bool;
}

/// Fallback handler for effects without domain-specific implementations
#[async_trait]
pub trait FallbackHandler: Send + Sync {
    /// Get the effect type this handler can process
    fn effect_type(&self) -> &'static str;
    
    /// Handle an effect
    async fn handle_effect(&self, effect: Arc<dyn AlgebraicEffect>, context: EffectContext) 
        -> EffectResult<EffectOutcome>;
}

/// Compiler for generating TEL code from effects
#[async_trait]
pub trait TelCompiler: Send + Sync {
    /// Generate TEL code for an effect
    fn generate_code(&self, effect: Arc<dyn AlgebraicEffect>, hints: &TelHints) 
        -> Result<TelScript, String>;
    
    /// Check if an effect can be compiled to TEL
    fn can_compile(&self, effect: Arc<dyn AlgebraicEffect>) -> bool;
}

/// A simple in-memory TEL compiler
pub struct SimpleTelCompiler;

impl SimpleTelCompiler {
    /// Create a new simple TEL compiler
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TelCompiler for SimpleTelCompiler {
    fn generate_code(&self, effect: Arc<dyn AlgebraicEffect>, hints: &TelHints) 
        -> Result<TelScript, String> 
    {
        // Start building the TEL code
        let mut code = String::new();
        
        // Add imports
        for import in &hints.required_imports {
            code.push_str(&format!("import {}\n", import));
        }
        code.push('\n');
        
        // Add function definition
        code.push_str(&format!("function {}() {{\n", hints.function_pattern));
        
        // Add parameters
        let params = effect.parameters();
        for (name, value) in params {
            if let Some(mapping) = hints.parameter_mappings.get(&name) {
                // This is a simple implementation - in a real compiler, we'd have
                // more sophisticated code generation
                code.push_str(&format!("  let {} = {};\n", 
                    mapping, 
                    serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string())
                ));
            }
        }
        
        // Add resource IDs
        for (i, resource_id) in effect.resource_ids().iter().enumerate() {
            code.push_str(&format!("  let resource{} = \"{}\";\n", i, resource_id));
        }
        
        // Add domain-specific code based on effect type
        match effect.effect_type() {
            "transfer" => {
                code.push_str("  // Transfer implementation\n");
                code.push_str("  return transfer(source, destination, resource0, amount);\n");
            },
            "deposit" => {
                code.push_str("  // Deposit implementation\n");
                code.push_str("  return deposit(destination, resource0, amount);\n");
            },
            "withdraw" => {
                code.push_str("  // Withdraw implementation\n");
                code.push_str("  return withdraw(source, resource0, amount);\n");
            },
            "store" => {
                code.push_str("  // Storage implementation\n");
                code.push_str("  return store(resource0, fields, visibility);\n");
            },
            "query" => {
                code.push_str("  // Query implementation\n");
                code.push_str("  return query(resource0, queryParams);\n");
            },
            _ => {
                code.push_str("  // Generic implementation\n");
                code.push_str("  return execute();\n");
            }
        }
        
        // Close function
        code.push_str("}\n");
        
        Ok(TelScript::new(code))
    }
    
    fn can_compile(&self, effect: Arc<dyn AlgebraicEffect>) -> bool {
        // Check if the effect has TEL hints
        effect.tel_hints().is_some()
    }
} 