//! Linear resource management for the minimal instruction set
//!
//! This module implements linear resource tracking with integrated nullifiers,
//! providing cryptographic proof of consumption for the zkVM environment.
//! 
//! **Mathematical Foundation**:
//! - Resources are objects in our symmetric monoidal closed category
//! - Nullifiers are morphisms that prove consumption without revealing resource identity
//! - Consumption is a linear transformation: Resource → (Value, Nullifier)
//! - The Lamport clock provides deterministic ordering for ZK proofs

use crate::{
    machine::value::MachineValue,
    system::{
        content_addressing::EntityId,
        deterministic::DeterministicSystem,
    },
    lambda::TypeInner,
};
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};
use ssz::{Encode, Decode};
use sha2::{Sha256, Digest};

/// Resource identifier (wrapper around EntityId)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub EntityId);

impl ResourceId {
    /// Create a new ResourceId from a unique identifier
    pub fn new(id: u64) -> Self {
        // Create content from the ID for content addressing
        let content = id; // Use the u64 directly for content addressing
        ResourceId(EntityId::from_content(&content))
    }
    
    pub fn inner(&self) -> &EntityId {
        &self.0
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceId({})", self.0)
    }
}

/// Zero-knowledge nullifier for proving resource consumption
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nullifier {
    /// Cryptographic commitment to the consumed resource
    pub commitment: [u8; 32],
    
    /// Lamport timestamp when consumption occurred
    pub lamport_time: u64,
    
    /// Nullifier hash (prevents double-spending)
    pub nullifier_hash: [u8; 32],
    
    /// Optional: ZK proof of valid consumption
    pub proof: Option<Vec<u8>>,
}

/// Linear resource (completely immutable)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    /// Unique identifier for this resource
    pub id: ResourceId,
    
    /// Type of the resource
    pub resource_type: TypeInner,
    
    /// Current value of the resource
    pub value: MachineValue,
    
    /// Lamport timestamp when resource was created
    pub created_at: u64,
    
    /// Secret key for nullifier generation (ephemeral, not serialized)
    #[serde(skip)]
    pub nullifier_key: [u8; 32],
}

/// Resource consumption result with nullifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumptionResult {
    /// The extracted value from the consumed resource
    pub value: MachineValue,
    
    /// Nullifier proving consumption
    pub nullifier: Nullifier,
    
    /// Lamport timestamp of consumption
    pub consumed_at: u64,
}

impl Resource {
    /// Create a new immutable resource with nullifier capability
    pub fn new(resource_type: MachineValue, init_value: MachineValue, allocation_counter: u64) -> Self {
        let lamport_time = crate::system::deterministic::deterministic_lamport_time();
        
        // Generate deterministic ID and nullifier key
        let id = ResourceId::new(allocation_counter);
        let nullifier_key = {
            let mut key_input = Vec::new();
            key_input.extend_from_slice(&allocation_counter.to_le_bytes());
            key_input.extend_from_slice(&lamport_time.to_le_bytes());
            // Add some simple entropy based on the value type
            key_input.extend_from_slice(b"nullifier_key_generation");
            Sha256::digest(&key_input).into()
        };
        
        Self {
            id,
            resource_type: resource_type.get_type(),
            value: init_value,
            created_at: lamport_time,
            nullifier_key,
        }
    }
    
    /// Generate a nullifier for consumption (pure function)
    pub fn generate_nullifier(&self, consumption_time: u64) -> Result<Nullifier, ResourceError> {
        // Create commitment to resource (hiding resource identity)
        let mut commitment_input = Vec::new();
        commitment_input.extend_from_slice(self.id.inner().as_bytes());
        commitment_input.extend_from_slice(&self.created_at.to_le_bytes());
        commitment_input.extend_from_slice(&self.nullifier_key);
        
        let commitment: [u8; 32] = Sha256::digest(&commitment_input).into();
        
        // Create nullifier hash (prevents double-spending)
        let mut nullifier_input = Vec::new();
        nullifier_input.extend_from_slice(&commitment);
        nullifier_input.extend_from_slice(&consumption_time.to_le_bytes());
        nullifier_input.extend_from_slice(&self.nullifier_key);
        
        let nullifier_hash: [u8; 32] = Sha256::digest(&nullifier_input).into();
        
        Ok(Nullifier {
            commitment,
            lamport_time: consumption_time,
            nullifier_hash,
            proof: None, // ZK proof would be generated here in full implementation
        })
    }
    
    /// Calculate approximate size for gas calculation
    pub fn calculate_size(&self) -> u64 {
        match &self.value {
            MachineValue::Unit => 1,
            MachineValue::Bool(_) => 1,
            MachineValue::Int(_) => 4,
            MachineValue::Symbol(s) => s.as_str().len() as u64,
            MachineValue::Product(l, r) => {
                Self::calculate_value_size(l) + Self::calculate_value_size(r)
            }
            MachineValue::Sum { value, .. } => {
                8 + Self::calculate_value_size(value) // tag + value
            }
            MachineValue::Tensor(l, r) => {
                Self::calculate_value_size(l) + Self::calculate_value_size(r)
            }
            MachineValue::ResourceRef(_) => 32, // EntityId size
            MachineValue::MorphismRef(_) => 4,  // RegisterId size
            MachineValue::Type(_) => 16,        // Approximate type size
            MachineValue::Channel(_) => 64,     // Approximate channel size
            MachineValue::Function { params, body, captured_env } => {
                let params_size = params.len() as u64 * 4;
                let body_size = body.len() as u64 * 32; // Approximate instruction size
                let env_size = captured_env.iter()
                    .map(|(_, v)| Self::calculate_value_size(v))
                    .sum::<u64>();
                params_size + body_size + env_size
            }
        }
    }
    
    fn calculate_value_size(value: &MachineValue) -> u64 {
        match value {
            MachineValue::Unit => 1,
            MachineValue::Bool(_) => 1,
            MachineValue::Int(_) => 4,
            MachineValue::Symbol(s) => s.as_str().len() as u64,
            MachineValue::Product(l, r) => {
                Self::calculate_value_size(l) + Self::calculate_value_size(r)
            }
            MachineValue::Sum { value, .. } => {
                8 + Self::calculate_value_size(value)
            }
            MachineValue::Tensor(l, r) => {
                Self::calculate_value_size(l) + Self::calculate_value_size(r)
            }
            MachineValue::ResourceRef(_) => 32,
            MachineValue::MorphismRef(_) => 4,
            MachineValue::Type(_) => 16,
            MachineValue::Channel(_) => 64,
            MachineValue::Function { params, body, captured_env } => {
                let params_size = params.len() as u64 * 4;
                let body_size = body.len() as u64 * 32;
                let env_size = captured_env.iter()
                    .map(|(_, v)| Self::calculate_value_size(v))
                    .sum::<u64>();
                params_size + body_size + env_size
            }
        }
    }
}

/// Nullifier set for tracking consumed resources
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NullifierSet {
    /// Set of nullifier hashes (prevents double-spending)
    nullifiers: BTreeMap<[u8; 32], Nullifier>,
    
    /// Lamport clock for ordering
    current_time: u64,
}

impl NullifierSet {
    /// Create a new nullifier set
    pub fn new() -> Self {
        Self {
            nullifiers: BTreeMap::new(),
            current_time: 0,
        }
    }
    
    /// Add a nullifier to the set
    pub fn add_nullifier(&mut self, nullifier: Nullifier) -> Result<(), ResourceError> {
        // Check for double-spending
        if self.nullifiers.contains_key(&nullifier.nullifier_hash) {
            return Err(ResourceError::DoubleSpending(nullifier.nullifier_hash));
        }
        
        // Update Lamport clock
        self.current_time = self.current_time.max(nullifier.lamport_time) + 1;
        
        // Add nullifier
        self.nullifiers.insert(nullifier.nullifier_hash, nullifier);
        
        Ok(())
    }
    
    /// Check if a nullifier exists (resource was consumed)
    pub fn contains(&self, nullifier_hash: &[u8; 32]) -> bool {
        self.nullifiers.contains_key(nullifier_hash)
    }
    
    /// Get all nullifiers (for ZK proof generation)
    pub fn get_all(&self) -> impl Iterator<Item = &Nullifier> {
        self.nullifiers.values()
    }
    
    /// Get nullifier count
    pub fn len(&self) -> usize {
        self.nullifiers.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nullifiers.is_empty()
    }
    
    /// Get current Lamport time
    pub fn current_time(&self) -> u64 {
        self.current_time
    }

    /// Generate ZK proof of nullifier set validity
    pub fn generate_proof(&self) -> Vec<u8> {
        // In a full implementation, this would generate a ZK-SNARK proof
        // that all nullifiers in the set are valid without revealing resource details
        
        // For now, return a deterministic "proof" based on the nullifier set
        let mut proof_input = Vec::new();
        for nullifier in self.nullifiers.values() {
            proof_input.extend_from_slice(&nullifier.commitment);
            proof_input.extend_from_slice(&nullifier.nullifier_hash);
        }
        
        Sha256::digest(&proof_input).to_vec()
    }
}

/// Resource dependency tracking for lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDependency {
    /// The resource that depends on another
    pub dependent: ResourceId,
    
    /// The resource being depended upon
    pub dependency: ResourceId,
    
    /// Type of dependency relationship
    pub dependency_type: DependencyType,
    
    /// When this dependency was created
    pub created_at: u64,
}

/// Types of resource dependencies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Resource A contains a reference to resource B
    Contains,
    
    /// Resource A was derived from resource B
    DerivedFrom,
    
    /// Resource A requires resource B to remain valid
    Requires,
    
    /// Resource A and B must be consumed together
    LinkedConsumption,
    
    /// Resource A is a channel endpoint paired with resource B
    ChannelPair,
}

impl PartialEq for ResourceDependency {
    fn eq(&self, other: &Self) -> bool {
        self.dependent == other.dependent && self.dependency == other.dependency
    }
}

impl Eq for ResourceDependency {}

impl PartialOrd for ResourceDependency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResourceDependency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.dependent.cmp(&other.dependent)
            .then_with(|| self.dependency.cmp(&other.dependency))
            .then_with(|| self.dependency_type.cmp(&other.dependency_type))
    }
}

impl PartialOrd for DependencyType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DependencyType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use DependencyType::*;
        match (self, other) {
            (Contains, Contains) => std::cmp::Ordering::Equal,
            (Contains, _) => std::cmp::Ordering::Less,
            (DerivedFrom, Contains) => std::cmp::Ordering::Greater,
            (DerivedFrom, DerivedFrom) => std::cmp::Ordering::Equal,
            (DerivedFrom, _) => std::cmp::Ordering::Less,
            (Requires, Contains) | (Requires, DerivedFrom) => std::cmp::Ordering::Greater,
            (Requires, Requires) => std::cmp::Ordering::Equal,
            (Requires, _) => std::cmp::Ordering::Less,
            (LinkedConsumption, ChannelPair) => std::cmp::Ordering::Less,
            (LinkedConsumption, LinkedConsumption) => std::cmp::Ordering::Equal,
            (LinkedConsumption, _) => std::cmp::Ordering::Greater,
            (ChannelPair, ChannelPair) => std::cmp::Ordering::Equal,
            (ChannelPair, _) => std::cmp::Ordering::Greater,
        }
    }
}

/// Resource manager for tracking linear resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManager {
    /// Active resources (immutable)
    resources: BTreeMap<ResourceId, Resource>,
    
    /// Nullifier set for consumed resources
    nullifiers: NullifierSet,
    
    /// Resource allocation counter
    allocation_counter: u64,
    
    /// Total memory used by resources
    total_memory: u64,
    
    /// Resource dependency graph
    dependencies: BTreeMap<ResourceId, BTreeSet<ResourceDependency>>,
    
    /// Reverse dependency lookup (what depends on this resource)
    reverse_dependencies: BTreeMap<ResourceId, BTreeSet<ResourceId>>,
}

/// Resource store (alias for ResourceManager for compatibility)
pub type ResourceStore = ResourceManager;

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
            nullifiers: NullifierSet::new(),
            allocation_counter: 0,
            total_memory: 0,
            dependencies: BTreeMap::new(),
            reverse_dependencies: BTreeMap::new(),
        }
    }
    
    /// Allocate a new resource
    pub fn allocate(&mut self, resource_type: MachineValue, init_value: MachineValue) -> ResourceId {
        // Increment allocation counter first
        self.allocation_counter += 1;
        
        let resource = Resource::new(resource_type, init_value, self.allocation_counter);
        let id = resource.id;
        
        self.total_memory += resource.calculate_size();
        
        self.resources.insert(id, resource);
        id
    }
    
    /// Allocate a placeholder resource (for testing)
    pub fn allocate_placeholder(&mut self, _det_sys: &mut DeterministicSystem) -> Result<ResourceId, ResourceError> {
        let placeholder_type = MachineValue::Unit;
        let placeholder_value = MachineValue::Unit;
        Ok(self.allocate(placeholder_type, placeholder_value))
    }
    
    /// Consume a resource with nullifier generation and dependency validation
    pub fn consume(&mut self, id: ResourceId) -> Result<ConsumptionResult, ResourceError> {
        // Validate that consumption is allowed based on dependencies
        if !self.can_consume(&id)? {
            return Err(ResourceError::OperationFailed(
                format!("Cannot consume resource {:?} due to dependency constraints", id)
            ));
        }
        
        // Check if resource exists
        let resource = self.resources.get(&id)
            .ok_or(ResourceError::NotFound(id))?;
        
        let consumption_time = crate::system::deterministic::deterministic_lamport_time();
        
        // Generate nullifier for this consumption
        let nullifier = resource.generate_nullifier(consumption_time)?;
        
        // Check for double-spending by trying to add nullifier
        self.nullifiers.add_nullifier(nullifier.clone())?;
        
        // Extract the value and remove from active resources
        let consumed_resource = self.resources.remove(&id).unwrap();
        self.total_memory -= consumed_resource.calculate_size();
        
        // Clean up dependencies involving this resource
        self.cleanup_dependencies(&id);
        
        Ok(ConsumptionResult {
            value: consumed_resource.value,
            nullifier,
            consumed_at: consumption_time,
        })
    }
    
    /// Peek at a resource without consuming it
    pub fn peek(&self, id: &ResourceId) -> Result<&MachineValue, ResourceError> {
        if let Some(resource) = self.resources.get(id) {
            Ok(&resource.value)
        } else {
            Err(ResourceError::NotFound(*id))
        }
    }
    
    /// Check if a resource is available (exists and not consumed)
    pub fn is_available(&self, id: &ResourceId) -> bool {
        self.resources.contains_key(id)
    }
    
    /// Check if a resource has been consumed (nullifier exists)
    pub fn is_consumed(&self, id: &ResourceId) -> bool {
        // To check if consumed, we'd need to generate the nullifier hash
        // and check if it exists in the nullifier set.
        // For now, we consider it consumed if it's not in active resources
        // and we have some nullifiers (simplified check)
        !self.resources.contains_key(id) && !self.nullifiers.is_empty()
    }
    
    /// Get resource count
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }
    
    /// Get total memory usage
    pub fn total_memory(&self) -> u64 {
        self.total_memory
    }
    
    /// Get the nullifier set (for ZK proof generation)
    pub fn nullifiers(&self) -> &NullifierSet {
        &self.nullifiers
    }
    
    /// Get mutable access to nullifiers (for verification)
    pub fn nullifiers_mut(&mut self) -> &mut NullifierSet {
        &mut self.nullifiers
    }
    
    /// Generate ZK proof of all resource operations
    pub fn generate_resource_proof(&self) -> Vec<u8> {
        self.nullifiers.generate_proof()
    }
    
    /// Verify a nullifier against the set
    pub fn verify_nullifier(&self, nullifier_hash: &[u8; 32]) -> bool {
        self.nullifiers.contains(nullifier_hash)
    }
    
    /// Get allocation statistics
    pub fn allocation_stats(&self) -> AllocationStats {
        AllocationStats {
            total_allocated: self.allocation_counter,
            active_count: self.resources.len() as u64,
            consumed_count: self.nullifiers.len() as u64,
            total_memory: self.total_memory,
        }
    }
    
    /// Get all active resource IDs
    pub fn active_resources(&self) -> Vec<ResourceId> {
        self.resources.keys().cloned().collect()
    }
    
    /// Create a snapshot of the resource store state
    pub fn snapshot(&self) -> ResourceStoreSnapshot {
        ResourceStoreSnapshot {
            resource_count: self.resources.len(),
            total_memory: self.total_memory,
            allocation_counter: self.allocation_counter,
            nullifier_count: self.nullifiers.len(),
        }
    }
    
    /// Add a dependency relationship between resources
    pub fn add_dependency(&mut self, dependent: ResourceId, dependency: ResourceId, dep_type: DependencyType) -> Result<(), ResourceError> {
        // Verify both resources exist
        if !self.resources.contains_key(&dependent) {
            return Err(ResourceError::NotFound(dependent));
        }
        if !self.resources.contains_key(&dependency) {
            return Err(ResourceError::NotFound(dependency));
        }
        
        // Create dependency record
        let dep_record = ResourceDependency {
            dependent,
            dependency,
            dependency_type: dep_type,
            created_at: 0, // Simplified for now
        };
        
        // Add to dependency graph
        self.dependencies.entry(dependent)
            .or_default()
            .insert(dep_record);
        
        // Add to reverse dependency lookup
        self.reverse_dependencies.entry(dependency)
            .or_default()
            .insert(dependent);
        
        Ok(())
    }
    
    /// Get all resources that depend on the given resource
    pub fn get_dependents(&self, resource_id: &ResourceId) -> Vec<ResourceId> {
        self.reverse_dependencies.get(resource_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get all resources that the given resource depends on
    pub fn get_dependencies(&self, resource_id: &ResourceId) -> Vec<ResourceDependency> {
        self.dependencies.get(resource_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Check if consuming a resource would violate dependencies
    pub fn can_consume(&self, resource_id: &ResourceId) -> Result<bool, ResourceError> {
        // Check if any resources depend on this one
        let dependents = self.get_dependents(resource_id);
        
        for dependent_id in dependents {
            // Get the dependency type
            if let Some(deps) = self.dependencies.get(&dependent_id) {
                for dep in deps {
                    if dep.dependency == *resource_id {
                        match dep.dependency_type {
                            DependencyType::Requires => {
                                // Cannot consume if another resource requires it
                                return Ok(false);
                            }
                            DependencyType::LinkedConsumption => {
                                // Must consume both together - check if dependent is also being consumed
                                // For now, allow consumption but this should be coordinated
                                continue;
                            }
                            DependencyType::ChannelPair => {
                                // Channel pairs should be consumed together
                                return Ok(false);
                            }
                            _ => {
                                // Other dependency types don't prevent consumption
                                continue;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(true)
    }
    
    /// Validate consumption order based on dependencies
    pub fn validate_consumption_order(&self, resource_id: &ResourceId) -> Result<Vec<ResourceId>, ResourceError> {
        let mut consumption_order = Vec::new();
        let mut visited = BTreeSet::new();
        
        self.build_consumption_order(resource_id, &mut consumption_order, &mut visited)?;
        
        Ok(consumption_order)
    }
    
    /// Recursively build the consumption order based on dependencies
    fn build_consumption_order(&self, resource_id: &ResourceId, order: &mut Vec<ResourceId>, visited: &mut BTreeSet<ResourceId>) -> Result<(), ResourceError> {
        if visited.contains(resource_id) {
            // Cycle detected - this is an error in dependency management
            return Err(ResourceError::OperationFailed(
                format!("Circular dependency detected involving resource {:?}", resource_id)
            ));
        }
        
        visited.insert(*resource_id);
        
        // First, add all dependencies that must be consumed after this resource
        if let Some(deps) = self.dependencies.get(resource_id) {
            for dep in deps {
                match dep.dependency_type {
                    DependencyType::DerivedFrom => {
                        // Derived resources should be consumed before their sources
                        self.build_consumption_order(&dep.dependency, order, visited)?;
                    }
                    DependencyType::LinkedConsumption => {
                        // Linked resources must be consumed together
                        if !order.contains(&dep.dependency) {
                            self.build_consumption_order(&dep.dependency, order, visited)?;
                        }
                    }
                    _ => {
                        // Other types don't affect consumption order
                    }
                }
            }
        }
        
        // Add this resource to the consumption order
        if !order.contains(resource_id) {
            order.push(*resource_id);
        }
        
        visited.remove(resource_id);
        Ok(())
    }
    
    /// Remove all dependencies involving a consumed resource
    fn cleanup_dependencies(&mut self, consumed_resource: &ResourceId) {
        // Remove from dependency graph
        self.dependencies.remove(consumed_resource);
        
        // Remove from reverse dependencies
        self.reverse_dependencies.remove(consumed_resource);
        
        // Remove this resource from other resources' dependency lists
        for (_, deps) in self.dependencies.iter_mut() {
            deps.retain(|dep| dep.dependency != *consumed_resource);
        }
        
        // Remove this resource from reverse dependency lists
        for (_, reverse_deps) in self.reverse_dependencies.iter_mut() {
            reverse_deps.remove(consumed_resource);
        }
    }

    /// Create a simple resource (for bounded execution)
    pub fn create_resource(&mut self) -> ResourceId {
        let placeholder_type = MachineValue::Unit;
        let placeholder_value = MachineValue::Unit;
        self.allocate(placeholder_type, placeholder_value)
    }
    
    /// Simple resource consumption (for bounded execution)
    pub fn consume_resource(&mut self, id: ResourceId) {
        // Simple consumption without error handling for bounded execution
        let _ = self.consume(id);
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of resource store state for execution tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStoreSnapshot {
    pub resource_count: usize,
    pub total_memory: u64,
    pub allocation_counter: u64,
    pub nullifier_count: usize,
}

/// Resource allocation statistics
#[derive(Debug, Clone)]
pub struct AllocationStats {
    pub total_allocated: u64,
    pub active_count: u64,
    pub consumed_count: u64,
    pub total_memory: u64,
}

/// Resource-related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    /// Resource not found
    NotFound(ResourceId),
    
    /// Resource already consumed
    AlreadyConsumed(ResourceId),
    
    /// Double-spending detected (nullifier already exists)
    DoubleSpending([u8; 32]),
    
    /// Resource type mismatch
    TypeMismatch {
        expected: String,
        found: String,
    },
    
    /// Resource operation failed
    OperationFailed(String),
    
    /// ZK proof verification failed
    ProofVerificationFailed,
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::NotFound(id) => write!(f, "Resource not found: {:?}", id),
            ResourceError::AlreadyConsumed(id) => write!(f, "Resource already consumed: {:?}", id),
            ResourceError::DoubleSpending(hash) => write!(f, "Double-spending detected: {:?}", hash),
            ResourceError::TypeMismatch { expected, found } => {
                write!(f, "Resource type mismatch: expected {}, found {}", expected, found)
            }
            ResourceError::OperationFailed(msg) => write!(f, "Resource operation failed: {}", msg),
            ResourceError::ProofVerificationFailed => write!(f, "ZK proof verification failed"),
        }
    }
}

impl std::error::Error for ResourceError {}

// SSZ encoding for nullifiers
impl Encode for Nullifier {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        32 + 8 + 32 + 4 + self.proof.as_ref().map_or(0, |p| p.len())
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.commitment.ssz_append(buf);
        self.lamport_time.ssz_append(buf);
        self.nullifier_hash.ssz_append(buf);
        self.proof.ssz_append(buf);
    }
}

impl Decode for Nullifier {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        if bytes.len() < 72 { // 32 + 8 + 32
            return Err(ssz::DecodeError::BytesInvalid("Nullifier too short".to_string()));
        }
        
        let mut offset = 0;
        
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;
        
        let lamport_time = u64::from_ssz_bytes(&bytes[offset..offset + 8])?;
        offset += 8;
        
        let mut nullifier_hash = [0u8; 32];
        nullifier_hash.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;
        
        let proof = if offset < bytes.len() {
            Some(bytes[offset..].to_vec())
        } else {
            None
        };
        
        Ok(Nullifier {
            commitment,
            lamport_time,
            nullifier_hash,
            proof,
        })
    }
}

impl Nullifier {
    /// Create a simple nullifier from a hash (for basic use cases)
    pub fn from_hash(hash: [u8; 32]) -> Self {
        let lamport_time = crate::system::deterministic::deterministic_lamport_time();
        
        Self {
            commitment: hash,
            lamport_time,
            nullifier_hash: hash, // Use same hash for both commitment and nullifier
            proof: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_creation() {
        let resource_type = MachineValue::Type(TypeInner::Base(crate::lambda::BaseType::Int));
        let init_value = MachineValue::Int(42);
        
        let resource = Resource::new(resource_type, init_value.clone(), 1);
        
        assert_eq!(resource.value, init_value);
        assert!(resource.nullifier_key != [0u8; 32]); // Should have a real nullifier key
    }
    
    #[test]
    fn test_resource_nullifier_generation() {
        let resource_type = MachineValue::Type(TypeInner::Base(crate::lambda::BaseType::Int));
        let init_value = MachineValue::Int(42);
        
        let resource = Resource::new(resource_type, init_value.clone(), 1);
        let consumption_time = crate::system::deterministic::deterministic_lamport_time();
        let nullifier = resource.generate_nullifier(consumption_time).unwrap();
        
        assert_ne!(nullifier.commitment, [0u8; 32]);
        assert_ne!(nullifier.nullifier_hash, [0u8; 32]);
        assert_eq!(nullifier.lamport_time, consumption_time);
    }
    
    #[test]
    fn test_nullifier_set() {
        let mut nullifier_set = NullifierSet::new();
        
        let resource_type = MachineValue::Type(TypeInner::Base(crate::lambda::BaseType::Int));
        let init_value = MachineValue::Int(42);
        
        let resource = Resource::new(resource_type, init_value, 1);
        let consumption_time = crate::system::deterministic::deterministic_lamport_time();
        let nullifier = resource.generate_nullifier(consumption_time).unwrap();
        
        // Add nullifier to set
        nullifier_set.add_nullifier(nullifier.clone()).unwrap();
        
        // Check nullifier exists
        assert!(nullifier_set.contains(&nullifier.nullifier_hash));
        assert_eq!(nullifier_set.len(), 1);
        
        // Try to add same nullifier again (should fail)
        let double_spend_result = nullifier_set.add_nullifier(nullifier);
        assert!(matches!(double_spend_result, Err(ResourceError::DoubleSpending(_))));
    }
    
    #[test]
    fn test_resource_manager_with_nullifiers() {
        let mut manager = ResourceManager::new();
        
        let resource_type = MachineValue::Type(TypeInner::Base(crate::lambda::BaseType::Int));
        let init_value = MachineValue::Int(42);
        
        // Allocate resource
        let id = manager.allocate(resource_type, init_value.clone());
        assert_eq!(manager.resource_count(), 1);
        assert!(manager.is_available(&id));
        
        // Peek at resource
        let peeked = manager.peek(&id).unwrap();
        assert_eq!(peeked, &init_value);
        
        // Consume resource with nullifier generation
        let consumption_result = manager.consume(id).unwrap();
        assert_eq!(consumption_result.value, init_value);
        assert_eq!(manager.resource_count(), 0);
        assert_eq!(manager.nullifiers().len(), 1);
        assert!(!manager.is_available(&id));
        
        // Verify nullifier exists
        assert!(manager.verify_nullifier(&consumption_result.nullifier.nullifier_hash));
        
        // Try to consume again (should fail)
        let result = manager.consume(id);
        assert!(matches!(result, Err(ResourceError::NotFound(_))));
    }

    #[test]
    fn test_zk_proof_generation() {
        let mut manager = ResourceManager::new();
        
        // Create and consume multiple resources
        for i in 0..5 {
            let resource_type = MachineValue::Type(TypeInner::Base(crate::lambda::BaseType::Int));
            let init_value = MachineValue::Int(i);
            
            let id = manager.allocate(resource_type, init_value);
            manager.consume(id).unwrap();
        }
        
        // Generate proof of all operations
        let proof = manager.generate_resource_proof();
        assert!(!proof.is_empty());
        assert_eq!(manager.nullifiers().len(), 5);
        
        // Proof should be deterministic
        let proof2 = manager.generate_resource_proof();
        assert_eq!(proof, proof2);
    }

    #[test]
    fn test_lamport_clock_ordering() {
        let mut nullifier_set = NullifierSet::new();
        let initial_time = nullifier_set.current_time();
        
        // Create resources with different timestamps
        let resource1 = Resource::new(
            MachineValue::Type(TypeInner::Base(crate::lambda::BaseType::Int)),
            MachineValue::Int(1),
            1
        );
        let resource2 = Resource::new(
            MachineValue::Type(TypeInner::Base(crate::lambda::BaseType::Int)),
            MachineValue::Int(2),
            2
        );
        
        let time1 = crate::system::deterministic::deterministic_lamport_time();
        let time2 = crate::system::deterministic::deterministic_lamport_time();
        
        let nullifier1 = resource1.generate_nullifier(time1).unwrap();
        let nullifier2 = resource2.generate_nullifier(time2).unwrap();
        
        // Add nullifiers in order
        nullifier_set.add_nullifier(nullifier1).unwrap();
        nullifier_set.add_nullifier(nullifier2).unwrap();
        
        // Lamport clock should have advanced
        assert!(nullifier_set.current_time() > initial_time);
    }
    
    #[test]
    fn test_double_spending_prevention() {
        let mut manager = ResourceManager::new();
        
        let resource_type = MachineValue::Type(TypeInner::Base(crate::lambda::BaseType::Int));
        let init_value = MachineValue::Int(42);
        
        // Allocate resource
        let id = manager.allocate(resource_type, init_value);
        
        // Consume resource
        let consumption_result = manager.consume(id).unwrap();
        
        // Try to add the same nullifier again (simulate double-spending)
        let double_spend_result = manager.nullifiers_mut()
            .add_nullifier(consumption_result.nullifier);
        
        assert!(matches!(double_spend_result, Err(ResourceError::DoubleSpending(_))));
    }
} 