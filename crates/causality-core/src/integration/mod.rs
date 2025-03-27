// Integration Module
//
// This module provides integration components for connecting different parts
// of the Causality system together, including domain integration, validation,
// and cross-component interoperability.

pub mod domain;
pub mod adapter;
#[cfg(test)]
mod tests;

pub use domain::{
    DomainIntegrationError, DomainIntegrationResult, DomainAdapterFactory,
    DomainValidation, GenericDomainAdapter, DomainEffectRouter,
    DomainResourceRouter, BasicDomainAdapterFactory,
    create_domain_integration_layer,
};

pub use adapter::{
    TestParameterValidator, TestDomainEffectHandler, TestDomainResourceAdapter,
    TestDomainAdapterFactory, create_test_domain_integration_layer,
}; 