// WebAssembly Deployment Support
//
// This module provides functionality for deploying ZK circuits to WebAssembly runtime.

#![cfg(feature = "host")]

// All imports removed as they were unused

//-----------------------------------------------------------------------------
// WASM Interface Function
//-----------------------------------------------------------------------------

/// Get witnesses for the ZK proof
///
/// This function is called by the Valence Coprocessor runtime
/// to prepare witness data for the RISC-V ZK circuit.
///
/// Follows the pattern described in valence-coprocessor-interaction.md
/// FFI-compatible result struct to return witness data safely across FFI boundaries
#[repr(C)]
pub struct WitnessResult {
    /// Pointer to the witness data buffer (owned by the guest, must be freed by host)
    pub data: *mut u8,
    /// Length of the witness data in bytes
    pub length: usize,
    /// Error code (0 = success, non-zero = specific error type)
    pub error_code: i32,
}

#[no_mangle]
pub extern "C" fn get_witnesses(
    trace_ptr: *const u8,
    trace_len: usize,
) -> WitnessResult {
    let execution_trace_bytes = unsafe {
        if trace_ptr.is_null() {
            return WitnessResult {
                data: std::ptr::null_mut(),
                length: 0,
                error_code: 1, // Error: null pointer
            };
        }
        std::slice::from_raw_parts(trace_ptr, trace_len)
    };

    let execution_trace = match from_slice::<ExecutionTrace>(execution_trace_bytes) {
        Ok(trace) => trace,
        Err(_e) => {
            return WitnessResult {
                data: std::ptr::null_mut(),
                length: 0,
                error_code: 2, // Error: deserialization failed
            };
        }
    };

    let witness_data_obj = match build_witness_from_trace(&execution_trace) {
        Ok(w) => w,
        Err(_) => {
            return WitnessResult {
                data: std::ptr::null_mut(),
                length: 0,
                error_code: 3, // Error: witness generation failed
            };
        }
    };

    if !witness_data_obj.constraints.is_empty() {
        match create_witness_context(witness_data_obj.clone()) {
            Ok(owned_zk_eval_ctx) => {
                let mut lisp_constraint_exprs: Vec<LispExpr> = Vec::new();
                for expr_id in &witness_data_obj.constraints {
                    match owned_zk_eval_ctx
                        .as_zk_eval_context()
                        .get_actual_expr(expr_id)
                    {
                        Some(lisp_expr) => {
                            lisp_constraint_exprs.push(lisp_expr.clone())
                        }
                        None => {
                            return WitnessResult {
                                data: std::ptr::null_mut(),
                                length: 0,
                                error_code: 6, // Error: constraint ExprId not found
                            };
                        }
                    }
                }

                if !lisp_constraint_exprs.is_empty() {
                    match futures::executor::block_on(validate_constraints(
                        &lisp_constraint_exprs,
                        &owned_zk_eval_ctx,
                    )) {
                        Ok(results) => {
                            if !results.iter().all(|&r| r) {
                                return WitnessResult {
                                    data: std::ptr::null_mut(),
                                    length: 0,
                                    error_code: 4, // Error: witness validation failed (constraint not met)
                                };
                            }
                        }
                        Err(_) => {
                            return WitnessResult {
                                data: std::ptr::null_mut(),
                                length: 0,
                                error_code: 7, // Error: constraint validation process error
                            };
                        }
                    }
                }
            }
            Err(_) => {
                return WitnessResult {
                    data: std::ptr::null_mut(),
                    length: 0,
                    error_code: 8, // New error code: Witness context creation failed
                };
            }
        }
    }

    let serialized = witness_data_obj.as_ssz_bytes();

    let len = serialized.len();
    let data = unsafe {
        let ptr =
            std::alloc::alloc(std::alloc::Layout::from_size_align(len, 1).unwrap())
                as *mut u8;
        if !ptr.is_null() {
            std::ptr::copy_nonoverlapping(serialized.as_ptr(), ptr, len);
        }
        ptr
    };

    WitnessResult {
        data,
        length: len,
        error_code: 0, // Success
    }
}
