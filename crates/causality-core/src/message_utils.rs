// Purpose: Utility functions for converting between Message and ValueExpr.

use causality_types::{
    primitive::{
        ids::{DomainId, MessageId, ResourceId},
        string::Str,
    },
    expression::{
        value::{ValueExpr, ValueExprMap, ValueConversionError},
    },
    system::pattern::Message,
};
use crate::utils::core::{id_from_hex, id_to_hex};
use std::collections::BTreeMap;

/// Converts a Message struct to its ValueExpr representation.
pub fn message_to_value_expr(message: &Message) -> ValueExpr {
    let mut map_data = BTreeMap::new();

    map_data.insert(
        Str::new("id"),
        ValueExpr::String(Str::new(id_to_hex(&message.id))),
    );

    map_data.insert(
        Str::new("target_domain_id"),
        ValueExpr::String(Str::new(id_to_hex(&message.target_domain_id))),
    );

    if let Some(id) = message.source_domain_id {
        map_data.insert(
            Str::new("source_domain_id"),
            ValueExpr::String(Str::new(id_to_hex(&id))),
        );
    } else {
        map_data.insert(Str::new("source_domain_id"), ValueExpr::Nil);
    }

    if let Some(id) = message.target_resource_id {
        map_data.insert(
            Str::new("target_resource_id"),
            ValueExpr::String(Str::new(id_to_hex(&id))),
        );
    } else {
        map_data.insert(Str::new("target_resource_id"), ValueExpr::Nil);
    }

    if let Some(ref content_val) = message.content {
        map_data.insert(Str::new("content"), content_val.clone());
    } else {
        map_data.insert(Str::new("content"), ValueExpr::Nil);
    }

    ValueExpr::Map(ValueExprMap(map_data))
}

/// Attempts to convert a ValueExpr to a Message struct.
/// This logic was previously in `impl AsTryFromValueExpr for Message`.
pub fn message_try_from_value_expr(
    value: &ValueExpr,
) -> Result<Message, ValueConversionError> {
    let map_data = match value {
        ValueExpr::Map(map) => &map.0,
        _ => {
            return Err(ValueConversionError::InvalidType(
                "Expected Map for Message".to_string(),
            ));
        }
    };

    let id = match map_data
        .get(&Str::new("id"))
        .ok_or_else(|| ValueConversionError::MissingField("id".to_string()))?
    {
        ValueExpr::String(s) => {
            id_from_hex::<MessageId>(&s.as_string()).map_err(|e| {
                ValueConversionError::InvalidValue(format!(
                    "Invalid MessageId hex: {}",
                    e
                ))
            })?
        }
        _ => {
            return Err(ValueConversionError::InvalidType(
                "id field must be a String".to_string(),
            ));
        }
    };

    let target_domain_id =
        match map_data.get(&Str::new("target_domain_id")).ok_or_else(|| {
            ValueConversionError::MissingField("target_domain_id".to_string())
        })? {
            ValueExpr::String(s) => id_from_hex::<DomainId>(&s.as_string())
                .map_err(|e| {
                    ValueConversionError::InvalidValue(format!(
                        "Invalid target_domain_id hex: {}",
                        e
                    ))
                })?,
            _ => {
                return Err(ValueConversionError::InvalidType(
                    "target_domain_id field must be a String".to_string(),
                ));
            }
        };

    let source_domain_id = match map_data.get(&Str::new("source_domain_id")) {
        Some(ValueExpr::String(s)) => {
            let s_str = s.as_string();
            if s_str.is_empty() || s_str == "null" {
                // Handle empty or "null" string representation
                None
            } else {
                Some(id_from_hex::<DomainId>(&s_str).map_err(|e| {
                    ValueConversionError::InvalidValue(format!(
                        "Invalid source_domain_id hex: {}",
                        e
                    ))
                })?)
            }
        }
        Some(ValueExpr::Nil) => None, // Explicitly Nil means None
        None => None,                  // Field not present means None
        Some(_) => {
            return Err(ValueConversionError::InvalidType(
                "source_domain_id field must be a String or Nil".to_string(),
            ));
        }
    };

    let target_resource_id = match map_data.get(&Str::new("target_resource_id")) {
        Some(ValueExpr::String(s)) => {
            let s_str = s.as_string();
            if s_str.is_empty() || s_str == "null" {
                // Handle empty or "null" string representation
                None
            } else {
                Some(id_from_hex::<ResourceId>(&s_str).map_err(|e| {
                    ValueConversionError::InvalidValue(format!(
                        "Invalid target_resource_id hex: {}",
                        e
                    ))
                })?)
            }
        }
        Some(ValueExpr::Nil) => None, // Explicitly Nil means None
        None => None,                  // Field not present means None
        Some(_) => {
            return Err(ValueConversionError::InvalidType(
                "target_resource_id field must be a String or Nil".to_string(),
            ));
        }
    };

    let content = match map_data.get(&Str::new("content")) {
        Some(ValueExpr::Nil) => None,
        Some(content_val) => Some(content_val.clone()),
        None => None, // If content field is missing, treat as None
    };

    Ok(Message {
        id,
        target_domain_id,
        source_domain_id,
        target_resource_id,
        content,
    })
}

// The free function `try_from_value` from message.rs used anyhow::Error.
// For consistency within causality-core, we might prefer ValueConversionError or a similar typed error.
// If its exact signature and error type are important, it can be moved and adapted.
// For now, message_try_from_value_expr covers the core conversion logic with ValueConversionError.
