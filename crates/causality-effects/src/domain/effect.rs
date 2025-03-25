//! Domain effect integration
//!
//! This module implements bidirectional integration between domains and effects
//! following the domain adapter pattern described in ADR-031.

use std::any::Any;
use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;

use crate::context::{Context, ContextConversion};
use crate::core::handler::{EffectHandler, DomainAdapterHandler};
use crate::error::EffectError;
use crate::types::effect::{Effect, QueryEffect, TransactionEffect};
use crate::types::id::{DomainId, EffectTypeId};

/// A marker trait for effects that operate on domains
pub trait DomainEffect: Effect {
    /// Get the domain ID this effect operates on
    fn domain_id(&self) -> DomainId;
    
    /// Get the operation name within the domain
    fn operation_name(&self) -> &str;
}

/// An effect that performs a read-only query against a domain
#[derive(Debug)]
pub struct DomainQueryEffect<P, O> {
    /// The domain ID this effect operates on
    domain_id: DomainId,
    
    /// The operation name within the domain
    operation: String,
    
    /// Type parameter markers
    _marker: PhantomData<(P, O)>,
}

impl<P, O> DomainQueryEffect<P, O>
where
    P: Send + Sync + Debug + 'static,
    O: Send + Sync + Debug + 'static,
{
    /// Create a new domain query effect
    pub fn new<S: Into<String>>(domain_id: DomainId, operation: S) -> Self {
        Self {
            domain_id,
            operation: operation.into(),
            _marker: PhantomData,
        }
    }
}

impl<P, O> Effect for DomainQueryEffect<P, O>
where
    P: Send + Sync + Debug + 'static,
    O: Send + Sync + Debug + 'static,
{
    type Param = P;
    type Outcome = O;
    
    fn type_id(&self) -> EffectTypeId {
        EffectTypeId::new(format!("domain.query.{}.{}", self.domain_id, self.operation))
    }
    
    async fn execute(
        &self,
        param: Self::Param,
        context: &Context,
    ) -> Result<Self::Outcome, EffectError> {
        // This is a placeholder implementation
        // Actual implementation will delegate to a domain adapter
        Err(EffectError::Unimplemented {
            effect_type: self.type_id(),
            message: format!(
                "Domain query effect to {} with operation {} is not implemented",
                self.domain_id, self.operation
            ),
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<P, O> QueryEffect for DomainQueryEffect<P, O>
where
    P: Send + Sync + Debug + 'static,
    O: Send + Sync + Debug + 'static,
{
    // Uses the default implementation from QueryEffect
}

impl<P, O> DomainEffect for DomainQueryEffect<P, O>
where
    P: Send + Sync + Debug + 'static,
    O: Send + Sync + Debug + 'static,
{
    fn domain_id(&self) -> DomainId {
        self.domain_id.clone()
    }
    
    fn operation_name(&self) -> &str {
        &self.operation
    }
}

/// An effect that performs a state-modifying transaction against a domain
#[derive(Debug)]
pub struct DomainTransactionEffect<P, O> {
    /// The domain ID this effect operates on
    domain_id: DomainId,
    
    /// The operation name within the domain
    operation: String,
    
    /// Type parameter markers
    _marker: PhantomData<(P, O)>,
}

impl<P, O> DomainTransactionEffect<P, O>
where
    P: Send + Sync + Debug + 'static,
    O: Send + Sync + Debug + 'static,
{
    /// Create a new domain transaction effect
    pub fn new<S: Into<String>>(domain_id: DomainId, operation: S) -> Self {
        Self {
            domain_id,
            operation: operation.into(),
            _marker: PhantomData,
        }
    }
}

impl<P, O> Effect for DomainTransactionEffect<P, O>
where
    P: Send + Sync + Debug + 'static,
    O: Send + Sync + Debug + 'static,
{
    type Param = P;
    type Outcome = O;
    
    fn type_id(&self) -> EffectTypeId {
        EffectTypeId::new(format!("domain.transaction.{}.{}", self.domain_id, self.operation))
    }
    
    async fn execute(
        &self,
        param: Self::Param,
        context: &Context,
    ) -> Result<Self::Outcome, EffectError> {
        // This is a placeholder implementation
        // Actual implementation will delegate to a domain adapter
        Err(EffectError::Unimplemented {
            effect_type: self.type_id(),
            message: format!(
                "Domain transaction effect to {} with operation {} is not implemented",
                self.domain_id, self.operation
            ),
        })
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<P, O> TransactionEffect for DomainTransactionEffect<P, O>
where
    P: Send + Sync + Debug + 'static,
    O: Send + Sync + Debug + 'static,
{
    // Uses the default implementation from TransactionEffect
}

impl<P, O> DomainEffect for DomainTransactionEffect<P, O>
where
    P: Send + Sync + Debug + 'static,
    O: Send + Sync + Debug + 'static,
{
    fn domain_id(&self) -> DomainId {
        self.domain_id.clone()
    }
    
    fn operation_name(&self) -> &str {
        &self.operation
    }
}

/// A trait for domain adapters that can handle effects
#[async_trait]
pub trait DomainAdapter: Send + Sync + Debug + 'static {
    /// Get the domain ID this adapter handles
    fn domain_id(&self) -> DomainId;
    
    /// Get the list of operations supported by this domain
    fn supported_operations(&self) -> Vec<String>;
    
    /// Check if this domain adapter supports the given operation
    fn supports_operation(&self, operation: &str) -> bool {
        self.supported_operations().iter().any(|op| op == operation)
    }
    
    /// Execute a query operation against this domain
    async fn execute_query<P, O>(
        &self,
        operation: &str,
        param: P,
        context: &Context,
    ) -> Result<O, EffectError>
    where
        P: Send + Sync + Debug + 'static,
        O: Send + Sync + Debug + 'static;
    
    /// Execute a transaction operation against this domain
    async fn execute_transaction<P, O>(
        &self,
        operation: &str,
        param: P,
        context: &Context,
    ) -> Result<O, EffectError>
    where
        P: Send + Sync + Debug + 'static,
        O: Send + Sync + Debug + 'static;
}

/// An adapter that converts a domain adapter into an effect handler
/// following the domain adapter pattern in ADR-031
#[derive(Debug)]
pub struct DomainHandlerAdapter<D> {
    domain: D,
}

impl<D> DomainHandlerAdapter<D>
where
    D: DomainAdapter,
{
    /// Create a new domain handler adapter
    pub fn new(domain: D) -> Self {
        Self { domain }
    }
    
    /// Get a reference to the underlying domain adapter
    pub fn domain(&self) -> &D {
        &self.domain
    }
}

#[async_trait]
impl<D> EffectHandler for DomainHandlerAdapter<D>
where
    D: DomainAdapter,
{
    async fn can_handle(&self, effect_type: &EffectTypeId) -> bool {
        let type_str = effect_type.as_str();
        
        // Check if this is a domain effect for this domain
        if let Some(prefix) = type_str.strip_prefix("domain.query.") {
            let parts: Vec<&str> = prefix.splitn(2, '.').collect();
            if parts.len() == 2 {
                let domain_id = DomainId::new(parts[0]);
                let operation = parts[1];
                
                return self.domain.domain_id() == domain_id && self.domain.supports_operation(operation);
            }
        }
        
        if let Some(prefix) = type_str.strip_prefix("domain.transaction.") {
            let parts: Vec<&str> = prefix.splitn(2, '.').collect();
            if parts.len() == 2 {
                let domain_id = DomainId::new(parts[0]);
                let operation = parts[1];
                
                return self.domain.domain_id() == domain_id && self.domain.supports_operation(operation);
            }
        }
        
        false
    }
    
    async fn handle(
        &self,
        effect_type: &EffectTypeId,
        param: Box<dyn Any + Send>,
        context: &Context,
    ) -> Result<Box<dyn Any + Send>, EffectError> {
        let type_str = effect_type.as_str();
        
        // Handle domain query effects
        if let Some(prefix) = type_str.strip_prefix("domain.query.") {
            let parts: Vec<&str> = prefix.splitn(2, '.').collect();
            if parts.len() == 2 {
                let domain_id = DomainId::new(parts[0]);
                let operation = parts[1];
                
                if self.domain.domain_id() != domain_id {
                    return Err(EffectError::InvalidHandler {
                        effect_type: effect_type.clone(),
                        handler_type: format!("DomainHandlerAdapter<{:?}>", self.domain.domain_id()),
                        message: format!(
                            "Domain ID mismatch: effect is for {}, but handler is for {}",
                            domain_id, self.domain.domain_id()
                        ),
                    });
                }
                
                if !self.domain.supports_operation(operation) {
                    return Err(EffectError::InvalidHandler {
                        effect_type: effect_type.clone(),
                        handler_type: format!("DomainHandlerAdapter<{:?}>", self.domain.domain_id()),
                        message: format!(
                            "Operation {} not supported by domain {}",
                            operation, self.domain.domain_id()
                        ),
                    });
                }
                
                // This is a type-erased proxy to the domain adapter's execute_query method
                // The actual implementation would need to use macros or code generation
                // to properly handle the type erasure
                return Err(EffectError::Unimplemented {
                    effect_type: effect_type.clone(),
                    message: "Type-erased domain query execution is not yet implemented".to_string(),
                });
            }
        }
        
        // Handle domain transaction effects
        if let Some(prefix) = type_str.strip_prefix("domain.transaction.") {
            let parts: Vec<&str> = prefix.splitn(2, '.').collect();
            if parts.len() == 2 {
                let domain_id = DomainId::new(parts[0]);
                let operation = parts[1];
                
                if self.domain.domain_id() != domain_id {
                    return Err(EffectError::InvalidHandler {
                        effect_type: effect_type.clone(),
                        handler_type: format!("DomainHandlerAdapter<{:?}>", self.domain.domain_id()),
                        message: format!(
                            "Domain ID mismatch: effect is for {}, but handler is for {}",
                            domain_id, self.domain.domain_id()
                        ),
                    });
                }
                
                if !self.domain.supports_operation(operation) {
                    return Err(EffectError::InvalidHandler {
                        effect_type: effect_type.clone(),
                        handler_type: format!("DomainHandlerAdapter<{:?}>", self.domain.domain_id()),
                        message: format!(
                            "Operation {} not supported by domain {}",
                            operation, self.domain.domain_id()
                        ),
                    });
                }
                
                // This is a type-erased proxy to the domain adapter's execute_transaction method
                // The actual implementation would need to use macros or code generation
                // to properly handle the type erasure
                return Err(EffectError::Unimplemented {
                    effect_type: effect_type.clone(),
                    message: "Type-erased domain transaction execution is not yet implemented".to_string(),
                });
            }
        }
        
        Err(EffectError::InvalidHandler {
            effect_type: effect_type.clone(),
            handler_type: format!("DomainHandlerAdapter<{:?}>", self.domain.domain_id()),
            message: format!(
                "Effect type {} is not a domain effect or not for this domain",
                effect_type
            ),
        })
    }
}

impl<D> DomainAdapterHandler for DomainHandlerAdapter<D>
where
    D: DomainAdapter,
{
    fn domain_id(&self) -> DomainId {
        self.domain.domain_id()
    }
}

/// A utility to convert between effect contexts and domain-specific contexts
pub struct ContextConverter;

impl ContextConverter {
    /// Convert from an effect context to a domain context
    ///
    /// This will filter the capabilities to only those relevant to the domain
    pub fn to_domain_context(context: &Context, domain_id: &DomainId) -> Context {
        let mut domain_context = Context::new();
        
        // Copy authorization info
        domain_context.set_authorization(context.authorization().cloned());
        
        // Filter capabilities to only those for this domain
        // In a real implementation, this would be more sophisticated
        for (id, capability) in context.capabilities() {
            if id.as_str().starts_with(&format!("{}.capability.", domain_id)) {
                domain_context.add_capability(id.clone(), capability.clone());
            }
        }
        
        domain_context
    }
    
    /// Convert from a domain context to an effect context
    ///
    /// This will ensure domain-specific capabilities are properly prefixed
    pub fn from_domain_context(domain_context: &Context, domain_id: &DomainId) -> Context {
        let mut effect_context = Context::new();
        
        // Copy authorization info
        effect_context.set_authorization(domain_context.authorization().cloned());
        
        // Add domain-specific prefix to capabilities if needed
        for (id, capability) in domain_context.capabilities() {
            if !id.as_str().starts_with(&format!("{}.capability.", domain_id)) {
                // Add domain prefix to capability ID
                let prefixed_id = format!("{}.capability.{}", domain_id, id);
                effect_context.add_capability(prefixed_id.into(), capability.clone());
            } else {
                // Keep the ID as is if it already has the domain prefix
                effect_context.add_capability(id.clone(), capability.clone());
            }
        }
        
        effect_context
    }
    
    /// Merge domain-specific context into a parent effect context
    pub fn merge_domain_context(
        parent_context: &mut Context,
        domain_context: &Context,
        domain_id: &DomainId,
    ) {
        // Merge authorization (prefer domain context's auth if present)
        if let Some(auth) = domain_context.authorization() {
            parent_context.set_authorization(Some(auth.clone()));
        }
        
        // Add domain capabilities to parent context
        for (id, capability) in domain_context.capabilities() {
            if !id.as_str().starts_with(&format!("{}.capability.", domain_id)) {
                // Add domain prefix to capability ID
                let prefixed_id = format!("{}.capability.{}", domain_id, id);
                parent_context.add_capability(prefixed_id.into(), capability.clone());
            } else {
                // Keep the ID as is if it already has the domain prefix
                parent_context.add_capability(id.clone(), capability.clone());
            }
        }
    }
}

impl ContextConversion for DomainId {
    fn convert_context_to(&self, context: &Context) -> Context {
        ContextConverter::to_domain_context(context, self)
    }
    
    fn convert_context_from(&self, domain_context: &Context) -> Context {
        ContextConverter::from_domain_context(domain_context, self)
    }
    
    fn merge_context(&self, parent_context: &mut Context, domain_context: &Context) {
        ContextConverter::merge_domain_context(parent_context, domain_context, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::types::id::CapabilityId;
    
    #[tokio::test]
    async fn test_domain_query_effect() {
        // Create a domain query effect
        let domain_id = DomainId::new("test.domain");
        let effect = DomainQueryEffect::<String, u32>::new(domain_id.clone(), "get_count");
        
        // Verify the effect type ID
        assert_eq!(
            effect.type_id().as_str(),
            "domain.query.test.domain.get_count"
        );
        
        // Verify domain ID and operation
        assert_eq!(effect.domain_id(), domain_id);
        assert_eq!(effect.operation_name(), "get_count");
        
        // Verify read-only flag
        assert!(effect.is_read_only());
    }
    
    #[tokio::test]
    async fn test_domain_transaction_effect() {
        // Create a domain transaction effect
        let domain_id = DomainId::new("test.domain");
        let effect = DomainTransactionEffect::<String, bool>::new(domain_id.clone(), "update_value");
        
        // Verify the effect type ID
        assert_eq!(
            effect.type_id().as_str(),
            "domain.transaction.test.domain.update_value"
        );
        
        // Verify domain ID and operation
        assert_eq!(effect.domain_id(), domain_id);
        assert_eq!(effect.operation_name(), "update_value");
        
        // Verify read-only flag
        assert!(!effect.is_read_only());
    }
    
    // A test implementation of the DomainAdapter trait
    #[derive(Debug)]
    struct TestDomainAdapter {
        domain_id: DomainId,
        operations: Vec<String>,
    }
    
    #[async_trait]
    impl DomainAdapter for TestDomainAdapter {
        fn domain_id(&self) -> DomainId {
            self.domain_id.clone()
        }
        
        fn supported_operations(&self) -> Vec<String> {
            self.operations.clone()
        }
        
        async fn execute_query<P, O>(
            &self,
            operation: &str,
            _param: P,
            _context: &Context,
        ) -> Result<O, EffectError>
        where
            P: Send + Sync + Debug + 'static,
            O: Send + Sync + Debug + 'static,
        {
            Err(EffectError::Unimplemented {
                effect_type: EffectTypeId::new(format!("domain.query.{}.{}", self.domain_id, operation)),
                message: "Test domain adapter does not implement query execution".to_string(),
            })
        }
        
        async fn execute_transaction<P, O>(
            &self,
            operation: &str,
            _param: P,
            _context: &Context,
        ) -> Result<O, EffectError>
        where
            P: Send + Sync + Debug + 'static,
            O: Send + Sync + Debug + 'static,
        {
            Err(EffectError::Unimplemented {
                effect_type: EffectTypeId::new(format!("domain.transaction.{}.{}", self.domain_id, operation)),
                message: "Test domain adapter does not implement transaction execution".to_string(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_domain_handler_adapter() {
        // Create a test domain adapter
        let domain_id = DomainId::new("test.domain");
        let domain_adapter = TestDomainAdapter {
            domain_id: domain_id.clone(),
            operations: vec!["get_count".to_string(), "update_value".to_string()],
        };
        
        // Create a domain handler adapter
        let handler = DomainHandlerAdapter::new(domain_adapter);
        
        // Verify that the handler can handle the correct effect types
        assert!(
            handler.can_handle(&EffectTypeId::new("domain.query.test.domain.get_count")).await
        );
        assert!(
            handler.can_handle(&EffectTypeId::new("domain.transaction.test.domain.update_value")).await
        );
        
        // Verify that the handler cannot handle incorrect effect types
        assert!(
            !handler.can_handle(&EffectTypeId::new("domain.query.other.domain.get_count")).await
        );
        assert!(
            !handler.can_handle(&EffectTypeId::new("domain.query.test.domain.unknown_op")).await
        );
        assert!(
            !handler.can_handle(&EffectTypeId::new("other.effect.type")).await
        );
        
        // Verify domain ID
        assert_eq!(handler.domain_id(), domain_id);
    }
    
    #[test]
    fn test_context_conversion() {
        let domain_id = DomainId::new("test.domain");
        
        // Create a effect context with mixed capabilities
        let mut context = Context::new();
        context.add_capability(
            CapabilityId::new("generic.capability"),
            Box::new("generic value")
        );
        context.add_capability(
            CapabilityId::new("test.domain.capability.specific"),
            Box::new("domain specific value")
        );
        
        // Convert to domain context
        let domain_context = ContextConverter::to_domain_context(&context, &domain_id);
        
        // Domain context should only have domain-specific capabilities
        assert!(domain_context.has_capability(&CapabilityId::new("test.domain.capability.specific")));
        assert!(!domain_context.has_capability(&CapabilityId::new("generic.capability")));
        
        // Create a domain context with domain-specific capabilities
        let mut domain_context = Context::new();
        domain_context.add_capability(
            CapabilityId::new("read"),
            Box::new("read permission")
        );
        
        // Convert back to effect context
        let effect_context = ContextConverter::from_domain_context(&domain_context, &domain_id);
        
        // Effect context should have domain-prefixed capabilities
        assert!(effect_context.has_capability(&CapabilityId::new("test.domain.capability.read")));
        
        // Test merging
        let mut parent_context = Context::new();
        parent_context.add_capability(
            CapabilityId::new("parent.capability"),
            Box::new("parent value")
        );
        
        ContextConverter::merge_domain_context(&mut parent_context, &domain_context, &domain_id);
        
        // Parent context should now have both capabilities
        assert!(parent_context.has_capability(&CapabilityId::new("parent.capability")));
        assert!(parent_context.has_capability(&CapabilityId::new("test.domain.capability.read")));
    }
} 