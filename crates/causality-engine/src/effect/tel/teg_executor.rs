// TEG Executor for the Causality Engine
//
// This module provides functionality for executing Temporal Effect Graphs (TEG)
// created from TEL programs.

use std::sync::Arc;
use std::collections::HashMap;

use anyhow::{Result, anyhow};
use async_trait::async_trait;

use causality_core::effect::{
    Effect as CoreEffect,
    EffectId as CoreEffectId,
    EffectType as CoreEffectType,
    EffectContext as CoreEffectContext,
    EffectOutcome as CoreEffectOutcome,
    EffectResult as CoreEffectResult,
    EffectError as CoreEffectError,
};

use causality_core::resource::{
    Resource as CoreResource,
    ResourceManager,
    ResourceId,
    ResourceError,
    ResourceResult
};

use causality_tel::types::effect::{TelEffect, EffectError};
use causality_ir::{
    TemporalEffectGraph, TEGFragment, EffectNode, ResourceNode, 
    EffectId, ResourceId as TegResourceId, DomainId,
    graph::edge::{Condition, TemporalRelation}
};

use crate::effect::registry::EffectRegistry;
use crate::effect::executor::EffectExecutor;
use super::TelEffectAdapter;
use super::teg_resource::TegResourceRegistry;

/// Execution result for a TEG
#[derive(Debug)]
pub struct TegExecutionResult {
    /// Output values from execution
    pub outputs: HashMap<String, serde_json::Value>,
    
    /// Execution metrics and statistics
    pub metrics: HashMap<String, serde_json::Value>,
    
    /// Execution trace for debugging
    pub trace: Vec<TraceEntry>,
}

/// Trace entry for execution debugging
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// Effect ID that was executed
    pub effect_id: EffectId,
    
    /// Effect type
    pub effect_type: String,
    
    /// Execution result
    pub result: serde_json::Value,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Executor for Temporal Effect Graphs
pub struct TegExecutor {
    /// Core effect executor
    core_executor: Arc<EffectExecutor>,
    
    /// Resource manager
    resource_manager: Arc<dyn ResourceManager>,
    
    /// Resource registry for TEG operations
    teg_resource_registry: TegResourceRegistry,
}

impl TegExecutor {
    /// Create a new TEG executor
    pub fn new(
        core_executor: Arc<EffectExecutor>,
        resource_manager: Arc<dyn ResourceManager>,
    ) -> Self {
        Self {
            core_executor,
            resource_manager: resource_manager.clone(),
            teg_resource_registry: TegResourceRegistry::new(resource_manager),
        }
    }
    
    /// Execute a Temporal Effect Graph
    pub async fn execute(&self, teg: &TemporalEffectGraph) -> Result<TegExecutionResult> {
        // Create execution context
        let context = TegExecutionContext {
            variables: HashMap::new(),
            resources: HashMap::new(),
            core_context: self.core_executor.create_context(),
        };
        
        // Find entry points (effects without dependencies)
        let entry_points = self.find_entry_points(teg)?;
        
        if entry_points.is_empty() {
            return Err(anyhow!("No entry points found in TEG"));
        }
        
        // Execute the graph starting from entry points
        let mut result = TegExecutionResult {
            outputs: HashMap::new(),
            metrics: HashMap::new(),
            trace: Vec::new(),
        };
        
        let mut execution_queue = entry_points;
        let mut executed_effects = HashMap::new();
        let mut trace = Vec::new();
        
        // Execute effects in topological order
        while !execution_queue.is_empty() {
            let effect_id = execution_queue.remove(0);
            
            // Skip if already executed
            if executed_effects.contains_key(&effect_id) {
                continue;
            }
            
            // Get the effect node
            let effect = teg.effect_nodes.get(&effect_id)
                .ok_or_else(|| anyhow!("Effect node not found: {}", effect_id))?;
            
            // Check if all dependencies are satisfied
            let dependencies = teg.effect_dependencies.get(&effect_id).cloned().unwrap_or_default();
            let all_deps_executed = dependencies.iter().all(|dep_id| executed_effects.contains_key(dep_id));
            
            if !all_deps_executed {
                // Put back in queue and continue
                execution_queue.push(effect_id);
                continue;
            }
            
            // Execute the effect
            let effect_outcome = self.execute_effect(effect, &context).await?;
            
            // Record the execution
            executed_effects.insert(effect_id.clone(), effect_outcome.clone());
            
            // Add to trace
            trace.push(TraceEntry {
                effect_id: effect_id.clone(),
                effect_type: effect.effect_type.clone(),
                result: serde_json::Value::String(format!("{:?}", effect_outcome)),
                timestamp: chrono::Utc::now(),
            });
            
            // Find continuations
            if let Some(continuations) = teg.effect_continuations.get(&effect_id) {
                for (next_effect_id, condition_opt) in continuations {
                    // Check condition if present
                    let should_continue = match condition_opt {
                        Some(condition) => self.evaluate_condition(condition, &context)?,
                        None => true,
                    };
                    
                    if should_continue {
                        execution_queue.push(next_effect_id.clone());
                    }
                }
            }
        }
        
        // Set trace in result
        result.trace = trace;
        
        // Collect outputs from exit nodes
        let exit_points = self.find_exit_points(teg)?;
        for exit_id in exit_points {
            if let Some(outcome) = executed_effects.get(&exit_id) {
                result.outputs.insert(exit_id, serde_json::Value::String(format!("{:?}", outcome)));
            }
        }
        
        Ok(result)
    }
    
    /// Find entry points in the graph (effects without dependencies)
    fn find_entry_points(&self, teg: &TemporalEffectGraph) -> Result<Vec<EffectId>> {
        let mut entry_points = Vec::new();
        
        for effect_id in teg.effect_nodes.keys() {
            let has_deps = teg.effect_dependencies.get(effect_id)
                .map(|deps| !deps.is_empty())
                .unwrap_or(false);
            
            if !has_deps {
                entry_points.push(effect_id.clone());
            }
        }
        
        Ok(entry_points)
    }
    
    /// Find exit points in the graph (effects with no continuations)
    fn find_exit_points(&self, teg: &TemporalEffectGraph) -> Result<Vec<EffectId>> {
        let mut exit_points = Vec::new();
        
        for effect_id in teg.effect_nodes.keys() {
            let has_continuations = teg.effect_continuations.get(effect_id)
                .map(|conts| !conts.is_empty())
                .unwrap_or(false);
            
            if !has_continuations {
                exit_points.push(effect_id.clone());
            }
        }
        
        Ok(exit_points)
    }
    
    /// Execute a single effect node
    async fn execute_effect(
        &self, 
        effect: &EffectNode,
        context: &TegExecutionContext,
    ) -> Result<CoreEffectOutcome> {
        // Create adapter based on effect type
        let adapter = match effect.effect_type.as_str() {
            "identity" => {
                // Create identity effect
                let adapter = TelEffectAdapter::new("identity", causality_tel::combinators::Combinator::I);
                adapter
            },
            "constant" => {
                // Create constant effect
                let value = effect.parameters.get("value")
                    .ok_or_else(|| anyhow!("Missing value parameter for constant effect"))?;
                
                let adapter = TelEffectAdapter::new(
                    "constant", 
                    causality_tel::combinators::Combinator::Literal(
                        causality_tel::combinators::Literal::from_json(value.clone())
                    )
                );
                adapter
            },
            effect_type if effect_type.starts_with("effect_") => {
                // Extract effect name (remove the "effect_" prefix)
                let effect_name = effect_type.strip_prefix("effect_").unwrap_or(effect_type);
                
                // Create effect adapter
                let adapter = TelEffectAdapter::new(
                    effect_name, 
                    causality_tel::combinators::Combinator::Effect {
                        effect_name: effect_name.to_string(),
                        args: Vec::new(), // Add parameters from effect node
                        core_effect: None,
                    }
                );
                adapter
            },
            // Handle resource operations
            resource_op if resource_op.starts_with("resource_") => {
                // For resource operations, use resource manager directly
                return self.handle_resource_operation(effect, context).await;
            },
            // Handle state transitions
            "state_transition" => {
                // Get resource ID and state information
                let resource_id = effect.parameters.get("resource_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing resource_id parameter for state transition"))?;
                
                let from_state = effect.parameters.get("from_state")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing from_state parameter for state transition"))?;
                
                let to_state = effect.parameters.get("to_state")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing to_state parameter for state transition"))?;
                
                // Create adapter for state transition
                let adapter = TelEffectAdapter::new(
                    "state_transition",
                    causality_tel::combinators::Combinator::StateTransition {
                        target_state: to_state.to_string(),
                        fields: HashMap::new(), // We could add fields from effect.parameters if needed
                        resource_id: Some(resource_id.to_string()),
                    }
                );
                adapter
            },
            _ => {
                // Default to passthrough
                let adapter = TelEffectAdapter::new(
                    &effect.effect_type,
                    causality_tel::combinators::Combinator::I
                );
                adapter
            }
        };
        
        // Execute the effect
        adapter.execute(&*context.core_context).await
            .map_err(|e| anyhow!("Failed to execute effect {}: {}", effect.id, e))
    }
    
    /// Handle a resource operation directly using the resource manager
    async fn handle_resource_operation(
        &self,
        effect: &EffectNode,
        context: &TegExecutionContext,
    ) -> Result<CoreEffectOutcome> {
        // Extract operation type from effect_type
        let operation = effect.effect_type.strip_prefix("resource_")
            .ok_or_else(|| anyhow!("Invalid resource operation: {}", effect.effect_type))?;
        
        // Get resource type and ID from parameters
        let resource_type = effect.parameters.get("resource_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing resource_type parameter for resource operation"))?;
        
        let resource_id = effect.parameters.get("resource_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Convert parameters to a format suitable for the resource manager
        let mut params = HashMap::new();
        for (key, value) in &effect.parameters {
            if key != "resource_type" && key != "resource_id" && key != "operation" {
                params.insert(key.clone(), value.clone());
            }
        }
        
        // Create a JSON representation of the parameters
        let params_json = serde_json::to_value(params)?;
        
        // Execute the resource operation
        match operation {
            "create" => {
                // Create a new resource
                let resource = self.resource_manager.create_resource(
                    resource_type,
                    resource_id.as_deref(),
                    params_json
                ).await
                .map_err(|e| anyhow!("Failed to create resource: {}", e))?;
                
                // Return the resource ID as the outcome
                let mut outcome_data = HashMap::new();
                outcome_data.insert("resource_id".to_string(), resource.id().to_string());
                outcome_data.insert("resource_type".to_string(), resource_type.to_string());
                
                Ok(CoreEffectOutcome::success(outcome_data))
            },
            "get" => {
                // Get an existing resource
                let resource_id = resource_id
                    .ok_or_else(|| anyhow!("Missing resource_id for get operation"))?;
                
                let resource = self.resource_manager.get_resource(
                    resource_type,
                    &resource_id
                ).await
                .map_err(|e| anyhow!("Failed to get resource {}: {}", resource_id, e))?;
                
                // Extract resource data
                let resource_data = resource.to_json()
                    .map_err(|e| anyhow!("Failed to serialize resource: {}", e))?;
                
                // Return the resource data as the outcome
                let mut outcome_data = HashMap::new();
                outcome_data.insert("resource_id".to_string(), resource_id);
                outcome_data.insert("resource_type".to_string(), resource_type.to_string());
                outcome_data.insert("data".to_string(), serde_json::to_string(&resource_data)?);
                
                Ok(CoreEffectOutcome::success(outcome_data))
            },
            "update" => {
                // Update an existing resource
                let resource_id = resource_id
                    .ok_or_else(|| anyhow!("Missing resource_id for update operation"))?;
                
                let resource = self.resource_manager.update_resource(
                    resource_type,
                    &resource_id,
                    params_json
                ).await
                .map_err(|e| anyhow!("Failed to update resource {}: {}", resource_id, e))?;
                
                // Return success outcome
                let mut outcome_data = HashMap::new();
                outcome_data.insert("resource_id".to_string(), resource_id);
                outcome_data.insert("resource_type".to_string(), resource_type.to_string());
                
                Ok(CoreEffectOutcome::success(outcome_data))
            },
            "delete" => {
                // Delete an existing resource
                let resource_id = resource_id
                    .ok_or_else(|| anyhow!("Missing resource_id for delete operation"))?;
                
                self.resource_manager.delete_resource(
                    resource_type,
                    &resource_id
                ).await
                .map_err(|e| anyhow!("Failed to delete resource {}: {}", resource_id, e))?;
                
                // Return success outcome
                let mut outcome_data = HashMap::new();
                outcome_data.insert("resource_id".to_string(), resource_id);
                outcome_data.insert("resource_type".to_string(), resource_type.to_string());
                
                Ok(CoreEffectOutcome::success(outcome_data))
            },
            "query" => {
                // Query resources based on parameters
                let query_params = params_json;
                
                // In a real implementation, this would use a more sophisticated query API
                // For now, we'll just return an empty result
                let result = Vec::<String>::new();
                
                // Return query result as outcome
                let mut outcome_data = HashMap::new();
                outcome_data.insert("resource_type".to_string(), resource_type.to_string());
                outcome_data.insert("results".to_string(), serde_json::to_string(&result)?);
                
                Ok(CoreEffectOutcome::success(outcome_data))
            },
            _ => Err(anyhow!("Unsupported resource operation: {}", operation))
        }
    }
    
    /// Evaluate a condition
    fn evaluate_condition(&self, condition: &Condition, context: &TegExecutionContext) -> Result<bool> {
        // Simple implementation for now
        match condition {
            Condition::Always => Ok(true),
            Condition::Never => Ok(false),
            Condition::Equals(key, value) => {
                // Check if variable exists and equals value
                if let Some(var) = context.variables.get(key) {
                    Ok(var.to_string() == value.to_string())
                } else {
                    Ok(false)
                }
            },
            _ => {
                // Default to true for other conditions (will be implemented later)
                Ok(true)
            }
        }
    }
}

/// Execution context for TEG execution
struct TegExecutionContext {
    /// Variables during execution
    variables: HashMap<String, serde_json::Value>,
    
    /// Resources during execution
    resources: HashMap<TegResourceId, ResourceNode>,
    
    /// Core effect context
    core_context: Box<dyn CoreEffectContext>,
} 