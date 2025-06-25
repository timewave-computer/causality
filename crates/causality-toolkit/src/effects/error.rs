//! Error types for automatic mock and test generation

use serde::{Serialize, Deserialize};
/// Errors that can occur during mock generation
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum MockError {
    #[error("Schema not found for effect: {0}")]
    SchemaNotFound(String),
    
    #[error("Invalid parameter value for {parameter}: {reason}")]
    InvalidParameter {
        parameter: String,
        reason: String,
    },
    
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    
    #[error("Type mismatch for parameter {parameter}: expected {expected}, got {actual}")]
    TypeMismatch {
        parameter: String,
        expected: String,
        actual: String,
    },
    
    #[error("Mock generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Unsupported parameter type: {0}")]
    UnsupportedType(String),
    
    #[error("Schema validation error: {0}")]
    SchemaValidation(String),
}

/// Errors that can occur during test generation
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum TestError {
    #[error("Test case generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Invalid test configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Test execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Mock not found for effect: {0}")]
    MockNotFound(String),
    
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),
    
    #[error("Test timeout after {duration_ms}ms")]
    Timeout { duration_ms: u64 },
    
    #[error("Effect handler error: {0}")]
    HandlerError(String),
    
    #[error("Schema error: {0}")]
    Schema(#[from] crate::effects::schema::SchemaError),
    
    #[error("Mock error: {0}")]
    Mock(#[from] MockError),
}

/// General errors for the automatic testing system
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum AutoTestError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Test error: {0}")]
    Test(#[from] TestError),
    
    #[error("Mock error: {0}")]
    Mock(#[from] MockError),
}

/// Result type for mock operations
pub type MockResult<T> = Result<T, MockError>;

/// Result type for test operations
pub type TestResult<T> = Result<T, TestError>;

/// Result type for general auto-test operations
pub type AutoTestResult<T> = Result<T, AutoTestError>;

impl MockError {
    /// Create a schema not found error
    pub fn schema_not_found(effect_name: impl Into<String>) -> Self {
        MockError::SchemaNotFound(effect_name.into())
    }
    
    /// Create an invalid parameter error
    pub fn invalid_parameter(parameter: impl Into<String>, reason: impl Into<String>) -> Self {
        MockError::InvalidParameter {
            parameter: parameter.into(),
            reason: reason.into(),
        }
    }
    
    /// Create a missing parameter error
    pub fn missing_parameter(parameter: impl Into<String>) -> Self {
        MockError::MissingParameter(parameter.into())
    }
    
    /// Create a type mismatch error
    pub fn type_mismatch(
        parameter: impl Into<String>,
        expected: impl Into<String>, 
        actual: impl Into<String>
    ) -> Self {
        MockError::TypeMismatch {
            parameter: parameter.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

impl TestError {
    /// Create a generation failed error
    pub fn generation_failed(reason: impl Into<String>) -> Self {
        TestError::GenerationFailed(reason.into())
    }
    
    /// Create an execution failed error
    pub fn execution_failed(reason: impl Into<String>) -> Self {
        TestError::ExecutionFailed(reason.into())
    }
    
    /// Create an assertion failed error
    pub fn assertion_failed(message: impl Into<String>) -> Self {
        TestError::AssertionFailed(message.into())
    }
    
    /// Create a timeout error
    pub fn timeout(duration_ms: u64) -> Self {
        TestError::Timeout { duration_ms }
    }
}

impl AutoTestError {
    /// Create a configuration error
    pub fn configuration(reason: impl Into<String>) -> Self {
        AutoTestError::Configuration(reason.into())
    }
    
    /// Create a runtime error
    pub fn runtime(reason: impl Into<String>) -> Self {
        AutoTestError::Runtime(reason.into())
    }
    
    /// Create an IO error
    pub fn io(reason: impl Into<String>) -> Self {
        AutoTestError::Io(reason.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_error_creation() {
        let error = MockError::schema_not_found("token_transfer");
        assert!(matches!(error, MockError::SchemaNotFound(_)));
        assert_eq!(error.to_string(), "Schema not found for effect: token_transfer");
        
        let error = MockError::invalid_parameter("amount", "must be positive");
        assert!(matches!(error, MockError::InvalidParameter { .. }));
        
        let error = MockError::missing_parameter("to_address");
        assert!(matches!(error, MockError::MissingParameter(_)));
        
        let error = MockError::type_mismatch("amount", "u64", "string");
        assert!(matches!(error, MockError::TypeMismatch { .. }));
    }
    
    #[test]
    fn test_test_error_creation() {
        let error = TestError::generation_failed("invalid schema");
        assert!(matches!(error, TestError::GenerationFailed(_)));
        
        let error = TestError::execution_failed("mock not found");
        assert!(matches!(error, TestError::ExecutionFailed(_)));
        
        let error = TestError::assertion_failed("expected success, got failure");
        assert!(matches!(error, TestError::AssertionFailed(_)));
        
        let error = TestError::timeout(5000);
        assert!(matches!(error, TestError::Timeout { duration_ms: 5000 }));
    }
    
    #[test]
    fn test_error_conversion() {
        let mock_error = MockError::schema_not_found("test_effect");
        let test_error: TestError = mock_error.into();
        assert!(matches!(test_error, TestError::Mock(_)));
        
        let auto_error: AutoTestError = test_error.into();
        assert!(matches!(auto_error, AutoTestError::Test(_)));
    }
    
    #[test]
    fn test_error_serialization() {
        let error = MockError::invalid_parameter("amount", "must be positive");
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: MockError = serde_json::from_str(&serialized).unwrap();
        
        match (&error, &deserialized) {
            (
                MockError::InvalidParameter { parameter: p1, reason: r1 },
                MockError::InvalidParameter { parameter: p2, reason: r2 }
            ) => {
                assert_eq!(p1, p2);
                assert_eq!(r1, r2);
            }
            _ => panic!("Serialization/deserialization failed"),
        }
    }
} 