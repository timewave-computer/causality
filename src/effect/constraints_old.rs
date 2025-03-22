//! Effect constraint traits for the three-layer effect architecture
//!
//! This module defines the constraint traits that specify behavior for different
//! categories of effects. These traits build on the base Effect trait to provide
//! type-safe interfaces for common effect patterns.

use std::collections::HashSet;
use std::fmt::Debug;
use std::time::Duration;
use async_trait::async_trait;
use std::sync::Arc;

use crate::address::Address;
use crate::resource::{ResourceId, Quantity};
use crate::domain::DomainId;
use crate::effect::{Effect, EffectContext, EffectOutcome, EffectResult};

/// Represents an effect that transfers assets between addresses
#[async_trait]
pub trait TransferEffect: Effect + Debug + Send + Sync {
    /// Get the source address for the transfer
    fn source(&self) -> &Address;
    
    /// Get the destination address for the transfer
    fn destination(&self) -> &Address;
    
    /// Get the amount being transferred
    fn amount(&self) -> &Quantity;
    
    /// Get the resource/token ID being transferred
    fn token(&self) -> &ResourceId;
    
    /// Get the domain ID where the transfer will occur
    fn domain_id(&self) -> &DomainId;
    
    /// Whether this transfer requires prior approval
    ///
    /// Defaults to false for simple transfers, but may be true for
    /// contract-based token transfers or other delegated operations.
    fn requires_approval(&self) -> bool {
        false
    }
    
    /// Get the fee for this transfer, if applicable
    fn fee(&self) -> Option<Quantity> {
        None
    }
    
    /// Validate the transfer parameters
    fn validate(&self) -> Result<(), String> {
        // Basic validation checks
        if self.amount().is_zero() {
            return Err("Transfer amount cannot be zero".to_string());
        }
        
        // Additional validation can be performed by implementations
        Ok(())
    }
    
    /// Execute the transfer with validation
    async fn execute_with_validation(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Validate before execution
        if let Err(err) = self.validate() {
            return Err(crate::effect::EffectError::ValidationError(err));
        }
        
        // Execute the effect - using the trait method that takes a reference
        self.execute_async(context).await
    }
}

/// Create a new transfer effect for the specified domain
///
/// This factory function uses the domain adapter system to create the appropriate
/// domain-specific implementation of the TransferEffect trait.
pub async fn create_transfer_effect(
    source: Address,
    destination: Address,
    amount: Quantity,
    token: ResourceId,
    domain_id: DomainId,
    domain_registry: &crate::domain::DomainRegistry,
) -> Result<Box<dyn TransferEffect>, crate::error::Error> {
    // Get domain info
    let domain_info = domain_registry.get_domain_info(&domain_id)
        .ok_or_else(|| crate::error::Error::DomainNotFound(domain_id.clone()))?;
    
    // Create the appropriate domain-specific implementation
    // This will be expanded as more domain adapters are implemented
    match domain_info.domain_type {
        crate::domain::DomainType::EVM => {
            // For now, return a placeholder implementation
            // In a real implementation, we would create an EVM-specific transfer effect
            Err(crate::error::Error::NotImplemented("EVM transfer effect not yet implemented".to_string()))
        },
        crate::domain::DomainType::CosmWasm => {
            // For now, return a placeholder implementation
            Err(crate::error::Error::NotImplemented("CosmWasm transfer effect not yet implemented".to_string()))
        },
        _ => {
            Err(crate::error::Error::UnsupportedOperation(format!(
                "Transfer not supported for domain type {:?}", 
                domain_info.domain_type
            )))
        }
    }
}

/// Represents an effect that performs a query operation
#[async_trait]
pub trait QueryEffect: Effect + Debug + Send + Sync {
    /// Get the query type
    fn query_type(&self) -> &str;
    
    /// Get the parameters for this query
    fn parameters(&self) -> &serde_json::Value;
    
    /// Get the timeout for this query
    fn timeout(&self) -> Duration;
    
    /// Get the cache time-to-live for this query, if applicable
    fn cache_ttl(&self) -> Option<Duration> {
        None
    }
    
    /// Whether this query requires fresh data (no caching)
    fn requires_fresh_data(&self) -> bool {
        false
    }
    
    /// Validate the query parameters
    fn validate(&self) -> Result<(), String> {
        // Default implementation performs basic validation
        if self.query_type().is_empty() {
            return Err("Query type cannot be empty".to_string());
        }
        
        // Additional validation can be performed by implementations
        Ok(())
    }
    
    /// Execute the query with validation
    async fn execute_with_validation(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Validate before execution
        if let Err(err) = self.validate() {
            return Err(crate::effect::EffectError::ValidationError(err));
        }
        
        // Execute the effect - using the trait method that takes a reference
        self.execute_async(context).await
    }
}

/// Represents an effect that performs storage operations
#[async_trait]
pub trait StorageEffect: Effect + Debug + Send + Sync {
    /// Get the register ID for this storage operation
    fn register_id(&self) -> &ResourceId;
    
    /// Get the fields affected by this storage operation
    fn fields(&self) -> &HashSet<String>;
    
    /// Get the domain ID where the storage operation will occur
    fn domain_id(&self) -> &DomainId;
    
    /// Whether this is an update operation (vs. a create operation)
    fn is_update(&self) -> bool {
        false
    }
    
    /// Get the hash of the previous state, if this is an update operation
    fn previous_state_hash(&self) -> Option<&str> {
        None
    }
    
    /// Validate the storage parameters
    fn validate(&self) -> Result<(), String> {
        // Default implementation performs basic validation
        if self.register_id().is_empty() {
            return Err("Register ID cannot be empty".to_string());
        }
        
        if self.fields().is_empty() {
            return Err("Fields cannot be empty".to_string());
        }
        
        // Additional validation can be performed by implementations
        Ok(())
    }
    
    /// Execute the storage operation with validation
    async fn execute_with_validation(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Validate before execution
        if let Err(err) = self.validate() {
            return Err(crate::effect::EffectError::ValidationError(err));
        }
        
        // Execute the effect - using the trait method that takes a reference
        self.execute_async(context).await
    }
}

/// Check if an effect satisfies the TransferEffect constraint
pub fn check_transfer_effect(
    effect: Arc<dyn Effect>,
    source: ResourceId,
    destination: ResourceId,
    amount: Option<u64>,
) -> Result<Box<dyn TransferEffect>, crate::error::Error> {
    // ... existing code ...
} 