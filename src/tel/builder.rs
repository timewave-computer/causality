// Builder implementations for creating TEL effects
use crate::tel::types::{
    Effect, 
    Authorization, 
    AuthorizedEffect, 
    ConditionalEffect, 
    TimedEffect, 
    Condition, 
    DomainId, 
    AssetId, 
    Amount, 
    Address, 
    ResourceId, 
    Timestamp,
    ResourceContents,
    VerificationKey,
    Proof,
};

/// Builder methods for constructing effects
impl Effect {
    /// Create a new sequence of effects
    pub fn sequence(effects: Vec<Effect>) -> Self {
        Effect::Sequence(effects)
    }
    
    /// Add authorization to an effect
    pub fn with_auth(self, auth: Authorization) -> AuthorizedEffect {
        AuthorizedEffect {
            effect: self,
            authorization: auth,
        }
    }
    
    /// Add a condition to an effect
    pub fn with_condition(self, condition: Condition) -> ConditionalEffect {
        ConditionalEffect {
            effect: self,
            condition,
        }
    }
    
    /// Add a timeout to an effect
    pub fn with_timeout(self, timeout: Timestamp) -> TimedEffect {
        TimedEffect {
            effect: self,
            timeout,
        }
    }
    
    // Convenience constructors for deposit effects
    pub fn deposit(domain: &str, asset: &str, amount: Amount) -> Self {
        Effect::Deposit {
            domain: domain.to_string(),
            asset: asset.to_string(),
            amount,
        }
    }
    
    // Convenience constructors for withdraw effects
    pub fn withdraw(domain: &str, asset: &str, amount: Amount, address: Address) -> Self {
        Effect::Withdraw {
            domain: domain.to_string(),
            asset: asset.to_string(),
            amount,
            address,
        }
    }
    
    // Convenience constructor for transfer effects
    pub fn transfer(from: Address, to: Address, asset: &str, amount: Amount) -> Self {
        Effect::Transfer {
            from,
            to,
            asset: asset.to_string(),
            amount,
        }
    }
    
    // Convenience constructor for resource creation
    pub fn create_resource(contents: ResourceContents) -> Self {
        Effect::ResourceCreate { contents }
    }
    
    // Convenience constructor for resource update
    pub fn update_resource(resource_id: ResourceId, contents: ResourceContents) -> Self {
        Effect::ResourceUpdate { resource_id, contents }
    }
    
    // Convenience constructor for resource transfer
    pub fn transfer_resource(resource_id: ResourceId, target_domain: &str) -> Self {
        Effect::ResourceTransfer { 
            resource_id, 
            target_domain: target_domain.to_string(),
        }
    }
    
    // Convenience constructor for resource merge
    pub fn merge_resources(source_ids: Vec<ResourceId>, target_id: ResourceId) -> Self {
        Effect::ResourceMerge { source_ids, target_id }
    }
    
    // Convenience constructor for resource split
    pub fn split_resource(
        source_id: ResourceId, 
        target_ids: Vec<ResourceId>, 
        distribution: Vec<Amount>,
    ) -> Self {
        Effect::ResourceSplit {
            source_id,
            target_ids,
            distribution,
        }
    }
    
    // Convenience constructor for proof verification
    pub fn verify_proof(verification_key: VerificationKey, proof: Proof) -> Self {
        Effect::VerifyProof { verification_key, proof }
    }
}

impl AuthorizedEffect {
    /// Chain a conditional effect after an authorized effect
    pub fn with_condition(self, condition: Condition) -> ConditionalEffect {
        ConditionalEffect {
            effect: self.effect,
            condition,
        }
    }
    
    /// Chain a timed effect after an authorized effect
    pub fn with_timeout(self, timeout: Timestamp) -> TimedEffect {
        TimedEffect {
            effect: self.effect,
            timeout,
        }
    }
}

impl ConditionalEffect {
    /// Chain an authorized effect after a conditional effect
    pub fn with_auth(self, auth: Authorization) -> AuthorizedEffect {
        AuthorizedEffect {
            effect: self.effect,
            authorization: auth,
        }
    }
    
    /// Chain a timed effect after a conditional effect
    pub fn with_timeout(self, timeout: Timestamp) -> TimedEffect {
        TimedEffect {
            effect: self.effect,
            timeout,
        }
    }
}

impl TimedEffect {
    /// Chain an authorized effect after a timed effect
    pub fn with_auth(self, auth: Authorization) -> AuthorizedEffect {
        AuthorizedEffect {
            effect: self.effect,
            authorization: auth,
        }
    }
    
    /// Chain a conditional effect after a timed effect
    pub fn with_condition(self, condition: Condition) -> ConditionalEffect {
        ConditionalEffect {
            effect: self.effect,
            condition,
        }
    }
}

// TEL Builder Module
//
// This module provides a builder pattern for constructing
// TEL components with proper configuration and dependencies.

use std::sync::Arc;
use uuid::Uuid;

use crate::tel::{
    error::TelResult,
    resource::{
        ResourceManager,
        ZkVerifier,
        SnapshotManager,
        VersionManager,
        SnapshotStorage,
        FileSnapshotStorage,
        verify::VerifierConfig,
    },
    effect::ResourceEffectAdapter,
};

/// Builder for TEL components
pub struct TelBuilder {
    /// Unique ID for this TEL instance
    instance_id: Uuid,
    /// Config for resource verification
    verifier_config: Option<VerifierConfig>,
    /// Storage for snapshots
    snapshot_storage: Option<Box<dyn SnapshotStorage>>,
}

impl TelBuilder {
    /// Create a new TEL builder
    pub fn new() -> Self {
        Self {
            instance_id: Uuid::new_v4(),
            verifier_config: None,
            snapshot_storage: None,
        }
    }
    
    /// Set a custom instance ID
    pub fn with_instance_id(mut self, id: Uuid) -> Self {
        self.instance_id = id;
        self
    }
    
    /// Configure the verifier
    pub fn with_verifier_config(mut self, config: VerifierConfig) -> Self {
        self.verifier_config = Some(config);
        self
    }
    
    /// Set a custom snapshot storage
    pub fn with_snapshot_storage<S: SnapshotStorage + 'static>(mut self, storage: S) -> Self {
        self.snapshot_storage = Some(Box::new(storage));
        self
    }
    
    /// Build a complete TEL system
    pub fn build(self) -> TelResult<TelSystem> {
        // Create verifier
        let verifier = Arc::new(ZkVerifier::new(
            self.verifier_config.unwrap_or_default()
        ));
        
        // Create resource manager
        let resource_manager = Arc::new(ResourceManager::new());
        
        // Create version manager
        let version_manager = Arc::new(VersionManager::default());
        
        // Create snapshot storage
        let snapshot_storage: Box<dyn SnapshotStorage> = match self.snapshot_storage {
            Some(storage) => storage,
            None => Box::new(FileSnapshotStorage::new(format!(
                "tel_snapshots_{}", self.instance_id
            ))?),
        };
        
        // Create snapshot manager with default schedule config
        let snapshot_manager = Arc::new(SnapshotManager::new(
            resource_manager.clone(),
            snapshot_storage,
            Default::default()
        ));
        
        // Create effect adapter
        let effect_adapter = Arc::new(ResourceEffectAdapter::new(
            resource_manager.clone()
        ));
        
        // Return the complete system
        Ok(TelSystem {
            instance_id: self.instance_id,
            resource_manager,
            verifier,
            snapshot_manager,
            version_manager,
            effect_adapter,
        })
    }
}

impl Default for TelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A complete TEL system with all components
pub struct TelSystem {
    /// Unique ID for this TEL instance
    pub instance_id: Uuid,
    /// Resource manager
    pub resource_manager: Arc<ResourceManager>,
    /// ZK verifier for operations
    pub verifier: Arc<ZkVerifier>,
    /// Snapshot manager
    pub snapshot_manager: Arc<SnapshotManager>,
    /// Version manager
    pub version_manager: Arc<VersionManager>,
    /// Effect adapter
    pub effect_adapter: Arc<ResourceEffectAdapter>,
} 