//! Tests for optimization strategy implementations

use super::*;
use crate::optimization::{OptimizationStrategy, OptimizationContext};
use causality_types::{
    core::id::{DomainId, ExprId},
    tel::optimization::TypedDomain,
};

#[test]
fn test_capital_efficiency_strategy_creation() {
    let strategy = CapitalEfficiencyStrategy::new();
    assert_eq!(strategy.strategy_id(), "capital_efficiency");
    assert!(strategy.supports_typed_domain(&TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]))));
}

#[test]
fn test_priority_based_strategy_creation() {
    let strategy = PriorityBasedStrategy::new();
    assert_eq!(strategy.strategy_id(), "priority_based");
    assert!(strategy.supports_typed_domain(&TypedDomain::ServiceDomain(DomainId::new([1u8; 32]))));
}

#[test]
fn test_dataflow_orchestration_strategy_creation() {
    let strategy = ProcessDataflowOrchestrationStrategy::new();
    assert_eq!(strategy.strategy_id(), "dataflow_orchestration");
    assert!(strategy.supports_typed_domain(&TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]))));
}

#[test]
fn test_expression_based_strategy_creation() {
    let strategy = ExpressionBasedStrategy::new();
    assert_eq!(strategy.strategy_id(), "expression_based");
    assert!(strategy.supports_typed_domain(&TypedDomain::VerifiableDomain(DomainId::new([1u8; 32]))));
    assert!(strategy.supports_typed_domain(&TypedDomain::ServiceDomain(DomainId::new([1u8; 32]))));
}

#[test]
fn test_expression_based_strategy_configuration() {
    // Test builder pattern
    let expr_id = ExprId::new([1u8; 32]);
    let configured_strategy = ExpressionBasedStrategy::new()
        .with_scoring_expression(expr_id)
        .with_filter_expression(expr_id)
        .with_domain_compatibility_expression(expr_id);
    
    // Note: These fields are private, so we can't directly test them
    // In a real implementation, we'd have getter methods or make them public
    assert_eq!(configured_strategy.strategy_id(), "expression_based");
} 