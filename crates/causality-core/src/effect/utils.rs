// Effect utilities
//
// Utilities for working with effects in the Causality system.

use crate::effect::types::EffectTypeId;
use crate::effect::outcome::EffectOutcome;
use crate::effect::EffectError;
use crate::id_utils;

/// Generate a new random effect type ID
pub fn generate_random_effect_type_id() -> EffectTypeId {
    let random_bytes = id_utils::generate_random_bytes(32);
    EffectTypeId::new(hex::encode(&random_bytes))
}

/// Check if an effect outcome is successful
pub fn is_successful_outcome(outcome: &EffectOutcome) -> bool {
    outcome.is_success()
}

/// Convert an error to an effect outcome
pub fn error_to_outcome<T>(error: T) -> EffectOutcome 
where
    T: Into<Box<EffectError>>
{
    EffectOutcome::error(error.into())
}

/// Format effect type ID for display
pub fn format_effect_type_id(effect_type_id: &EffectTypeId) -> String {
    format!("Effect[{}]", effect_type_id)
}

/// Convert an error to a map representation
pub fn error_to_map(error: &EffectError) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("error".to_string(), error.to_string());
    map.insert("type".to_string(), "effect_error".to_string());
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_generate_random_effect_type_id() {
        let id1 = generate_random_effect_type_id();
        let id2 = generate_random_effect_type_id();
        
        // IDs should be unique
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_is_successful_outcome() {
        let success = EffectOutcome::success(HashMap::new());
        let error = EffectOutcome::error(Box::new(EffectError::Other("Test error".to_string())));
        
        assert!(is_successful_outcome(&success));
        assert!(!is_successful_outcome(&error));
    }
} 