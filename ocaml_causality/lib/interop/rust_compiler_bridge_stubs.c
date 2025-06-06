/* C stubs for OCaml to Rust FFI bridge */

#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/alloc.h>
#include <caml/fail.h>
#include <caml/callback.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

/* Forward declarations of Rust functions */
extern const char* rust_compiler_version(void);
extern const char* rust_test_compilation(const char* source);
extern void rust_free_string(char* s);

/* Simplified C compilation result - we'll implement a simple version first */
typedef struct {
    int success;
    char** instructions;
    int instruction_count;
    int registers_used;
    int resource_allocations;
    int resource_consumptions;
    char* error_message;
} simple_compilation_result;

/* Simplified compilation function for testing */
simple_compilation_result* simple_compile_term(int term_type, const char* term_data) {
    simple_compilation_result* result = malloc(sizeof(simple_compilation_result));
    
    /* For now, return a simple success result */
    result->success = 1;
    result->instruction_count = 2;
    result->instructions = malloc(2 * sizeof(char*));
    result->instructions[0] = strdup("LoadImmediate { value: 42, dst: RegisterId(0) }");
    result->instructions[1] = strdup("Alloc { src: RegisterId(0), dst: RegisterId(1) }");
    result->registers_used = 2;
    result->resource_allocations = 1;
    result->resource_consumptions = 0;
    result->error_message = NULL;
    
    return result;
}

void free_compilation_result(simple_compilation_result* result) {
    if (result) {
        if (result->instructions) {
            for (int i = 0; i < result->instruction_count; i++) {
                free(result->instructions[i]);
            }
            free(result->instructions);
        }
        if (result->error_message) {
            free(result->error_message);
        }
        free(result);
    }
}

/* OCaml stub functions */

/* rust_compiler_version : unit -> string */
value rust_compiler_version_stub(value unit) {
    CAMLparam1(unit);
    CAMLlocal1(result);
    
    const char* version = rust_compiler_version();
    result = caml_copy_string(version);
    
    CAMLreturn(result);
}

/* rust_test_compilation : string -> string */
value rust_test_compilation_stub(value source) {
    CAMLparam1(source);
    CAMLlocal1(result);
    
    const char* source_str = String_val(source);
    const char* test_result = rust_test_compilation(source_str);
    result = caml_copy_string(test_result);
    
    CAMLreturn(result);
}

/* rust_free_string : string -> unit */
value rust_free_string_stub(value str) {
    CAMLparam1(str);
    
    /* Note: We can't directly free OCaml strings, this is just for completeness */
    /* In practice, the Rust side manages its own string memory */
    
    CAMLreturn(Val_unit);
}

/* rust_compile_lambda_term_stub : int -> string -> c_compilation_result */
value rust_compile_lambda_term_stub(value term_type, value term_data) {
    CAMLparam2(term_type, term_data);
    CAMLlocal5(result, instructions_array, success_val, error_val, record);
    
    int type_int = Int_val(term_type);
    const char* data_str = String_val(term_data);
    
    /* Call simplified compilation function */
    simple_compilation_result* c_result = simple_compile_term(type_int, data_str);
    
    /* Convert C result to OCaml record */
    /* Create instructions array */
    instructions_array = caml_alloc(c_result->instruction_count, 0);
    for (int i = 0; i < c_result->instruction_count; i++) {
        Store_field(instructions_array, i, caml_copy_string(c_result->instructions[i]));
    }
    
    /* Create error option */
    if (c_result->error_message) {
        error_val = caml_alloc(1, 0); /* Some */
        Store_field(error_val, 0, caml_copy_string(c_result->error_message));
    } else {
        error_val = Val_int(0); /* None */
    }
    
    /* Create record with 7 fields */
    record = caml_alloc(7, 0);
    Store_field(record, 0, Val_int(c_result->success));                    /* success */
    Store_field(record, 1, instructions_array);                            /* instructions */
    Store_field(record, 2, Val_int(c_result->instruction_count));          /* instruction_count */
    Store_field(record, 3, Val_int(c_result->registers_used));             /* registers_used */
    Store_field(record, 4, Val_int(c_result->resource_allocations));       /* resource_allocations */
    Store_field(record, 5, Val_int(c_result->resource_consumptions));      /* resource_consumptions */
    Store_field(record, 6, error_val);                                     /* error_message */
    
    /* Clean up C result */
    free_compilation_result(c_result);
    
    CAMLreturn(record);
} 