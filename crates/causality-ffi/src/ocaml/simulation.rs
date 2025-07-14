//! FFI bindings for OCaml to access Rust simulation functionality
//! Generic simulation interface - no domain-specific logic

use crate::error::FFIError;
use causality_simulation::{SessionSimulationEnvironment, SessionSimulationConfig, SimulationEngine, SimulationConfig};
use causality_core::machine::Instruction;
use causality_lisp::{compile_for_simulation, LispValue};
use ocaml::{Value, ToValue, FromValue, Runtime};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Global registry for simulation engines
/// Maps OCaml handle IDs to actual engine instances
static ENGINE_REGISTRY: Lazy<Arc<Mutex<HashMap<u64, SimulationEngine>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Global registry for simulation environments
static ENVIRONMENT_REGISTRY: Lazy<Arc<Mutex<HashMap<u64, SessionSimulationEnvironment>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Global counter for generating unique handles
static HANDLE_COUNTER: Lazy<Arc<Mutex<u64>>> = Lazy::new(|| Arc::new(Mutex::new(1)));

/// Generate a unique handle for registry storage
fn generate_handle() -> u64 {
    let mut counter = HANDLE_COUNTER.lock().unwrap();
    let handle = *counter;
    *counter += 1;
    handle
}

/// Create a new simulation environment
#[ocaml::func]
pub fn create_simulation_environment(_gc: &mut Runtime) -> Value {
    let config = SessionSimulationConfig::default();
    let env = SessionSimulationEnvironment::new(config);
    
    let handle = generate_handle();
    {
        let mut registry = ENVIRONMENT_REGISTRY.lock().unwrap();
        registry.insert(handle, env);
    }
    
    // Return the handle as an OCaml integer
    Value::int(handle as i64)
}

/// Create simulation engine with custom configuration
#[ocaml::func]
pub fn create_simulation_engine_with_config(
    _gc: &mut Runtime,
    max_steps: i64,
    max_gas: i64,
    enable_snapshots: bool,
) -> Value {
    let config = SimulationConfig {
        max_steps: max_steps as usize,
        gas_limit: max_gas as u64,
        enable_snapshots,
        timeout_ms: 30000,
        step_by_step_mode: false,
    };
    let engine = SimulationEngine::new_with_config(config);
    
    let handle = generate_handle();
    {
        let mut registry = ENGINE_REGISTRY.lock().unwrap();
        registry.insert(handle, engine);
    }
    
    // Return the handle as an OCaml integer
    Value::int(handle as i64)
}

/// Helper function to get engine from registry
fn with_engine<F, R>(engine_handle: Value, f: F) -> Result<R, String>
where
    F: FnOnce(&mut SimulationEngine) -> Result<R, String>,
{
    let handle = engine_handle.int_val() as u64;
    let mut registry = ENGINE_REGISTRY.lock().map_err(|e| format!("Registry lock error: {}", e))?;
    
    match registry.get_mut(&handle) {
        Some(engine) => f(engine),
        None => Err(format!("Invalid engine handle: {}", handle)),
    }
}

/// Compile Lisp S-expression to instructions and execute in simulation
#[ocaml::func]
pub fn compile_and_simulate_lisp(
    _gc: &mut Runtime,
    engine_handle: Value,
    lisp_code: String,
) -> Result<String, String> {
    with_engine(engine_handle, |engine| {
        // Compile Lisp code to instructions
        let e2e_result = compile_for_simulation(&lisp_code)
            .map_err(|e| format!("Failed to compile Lisp code: {:?}", e))?;
        
        // Execute the instructions
        match engine.execute(&e2e_result.instructions) {
            Ok(()) => {
                let metrics = engine.metrics();
                let result = format!(
                    "{{\"status\": \"success\", \"instructions_executed\": {}, \"effects_executed\": {}, \"gas_consumed\": {}, \"execution_time_ms\": {}, \"result_register\": {}}}",
                    e2e_result.instruction_count,
                    metrics.effects_executed,
                    metrics.total_gas_consumed,
                    metrics.execution_time_ms,
                    e2e_result.result_register
                );
                Ok(result)
            },
            Err(e) => Err(format!("Simulation execution failed: {:?}", e)),
        }
    })
}

/// Execute pre-compiled instructions
#[ocaml::func]
pub fn execute_instructions(
    _gc: &mut Runtime,
    engine_handle: Value,
    instruction_count: i64,
) -> Result<String, String> {
    with_engine(engine_handle, |engine| {
        // Generate generic instructions for simulation
        let instructions: Vec<Instruction> = (0..instruction_count)
            .map(|i| Instruction::Nop { register: i as u32 })
            .collect();
        
        // Execute the instructions
        match engine.execute(&instructions) {
            Ok(()) => {
                let metrics = engine.metrics();
                let result = format!(
                    "{{\"status\": \"success\", \"instructions_executed\": {}, \"effects_executed\": {}, \"gas_consumed\": {}, \"execution_time_ms\": {}}}",
                    instructions.len(),
                    metrics.effects_executed,
                    metrics.total_gas_consumed,
                    metrics.execution_time_ms
                );
                Ok(result)
            },
            Err(e) => Err(format!("Simulation execution failed: {:?}", e)),
        }
    })
}

/// Execute an effect expression
#[ocaml::func]
pub fn execute_effect_expression(
    _gc: &mut Runtime,
    engine_handle: Value,
    effect_expr: String,
) -> Result<String, String> {
    with_engine(engine_handle, |engine| {
        // Use tokio runtime for async execution
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        
        match rt.block_on(engine.execute_effect(effect_expr)) {
            Ok(result) => Ok(format!("{:?}", result)),
            Err(e) => Err(format!("Effect execution failed: {:?}", e)),
        }
    })
}

/// Get simulation engine statistics
#[ocaml::func]
pub fn get_simulation_stats(
    _gc: &mut Runtime,
    engine_handle: Value,
) -> (i64, i64, i64) {
    with_engine(engine_handle, |engine| {
        let metrics = engine.metrics();
        Ok((
            metrics.total_instructions_executed as i64,
            metrics.total_gas_consumed as i64,
            metrics.effects_executed as i64,
        ))
    }).unwrap_or((0, 0, 0))
}

/// Reset simulation engine state
#[ocaml::func]
pub fn reset_simulation_engine(
    _gc: &mut Runtime,
    engine_handle: Value,
) -> Value {
    let result = with_engine(engine_handle, |engine| {
        match engine.reset() {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    });
    
    match result {
        Ok(success) => Value::bool(success),
        Err(_) => Value::bool(false),
    }
}

/// Create a simulation snapshot
#[ocaml::func]
pub fn create_simulation_snapshot(
    _gc: &mut Runtime,
    engine_handle: Value,
    description: String,
) -> String {
    with_engine(engine_handle, |engine| {
        // Use tokio runtime for async execution
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        
        match rt.block_on(engine.create_snapshot(description.clone())) {
            Ok(snapshot_id) => Ok(snapshot_id),
            Err(e) => Err(format!("Failed to create snapshot: {:?}", e)),
        }
    }).unwrap_or_else(|e| format!("snapshot_error_{}", e))
}

/// Restore from a simulation snapshot
#[ocaml::func]
pub fn restore_simulation_snapshot(
    _gc: &mut Runtime,
    engine_handle: Value,
    snapshot_id: String,
) -> Value {
    let result = with_engine(engine_handle, |engine| {
        // Use tokio runtime for async execution
        let rt = tokio::runtime::Runtime::new()
            .map_err(|_| "Failed to create runtime")?;
        
        match rt.block_on(engine.restore_snapshot(&snapshot_id)) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    });
    
    match result {
        Ok(success) => Value::bool(success),
        Err(_) => Value::bool(false),
    }
}

/// Clean up engine from registry
#[ocaml::func]
pub fn cleanup_simulation_engine(
    _gc: &mut Runtime,
    engine_handle: Value,
) -> Value {
    let handle = engine_handle.int_val() as u64;
    let mut registry = ENGINE_REGISTRY.lock().unwrap();
    let removed = registry.remove(&handle).is_some();
    Value::bool(removed)
}

/// Clean up environment from registry
#[ocaml::func]
pub fn cleanup_simulation_environment(
    _gc: &mut Runtime,
    env_handle: Value,
) -> Value {
    let handle = env_handle.int_val() as u64;
    let mut registry = ENVIRONMENT_REGISTRY.lock().unwrap();
    let removed = registry.remove(&handle).is_some();
    Value::bool(removed)
}

/// Get registry statistics for debugging
#[ocaml::func]
pub fn get_registry_stats(_gc: &mut Runtime) -> (i64, i64) {
    let engine_count = ENGINE_REGISTRY.lock().unwrap().len() as i64;
    let env_count = ENVIRONMENT_REGISTRY.lock().unwrap().len() as i64;
    (engine_count, env_count)
} 