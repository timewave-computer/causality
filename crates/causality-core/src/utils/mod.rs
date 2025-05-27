// Purpose: Unified utilities module organizing core functionality into logical groupings.

pub mod core;
pub mod expr;
pub mod serialization;
pub mod tel;

// Re-export commonly used utilities for convenience
pub use self::{
    core::{
        create_content_addressed_id, create_random_id, default_compute_hash, id_from_hex,
        id_starts_with, id_to_hex, is_valid_hash, merge_hashes, namespaced_hash,
    },
    expr::{
        compute_expr_hash, compute_expr_id, compute_type_expr_hash, compute_type_expr_id,
        compute_value_expr_hash, compute_value_expr_id, create_expr_from_value_expr,
        create_type_expr_list, create_value_expr_bool, create_value_expr_int,
        create_value_expr_list, create_value_expr_string, expr_as_value, serialize_expr,
        type_expr_is_primitive, value_expr_as_bool, value_expr_as_int, value_expr_as_string,
        value_expr_is_unit,
    },
    serialization::{
        deserialize_map, deserialize_vector, from_hex_string, from_slice, serialize_map,
        serialize_vector, size_of, to_hex_string, to_vec, SmtCollection,
    },
    tel::{
        compute_effect_hash, compute_resource_hash, create_resource, create_resource_from_effect,
        create_resource_from_handler, create_resource_from_intent, handler_to_value_expr,
        is_basic_type_value_expr, resource_id, serialize_effect, value_expr_from_effect,
        value_expr_from_intent, ResourceExt, ToBytes,
    },
}; 