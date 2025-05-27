//! Core traits for unified object types in the Causality framework.
//! 
//! This module defines the trait hierarchy for Effect, Intent, Handler, Transaction,
//! and Resource types, providing consistent interfaces and behavior across all
//! implementations.

use crate::primitive::{string::Str, ids::{DomainId, ExprId}, time::Timestamp};
use crate::primitive::ids::EntityId;
// Temporarily comment out until resource types are properly structured
// use crate::resource::{ResourcePattern, ResourceFlow};

/// Core identification trait for all causality objects
pub trait AsIdentifiable {
    /// Unique identifier for this object
    fn id(&self) -> &EntityId;
    
    /// Human-readable name or description
    fn name(&self) -> &Str;
}

/// Domain association trait for objects that belong to a specific domain
pub trait HasDomainId {
    /// The domain this object belongs to
    fn domain_id(&self) -> &DomainId;
}

/// Trait for types that have input resources
pub trait HasInputs {
    fn inputs(&self) -> &[crate::resource::flow::ResourceFlow];
}

/// Trait for types that have output resources  
pub trait HasOutputs {
    fn outputs(&self) -> &[crate::resource::flow::ResourceFlow];
}

/// Expression-based validation/execution logic
pub trait HasExpression {
    /// TEL expression for validation, constraints, or execution logic
    fn expression(&self) -> Option<&ExprId>;
}

/// Temporal tracking for objects with time-based behavior
pub trait HasTimestamp {
    /// When this object was created or became active
    fn timestamp(&self) -> &Timestamp;
}

//-----------------------------------------------------------------------------
// Specialized Traits
//-----------------------------------------------------------------------------

/// Effect-specific behavior and properties
pub trait AsEffect: AsIdentifiable + HasDomainId + HasInputs + HasOutputs + HasExpression + HasTimestamp {
    /// Get the effect type identifier
    fn effect_type(&self) -> &Str;
    
    /// Check if this effect can be applied in the given context
    fn can_apply(&self, _context: &dyn std::any::Any) -> bool {
        // Default implementation - can be overridden
        true
    }
}

/// Intent-specific behavior and properties  
pub trait AsIntent: AsIdentifiable + HasDomainId + HasInputs + HasOutputs + HasExpression + HasTimestamp {
    /// Get the intent priority level
    fn priority(&self) -> u32;
    
    /// Check if this intent is satisfied by the given effects
    fn is_satisfied_by(&self, effects: &[&dyn AsEffect]) -> bool;
}

/// Handler-specific behavior and properties
pub trait AsHandler: AsIdentifiable + HasDomainId + HasExpression + HasTimestamp {
    /// Get the handler type this can process
    fn handles_type(&self) -> &Str;
    
    /// Get the handler priority for conflict resolution
    fn priority(&self) -> u32;
    
    /// Check if this handler can process the given object
    fn can_handle(&self, target: &dyn std::any::Any) -> bool;
}

/// Transaction-specific behavior and properties
pub trait AsTransaction: AsIdentifiable + HasDomainId + HasInputs + HasOutputs + HasTimestamp {
    /// Get all effects included in this transaction
    fn effects(&self) -> &[EntityId];
    
    /// Get all intents satisfied by this transaction
    fn intents(&self) -> &[EntityId];
    
    /// Check if this transaction is valid
    fn is_valid(&self) -> bool;
}

/// Trait for resource-like objects
pub trait AsResource {
    /// Get the resource type identifier
    fn resource_type(&self) -> &crate::primitive::string::Str;
    
    /// Get the quantity of this resource
    fn quantity(&self) -> u64;
    
    /// Check if this resource matches a given pattern
    fn matches_pattern(&self, pattern: &crate::resource::flow::ResourcePattern) -> bool;
}

//-----------------------------------------------------------------------------
// Composite Traits for Common Patterns
//-----------------------------------------------------------------------------

/// Complete causality object with all common properties
pub trait AsCausalityObject: AsIdentifiable + HasDomainId + HasExpression + HasTimestamp {}

/// Resource-transforming object (Effects, Transactions)
pub trait AsResourceTransformer: AsCausalityObject + HasInputs + HasOutputs {}

/// Executable object that can be processed by handlers
pub trait AsExecutable: AsCausalityObject {
    /// Get the execution priority
    fn execution_priority(&self) -> u32 {
        0 // Default priority
    }
}

//-----------------------------------------------------------------------------
// Blanket Implementations
//-----------------------------------------------------------------------------

// Automatically implement composite traits for types that have all required traits
impl<T> AsCausalityObject for T 
where 
    T: AsIdentifiable + HasDomainId + HasExpression + HasTimestamp 
{}

impl<T> AsResourceTransformer for T 
where 
    T: AsCausalityObject + HasInputs + HasOutputs 
{}

impl<T> AsExecutable for T 
where 
    T: AsCausalityObject 
{}
