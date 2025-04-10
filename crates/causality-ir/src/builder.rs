// Builder module for the Temporal Effect Graph
// This module provides a fluent API for constructing TEGs in a readable and maintainable way.

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use causality_types::ContentHash;

use crate::{
    EffectNode, ResourceNode, TemporalEffectGraph, TEGFragment,
    EffectId, ResourceId, CapabilityId, DomainId, FactId,
    graph::edge::{Edge, EdgeId, Condition, TemporalRelation, RelationshipType, AccessMode},
    graph::TemporalConstraint,
};

/// Builder for creating a TemporalEffectGraph
pub struct TEGBuilder {
    teg: TemporalEffectGraph,
}

impl TEGBuilder {
    /// Create a new TEG builder
    pub fn new() -> Self {
        Self {
            teg: TemporalEffectGraph::new(),
        }
    }

    /// Add an effect node to the graph
    pub fn effect(&mut self, effect_id: impl Into<EffectId>) -> EffectNodeBuilder {
        EffectNodeBuilder::new(effect_id.into(), self)
    }

    /// Add a resource node to the graph
    pub fn resource(&mut self, resource_id: impl Into<ResourceId>) -> ResourceNodeBuilder {
        ResourceNodeBuilder::new(resource_id.into(), self)
    }

    /// Add a relationship between effects
    pub fn relationship(&mut self) -> RelationshipBuilder {
        RelationshipBuilder::new(self)
    }

    /// Add a fragment to the graph
    pub fn fragment(&mut self, fragment: TEGFragment) -> Result<&mut Self> {
        // Add all effect nodes from the fragment
        for (id, effect) in fragment.effect_nodes {
            self.teg.effect_nodes.insert(id.clone(), effect);
        }

        // Add all resource nodes from the fragment
        for (id, resource) in fragment.resource_nodes {
            self.teg.resource_nodes.insert(id.clone(), resource);
        }

        // Add all edges from the fragment
        for (id, edge) in fragment.edges {
            self.teg.edges.insert(id.clone(), edge);
        }

        // Add all effect dependencies
        for (id, deps) in fragment.effect_dependencies {
            if let Some(existing_deps) = self.teg.effect_dependencies.get_mut(&id) {
                existing_deps.extend(deps);
            } else {
                self.teg.effect_dependencies.insert(id, deps);
            }
        }

        // Add all effect continuations
        for (id, conts) in fragment.effect_continuations {
            if let Some(existing_conts) = self.teg.effect_continuations.get_mut(&id) {
                existing_conts.extend(conts);
            } else {
                self.teg.effect_continuations.insert(id, conts);
            }
        }

        // Add all resource relationships
        for (id, rels) in fragment.resource_relationships {
            if let Some(existing_rels) = self.teg.resource_relationships.get_mut(&id) {
                existing_rels.extend(rels);
            } else {
                self.teg.resource_relationships.insert(id, rels);
            }
        }

        Ok(self)
    }

    /// Add metadata to the graph
    pub fn metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.teg.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the final TemporalEffectGraph
    pub fn build(self) -> TemporalEffectGraph {
        self.teg
    }
}

/// Builder for creating an EffectNode
pub struct EffectNodeBuilder<'a> {
    effect_id: EffectId,
    effect_type: String,
    parameters: HashMap<String, serde_json::Value>,
    required_capabilities: Vec<CapabilityId>,
    resources_accessed: Vec<ResourceId>,
    fact_dependencies: Vec<FactId>,
    domain_id: Option<DomainId>,
    metadata: HashMap<String, String>,
    teg_builder: &'a mut TEGBuilder,
}

impl<'a> EffectNodeBuilder<'a> {
    /// Create a new effect node builder
    fn new(effect_id: EffectId, teg_builder: &'a mut TEGBuilder) -> Self {
        Self {
            effect_id,
            effect_type: String::new(),
            parameters: HashMap::new(),
            required_capabilities: Vec::new(),
            resources_accessed: Vec::new(),
            fact_dependencies: Vec::new(),
            domain_id: None,
            metadata: HashMap::new(),
            teg_builder,
        }
    }

    /// Set the effect type
    pub fn effect_type(mut self, effect_type: impl Into<String>) -> Self {
        self.effect_type = effect_type.into();
        self
    }

    /// Add a parameter to the effect
    pub fn parameter(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Add a required capability
    pub fn require_capability(mut self, capability_id: impl Into<CapabilityId>) -> Self {
        self.required_capabilities.push(capability_id.into());
        self
    }

    /// Add an accessed resource
    pub fn access_resource(mut self, resource_id: impl Into<ResourceId>) -> Self {
        self.resources_accessed.push(resource_id.into());
        self
    }

    /// Add a fact dependency
    pub fn depend_on_fact(mut self, fact_id: impl Into<FactId>) -> Self {
        self.fact_dependencies.push(fact_id.into());
        self
    }

    /// Set the domain ID
    pub fn domain(mut self, domain_id: impl Into<DomainId>) -> Self {
        self.domain_id = Some(domain_id.into());
        self
    }

    /// Add metadata to the effect
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the effect node and add it to the graph
    pub fn add(self) -> &'a mut TEGBuilder {
        let domain_id = self.domain_id.unwrap_or_else(|| "default".to_string());
        
        // Create the effect node
        let effect = EffectNode {
            id: self.effect_id,
            effect_type: self.effect_type,
            parameters: self.parameters,
            required_capabilities: self.required_capabilities,
            resources_accessed: self.resources_accessed,
            fact_dependencies: self.fact_dependencies,
            domain_id,
            metadata: self.metadata,
            content_hash: ContentHash::default(), // Will be computed on finalization
        };
        
        // Add the effect node to the graph
        self.teg_builder.teg.add_effect_node(effect);
        self.teg_builder
    }
}

/// Builder for creating a ResourceNode
pub struct ResourceNodeBuilder<'a> {
    resource_id: ResourceId,
    resource_type: String,
    state: serde_json::Value,
    domain_id: Option<DomainId>,
    metadata: HashMap<String, String>,
    teg_builder: &'a mut TEGBuilder,
}

impl<'a> ResourceNodeBuilder<'a> {
    /// Create a new resource node builder
    fn new(resource_id: ResourceId, teg_builder: &'a mut TEGBuilder) -> Self {
        Self {
            resource_id,
            resource_type: String::new(),
            state: serde_json::Value::Null,
            domain_id: None,
            metadata: HashMap::new(),
            teg_builder,
        }
    }

    /// Set the resource type
    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = resource_type.into();
        self
    }

    /// Set the resource state
    pub fn state(mut self, state: impl Into<serde_json::Value>) -> Self {
        self.state = state.into();
        self
    }

    /// Set the domain ID
    pub fn domain(mut self, domain_id: impl Into<DomainId>) -> Self {
        self.domain_id = Some(domain_id.into());
        self
    }

    /// Add metadata to the resource
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the resource node and add it to the graph
    pub fn add(self) -> &'a mut TEGBuilder {
        let domain_id = self.domain_id.unwrap_or_else(|| "default".to_string());
        
        // Create the resource node
        let resource = ResourceNode {
            id: self.resource_id,
            resource_type: self.resource_type,
            state: self.state,
            domain_id,
            metadata: self.metadata,
            content_hash: ContentHash::default(), // Will be computed on finalization
        };
        
        // Add the resource node to the graph
        self.teg_builder.teg.add_resource_node(resource);
        self.teg_builder
    }
}

/// Builder for creating relationships between nodes
pub struct RelationshipBuilder<'a> {
    teg_builder: &'a mut TEGBuilder,
}

impl<'a> RelationshipBuilder<'a> {
    /// Create a new relationship builder
    fn new(teg_builder: &'a mut TEGBuilder) -> Self {
        Self { teg_builder }
    }

    /// Add an effect dependency relationship
    pub fn effect_depends_on(&mut self, dependent: impl Into<EffectId>, dependency: impl Into<EffectId>) -> &mut Self {
        let dependent_id = dependent.into();
        let dependency_id = dependency.into();
        
        if let Some(dependencies) = self.teg_builder.teg.effect_dependencies.get_mut(&dependent_id) {
            dependencies.push(dependency_id);
        } else {
            self.teg_builder.teg.effect_dependencies.insert(
                dependent_id,
                vec![dependency_id],
            );
        }
        
        self
    }

    /// Add an effect continuation relationship
    pub fn effect_continues_to(
        &mut self,
        source: impl Into<EffectId>,
        target: impl Into<EffectId>,
        condition: Option<Condition>,
    ) -> &mut Self {
        let source_id = source.into();
        let target_id = target.into();
        
        if let Some(continuations) = self.teg_builder.teg.effect_continuations.get_mut(&source_id) {
            continuations.push((target_id, condition));
        } else {
            self.teg_builder.teg.effect_continuations.insert(
                source_id,
                vec![(target_id, condition)],
            );
        }
        
        self
    }

    /// Add a resource relationship
    pub fn resource_relates_to(
        &mut self,
        source: impl Into<ResourceId>,
        target: impl Into<ResourceId>,
        relationship_type: RelationshipType,
    ) -> &mut Self {
        let source_id = source.into();
        let target_id = target.into();
        
        if let Some(relationships) = self.teg_builder.teg.resource_relationships.get_mut(&source_id) {
            relationships.push((target_id, relationship_type));
        } else {
            self.teg_builder.teg.resource_relationships.insert(
                source_id,
                vec![(target_id, relationship_type)],
            );
        }
        
        self
    }

    /// Add a temporal constraint between effects
    pub fn temporal_constraint(
        &mut self,
        source: impl Into<EffectId>,
        target: impl Into<EffectId>,
        relation: TemporalRelation,
    ) -> &mut Self {
        let source_id = source.into();
        let target_id = target.into();
        
        let constraint = TemporalConstraint {
            source: source_id.clone(),
            target: target_id,
            relation,
        };
        
        if let Some(constraints) = self.teg_builder.teg.temporal_constraints.get_mut(&source_id) {
            constraints.push(constraint);
        } else {
            self.teg_builder.teg.temporal_constraints.insert(
                source_id,
                vec![constraint],
            );
        }
        
        self
    }

    /// Add a capability authorization for an effect
    pub fn authorize_capability(
        &mut self,
        effect_id: impl Into<EffectId>,
        capability_id: impl Into<CapabilityId>,
    ) -> &mut Self {
        let effect_id = effect_id.into();
        let capability_id = capability_id.into();
        
        if let Some(authorizations) = self.teg_builder.teg.capability_authorizations.get_mut(&effect_id) {
            authorizations.push(capability_id);
        } else {
            self.teg_builder.teg.capability_authorizations.insert(
                effect_id,
                vec![capability_id],
            );
        }
        
        self
    }

    /// Return to the main builder
    pub fn done(&self) -> &'a mut TEGBuilder {
        self.teg_builder
    }
}

/// Helper for building a complete graph
pub fn build_graph() -> TEGBuilder {
    TEGBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builder_basics() {
        let teg = build_graph()
            .effect("effect1")
                .effect_type("test_effect")
                .parameter("param1", "value1")
                .domain("test_domain")
                .add()
            .resource("resource1")
                .resource_type("test_resource")
                .state(serde_json::json!({"key": "value"}))
                .domain("test_domain")
                .add()
            .build();
        
        assert_eq!(teg.effect_nodes.len(), 1);
        assert_eq!(teg.resource_nodes.len(), 1);
        assert!(teg.effect_nodes.contains_key("effect1"));
        assert!(teg.resource_nodes.contains_key("resource1"));
    }
    
    #[test]
    fn test_relationship_building() {
        let mut builder = build_graph();
        
        // Add effects and resources
        builder.effect("effect1")
            .effect_type("test_effect")
            .domain("test_domain")
            .add();
            
        builder.effect("effect2")
            .effect_type("test_effect")
            .domain("test_domain")
            .add();
            
        builder.resource("resource1")
            .resource_type("test_resource")
            .domain("test_domain")
            .add();
            
        builder.resource("resource2")
            .resource_type("test_resource")
            .domain("test_domain")
            .add();
        
        // Add relationships
        builder.relationship()
            .effect_depends_on("effect2", "effect1")
            .effect_continues_to("effect1", "effect2", Some(Condition::Success))
            .resource_relates_to("resource1", "resource2", RelationshipType::Derivative)
            .temporal_constraint("effect1", "effect2", TemporalRelation::Before)
            .authorize_capability("effect1", "capability1")
            .done();
        
        let teg = builder.build();
        
        // Assert relationships were created
        assert!(teg.effect_dependencies.contains_key("effect2"));
        assert_eq!(teg.effect_dependencies["effect2"].len(), 1);
        assert_eq!(teg.effect_dependencies["effect2"][0], "effect1");
        
        assert!(teg.effect_continuations.contains_key("effect1"));
        assert_eq!(teg.effect_continuations["effect1"].len(), 1);
        assert_eq!(teg.effect_continuations["effect1"][0].0, "effect2");
        
        assert!(teg.resource_relationships.contains_key("resource1"));
        assert_eq!(teg.resource_relationships["resource1"].len(), 1);
        assert_eq!(teg.resource_relationships["resource1"][0].0, "resource2");
        
        assert!(teg.temporal_constraints.contains_key("effect1"));
        assert_eq!(teg.temporal_constraints["effect1"].len(), 1);
        assert_eq!(teg.temporal_constraints["effect1"][0].target, "effect2");
        
        assert!(teg.capability_authorizations.contains_key("effect1"));
        assert_eq!(teg.capability_authorizations["effect1"].len(), 1);
        assert_eq!(teg.capability_authorizations["effect1"][0], "capability1");
    }
    
    #[test]
    fn test_fragment_incorporation() {
        // Create a fragment
        let mut fragment = TEGFragment::new();
        
        // Add an effect node to the fragment
        let effect = EffectNode {
            id: "fragment_effect".to_string(),
            effect_type: "test_effect".to_string(),
            parameters: HashMap::new(),
            required_capabilities: Vec::new(),
            resources_accessed: Vec::new(),
            fact_dependencies: Vec::new(),
            domain_id: "test_domain".to_string(),
            metadata: HashMap::new(),
            content_hash: ContentHash::default(),
        };
        
        fragment.effect_nodes.insert(effect.id.clone(), effect);
        fragment.entry_points.push("fragment_effect".to_string());
        fragment.exit_points.push("fragment_effect".to_string());
        
        // Build a graph with the fragment
        let teg = build_graph()
            .effect("effect1")
                .effect_type("test_effect")
                .domain("test_domain")
                .add()
            .fragment(fragment)
                .unwrap()
            .build();
        
        // Verify that both the direct effect and the fragment effect are in the graph
        assert_eq!(teg.effect_nodes.len(), 2);
        assert!(teg.effect_nodes.contains_key("effect1"));
        assert!(teg.effect_nodes.contains_key("fragment_effect"));
    }
}
