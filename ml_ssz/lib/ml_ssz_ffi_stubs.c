#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/alloc.h>
#include <caml/fail.h>
#include <caml/callback.h>
#include <caml/custom.h>
#include <string.h>
#include <stdbool.h>
#include <stdlib.h>

/**
 * Mock implementation stubs for the ml_ssz library
 * 
 * These stubs provide mock implementations of the Rust SSZ functions.
 * In a real implementation, these would call actual Rust functions.
 */

/*
 * Combined FFI stubs for ml_ssz
 * 
 * This file contains mock implementations for both:
 * 1. OCaml -> Rust calls (caml_rust_* functions)
 * 2. Rust -> OCaml calls (ocaml_mock_* functions)
 */

/******************************************************************************
 * Data structures for FFI
 *****************************************************************************/

// FfiSszBytes struct for Rust FFI
typedef struct {
    bool success;
    unsigned char* data;
    size_t data_len;
    char* error_msg;
} FfiSszBytes;

typedef struct {
    bool valid;
    char* error_msg;
} FfiValidationResult;

/******************************************************************************
 * Helper functions for FFI data conversion
 *****************************************************************************/

// Helper function to convert FfiSszBytes to OCaml record
static value to_ffi_ssz_bytes_record(FfiSszBytes result) {
    CAMLparam0();
    CAMLlocal3(record, data_option, error_msg_option);
    
    // Handle data field
    if (result.success && result.data != NULL && result.data_len > 0) {
        data_option = caml_alloc(1, 0); // Some
        Store_field(data_option, 0, caml_alloc_string(result.data_len));
        memcpy(String_val(Field(data_option, 0)), result.data, result.data_len);
    } else {
        data_option = Val_int(0); // None
    }
    
    // Handle error_msg field
    if (!result.success && result.error_msg != NULL) {
        error_msg_option = caml_alloc(1, 0); // Some
        Store_field(error_msg_option, 0, caml_copy_string(result.error_msg));
    } else {
        error_msg_option = Val_int(0); // None
    }
    
    // Create the record
    record = caml_alloc(3, 0);
    Store_field(record, 0, Val_bool(result.success));
    Store_field(record, 1, data_option);
    Store_field(record, 2, error_msg_option);
    
    CAMLreturn(record);
}

// Helper to create OCaml validation result record from FfiValidationResult
value to_ffi_validation_result_record(FfiValidationResult result) {
    CAMLparam0();
    CAMLlocal2(record, error_msg_option);
    
    // Handle error_msg field
    if (!result.valid && result.error_msg != NULL) {
        error_msg_option = caml_alloc(1, 0); // Some
        Store_field(error_msg_option, 0, caml_copy_string(result.error_msg));
    } else {
        error_msg_option = Val_int(0); // None
    }
    
    // Create the record
    record = caml_alloc(2, 0);
    Store_field(record, 0, Val_bool(result.valid));
    Store_field(record, 1, error_msg_option);
    
    CAMLreturn(record);
}

/******************************************************************************
 * Mock implementations for Rust -> OCaml calls
 *****************************************************************************/

// Mock implementation for serialization
FfiSszBytes mock_to_ssz(void* ptr) {
    FfiSszBytes result;
    result.success = true;
    result.data = (unsigned char*)"mock serialized data";
    result.data_len = strlen((char*)result.data);
    result.error_msg = NULL;
    return result;
}

// Mock implementation for deserialization
void* mock_from_ssz(const unsigned char* data, size_t data_len, char** error_msg) {
    // Return a dummy pointer for testing
    return (void*)1;
}

// Generic validation function
FfiValidationResult mock_validate_bytes(const unsigned char* data, size_t data_len) {
    FfiValidationResult result;
    result.valid = true;
    result.error_msg = NULL;
    return result;
}

// Generic free function
void mock_free(void* ptr) {
    // Nothing to do in mock implementation
}

// These functions would be called from Rust
CAMLprim value ocaml_mock_serialize(value v) {
    CAMLparam1(v);
    
    void* ptr = (void*)v;
    FfiSszBytes result = mock_to_ssz(ptr);
    value ocaml_result = to_ffi_ssz_bytes_record(result);
    
    CAMLreturn(ocaml_result);
}

CAMLprim value ocaml_mock_deserialize(value bytes, value length) {
    CAMLparam2(bytes, length);
    
    size_t len = Long_val(length);
    const unsigned char* data = (const unsigned char*)String_val(bytes);
    char* error_msg = NULL;
    
    void* ptr = mock_from_ssz(data, len, &error_msg);
    
    if (ptr == NULL) {
        if (error_msg != NULL) {
            caml_failwith(error_msg);
            free(error_msg);
        } else {
            caml_failwith("Failed to deserialize");
        }
    }
    
    CAMLreturn((value)ptr);
}

CAMLprim value ocaml_mock_validate(value bytes, value length) {
    CAMLparam2(bytes, length);
    
    size_t len = Long_val(length);
    const unsigned char* data = (const unsigned char*)String_val(bytes);
    
    FfiValidationResult result = mock_validate_bytes(data, len);
    value ocaml_result = to_ffi_validation_result_record(result);
    
    CAMLreturn(ocaml_result);
}

CAMLprim value ocaml_mock_free(value v) {
    CAMLparam1(v);
    
    void* ptr = (void*)v;
    mock_free(ptr);
    
    CAMLreturn(Val_unit);
}

/******************************************************************************
 * OCaml -> Rust function call stubs (mock implementations)
 *****************************************************************************/

/* Boolean serialization */
CAMLprim value caml_rust_serialize_bool(value v) {
    CAMLparam1(v);
    CAMLlocal1(result);
    
    int b = Bool_val(v);
    result = caml_alloc_string(1);
    char* data = String_val(result);
    data[0] = b ? 1 : 0;
    
    CAMLreturn(result);
}

/* Boolean deserialization */
CAMLprim value caml_rust_deserialize_bool(value v) {
    CAMLparam1(v);
    
    const char* data = String_val(v);
    int len = caml_string_length(v);
    int result = (len > 0 && data[0] != 0);
    
    CAMLreturn(Val_bool(result));
}

/* uint32 serialization */
CAMLprim value caml_rust_serialize_u32(value v) {
    CAMLparam1(v);
    CAMLlocal1(result);
    
    int n = Int_val(v);
    result = caml_alloc_string(4);
    unsigned char* data = (unsigned char*)Bytes_val(result);
    
    data[0] = n & 0xFF;
    data[1] = (n >> 8) & 0xFF;
    data[2] = (n >> 16) & 0xFF;
    data[3] = (n >> 24) & 0xFF;
    
    CAMLreturn(result);
}

/* uint32 deserialization */
CAMLprim value caml_rust_deserialize_u32(value v) {
    CAMLparam1(v);
    
    const unsigned char* data = (const unsigned char*)String_val(v);
    int len = caml_string_length(v);
    
    if (len < 4) {
        CAMLreturn(Val_int(0));
    }
    
    int result = data[0] | (data[1] << 8) | (data[2] << 16) | (data[3] << 24);
    
    CAMLreturn(Val_int(result));
}

/* String serialization */
CAMLprim value caml_rust_serialize_string(value v) {
    CAMLparam1(v);
    CAMLlocal1(result);
    
    const char* str = String_val(v);
    int str_len = caml_string_length(v);
    
    result = caml_alloc_string(4 + str_len);
    unsigned char* data = (unsigned char*)Bytes_val(result);
    
    /* Write the length as little-endian uint32 */
    data[0] = str_len & 0xFF;
    data[1] = (str_len >> 8) & 0xFF;
    data[2] = (str_len >> 16) & 0xFF;
    data[3] = (str_len >> 24) & 0xFF;
    
    /* Copy the string content */
    memcpy(data + 4, str, str_len);
    
    CAMLreturn(result);
}

/* String deserialization */
CAMLprim value caml_rust_deserialize_string(value v) {
    CAMLparam1(v);
    CAMLlocal1(result);
    
    const unsigned char* data = (const unsigned char*)String_val(v);
    int len = caml_string_length(v);
    
    if (len < 4) {
        CAMLreturn(caml_copy_string(""));
    }
    
    int str_len = data[0] | (data[1] << 8) | (data[2] << 16) | (data[3] << 24);
    
    if (len < 4 + str_len) {
        CAMLreturn(caml_copy_string(""));
    }
    
    result = caml_alloc_string(str_len);
    memcpy(Bytes_val(result), data + 4, str_len);
    
    CAMLreturn(result);
}

/* Simple hash function */
CAMLprim value caml_rust_simple_hash(value v) {
    CAMLparam1(v);
    CAMLlocal1(result);
    
    const char* data = String_val(v);
    int len = caml_string_length(v);
    
    /* Very simple hash algorithm - not cryptographically secure */
    unsigned int hash = 0;
    for (int i = 0; i < len; i++) {
        hash = (hash * 31 + data[i]) & 0xFFFFFFFF;
    }
    
    /* Create a 32-byte result filled with the hash value */
    result = caml_alloc_string(32);
    unsigned char* hash_bytes = (unsigned char*)Bytes_val(result);
    
    for (int i = 0; i < 8; i++) {
        unsigned char value = (hash >> (i * 4)) & 0xF;
        for (int j = 0; j < 4; j++) {
            hash_bytes[i * 4 + j] = value;
        }
    }
    
    CAMLreturn(result);
}

/* Roundtrip functions */
CAMLprim value caml_rust_roundtrip_bool(value v) {
    CAMLparam1(v);
    value serialized = caml_rust_serialize_bool(v);
    value result = caml_rust_deserialize_bool(serialized);
    CAMLreturn(result);
}

CAMLprim value caml_rust_roundtrip_u32(value v) {
    CAMLparam1(v);
    value serialized = caml_rust_serialize_u32(v);
    value result = caml_rust_deserialize_u32(serialized);
    CAMLreturn(result);
}

CAMLprim value caml_rust_roundtrip_string(value v) {
    CAMLparam1(v);
    value serialized = caml_rust_serialize_string(v);
    value result = caml_rust_deserialize_string(serialized);
    CAMLreturn(result);
} 