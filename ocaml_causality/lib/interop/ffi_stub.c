#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/alloc.h>
#include <caml/custom.h>
#include <caml/fail.h>
#include <string.h>
#include "causality-ffi.h"

// Custom block operations for CausalityValue
static void causality_value_finalize(value v) {
    CausalityValue* val = (CausalityValue*)Data_custom_val(v);
    if (val != NULL) {
        causality_value_free(val);
    }
}

static struct custom_operations causality_value_ops = {
    "causality_value",
    causality_value_finalize,
    custom_compare_default,
    custom_hash_default,
    custom_serialize_default,
    custom_deserialize_default,
    custom_compare_ext_default,
    custom_fixed_length_default
};

// Custom block operations for CausalityResource
static void causality_resource_finalize(value v) {
    CausalityResource* res = (CausalityResource*)Data_custom_val(v);
    if (res != NULL) {
        causality_resource_free(res);
    }
}

static struct custom_operations causality_resource_ops = {
    "causality_resource",
    causality_resource_finalize,
    custom_compare_default,
    custom_hash_default,
    custom_serialize_default,
    custom_deserialize_default,
    custom_compare_ext_default,
    custom_fixed_length_default
};

// Custom block operations for CausalityExpr
static void causality_expr_finalize(value v) {
    CausalityExpr* expr = (CausalityExpr*)Data_custom_val(v);
    if (expr != NULL) {
        causality_expr_free(expr);
    }
}

static struct custom_operations causality_expr_ops = {
    "causality_expr",
    causality_expr_finalize,
    custom_compare_default,
    custom_hash_default,
    custom_serialize_default,
    custom_deserialize_default,
    custom_compare_ext_default,
    custom_fixed_length_default
};

// Helper to wrap a CausalityValue in an OCaml custom block
static value wrap_causality_value(CausalityValue* val) {
    if (val == NULL) {
        caml_failwith("Failed to create causality value");
    }
    value v = caml_alloc_custom(&causality_value_ops, sizeof(CausalityValue*), 0, 1);
    *((CausalityValue**)Data_custom_val(v)) = val;
    return v;
}

// Helper to unwrap a CausalityValue from an OCaml custom block
static CausalityValue* unwrap_causality_value(value v) {
    return *((CausalityValue**)Data_custom_val(v));
}

// Helper to wrap a CausalityResource in an OCaml custom block
static value wrap_causality_resource(CausalityResource* res) {
    if (res == NULL) {
        caml_failwith("Failed to create causality resource");
    }
    value v = caml_alloc_custom(&causality_resource_ops, sizeof(CausalityResource*), 0, 1);
    *((CausalityResource**)Data_custom_val(v)) = res;
    return v;
}

// Helper to unwrap a CausalityResource from an OCaml custom block
static CausalityResource* unwrap_causality_resource(value v) {
    return *((CausalityResource**)Data_custom_val(v));
}

// Helper to wrap a CausalityExpr in an OCaml custom block
static value wrap_causality_expr(CausalityExpr* expr) {
    if (expr == NULL) {
        caml_failwith("Failed to create causality expression");
    }
    value v = caml_alloc_custom(&causality_expr_ops, sizeof(CausalityExpr*), 0, 1);
    *((CausalityExpr**)Data_custom_val(v)) = expr;
    return v;
}

// Helper to unwrap a CausalityExpr from an OCaml custom block
static CausalityExpr* unwrap_causality_expr(value v) {
    return *((CausalityExpr**)Data_custom_val(v));
}

//-----------------------------------------------------------------------------
// Value FFI functions
//-----------------------------------------------------------------------------

value ocaml_causality_value_unit(value unit) {
    CAMLparam1(unit);
    CAMLreturn(wrap_causality_value(causality_value_unit()));
}

value ocaml_causality_value_bool(value b) {
    CAMLparam1(b);
    CAMLreturn(wrap_causality_value(causality_value_bool(Bool_val(b))));
}

value ocaml_causality_value_int(value i) {
    CAMLparam1(i);
    CAMLreturn(wrap_causality_value(causality_value_int(Int_val(i))));
}

value ocaml_causality_value_string(value s) {
    CAMLparam1(s);
    CAMLreturn(wrap_causality_value(causality_value_string(String_val(s))));
}

value ocaml_causality_value_symbol(value s) {
    CAMLparam1(s);
    CAMLreturn(wrap_causality_value(causality_value_symbol(String_val(s))));
}

value ocaml_causality_value_free(value v) {
    CAMLparam1(v);
    // The finalizer will handle freeing, but we can also free explicitly
    CausalityValue* val = unwrap_causality_value(v);
    if (val != NULL) {
        causality_value_free(val);
        *((CausalityValue**)Data_custom_val(v)) = NULL;
    }
    CAMLreturn(Val_unit);
}

value ocaml_causality_test_roundtrip(value v) {
    CAMLparam1(v);
    CausalityValue* val = unwrap_causality_value(v);
    int result = causality_test_roundtrip(val);
    CAMLreturn(Val_bool(result));
}

value ocaml_causality_value_type(value v) {
    CAMLparam1(v);
    CausalityValue* val = unwrap_causality_value(v);
    ValueType type = causality_value_type(val);
    CAMLreturn(Val_int(type));
}

value ocaml_causality_value_as_bool(value v) {
    CAMLparam1(v);
    CausalityValue* val = unwrap_causality_value(v);
    int result = causality_value_as_bool(val);
    CAMLreturn(Val_int(result));
}

value ocaml_causality_value_as_int(value v) {
    CAMLparam1(v);
    CausalityValue* val = unwrap_causality_value(v);
    uint32_t result = causality_value_as_int(val);
    CAMLreturn(Val_int(result));
}

value ocaml_causality_value_as_string(value v) {
    CAMLparam1(v);
    CAMLlocal1(result);
    CausalityValue* val = unwrap_causality_value(v);
    char* str = causality_value_as_string(val);
    if (str != NULL) {
        result = caml_alloc(1, 0); // Some
        Store_field(result, 0, caml_copy_string(str));
        causality_free_string(str);
    } else {
        result = Val_int(0); // None
    }
    CAMLreturn(result);
}

value ocaml_causality_value_serialize(value v) {
    CAMLparam1(v);
    CAMLlocal4(result, data_val, error_msg_val, tuple);
    CausalityValue* val = unwrap_causality_value(v);
    SerializationResult ser_result = causality_value_serialize(val);
    
    // Create bytes from data
    if (ser_result.data != NULL && ser_result.length > 0) {
        data_val = caml_alloc_string(ser_result.length);
        memcpy(Bytes_val(data_val), ser_result.data, ser_result.length);
    } else {
        data_val = caml_alloc_string(0);
    }
    
    // Create error message option
    if (ser_result.error_message != NULL) {
        error_msg_val = caml_alloc(1, 0); // Some
        Store_field(error_msg_val, 0, caml_copy_string(ser_result.error_message));
        causality_free_error_message(ser_result.error_message);
    } else {
        error_msg_val = Val_int(0); // None
    }
    
    // Create tuple (data, length, error_code, error_message)
    tuple = caml_alloc_tuple(4);
    Store_field(tuple, 0, data_val);
    Store_field(tuple, 1, Val_int(ser_result.length));
    Store_field(tuple, 2, Val_int(ser_result.error_code));
    Store_field(tuple, 3, error_msg_val);
    
    // Free the serialized data
    if (ser_result.data != NULL) {
        causality_free_serialized_data(ser_result.data, ser_result.length);
    }
    
    CAMLreturn(tuple);
}

value ocaml_causality_value_deserialize(value data, value length) {
    CAMLparam2(data, length);
    CAMLlocal1(result);
    
    uint8_t* data_ptr = (uint8_t*)Bytes_val(data);
    uintptr_t len = Int_val(length);
    
    CausalityValue* val = causality_value_deserialize(data_ptr, len);
    if (val != NULL) {
        result = caml_alloc(1, 0); // Some
        Store_field(result, 0, wrap_causality_value(val));
    } else {
        result = Val_int(0); // None
    }
    
    CAMLreturn(result);
}

value ocaml_causality_free_serialized_data(value data, value length) {
    CAMLparam2(data, length);
    // In our case, the data is managed by OCaml, so we don't need to free it here
    // The Rust side already freed its copy in the serialize function
    CAMLreturn(Val_unit);
}

value ocaml_causality_test_all_roundtrips(value unit) {
    CAMLparam1(unit);
    int result = causality_test_all_roundtrips();
    CAMLreturn(Val_bool(result));
}

value ocaml_causality_ffi_version(value unit) {
    CAMLparam1(unit);
    char* version = causality_ffi_version();
    value result = caml_copy_string(version);
    causality_free_string(version);
    CAMLreturn(result);
}

value ocaml_causality_value_debug_info(value v) {
    CAMLparam1(v);
    CausalityValue* val = unwrap_causality_value(v);
    char* debug_str = causality_value_debug_info(val);
    value result = caml_copy_string(debug_str);
    causality_free_string(debug_str);
    CAMLreturn(result);
}

//-----------------------------------------------------------------------------
// Resource Management FFI functions
//-----------------------------------------------------------------------------

value ocaml_causality_create_resource(value resource_type, value domain_id, value quantity) {
    CAMLparam3(resource_type, domain_id, quantity);
    
    // Extract domain_id bytes (expecting 32 bytes)
    if (caml_string_length(domain_id) != 32) {
        caml_failwith("Domain ID must be exactly 32 bytes");
    }
    
    uint8_t* domain_bytes = (uint8_t*)Bytes_val(domain_id);
    uint64_t qty = Int64_val(quantity);
    
    CausalityResource* resource = causality_create_resource(
        String_val(resource_type),
        domain_bytes,
        qty
    );
    
    if (resource == NULL) {
        caml_failwith("Failed to create resource");
    }
    
    CAMLreturn(wrap_causality_resource(resource));
}

value ocaml_causality_consume_resource(value resource) {
    CAMLparam1(resource);
    CausalityResource* res = unwrap_causality_resource(resource);
    int result = causality_consume_resource(res);
    CAMLreturn(Val_bool(result));
}

value ocaml_causality_is_resource_valid(value resource) {
    CAMLparam1(resource);
    CausalityResource* res = unwrap_causality_resource(resource);
    int result = causality_is_resource_valid(res);
    CAMLreturn(Val_bool(result));
}

value ocaml_causality_resource_id(value resource) {
    CAMLparam1(resource);
    CAMLlocal1(result);
    CausalityResource* res = unwrap_causality_resource(resource);
    const uint8_t* id_bytes = causality_resource_id(res);
    
    if (id_bytes != NULL) {
        result = caml_alloc_string(32); // 32 byte resource ID
        memcpy(Bytes_val(result), id_bytes, 32);
    } else {
        caml_failwith("Failed to get resource ID");
    }
    
    CAMLreturn(result);
}

//-----------------------------------------------------------------------------
// Expression Management FFI functions
//-----------------------------------------------------------------------------

value ocaml_causality_compile_expr(value expr_string) {
    CAMLparam1(expr_string);
    
    CausalityExpr* expr = causality_compile_expr(String_val(expr_string));
    
    if (expr == NULL) {
        caml_failwith("Failed to compile expression");
    }
    
    CAMLreturn(wrap_causality_expr(expr));
}

value ocaml_causality_expr_id(value expr) {
    CAMLparam1(expr);
    CAMLlocal1(result);
    CausalityExpr* ex = unwrap_causality_expr(expr);
    const uint8_t* id_bytes = causality_expr_id(ex);
    
    if (id_bytes != NULL) {
        result = caml_alloc_string(32); // 32 byte expression ID
        memcpy(Bytes_val(result), id_bytes, 32);
    } else {
        caml_failwith("Failed to get expression ID");
    }
    
    CAMLreturn(result);
}

value ocaml_causality_submit_intent(value name, value domain_id, value expr_string) {
    CAMLparam3(name, domain_id, expr_string);
    
    // Extract domain_id bytes (expecting 32 bytes)
    if (caml_string_length(domain_id) != 32) {
        caml_failwith("Domain ID must be exactly 32 bytes");
    }
    
    uint8_t* domain_bytes = (uint8_t*)Bytes_val(domain_id);
    
    int result = causality_submit_intent(
        String_val(name),
        domain_bytes,
        String_val(expr_string)
    );
    
    CAMLreturn(Val_bool(result));
}

value ocaml_causality_get_system_metrics(value unit) {
    CAMLparam1(unit);
    char* metrics = causality_get_system_metrics();
    value result = caml_copy_string(metrics);
    causality_free_string(metrics);
    CAMLreturn(result);
} 