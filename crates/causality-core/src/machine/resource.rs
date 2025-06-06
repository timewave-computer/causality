//! Resource management and heap operations
//!
//! This module implements linear resource management with heap allocation,
//! consumption tracking, and ownership semantics for the register machine.

use crate::{
    system::{
        content_addressing::{ResourceId, Timestamp, DomainId, Str},
        causality::CausalProof,
        error::MachineError,
    },
    lambda::{
        TypeInner, Symbol, 
        base::Value,
    },
    machine::{
        value::MachineValue,
        nullifier::NullifierSet,
    },
};
use std::collections::BTreeMap;
use ssz::{Encode, Decode};

/// Linear resource with full architectural metadata (immutable)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    /// Unique identifier for this resource (content-addressed)
    pub id: ResourceId,
    
    /// Human-readable name or description  
    pub name: Str,
    
    /// Domain this resource belongs to
    pub domain: DomainId,
    
    /// Resource type identifier (e.g., "token", "bandwidth")
    pub label: Str,

    /// Whether this resource is ephemeral (created and destroyed instantaneously during a transaction)
    pub ephemeral: bool,

    /// Current quantity/amount of this resource (for quantifiable assets)
    pub quantity: u64,
    
    /// When this resource was created or last updated
    pub timestamp: Timestamp,
    
    /// A Product row type instance defining associated capabilities (used for capability patterns)
    pub capabilities: Value,
    
    /// A Product row type instance holding the intrinsic data of the resource
    pub data: Value,
    
    /// Cryptographic proof of its origin and transformation history
    pub causality: CausalProof,
    
    /// Optional: if this resource also represents a computational budget
    pub budget: Option<u64>,
}

/// Resource heap with nullifier-based consumption tracking and external state management
#[derive(Debug, Clone)]
pub struct ResourceHeap {
    /// Map from resource IDs to immutable resources
    resources: BTreeMap<ResourceId, Resource>,
    
    /// Nullifier set tracking consumed resources
    nullifiers: NullifierSet,
    
    /// External state tracking for resources (mutable state machine states)
    /// Maps resource ID to current state value
    resource_states: BTreeMap<ResourceId, Value>,
    
    /// Default domain for new resources
    default_domain: DomainId,
}

impl ResourceHeap {
    /// Create a new empty resource heap
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
            nullifiers: NullifierSet::new(),
            resource_states: BTreeMap::new(),
            default_domain: DomainId::from_bytes([0; 32]), // Default domain ID
        }
    }
    
    /// Create a resource heap with a specific default domain
    pub fn with_domain(domain: DomainId) -> Self {
        Self {
            resources: BTreeMap::new(),
            nullifiers: NullifierSet::new(),
            resource_states: BTreeMap::new(),
            default_domain: domain,
        }
    }
    
    /// Allocate a resource on the heap (creates immutable resource)
    pub fn alloc_resource(&mut self, value: MachineValue, resource_type: TypeInner) -> ResourceId {
        let resource = Resource::simple(value, resource_type, self.default_domain);
        let id = resource.id;
        self.resources.insert(id, resource);
        id
    }
    
    /// Allocate a resource with full metadata
    pub fn alloc_full_resource(&mut self, resource: Resource) -> ResourceId {
        let id = resource.id;
        self.resources.insert(id, resource);
        id
    }
    
    /// Consume a resource using nullifier-based tracking
    pub fn consume_resource(&mut self, id: ResourceId) -> Result<MachineValue, MachineError> {
        // Check if resource exists
        let resource = self.resources.get(&id)
            .ok_or(MachineError::InvalidResource(id))?;
        
        // Check if already consumed via nullifier
        if self.nullifiers.is_resource_consumed(id, "consume", None) {
            return Err(MachineError::ResourceAlreadyConsumed(id));
        }
        
        // Get the machine value before any mutable operations
        let machine_value = resource.as_machine_value();
        
        // Generate and add nullifier for consumption
        self.nullifiers.consume_resource(id, "consume", None)
            .map_err(|_| MachineError::ResourceAlreadyConsumed(id))?;
        
        // Clear any external state tracking for this resource
        self.clear_resource_state(id);
        
        // Return the resource's data (resource itself remains in heap, immutable)
        Ok(machine_value)
    }
    
    /// Check if a resource exists and hasn't been consumed
    pub fn is_available(&self, id: ResourceId) -> bool {
        self.resources.contains_key(&id) && !self.nullifiers.is_resource_consumed(id, "consume", None)
    }
    
    /// Get a reference to a resource without consuming it
    pub fn peek_resource(&self, id: ResourceId) -> Result<&Resource, MachineError> {
        self.resources.get(&id)
            .ok_or(MachineError::InvalidResource(id))
    }
    
    /// Check if a resource has been consumed
    pub fn is_consumed(&self, id: ResourceId) -> bool {
        self.nullifiers.is_resource_consumed(id, "consume", None)
    }
    
    /// Get the nullifier set (for ZK proof generation)
    pub fn get_nullifiers(&self) -> &NullifierSet {
        &self.nullifiers
    }
    
    /// Get mutable access to nullifiers (for advanced operations)
    pub fn get_nullifiers_mut(&mut self) -> &mut NullifierSet {
        &mut self.nullifiers
    }
    
    /// Get total number of resources allocated
    pub fn total_resources(&self) -> usize {
        self.resources.len()
    }
    
    /// Get number of consumed resources
    pub fn consumed_count(&self) -> usize {
        self.nullifiers.size()
    }
    
    /// Get number of available (unconsumed) resources
    pub fn available_count(&self) -> usize {
        self.total_resources() - self.consumed_count()
    }
    
    /// Set the state for a resource (external state tracking)
    pub fn set_resource_state(&mut self, id: ResourceId, state: Value) -> Result<(), MachineError> {
        // Verify resource exists
        if !self.resources.contains_key(&id) {
            return Err(MachineError::InvalidResource(id));
        }
        
        self.resource_states.insert(id, state);
        Ok(())
    }
    
    /// Get the current state for a resource
    pub fn get_resource_state(&self, id: ResourceId) -> Option<&Value> {
        self.resource_states.get(&id)
    }
    
    /// Remove state tracking for a resource (typically when consumed)
    pub fn clear_resource_state(&mut self, id: ResourceId) {
        self.resource_states.remove(&id);
    }
    
    /// Get all resources with their current states
    pub fn get_resources_with_states(&self) -> impl Iterator<Item = (&Resource, Option<&Value>)> {
        self.resources.values().map(move |resource| {
            let state = self.resource_states.get(&resource.id);
            (resource, state)
        })
    }
}

impl Default for ResourceHeap {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for machine state to manage resources
pub trait ResourceManager {
    /// Allocate a resource on the heap
    fn alloc_resource(&mut self, value: MachineValue, resource_type: TypeInner) -> ResourceId;
    
    /// Consume a resource from the heap
    fn consume_resource(&mut self, id: ResourceId) -> Result<MachineValue, MachineError>;
}

impl Resource {
    /// Create a new resource with minimal data (for compatibility with existing code)
    pub fn simple(value: MachineValue, resource_type: TypeInner, domain: DomainId) -> Self {
        let name = Str::new("resource");
        let resource_type_str = Str::new(&format!("{:?}", resource_type));
        let timestamp = Timestamp::now();
        let causality = CausalProof::genesis("alloc");
        
        // Convert MachineValue to Layer 1 Value for data field
        let data_value = match value {
            MachineValue::Unit => Value::Unit,
            MachineValue::Bool(b) => Value::Bool(b),
            MachineValue::Int(i) => Value::Int(i),
            MachineValue::Symbol(s) => Value::Symbol(Str::new(s.name().unwrap_or("unknown"))),
            _ => Value::Unit, // Fallback for complex types
        };
        
        let mut resource = Self {
            id: ResourceId::from_bytes([0; 32]), // Placeholder, will be computed
            name,
            domain,
            label: resource_type_str,
            quantity: 1,
            timestamp,
            capabilities: Value::Unit, // Default empty capabilities
            data: data_value,
            causality,
            budget: None,
            ephemeral: false,
        };
        
        // Compute content-addressed ID based on resource content
        resource.id = resource.compute_id();
        resource
    }
    
    /// Create a new resource with full metadata
    pub fn new(
        name: impl Into<String>,
        domain: DomainId,
        resource_type: impl Into<String>,
        quantity: u64,
        capabilities: Value,
        data: Value,
        causality: CausalProof,
        budget: Option<u64>,
        ephemeral: bool,
    ) -> Self {
        let name = Str::new(&name.into());
        let resource_type = Str::new(&resource_type.into());
        let timestamp = Timestamp::now();
        
        let mut resource = Self {
            id: ResourceId::from_bytes([0; 32]), // Placeholder
            name,
            domain,
            label: resource_type,
            quantity,
            timestamp,
            capabilities,
            data,
            causality,
            budget,
            ephemeral,
        };
        
        // Compute content-addressed ID
        resource.id = resource.compute_id();
        resource
    }
    
    /// Create an ephemeral resource (convenience method)
    pub fn ephemeral(
        name: impl Into<String>,
        domain: DomainId,
        resource_type: impl Into<String>,
        data: Value,
    ) -> Self {
        Self::new(
            name,
            domain,
            resource_type,
            1, // Ephemeral resources typically have quantity 1
            Value::Unit, // No special capabilities needed
            data,
            CausalProof::genesis("ephemeral_alloc"),
            None, // No budget
            true, // Mark as ephemeral
        )
    }
    
    /// Get the underlying data as a MachineValue (for compatibility)
    pub fn as_machine_value(&self) -> MachineValue {
        match &self.data {
            Value::Unit => MachineValue::Unit,
            Value::Bool(b) => MachineValue::Bool(*b),
            Value::Int(i) => MachineValue::Int(*i),
            Value::Symbol(s) => MachineValue::Symbol(Symbol::new(&s.as_str())),
            _ => MachineValue::Unit, // Fallback for complex types
        }
    }
    
    /// Compute the content-addressed ID for this resource
    fn compute_id(&self) -> ResourceId {
        // Use SSZ serialization for deterministic content addressing
        // Note: All fields are included since resources are immutable
        // State is NOT included since it's tracked externally
        let data_for_hash = (
            &self.name,
            &self.domain,
            &self.label,
            &self.quantity,
            &self.timestamp,
            &self.capabilities,
            &self.data,
            &self.causality,
            &self.budget,
            &self.ephemeral,
        );
        
        ResourceId::from_content(&data_for_hash)
    }
}

impl Encode for Resource {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        // Simple length calculation for basic fields
        32 + // id
        self.name.ssz_bytes_len() +
        32 + // domain_id  
        self.label.ssz_bytes_len() +
        8 + // quantity (u64)
        8 + // timestamp (simplified)
        1 + // capabilities (simplified as unit)
        1 + // data (simplified as unit)
        1 + // causality (simplified)
        9 + // budget (Option<u64>)
        1 // ephemeral (bool)
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.id.ssz_append(buf);
        self.name.ssz_append(buf);
        self.domain.ssz_append(buf);
        self.label.ssz_append(buf);
        self.quantity.ssz_append(buf);
        
        // Simplified encoding for complex types
        self.timestamp.ssz_append(buf);
        0u8.ssz_append(buf); // capabilities placeholder
        0u8.ssz_append(buf); // data placeholder
        0u8.ssz_append(buf); // causality placeholder
        self.budget.ssz_append(buf);
        self.ephemeral.ssz_append(buf);
    }
}

impl Decode for Resource {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(_bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        // For now, return a simple error since complex decoding isn't needed
        // This can be improved later when we have proper DecodeWithRemainder implementations
        Err(ssz::DecodeError::BytesInvalid("Complex Resource decoding not yet implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lambda::base::Value;

    #[test]
    fn test_ephemeral_resource_creation() {
        let domain = DomainId::from_bytes([1; 32]);
        
        // Create a regular resource
        let regular_resource = Resource::simple(
            MachineValue::Int(42),
            TypeInner::Base(crate::lambda::BaseType::Int),
            domain
        );
        assert!(!regular_resource.ephemeral);
        
        // Create an ephemeral resource using the convenience method
        let ephemeral_resource = Resource::ephemeral(
            "temp_computation",
            domain,
            "ComputationResult",
            Value::Int(100)
        );
        assert!(ephemeral_resource.ephemeral);
        assert_eq!(ephemeral_resource.quantity, 1);
        assert_eq!(ephemeral_resource.name.as_str(), "temp_computation");
        assert_eq!(ephemeral_resource.label.as_str(), "ComputationResult");
        
        // Create an ephemeral resource using the full constructor
        let full_ephemeral = Resource::new(
            "full_ephemeral",
            domain,
            "TempData",
            5,
            Value::Unit,
            Value::Bool(true),
            CausalProof::genesis("test"),
            None,
            true // ephemeral
        );
        assert!(full_ephemeral.ephemeral);
        assert_eq!(full_ephemeral.quantity, 5);
    }

    #[test]
    fn test_ephemeral_resource_heap_operations() {
        let mut heap = ResourceHeap::new();
        
        // Create and allocate ephemeral resource
        let ephemeral_resource = Resource::ephemeral(
            "temp_data",
            DomainId::from_bytes([2; 32]),
            "TempType",
            Value::Int(123)
        );
        
        let resource_id = heap.alloc_full_resource(ephemeral_resource.clone());
        
        // Verify resource exists and is available
        assert!(heap.is_available(resource_id));
        assert_eq!(heap.total_resources(), 1);
        assert_eq!(heap.available_count(), 1);
        
        // Check that we can peek at the ephemeral resource
        let peeked = heap.peek_resource(resource_id).unwrap();
        assert!(peeked.ephemeral);
        assert_eq!(peeked.name.as_str(), "temp_data");
        
        // Consume the ephemeral resource
        let consumed_value = heap.consume_resource(resource_id).unwrap();
        assert_eq!(consumed_value, MachineValue::Int(123));
        
        // Verify resource is now consumed
        assert!(!heap.is_available(resource_id));
        assert!(heap.is_consumed(resource_id));
        assert_eq!(heap.consumed_count(), 1);
        assert_eq!(heap.available_count(), 0);
    }
} 