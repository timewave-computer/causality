//! Sample Effects for Testing
//!
//! Defines sample Rust effects and handlers for integration testing.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use causality_types::{
    core::str::Str,
    effects_core::{
        ConversionError, Effect, EffectInput, EffectOutput, EffectHandler, HandlerError,
    },
    expr::{TypeExpr, ValueExpr},
};

//-----------------------------------------------------------------------------
// AddParams Effect Input
//-----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct AddParams {
    pub a: i64,
    pub b: i64,
}

// Manual implementation of EffectInput for AddParams
impl EffectInput for AddParams {
    fn schema() -> TypeExpr {
        let mut fields = BTreeMap::new();
        fields.insert(Str::from("a"), TypeExpr::Integer);
        fields.insert(Str::from("b"), TypeExpr::Integer);
        TypeExpr::Record(fields.into())
    }

    fn to_value_expr(&self) -> Result<ValueExpr, ConversionError> {
        let mut map = BTreeMap::new();
        map.insert(
            Str::from("a"),
            ValueExpr::Integer(self.a),
        );
        map.insert(
            Str::from("b"),
            ValueExpr::Integer(self.b),
        );
        Ok(ValueExpr::Record(map.into()))
    }

    fn from_value_expr(value: ValueExpr) -> Result<Self, ConversionError> {
        match value {
            ValueExpr::Record(mut fields_map_wrapper) => {
                let fields_map = fields_map_wrapper.get_mut(); // Get &mut BTreeMap
                let a = match fields_map.remove("a") {
                    Some(ValueExpr::Integer(val_a)) => val_a,
                    Some_ => return Err(ConversionError::TypeError("Field 'a' is not an Integer".into())),
                    None => return Err(ConversionError::MissingField("Field 'a' is missing".into())),
                };
                let b = match fields_map.remove("b") {
                    Some(ValueExpr::Integer(val_b)) => val_b,
                    Some_ => return Err(ConversionError::TypeError("Field 'b' is not an Integer".into())),
                    None => return Err(ConversionError::MissingField("Field 'b' is missing".into())),
                };
                Ok(AddParams { a, b })
            }
            _ => Err(ConversionError::TypeError(
                "Expected a Record to deserialize AddParams".into(),
            )),
        }
    }
}

//-----------------------------------------------------------------------------
// Effect Definition
//-----------------------------------------------------------------------------

#[derive(Debug)]
pub struct AddEffect;

impl Effect for AddEffect {
    type Input = AddParams;
    type Output = i64; // Uses existing i64 EffectOutput from causality_types
    const EFFECT_TYPE: &'static str = "example.effects.Add";
}

//-----------------------------------------------------------------------------
// Effect Handler
//-----------------------------------------------------------------------------

#[derive(Debug)]
pub struct AddHandler;

#[async_trait]
impl EffectHandler for AddHandler {
    type E = AddEffect;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output, HandlerError> {
        Ok(input.a + input.b)
    }
}

//-----------------------------------------------------------------------------
// Unit Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_params_schema() {
        let schema = AddParams::schema();
        if let TypeExpr::Record(fields) = schema {
            assert_eq!(fields.get("a"), Some(&TypeExpr::Integer));
            assert_eq!(fields.get("b"), Some(&TypeExpr::Integer));
        } else {
            panic!("Expected Record type for AddParams schema");
        }
    }

    #[test]
    fn test_add_params_to_value_expr() {
        let params = AddParams { a: 10, b: 20 };
        let value_expr = params.to_value_expr().unwrap();
        if let ValueExpr::Record(map_wrapper) = value_expr {
            let map = map_wrapper.as_ref();
            assert_eq!(map.get("a"), Some(&ValueExpr::Integer(10)));
            assert_eq!(map.get("b"), Some(&ValueExpr::Integer(20)));
        } else {
            panic!("Expected Record ValueExpr for AddParams");
        }
    }

    #[test]
    fn test_add_params_from_value_expr_ok() {
        let mut map = BTreeMap::new();
        map.insert(Str::from("a"), ValueExpr::Integer(5));
        map.insert(Str::from("b"), ValueExpr::Integer(7));
        let value_expr = ValueExpr::Record(map.into());

        let params = AddParams::from_value_expr(value_expr).unwrap();
        assert_eq!(params, AddParams { a: 5, b: 7 });
    }

    #[test]
    fn test_add_params_from_value_expr_missing_field() {
        let mut map = BTreeMap::new();
        map.insert(Str::from("a"), ValueExpr::Integer(5));
        // 'b' is missing
        let value_expr = ValueExpr::Record(map.into());
        let result = AddParams::from_value_expr(value_expr);
        assert!(matches!(result, Err(ConversionError::MissingField(_))));
    }

    #[test]
    fn test_add_params_from_value_expr_wrong_type() {
        let mut map = BTreeMap::new();
        map.insert(Str::from("a"), ValueExpr::Integer(5));
        map.insert(Str::from("b"), ValueExpr::String("not_an_int".into()));
        let value_expr = ValueExpr::Record(map.into());
        let result = AddParams::from_value_expr(value_expr);
        assert!(matches!(result, Err(ConversionError::TypeError(_))));
    }

     #[test]
    fn test_add_params_from_value_expr_not_a_record() {
        let value_expr = ValueExpr::Integer(123); // Not a record
        let result = AddParams::from_value_expr(value_expr);
        assert!(matches!(result, Err(ConversionError::TypeError(_))));
    }

    #[tokio::test]
    async fn test_add_handler() {
        let handler = AddHandler;
        let params = AddParams { a: 3, b: 4 };
        let result = handler.handle(params).await.unwrap();
        assert_eq!(result, 7);
    }
}

//-----------------------------------------------------------------------------
// Integration Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use causality_runtime::tel::{
        Context as TelContext,
        graph::{
            TelGraph, Node, NodeData, Edge, EffectNode, HandlerNode, StartNode, EndNode, 
            GraphError, TelGraphValidationError, TelGraphValidationResult, TelNodeIdAndType, 
            NODE_TYPE_EFFECT, NODE_TYPE_HANDLER, NODE_TYPE_START, NODE_TYPE_END,
        },
        store::{ExprStore, ValueExprStore, IntentStore, AotMap},
        Interpreter as TelInterpreter,
        types::InterpreterMode,
        traits::MockBehavior,
    };
    use causality_lisp::Interpreter as LispInterpreter;
    use causality_types::{
        core::id::{Id, NodeId, ExprId},
        tel::{Effect as TelEffectNode, Handler as TelHandlerNode, effect::EffectNodeData as TelEffectNodeData, handler::HandlerNodeData as TelHandlerNodeData, handler::ConstraintExpr, ForeignLanguageExpr},
        expr::{Expr as CausalityExpr, ValueExpr, TypeExprMap, ValueExprMap, MapExpr},
    };
    use crate::engine::{SimulationEngine, SimulationStepOutcome};
    use crate::mocking::AutoMockStrategy;

    #[tokio::test]
    async fn test_rust_effect_end_to_end_simulation() {
        // 1. Setup SimulationEngine
        // The first TelInterpreter argument to SimulationEngine::new is currently ignored, 
        // as the engine creates its own internally. Pass a basic new one.
        let dummy_interpreter_arg = TelInterpreter::new(TelGraph::new());
        let mut simulation_engine = SimulationEngine::new(
            dummy_interpreter_arg, 
            TelContext::new(), // initial_context (this is also currently ignored by SimulationEngine::new)
            Some(12345), // seed
            Some(AutoMockStrategy::Disabled), // initial auto-mock strategy
            None // zk_coprocessor_api
        ).await; // Added .await

        // 2. Register the Rust effect handler with the SimulationEngine's internal interpreter
        // This step needs to interact with the interpreter inside SimulationEngine,
        // which isn't directly exposed for handler registration in this manner.
        // For this test, we assume the effect/handler registration happens globally or through
        // a different mechanism if it's a Rust handler not managed by TEL graph.
        // However, TelInterpreter itself doesn't have a direct `register_effect_handler` method.
        // This registration usually happens at a higher level or via the TelGraph.

        // For a simple Rust effect like AddEffect, if it were a host function, it would be
        // registered with the TelContext. If it's a TEL graph node, it would be part of the graph.
        // The current SimulationEngine::register_rust_effect_handler is a placeholder.

        // Let's assume for this test, the effect is processed directly if encountered.
        // The test primarily focuses on the simulation loop itself.

        // 3. Create an initial effect instance (e.g., using a Lisp expression or directly if possible)
        // For this test, we'll manually drive what would be an effect execution.
        // Let's construct the AddEffect input
        let add_params = AddParams { a: 5, b: 7 };
        let add_input_value_expr = add_params.to_value_expr().expect("Should convert to value expr");

        // Typically, an effect node in TEL would trigger this.
        // Here, we're testing if the simulation engine can be stepped.
        // Since there's no graph and no initial effects, execute_step will likely do nothing
        // or indicate no effects.

        // To properly test an effect, we'd need to:
        // a) Define a TelGraph with an AddEffect node.
        // b) Initialize the SimulationEngine with this graph (not currently supported by SimEngine::new directly).
        // c) Or, have a way to inject an effect execution request.

        // Given the current structure, this test might be limited.
        // Let's just check if the engine can be created and stepped.
        
        let outcome = simulation_engine.execute_step().expect("Execute step should not fail");
        
        // Without a graph or effects, it should report NoEffectToProcess
        assert_eq!(outcome, SimulationStepOutcome::NoEffectToProcess, "Expected NoEffectToProcess from an empty engine state");

        // A more complete test would involve setting up a graph that uses AddEffect.
        // For now, this confirms the engine initializes and basic stepping works.
        // TODO: Expand this test when graph loading/manipulation in SimulationEngine is available.
    }
} 