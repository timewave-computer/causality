#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/alloc.h>
#include <caml/custom.h>
#include <caml/fail.h>
#include <string.h>
#include "causality-ffi.h"

// Custom block operations for CausalityValue
static void causality_value_finalize(value v) {
    struct CausalityValue* val = (struct CausalityValue*)Data_custom_val(v);
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

// Helper to wrap a CausalityValue in an OCaml custom block
static value wrap_causality_value(struct CausalityValue* val) {
    if (val == NULL) {
        caml_failwith("Failed to create causality value");
    }
    value v = caml_alloc_custom(&causality_value_ops, sizeof(struct CausalityValue*), 0, 1);
    *((struct CausalityValue**)Data_custom_val(v)) = val;
    return v;
}

// Helper to unwrap a CausalityValue from an OCaml custom block
static struct CausalityValue* unwrap_causality_value(value v) {
    return *((struct CausalityValue**)Data_custom_val(v));
}

// OCaml FFI functions
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
    struct CausalityValue* val = unwrap_causality_value(v);
    if (val != NULL) {
        causality_value_free(val);
        *((struct CausalityValue**)Data_custom_val(v)) = NULL;
    }
    CAMLreturn(Val_unit);
}

value ocaml_causality_test_roundtrip(value v) {
    CAMLparam1(v);
    struct CausalityValue* val = unwrap_causality_value(v);
    int result = causality_test_roundtrip(val);
    CAMLreturn(Val_bool(result));
}

value ocaml_causality_value_type(value v) {
    CAMLparam1(v);
    struct CausalityValue* val = unwrap_causality_value(v);
    enum ValueType type = causality_value_type(val);
    CAMLreturn(Val_int(type));
}

value ocaml_causality_value_as_bool(value v) {
    CAMLparam1(v);
    struct CausalityValue* val = unwrap_causality_value(v);
    int result = causality_value_as_bool(val);
    CAMLreturn(Val_int(result));
}

value ocaml_causality_value_as_int(value v) {
    CAMLparam1(v);
    struct CausalityValue* val = unwrap_causality_value(v);
    unsigned int result = causality_value_as_int(val);
    CAMLreturn(Val_int(result));
}

value ocaml_causality_value_as_string(value v) {
    CAMLparam1(v);
    CAMLlocal1(result);
    struct CausalityValue* val = unwrap_causality_value(v);
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
    struct CausalityValue* val = unwrap_causality_value(v);
    struct SerializationResult ser_result = causality_value_serialize(val);
    
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
    
    struct CausalityValue* val = causality_value_deserialize(data_ptr, len);
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
    struct CausalityValue* val = unwrap_causality_value(v);
    char* debug_info = causality_value_debug_info(val);
    value result = caml_copy_string(debug_info);
    causality_free_string(debug_info);
    CAMLreturn(result);
} 