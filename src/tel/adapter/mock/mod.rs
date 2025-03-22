//! Mock adapter for testing TEL effect compilation
//! 
//! This module provides a simple mock adapter for testing the TEL
//! effect compilation infrastructure without external dependencies.

use std::any::Any;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::tel::types::*;
use crate::tel::error::TelError;
use crate::tel::adapter::traits::*;
use crate::tel::adapter::common::*;

/// Mock domain ID for testing
pub const MOCK_DOMAIN_ID: DomainId = "mock";

/// Mock asset type for testing
pub const MOCK_ASSET: AssetId = "MOCK";

/// Mock adapter for testing effect compilation
#[derive(Debug, Clone)]
pub struct MockAdapter {
    /// Configuration for the mock adapter
    pub config: MockAdapterConfig,
    /// Mock compilation state for testing
    pub state: HashMap<String, String>,
}

/// Configuration for the mock adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockAdapterConfig {
    /// Whether compilation should succeed
    pub succeed: bool,
    /// Whether validation should succeed
    pub validate: bool,
    /// Mock gas cost for each operation
    pub gas_cost: u64,
}

impl Default for MockAdapterConfig {
    fn default() -> Self {
        Self {
            succeed: true,
            validate: true,
            gas_cost: 100,
        }
    }
}

impl Default for MockAdapter {
    fn default() -> Self {
        Self {
            config: MockAdapterConfig::default(),
            state: HashMap::new(),
        }
    }
}

impl EffectCompiler for MockAdapter {
    // Define the output type as Vec<u8> (binary data)
    type Output = Vec<u8>;

    fn compile(&self, effect: &Effect, context: &CompilerContext) -> Result<CompilationResult, TelError> {
        // Simple mock implementation that just serializes the effect to JSON
        if !self.config.succeed {
            return Err(TelError::CompilationError(format!("Mock compilation failure for {:?}", effect)));
        }

        // Simulate compilation by serializing to JSON
        let serialized = serde_json::to_string(effect)
            .map_err(|e| TelError::SerializationError(e.to_string()))?;
        
        Ok(CompilationResult {
            domain_id: MOCK_DOMAIN_ID.to_string(),
            output: serialized.into_bytes(),
            estimated_cost: self.config.gas_cost,
            size: serialized.len() as u64,
            metadata: HashMap::new(),
        })
    }

    fn validate(&self, effect: &Effect, context: &CompilerContext) -> ValidationResult {
        if !self.config.validate {
            return Err(ValidationError {
                error_type: ValidationErrorType::ValidationFailed,
                message: "Mock validation failure".to_string(),
                path: "effect".to_string(),
                context: None,
            });
        }
        
        // Simple validation for mock domain
        match effect {
            Effect::Deposit(deposit) => {
                if deposit.domain != MOCK_DOMAIN_ID {
                    return Err(ValidationError {
                        error_type: ValidationErrorType::InvalidDomain,
                        message: format!("Expected domain {}, got {}", MOCK_DOMAIN_ID, deposit.domain),
                        path: "deposit.domain".to_string(),
                        context: None,
                    });
                }
                
                if deposit.asset != MOCK_ASSET {
                    return Err(ValidationError {
                        error_type: ValidationErrorType::InvalidAsset,
                        message: format!("Expected asset {}, got {}", MOCK_ASSET, deposit.asset),
                        path: "deposit.asset".to_string(),
                        context: None,
                    });
                }
            },
            // Add validation for other effect types as needed
            _ => {}
        }
        
        Ok(())
    }

    fn estimate_cost(&self, effect: &Effect, context: &CompilerContext) -> Result<u64, TelError> {
        // Simple mock implementation that returns configured gas cost
        Ok(self.config.gas_cost)
    }

    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            name: "mock".to_string(),
            version: "0.1.0".to_string(),
            domain_id: MOCK_DOMAIN_ID.to_string(),
            description: "Mock adapter for testing".to_string(),
            supported_effects: vec![
                "Deposit".to_string(),
                "Withdraw".to_string(),
                "Transfer".to_string(),
            ],
            status: AdapterStatus::Available,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_adapter_compilation() {
        let adapter = MockAdapter::default();
        let effect = Effect::Deposit(DepositEffect {
            domain: MOCK_DOMAIN_ID.to_string(),
            asset: MOCK_ASSET.to_string(),
            amount: 100.into(),
            source_address: "mock:source".to_string(),
            target_address: "mock:target".to_string(),
        });
        
        let context = CompilerContext {
            domain_parameters: HashMap::new(),
            resource_ids: HashMap::new(),
            chain_context: None,
            options: CompilationOptions::default(),
        };
        
        let result = adapter.compile(&effect, &context);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_mock_adapter_validation() {
        let adapter = MockAdapter::default();
        let valid_effect = Effect::Deposit(DepositEffect {
            domain: MOCK_DOMAIN_ID.to_string(),
            asset: MOCK_ASSET.to_string(),
            amount: 100.into(),
            source_address: "mock:source".to_string(),
            target_address: "mock:target".to_string(),
        });
        
        let context = CompilerContext {
            domain_parameters: HashMap::new(),
            resource_ids: HashMap::new(),
            chain_context: None,
            options: CompilationOptions::default(),
        };
        
        let valid_result = adapter.validate(&valid_effect, &context);
        assert!(valid_result.is_ok());
        
        let invalid_effect = Effect::Deposit(DepositEffect {
            domain: "invalid".to_string(),
            asset: MOCK_ASSET.to_string(),
            amount: 100.into(),
            source_address: "mock:source".to_string(),
            target_address: "mock:target".to_string(),
        });
        
        let invalid_result = adapter.validate(&invalid_effect, &context);
        assert!(invalid_result.is_err());
    }
} 