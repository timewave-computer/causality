//! Effect handler interfaces
//!
//! This module defines interfaces for handling effects.

use std::any::Any;
use std::fmt::Debug;

use async_trait::async_trait;

use crate::effect::runtime::context::Context;
use crate::effect::runtime::error::{EffectError, EffectResult};
use crate::effect::runtime::types::id::EffectTypeId;

/// Interface for handling effects
#[async_trait]
pub trait EffectHandler: Debug + Send + Sync {
    /// Check if this handler can handle the given effect type
    async fn can_handle(&self, effect_type: &EffectTypeId) -> bool;
    
    /// Handle an effect
    ///
    /// This is a low-level function that takes boxed types.
    /// For type-safe handling, implement a specific handler for your effect type.
    async fn handle(
        &self,
        effect_type: &EffectTypeId,
        param: Box<dyn Any + Send>,
        context: &dyn Context,
    ) -> Result<Box<dyn Any + Send>, EffectError>;
} 