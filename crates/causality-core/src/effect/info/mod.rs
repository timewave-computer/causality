//! Effect information representation that can be serialized and deserialized
//! This solves the trait object clone/serialize issues by using concrete types

use serde::{Serialize, Deserialize};
use anyhow::Result;
use causality_types::ContentHash;
use crate::utils::content_addressing;

/// EffectType enum represents all possible effect types that can be used
/// This replaces the trait object Vec<Box<dyn Effect>> with a concrete type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectType {
    /// Add a capability to a resource
    AddCapability {
        /// The resource ID to add capability to
        resource_id: String,
        /// The capability to add
        capability: String,
    },
    /// Remove a capability from a resource
    RemoveCapability {
        /// The resource ID to remove capability from
        resource_id: String,
        /// The capability to remove
        capability: String,
    },
    /// Execute a command
    ExecuteCommand {
        /// The command to execute
        command: String,
        /// Optional arguments
        args: Vec<String>,
    },
    /// Create a resource
    CreateResource {
        /// The type of resource to create
        resource_type: String,
        /// Data for the resource
        data: String,
    },
    /// Update a resource
    UpdateResource {
        /// The ID of the resource to update
        resource_id: String,
        /// The updated data
        data: String,
    },
    /// Delete a resource
    DeleteResource {
        /// The ID of the resource to delete
        resource_id: String,
    },
    /// Generic effect with a string representation
    Generic {
        /// The effect type
        effect_type: String,
        /// The effect data
        data: String,
    },
}

/// EffectInfo represents all the data needed to recreate an effect
/// This replaces Box<dyn Effect> with a concrete type that can be cloned and serialized
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectInfo {
    /// The type of effect
    pub effect_type: EffectType,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

impl EffectInfo {
    /// Create a new EffectInfo from an effect type
    pub fn new(effect_type: EffectType) -> Self {
        Self {
            effect_type,
            metadata: None,
        }
    }

    /// Add metadata to the effect
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Get the content hash of this effect
    pub fn content_hash(&self) -> Result<ContentHash, String> {
        content_addressing::hash_object(self)
    }
}

/// Convert from a Box<dyn Effect> to an EffectInfo
/// This would need to be expanded with all the possible effect types in your system
pub fn effect_to_info(_effect: &dyn crate::effect::Effect) -> EffectInfo {
    // This is a simplified implementation for demonstration
    // In a real system, you would inspect the effect and create the appropriate EffectType
    EffectInfo::new(EffectType::Generic {
        effect_type: "unknown".to_string(),
        data: "{}".to_string(),
    })
}

/// Create a collection of EffectInfo from a collection of Box<dyn Effect>
pub fn effects_to_info(effects: &[Box<dyn crate::effect::Effect>]) -> Vec<EffectInfo> {
    effects.iter().map(|e| effect_to_info(e.as_ref())).collect()
}
