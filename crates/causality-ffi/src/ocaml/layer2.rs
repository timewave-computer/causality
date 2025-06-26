//! Layer 2 (Effect System) bindings for OCaml FFI
//!
//! This module provides OCaml access to the Causality effect system,
//! enabling compilation from OCaml Layer 2 DSL to Rust Layer 2 structures.

use crate::ocaml::{core_types::*, error_handling::result_to_ocaml};
use causality_core::{
    effect::{
        intent::{Intent, IntentId, IntentPriority, LocationRequirements, ResourceRef, AccessPattern},
        core::{EffectExpr, EffectExprKind},
        transform_constraint::{TransformConstraint, TransformDefinition},
    },
    lambda::base::{TypeInner, BaseType, Location},
    system::{
        content_addressing::{EntityId, ResourceId},
        deterministic::DeterministicSystem,
    },
};
use std::collections::{BTreeMap, BTreeSet};

//-----------------------------------------------------------------------------
// OCaml-Compatible Types for Layer 2
//-----------------------------------------------------------------------------

/// OCaml-compatible Intent type for FFI
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, ocaml::FromValue, ocaml::ToValue)]
pub struct OcamlIntent {
    pub id: Vec<u8>,  // EntityId as bytes
    pub name: String,
    pub domain_id: Vec<u8>,  // Domain ID as bytes
    pub priority: i32,
    pub inputs: Vec<OcamlResourceFlow>,
    pub outputs: Vec<OcamlResourceFlow>,
    pub expression: Option<Vec<u8>>,  // ExprId as bytes
    pub timestamp: i64,
    pub hint: Option<Vec<u8>>,  // ExprId as bytes
}

/// OCaml-compatible Effect type for FFI
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, ocaml::FromValue, ocaml::ToValue)]
pub struct OcamlEffect {
    pub id: Vec<u8>,  // EntityId as bytes
    pub name: String,
    pub domain_id: Vec<u8>,  // Domain ID as bytes
    pub effect_type: String,
    pub inputs: Vec<OcamlResourceFlow>,
    pub outputs: Vec<OcamlResourceFlow>,
    pub expression: Option<Vec<u8>>,  // ExprId as bytes
    pub timestamp: i64,
    pub hint: Option<Vec<u8>>,  // ExprId as bytes
}

/// OCaml-compatible Handler type for FFI
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, ocaml::FromValue, ocaml::ToValue)]
pub struct OcamlHandler {
    pub id: Vec<u8>,  // EntityId as bytes
    pub name: String,
    pub domain_id: Vec<u8>,  // Domain ID as bytes
    pub handles_type: String,
    pub priority: i32,
    pub expression: Option<Vec<u8>>,  // ExprId as bytes
    pub timestamp: i64,
    pub hint: Option<Vec<u8>>,  // ExprId as bytes
}

/// OCaml-compatible Transaction type for FFI
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, ocaml::FromValue, ocaml::ToValue)]
pub struct OcamlTransaction {
    pub id: Vec<u8>,  // EntityId as bytes
    pub name: String,
    pub domain_id: Vec<u8>,  // Domain ID as bytes
    pub effects: Vec<Vec<u8>>,  // Effect IDs as bytes
    pub intents: Vec<Vec<u8>>,  // Intent IDs as bytes
    pub inputs: Vec<OcamlResourceFlow>,
    pub outputs: Vec<OcamlResourceFlow>,
    pub timestamp: i64,
}

/// OCaml-compatible ResourceFlow type for FFI
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, ocaml::FromValue, ocaml::ToValue)]
pub struct OcamlResourceFlow {
    pub resource_type: String,
    pub quantity: i64,
    pub domain_id: Vec<u8>,  // Domain ID as bytes
}

//-----------------------------------------------------------------------------
// Conversion Functions: OCaml ↔ Rust
//-----------------------------------------------------------------------------

/// Convert OCaml Intent to Rust Intent
pub fn ocaml_intent_to_rust(ocaml_intent: &OcamlIntent) -> Result<Intent, String> {
    let id = IntentId::new(
        u64::from_le_bytes(
            ocaml_intent.id.get(0..8)
                .ok_or("Invalid intent ID")?
                .try_into()
                .map_err(|_| "Invalid intent ID format")?
        )
    );
    
    let domain = Location::Local; // Simplified for minimal implementation
    
    let mut intent = Intent::new(domain);
    intent.id = id;
    
    Ok(intent)
}

/// Convert Rust Intent to OCaml Intent (simplified)
pub fn rust_intent_to_ocaml(intent: &Intent) -> OcamlIntent {
    OcamlIntent {
        id: intent.id.0.to_le_bytes().to_vec(),
        name: "rust_intent".to_string(), // Simplified
        domain_id: vec![0u8; 32], // Simplified domain ID
        priority: match intent.priority {
            IntentPriority::Low => 1,
            IntentPriority::Normal => 2,
            IntentPriority::High => 3,
            IntentPriority::Critical => 4,
            IntentPriority::Immediate => 5,
        },
        inputs: vec![], // Simplified
        outputs: vec![], // Simplified
        expression: None,
        timestamp: 0,
        hint: None,
    }
}

/// Convert OCaml Effect to Rust EffectExpr (simplified)
pub fn ocaml_effect_to_rust(ocaml_effect: &OcamlEffect) -> Result<EffectExpr, String> {
    // For minimal implementation, create a simple perform effect
    use causality_core::lambda::{Term, Symbol};
    
    let effect_expr = EffectExpr {
        kind: EffectExprKind::Perform {
            effect_tag: ocaml_effect.effect_type.clone(),
            args: vec![], // Simplified
        },
        ty: Some(TypeInner::Base(BaseType::Unit)),
    };
    
    Ok(effect_expr)
}

//-----------------------------------------------------------------------------
// FFI Functions for OCaml
//-----------------------------------------------------------------------------

/// Create a new Intent from OCaml
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn intent_create(name: String, domain_id: Vec<u8>) -> ocaml::Value {
    let result = || -> Result<Vec<u8>, String> {
        // Create a simple intent ID from the name hash
        let id_bytes = name.as_bytes();
        let mut intent_id = [0u8; 8];
        intent_id[..std::cmp::min(8, id_bytes.len())].copy_from_slice(&id_bytes[..std::cmp::min(8, id_bytes.len())]);
        
        Ok(intent_id.to_vec())
    };
    
    result_to_ocaml(result())
}

/// Add a constraint to an Intent
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn intent_add_constraint(intent_id: Vec<u8>, constraint_type: String) -> ocaml::Value {
    let result = || -> Result<bool, String> {
        // For minimal implementation, just validate the constraint type
        match constraint_type.as_str() {
            "local_transform" | "remote_transform" | "data_migration" => Ok(true),
            _ => Err(format!("Unknown constraint type: {}", constraint_type)),
        }
    };
    
    result_to_ocaml(result())
}

/// Add a capability requirement to an Intent
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn intent_add_capability(intent_id: Vec<u8>, capability_name: String) -> ocaml::Value {
    let result = || -> Result<bool, String> {
        // For minimal implementation, just validate the capability name
        if capability_name.is_empty() {
            Err("Capability name cannot be empty".to_string())
        } else {
            Ok(true)
        }
    };
    
    result_to_ocaml(result())
}

/// Compile an Intent to Layer 1 (returns ExprId)
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn intent_compile(intent_id: Vec<u8>) -> ocaml::Value {
    let result = || -> Result<Vec<u8>, String> {
        // For minimal implementation, create a placeholder expression ID
        let expr_id = [0u8; 32]; // 32-byte expression ID
        Ok(expr_id.to_vec())
    };
    
    result_to_ocaml(result())
}

/// Create a pure effect
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn effect_pure(value: LispValue) -> ocaml::Value {
    let result = || -> Result<Vec<u8>, String> {
        // For minimal implementation, create a placeholder effect ID
        let effect_id = [1u8; 32]; // 32-byte effect ID
        Ok(effect_id.to_vec())
    };
    
    result_to_ocaml(result())
}

/// Create an effect bind operation
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn effect_bind(effect1_id: Vec<u8>, var_name: String, effect2_id: Vec<u8>) -> ocaml::Value {
    let result = || -> Result<Vec<u8>, String> {
        // For minimal implementation, create a placeholder bind effect ID
        let bind_effect_id = [2u8; 32]; // 32-byte effect ID
        Ok(bind_effect_id.to_vec())
    };
    
    result_to_ocaml(result())
}

/// Create a perform effect
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn effect_perform(effect_tag: String, args: Vec<LispValue>) -> ocaml::Value {
    let result = || -> Result<Vec<u8>, String> {
        // For minimal implementation, create a placeholder perform effect ID
        let perform_effect_id = [3u8; 32]; // 32-byte effect ID
        Ok(perform_effect_id.to_vec())
    };
    
    result_to_ocaml(result())
}

/// Compile an Effect to Layer 1 (returns ExprId)
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn effect_compile(effect_id: Vec<u8>) -> ocaml::Value {
    let result = || -> Result<Vec<u8>, String> {
        // For minimal implementation, create a placeholder expression ID
        let expr_id = [4u8; 32]; // 32-byte expression ID
        Ok(expr_id.to_vec())
    };
    
    result_to_ocaml(result())
}

//-----------------------------------------------------------------------------
// Utility Functions
//-----------------------------------------------------------------------------

/// Helper to convert bytes to EntityId
fn bytes_to_entity_id(bytes: &[u8]) -> Result<EntityId, String> {
    if bytes.len() != 32 {
        return Err("EntityId must be exactly 32 bytes".to_string());
    }
    
    let mut id_bytes = [0u8; 32];
    id_bytes.copy_from_slice(bytes);
    Ok(EntityId::from_bytes(id_bytes))
}

/// Helper to convert EntityId to bytes
fn entity_id_to_bytes(id: &EntityId) -> Vec<u8> {
    id.as_bytes().to_vec()
} 