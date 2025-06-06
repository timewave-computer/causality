#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/alloc.h>
#include <caml/custom.h>
#include <caml/fail.h>
#include "../../../target/debug/build/causality-ffi-458dc370eb3fca28/out/include/causality-ffi.h"

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

value ocaml_causality_test_roundtrip(value v) {
    CAMLparam1(v);
    struct CausalityValue* val = unwrap_causality_value(v);
    int result = causality_test_roundtrip(val);
    CAMLreturn(Val_bool(result));
} 