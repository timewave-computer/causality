//-----------------------------------------------------------------------------
// Error Handling Utilities for the Causality Framework
//-----------------------------------------------------------------------------

use causality_types::core::error::{
    EffectHandlingError, ErrorCategory, ResourceError,
};
use causality_types::primitive::string::Str;
use causality_types::effects_core::{ConversionError, HandlerError};
use causality_types::expr::result::ExprError;

//-----------------------------------------------------------------------------
// Error Utility Functions
//-----------------------------------------------------------------------------

/// Creates a resource error with the specified error message and category
pub fn create_resource_error(
    message: &str,
    category: ErrorCategory,
) -> ResourceError {
    match category {
        ErrorCategory::ResourceNotFound => ResourceError::NotFound(message.to_string()),
        ErrorCategory::Authorization => ResourceError::PermissionDenied(message.to_string()),
        ErrorCategory::Validation => ResourceError::ValidationFailed(message.to_string()),
        ErrorCategory::StateTransition => ResourceError::InvalidState(message.to_string()),
        _ => ResourceError::Unknown(message.to_string()),
    }
}

/// Creates an effect handling error with the specified message
pub fn create_effect_handling_error(message: &str) -> EffectHandlingError {
    EffectHandlingError::new(message)
}

/// Converts a HandlerError into an EffectHandlingError
pub fn handler_error_to_effect_handling_error(
    error: HandlerError,
) -> EffectHandlingError {
    // Using the error directly in the pattern match below
    // No need to pre-convert to string
    match error {
        HandlerError::EffectExecutionFailed(details) => {
            EffectHandlingError::new(format!("Effect execution failed: {}", details))
        }
        HandlerError::InputConversionFailed(err) => {
            EffectHandlingError::new(format!("Input conversion failed: {}", err))
        }
        HandlerError::OutputConversionFailed(err) => {
            EffectHandlingError::new(format!("Output conversion failed: {}", err))
        }
        HandlerError::InternalError(details) => {
            EffectHandlingError::new(format!("Handler internal error: {}", details))
        }
        HandlerError::LispError(details) => {
            EffectHandlingError::new(format!("Lisp evaluation error: {}", details))
        }
    }
}

/// Converts a ConversionError into a ResourceError
pub fn conversion_error_to_resource_error(error: ConversionError) -> ResourceError {
    let message = error.to_string();
    let category = ErrorCategory::Validation;

    create_resource_error(&message, category)
}

/// Create an ExprError from a generic error type
pub fn to_expr_error<E: std::error::Error>(error: E) -> ExprError {
    let _message = error.to_string(); // Prefixed as message was unused directly
    ExprError::ExecutionError { message: Str::from(format!("External error: {}", error)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_resource_error() {
        let error = create_resource_error("Test error", ErrorCategory::General);
        match error {
            ResourceError::Unknown(message) => {
                assert_eq!(message, "Test error");
            },
            _ => panic!("Expected ResourceError::Unknown variant")
        }
    }

    #[test]
    fn test_create_effect_handling_error() {
        let error = create_effect_handling_error("Test error");
        // EffectHandlingError now wraps an anyhow error, so we can only check that it was created
        assert!(error.to_string().contains("Test error"));
    }

    #[test]
    fn test_handler_error_to_effect_handling_error() {
        let handler_error =
            HandlerError::EffectExecutionFailed("Test error".to_string());
        let effect_error = handler_error_to_effect_handling_error(handler_error);
        assert!(effect_error.to_string().contains("Test error"));
    }

    #[test]
    fn test_create_resource_error_with_validation_category() {
        let resource_error = create_resource_error(
            "Test error for validation",
            ErrorCategory::Validation,
        );
        match resource_error {
            ResourceError::ValidationFailed(message) => {
                assert!(message.contains("Test error"));
            },
            _ => panic!("Expected ResourceError::ValidationFailed variant")
        }
    }

    #[test]
    fn test_conversion_error_to_resource_error() {
        let conversion_error = ConversionError::Custom("Test error".to_string());
        let resource_error = conversion_error_to_resource_error(conversion_error);
        match resource_error {
            ResourceError::ValidationFailed(message) => {
                assert!(message.contains("Test error"));
            },
            _ => panic!("Expected ResourceError::ValidationFailed variant")
        }
    }
}
