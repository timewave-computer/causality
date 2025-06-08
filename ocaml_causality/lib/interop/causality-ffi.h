#ifndef CAUSALITY_FFI_H
#define CAUSALITY_FFI_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

//-----------------------------------------------------------------------------
// Core Types
//-----------------------------------------------------------------------------

/// Opaque handle to a Causality value
typedef struct CausalityValue CausalityValue;

/// Value type enumeration
typedef enum {
    VALUE_TYPE_UNIT = 0,
    VALUE_TYPE_BOOL = 1,
    VALUE_TYPE_INT = 2,
    VALUE_TYPE_SYMBOL = 3,
    VALUE_TYPE_STRING = 4,
    VALUE_TYPE_PRODUCT = 5,
    VALUE_TYPE_SUM = 6,
    VALUE_TYPE_RECORD = 7,
} ValueType;

/// FFI error codes
typedef enum {
    FFI_SUCCESS = 0,
    FFI_INVALID_INPUT = 1,
    FFI_SERIALIZATION_ERROR = 2,
    FFI_DESERIALIZATION_ERROR = 3,
    FFI_MEMORY_ERROR = 4,
    FFI_INTERNAL_ERROR = 5,
} FfiErrorCode;

/// Serialization result
typedef struct {
    uint8_t* data;
    uintptr_t length;
    FfiErrorCode error_code;
    char* error_message;
} SerializationResult;

//-----------------------------------------------------------------------------
// Value Creation and Management
//-----------------------------------------------------------------------------

/// Create a unit value
CausalityValue* causality_value_unit(void);

/// Create a boolean value
CausalityValue* causality_value_bool(int b);

/// Create an integer value
CausalityValue* causality_value_int(uint32_t i);

/// Create a string value
CausalityValue* causality_value_string(const char* s);

/// Create a symbol value
CausalityValue* causality_value_symbol(const char* s);

/// Free a Causality value
void causality_value_free(CausalityValue* value);

//-----------------------------------------------------------------------------
// Value Inspection
//-----------------------------------------------------------------------------

/// Get the type of a value
ValueType causality_value_type(const CausalityValue* value);

/// Extract boolean value (-1 if not bool, 0 for false, 1 for true)
int causality_value_as_bool(const CausalityValue* value);

/// Extract integer value
uint32_t causality_value_as_int(const CausalityValue* value);

/// Extract string value (caller must free with causality_free_string)
char* causality_value_as_string(const CausalityValue* value);

/// Free a string returned by the library
void causality_free_string(char* s);

//-----------------------------------------------------------------------------
// Serialization
//-----------------------------------------------------------------------------

/// Serialize a value to SSZ bytes
SerializationResult causality_value_serialize(const CausalityValue* value);

/// Deserialize SSZ bytes to a value
CausalityValue* causality_value_deserialize(const uint8_t* data, uintptr_t length);

/// Free serialized data
void causality_free_serialized_data(uint8_t* data, uintptr_t length);

/// Free error message
void causality_free_error_message(char* message);

//-----------------------------------------------------------------------------
// Testing and Diagnostics
//-----------------------------------------------------------------------------

/// Test round-trip serialization/deserialization for a value
int causality_test_roundtrip(const CausalityValue* value);

/// Test round-trip for all basic value types
int causality_test_all_roundtrips(void);

/// Get FFI version information (caller must free with causality_free_string)
char* causality_ffi_version(void);

/// Get debug information about a value (caller must free with causality_free_string)
char* causality_value_debug_info(const CausalityValue* value);

//-----------------------------------------------------------------------------
// Resource Management Extensions
//-----------------------------------------------------------------------------

/// Resource handle
typedef struct CausalityResource CausalityResource;

/// Create a resource
CausalityResource* causality_create_resource(const char* resource_type, const uint8_t* domain_id, uint64_t quantity);

/// Consume a resource (returns 1 on success, 0 on failure)
int causality_consume_resource(CausalityResource* resource);

/// Check if a resource is valid
int causality_is_resource_valid(const CausalityResource* resource);

/// Free a resource
void causality_resource_free(CausalityResource* resource);

/// Get resource ID as bytes (32 bytes, caller must not free)
const uint8_t* causality_resource_id(const CausalityResource* resource);

//-----------------------------------------------------------------------------
// Expression Management Extensions
//-----------------------------------------------------------------------------

/// Expression handle
typedef struct CausalityExpr CausalityExpr;

/// Compile an expression from string
CausalityExpr* causality_compile_expr(const char* expr_string);

/// Get expression ID as bytes (32 bytes, caller must not free)
const uint8_t* causality_expr_id(const CausalityExpr* expr);

/// Free an expression
void causality_expr_free(CausalityExpr* expr);

/// Submit an intent
int causality_submit_intent(const char* name, const uint8_t* domain_id, const char* expr_string);

/// Get system metrics (caller must free with causality_free_string)  
char* causality_get_system_metrics(void);

#ifdef __cplusplus
}
#endif

#endif // CAUSALITY_FFI_H
