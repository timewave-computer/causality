//! Core effect system types and trait extensions
//!
//! This module provides core trait extensions and implementations for the toolkit,
//! including effect type definitions, resource state management utilities, and
//! composable effect expressions for building complex workflows.

use std::any::Any;
use std::fmt::Debug;
use std::marker::PhantomData;

use causality_types::{
    primitive::{
        ids::{
            DomainId, EffectId, IntentId, NodeId, NullifierId, ResourceId, EntityId,
        },
        string::Str,
    },
    effect::types::Effect,
};

//-----------------------------------------------------------------------------
// Toolkit Specific Trait Extensions & Implementations
//-----------------------------------------------------------------------------

/// A marker trait for toolkit effects. Defines basic properties of a toolkit-specific effect type.
pub trait ToolkitEffect: Send + Sync + Debug + 'static {
    /// Provides a string representation of the effect's type, e.g., "token.create".
    fn effect_type_str(&self) -> Str;

    /// Provides a unique EffectId for the logic/definition of this effect type.
    /// This can be content-addressed based on the struct definition or a static unique ID.
    fn effect_logic_id(&self) -> EffectId;

    /// Allows downcasting to a concrete type.
    fn as_any(&self) -> &dyn Any;
}

/// Provides data needed to construct a `tel::Effect` from a toolkit effect.
pub trait ToolkitTelEffectData {
    /// Resource IDs that this effect conceptually consumes or reads.
    fn input_resources(&self) -> Vec<ResourceId>;

    /// Resource IDs that this effect conceptually produces or writes.
    fn output_resources(&self) -> Vec<ResourceId>;

    /// Resource IDs that this effect nullifies.
    fn nullified_resources(&self) -> Vec<ResourceId>;
}

/// Trait to convert a toolkit-defined effect structure into a concrete `core::Effect`.
/// Automatically implemented for types that implement `ToolkitEffect` and `ToolkitTelEffectData`.
pub trait ToTelEffect: ToolkitEffect + ToolkitTelEffectData {
    fn to_tel_effect(
        &self,
        node_id: NodeId,
        domain_id: DomainId,
        _intent_id: IntentId,
    ) -> Effect {
        Effect {
            id: EntityId::new(node_id.0),
            name: self.effect_type_str(),
            domain_id,
            effect_type: self.effect_type_str(),
            inputs: self.input_resources().into_iter().map(|_id| {
                use causality_types::resource::flow::ResourceFlow;
                ResourceFlow::new(Str::from("toolkit_resource"), 1, domain_id)
            }).collect(),
            outputs: self.output_resources().into_iter().map(|_id| {
                use causality_types::resource::flow::ResourceFlow;
                ResourceFlow::new(Str::from("toolkit_resource"), 1, domain_id)
            }).collect(),
            expression: None,
            timestamp: causality_types::primitive::time::Timestamp::now(),
            hint: None,
        }
    }
}

// Blanket implementation
impl<T> ToTelEffect for T where T: ToolkitEffect + ToolkitTelEffectData {}

/// A marker trait for toolkit effects (OLD - to be replaced by ToolkitEffect)
pub trait DefaultAsResourceConvertible: Send + Sync {
    /// Generate a default effect ID based on the type name
    fn default_effect_id(&self) -> EffectId;

    /// Default implementation returns no input resources
    fn default_input_resources(&self) -> Vec<ResourceId>;

    /// Default implementation returns no output resources
    fn default_output_resources(&self) -> Vec<ResourceId>;
    
    /// Get effect ID
    fn effect_id(&self) -> EffectId;
}

//-----------------------------------------------------------------------------
// Resource State Management
//-----------------------------------------------------------------------------

/// Resource lifecycle states for type-state pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    /// Resource exists but hasn't been consumed
    Active,
    /// Resource has been consumed and can't be used again
    Consumed,
    /// Resource has been created but not yet committed
    Created,
}

/// Type-safe resource reference with lifecycle state
pub struct TypedResource<T, S: Copy = ResourceState> {
    /// Resource identifier
    pub id: ResourceId,
    /// Resource data type
    _type: PhantomData<T>,
    /// Resource state
    _state: PhantomData<S>,
}

impl<T, S: Copy> TypedResource<T, S> {
    /// Create a new typed resource reference
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            _type: PhantomData,
            _state: PhantomData,
        }
    }

    /// Get the resource ID
    pub fn id(&self) -> ResourceId {
        self.id
    }
}

/// Type-safe consumed resource that can't be used again
pub struct ConsumedResource<T>(ResourceId, PhantomData<T>);

impl<T> ConsumedResource<T> {
    /// Mark a resource as consumed
    pub fn consume(resource: TypedResource<T, ResourceState>) -> Self {
        Self(resource.id, PhantomData)
    }

    /// Get the resource ID
    pub fn id(&self) -> ResourceId {
        self.0
    }

    /// Get the nullifier for this consumed resource
    pub fn nullifier(&self) -> causality_types::resource::Nullifier {
        use causality_types::primitive::ids::EntityId;
        causality_types::resource::Nullifier::new(EntityId::new(self.0 .0))
    }

    /// Get the nullifier ID by creating a simple hash
    pub fn nullifier_id(&self) -> NullifierId {
        use sha2::{Digest, Sha256};
        let nullifier = self.nullifier();
        let mut hasher = Sha256::new();
        hasher.update(nullifier.resource_id .0);
        hasher.update(nullifier.nullifier_hash);
        let hash = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash);
        NullifierId::new(hash_array)
    }
}

//-----------------------------------------------------------------------------
// Simplified CloneableEffectBox
//-----------------------------------------------------------------------------

/// A cloneable wrapper for effect trait objects
#[derive(Clone)]
pub struct CloneableEffectBox {
    /// Type name of the wrapped effect for debugging
    type_name: String,
    /// Placeholder for actual effect handling
    _phantom: PhantomData<()>,
}

impl CloneableEffectBox {
    /// Create a new cloneable effect box
    pub fn new<E: Send + Sync + 'static>(_effect: E) -> Self {
        Self {
            type_name: std::any::type_name::<E>().to_string(),
            _phantom: PhantomData,
        }
    }
    
    /// Handle the effect (placeholder implementation)
    pub fn handle(&self, _handler: &dyn crate::control_flow::SimpleEffectHandler) -> crate::ToolkitResult<()> {
        // Placeholder implementation - in a real implementation this would
        // store the actual effect and call its handle method
        Ok(())
    }
}

impl Debug for CloneableEffectBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloneableEffectBox")
            .field("type", &self.type_name)
            .finish()
    }
}

/// Used to compose multiple effects
#[derive(Debug, Clone)]
pub enum EffectExpr {
    /// Pure effect (no-op)
    Pure,
    /// Single effect
    Single(CloneableEffectBox),
    /// Sequence of effects  
    Sequence(Vec<EffectExpr>),
}

impl EffectExpr {
    /// Create a pure (no-op) effect
    pub fn pure() -> Self {
        EffectExpr::Pure
    }

    /// Create a single effect
    pub fn single<E: Send + Sync + 'static>(effect: E) -> Self {
        EffectExpr::Single(CloneableEffectBox::new(effect))
    }

    /// Create a sequence of effects
    pub fn sequence(effects: Vec<EffectExpr>) -> Self {
        EffectExpr::Sequence(effects)
    }

    /// Add an effect to the end of this expression
    pub fn then<E: Send + Sync + 'static>(self, effect: E) -> Self {
        match self {
            EffectExpr::Pure => EffectExpr::single(effect),
            EffectExpr::Single(e) => EffectExpr::Sequence(vec![
                EffectExpr::Single(e),
                EffectExpr::single(effect),
            ]),
            EffectExpr::Sequence(mut effects) => {
                effects.push(EffectExpr::single(effect));
                EffectExpr::Sequence(effects)
            }
        }
    }
}

/// Testing-specific module
#[cfg(feature = "testing")]
pub mod testing {
    /// A handler that records handled effects for testing
    pub struct RecordingHandler;

    impl RecordingHandler {
        /// Create a new recording handler
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Default for RecordingHandler {
        fn default() -> Self {
            Self::new()
        }
    }
}

//-----------------------------------------------------------------------------
// EffectHandler Extensions
//-----------------------------------------------------------------------------

/// Trait for handlers that can handle a specific effect type
pub trait Handles<E: causality_types::effect::core::Effect>: Send + Sync {
    /// Handle a specific effect with this handler
    fn handle(&self, effect: &E) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

// Commenting out generic EffectHandler impl as it causes conflicts
// impl<T, E> EffectHandler for T
// where
//     T: Handles<E> + Send + Sync + 'static,
//     E: Effect + 'static,
// {
//     fn handle(&self, effect: &dyn Effect) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//         // Try to downcast to the specific effect type
//         if let Some(e) = effect.as_any().downcast_ref::<E>() {
//             self.handle(e)
//         } else {
//             Err(Box::new(std::io::Error::new(
//                 std::io::ErrorKind::InvalidData,
//                 format!(
//                     "Effect type mismatch: handler for {} received {}",
//                     std::any::type_name::<E>(),
//                     effect.typ()
//                 ),
//             )))
//         }
//     }
// }
