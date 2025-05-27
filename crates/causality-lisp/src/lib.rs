//! Purpose: Lisp interpreter for the Causality framework.

pub mod core;
pub mod context;
pub mod dataflow_combinators;

// Re-export key components
pub use core::{Interpreter, Evaluator, ExprContextual};
pub use context::DefaultExprContext;
pub use dataflow_combinators::{
    DataflowOrchestrationContext,
    get_dataflow_definition,
    evaluate_gating_condition,
    instantiate_effect_from_node,
    emit_effect_on_domain,
    update_dataflow_instance_state,
    is_zk_compatible_operation,
    validate_dataflow_step_constraints,
};
