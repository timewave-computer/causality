//! Causality Toolkit
//!
//! Development utilities and tools for working with the Causality framework.
//! This crate provides helper functions, debugging tools, development utilities,
//! and automatic mock/test generation for algebraic effects.

pub mod utils;
pub mod debug;
pub mod effects;
pub mod mocks;
pub mod testing;
pub mod dsl;
pub mod resources;
pub mod formal_verification;
pub mod cross_language;

pub use utils::*;
pub use debug::*;
pub use effects::*;
pub use dsl::*;
pub use resources::*;
pub use formal_verification::*;
pub use cross_language::*;
// Re-export specific mock components
pub use mocks::{
    strategy::*,
    generator::MockGenerator,
    blockchain::*,
};
// Re-export specific testing components  
pub use testing::{
    generator::TestGenerator,
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
    // Property testing exports
    properties::PropertyTestGenerator,
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
    // Composition testing exports
    composition::CompositionTestGenerator,
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