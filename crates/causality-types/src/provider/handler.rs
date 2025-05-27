// Purpose: Defines the ErasedEffectHandler trait for type-erased effect handling.

use crate::effects_core::{
    Effect, EffectHandler, EffectInput, EffectOutput, HandlerError,
};
use crate::expr::value::ValueExpr;
use anyhow::Result; // For the Result in handle_erased
use async_trait::async_trait;
// use std::sync::Arc; // Removed unused import

/// A type-erased version of an effect handler.
/// This allows storing different effect handlers in a common collection (e.g., a HashMap)
/// without needing to know their specific effect types at compile time in the collection itself.
/// The actual handling logic will still need to downcast/convert the `ValueExpr` payloads
/// to the concrete `EffectInput` and `EffectOutput` types of the specific handler.
#[async_trait]
pub trait ErasedEffectHandler: Send + Sync + 'static {
    /// Returns the unique type string of the effect this handler processes.
    fn effect_type_str(&self) -> &'static str;

    /// Handles an effect, taking an input `ValueExpr` and returning an output `ValueExpr`.
    /// The concrete `EffectInput` and `EffectOutput` conversions happen internally.
    async fn handle_erased(
        &self,
        input_payload: ValueExpr,
        // TODO: Consider passing a context here if handlers need access to runtime services.
        // context: Arc<dyn AsRuntimeContext>, // Example context
    ) -> Result<ValueExpr, HandlerError>;
}

/// Generic implementation of `ErasedEffectHandler` for any concrete `EffectHandler`.
#[async_trait]
impl<H> ErasedEffectHandler for H
where
    H: EffectHandler + Send + Sync + 'static,
    // <H::E as Effect>::Input: EffectInput + TryFrom<ValueExpr, Error = ConversionError>,
    // No need for the TryFrom bound here, EffectInput::from_value_expr is used.
    <H::E as Effect>::Output: EffectOutput + Into<ValueExpr>,
    // HandlerError: From<<H::E as Effect>::InputConversionError>, // Not needed if mapping directly
{
    fn effect_type_str(&self) -> &'static str {
        <H::E as Effect>::EFFECT_TYPE
    }

    async fn handle_erased(
        &self,
        input_payload: ValueExpr,
    ) -> Result<ValueExpr, HandlerError> {
        // 1. Try to convert the generic ValueExpr input into the handler's specific EffectInput type.
        let concrete_input = <H::E as Effect>::Input::from_value_expr(input_payload)
            .map_err(HandlerError::InputConversionFailed)?;

        // 2. Call the concrete handler's handle method.
        let concrete_output = self.handle(concrete_input).await?;

        // 3. Convert the handler's specific EffectOutput type back into a generic ValueExpr.
        Ok(concrete_output.into())
    }
}

// Need ConversionError in scope for the impl block above.
// This is typically part of effects_core or defined alongside EffectInput/Output.
// use crate::effects_core::ConversionError; // Removed redundant import
