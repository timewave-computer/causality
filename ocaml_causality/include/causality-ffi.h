#ifndef CAUSALITY-FFI_H
#define CAUSALITY-FFI_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * FFI error codes for external bindings
 */
typedef enum FfiErrorCode {
  /**
   * Operation succeeded
   */
  Success = 0,
  /**
   * Invalid input parameter
   */
  InvalidInput = 1,
  /**
   * Serialization failed
   */
  SerializationError = 2,
  /**
   * Deserialization failed
   */
  DeserializationError = 3,
  /**
   * Memory allocation/deallocation error
   */
  MemoryError = 4,
  /**
   * Internal system error
   */
  InternalError = 5,
} FfiErrorCode;

/**
 * Value type enumeration for C interface
 */
typedef enum ValueType {
  Unit = 0,
  Bool = 1,
  Int = 2,
  Symbol = 3,
  String = 4,
  Product = 5,
  Sum = 6,
  Record = 7,
} ValueType;

/**
 * Opaque handle to a Causality Value
 */
typedef struct CausalityValue {
  uint8_t _private[0];
} CausalityValue;

/**
 * Serialization result
 */
typedef struct SerializationResult {
  uint8_t *data;
  uintptr_t length;
  enum FfiErrorCode error_code;
  char *error_message;
} SerializationResult;

/**
 * Create a unit value
 */
struct CausalityValue *causality_value_unit(void);

/**
 * Create a boolean value
 */
struct CausalityValue *causality_value_bool(int b);

/**
 * Create an integer value
 */
struct CausalityValue *causality_value_int(unsigned int i);

/**
 * Create a string value
 */
struct CausalityValue *causality_value_string(const char *s);

/**
 * Create a symbol value
 */
struct CausalityValue *causality_value_symbol(const char *s);

/**
 * Free a Causality value
 */
void causality_value_free(struct CausalityValue *value);

/**
 * Serialize a Causality value to SSZ bytes
 */
struct SerializationResult causality_value_serialize(const struct CausalityValue *value);

/**
 * Deserialize SSZ bytes to a Causality value
 */
struct CausalityValue *causality_value_deserialize(const uint8_t *data, uintptr_t length);

/**
 * Free serialized data
 */
void causality_free_serialized_data(uint8_t *data, uintptr_t length);

/**
 * Free error message
 */
void causality_free_error_message(char *message);

/**
 * Get the type of a Causality value
 */
enum ValueType causality_value_type(const struct CausalityValue *value);

/**
 * Extract boolean value (returns 0 for false, 1 for true, -1 for error)
 */
int causality_value_as_bool(const struct CausalityValue *value);

/**
 * Extract integer value (returns 0 for error cases)
 */
unsigned int causality_value_as_int(const struct CausalityValue *value);

/**
 * Extract string value (caller must free with causality_free_string)
 */
char *causality_value_as_string(const struct CausalityValue *value);

/**
 * Free a string returned by causality_value_as_string
 */
void causality_free_string(char *s);

/**
 * Test round-trip serialization for a value (returns 1 for success, 0 for failure)
 */
int causality_test_roundtrip(const struct CausalityValue *value);

/**
 * Test all round-trip serializations for basic types
 */
int causality_test_all_roundtrips(void);

/**
 * Get FFI version string (caller must free with causality_free_string)
 */
char *causality_ffi_version(void);

/**
 * Get debug information for a value (caller must free with causality_free_string)
 */
char *causality_value_debug_info(const struct CausalityValue *value);

#endif /* CAUSALITY-FFI_H */
