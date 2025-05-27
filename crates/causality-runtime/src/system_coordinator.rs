//! System Coordinator - Main orchestration component for Causality Runtime
//!
//! Handles high-level coordination of runtime components and execution flow.

use std::sync::Arc;
use anyhow::Result;

// Import TEL-specific types with explicit imports to avoid confusion
use causality_types::{
    tel::EffectGraph,
    graph::execution::InterpreterMode,
    graph::execution::GraphExecutionContext,
    core::Intent as CoreIntent,
};
use crate::tel::interpreter::Interpreter as LispInterpreterService;
use crate::tel::intent_processor::IntentProcessor;
use crate::tel::graph_executor::EffectGraphExecutor;

#[derive(Debug)]
pub struct SystemCoordinator {
    lisp_service: Arc<LispInterpreterService>,
}

impl SystemCoordinator {
    pub fn new(lisp_service: Arc<LispInterpreterService>) -> Self {
        Self { lisp_service }
    }

    pub async fn process_intent_and_execute_graph(
        &self,
        intent: CoreIntent,
        initial_mode: InterpreterMode,
    ) -> Result<(EffectGraph, GraphExecutionContext)> {
        log::info!("SystemCoordinator: Processing intent {:?} and executing graph in mode {:?}", intent.id, initial_mode);

        let intent_processor = IntentProcessor::new(Arc::clone(&self.lisp_service));
        let effect_graph_executor = EffectGraphExecutor::new(Arc::clone(&self.lisp_service));

        // 1. Create initial GraphExecutionContext for the intent processing itself.
        //    Intents might need their own context, potentially simpler or with specific setup.
        //    For now, let's use the target execution mode for the intent's graph generation phase too.
        let intent_processing_context = GraphExecutionContext::new(initial_mode);
        
        log::debug!("SystemCoordinator: Generating graph from intent {:?} with context: {:?}", intent.id, intent_processing_context);
        let effect_graph = intent_processor.process_intent(&intent, &intent_processing_context).await?;
        log::info!("SystemCoordinator: Generated graph {:?} from intent {:?}", effect_graph.id, intent.id);

        // 2. Prepare GraphExecutionContext for the graph execution.
        //    This might be the same context, or a new one initialized based on the graph or intent processing results.
        //    For simplicity, we'll create a new context for execution using the `initial_mode`.
        //    Visible resources from intent processing might be relevant here, but for now, starting fresh for graph exec.
        let graph_execution_context = GraphExecutionContext::new(initial_mode);

        log::debug!("SystemCoordinator: Executing generated graph {:?} with context: {:?}", effect_graph.id, graph_execution_context);
        let (final_graph, final_context) = effect_graph_executor.execute_graph(effect_graph, graph_execution_context).await?;
        log::info!("SystemCoordinator: Finished executing graph from intent {:?}. Final graph ID: {:?}, Final context completed effects: {}", 
            intent.id, final_graph.id, final_context.completed_effects.len());

        Ok((final_graph, final_context))
    }
} 