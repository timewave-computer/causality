//! OCaml Adapter Functions
//!
//! This module provides adapter functions for converting between Rust types
//! and OCaml-compatible byte representations using SSZ serialization.

use anyhow::{anyhow, Result};
use causality_types::{
    expr::value::ValueExpr,
    resource::Resource,
    tel::{
        Handler, Effect, Intent, Edge, EdgeKind, EffectGraph,
    },
    serialization::{
        Encode, Decode, SimpleSerialize,
        serialize_for_ffi, deserialize_from_ffi,
        serialize_to_hex, deserialize_from_hex
    }
};

//-----------------------------------------------------------------------------
// Core Type Adapters
//-----------------------------------------------------------------------------

/// Serializes a Rust value to a byte array for OCaml
pub fn serialize_to_ocaml<T: Encode + SimpleSerialize>(value: &T) -> Vec<u8> {
    serialize_for_ffi(value)
}

/// Deserializes a value from OCaml
pub fn deserialize_from_ocaml<T: Decode + SimpleSerialize>(bytes: &[u8]) -> Result<T> {
    deserialize_from_ffi(bytes).map_err(|e| anyhow!("Failed to deserialize from OCaml: {}", e))
}

/// Convert a ValueExpr to OCaml bytes
pub fn value_expr_to_ocaml(value: &ValueExpr) -> Vec<u8> {
    value.as_ssz_bytes()
}

/// Convert OCaml bytes to a ValueExpr
pub fn value_expr_from_ocaml(bytes: &[u8]) -> Result<ValueExpr> {
    ValueExpr::from_ssz_bytes(bytes).map_err(|e| anyhow::anyhow!("SSZ decode error: {}", e.message))
}

/// Convert a Resource to OCaml bytes
pub fn resource_to_ocaml(resource: &Resource) -> Vec<u8> {
    resource.as_ssz_bytes()
}

/// Convert OCaml bytes to a Resource
pub fn resource_from_ocaml(bytes: &[u8]) -> Result<Resource> {
    Resource::from_ssz_bytes(bytes).map_err(|e| anyhow::anyhow!("SSZ decode error: {}", e.message))
}

/// Convert a Handler to OCaml bytes
pub fn handler_to_ocaml(handler: &Handler) -> Vec<u8> {
    handler.as_ssz_bytes()
}

/// Convert OCaml bytes to a Handler
pub fn handler_from_ocaml(bytes: &[u8]) -> Result<Handler> {
    Handler::from_ssz_bytes(bytes).map_err(|e| anyhow::anyhow!("SSZ decode error: {}", e.message))
}

//-----------------------------------------------------------------------------
// TEL Core Type Adapters
//-----------------------------------------------------------------------------

/// Convert an Effect to OCaml bytes
pub fn effect_to_ocaml(effect: &Effect) -> Vec<u8> {
    effect.as_ssz_bytes()
}

/// Convert OCaml bytes to an Effect
pub fn effect_from_ocaml(bytes: &[u8]) -> Result<Effect> {
    Effect::from_ssz_bytes(bytes).map_err(|e| anyhow::anyhow!("SSZ decode error: {}", e.message))
}

/// Convert an Intent to OCaml bytes
pub fn intent_to_ocaml(intent: &Intent) -> Vec<u8> {
    intent.as_ssz_bytes()
}

/// Convert OCaml bytes to an Intent
pub fn intent_from_ocaml(bytes: &[u8]) -> Result<Intent> {
    Intent::from_ssz_bytes(bytes).map_err(|e| anyhow::anyhow!("SSZ decode error: {}", e.message))
}

/// Convert an Edge to OCaml bytes
pub fn edge_to_ocaml(edge: &Edge) -> Vec<u8> {
    edge.as_ssz_bytes()
}

/// Convert OCaml bytes to an Edge
pub fn edge_from_ocaml(bytes: &[u8]) -> Result<Edge> {
    Edge::from_ssz_bytes(bytes).map_err(|e| anyhow::anyhow!("SSZ decode error: {}", e.message))
}

//-----------------------------------------------------------------------------
// Hex String Utilities
//-----------------------------------------------------------------------------

/// Convert a hex string from OCaml
pub fn hex_from_ocaml(hex_str: &str) -> Result<Vec<u8>> {
    hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Hex decode error: {}", e))
}

/// Convert bytes to a hex string for OCaml
pub fn hex_to_ocaml(bytes: &[u8]) -> String {
    hex::encode(bytes)
} 