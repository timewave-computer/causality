// TEL adapter tests
// Original file: src/tel/adapter/tests.rs

//! Tests for the TEL adapter framework
//!
//! This module provides tests for the adapter framework functionality.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use causality_tel::*;
    use crate::tel::adapter::*;
    use crate::tel::adapter::mock::*;
    use causality_tel::TelError;

    #[test]
    fn test_adapter_registry() {
        let mut registry = AdapterRegistry::new();
        let mock_adapter = MockAdapter::default();
        
        // Register the mock adapter
        registry.register(Box::new(mock_adapter.clone()));
        
        // Test adapter registration
        assert_eq!(registry.list_adapters().len(), 1);
        
        // Test getting adapter by domain
        let adapter = registry.get_adapter(MOCK_DOMAIN_ID).unwrap();
        assert_eq!(adapter.metadata().domain_id, MOCK_DOMAIN_ID);
        
        // Test adapter compilation
        let effect = Effect::Deposit(DepositEffect {
            domain: MOCK_DOMAIN_ID.to_string(),
            asset: MOCK_ASSET.to_string(),
            amount: 100.into(),
            source_address: "mock:source".to_string(),
            target_address: "mock:target".to_string(),
        });
        
        let result = registry.compile_effect(&effect, None);
        assert!(result.is_ok());
        
        // Test unregistering adapter
        registry.unregister(MOCK_DOMAIN_ID);
        assert_eq!(registry.list_adapters().len(), 0);
        
        // Test getting adapter after unregistration
        let error = registry.get_adapter(MOCK_DOMAIN_ID).unwrap_err();
        match error {
            TelError::AdapterNotFound(_) => {}, // Expected error
            _ => panic!("Expected AdapterNotFound error"),
        }
    }
    
    #[test]
    fn test_validation() {
        let validators = CommonValidators::new();
        
        // Register a domain configuration
        let mut domain_config = HashMap::new();
        domain_config.insert("asset".to_string(), MOCK_ASSET.to_string());
        validators.register_domain(MOCK_DOMAIN_ID, domain_config);
        
        // Test valid domain validation
        assert!(validators.validate_domain(MOCK_DOMAIN_ID).is_ok());
        
        // Test invalid domain validation
        let invalid_result = validators.validate_domain("invalid");
        assert!(invalid_result.is_err());
        
        // Test effect validation
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
        
        let adapter = MockAdapter::default();
        let valid_result = adapter.validate(&valid_effect, &context);
        assert!(valid_result.is_ok());
    }
    
    #[test]
    fn test_compilation_options() {
        let default_options = CompilationOptions::default();
        assert!(default_options.validate);
        assert!(default_options.optimize);
        
        let custom_options = CompilationOptions {
            validate: false,
            optimize: false,
            gas_limit: Some(5000),
            dry_run: true,
        };
        
        assert!(!custom_options.validate);
        assert!(!custom_options.optimize);
        assert_eq!(custom_options.gas_limit, Some(5000));
        assert!(custom_options.dry_run);
    }
} 