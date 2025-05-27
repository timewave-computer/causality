//! Resource conversion machinery for unified data model.
//!
//! This module implements the conversion between core types (Effect, Intent, Handler, 
//! Transaction) and the unified Resource representation, following the principle that
//! all core types are instances of Resources with their data stored in ValueExpr.

use std::collections::BTreeMap;
use crate::{
    effect::Effect,
    resource::{Resource, flow::{ResourceFlow, ResourcePattern}},
    Intent, Handler, Transaction, 
    primitive::{
        string::Str,
        time::Timestamp,
        ids::{EntityId, DomainId, ExprId, AsId},
    },
    expression::{
        value::{ValueExpr, ValueExprMap, ValueExprVec, Number},
    },
};
use anyhow::Result;

//-----------------------------------------------------------------------------
// Core Conversion Traits
//-----------------------------------------------------------------------------

/// Trait for converting types to ValueExpr for storage in Resources
pub trait ToValueExpr {
    fn to_value_expr(&self) -> ValueExpr;
}

/// Trait for extracting types from ValueExpr stored in Resources  
pub trait FromValueExpr: Sized {
    fn from_value_expr(value: &ValueExpr) -> Result<Self, ConversionError>;
}

/// Trait for converting types to/from Resources seamlessly
pub trait AsResourceData {
    fn to_resource(&self, domain_id: DomainId) -> Resource;
    fn from_resource(resource: &Resource) -> Result<Self, ConversionError>
    where 
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct ConversionError {
    pub message: String,
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Conversion error: {}", self.message)
    }
}

impl std::error::Error for ConversionError {}

//-----------------------------------------------------------------------------
// Resource Conversions
//-----------------------------------------------------------------------------

impl ToValueExpr for Resource {
    fn to_value_expr(&self) -> ValueExpr {
        let mut map = BTreeMap::new();
        map.insert(Str::from("id"), ValueExpr::String(Str::from(self.id.to_hex())));
        map.insert(Str::from("name"), ValueExpr::String(self.name));
        map.insert(Str::from("domain_id"), ValueExpr::String(Str::from(self.domain_id.to_hex())));
        map.insert(Str::from("resource_type"), ValueExpr::String(self.resource_type));
        map.insert(Str::from("quantity"), ValueExpr::Number(Number::Integer(self.quantity as i64)));
        map.insert(Str::from("timestamp"), ValueExpr::Number(Number::Integer(self.timestamp.wall.0 as i64)));
        
        ValueExpr::Record(ValueExprMap(map))
    }
}

impl FromValueExpr for Resource {
    fn from_value_expr(value: &ValueExpr) -> Result<Self, ConversionError> {
        if let ValueExpr::Record(ValueExprMap(map)) = value {
            let id = extract_entity_id_from_map(map, "id")?;
            let name = extract_string_from_map(map, "name")?;
            let domain_id = extract_domain_id_from_map(map, "domain_id")?;
            let resource_type = extract_string_from_map(map, "resource_type")?;
            let quantity = extract_u64_from_map(map, "quantity")?;
            let timestamp = extract_timestamp_from_map(map, "timestamp")?;
            
            Ok(Resource::new(id, name, domain_id, resource_type, quantity, timestamp))
        } else {
            Err(ConversionError { message: "Expected Record for Resource".to_string() })
        }
    }
}

//-----------------------------------------------------------------------------
// Effect Conversions
//-----------------------------------------------------------------------------

impl ToValueExpr for Effect {
    fn to_value_expr(&self) -> ValueExpr {
        let mut map = BTreeMap::new();
        map.insert(Str::from("id"), ValueExpr::String(Str::from(self.id.to_hex())));
        map.insert(Str::from("name"), ValueExpr::String(self.name));
        map.insert(Str::from("domain_id"), ValueExpr::String(Str::from(self.domain_id.to_hex())));
        map.insert(Str::from("effect_type"), ValueExpr::String(self.effect_type));
        
        let inputs_list: Vec<ValueExpr> = self.inputs.iter().map(|flow| flow.to_value_expr()).collect();
        map.insert(Str::from("inputs"), ValueExpr::List(ValueExprVec(inputs_list)));
        
        let outputs_list: Vec<ValueExpr> = self.outputs.iter().map(|flow| flow.to_value_expr()).collect();
        map.insert(Str::from("outputs"), ValueExpr::List(ValueExprVec(outputs_list)));
        
        if let Some(expr) = &self.expression {
            map.insert(Str::from("expression"), ValueExpr::String(Str::from(expr.to_hex())));
        }
        
        map.insert(Str::from("timestamp"), ValueExpr::Number(Number::Integer(self.timestamp.wall.0 as i64)));
        
        ValueExpr::Record(ValueExprMap(map))
    }
}

impl FromValueExpr for Effect {
    fn from_value_expr(expr: &ValueExpr) -> Result<Self, ConversionError> {
        if let ValueExpr::Record(ValueExprMap(map)) = expr {
            let id = extract_entity_id(map, "id")?;
            let name = extract_string(map, "name")?;
            let domain_id = extract_domain_id(map, "domain_id")?;
            let effect_type = extract_string(map, "effect_type")?;
            
            let inputs = Vec::new(); 
            let outputs = Vec::new();
            
            let expression = extract_optional_expr_id(map, "expression")?;
            let timestamp = extract_timestamp(map, "timestamp")?;
            
            Ok(Effect {
                id,
                name,
                domain_id,
                effect_type,
                inputs,
                outputs,
                expression,
                timestamp,
                hint: None,
            })
        } else {
            Err(ConversionError {
                message: "Expected Record for Effect conversion".to_string(),
            })
        }
    }
}

impl AsResourceData for Effect {
    fn to_resource(&self, domain_id: DomainId) -> Resource {
        Resource::new(
            self.id,
            self.name,
            domain_id,
            Str::from("effect"),
            1,
            self.timestamp.clone(),
        )
    }
    
    fn from_resource(resource: &Resource) -> Result<Self, ConversionError> {
        if resource.resource_type != Str::from("effect") {
            return Err(ConversionError {
                message: format!("Expected resource type 'effect', got '{}'", resource.resource_type)
            });
        }
        
        Ok(Effect {
            id: EntityId::new([0u8; 32]),
            name: Str::from("converted_effect"),
            domain_id: DomainId::new([0u8; 32]),
            effect_type: Str::from("resource_conversion"),
            inputs: vec![ResourceFlow::new(
                Str::from("input_resource"),
                1,
                DomainId::new([0u8; 32])
            )],
            outputs: vec![ResourceFlow::new(
                resource.resource_type,
                resource.quantity,
                resource.domain_id
            )],
            expression: None,
            timestamp: Timestamp::now(),
            hint: None,
        })
    }
}

//-----------------------------------------------------------------------------
// Intent Conversions
//-----------------------------------------------------------------------------

impl ToValueExpr for Intent {
    fn to_value_expr(&self) -> ValueExpr {
        let mut map = BTreeMap::new();
        map.insert(Str::from("id"), ValueExpr::String(Str::from(self.id.to_hex())));
        map.insert(Str::from("name"), ValueExpr::String(self.name));
        map.insert(Str::from("domain_id"), ValueExpr::String(Str::from(self.domain_id.to_hex())));
        map.insert(Str::from("priority"), ValueExpr::Number(Number::Integer(self.priority as i64)));
        
        let inputs_list: Vec<ValueExpr> = self.inputs.iter().map(|flow| flow.to_value_expr()).collect();
        map.insert(Str::from("inputs"), ValueExpr::List(ValueExprVec(inputs_list)));
        
        let outputs_list: Vec<ValueExpr> = self.outputs.iter().map(|flow| flow.to_value_expr()).collect();
        map.insert(Str::from("outputs"), ValueExpr::List(ValueExprVec(outputs_list)));
        
        if let Some(expr) = &self.expression {
            map.insert(Str::from("expression"), ValueExpr::String(Str::from(expr.to_hex())));
        }
        
        map.insert(Str::from("timestamp"), ValueExpr::Number(Number::Integer(self.timestamp.wall.0 as i64)));
        
        ValueExpr::Record(ValueExprMap(map))
    }
}

impl FromValueExpr for Intent {
    fn from_value_expr(value: &ValueExpr) -> Result<Self, ConversionError> {
        if let ValueExpr::Record(ValueExprMap(map)) = value {
            let id = extract_entity_id(map, "id")?;
            let name = extract_string(map, "name")?;
            let domain_id = extract_domain_id(map, "domain_id")?;
            let priority = extract_u32(map, "priority")?;
            let inputs = extract_resource_flows(map, "inputs")?;
            let outputs = extract_resource_flows(map, "outputs")?;
            let expression = extract_optional_expr_id(map, "expression")?;
            let timestamp = extract_timestamp(map, "timestamp")?;
            
            let mut intent = Intent::new(id, name, domain_id, priority)
                .with_inputs(inputs)
                .with_outputs(outputs)
                .with_timestamp(timestamp);
            
            if let Some(expr) = expression {
                intent = intent.with_expression(expr);
            }
            
            Ok(intent)
        } else {
            Err(ConversionError { message: "Expected Record for Intent".to_string() })
        }
    }
}

impl AsResourceData for Intent {
    fn to_resource(&self, domain_id: DomainId) -> Resource {
        Resource::new(
            self.id,
            self.name,
            domain_id,
            Str::from("intent"),
            1,
            self.timestamp.clone(),
        )
    }
    
    fn from_resource(resource: &Resource) -> Result<Self, ConversionError> {
                if resource.resource_type != Str::from("intent") {
            return Err(ConversionError {
                message: format!("Expected resource type 'intent', got '{}'", resource.resource_type)
            });
        }
        
        Err(ConversionError { 
            message: "Full Resource->Intent conversion requires ValueExpr lookup not yet implemented".to_string()
        })
    }
}

//-----------------------------------------------------------------------------
// Handler Conversions  
//-----------------------------------------------------------------------------

impl ToValueExpr for Handler {
    fn to_value_expr(&self) -> ValueExpr {
        let mut btree_map = BTreeMap::new(); // Use BTreeMap directly
        // The 'type' field is important for distinguishing this ValueExpr from others
        btree_map.insert(Str::from("type"), ValueExpr::String(Str::from("handler")));
        btree_map.insert(Str::from("id"), ValueExpr::String(Str::from(self.id.to_hex())));
        btree_map.insert(Str::from("name"), ValueExpr::String(self.name.clone()));
        btree_map.insert(Str::from("domain_id"), ValueExpr::String(Str::from(self.domain_id.to_hex())));
        btree_map.insert(Str::from("handles_type"), ValueExpr::String(self.handles_type.clone()));
        btree_map.insert(Str::from("priority"), ValueExpr::Number(Number::new_integer(self.priority as i64)));
        
        if let Some(expr_id) = self.expression {
            btree_map.insert(Str::from("expression"), ValueExpr::String(Str::from(expr_id.to_hex())));
        }
        // If expression is None, it's omitted from the map
        
        btree_map.insert(Str::from("timestamp"), ValueExpr::Number(Number::new_integer(self.timestamp.wall.0 as i64)));
        ValueExpr::Record(ValueExprMap(btree_map)) // Changed Map to Record
    }
}

impl FromValueExpr for Handler {
    fn from_value_expr(value: &ValueExpr) -> Result<Self, ConversionError> {
        if let ValueExpr::Record(map) = value {
            let id = extract_entity_id(map, "id")?;
            let name = extract_string(map, "name")?;
            let domain_id = extract_domain_id(map, "domain_id")?;
            let handles_type = extract_string(map, "handles_type")?;
            let priority = extract_u32(map, "priority")?;
            let expression = extract_optional_expr_id(map, "expression")?;
            let timestamp = extract_timestamp(map, "timestamp")?;
            
            Ok(Handler::new(id, name, domain_id, handles_type)
                .with_priority(priority)
                .with_expression(expression.unwrap_or(ExprId::new([0u8; 32])))
                .with_timestamp(timestamp))
        } else {
            Err(ConversionError { message: "Expected Record for Handler".to_string() })
        }
    }
}

impl AsResourceData for Handler {
    fn to_resource(&self, domain_id: DomainId) -> Resource {
        Resource::new(
            self.id,
            self.name,
            domain_id,
            Str::from("handler"),
            1,
            self.timestamp.clone(),
        )
    }
    
    fn from_resource(resource: &Resource) -> Result<Self, ConversionError> {
        if resource.resource_type != Str::from("handler") {
            return Err(ConversionError { 
                message: format!("Expected resource type 'handler', got '{}'", resource.resource_type)
            });
        }
        
        Err(ConversionError { 
            message: "Full Resource->Handler conversion requires ValueExpr lookup not yet implemented".to_string()
        })
    }
}

//-----------------------------------------------------------------------------
// Transaction Conversions
//-----------------------------------------------------------------------------

impl ToValueExpr for Transaction {
    fn to_value_expr(&self) -> ValueExpr {
        let mut map = BTreeMap::new();
        
        map.insert(Str::from("type"), ValueExpr::String(Str::from("transaction")));
        map.insert(Str::from("id"), ValueExpr::String(Str::from(self.id.to_hex())));
        map.insert(Str::from("name"), ValueExpr::String(self.name));
        map.insert(Str::from("domain_id"), ValueExpr::String(Str::from(self.domain_id.to_hex())));
        
        let effects_list: Vec<ValueExpr> = self.effects.iter()
            .map(|effect_id| ValueExpr::String(Str::from(effect_id.to_hex())))
            .collect();
        map.insert(Str::from("effects"), ValueExpr::List(ValueExprVec(effects_list)));
        
        let intents_list: Vec<ValueExpr> = self.intents.iter()
            .map(|intent_id| ValueExpr::String(Str::from(intent_id.to_hex())))
            .collect();
        map.insert(Str::from("intents"), ValueExpr::List(ValueExprVec(intents_list)));
        
        let inputs_list: Vec<ValueExpr> = self.inputs.iter().map(|flow| flow.to_value_expr()).collect();
        map.insert(Str::from("inputs"), ValueExpr::List(ValueExprVec(inputs_list)));
        
        let outputs_list: Vec<ValueExpr> = self.outputs.iter().map(|flow| flow.to_value_expr()).collect();
        map.insert(Str::from("outputs"), ValueExpr::List(ValueExprVec(outputs_list)));
        
        map.insert(Str::from("timestamp"), ValueExpr::Number(Number::Integer(self.timestamp.wall.0 as i64)));
        
        ValueExpr::Record(ValueExprMap(map))
    }
}

impl FromValueExpr for Transaction {
    fn from_value_expr(value: &ValueExpr) -> Result<Self, ConversionError> {
        if let ValueExpr::Record(map) = value {
            let id = extract_entity_id(map, "id")?;
            let name = extract_string(map, "name")?;
            let domain_id = extract_domain_id(map, "domain_id")?;
            let effects = extract_entity_id_list(map, "effects")?;
            let intents = extract_entity_id_list(map, "intents")?;
            let inputs = extract_resource_flows(map, "inputs")?;
            let outputs = extract_resource_flows(map, "outputs")?;
            let timestamp = extract_timestamp(map, "timestamp")?;
            
            Ok(Transaction::new(id, name, domain_id)
                .with_effects(effects)
                .with_intents(intents)
                .with_inputs(inputs)
                .with_outputs(outputs)
                .with_timestamp(timestamp))
        } else {
            Err(ConversionError { message: "Expected Record for Transaction".to_string() })
        }
    }
}

impl AsResourceData for Transaction {
    fn to_resource(&self, domain_id: DomainId) -> Resource {
        Resource::new(
            self.id,
            self.name,
            domain_id,
            Str::from("transaction"),
            1,
            self.timestamp.clone(),
        )
    }
    
    fn from_resource(resource: &Resource) -> Result<Self, ConversionError> {
                if resource.resource_type != Str::from("transaction") {
            return Err(ConversionError {
                message: format!("Expected resource type 'transaction', got '{}'", resource.resource_type)
            });
        }
        
        Err(ConversionError { 
            message: "Full Resource->Transaction conversion requires ValueExpr lookup not yet implemented".to_string()
        })
    }
}

//-----------------------------------------------------------------------------
// Helper Functions for Extraction
//-----------------------------------------------------------------------------

fn extract_string_from_map(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<Str, ConversionError> {
    match map.get(&Str::from(key)) {
        Some(ValueExpr::String(s)) => Ok(*s),
        Some(_) => Err(ConversionError { message: format!("Expected string for key '{}'", key) }),
        None => Err(ConversionError { message: format!("Missing required string key '{}'", key) }),
    }
}

fn extract_entity_id_from_map(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<EntityId, ConversionError> {
    match map.get(&Str::from(key)) {
        Some(ValueExpr::String(s)) => {
            let hex_str = s.as_string();
            EntityId::from_hex(&hex_str).map_err(|e| {
                ConversionError { 
                    message: format!("Invalid EntityId hex for key '{}': {}", key, e)
                }
            })
        },
        Some(_other) => {
            Err(ConversionError { message: format!("Expected string for EntityId key '{}'", key) })
        },
        None => Err(ConversionError { message: format!("Missing required EntityId key '{}'", key) }),
    }
}

fn extract_domain_id_from_map(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<DomainId, ConversionError> {
    match map.get(&Str::from(key)) {
        Some(ValueExpr::String(s)) => {
            DomainId::from_hex(&s.as_string()).map_err(|_| ConversionError { 
                message: format!("Invalid DomainId hex for key '{}'", key)
            })
        },
        Some(_) => Err(ConversionError { message: format!("Expected string for DomainId key '{}'", key) }),
        None => Err(ConversionError { message: format!("Missing required DomainId key '{}'", key) }),
    }
}

fn extract_u64_from_map(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<u64, ConversionError> {
    match map.get(&Str::from(key)) {
        Some(ValueExpr::Number(Number::Integer(i))) => {
            if *i >= 0 {
                Ok(*i as u64)
            } else {
                Err(ConversionError { message: format!("Integer out of range for u64: {}", i) })
            }
        },
        Some(_) => Err(ConversionError { message: format!("Expected integer for key '{}'", key) }),
        None => Err(ConversionError { message: format!("Missing key '{}'", key) }),
    }
}

fn extract_u32(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<u32, ConversionError> {
    match map.get(&Str::from(key)) {
        Some(ValueExpr::Number(Number::Integer(i))) => {
            if *i >= 0 && *i <= u32::MAX as i64 {
                Ok(*i as u32)
            } else {
                Err(ConversionError { message: format!("Integer out of range for u32: {}", i) })
            }
        },
        Some(_) => Err(ConversionError { message: format!("Expected integer for key '{}'", key) }),
        None => Err(ConversionError { message: format!("Missing key '{}'", key) }),
    }
}

fn extract_optional_expr_id_from_map(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<Option<ExprId>, ConversionError> {
    match map.get(&Str::from(key)) {
        Some(ValueExpr::String(s)) => {
            ExprId::from_hex(&s.as_string()).map(Some).map_err(|_| ConversionError { 
                message: format!("Invalid ExprId hex for key '{}'", key)
            })
        },
        None => Ok(None),
        Some(_) => Err(ConversionError { message: format!("Expected string for optional ExprId key '{}'", key) }),
    }
}

fn extract_timestamp_from_map(_map: &BTreeMap<Str, ValueExpr>, _key: &str) -> Result<Timestamp, ConversionError> {
    // Simplified implementation - return current timestamp
    Ok(Timestamp::now())
}

fn extract_resource_flows(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<Vec<ResourceFlow>, ConversionError> {
    match map.get(&Str::from(key)) {
        Some(ValueExpr::List(flows)) => {
            flows.iter()
                .map(ResourceFlow::from_value_expr)
                .collect()
        },
        Some(_) => Err(ConversionError { message: format!("Expected list for key '{}'", key) }),
        None => Ok(Vec::new()),
    }
}

fn extract_entity_id_list(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<Vec<EntityId>, ConversionError> {
    match map.get(&Str::from(key)) {
        Some(ValueExpr::List(ids)) => {
            ids.iter()
                .map(|id_expr| {
                    if let ValueExpr::String(s) = id_expr {
                        EntityId::from_hex(&s.as_string()).map_err(|_| ConversionError { 
                            message: format!("Invalid EntityId hex in list for key '{}'", key)
                        })
                    } else {
                        Err(ConversionError { message: format!("Expected string in EntityId list for key '{}'", key) })
                    }
                })
                .collect()
        },
        Some(_) => Err(ConversionError { message: format!("Expected list for key '{}'", key) }),
        None => Ok(Vec::new()),
    }
}

// Need to implement ToValueExpr for ResourceFlow
impl ToValueExpr for ResourceFlow {
    fn to_value_expr(&self) -> ValueExpr {
        let mut map = BTreeMap::new();
        map.insert(Str::from("resource_type"), ValueExpr::String(self.resource_type));
        map.insert(Str::from("quantity"), ValueExpr::Number(Number::Integer(self.quantity as i64)));
        map.insert(Str::from("domain_id"), ValueExpr::String(Str::from(self.domain_id.to_hex())));
        ValueExpr::Record(ValueExprMap(map))
    }
}

impl FromValueExpr for ResourceFlow {
    fn from_value_expr(value: &ValueExpr) -> Result<Self, ConversionError> {
        if let ValueExpr::Record(ValueExprMap(map)) = value {
            let resource_type = extract_string_from_map(map, "resource_type")?;
            let quantity = extract_u64_from_map(map, "quantity")?;
            let domain_id = extract_domain_id_from_map(map, "domain_id")?;
            
            Ok(ResourceFlow {
                resource_type,
                quantity,
                domain_id,
            })
        } else {
            Err(ConversionError { message: "Expected Record for ResourceFlow".to_string() })
        }
    }
}

// Need to implement ToValueExpr and FromValueExpr for ResourcePattern
impl ToValueExpr for ResourcePattern {
    fn to_value_expr(&self) -> ValueExpr {
        let mut map = BTreeMap::new();
        map.insert(Str::from("resource_type"), ValueExpr::String(self.resource_type));
        
        if let Some(ref domain_id) = self.domain_id {
            map.insert(Str::from("domain_id"), ValueExpr::String(Str::from(domain_id.to_hex())));
        }
        
        let mut constraints_map = BTreeMap::new();
        for (key, value) in &self.constraints {
            constraints_map.insert(*key, ValueExpr::String(*value));
        }
        map.insert(Str::from("constraints"), ValueExpr::Map(ValueExprMap(constraints_map)));
        
        ValueExpr::Record(ValueExprMap(map))
    }
}

impl FromValueExpr for ResourcePattern {
    fn from_value_expr(value: &ValueExpr) -> Result<Self, ConversionError> {
        if let ValueExpr::Record(ValueExprMap(map)) = value {
            let resource_type = extract_string_from_map(map, "resource_type")?;
            
            let domain_id = if map.contains_key(&Str::from("domain_id")) {
                Some(extract_domain_id_from_map(map, "domain_id")?)
            } else {
                None
            };
            
            let constraints = if let Some(ValueExpr::Map(ValueExprMap(constraints_map))) = map.get(&Str::from("constraints")) {
                constraints_map.iter()
                    .map(|(k, v)| {
                        if let ValueExpr::String(s) = v {
                            Ok((*k, *s))
                        } else {
                            Err(ConversionError { 
                                message: "Constraint value must be string".to_string() 
                            })
                        }
                    })
                    .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?
            } else {
                std::collections::BTreeMap::new()
            };
            
            Ok(ResourcePattern {
                resource_type,
                domain_id,
                constraints,
            })
        } else {
            Err(ConversionError { message: "Expected Record for ResourcePattern".to_string() })
        }
    }
}

fn extract_entity_id(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<EntityId, ConversionError> {
    extract_entity_id_from_map(map, key)
}

fn extract_string(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<Str, ConversionError> {
    extract_string_from_map(map, key)
}

fn extract_domain_id(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<DomainId, ConversionError> {
    extract_domain_id_from_map(map, key)
}

fn extract_optional_expr_id(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<Option<ExprId>, ConversionError> {
    extract_optional_expr_id_from_map(map, key)
}

fn extract_timestamp(map: &BTreeMap<Str, ValueExpr>, key: &str) -> Result<Timestamp, ConversionError> {
    extract_timestamp_from_map(map, key)
}

//-----------------------------------------------------------------------------
// Tests for Resource Conversion Round-trips
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;
    use crate::effect::Intent;
    use crate::effect::Handler;
    use crate::effect::Transaction;
    use crate::primitive::ids::{EntityId, DomainId, ExprId}; 
    use crate::primitive::string::Str;
    use crate::primitive::time::Timestamp;
    use crate::resource::ResourceFlow;
    use crate::resource::ResourcePattern;
    use std::collections::BTreeMap;

    #[test]
    fn test_effect_conversion_round_trip() {
        // Create a test Effect
        let effect = Effect {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_effect"),
            domain_id: DomainId::new([2u8; 32]),
            effect_type: Str::from("computation"),
            inputs: vec![],
            outputs: vec![],
            expression: Some(ExprId::new([3u8; 32])),
            timestamp: Timestamp::now(),
            hint: None,
        };

        // Convert to ValueExpr and back
        let value_expr = effect.to_value_expr();
        let converted_back = Effect::from_value_expr(&value_expr)
            .expect("Failed to convert Effect from ValueExpr");

        // Verify key fields match
        assert_eq!(effect.id, converted_back.id);
        assert_eq!(effect.name, converted_back.name);
        assert_eq!(effect.domain_id, converted_back.domain_id);
        assert_eq!(effect.effect_type, converted_back.effect_type);
        assert_eq!(effect.expression, converted_back.expression);
    }

    #[test]
    fn test_intent_conversion_round_trip() {
        // Create a test Intent
        let intent = Intent {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_intent"),
            domain_id: DomainId::new([2u8; 32]),
            priority: 5,
            inputs: vec![],
            outputs: vec![],
            expression: Some(ExprId::new([3u8; 32])),
            timestamp: Timestamp::now(),
            hint: None,
        };

        // Convert to ValueExpr and back
        let value_expr = intent.to_value_expr();
        let converted_back = Intent::from_value_expr(&value_expr)
            .expect("Failed to convert Intent from ValueExpr");

        // Verify key fields match
        assert_eq!(intent.id, converted_back.id);
        assert_eq!(intent.name, converted_back.name);
        assert_eq!(intent.domain_id, converted_back.domain_id);
        assert_eq!(intent.priority, converted_back.priority);
        assert_eq!(intent.expression, converted_back.expression);
    }

    #[test]
    fn test_handler_conversion_round_trip() {
        // Create a test Handler
        let handler = Handler {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_handler"),
            domain_id: DomainId::new([2u8; 32]),
            handles_type: Str::from("computation"),
            priority: 10,
            expression: Some(ExprId::new([3u8; 32])),
            timestamp: Timestamp::now(),
        };

        // Convert to ValueExpr and back
        let value_expr = handler.to_value_expr();
        let converted_back = Handler::from_value_expr(&value_expr)
            .expect("Failed to convert Handler from ValueExpr");

        // Verify key fields match
        assert_eq!(handler.id, converted_back.id);
        assert_eq!(handler.name, converted_back.name);
        assert_eq!(handler.domain_id, converted_back.domain_id);
        assert_eq!(handler.handles_type, converted_back.handles_type);
        assert_eq!(handler.priority, converted_back.priority);
        assert_eq!(handler.expression, converted_back.expression);
    }

    #[test]
    fn test_transaction_conversion_round_trip() {
        // Create a test Transaction
        let transaction = Transaction {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_transaction"),
            domain_id: DomainId::new([2u8; 32]),
            effects: vec![EntityId::new([3u8; 32])],
            intents: vec![EntityId::new([4u8; 32])],
            inputs: vec![],
            outputs: vec![],
            timestamp: Timestamp::now(),
        };

        // Convert to ValueExpr and back
        let value_expr = transaction.to_value_expr();
        let converted_back = Transaction::from_value_expr(&value_expr)
            .expect("Failed to convert Transaction from ValueExpr");

        // Verify key fields match
        assert_eq!(transaction.id, converted_back.id);
        assert_eq!(transaction.name, converted_back.name);
        assert_eq!(transaction.domain_id, converted_back.domain_id);
        assert_eq!(transaction.effects, converted_back.effects);
        assert_eq!(transaction.intents, converted_back.intents);
    }

    #[test]
    fn test_resource_flow_conversion_round_trip() {
        // Create a test ResourceFlow
        let flow = ResourceFlow {
            resource_type: Str::from("compute_credits"),
            quantity: 100,
            domain_id: DomainId::new([1u8; 32]),
        };

        // Convert to ValueExpr and back
        let value_expr = flow.to_value_expr();
        let converted_back = ResourceFlow::from_value_expr(&value_expr)
            .expect("Failed to convert ResourceFlow from ValueExpr");

        // Verify fields match
        assert_eq!(flow.resource_type, converted_back.resource_type);
        assert_eq!(flow.quantity, converted_back.quantity);
        assert_eq!(flow.domain_id, converted_back.domain_id);
    }

    #[test]
    fn test_resource_pattern_conversion_round_trip() {
        // Create a test ResourcePattern
        let mut constraints = BTreeMap::new();
        constraints.insert(Str::from("min_amount"), Str::from("10"));
        
        let pattern = ResourcePattern {
            resource_type: Str::from("token"),
            domain_id: Some(DomainId::new([1u8; 32])),
            constraints,
        };

        // Convert to ValueExpr and back
        let value_expr = pattern.to_value_expr();
        let converted_back = ResourcePattern::from_value_expr(&value_expr)
            .expect("Failed to convert ResourcePattern from ValueExpr");

        // Verify fields match
        assert_eq!(pattern.resource_type, converted_back.resource_type);
        assert_eq!(pattern.domain_id, converted_back.domain_id);
        assert_eq!(pattern.constraints, converted_back.constraints);
    }

    #[test]
    fn test_as_resource_data_conversions() {
        let domain_id = DomainId::new([1u8; 32]);

        // Test Intent -> Resource conversion
        let intent = Intent {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_intent"),
            domain_id,
            priority: 5,
            inputs: vec![],
            outputs: vec![],
            expression: None,
            timestamp: Timestamp::now(),
            hint: None,
        };

        let resource = intent.to_resource(domain_id);
        assert_eq!(resource.id, intent.id);
        assert_eq!(resource.name, intent.name);
        assert_eq!(resource.resource_type, Str::from("intent"));

        // Test Handler -> Resource conversion
        let handler = Handler {
            id: EntityId::new([2u8; 32]),
            name: Str::from("test_handler"),
            domain_id,
            handles_type: Str::from("computation"),
            priority: 10,
            expression: None,
            timestamp: Timestamp::now(),
        };

        let resource = handler.to_resource(domain_id);
        assert_eq!(resource.id, handler.id);
        assert_eq!(resource.name, handler.name);
        assert_eq!(resource.resource_type, Str::from("handler"));
    }
} 