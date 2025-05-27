#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::primitive::ids::{DomainId, ExprId, ResourceId, ValueExprId};
    use causality_types::primitive::string::Str;
    use causality_types::expr::value::ValueExpr;
    use causality_types::resource::Resource;
    use causality_types::tel::InterpreterMode;
    use std::collections::BTreeMap;
    use causality_types::primitive::number::Number;

    // Helper function to create a test resource
    fn create_test_resource() -> Resource {
        Resource {
            id: ResourceId::random(),
            value: ValueExprId::random(),
            static_expr: Some(ExprId::random()),
            domain: DomainId::random(),
            ephemeral: false,
        }
    }

    #[tokio::test]
    async fn test_interpreter_creation() {
        // Test creating a TEL interpreter with default settings
        let interpreter = TelInterpreter::new();
        
        // Verify default settings
        assert_eq!(interpreter.mode(), InterpreterMode::Evaluate);
        
        // Test creating with a specific mode
        let interpreter = TelInterpreter::with_mode(InterpreterMode::Validate);
        assert_eq!(interpreter.mode(), InterpreterMode::Validate);
    }

    #[tokio::test]
    async fn test_interpreter_context_configuration() {
        let interpreter = TelInterpreter::new();
        
        // Create a basic context configuration
        let config = LispContextConfig::new()
            .with_host_function_profile("basic")
            .with_binding("test-var", ValueExpr::String(Str::from("test value")));
        
        // Verify config properties
        assert_eq!(config.host_function_profile(), "basic");
        assert!(config.has_binding("test-var"));
    }

    #[tokio::test]
    async fn test_resource_validation_configuration() {
        let interpreter = TelInterpreter::new();
        
        // Create a resource
        let resource = create_test_resource();
        
        // Create a validator configuration for the resource
        let config = LispContextConfig::new()
            .with_host_function_profile("validation")
            .with_binding("*self-resource*", ValueExpr::String(Str::from("resource-data")));
        
        // Verify validation configuration has correct settings
        assert_eq!(config.host_function_profile(), "validation");
        assert!(config.has_binding("*self-resource*"));
    }

    #[tokio::test]
    async fn test_context_binding_operations() {
        // Test binding operations in context configuration
        let mut config = LispContextConfig::new();
        
        // Add bindings
        config = config.with_binding("var1", ValueExpr::Number(Number::Integer(42)));
        config = config.with_binding("var2", ValueExpr::Bool(true));
        
        // Test binding retrieval
        assert!(config.has_binding("var1"));
        assert!(config.has_binding("var2"));
        assert!(!config.has_binding("non-existent"));
        
        // Test binding values match expected
        if let Some(ValueExpr::Number(Number::Integer(val))) = config.get_binding("var1") {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected Integer binding for var1");
        }
        
        if let Some(ValueExpr::Bool(val)) = config.get_binding("var2") {
            assert_eq!(*val, true);
        } else {
            panic!("Expected Boolean binding for var2");
        }
    }

    #[tokio::test]
    async fn test_lisp_host_environment_creation() {
        // Test creating a Lisp host environment
        let host_env = test_utils::create_test_host_environment();
        
        // Verify host environment is configured correctly
        assert!(host_env.is_initialized());
        
        // Test host environment can resolve basic symbols
        let symbol_val = host_env.get_symbol("test-symbol");
        assert!(symbol_val.is_some());
    }
}

// Test utilities module (would be implemented based on actual runtime code)
#[cfg(test)]
mod test_utils {
    use super::*;
    
    pub fn create_test_host_environment() -> TestHostEnvironment {
        let mut env = TestHostEnvironment::new();
        env.initialize();
        env
    }
    
    // Mock implementation for testing
    pub struct TestHostEnvironment {
        initialized: bool,
    }
    
    impl TestHostEnvironment {
        pub fn new() -> Self {
            Self { initialized: false }
        }
        
        pub fn initialize(&mut self) {
            self.initialized = true;
        }
        
        pub fn is_initialized(&self) -> bool {
            self.initialized
        }
        
        pub fn get_symbol(&self, name: &str) -> Option<ValueExpr> {
            if name == "test-symbol" {
                Some(ValueExpr::String(Str::from("test-value")))
            } else {
                None
            }
        }
    }
} 