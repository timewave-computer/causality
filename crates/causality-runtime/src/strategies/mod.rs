//! Basic optimization strategy implementations
//!
//! This module provides concrete implementations of optimization strategies that are
//! TypedDomain-aware and can handle ProcessDataflowBlock orchestration.

// Strategy modules
pub mod capital_efficiency;
pub mod priority_based;
pub mod process_dataflow_orchestration;
pub mod expression_based;

// Re-export all strategies for convenience
pub use capital_efficiency::CapitalEfficiencyStrategy;
pub use priority_based::PriorityBasedStrategy;
pub use process_dataflow_orchestration::ProcessDataflowOrchestrationStrategy;
pub use expression_based::ExpressionBasedStrategy;

#[cfg(test)]
mod tests; 