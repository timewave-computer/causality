// Purpose: Crate for core business logic and implementations, separate from type definitions.

//-----------------------------------------------------------------------------
// Module declarations
//-----------------------------------------------------------------------------

pub mod content_addressing;
pub mod extension_traits;
pub mod graph_analysis;
pub mod graph_registry;
pub mod lisp_adapter;

pub mod message_utils;
pub mod sexpr_ffi;
pub mod sexpr_ffi_test;
pub mod sexpr_utils;
pub mod smt;
pub mod domain;
pub mod teg_state_root;
pub mod smt_collections;
pub mod teg_proofs;
pub mod teg_direct_write;
pub mod teg_zkp;
pub mod teg_deployment;
pub mod teg_persistence;
pub mod tracing;

// System-level testing utilities
pub mod test_logging;
pub mod test_mocks;

// Consolidated utilities module
pub mod utils;

//-----------------------------------------------------------------------------
// Re-exports for easier access to common functionality
//-----------------------------------------------------------------------------

pub use content_addressing::{
    ContentAddressable, content_id_from_bytes, content_id_hex_from_bytes
};

// Core utilities
pub use utils::{
    create_content_addressed_id, create_random_id, id_from_hex, id_starts_with, id_to_hex,
    compute_expr_hash, compute_expr_id, create_expr_from_value_expr, expr_as_value, serialize_expr,
    compute_value_expr_hash, compute_value_expr_id, create_value_expr_bool, create_value_expr_int,
    create_value_expr_list, create_value_expr_string, value_expr_as_bool, value_expr_as_int,
    value_expr_as_string, compute_type_expr_hash, compute_type_expr_id, create_type_expr_list,
    type_expr_is_primitive, compute_resource_hash, create_resource, resource_id,
    serialize_vector, deserialize_vector, to_vec, from_slice,
};

// TEL utilities  
pub use utils::tel::{
    compute_effect_hash, create_resource_from_effect, create_resource_from_handler,
    create_resource_from_intent, handler_to_value_expr, is_basic_type_value_expr,
    serialize_effect, value_expr_from_effect, value_expr_from_intent,
};

// Message utilities
pub use message_utils::{message_to_value_expr, message_try_from_value_expr};

// Graph utilities
pub use graph_analysis::tel_graph_has_cycles;
pub use graph_registry::{EdgeRegistry, NodeRegistry};

// Tracing utilities
pub use tracing::init_tracing;

// System-level testing utilities
pub use test_logging::{init_test_logging, init_debug_logging, TestLogger};
pub use test_mocks::{MockLogger, MockProvider, MockStore};

// Legacy re-exports for backward compatibility (to be phased out)
pub use utils::core::default_compute_hash as get_current_time_ms; // Placeholder - time utils needs proper consolidation

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
