//! Configuration Types for Causality Framework
//!
//! This module defines configuration structures and traits for various components
//! within the Causality framework, including Lisp interpreter integration,
//! runtime settings, and system-wide configuration options.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;

use crate::primitive::ids::{ExprId, ResourceId, ValueExprId};
use crate::primitive::string::Str;
use crate::expression::ast::Expr;
use crate::expression::result::ExprError;
use crate::expression::value::ValueExpr;
use crate::resource::{Nullifier, Resource};

//-----------------------------------------------------------------------------
// Host Function Types
//-----------------------------------------------------------------------------

/// Type alias for host function
pub type HostFunction = Arc<dyn Fn(Vec<ValueExpr>) -> Result<ValueExpr, ExprError> + Send + Sync>;

//-----------------------------------------------------------------------------
// Lisp Interpreter Configuration
//-----------------------------------------------------------------------------

/// Configuration for creating a Lisp evaluation context.
#[derive(Default)]
pub struct LispContextConfig {
    /// Optional host function profile name 
    pub host_function_profile: Option<Str>,
    
    /// Additional host functions available to the interpreter
    pub additional_host_functions: BTreeMap<Str, HostFunction>,
    
    /// Initial bindings for variables in the context
    pub initial_bindings: BTreeMap<Str, ValueExpr>,
}

// Manual Debug implementation for LispContextConfig
impl fmt::Debug for LispContextConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LispContextConfig")
            .field("host_function_profile", &self.host_function_profile)
            .field("initial_bindings", &self.initial_bindings)
            .field(
                "additional_host_functions",
                &format!(
                    "<{} additional host functions>",
                    self.additional_host_functions.len()
                ),
            )
            .finish()
    }
}

// Manual Clone implementation for LispContextConfig
impl Clone for LispContextConfig {
    fn clone(&self) -> Self {
        Self {
            host_function_profile: self.host_function_profile,
            initial_bindings: self.initial_bindings.clone(),
            additional_host_functions: self.additional_host_functions.clone(),
        }
    }
}

impl LispContextConfig {
    /// Create a new configuration with the specified host function profile
    pub fn with_host_function_profile(self, profile_name: Option<Str>) -> Self {
        Self {
            host_function_profile: profile_name,
            additional_host_functions: self.additional_host_functions,
            initial_bindings: self.initial_bindings,
        }
    }

    /// Add a host function to the configuration
    pub fn with_host_function(mut self, name: Str, function: HostFunction) -> Self {
        self.additional_host_functions.insert(name, function);
        self
    }

    /// Add an initial binding to the configuration
    pub fn with_initial_binding(mut self, name: Str, value: ValueExpr) -> Self {
        self.initial_bindings.insert(name, value);
        self
    }
}

//-----------------------------------------------------------------------------
// Lisp Evaluation Error Types
//-----------------------------------------------------------------------------

/// Error types for Lisp evaluation operations
#[derive(Debug)]
pub enum LispEvaluationError {
    EvaluationFailed(String),
    ResourceCreationFailed(String),
    ResourceNullificationFailed(String),
    ValueStorageFailed(String),
    ExprResolutionFailed(String),
}

impl std::fmt::Display for LispEvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispEvaluationError::EvaluationFailed(s) => {
                write!(f, "Lisp evaluation failed: {}", s)
            }
            LispEvaluationError::ResourceCreationFailed(s) => {
                write!(f, "Resource creation failed: {}", s)
            }
            LispEvaluationError::ResourceNullificationFailed(s) => {
                write!(f, "Resource nullification failed: {}", s)
            }
            LispEvaluationError::ValueStorageFailed(s) => {
                write!(f, "Value storage failed: {}", s)
            }
            LispEvaluationError::ExprResolutionFailed(s) => {
                write!(f, "Expression resolution failed: {}", s)
            }
        }
    }
}

impl std::error::Error for LispEvaluationError {}

//-----------------------------------------------------------------------------
// Lisp Evaluator Interface
//-----------------------------------------------------------------------------

/// Trait for Lisp expression evaluation within the Causality framework
#[async_trait]
pub trait LispEvaluator: Send + Sync {
    /// Get an expression by ID (synchronous version)
    fn get_expr_sync(&self, id: &ExprId) -> Result<Option<Expr>, LispEvaluationError>;

    /// Evaluate a Lisp expression in the given context
    async fn evaluate_lisp_in_context(
        &self,
        expr_to_eval: &Expr,
        args: Vec<ValueExpr>,
        config: &LispContextConfig,
    ) -> Result<ValueExpr, LispEvaluationError>;

    /// Store a value expression and return its ID
    async fn store_value_expr(
        &self,
        value_expr: ValueExpr,
    ) -> Result<ValueExprId, LispEvaluationError>;

    /// Create a resource during evaluation
    async fn create_resource_for_evaluator(
        &mut self,
        resource: Resource,
    ) -> Result<ResourceId, LispEvaluationError>;

    /// Nullify a resource during evaluation
    async fn nullify_resource_for_evaluator(
        &mut self,
        nullifier: Nullifier,
    ) -> Result<(), LispEvaluationError>;
}

//-----------------------------------------------------------------------------
// Runtime Configuration
//-----------------------------------------------------------------------------

/// Configuration for the Causality runtime system
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
    
    /// Timeout for individual operations (in milliseconds)
    pub operation_timeout_ms: u64,
    
    /// Enable debug logging
    pub debug_logging: bool,
    
    /// Maximum recursion depth for expression evaluation
    pub max_recursion_depth: usize,
    
    /// Memory limit for individual operations (in bytes)
    pub memory_limit_bytes: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 100,
            operation_timeout_ms: 30000, // 30 seconds
            debug_logging: false,
            max_recursion_depth: 1000,
            memory_limit_bytes: 100 * 1024 * 1024, // 100 MB
        }
    }
}

impl RuntimeConfig {
    /// Create a new runtime configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable debug logging
    pub fn with_debug_logging(mut self, enabled: bool) -> Self {
        self.debug_logging = enabled;
        self
    }

    /// Set maximum concurrent operations
    pub fn with_max_concurrent_operations(mut self, max: usize) -> Self {
        self.max_concurrent_operations = max;
        self
    }

    /// Set operation timeout
    pub fn with_operation_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.operation_timeout_ms = timeout_ms;
        self
    }

    /// Set maximum recursion depth
    pub fn with_max_recursion_depth(mut self, depth: usize) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Set memory limit
    pub fn with_memory_limit_bytes(mut self, limit: usize) -> Self {
        self.memory_limit_bytes = limit;
        self
    }
}

//-----------------------------------------------------------------------------
// Domain Configuration
//-----------------------------------------------------------------------------

/// Configuration for domain-specific operations
#[derive(Debug, Clone)]
pub struct DomainConfig {
    /// Domain identifier
    pub domain_name: String,
    
    /// Whether zero-knowledge proofs are required
    pub zk_proofs_required: bool,
    
    /// Whether external API calls are allowed
    pub external_apis_allowed: bool,
    
    /// Maximum resource count per transaction
    pub max_resources_per_transaction: usize,
    
    /// Custom domain-specific settings
    pub custom_settings: BTreeMap<String, String>,
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self {
            domain_name: "default".to_string(),
            zk_proofs_required: false,
            external_apis_allowed: false,
            max_resources_per_transaction: 1000,
            custom_settings: BTreeMap::new(),
        }
    }
}

impl DomainConfig {
    /// Create a new domain configuration
    pub fn new(domain_name: String) -> Self {
        Self {
            domain_name,
            ..Default::default()
        }
    }

    /// Require zero-knowledge proofs
    pub fn with_zk_proofs_required(mut self, required: bool) -> Self {
        self.zk_proofs_required = required;
        self
    }

    /// Allow external API calls
    pub fn with_external_apis_allowed(mut self, allowed: bool) -> Self {
        self.external_apis_allowed = allowed;
        self
    }

    /// Set maximum resources per transaction
    pub fn with_max_resources_per_transaction(mut self, max: usize) -> Self {
        self.max_resources_per_transaction = max;
        self
    }

    /// Add a custom setting
    pub fn with_custom_setting(mut self, key: String, value: String) -> Self {
        self.custom_settings.insert(key, value);
        self
    }
}

//-----------------------------------------------------------------------------
// System Configuration
//-----------------------------------------------------------------------------

/// Top-level system configuration that combines all configuration types
#[derive(Debug, Clone)]
pub struct SystemConfig {
    /// Runtime configuration
    pub runtime: RuntimeConfig,
    
    /// Default domain configuration
    pub default_domain: DomainConfig,
    
    /// Domain-specific configurations
    pub domain_configs: BTreeMap<String, DomainConfig>,
    
    /// Lisp interpreter configuration
    pub lisp_config: LispContextConfig,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            runtime: RuntimeConfig::default(),
            default_domain: DomainConfig::default(),
            domain_configs: BTreeMap::new(),
            lisp_config: LispContextConfig::default(),
        }
    }
}

impl SystemConfig {
    /// Create a new system configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set runtime configuration
    pub fn with_runtime_config(mut self, config: RuntimeConfig) -> Self {
        self.runtime = config;
        self
    }

    /// Set default domain configuration
    pub fn with_default_domain_config(mut self, config: DomainConfig) -> Self {
        self.default_domain = config;
        self
    }

    /// Add a domain-specific configuration
    pub fn with_domain_config(mut self, domain_name: String, config: DomainConfig) -> Self {
        self.domain_configs.insert(domain_name, config);
        self
    }

    /// Set Lisp interpreter configuration
    pub fn with_lisp_config(mut self, config: LispContextConfig) -> Self {
        self.lisp_config = config;
        self
    }

    /// Get configuration for a specific domain
    pub fn get_domain_config(&self, domain_name: &str) -> &DomainConfig {
        self.domain_configs.get(domain_name).unwrap_or(&self.default_domain)
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config_builder() {
        let config = RuntimeConfig::new()
            .with_debug_logging(true)
            .with_max_concurrent_operations(50)
            .with_operation_timeout_ms(10000);

        assert!(config.debug_logging);
        assert_eq!(config.max_concurrent_operations, 50);
        assert_eq!(config.operation_timeout_ms, 10000);
    }

    #[test]
    fn test_domain_config_builder() {
        let config = DomainConfig::new("test_domain".to_string())
            .with_zk_proofs_required(true)
            .with_external_apis_allowed(false)
            .with_custom_setting("key1".to_string(), "value1".to_string());

        assert_eq!(config.domain_name, "test_domain");
        assert!(config.zk_proofs_required);
        assert!(!config.external_apis_allowed);
        assert_eq!(config.custom_settings.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_system_config_builder() {
        let runtime_config = RuntimeConfig::new().with_debug_logging(true);
        let domain_config = DomainConfig::new("test".to_string()).with_zk_proofs_required(true);

        let system_config = SystemConfig::new()
            .with_runtime_config(runtime_config)
            .with_domain_config("test".to_string(), domain_config.clone());

        assert!(system_config.runtime.debug_logging);
        assert_eq!(system_config.get_domain_config("test").domain_name, "test");
        assert!(system_config.get_domain_config("test").zk_proofs_required);

        // Test fallback to default domain
        let default_config = system_config.get_domain_config("nonexistent");
        assert_eq!(default_config.domain_name, "default");
    }

    #[test]
    fn test_lisp_context_config_builder() {
        let config = LispContextConfig::default()
            .with_host_function_profile(Some(Str::new("test_profile")))
            .with_initial_binding(Str::new("test_var"), ValueExpr::Number(crate::primitive::number::Number::Integer(42)));

        assert_eq!(config.host_function_profile, Some(Str::new("test_profile")));
        assert_eq!(config.initial_bindings.get(&Str::new("test_var")), Some(&ValueExpr::Number(crate::primitive::number::Number::Integer(42))));
    }
} 