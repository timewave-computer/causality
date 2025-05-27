// Builder module for the Temporal Effect Graph
// This module provides a fluent API for constructing TEGs in a readable and maintainable way.

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use causality_types::{ContentHash, test_content_hash};

use crate::{
    EffectNode, ResourceNode, TemporalEffectGraph, TEGFragment,
    EffectId, ResourceId, CapabilityId, DomainId, FactId,
    graph::edge::{Edge, EdgeId, Condition, TemporalRelation, RelationshipType, AccessMode},
    graph::TemporalConstraint,
    effect_node::ParameterValue,
    resource_node::ResourceState,
    ResourceType,
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
    pub fn build(&mut self) -> Result<TemporalEffectGraph, anyhow::Error> {
        let mut result = TemporalEffectGraph::new();
        std::mem::swap(&mut result, &mut self.teg);
        Ok(result)
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
    metadata: HashMap<String, ParameterValue>,
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
        self.metadata.insert(key.into(), ParameterValue::String(value.into()));
        self
    }

    /// Build the effect node and add it to the graph
    pub fn add(self) -> &'a mut TEGBuilder {
        let domain_id = self.domain_id.unwrap_or_else(|| "default".to_string());
        
        // Need to convert parameters to ParameterValue format
        let mut param_values = HashMap::new();
        for (key, value) in self.parameters {
            if let Ok(param_value) = crate::tel::to_teg::value_to_parameter_value(&value) {
                param_values.insert(key, param_value);
            }
        }
        
        // Create the effect node
        let effect = EffectNode {
            id: self.effect_id,
            effect_type: self.effect_type,
            parameters: param_values,
            required_capabilities: self.required_capabilities,
            resources_accessed: self.resources_accessed,
            fact_dependencies: self.fact_dependencies,
            domain_id,
            metadata: self.metadata,
            content_hash: test_content_hash(),
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
    state: ResourceState,
    domain_id: Option<DomainId>,
    metadata: HashMap<String, ParameterValue>,
    teg_builder: &'a mut TEGBuilder,
}

impl<'a> ResourceNodeBuilder<'a> {
    /// Create a new resource node builder
    fn new(resource_id: ResourceId, teg_builder: &'a mut TEGBuilder) -> Self {
        Self {
            resource_id,
            resource_type: String::new(),
            state: ResourceState::Inactive,
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
    pub fn state(mut self, state: ResourceState) -> Self {
        self.state = state;
        self
    }

    /// Set the domain ID
    pub fn domain(mut self, domain_id: impl Into<DomainId>) -> Self {
        self.domain_id = Some(domain_id.into());
        self
    }

    /// Add metadata to the resource
    pub fn metadata(mut self, key: impl Into<String>, value: ParameterValue) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Build the resource node and add it to the graph
    pub fn add(self) -> &'a mut TEGBuilder {
        let domain_id = self.domain_id.unwrap_or_else(|| "default".to_string());
        
        // Create the resource node
        let resource = ResourceNode::builder()
            .id(self.resource_id.clone())
            .resource_type(self.resource_type)
            .state(self.state)
            .domain(domain_id)
            .build()
            .unwrap();
        
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

    /// Add a temporal constraint
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

    /// Authorize a capability for an effect
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
    pub fn done(&'a mut self) -> &'a mut TEGBuilder {
        self.teg_builder
    }
}

/// Helper for building a complete graph
pub fn build_graph() -> TEGBuilder {
    TEGBuilder::new()
}

/// Build a simple test TEG for demonstration purposes
pub fn build_test_teg() -> TemporalEffectGraph {
    // Create a new builder
    let mut builder = TEGBuilder::new();
    
    // Add some effects
    builder.effect("effect1")
        .effect_type("test_effect")
        .domain("test_domain")
        .add();
        
    builder.effect("effect2")
        .effect_type("test_effect")
        .domain("test_domain")
        .add();
        
    // Add some resources
    builder.resource("resource1")
        .resource_type("test_resource")
        .state(ResourceState::Active)
        .domain("test_domain")
        .add();
        
    // Add relationships
    builder.relationship()
        .effect_depends_on("effect2", "effect1")
        .effect_continues_to("effect1", "effect2", Some(Condition::Success))
        .resource_relates_to("resource1", "resource2", RelationshipType::Reference)
        .temporal_constraint("effect1", "effect2", TemporalRelation::Before)
        .authorize_capability("effect1", "capability1")
        .done();
        
    // Build the TEG
    builder.build().unwrap()
}

// Graph Builder implementation for Temporal Effect Graph
// Provides utility functions for constructing TEG graphs in a builder pattern

/// Builder for constructing Temporal Effect Graphs
#[derive(Debug)]
pub struct GraphBuilder {
    /// Effects to add to the graph
    effects: HashMap<EffectId, EffectNode>,
    
    /// Resources to add to the graph
    resources: HashMap<ResourceId, ResourceNode>,
    
    /// Effect-to-effect edges
    effect_edges: Vec<(EffectId, EffectId, TemporalRelation)>,
    
    /// Effect-to-resource edges
    resource_edges: Vec<(EffectId, ResourceId, AccessMode)>,
}

impl GraphBuilder {
    /// Create a new empty graph builder
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
            resources: HashMap::new(),
            effect_edges: Vec::new(),
            resource_edges: Vec::new(),
        }
    }
    
    /// Add an effect to the graph
    pub fn add_effect(&mut self, id: impl Into<String>, domain: impl Into<String>) -> EffectId {
        let effect_id = id.into();
        let domain_id = domain.into();
        
        let effect = EffectNode::new(
            effect_id.clone(),
            "operation".to_string(), // Default type
            domain_id,
        );
        
        self.effects.insert(effect_id.clone(), effect);
        effect_id
    }
    
    /// Add a resource to the graph
    pub fn add_resource(&mut self, id: impl Into<String>, resource_type: impl Into<String>) -> ResourceId {
        let resource_id = id.into();
        
        // Create a resource node
        let resource = ResourceNode::builder()
            .id(resource_id.clone())
            .resource_type(resource_type.into())
            .state(ResourceState::Active)
            .domain("default".to_string())
            .build()
            .unwrap();
        
        self.resources.insert(resource_id.clone(), resource);
        resource_id
    }
    
    /// Connect two effects with a temporal relation
    pub fn connect_effects(&mut self, from: &String, to: &String) -> &mut Self {
        self.effect_edges.push((
            from.clone(),
            to.clone(),
            TemporalRelation::Before, // Default relation
        ));
        self
    }
    
    /// Add a dependency between two effects
    pub fn add_dependency(&mut self, from: impl Into<String>, to: impl Into<String>) -> &mut Self {
        let from_str = from.into();
        let to_str = to.into();
        self.connect_effects(&from_str, &to_str)
    }
    
    /// Connect an effect to a resource with an access mode
    pub fn connect_effect_to_resource(
        &mut self,
        effect_id: &str,
        resource_id: &str,
        access_mode: AccessMode,
    ) -> &mut Self {
        self.resource_edges.push((
            effect_id.to_string(),
            resource_id.to_string(), 
            access_mode
        ));
        self
    }
    
    /// Build the graph from the collected components
    pub fn build(&self) -> Result<TemporalEffectGraph> {
        let mut teg = TemporalEffectGraph::new();
        
        // Add all effects
        for (id, effect) in &self.effects {
            teg.add_effect(effect.clone())?;
        }
        
        // Add all resources
        for (id, resource) in &self.resources {
            teg.add_resource(resource.clone())?;
        }
        
        // Add effect edges
        for (source, target, relation) in &self.effect_edges {
            teg.connect_effects(source, target, relation.clone(), None)?;
        }
        
        // Add resource edges
        for (effect_id, resource_id, access_mode) in &self.resource_edges {
            teg.connect_effect_to_resource(effect_id, resource_id, access_mode.clone())?;
        }
        
        Ok(teg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::test_content_hash;

    #[test]
    fn test_builder_basics() {
        let mut builder = build_graph();
        
        builder.effect("effect1")
            .effect_type("test_effect")
            .parameter("param1", "value1")
            .domain("test_domain")
            .add();
            
        builder.resource("resource1")
            .resource_type("test_resource")
            .state(ResourceState::Active)
            .domain("test_domain")
            .add();
            
        let teg = builder.build().unwrap();
        
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
            .state(ResourceState::Active)
            .domain("test_domain")
            .add();
            
        builder.resource("resource2")
            .resource_type("test_resource")
            .state(ResourceState::Active)
            .domain("test_domain")
            .add();
        
        // Add relationships
        {
            let mut rel = builder.relationship();
            rel.effect_depends_on("effect2", "effect1");
            rel.effect_continues_to("effect1", "effect2", Some(Condition::Success));
            rel.resource_relates_to("resource1", "resource2", RelationshipType::Reference);
            rel.temporal_constraint("effect1", "effect2", TemporalRelation::Before);
            rel.authorize_capability("effect1", "capability1");
            rel.done();
        }
        
        let teg = builder.build().unwrap();
        
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
        let effect = crate::effect_node::EffectNode {
            id: "fragment_effect".to_string(),
            effect_type: "test_effect".to_string(),
            parameters: HashMap::new(),
            required_capabilities: Vec::new(),
            resources_accessed: Vec::new(),
            fact_dependencies: Vec::new(),
            domain_id: "test_domain".to_string(),
            metadata: HashMap::new(),
            content_hash: test_content_hash(),
        };
        
        fragment.effect_nodes.insert(effect.id.clone(), effect);
        fragment.entry_points.push("fragment_effect".to_string());
        fragment.exit_points.push("fragment_effect".to_string());
        
        // Build a graph with the fragment
        let mut builder = build_graph();
        
        builder.effect("effect1")
            .effect_type("test_effect")
            .domain("test_domain")
            .add();
            
        builder.fragment(fragment).unwrap();
        
        let teg = builder.build().unwrap();
        
        // Verify that both the direct effect and the fragment effect are in the graph
        assert_eq!(teg.effect_nodes.len(), 2);
        assert!(teg.effect_nodes.contains_key("effect1"));
        assert!(teg.effect_nodes.contains_key("fragment_effect"));
    }

    #[test]
    fn test_builders_use_test_content_hash() {
        let mut builder = TEGBuilder::new();
        
        // Add an effect node
        let effect_id = "test-effect-id".to_string();
        let effect_type = "test-effect-type".to_string();
        let domain_id = "test-domain-id".to_string();
        
        builder.effect(effect_id.clone())
            .effect_type(effect_type.clone())
            .domain(domain_id.clone())
            .add();
        
        // Add a resource node
        let resource_id = "test-resource-id".to_string();
        let resource_type = "test-resource-type".to_string();
        
        builder.resource(resource_id.clone())
            .resource_type(resource_type.clone())
            .domain(domain_id.clone())
            .add();
        
        // Build the graph
        let teg = builder.build().unwrap();
        
        // Verify effect node content hash
        let expected_hash = test_content_hash();
        let effect_node = teg.effect_nodes.get(&effect_id).unwrap();
        assert_eq!(effect_node.content_hash.algorithm, expected_hash.algorithm);
        assert_eq!(effect_node.content_hash.bytes, expected_hash.bytes);
        
        // Verify resource node content hash
        let resource_node = teg.resource_nodes.get(&resource_id).unwrap();
        assert_eq!(resource_node.content_hash.algorithm, expected_hash.algorithm);
        assert_eq!(resource_node.content_hash.bytes, expected_hash.bytes);
    }

    #[test]
    fn test_graph_builder_basic() {
        let mut builder = GraphBuilder::new();
        
        let effect1 = builder.add_effect("effect1", "test");
        let effect2 = builder.add_effect("effect2", "test");
        let resource = builder.add_resource("resource1", "test_type");
        
        builder.connect_effects(&effect1, &effect2);
        builder.connect_effect_to_resource(&effect2, &resource, AccessMode::Read);
        
        let teg = builder.build().unwrap();
        
        // Check that the TEG contains the expected nodes and relationships
        assert!(teg.effect_nodes.contains_key(&effect1));
        assert!(teg.effect_nodes.contains_key(&effect2));
        assert!(teg.resource_nodes.contains_key(&resource));
        assert!(teg.effect_dependencies.contains_key(&effect2));
    }

    #[test]
    fn test_connect_effect_to_resource() {
        let mut builder = GraphBuilder::new();
        
        // Add an effect and a resource
        let effect_id = builder.add_effect("effect1", "test_domain");
        let resource_id = builder.add_resource("resource1", "test_resource");
        
        // Connect the effect to the resource
        builder.connect_effect_to_resource(&effect_id, &resource_id, AccessMode::Read);
        
        // Build the graph
        let teg = builder.build().unwrap();
        
        // Verify the connection exists
        let resource_edges = teg.resource_edges.iter()
            .filter(|(effect_id, resource_id, _)| 
                *effect_id == "effect1" && *resource_id == "resource1")
            .collect::<Vec<_>>();
            
        assert_eq!(resource_edges.len(), 1, "Should have exactly one edge connecting effect1 to resource1");
        assert_eq!(resource_edges[0].2, AccessMode::Read, "The access mode should be Read");
    }
}
