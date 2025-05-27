// Effect Execution Context
//
// Defines the context in which effects are executed, providing access
// to capabilities, resources, and other relevant information.

use std::collections::{HashSet, HashMap};
use std::sync::Arc;
use std::any::Any;
use std::fmt::Debug;
use async_trait::async_trait;

// Corrected imports
use super::types::{EffectId, Right}; // Removed ExecutionBoundary as it was unused
use crate::resource::types::ResourceId;

/// Capability representation within the effect context.
/// Simplifies resource access checks.
#[derive(Debug, Clone, PartialEq, Eq, Hash)] // Added derives
pub struct Capability {
    pub resource_id: ResourceId,
    pub right: Right,
}

// Implement Display for easier use in error messages, etc.
impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}: {})", self.right, self.resource_id)
    }
}

/// Trait representing the execution context for an effect.
///
/// This context provides the effect with necessary information and capabilities
/// to interact with the system, including resource access, metadata, and
/// the ability to derive sub-contexts.
#[async_trait]
pub trait EffectContext: Send + Sync + Debug + Any {
    /// Get the unique ID of the effect being executed.
    fn effect_id(&self) -> &EffectId;

    /// Get the capabilities available in this context.
    /// Capabilities define what operations the effect is permitted to perform.
    fn capabilities(&self) -> &[Capability];

    /// Get the set of resource IDs relevant to this context.
    /// This helps scope resource access and discovery.
    fn resources(&self) -> &HashSet<ResourceId>;

    /// Get the parent context, if this context was derived.
    fn parent_context(&self) -> Option<&Arc<dyn EffectContext>>;

    /// Check if a specific capability is present in the context.
    fn has_capability(&self, capability: &Capability) -> bool;

    /// Access metadata associated with this execution context.
    fn metadata(&self) -> &HashMap<String, String>;

    // --- Context Manipulation Methods ---

    /// Derive a new context for a sub-effect or related operation.
    /// The derived context typically inherits capabilities and resources
    /// but has a new, distinct effect ID.
    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext>;

    /// Create a new context based on the current one, but with additional capabilities.
    /// This is useful for temporarily elevating privileges or granting specific access.
    fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext>;

    /// Create a new context based on the current one, including additional resource IDs.
    /// Expands the scope of resources the context is aware of.
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext>;

    /// Create a new context with additional metadata merged into the existing metadata.
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext>;

    /// Clone the current context into a new Box.
    fn clone_context(&self) -> Box<dyn EffectContext>;

    /// Provides `Any` type support for downcasting.
    fn as_any(&self) -> &dyn Any;
}

// --- Default Implementation (Example) ---
// A simple in-memory implementation for testing or basic scenarios.

#[derive(Debug, Clone)] // Added Clone
pub struct DefaultEffectContext {
    effect_id: EffectId,
    capabilities: Vec<Capability>,
    resources: HashSet<ResourceId>,
    metadata: HashMap<String, String>,
    parent: Option<Arc<dyn EffectContext>>,
}

impl DefaultEffectContext {
    pub fn new(
        effect_id: EffectId,
        capabilities: Vec<Capability>,
        resources: HashSet<ResourceId>,
        metadata: HashMap<String, String>,
        parent: Option<Arc<dyn EffectContext>>,
    ) -> Self {
        Self {
            effect_id,
            capabilities,
            resources,
            metadata,
            parent,
        }
    }

    // Helper to create a root context
    pub fn root(effect_id: EffectId) -> Self {
        Self::new(effect_id, vec![], HashSet::new(), HashMap::new(), None)
    }
}

#[async_trait]
impl EffectContext for DefaultEffectContext {
    fn effect_id(&self) -> &EffectId {
        &self.effect_id
    }

    fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    fn resources(&self) -> &HashSet<ResourceId> {
        &self.resources
    }

    fn parent_context(&self) -> Option<&Arc<dyn EffectContext>> {
        self.parent.as_ref()
    }

    fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.iter().any(|ctx_cap| {
            ctx_cap.resource_id == capability.resource_id && ctx_cap.right >= capability.right
        })
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
        Box::new(Self::new(
            effect_id,
            self.capabilities.clone(),
            self.resources.clone(),
            self.metadata.clone(),
            Some(Arc::new(self.clone())), // Clone self into Arc for parent
        ))
    }

    fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext> {
        let mut new_caps = self.capabilities.clone();
        new_caps.extend(capabilities);
        // Consider merging/deduplicating capabilities
        Box::new(Self::new(
            self.effect_id.clone(),
            new_caps,
            self.resources.clone(),
            self.metadata.clone(),
            self.parent.clone(),
        ))
    }

    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
        let mut new_res = self.resources.clone();
        new_res.extend(resources);
        Box::new(Self::new(
            self.effect_id.clone(),
            self.capabilities.clone(),
            new_res,
            self.metadata.clone(),
            self.parent.clone(),
        ))
    }

    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
        let mut new_meta = self.metadata.clone();
        new_meta.extend(metadata);
        Box::new(Self::new(
            self.effect_id.clone(),
            self.capabilities.clone(),
            self.resources.clone(),
            new_meta,
            self.parent.clone(),
        ))
    }

    fn clone_context(&self) -> Box<dyn EffectContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 