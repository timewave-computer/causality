//! Types for the effect runtime system
//!
//! This module defines the core types used in the effect runtime system,
//! including the Effect trait and effect type IDs.

use std::any::Any;
use std::fmt::Debug;
use async_trait::async_trait;

pub mod id;
pub use id::EffectTypeId;

use super::context::Context;
use super::error::EffectError;

/// The core Effect trait for the runtime system
#[async_trait]
pub trait Effect: Debug + Send + Sync {
    /// Parameter type for the effect
    type Param: Send + Sync + 'static;
    
    /// Outcome type for the effect
    type Outcome: Send + Sync + 'static;
    
    /// Returns the type of this effect
    fn type_id(&self) -> EffectTypeId;
    
    /// Executes this effect with the given parameters and context
    async fn execute(
        &self,
        param: Self::Param,
        context: &dyn Context,
    ) -> Result<Self::Outcome, EffectError>;
    
    /// Allows downcasting to concrete effect types
    fn as_any(&self) -> &dyn Any;
} 