//! Temporal Effect Language (TEL)
//!
//! Defines the core data structures and types for the Temporal Effect Language,
//! which is used to express temporal effects and their relationships in the
//! Causality framework.

//-----------------------------------------------------------------------------
// Module Exports
//-----------------------------------------------------------------------------

pub mod common_refs;
pub mod cost_model;
pub mod domain_aware_nodes;
pub mod execution_context;
pub mod graph;
pub mod graph_structure;
pub mod graph_types;
pub mod mode;
pub mod optimization;
pub mod process_dataflow;
pub mod strategy;
pub mod traits;

//-----------------------------------------------------------------------------
// Type Re-exports
//-----------------------------------------------------------------------------

pub use common_refs::*;
pub use cost_model::*;
pub use domain_aware_nodes::*;
pub use execution_context::*;
pub use graph::*;
pub use graph_structure::*;
pub use graph_types::*;
pub use mode::*;
pub use optimization::*;
pub use process_dataflow::*;
pub use strategy::*;
pub use traits::*;

// Re-export unified core types for convenience in TEL context
pub use crate::core::{Effect, Intent, Handler, Transaction, Resource};

//-----------------------------------------------------------------------------
// TEL Integration Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tel_integration_tests {
    use super::*;
    use crate::primitive::ids::{EntityId, DomainId, ExprId, AsId};
    use crate::core::time::Timestamp;
    use crate::primitive::string::Str;
    use crate::core::resource_conversion::{ToValueExpr, FromValueExpr, AsResourceData};

    /// Test that unified Effect type integrates correctly with TEL representations
    #[test]
    fn test_unified_effect_tel_integration() {
        // Create a unified Effect
        let effect = Effect {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_transfer"),
            domain_id: DomainId::new([2u8; 32]),
            effect_type: Str::from("transfer"),
            inputs: vec![],
            outputs: vec![],
            expression: Some(ExprId::new([3u8; 32])),
            timestamp: Timestamp::now(),
            resources: vec![],
            nullifiers: vec![],
            scoped_by: crate::primitive::ids::HandlerId::new([4u8; 32]),
            intent_id: Some(ExprId::new([5u8; 32])),
            source_typed_domain: TypedDomain::default(),
            target_typed_domain: TypedDomain::default(),
            cost_model: None,
            resource_usage_estimate: None,
            originating_dataflow_instance: None,
        };

        // Test conversion to ValueExpr (TEL representation)
        let value_expr = effect.to_value_expr();
        
        // Verify TEL representation has expected structure
        if let crate::expr::value::ValueExpr::Record(map) = &value_expr {
            assert!(map.0.contains_key(&Str::from("id")));
            assert!(map.0.contains_key(&Str::from("effect_type")));
            assert!(map.0.contains_key(&Str::from("domain_id")));
            assert!(map.0.contains_key(&Str::from("expression")));
        } else {
            panic!("Expected Record type for TEL Effect representation");
        }

        // Test round-trip conversion
        let converted_back = Effect::from_value_expr(&value_expr)
            .expect("Failed to convert Effect from TEL representation");
        
        assert_eq!(effect.id, converted_back.id);
        assert_eq!(effect.effect_type, converted_back.effect_type);
        assert_eq!(effect.domain_id, converted_back.domain_id);
    }

    /// Test that unified Intent type integrates correctly with TEL representations
    #[test]
    fn test_unified_intent_tel_integration() {
        // Create a unified Intent
        let intent = Intent {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_intent"),
            domain_id: DomainId::new([2u8; 32]),
            priority: 5,
            inputs: vec![],
            outputs: vec![],
            expression: Some(ExprId::new([3u8; 32])),
            timestamp: Timestamp::now(),
            optimization_hint: None,
            compatibility_metadata: vec![],
            resource_preferences: vec![],
            target_typed_domain: None,
            process_dataflow_hint: None,
        };

        // Test conversion to Resource (TEL storage)
        let resource = intent.to_resource(intent.domain_id);
        
        // Verify Resource representation
        assert_eq!(resource.id, intent.id);
        assert_eq!(resource.domain_id, intent.domain_id);
        assert_eq!(resource.resource_type, Str::from("intent"));

        // Test ValueExpr conversion
        let value_expr = intent.to_value_expr();
        let converted_back = Intent::from_value_expr(&value_expr)
            .expect("Failed to convert Intent from TEL representation");
        
        assert_eq!(intent.id, converted_back.id);
        assert_eq!(intent.priority, converted_back.priority);
        assert_eq!(intent.domain_id, converted_back.domain_id);
    }

    /// Test that unified Handler type integrates correctly with TEL representations
    #[test]
    fn test_unified_handler_tel_integration() {
        // Create a unified Handler
        let handler = Handler {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_handler"),
            domain_id: DomainId::new([2u8; 32]),
            handles_type: Str::from("transfer"),
            priority: 10,
            expression: Some(ExprId::new([3u8; 32])),
            timestamp: Timestamp::now(),
        };

        // Test conversion to Resource (TEL storage)
        let resource = handler.to_resource(handler.domain_id);
        
        // Verify Resource representation
        assert_eq!(resource.id, handler.id);
        assert_eq!(resource.domain_id, handler.domain_id);
        assert_eq!(resource.resource_type, Str::from("handler"));

        // Test ValueExpr conversion
        let value_expr = handler.to_value_expr();
        let converted_back = Handler::from_value_expr(&value_expr)
            .expect("Failed to convert Handler from TEL representation");
        
        assert_eq!(handler.id, converted_back.id);
        assert_eq!(handler.handles_type, converted_back.handles_type);
        assert_eq!(handler.priority, converted_back.priority);
    }

    /// Test EffectGraph creation with unified types
    #[test]
    fn test_effect_graph_with_unified_types() {
        // Create unified types
        let intent = Intent {
            id: EntityId::new([1u8; 32]),
            name: Str::from("test_intent"),
            domain_id: DomainId::new([2u8; 32]),
            priority: 5,
            inputs: vec![],
            outputs: vec![],
            expression: Some(ExprId::new([3u8; 32])),
            timestamp: Timestamp::now(),
            optimization_hint: None,
            compatibility_metadata: vec![],
            resource_preferences: vec![],
            target_typed_domain: None,
            process_dataflow_hint: None,
        };

        let effect = Effect {
            id: EntityId::new([4u8; 32]),
            name: Str::from("test_effect"),
            domain_id: DomainId::new([2u8; 32]),
            effect_type: Str::from("computation"),
            inputs: vec![],
            outputs: vec![],
            expression: Some(ExprId::new([5u8; 32])),
            timestamp: Timestamp::now(),
            resources: vec![],
            nullifiers: vec![],
            scoped_by: crate::primitive::ids::HandlerId::new([6u8; 32]),
            intent_id: Some(ExprId::new([3u8; 32])),
            source_typed_domain: TypedDomain::default(),
            target_typed_domain: TypedDomain::default(),
            cost_model: None,
            resource_usage_estimate: None,
            originating_dataflow_instance: None,
        };

        let handler = Handler {
            id: EntityId::new([7u8; 32]),
            name: Str::from("test_handler"),
            domain_id: DomainId::new([2u8; 32]),
            handles_type: Str::from("computation"),
            priority: 10,
            expression: Some(ExprId::new([8u8; 32])),
            timestamp: Timestamp::now(),
        };

        // Create EffectGraph with unified types
        let graph = EffectGraph {
            id: Some(Str::from("test_graph")),
            intents: vec![intent],
            effects: vec![effect],
            handlers: vec![handler],
            edges: vec![],
        };

        // Verify graph structure
        assert_eq!(graph.intents.len(), 1);
        assert_eq!(graph.effects.len(), 1);
        assert_eq!(graph.handlers.len(), 1);
        assert_eq!(graph.intents[0].name, Str::from("test_intent"));
        assert_eq!(graph.effects[0].effect_type, Str::from("computation"));
        assert_eq!(graph.handlers[0].handles_type, Str::from("computation"));
    }

    /// Test ResourceRef with unified types
    #[test]
    fn test_resource_ref_unified_integration() {
        let domain_id = DomainId::new([1u8; 32]);
        let resource_id = crate::primitive::ids::ResourceId::new([2u8; 32]);
        
        // Create ResourceRef
        let resource_ref = ResourceRef::from(resource_id);
        
        // Test that ResourceRef integrates properly with unified Resource type
        let resource = Resource {
            id: EntityId::new(resource_id.inner()),
            name: Str::from("test_resource"),
            domain_id,
            resource_type: Str::from("token"),
            quantity: 100,
            timestamp: Timestamp::now(),
        };

        // Verify compatibility
        assert_eq!(EntityId::new(resource_ref.0.inner()), resource.id);
    }

    /// Test edge creation with unified types
    #[test]
    fn test_edge_creation_with_unified_types() {
        use crate::tel::graph::{Edge, EdgeKind};
        use crate::primitive::ids::{NodeId, EdgeId, HandlerId};

        let source_node = NodeId::new([1u8; 32]);
        let target_node = NodeId::new([2u8; 32]);
        let handler_id = HandlerId::new([3u8; 32]);

        // Create edge with unified types
        let edge = Edge {
            id: EdgeId::new([4u8; 32]),
            source: source_node,
            target: target_node,
            kind: EdgeKind::Applies(handler_id),
            metadata: None,
        };

        // Verify edge structure
        assert_eq!(edge.source, source_node);
        assert_eq!(edge.target, target_node);
        
        match edge.kind {
            EdgeKind::Applies(h_id) => assert_eq!(h_id, handler_id),
            _ => panic!("Expected Applies edge kind"),
        }
    }

    /// Test domain awareness in TEL integration
    #[test]
    fn test_domain_awareness_tel_integration() {
        let domain1 = DomainId::new([1u8; 32]);
        let domain2 = DomainId::new([2u8; 32]);

        // Create effects in different domains
        let effect1 = Effect {
            id: EntityId::new([3u8; 32]),
            name: Str::from("effect_domain1"),
            domain_id: domain1,
            effect_type: Str::from("transfer"),
            inputs: vec![],
            outputs: vec![],
            expression: None,
            timestamp: Timestamp::now(),
            resources: vec![],
            nullifiers: vec![],
            scoped_by: crate::primitive::ids::HandlerId::new([4u8; 32]),
            intent_id: None,
            source_typed_domain: TypedDomain::default(),
            target_typed_domain: TypedDomain::default(),
            cost_model: None,
            resource_usage_estimate: None,
            originating_dataflow_instance: None,
        };

        let effect2 = Effect {
            id: EntityId::new([5u8; 32]),
            name: Str::from("effect_domain2"),
            domain_id: domain2,
            effect_type: Str::from("transfer"),
            inputs: vec![],
            outputs: vec![],
            expression: None,
            timestamp: Timestamp::now(),
            resources: vec![],
            nullifiers: vec![],
            scoped_by: crate::primitive::ids::HandlerId::new([6u8; 32]),
            intent_id: None,
            source_typed_domain: TypedDomain::default(),
            target_typed_domain: TypedDomain::default(),
            cost_model: None,
            resource_usage_estimate: None,
            originating_dataflow_instance: None,
        };

        // Verify domain isolation
        assert_ne!(effect1.domain_id, effect2.domain_id);
        
        // Test that both can be converted to TEL representations
        let value1 = effect1.to_value_expr();
        let value2 = effect2.to_value_expr();
        
        // Both should be valid TEL representations
        assert!(matches!(value1, crate::expr::value::ValueExpr::Record(_)));
        assert!(matches!(value2, crate::expr::value::ValueExpr::Record(_)));
    }
}
