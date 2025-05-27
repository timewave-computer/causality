//! Map Helper Functions for ValueExpr
//!
//! Provides utility functions and traits for working with ValueExpr maps,
//! making it easier to extract and convert values from map structures.
//! These helpers follow a consistent error handling pattern for improved
//! robustness in type conversions.

use crate::primitive::string::Str;
use crate::expression::value::{ValueExpr, ValueConversionError};
use std::collections::BTreeMap;
use std::str::FromStr;

//-----------------------------------------------------------------------------
// Helper Traits
//-----------------------------------------------------------------------------

/// Extension trait for BTreeMap<Str, ValueExpr> to add helper methods
pub trait ValueExprMapHelpers {
    /// Get a string value as String
    fn get_string_str(&self, key: &Str) -> Result<String, ValueConversionError>;

    /// Get an optional string value as Option<String>
    fn get_optional_string_str(
        &self,
        key: &Str,
    ) -> Result<Option<String>, ValueConversionError>;

    /// Get a value as a boolean
    fn get_bool(&self, key: &Str) -> Result<bool, ValueConversionError>;

    /// Get a value expression
    fn get_value_expr(&self, key: &Str) -> Result<&ValueExpr, ValueConversionError>;

    /// Get an optional value expression
    fn get_optional_value_expr(
        &self,
        key: &Str,
    ) -> Result<Option<&ValueExpr>, ValueConversionError>;

    /// Get a value as a list
    fn get_optional_list(
        &self,
        key: &Str,
    ) -> Result<Option<&[ValueExpr]>, ValueConversionError>;

    /// Get a value as an ID of the specified type (from string)
    fn get_optional_id<T: FromStr>(
        &self,
        key: &Str,
    ) -> Result<Option<T>, ValueConversionError>
    where
        T::Err: std::fmt::Display;
}

//-----------------------------------------------------------------------------
// Implementation
//-----------------------------------------------------------------------------

impl ValueExprMapHelpers for BTreeMap<Str, ValueExpr> {
    fn get_string_str(&self, key: &Str) -> Result<String, ValueConversionError> {
        match self.get(key) {
            Some(ValueExpr::String(s)) => Ok(s.as_str().to_string()),
            Some(_) => Err(ValueConversionError::InvalidType(format!(
                "Value for key '{}' is not a string",
                key
            ))),
            None => Err(ValueConversionError::MissingField(format!(
                "Key '{}' not found",
                key
            ))),
        }
    }

    fn get_optional_string_str(
        &self,
        key: &Str,
    ) -> Result<Option<String>, ValueConversionError> {
        if let Some(value) = self.get(key) {
            match value {
                ValueExpr::String(s) => Ok(Some(s.as_str().to_string())),
                ValueExpr::Unit | ValueExpr::Nil => Ok(None),
                _ => Err(ValueConversionError::InvalidType(format!(
                    "Value for key '{}' is not a string",
                    key
                ))),
            }
        } else {
            Ok(None)
        }
    }

    fn get_bool(&self, key: &Str) -> Result<bool, ValueConversionError> {
        match self.get(key) {
            Some(ValueExpr::Bool(b)) => Ok(*b),
            Some(_) => Err(ValueConversionError::InvalidType(format!(
                "Value for key '{}' is not a boolean",
                key
            ))),
            None => Err(ValueConversionError::MissingField(format!(
                "Key '{}' not found",
                key
            ))),
        }
    }

    fn get_value_expr(&self, key: &Str) -> Result<&ValueExpr, ValueConversionError> {
        self.get(key).ok_or_else(|| {
            ValueConversionError::MissingField(format!("Key '{}' not found", key))
        })
    }

    fn get_optional_value_expr(
        &self,
        key: &Str,
    ) -> Result<Option<&ValueExpr>, ValueConversionError> {
        if let Some(value) = self.get(key) {
            match value {
                ValueExpr::Unit | ValueExpr::Nil => Ok(None),
                _ => Ok(Some(value)),
            }
        } else {
            Ok(None)
        }
    }

    fn get_optional_list(
        &self,
        key: &Str,
    ) -> Result<Option<&[ValueExpr]>, ValueConversionError> {
        if let Some(value) = self.get(key) {
            match value {
                ValueExpr::List(list) => Ok(Some(&list[..])),
                ValueExpr::Unit | ValueExpr::Nil => Ok(None),
                _ => Err(ValueConversionError::InvalidType(format!(
                    "Value for key '{}' is not a list",
                    key
                ))),
            }
        } else {
            Ok(None)
        }
    }

    fn get_optional_id<T: FromStr>(
        &self,
        key: &Str,
    ) -> Result<Option<T>, ValueConversionError>
    where
        T::Err: std::fmt::Display,
    {
        if let Some(value) = self.get(key) {
            match value {
                ValueExpr::String(s) => {
                    let str_val = s.as_str().to_string();
                    T::from_str(&str_val).map(Some).map_err(|e| {
                        ValueConversionError::InvalidValue(format!(
                            "Failed to parse '{}' as ID: {}",
                            s, e
                        ))
                    })
                }
                ValueExpr::Unit | ValueExpr::Nil => Ok(None),
                _ => Err(ValueConversionError::InvalidType(format!(
                    "Value for key '{}' is not a string ID",
                    key
                ))),
            }
        } else {
            Ok(None)
        }
    }
}
