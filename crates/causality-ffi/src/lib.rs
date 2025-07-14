//! Causality FFI: Minimal Interface for Simulation
//!
//! This FFI layer provides a minimal, stateful interface for external
//! applications (like OCaml) to interact with the Causality simulation runtime.
//! The core idea is to load a pre-compiled bytecode artifact once, and then
//! interact with the resulting simulation state via an opaque pointer.

#![warn(missing_docs)]

use causality_compiler::CompiledArtifact;
use causality_runtime::Executor;
use std::ffi::c_void;
use std::slice;

//-----------------------------------------------------------------------------
// Opaque State Pointer
//-----------------------------------------------------------------------------

/// An opaque struct representing the state of a simulation instance.
/// The pointer to this struct is what's passed across the FFI boundary.
pub struct SimulationState {
    executor: Executor,
}

//-----------------------------------------------------------------------------
// FFI-Safe Result Type
//-----------------------------------------------------------------------------

/// A result type for C FFI functions
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CResult {
    /// Success result
    Ok,
    /// Error result with error code
    Err(i32),
}

/// A result type for C FFI functions that return pointers
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CPointerResult {
    /// Success result with pointer
    Ok(*const c_void), // Placeholder for a result pointer
    /// Error result with error code
    Err(i32),
}

//-----------------------------------------------------------------------------
// FFI Interface Functions
//-----------------------------------------------------------------------------

/// Load a compiled bytecode artifact and initialize a simulation.
///
/// Takes a byte slice containing the bincode-serialized `CompiledArtifact`.
/// Returns a pointer to an opaque `SimulationState` struct.
/// The caller is responsible for freeing this state later using `causality_free_simulation_state`.
///
/// # Safety
/// The `bytecode_ptr` must be a valid pointer to a byte array of `bytecode_len` length.
#[no_mangle]
pub unsafe extern "C" fn causality_load_bytecode(
    bytecode_ptr: *const u8,
    bytecode_len: usize,
) -> *mut SimulationState {
    if bytecode_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let bytecode_slice = slice::from_raw_parts(bytecode_ptr, bytecode_len);

    let artifact: CompiledArtifact = match bincode::deserialize(bytecode_slice) {
        Ok(art) => art,
        Err(_) => return std::ptr::null_mut(),
    };

    let mut executor = Executor::new();
    executor.execute(&artifact.instructions).ok(); // Prime the executor

    let state = SimulationState { executor };

    Box::into_raw(Box::new(state))
}

/// Frees the memory associated with a simulation state.
///
/// # Safety
/// The `state_ptr` must be a valid pointer to a `SimulationState` that was
/// allocated by `causality_load_bytecode`. Passing a null pointer is safe.
#[no_mangle]
pub unsafe extern "C" fn causality_free_simulation_state(
    state_ptr: *mut SimulationState,
) {
    if !state_ptr.is_null() {
        drop(Box::from_raw(state_ptr));
    }
}

/// Run a single step of the simulation.
///
/// # Safety
/// The `state_ptr` must be a valid pointer to a `SimulationState`.
#[no_mangle]
pub unsafe extern "C" fn causality_run_simulation_step(
    state_ptr: *mut SimulationState,
) -> CResult {
    if let Some(state) = state_ptr.as_mut() {
        match state.executor.step() {
            Ok(_) => CResult::Ok,
            Err(_) => CResult::Err(-1), // Generic error
        }
    } else {
        CResult::Err(-2) // Null pointer error
    }
}

/// Get the current result from the simulation.
/// (Placeholder implementation)
///
/// # Safety
/// The `state_ptr` must be a valid pointer to a `SimulationState`.
#[no_mangle]
pub unsafe extern "C" fn causality_get_simulation_result(
    state_ptr: *mut SimulationState,
) -> CPointerResult {
    if let Some(state) = state_ptr.as_mut() {
        // This is a placeholder. A real implementation would serialize
        // the result from state.executor.get_result() into a returnable format.
        let _ = state.executor.get_result();
        CPointerResult::Ok(std::ptr::null())
    } else {
        CPointerResult::Err(-2) // Null pointer error
    }
}

/// FFI error type
#[derive(Debug, thiserror::Error)]
pub enum FfiError {
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),
}
