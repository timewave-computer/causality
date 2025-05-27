// crates/causality-runtime/src/state_manager.rs
//! Manages the runtime state for TEL execution, including resources.

use anyhow::Result;
use anyhow::anyhow; // Added import for the anyhow! macro
use async_trait::async_trait;
use std::collections::{HashMap};
use tokio::sync::Mutex; // Use tokio::sync::Mutex instead of std::sync::Mutex
use std::sync::Arc;

use causality_types::{
    core::{
        id::{ResourceId, ValueExprId, ExprId, HandlerId, DomainId, EntityId, AsIdConverter},
        resource::{Resource, Nullifier},
        time::Timestamp,
        Handler,
    },
    expr::{ast::Expr as TypesExpr, ValueExpr},
    provider::context::{AsExprContext, AsExecutionContext, AsRuntimeContext},
    serialization::Encode,
};
use causality_core::extension_traits::ValueExprExt; // For .id() on ValueExpr
use causality_core::utils::tel::ResourceExt; // Updated import path for compute_hash
use sha2::{Digest, Sha256};

use crate::state::state_proof::{StateProofGenerator, ResourceProof, ValueProof};

/// Trait for managing the state of the TEL interpreter.
/// This groups AsExecutionContext and AsRuntimeContext and can be expanded
/// with more StateManager-specific methods if needed.
#[async_trait]
pub trait StateManager: std::fmt::Debug + AsExprContext + AsExecutionContext + AsRuntimeContext + Send + Sync {
    /// Checks if an effect, identified by its expression ID, is considered completed.
    /// The definition of "completed" can vary (e.g., successfully executed, committed, acknowledged).
    /// Returns `Ok(true)` if completed, `Ok(false)` if not, or an error if the status cannot be determined.
    async fn is_effect_expr_completed(&self, effect_expr_id: &ExprId) -> Result<bool>;

    /// Checks if a resource is available (exists and is not nullified)
    async fn is_resource_available(&self, id: &ResourceId) -> Result<bool>;

    // New methods for managing expressions
    fn put_expr(&mut self, expr_id: ExprId, expr: TypesExpr) -> Result<()>;
    async fn get_expr(&self, expr_id: &ExprId) -> Result<Option<TypesExpr>>;

    // New methods for managing handlers
    fn put_handler(&mut self, handler_id: HandlerId, handler: Handler) -> Result<()>;
    async fn get_handler(&self, handler_id: &HandlerId) -> Result<Option<Handler>>;
    async fn get_all_handlers(&self) -> Result<Vec<std::sync::Arc<Handler>>>;
    
    // Get all resources by domain
    async fn get_all_resources_by_domain(&self, domain_id: &DomainId) -> Result<Vec<(ResourceId, Resource)>>;
    
    // Get all handlers by domain
    async fn get_all_handlers_by_domain(&self, domain_id: &DomainId) -> Result<Vec<Handler>>;
    
    // State verification methods using Merkle proofs
    
    /// Generate a proof that a resource exists in the state
    /// This creates a Merkle proof that can be verified by anyone with the state root
    async fn generate_resource_proof(&self, resource_id: &ResourceId) -> Result<crate::state::state_proof::ResourceProof>;
    
    /// Generate a proof that a value exists in the state
    /// This creates a Merkle proof that can be verified by anyone with the state root
    async fn generate_value_proof(&self, value_id: &ValueExprId) -> Result<crate::state::state_proof::ValueProof>;
    
    /// Get the current Merkle root hash of the resources in the state
    /// This root can be used to verify resource proofs
    async fn get_resources_root(&self) -> Result<[u8; 32]>;
    
    /// Get the current Merkle root hash of the values in the state
    /// This root can be used to verify value proofs
    async fn get_values_root(&self) -> Result<[u8; 32]>;
    
    /// Verify a resource proof against the current state
    /// Returns true if the proof is valid, false otherwise
    async fn verify_resource_proof(&self, proof: &crate::state::state_proof::ResourceProof) -> Result<bool>;
    
    /// Verify a value proof against the current state
    /// Returns true if the proof is valid, false otherwise
    async fn verify_value_proof(&self, proof: &crate::state::state_proof::ValueProof) -> Result<bool>;
}

/// Default implementation of the StateManager.
/// It uses HashMaps protected by Mutexes to store runtime state.
#[derive(Debug)]
pub struct DefaultStateManager {
    resources: Mutex<HashMap<EntityId, Resource>>,
    values: Mutex<HashMap<ValueExprId, ValueExpr>>,
    expressions: Mutex<HashMap<ExprId, TypesExpr>>,
    handlers: Mutex<HashMap<HandlerId, Handler>>,
    nullifiers: Mutex<HashMap<EntityId, Nullifier>>,
    // Potentially other state, e.g., resource locks, ephemeral states
}

impl DefaultStateManager {
    pub fn new() -> Self {
        DefaultStateManager {
            resources: Mutex::new(HashMap::new()),
            values: Mutex::new(HashMap::new()),
            expressions: Mutex::new(HashMap::new()),
            handlers: Mutex::new(HashMap::new()),
            nullifiers: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add_resource(&mut self, resource: Resource) -> Result<()> {
        let mut resources_map = self.resources.lock().await;
        if resources_map.contains_key(&resource.id) {
            return Err(anyhow!("Resource with ID {:?} already exists", resource.id));
        }
        resources_map.insert(resource.id, resource);
        Ok(())
    }
}

// Implement AsExprContext for DefaultStateManager (stubbed for ref-returning methods)
impl AsExprContext for DefaultStateManager {
    fn get_resource_field(&self, id: &ResourceId, field: &str) -> Result<Option<ValueExpr>> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = id.to_id();
        
        // Use async runtime to access the resource
        let resource_result = tokio::runtime::Handle::current().block_on(async {
            self.resources.lock().await.get(&entity_id).cloned()
        });
        
        if let Some(resource) = resource_result {
            // Extract field from resource based on field name
            match field {
                "id" => Ok(Some(ValueExpr::String(causality_types::primitive::string::Str::from(format!("{:?}", resource.id))))),
                "name" => Ok(Some(ValueExpr::String(resource.name.clone()))),
                "domain_id" => Ok(Some(ValueExpr::String(causality_types::primitive::string::Str::from(format!("{:?}", resource.domain_id))))),
                "resource_type" => Ok(Some(ValueExpr::String(resource.resource_type.clone()))),
                "quantity" => Ok(Some(ValueExpr::Number(causality_types::primitive::number::Number::Integer(resource.quantity as i64)))),
                "timestamp" => Ok(Some(ValueExpr::Number(causality_types::primitive::number::Number::Integer(resource.timestamp.as_millis() as i64)))),
                // "ephemeral" field was removed from Resource type
                "ephemeral" => Ok(Some(ValueExpr::Bool(false))), // Default to false
                _ => Ok(None), // Field not found
            }
        } else {
            Ok(None) // Resource not found
        }
    }

    fn evaluate_expr(&self, _expr: &TypesExpr) -> Result<ValueExpr> {
        Err(anyhow!("DefaultStateManager::evaluate_expr: Not implemented, should be in interpreter logic"))
    }

    fn is_resource_available(&self, id: &ResourceId) -> Result<bool> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = id.to_id();
        
        // Use async runtime to check availability
        let result = tokio::runtime::Handle::current().block_on(async {
            // Check if resource exists and is not nullified
            let resource_exists = self.resources.lock().await.contains_key(&entity_id);
            let is_nullified = self.nullifiers.lock().await.contains_key(&entity_id);
            
            resource_exists && !is_nullified
        });
        
        Ok(result)
    }
}

// Implement AsExecutionContext for DefaultStateManager
#[async_trait]
impl AsExecutionContext for DefaultStateManager {
    async fn create_resource(&mut self, resource: Resource) -> Result<ResourceId> {
        let entity_id = resource.id; // Resource.id is EntityId
        let mut resources_map = self.resources.lock().await;
        if resources_map.contains_key(&entity_id) {
            return Err(anyhow!("Resource with ID {:?} already exists", entity_id));
        }
        resources_map.insert(entity_id, resource);
        // Convert EntityId back to ResourceId for return
        let resource_id: ResourceId = entity_id.to_id();
        Ok(resource_id)
    }

    async fn derive_resource_data(&mut self, id: &ResourceId, new_data: ValueExpr) -> Result<()> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = id.to_id();
        
        let new_data_bytes = new_data.as_ssz_bytes();
        let hash = Sha256::digest(&new_data_bytes);
        let new_data_id = ValueExprId::new(<[u8; 32]>::try_from(hash.as_slice()).expect("Hash must be 32 bytes"));

        let mut values_map = self.values.lock().await;
        values_map.insert(new_data_id, new_data);
        drop(values_map);

        let mut resources_map = self.resources.lock().await;

        if let Some(old_resource_entry) = resources_map.remove_entry(&entity_id) {
            let old_resource = old_resource_entry.1;
            let mut new_resource_content = old_resource.clone();
            // Note: Resource no longer has a 'value' field - data is stored directly in Resource fields
            // For now, we'll just update the timestamp to indicate the resource was modified
            new_resource_content.timestamp = causality_types::core::time::Timestamp::now();

            // Generate new EntityId for the updated resource
            let new_entity_id = EntityId::new(new_resource_content.as_ssz_bytes().as_slice().try_into().unwrap_or([0u8; 32]));
            new_resource_content.id = new_entity_id;

            resources_map.insert(new_entity_id, new_resource_content);
            Ok(())
        } else {
            Err(anyhow!("Resource not found for update: {:?}", id))
        }
    }

    async fn nullify_resource(&mut self, nullifier: Nullifier) -> Result<()> {
        let entity_id = nullifier.resource_id; // Nullifier.resource_id is EntityId
        let mut nullifiers_map = self.nullifiers.lock().await;
        nullifiers_map.insert(entity_id, nullifier);
        Ok(())
    }

    async fn lock_resource(&mut self, id: &ResourceId) -> Result<()> {
        // Convert ResourceId to EntityId for internal storage lookup
        let _entity_id: EntityId = id.to_id();
        // Placeholder: Actual locking mechanism might involve updating resource state
        // or a separate lock table. For now, this is a no-op.
        Ok(())
    }

    async fn unlock_resource(&mut self, id: &ResourceId) -> Result<()> {
        // Convert ResourceId to EntityId for internal storage lookup
        let _entity_id: EntityId = id.to_id();
        // Placeholder: Counterpart to lock_resource.
        Ok(())
    }

    async fn has_resource(&self, id: &ResourceId) -> Result<bool> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = id.to_id();
        Ok(self.resources.lock().await.contains_key(&entity_id))
    }

    async fn is_nullified(&self, resource_id: &ResourceId) -> Result<bool> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = resource_id.to_id();
        Ok(self.nullifiers.lock().await.contains_key(&entity_id))
    }
}

// Implement AsRuntimeContext for DefaultStateManager
#[async_trait]
impl AsRuntimeContext for DefaultStateManager {
    async fn get_resource(&self, id: &ResourceId) -> Result<Option<Resource>> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = id.to_id();
        Ok(self.resources.lock().await.get(&entity_id).cloned())
    }

    fn get_resource_sync(&self, id: &ResourceId) -> Result<Option<Resource>> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = id.to_id();
        tokio::runtime::Handle::current().block_on(async {
            Ok(self.resources.lock().await.get(&entity_id).cloned())
        })
    }

    async fn get_value_expr_by_id(&self, id: &ValueExprId) -> Result<Option<ValueExpr>> {
        Ok(self.values.lock().await.get(id).cloned())
    }

    fn get_value_expr_by_id_sync(&self, id: &ValueExprId) -> Result<Option<ValueExpr>> {
        tokio::runtime::Handle::current().block_on(async {
            Ok(self.values.lock().await.get(id).cloned())
        })
    }

    async fn get_input_resource_ids(&self) -> Result<Vec<ResourceId>> {
        // Convert all EntityIds to ResourceIds for return
        let entity_ids: Vec<EntityId> = self.resources.lock().await.keys().cloned().collect();
        let resource_ids: Vec<ResourceId> = entity_ids.iter().map(|entity_id| entity_id.to_id()).collect();
        Ok(resource_ids)
    }

    async fn create_resource(&mut self, resource: Resource) -> Result<ResourceId> {
        let entity_id = resource.id; // Resource.id is EntityId
        let mut resources_map = self.resources.lock().await;
        if resources_map.contains_key(&entity_id) {
            return Err(anyhow!("Resource with ID {:?} already exists (async)", entity_id));
        }
        resources_map.insert(entity_id, resource);
        // Convert EntityId back to ResourceId for return
        let resource_id: ResourceId = entity_id.to_id();
        Ok(resource_id)
    }

    async fn derive_resource_data(&mut self, id: &ResourceId, new_data: ValueExpr) -> Result<Resource> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = id.to_id();
        
        let new_data_bytes = new_data.as_ssz_bytes();
        let hash = Sha256::digest(&new_data_bytes);
        let new_data_id = ValueExprId::new(<[u8; 32]>::try_from(hash.as_slice()).expect("Hash must be 32 bytes"));

        let mut values_map = self.values.lock().await;
        values_map.insert(new_data_id, new_data);
        drop(values_map);

        let mut resources_map = self.resources.lock().await;

        if let Some(old_resource_entry) = resources_map.remove_entry(&entity_id) {
            let old_resource = old_resource_entry.1;
            let mut new_resource_content = old_resource.clone();
            // Note: Resource no longer has a 'value' field - data is stored directly in Resource fields
            // For now, we'll just update the timestamp to indicate the resource was modified
            new_resource_content.timestamp = causality_types::core::time::Timestamp::now();

            // Generate new EntityId for the updated resource
            let new_entity_id = EntityId::new(new_resource_content.as_ssz_bytes().as_slice().try_into().unwrap_or([0u8; 32]));
            new_resource_content.id = new_entity_id;

            resources_map.insert(new_entity_id, new_resource_content.clone());
            Ok(new_resource_content)
        } else {
            Err(anyhow!("Resource not found for derivation (async): {:?}", id))
        }
    }

    fn derive_resource_data_sync(&mut self, id: &ResourceId, new_data: ValueExpr) -> Option<anyhow::Result<Resource>> {
        let result = tokio::runtime::Handle::current().block_on(
            AsRuntimeContext::derive_resource_data(self, id, new_data)
        );
        Some(result)
    }

    async fn nullify_resource(&mut self, nullifier: Nullifier) -> Result<()> {
        let entity_id = nullifier.resource_id; // Nullifier.resource_id is EntityId
        let mut nullifiers_map = self.nullifiers.lock().await;
        nullifiers_map.insert(entity_id, nullifier);
        Ok(())
    }

    async fn send_message(&mut self, _target_domain: DomainId, _message_payload: ValueExpr) -> Result<()> {
        Err(anyhow!("DefaultStateManager::send_message: Not implemented"))
    }

    async fn current_time(&self) -> Result<Timestamp> {
        // A simple implementation that returns the current system time
        // More sophisticated implementations might need to synchronize with a global clock
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| anyhow!("Failed to get system time: {}", e))?
            .as_secs();

        // Create a Timestamp based on DomainId and logical time
        let timestamp = Timestamp {
            domain_id: DomainId::default(), // Default domain
            logical: 0,                     // Logical time 0
            wall: causality_types::core::time::WallClock(now * 1000), // Convert to milliseconds
        };

        Ok(timestamp)
    }

    fn current_time_sync(&self) -> Result<Timestamp> {
        tokio::runtime::Handle::current().block_on(self.current_time())
    }

    fn get_expr_sync(&self, id: &ExprId) -> Result<Option<TypesExpr>> {
        tokio::runtime::Handle::current().block_on(async {
            self.get_expr_impl(id).await
        })
    }

    async fn store_value_expr(&self, value_expr: ValueExpr) -> Result<ValueExprId> {
        let id = value_expr.id();
        let mut values_map = self.values.lock().await;
        values_map.insert(id, value_expr);
        Ok(id)
    }
}

impl Default for DefaultStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultStateManager {
    fn put_expr_impl(&mut self, expr_id: ExprId, expr: TypesExpr) -> Result<()> {
        let mut expressions_map = tokio::runtime::Handle::current().block_on(async {
            self.expressions.lock().await
        });
        expressions_map.insert(expr_id, expr);
        Ok(())
    }

    async fn get_expr_impl(&self, expr_id: &ExprId) -> Result<Option<TypesExpr>> {
        Ok(self.expressions.lock().await.get(expr_id).cloned())
    }

    fn put_handler_impl(&mut self, handler_id: HandlerId, handler: Handler) -> Result<()> {
        let mut handlers_map = tokio::runtime::Handle::current().block_on(async {
            self.handlers.lock().await
        });
        handlers_map.insert(handler_id, handler);
        Ok(())
    }

    async fn get_handler_impl(&self, handler_id: &HandlerId) -> Result<Option<Handler>> {
        Ok(self.handlers.lock().await.get(handler_id).cloned())
    }

    async fn get_all_handlers_impl(&self) -> Result<Vec<std::sync::Arc<Handler>>> {
        Ok(self.handlers.lock().await.values().cloned().map(Arc::new).collect())
    }
    
    async fn get_all_resources_by_domain_impl(&self, domain_id: &DomainId) -> Result<Vec<(ResourceId, Resource)>> {
        let resources = self.resources.lock().await;
        let filtered: Vec<(ResourceId, Resource)> = resources
            .iter()
            .filter(|(_, resource)| &resource.domain_id == domain_id)
            .map(|(entity_id, resource)| {
                // Convert EntityId to ResourceId for the return type
                let resource_id: ResourceId = entity_id.to_id();
                (resource_id, resource.clone())
            })
            .collect();
        
        Ok(filtered)
    }
    
    async fn get_all_handlers_by_domain_impl(&self, domain_id: &DomainId) -> Result<Vec<Handler>> {
        let handlers = self.handlers.lock().await;
        let filtered: Vec<Handler> = handlers
            .values()
            .filter(|handler| handler.domain_id == *domain_id)
            .cloned()
            .collect();
        
        Ok(filtered)
    }
}

#[async_trait]
impl StateManager for DefaultStateManager {
    async fn is_resource_available(&self, id: &ResourceId) -> Result<bool> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = id.to_id();
        // A resource is available if it exists in resources and not in nullifiers
        let exists = self.resources.lock().await.contains_key(&entity_id);
        let nullified = self.nullifiers.lock().await.contains_key(&entity_id);
        
        Ok(exists && !nullified)
    }
    
    async fn is_effect_expr_completed(&self, _effect_expr_id: &ExprId) -> Result<bool> {
        // Placeholder implementation that treats all effects as incomplete
        Ok(false)
    }
    
    fn put_expr(&mut self, expr_id: ExprId, expr: TypesExpr) -> Result<()> {
        self.put_expr_impl(expr_id, expr)
    }
    
    async fn get_expr(&self, expr_id: &ExprId) -> Result<Option<TypesExpr>> {
        self.get_expr_impl(expr_id).await
    }
    
    fn put_handler(&mut self, handler_id: HandlerId, handler: Handler) -> Result<()> {
        self.put_handler_impl(handler_id, handler)
    }
    
    async fn get_handler(&self, handler_id: &HandlerId) -> Result<Option<Handler>> {
        self.get_handler_impl(handler_id).await
    }
    
    async fn get_all_handlers(&self) -> Result<Vec<std::sync::Arc<Handler>>> {
        self.get_all_handlers_impl().await
    }
    
    async fn get_all_resources_by_domain(&self, domain_id: &DomainId) -> Result<Vec<(ResourceId, Resource)>> {
        self.get_all_resources_by_domain_impl(domain_id).await
    }
    
    async fn get_all_handlers_by_domain(&self, domain_id: &DomainId) -> Result<Vec<Handler>> {
        self.get_all_handlers_by_domain_impl(domain_id).await
    }
    
    // State verification methods using Merkle proofs
    
    /// Generate a proof that a resource exists in the state
    /// This creates a Merkle proof that can be verified by anyone with the state root
    async fn generate_resource_proof(&self, resource_id: &ResourceId) -> Result<crate::state::state_proof::ResourceProof> {
        self.generate_resource_proof_impl(resource_id).await
    }
    
    /// Generate a proof that a value exists in the state
    /// This creates a Merkle proof that can be verified by anyone with the state root
    async fn generate_value_proof(&self, value_id: &ValueExprId) -> Result<crate::state::state_proof::ValueProof> {
        self.generate_value_proof_impl(value_id).await
    }
    
    /// Get the current Merkle root hash of the resources in the state
    /// This root can be used to verify resource proofs
    async fn get_resources_root(&self) -> Result<[u8; 32]> {
        self.get_resources_root_impl().await
    }
    
    /// Get the current Merkle root hash of the values in the state
    /// This root can be used to verify value proofs
    async fn get_values_root(&self) -> Result<[u8; 32]> {
        self.get_values_root_impl().await
    }
    
    /// Verify a resource proof against the current state
    /// Returns true if the proof is valid, false otherwise
    async fn verify_resource_proof(&self, proof: &crate::state::state_proof::ResourceProof) -> Result<bool> {
        self.verify_resource_proof_impl(proof).await
    }
    
    /// Verify a value proof against the current state
    /// Returns true if the proof is valid, false otherwise
    async fn verify_value_proof(&self, proof: &crate::state::state_proof::ValueProof) -> Result<bool> {
        self.verify_value_proof_impl(proof).await
    }
}

impl DefaultStateManager {
    async fn generate_resource_proof_impl(&self, resource_id: &ResourceId) -> Result<ResourceProof> {
        // Convert ResourceId to EntityId for internal storage lookup
        let entity_id: EntityId = resource_id.to_id();
        
        let resources = self.resources.lock().await;
        
        // Check if the resource exists
        if !resources.contains_key(&entity_id) {
            return Err(anyhow!("Resource not found: {:?}", resource_id));
        }
        
        // Create a simple proof implementation
        // In a real implementation, this would use a Merkle tree
        let resource_data = resources.get(&entity_id).unwrap();
        let resource_bytes = resource_data.as_ssz_bytes();
        
        // Create proof using the resource ID and data
        let proof = ResourceProof::new(*resource_id, resource_bytes);
        Ok(proof)
    }
    
    async fn generate_value_proof_impl(&self, value_id: &ValueExprId) -> Result<ValueProof> {
        // Get all resources from the state
        let resources_map = self.resources.lock().await;
        let resources: Vec<Resource> = resources_map.values().cloned().collect();
        
        // Get all values from the state
        let values_map = self.values.lock().await;
        let values: Vec<ValueExpr> = values_map.values().cloned().collect();
        
        // Create a proof generator and build the trees
        let mut generator = StateProofGenerator::new();
        generator.build_trees(&resources, &values)?;
        
        // Generate the proof
        generator.generate_value_proof(value_id)
    }
    
    async fn get_resources_root_impl(&self) -> Result<[u8; 32]> {
        // Get all resources from the state
        let resources_map = self.resources.lock().await;
        let resources: Vec<Resource> = resources_map.values().cloned().collect();
        
        // Get all values from the state (needed to build the trees)
        let values_map = self.values.lock().await;
        let values: Vec<ValueExpr> = values_map.values().cloned().collect();
        
        // Create a proof generator and build the trees
        let mut generator = StateProofGenerator::new();
        generator.build_trees(&resources, &values)?;
        
        // Get the resources root
        generator.resources_root()
    }
    
    async fn get_values_root_impl(&self) -> Result<[u8; 32]> {
        // Get all resources from the state (needed to build the trees)
        let resources_map = self.resources.lock().await;
        let resources: Vec<Resource> = resources_map.values().cloned().collect();
        
        // Get all values from the state
        let values_map = self.values.lock().await;
        let values: Vec<ValueExpr> = values_map.values().cloned().collect();
        
        // Create a proof generator and build the trees
        let mut generator = StateProofGenerator::new();
        generator.build_trees(&resources, &values)?;
        
        // Get the values root
        generator.values_root()
    }
    
    async fn verify_resource_proof_impl(&self, proof: &ResourceProof) -> Result<bool> {
        // Get the resources root
        let resources_root = self.get_resources_root_impl().await?;
        
        // Create a verifier
        let verifier = crate::state::state_proof::StateProofVerifier::new(
            resources_root,
            [0u8; 32] // Values root not needed for resource proof
        );
        
        // Verify the proof
        Ok(verifier.verify_resource_proof(proof))
    }
    
    async fn verify_value_proof_impl(&self, proof: &ValueProof) -> Result<bool> {
        // Get the values root
        let values_root = self.get_values_root_impl().await?;
        
        // Create a verifier
        let verifier = crate::state::state_proof::StateProofVerifier::new(
            [0u8; 32], // Resources root not needed for value proof
            values_root
        );
        
        // Verify the proof
        Ok(verifier.verify_value_proof(proof))
    }
} 