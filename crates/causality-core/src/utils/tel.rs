// Purpose: TEL (Temporal Expression Language) utility functions for Effects, Intents, Handlers, and Resources.

use crate::extension_traits::ValueExprExt;
use causality_types::{
    core::{
        id::{AsId, DomainId, EntityId, ExprId, compute_id},
        numeric::Number,
        str::Str,
        Effect, Intent, Handler,
        resource_conversion::ToValueExpr,
    },
    expr::{
        expr_type::{TypeExpr, TypeExprMap},
        value::{ValueExpr, ValueExprMap, ValueExprVec},
    },
    Resource,
    serialization::Encode,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

//-----------------------------------------------------------------------------
// Effect Utilities
//-----------------------------------------------------------------------------

/// Convert an Effect to ValueExpr for serialization or debugging
pub fn value_expr_from_effect(effect: &Effect) -> ValueExpr {
    let mut map = BTreeMap::new();
    map.insert(
        Str::from("id"),
        ValueExpr::String(Str::from(effect.id.to_hex())),
    );
    map.insert(
        Str::from("domain_id"),
        ValueExpr::String(Str::from(effect.domain_id.to_hex())),
    );
    map.insert(
        Str::from("name"),
        ValueExpr::String(effect.name),
    );
    map.insert(
        Str::from("effect_type"),
        ValueExpr::String(effect.effect_type),
    );
    if let Some(expr_id) = &effect.expression {
        map.insert(
            Str::from("expression"),
            ValueExpr::String(Str::from(expr_id.to_hex())),
        );
    }
    map.insert(
        Str::from("inputs"),
        ValueExpr::List(ValueExprVec(
            effect
                .inputs
                .iter()
                .map(|flow| flow.to_value_expr())
                .collect(),
        )),
    );
    map.insert(
        Str::from("outputs"),
        ValueExpr::List(ValueExprVec(
            effect
                .outputs
                .iter()
                .map(|flow| flow.to_value_expr())
                .collect(),
        )),
    );
    map.insert(
        Str::from("timestamp"),
        ValueExpr::Number(Number::Integer(effect.timestamp.wall.0 as i64)),
    );
    ValueExpr::Record(ValueExprMap(map))
}

/// Compute the hash of an Effect
pub fn compute_effect_hash(effect: &Effect) -> [u8; 32] {
    let bytes = effect.as_ssz_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.finalize().into()
}

/// Serialize an effect to bytes for external storage
pub fn serialize_effect(effect: &Effect) -> Vec<u8> {
    let bytes = effect.as_ssz_bytes();
    println!(
        "Serialized effect {} to {} bytes",
        effect.id,
        bytes.len()
    );
    bytes
}

//-----------------------------------------------------------------------------
// Intent Utilities
//-----------------------------------------------------------------------------

/// Convert an Intent to ValueExpr for serialization or debugging
pub fn value_expr_from_intent(intent: &Intent) -> ValueExpr {
    let mut map = BTreeMap::new();

    map.insert(
        Str::from("id"),
        ValueExpr::String(Str::from(intent.id.to_hex())),
    );
    map.insert(
        Str::from("domain_id"),
        ValueExpr::String(Str::from(intent.domain_id.to_hex())),
    );
    map.insert(
        Str::from("name"),
        ValueExpr::String(intent.name),
    );
    map.insert(
        Str::from("priority"),
        ValueExpr::Number(Number::Integer(intent.priority as i64)),
    );

    map.insert(
        Str::from("inputs"),
        ValueExpr::List(ValueExprVec(
            intent
                .inputs
                .iter()
                .map(|flow| flow.to_value_expr())
                .collect(),
        )),
    );

    map.insert(
        Str::from("outputs"),
        ValueExpr::List(ValueExprVec(
            intent
                .outputs
                .iter()
                .map(|flow| flow.to_value_expr())
                .collect(),
        )),
    );

    if let Some(expr_id) = &intent.expression {
        map.insert(
            Str::from("expression"),
            ValueExpr::String(Str::from(expr_id.to_hex())),
        );
    }

    map.insert(
        Str::from("timestamp"),
        ValueExpr::Number(Number::Integer(intent.timestamp.wall.0 as i64)),
    );

    ValueExpr::Record(ValueExprMap(map))
}

//-----------------------------------------------------------------------------
// Handler Utilities
//-----------------------------------------------------------------------------

/// Convert a Handler to ValueExpr for serialization or display purposes
pub fn handler_to_value_expr(handler: &Handler) -> ValueExpr {
    let mut map = BTreeMap::new();
    map.insert(
        Str::from("id"),
        ValueExpr::String(Str::from(handler.id.to_hex())),
    );
    map.insert(
        Str::from("domain_id"),
        ValueExpr::String(Str::from(handler.domain_id.to_hex())),
    );
    map.insert(
        Str::from("name"),
        ValueExpr::String(handler.name),
    );
    map.insert(
        Str::from("handles_type"),
        ValueExpr::String(handler.handles_type),
    );
    if let Some(expr_id) = &handler.expression {
        map.insert(
            Str::from("expression"),
            ValueExpr::String(Str::from(expr_id.to_hex())),
        );
    }
    map.insert(
        Str::from("priority"),
        ValueExpr::Number(Number::Integer(handler.priority as i64)),
    );
    map.insert(
        Str::from("timestamp"),
        ValueExpr::Number(Number::Integer(handler.timestamp.wall.0 as i64)),
    );
    ValueExpr::Record(ValueExprMap(map))
}

//-----------------------------------------------------------------------------
// Resource Utilities
//-----------------------------------------------------------------------------

/// Extension trait for Resource identifiable operations
pub trait ResourceExt {
    /// Get the ID of a Resource
    fn id(&self) -> EntityId;

    /// Compute the hash of a Resource
    fn compute_hash(&self) -> [u8; 32];
}

/// Implement ResourceExt for Resource to provide identifiable operations
impl ResourceExt for Resource {
    fn id(&self) -> EntityId {
        self.id
    }

    fn compute_hash(&self) -> [u8; 32] {
        let bytes = self.as_ssz_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        hasher.finalize().into()
    }
}

/// Convert a Resource to an Identifiable value using the ResourceExt trait
pub fn resource_id(resource: &Resource) -> EntityId {
    ResourceExt::id(resource)
}

/// Computes the SHA256 hash of a resource instance
pub fn compute_resource_hash(instance: &Resource) -> [u8; 32] {
    ResourceExt::compute_hash(instance)
}

/// Creates a new Resource with its content-addressed ID computed
pub fn create_resource(
    domain_id: DomainId,
    resource_type: &str,
    quantity: u64,
    name: &str,
) -> Resource {
    use causality_types::core::time::Timestamp;
    
    let id = EntityId::new(compute_id(format!("{}:{}:{}", domain_id.to_hex(), resource_type, name).as_bytes()));
    let timestamp = Timestamp::for_domain(domain_id);
    
    Resource::new(
        id,
        Str::from(name),
        domain_id,
        Str::from(resource_type),
        quantity,
        timestamp,
    )
}

/// Checks if a ValueExpr is considered a "basic type" for direct inclusion
pub fn is_basic_type_value_expr(value_expr: &ValueExpr) -> bool {
    match value_expr {
        ValueExpr::Unit
        | ValueExpr::Nil
        | ValueExpr::Bool(_)
        | ValueExpr::String(_)
        | ValueExpr::Number(_) => true,
        ValueExpr::List(ValueExprVec(ref items)) => {
            items.iter().all(is_basic_type_value_expr)
        }
        ValueExpr::Map(ValueExprMap(ref map_items))
        | ValueExpr::Record(ValueExprMap(ref map_items)) => {
            map_items.values().all(is_basic_type_value_expr)
        }
        ValueExpr::Ref(_) | ValueExpr::Lambda { .. } => false,
    }
}

/// Creates a Resource from an Intent
pub fn create_resource_from_intent(
    intent: &Intent,
    resource_domain_id: DomainId,
    _is_ephemeral: bool,
    _resource_static_expr_id: Option<ExprId>,
) -> Resource {
    let intent_value_expr = value_expr_from_intent(intent);
    let _value_id = intent_value_expr.id();

    // Define a TypeExpr for "TelIntentResource"
    let mut fields = BTreeMap::new();
    fields.insert(Str::from("id"), TypeExpr::String);
    fields.insert(Str::from("domain"), TypeExpr::String);
    fields.insert(Str::from("creator"), TypeExpr::String);
    fields.insert(Str::from("action"), TypeExpr::String);
    fields.insert(
        Str::from("inputs"),
        TypeExpr::List(Box::new(TypeExpr::String).into()),
    );
    fields.insert(Str::from("parameters"), TypeExpr::Any);
    fields.insert(Str::from("dynamic_expr"), TypeExpr::String);
    fields.insert(
        Str::from("constraints"),
        TypeExpr::List(Box::new(TypeExpr::String).into()),
    );
    fields.insert(
        Str::from("nullifiers"),
        TypeExpr::List(Box::new(TypeExpr::String).into()),
    );
    fields.insert(Str::from("max_compute_units"), TypeExpr::Integer);
    fields.insert(Str::from("max_ledger_writes"), TypeExpr::Integer);
    fields.insert(Str::from("priority"), TypeExpr::Integer);
    fields.insert(Str::from("deadline"), TypeExpr::Integer);

    create_resource(
        resource_domain_id,
        "intent_resource",
        1,
        &format!("intent_{}", intent.id.to_hex()),
    )
}

/// Creates a Resource from an Effect
pub fn create_resource_from_effect(
    effect: &Effect,
    resource_domain_id: DomainId,
    _is_ephemeral: bool,
    _resource_static_expr_id: Option<ExprId>,
) -> Resource {
    let effect_value_expr = value_expr_from_effect(effect);
    let _value_id = effect_value_expr.id();

    // Define a TypeExpr for "TelEffectResource"
    let mut fields = BTreeMap::new();
    fields.insert(Str::from("id"), TypeExpr::String);
    fields.insert(Str::from("domain"), TypeExpr::String);
    fields.insert(Str::from("intent_id"), TypeExpr::String);
    fields.insert(Str::from("effect_type"), TypeExpr::String);
    fields.insert(Str::from("payload"), TypeExpr::Any);
    fields.insert(Str::from("dynamic_expr"), TypeExpr::String);
    fields.insert(
        Str::from("inputs"),
        TypeExpr::List(Box::new(TypeExpr::String).into()),
    );
    fields.insert(
        Str::from("outputs"),
        TypeExpr::List(Box::new(TypeExpr::Record(TypeExprMap(BTreeMap::new()))).into()),
    );
    fields.insert(
        Str::from("constraints"),
        TypeExpr::List(Box::new(TypeExpr::String).into()),
    );
    fields.insert(Str::from("scoped_handler"), TypeExpr::String);

    create_resource(
        resource_domain_id,
        "effect_resource",
        1,
        &format!("effect_{}", effect.id.to_hex()),
    )
}

/// Creates a Resource from a Handler
pub fn create_resource_from_handler(
    handler: &Handler,
    resource_domain_id: DomainId,
    _is_ephemeral: bool,
    _resource_static_expr_id: Option<ExprId>,
) -> Resource {
    let handler_value_expr = handler_to_value_expr(handler);
    let _value_id = handler_value_expr.id();

    // Define a TypeExpr for "TelHandlerResource"
    let mut fields = BTreeMap::new();
    fields.insert(Str::from("id"), TypeExpr::String);
    fields.insert(Str::from("domain"), TypeExpr::String);
    fields.insert(Str::from("effect_type"), TypeExpr::String);
    fields.insert(
        Str::from("constraints"),
        TypeExpr::List(Box::new(TypeExpr::String).into()),
    );
    fields.insert(Str::from("dynamic_expr"), TypeExpr::String);
    fields.insert(Str::from("priority"), TypeExpr::Integer);
    fields.insert(Str::from("cost"), TypeExpr::Integer);
    fields.insert(Str::from("ephemeral"), TypeExpr::Bool);

    create_resource(
        resource_domain_id,
        "handler_resource", 
        1,
        &format!("handler_{}", handler.id.to_hex()),
    )
}

/// Trait for projecting Resources to specific types
pub trait ResourceProjectableExt {
    /// Project a ValueExpr to a specific type
    fn project<T>(&self, value_expr: &ValueExpr) -> Option<T>
    where
        T: TryFrom<ValueExpr>;
}

impl ResourceProjectableExt for Resource {
    fn project<T>(&self, value_expr: &ValueExpr) -> Option<T>
    where
        T: TryFrom<ValueExpr>,
    {
        T::try_from(value_expr.clone()).ok()
    }
}

/// Project a resource to a specific type
pub fn project_resource<T>(resource: &Resource, value_expr: &ValueExpr) -> Option<T>
where
    T: TryFrom<ValueExpr>,
{
    resource.project(value_expr)
}

/// Trait for types that can be converted to bytes
pub trait ToBytes {
    fn to_bytes(&self) -> Option<Vec<u8>>;
}

impl ToBytes for Resource {
    fn to_bytes(&self) -> Option<Vec<u8>> {
        Some(self.as_ssz_bytes())
    }
}
