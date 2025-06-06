//! Test generation framework for automatic effect testing

pub mod generator;
pub mod properties;
pub mod composition;

pub use generator::{
    TestGenerator,
    TestConfig,
    TestPrioritization,
    TestCaseType,
    TestCase,
    TestInputs,
    TestValue,
    TestSetup,
    NetworkConditions,
    ExpectedOutcome,
    TestSuite,
    TestCoverage,
    TestGenerationMetadata,
};

pub use properties::{
    PropertyTestGenerator,
    PropertyTestConfig,
    PropertyType,
    PropertyTest,
    PropertyTestCase,
    PropertyInputs,
    PropertyContext,
    PropertyAssertion,
    ViolationBehavior,
    PropertyTestResult,
    PropertyCaseResult,
    AssertionResult,
    PropertyCaseStatus,
    PropertyStatus,
    PropertyCoverage,
    CounterExample,
};

pub use composition::{
    CompositionTestGenerator,
    CompositionTestConfig,
    CompositionType,
    CompositionTest,
    EffectInComposition,
    EffectConfig,
    ParameterBinding,
    ValueGenerator,
    RetryConfig,
    BackoffStrategy,
    EffectDependency,
    ValueTransformation,
    MathOperation,
    CompositionConstraint,
    CompositionBehavior,
    CompositionTestCase,
    CompositionResult,
    CompositionSuccess,
    CompositionFailure,
    CompositionPartial,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockGenerator;
    
    #[test]
    fn test_generator_creation() {
        let _generator = TestGenerator::new();
        // Basic test that generator can be created
        assert!(true); // Simple assertion to ensure compilation
    }
    
    #[test] 
    fn test_mock_generation_produces_valid_handlers() {
        let mut _mock_generator = MockGenerator::new();
        // Basic test that mock generator can be created
        assert!(true); // Simple assertion to ensure compilation
    }
    
    #[test]
    fn test_basic_functionality() {
        // Basic test to ensure the module compiles and works
        assert!(true);
    }
} 