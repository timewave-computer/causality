/* FFI stub implementations for Causality OCaml bindings */
/* These are simple mock implementations to allow tests to run */

#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/alloc.h>
#include <caml/custom.h>
#include <string.h>
#include <stdio.h>

/* Effect FFI functions */
value effect_pure(value v_value) {
    CAMLparam1(v_value);
    CAMLlocal1(result);
    
    /* Create a dummy 8-byte result */
    result = caml_alloc_string(8);
    memset(String_val(result), 0x01, 8);
    
    CAMLreturn(result);
}

value effect_bind(value v_effect1, value v_var_name, value v_effect2) {
    CAMLparam3(v_effect1, v_var_name, v_effect2);
    CAMLlocal1(result);
    
    /* Create a dummy 8-byte result */
    result = caml_alloc_string(8);
    memset(String_val(result), 0x02, 8);
    
    CAMLreturn(result);
}

value effect_perform(value v_tag, value v_args) {
    CAMLparam2(v_tag, v_args);
    CAMLlocal1(result);
    
    /* Create a dummy 8-byte result */
    result = caml_alloc_string(8);
    memset(String_val(result), 0x03, 8);
    
    CAMLreturn(result);
}

value effect_compile(value v_effect_id) {
    CAMLparam1(v_effect_id);
    CAMLlocal1(result);
    
    /* Create a dummy 8-byte result */
    result = caml_alloc_string(8);
    memset(String_val(result), 0x04, 8);
    
    CAMLreturn(result);
}

/* Intent FFI functions */
value intent_create(value v_name, value v_domain_id) {
    CAMLparam2(v_name, v_domain_id);
    CAMLlocal1(result);
    
    /* Create a dummy 32-byte result */
    result = caml_alloc_string(32);
    memset(String_val(result), 0x10, 32);
    
    CAMLreturn(result);
}

value intent_add_constraint(value v_intent_id, value v_constraint_type) {
    CAMLparam2(v_intent_id, v_constraint_type);
    
    /* Always return true (Val_true) */
    CAMLreturn(Val_true);
}

value intent_add_capability(value v_intent_id, value v_capability_name) {
    CAMLparam2(v_intent_id, v_capability_name);
    
    /* Always return true (Val_true) */
    CAMLreturn(Val_true);
}

value intent_compile(value v_intent_id) {
    CAMLparam1(v_intent_id);
    CAMLlocal1(result);
    
    /* Create a dummy 32-byte result */
    result = caml_alloc_string(32);
    memset(String_val(result), 0x20, 32);
    
    CAMLreturn(result);
}
